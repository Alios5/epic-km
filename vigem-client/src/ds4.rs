use std::{fmt, mem, ptr};
use std::borrow::Borrow;
use crate::*;

/// DualShock4 HID Input report.
#[cfg(feature = "unstable_ds4")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct DS4Report {
	pub thumb_lx: u8,
	pub thumb_ly: u8,
	pub thumb_rx: u8,
	pub thumb_ry: u8,
	pub buttons: u16,
	pub special: u8,
	pub trigger_l: u8,
	pub trigger_r: u8,
}
#[cfg(feature = "unstable_ds4")]
impl Default for DS4Report {
	#[inline]
	fn default() -> Self {
		DS4Report {
			thumb_lx: 0x80,
			thumb_ly: 0x80,
			thumb_rx: 0x80,
			thumb_ry: 0x80,
			buttons: 0x8,
			special: 0,
			trigger_l: 0,
			trigger_r: 0,
		}
	}
}

/// DualShock 4 touchpad event (10 bytes on the wire, packed).
#[cfg(feature = "unstable_ds4")]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct DS4Touch {
	pub packet_counter: u8,
	pub is_up_tracking_num1: u8,
	pub touch_data1: [u8; 3],
	pub is_up_tracking_num2: u8,
	pub touch_data2: [u8; 3],
}
#[cfg(feature = "unstable_ds4")]
impl DS4Touch {
	fn write_to(&self, out: &mut [u8]) {
		out[0] = self.packet_counter;
		out[1] = self.is_up_tracking_num1;
		out[2..5].copy_from_slice(&self.touch_data1);
		out[5] = self.is_up_tracking_num2;
		out[6..9].copy_from_slice(&self.touch_data2);
		out[9] = 0;
	}
}

/// DualShock4 v1 complete HID Input report (63 bytes on the wire, packed).
///
/// Mirrors the C SDK's `DS4_REPORT_EX` 1:1. The wire layout is `pack(1)`,
/// so instead of a repr(C) struct we serialize explicitly via `write_to`
/// (little-endian) — no unaligned packed-field access involved.
#[cfg(feature = "unstable_ds4")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct DS4ReportEx {
	pub thumb_lx: u8,
	pub thumb_ly: u8,
	pub thumb_rx: u8,
	pub thumb_ry: u8,
	pub buttons: u16,
	pub special: u8,
	pub trigger_l: u8,
	pub trigger_r: u8,
	pub timestamp: u16,
	pub battery_lvl: u8,
	pub gyro_x: i16,
	pub gyro_y: i16,
	pub gyro_z: i16,
	pub accel_x: i16,
	pub accel_y: i16,
	pub accel_z: i16,
	pub unknown1: [u8; 5],
	pub battery_lvl_special: u8,
	pub unknown2: [u8; 2],
	pub touch_packets_n: u8, // 0x00 to 0x03 (USB max)
	pub current_touch: DS4Touch,
	pub previous_touch: [DS4Touch; 2],
}
#[cfg(feature = "unstable_ds4")]
impl Default for DS4ReportEx {
	#[inline]
	fn default() -> Self {
		DS4ReportEx {
			thumb_lx: 0x80,
			thumb_ly: 0x80,
			thumb_rx: 0x80,
			thumb_ry: 0x80,
			buttons: 0x8, // D-Pad neutral (HAT = NONE)
			special: 0,
			trigger_l: 0,
			trigger_r: 0,
			timestamp: 0,
			battery_lvl: 0,
			gyro_x: 0,
			gyro_y: 0,
			gyro_z: 0,
			accel_x: 0,
			accel_y: 0,
			accel_z: 0,
			unknown1: [0; 5],
			battery_lvl_special: 0,
			unknown2: [0; 2],
			touch_packets_n: 0,
			current_touch: DS4Touch::default(),
			previous_touch: [DS4Touch::default(); 2],
		}
	}
}
#[cfg(feature = "unstable_ds4")]
impl DS4ReportEx {
	/// Serializes into the 63-byte wire representation expected by the driver.
	pub fn write_to(&self, out: &mut [u8]) {
		debug_assert!(out.len() >= 63);
		out[0] = self.thumb_lx;
		out[1] = self.thumb_ly;
		out[2] = self.thumb_rx;
		out[3] = self.thumb_ry;
		out[4..6].copy_from_slice(&self.buttons.to_le_bytes());
		out[6] = self.special;
		out[7] = self.trigger_l;
		out[8] = self.trigger_r;
		out[9..11].copy_from_slice(&self.timestamp.to_le_bytes());
		out[11] = self.battery_lvl;
		out[12..14].copy_from_slice(&self.gyro_x.to_le_bytes());
		out[14..16].copy_from_slice(&self.gyro_y.to_le_bytes());
		out[16..18].copy_from_slice(&self.gyro_z.to_le_bytes());
		out[18..20].copy_from_slice(&self.accel_x.to_le_bytes());
		out[20..22].copy_from_slice(&self.accel_y.to_le_bytes());
		out[22..24].copy_from_slice(&self.accel_z.to_le_bytes());
		out[24..29].copy_from_slice(&self.unknown1);
		out[29] = self.battery_lvl_special;
		out[30..32].copy_from_slice(&self.unknown2);
		out[32] = self.touch_packets_n;
		self.current_touch.write_to(&mut out[33..43]);
		self.previous_touch[0].write_to(&mut out[43..53]);
		self.previous_touch[1].write_to(&mut out[53..63]);
	}
}

/// A virtual Sony DualShock 4 (wired).
pub struct DualShock4Wired<CL: Borrow<Client>> {
	client: CL,
	event: Event,
	serial_no: u32,
	id: TargetId,
}

impl<CL: Borrow<Client>> DualShock4Wired<CL> {
	/// Creates a new instance.
	#[inline]
	pub fn new(client: CL, id: TargetId) -> DualShock4Wired<CL> {
		let event = Event::new(false, false);
		DualShock4Wired { client, event, serial_no: 0, id }
	}

	/// Returns if the controller is plugged in.
	#[inline]
	pub fn is_attached(&self) -> bool {
		self.serial_no != 0
	}

	/// Returns the id the controller was constructed with.
	#[inline]
	pub fn id(&self) -> TargetId {
		self.id
	}

	/// Returns the client.
	#[inline]
	pub fn client(&self) -> &CL {
		&self.client
	}

	/// Unplugs and destroys the controller, returning the client.
	#[inline]
	pub fn drop(mut self) -> CL {
		let _ = self.unplug();

		unsafe {
			let client = (&self.client as *const CL).read();
			ptr::drop_in_place(&mut self.event);
			mem::forget(self);
			client
		}
	}

	/// Plugs the controller in.
	#[inline(never)]
	pub fn plugin(&mut self) -> Result<(), Error> {
		if self.is_attached() {
			return Err(Error::AlreadyConnected);
		}

		self.serial_no = unsafe {
			let mut plugin = bus::PluginTarget::ds4_wired(1, self.id.vendor, self.id.product);
			let device = self.client.borrow().device;

			// Yes this is how the driver is implemented
			while plugin.ioctl(device, self.event.handle).is_err() {
				plugin.SerialNo += 1;
				if plugin.SerialNo >= u16::MAX as u32 {
					return Err(Error::NoFreeSlot);
				}
			}

			plugin.SerialNo
		};

		Ok(())
	}

	/// Unplugs the controller.
	#[inline(never)]
	pub fn unplug(&mut self) -> Result<(), Error> {
		if !self.is_attached() {
			return Err(Error::NotPluggedIn);
		}

		unsafe {
			let mut unplug = bus::UnplugTarget::new(self.serial_no);
			let device = self.client.borrow().device;
			unplug.ioctl(device, self.event.handle)?;
		}

		self.serial_no = 0;
		Ok(())
	}

	/// Waits until the virtual controller is ready.
	///
	/// Any updates submitted before the virtual controller is ready may return an error.
	#[inline(never)]
	pub fn wait_ready(&mut self) -> Result<(), Error> {
		if !self.is_attached() {
			return Err(Error::NotPluggedIn);
		}

		unsafe {
			let mut wait = bus::WaitDeviceReady::new(self.serial_no);
			let device = self.client.borrow().device;
			wait.ioctl(device, self.event.handle)?;
		}

		Ok(())
	}

	/// Updates the virtual controller state.
	#[cfg(feature = "unstable_ds4")]
	#[inline(never)]
	pub fn update(&mut self, report: &DS4Report) -> Result<(), Error> {
		if !self.is_attached() {
			return Err(Error::NotPluggedIn);
		}

		unsafe {
			let mut dsr = bus::DS4SubmitReport::new(self.serial_no, *report);
			let device = self.client.borrow().device;
			dsr.ioctl(device, self.event.handle)?;
		}

		Ok(())
	}

	/// Updates the virtual controller state with the full extended report
	/// (gyroscope, accelerometer, touchpad...). The driver accepts it through
	/// the same IOCTL as the basic report, differing only by size.
	#[cfg(feature = "unstable_ds4")]
	#[inline(never)]
	pub fn update_ex(&mut self, report: &DS4ReportEx) -> Result<(), Error> {
		if !self.is_attached() {
			return Err(Error::NotPluggedIn);
		}

		unsafe {
			let mut dsr = bus::DS4SubmitReportEx::new(self.serial_no, report);
			let device = self.client.borrow().device;
			dsr.ioctl(device, self.event.handle)?;
		}

		Ok(())
	}
}

impl<CL: Borrow<Client>> fmt::Debug for DualShock4Wired<CL> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("DualShock4Wired")
			.field("serial_no", &self.serial_no)
			.field("vendor_id", &self.id.vendor)
			.field("product_id", &self.id.product)
			.finish()
	}
}

impl<CL: Borrow<Client>> Drop for DualShock4Wired<CL> {
	#[inline]
	fn drop(&mut self) {
		let _ = self.unplug();
	}
}
