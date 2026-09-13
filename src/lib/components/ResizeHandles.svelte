<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  // Frameless windows have no native resize borders on Linux (unlike
  // Windows, where the OS still provides them). These invisible edge
  // strips forward mousedown to the platform resize-drag implementation.
  const appWindow = getCurrentWindow();

  let maximized = $state(false);

  onMount(() => {
    let unlisten: (() => void) | undefined;
    appWindow.isMaximized().then((m) => (maximized = m));
    appWindow
      .onResized(async () => {
        maximized = await appWindow.isMaximized();
      })
      .then((fn) => (unlisten = fn));
    return () => unlisten?.();
  });

  // Matches the non-exported ResizeDirection union in @tauri-apps/api.
  type ResizeDirection =
    | "East"
    | "North"
    | "NorthEast"
    | "NorthWest"
    | "South"
    | "SouthEast"
    | "SouthWest"
    | "West";

  function startResize(direction: ResizeDirection) {
    return (e: MouseEvent) => {
      if (e.button !== 0 || maximized) return;
      appWindow.startResizeDragging(direction);
    };
  }

  const EDGE = 6; // px grab width along edges
  const CORNER = 12; // px grab size at corners
</script>

{#if !maximized}
  <!-- Edges -->
  <div
    class="fixed top-0 left-0 right-0 z-[60] cursor-n-resize"
    style:height="{EDGE}px"
    onmousedown={startResize("North")}
    role="presentation"
  ></div>
  <div
    class="fixed bottom-0 left-0 right-0 z-[60] cursor-s-resize"
    style:height="{EDGE}px"
    onmousedown={startResize("South")}
    role="presentation"
  ></div>
  <div
    class="fixed top-0 bottom-0 left-0 z-[60] cursor-w-resize"
    style:width="{EDGE}px"
    onmousedown={startResize("West")}
    role="presentation"
  ></div>
  <div
    class="fixed top-0 bottom-0 right-0 z-[60] cursor-e-resize"
    style:width="{EDGE}px"
    onmousedown={startResize("East")}
    role="presentation"
  ></div>

  <!-- Corners (rendered after edges so they take precedence) -->
  <div
    class="fixed top-0 left-0 z-[61] cursor-nw-resize"
    style:width="{CORNER}px"
    style:height="{CORNER}px"
    onmousedown={startResize("NorthWest")}
    role="presentation"
  ></div>
  <div
    class="fixed top-0 right-0 z-[61] cursor-ne-resize"
    style:width="{CORNER}px"
    style:height="{CORNER}px"
    onmousedown={startResize("NorthEast")}
    role="presentation"
  ></div>
  <div
    class="fixed bottom-0 left-0 z-[61] cursor-sw-resize"
    style:width="{CORNER}px"
    style:height="{CORNER}px"
    onmousedown={startResize("SouthWest")}
    role="presentation"
  ></div>
  <div
    class="fixed bottom-0 right-0 z-[61] cursor-se-resize"
    style:width="{CORNER}px"
    style:height="{CORNER}px"
    onmousedown={startResize("SouthEast")}
    role="presentation"
  ></div>
{/if}
