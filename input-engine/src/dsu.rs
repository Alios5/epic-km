//! Cemuhook/DSU motion server (UDP port 26760).
//!
//! Emulators (Ryujinx, Cemu, yuzu forks, Dolphin…) can take motion input
//! over this protocol instead of reading it from the HID controller: the
//! values travel as plain little-endian floats — no HID report, no
//! calibration blob, no assumed bias — so "mouse still" reaches the game
//! as exactly 0 °/s. Enabling it bypasses the whole class of rest-drift
//! issues seen on the ViGEmBus → SDL path.
//!
//! Protocol reference: <https://v1993.github.io/cemuhook-protocol/>
//! Gyroscope values are in deg/s, accelerometer in g's. The rest gravity
//! mirrors the values pad-motion proved against Ryujinx: (0, 9.81, 0).

use crate::engine::{elog, EngineState};
use crate::mapping::GamepadState;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// UDP port every DSU client expects the server on.
const DSU_PORT: u16 = 26760;
/// Only protocol version ever published.
const PROTOCOL_VERSION: u16 = 1001;
/// Arbitrary server ID, stable across packets.
const SERVER_ID: u32 = 0x4550_4D31; // "EPM1"
/// Fixed locally-administered MAC so clients recognise the pad between runs.
const SERVER_MAC: [u8; 6] = [0x02, 0x45, 0x50, 0x49, 0x43, 0x01]; // "EPIC"

const MSG_VERSION: u32 = 0x0010_0000;
const MSG_PORTS: u32 = 0x0010_0001;
const MSG_DATA: u32 = 0x0010_0002;

/// Motion streaming rate to subscribed clients.
const SEND_INTERVAL: Duration = Duration::from_millis(4); // 250 Hz
/// A client that stops re-requesting data for this long is dropped.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);

/// DS4 gyro raw unit (what `GamepadState` carries): 16 LSB per °/s.
const DS4_GYRO_LSB_PER_DPS: f32 = 16.0;

struct Subscriber {
    last_seen: Instant,
    packet_number: u32,
}

/// DSU server thread entry point. Runs for the whole engine lifetime but
/// only occupies the UDP port while the profile enables the feature, so an
/// external DSU server (e.g. pad-motion) keeps working when it's off.
pub fn dsu_thread(state: Arc<EngineState>) {
    let mut warned_busy = false;
    while state.running.load(Ordering::SeqCst) {
        if !state.profile.lock().dsu_enabled {
            warned_busy = false;
            std::thread::sleep(Duration::from_millis(200));
            continue;
        }
        match UdpSocket::bind(("0.0.0.0", DSU_PORT)) {
            Ok(socket) => {
                elog(&state, "DSU motion server listening on UDP port 26760");
                serve(&state, &socket);
            }
            Err(e) => {
                if !warned_busy {
                    elog(
                        &state,
                        &format!(
                            "DSU server: UDP port 26760 unavailable ({}) — another DSU server (pad-motion?) is running. Retrying…",
                            e
                        ),
                    );
                    warned_busy = true;
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }
}

/// Serves DSU requests until the engine stops or the feature is disabled.
fn serve(state: &Arc<EngineState>, socket: &UdpSocket) {
    let _ = socket.set_read_timeout(Some(Duration::from_millis(1)));
    let mut subscribers: HashMap<SocketAddr, Subscriber> = HashMap::new();
    let mut next_send = Instant::now();
    let mut announced = false;
    let mut buf = [0u8; 512];

    while state.running.load(Ordering::SeqCst) && state.profile.lock().dsu_enabled {
        // Drain every pending request.
        loop {
            match socket.recv_from(&mut buf) {
                Ok((n, from)) => {
                    handle_packet(state, socket, &buf[..n], from, &mut subscribers, &mut announced)
                }
                // Read timeout (or transient error): go on to the send step.
                Err(_) => break,
            }
        }

        // Stream controller data to every live subscriber.
        let now = Instant::now();
        if now >= next_send {
            next_send = now + SEND_INTERVAL;
            subscribers.retain(|_, s| s.last_seen.elapsed() < CLIENT_TIMEOUT);
            if !subscribers.is_empty() {
                let gamepad = *state.gamepad.lock();
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_micros() as u64)
                    .unwrap_or(0);
                for (addr, sub) in subscribers.iter_mut() {
                    sub.packet_number = sub.packet_number.wrapping_add(1);
                    let packet = build_data_packet(&gamepad, sub.packet_number, timestamp);
                    let _ = socket.send_to(&packet, addr);
                }
            }
        }
    }
}

/// Handles one incoming DSUC packet (version query, port listing, or a
/// controller-data subscription request).
fn handle_packet(
    state: &Arc<EngineState>,
    socket: &UdpSocket,
    data: &[u8],
    from: SocketAddr,
    subscribers: &mut HashMap<SocketAddr, Subscriber>,
    announced: &mut bool,
) {
    if data.len() < 20 || &data[0..4] != b"DSUC" {
        return;
    }
    let msg_type = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
    let payload = &data[20..];
    match msg_type {
        MSG_VERSION => {
            // Reply with the maximal protocol version we support.
            let _ = socket.send_to(&wrap_message(MSG_VERSION, &PROTOCOL_VERSION.to_le_bytes()), from);
        }
        MSG_PORTS => {
            if payload.len() < 4 {
                return;
            }
            let count =
                i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]).clamp(0, 4)
                    as usize;
            for i in 0..count {
                let Some(&slot) = payload.get(4 + i) else { break };
                if slot > 3 {
                    continue;
                }
                // 12 bytes: controller identification + terminating zero.
                let mut info = [0u8; 12];
                info[0] = slot;
                if slot == 0 {
                    info[1] = 2; // slot state: connected
                    info[2] = 2; // device model: full gyro
                    info[3] = 1; // connection type: USB
                    info[4..10].copy_from_slice(&SERVER_MAC);
                    info[10] = 0x05; // battery: full
                }
                let _ = socket.send_to(&wrap_message(MSG_PORTS, &info), from);
            }
        }
        MSG_DATA => {
            if payload.len() < 8 {
                return;
            }
            // Registration can target all controllers (0), a slot (1), or a
            // MAC address (2); we serve exactly one controller: slot 0.
            let targeted_at_us = match payload[0] {
                0 => true,
                1 => payload[1] == 0,
                2 => payload[2..8] == SERVER_MAC,
                _ => false,
            };
            if targeted_at_us {
                let sub = subscribers.entry(from).or_insert(Subscriber {
                    last_seen: Instant::now(),
                    packet_number: 0,
                });
                sub.last_seen = Instant::now();
                if !*announced {
                    elog(state, &format!("DSU client connected ({}) — streaming motion", from));
                    *announced = true;
                }
            }
        }
        _ => {}
    }
}

/// Builds the 80-byte "actual controller data" payload. Mirrors the virtual
/// gamepad (buttons/sticks included, so full-DSU clients like Dolphin could
/// use it as-is); the gyroscope is converted from DS4 raw units to °/s with
/// no rest-offset trim — the DSU path has no calibration blob to compensate.
fn build_data_packet(gamepad: &GamepadState, packet_number: u32, timestamp_us: u64) -> Vec<u8> {
    let b = &gamepad.buttons;
    let buttons_1 = (b.dpad_left as u8) << 7
        | (b.dpad_down as u8) << 6
        | (b.dpad_right as u8) << 5
        | (b.dpad_up as u8) << 4
        | (b.start as u8) << 3
        | (b.right_thumb as u8) << 2
        | (b.left_thumb as u8) << 1
        | b.back as u8;
    // Face buttons use Nintendo naming on the wire: Y=west, B=south,
    // A=east, X=north (i.e. DS4 Square/Cross/Circle/Triangle).
    let buttons_2 = (b.x as u8) << 7
        | (b.a as u8) << 6
        | (b.b as u8) << 5
        | (b.y as u8) << 4
        | (b.right_shoulder as u8) << 3
        | (b.left_shoulder as u8) << 2
        | (b.right_trigger as u8) << 1
        | b.left_trigger as u8;

    let stick = |v: i16| ((v as i32 + 32768) >> 8) as u8; // 128 = neutral
    let analog = |pressed: bool| if pressed { 255u8 } else { 0 };

    let mut payload = Vec::with_capacity(80);
    // Controller identification (slot 0, connected, full gyro, USB).
    payload.push(0);
    payload.push(2);
    payload.push(2);
    payload.push(1);
    payload.extend_from_slice(&SERVER_MAC);
    payload.push(0x05); // battery: full
    payload.push(1); // controller connected
    payload.extend_from_slice(&packet_number.to_le_bytes());
    payload.push(buttons_1);
    payload.push(buttons_2);
    payload.push(0); // PS button
    payload.push(0); // touch button
    payload.push(stick(gamepad.left_stick_x));
    payload.push(stick(gamepad.left_stick_y));
    payload.push(stick(gamepad.right_stick_x));
    payload.push(stick(gamepad.right_stick_y));
    payload.push(analog(b.dpad_left));
    payload.push(analog(b.dpad_down));
    payload.push(analog(b.dpad_right));
    payload.push(analog(b.dpad_up));
    payload.push(analog(b.x));
    payload.push(analog(b.a));
    payload.push(analog(b.b));
    payload.push(analog(b.y));
    payload.push(analog(b.right_shoulder));
    payload.push(analog(b.left_shoulder));
    payload.push(analog(b.right_trigger));
    payload.push(analog(b.left_trigger));
    payload.extend_from_slice(&[0u8; 12]); // two inactive touch points
    payload.extend_from_slice(&timestamp_us.to_le_bytes());
    // Rest gravity + gyro rates (deg/s). Signs match pad-motion's proven
    // mapping: mouse right → +yaw, mouse up → +pitch.
    for v in [0.0f32, 9.81, 0.0] {
        payload.extend_from_slice(&v.to_le_bytes());
    }
    let gyro_pitch = gamepad.gyro_pitch as f32 / DS4_GYRO_LSB_PER_DPS;
    let gyro_yaw = gamepad.gyro_yaw as f32 / DS4_GYRO_LSB_PER_DPS;
    for v in [gyro_pitch, gyro_yaw, 0.0] {
        payload.extend_from_slice(&v.to_le_bytes());
    }
    wrap_message(MSG_DATA, &payload)
}

/// Builds a full DSUS packet: 16-byte header, message type, payload.
/// The CRC32 covers the whole packet with the checksum field zeroed.
fn wrap_message(msg_type: u32, payload: &[u8]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(20 + payload.len());
    packet.extend_from_slice(b"DSUS");
    packet.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
    packet.extend_from_slice(&(4 + payload.len() as u16).to_le_bytes());
    packet.extend_from_slice(&0u32.to_le_bytes()); // CRC placeholder
    packet.extend_from_slice(&SERVER_ID.to_le_bytes());
    packet.extend_from_slice(&msg_type.to_le_bytes());
    packet.extend_from_slice(payload);
    let crc = crc32(&packet);
    packet[8..12].copy_from_slice(&crc.to_le_bytes());
    packet
}

/// CRC-32 (IEEE 802.3), as required by the protocol header.
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}
