> **V2 AMENDMENT (2026-08-20):** The product vision has been restated. AIOS is *"a Linux system for ethical hacking on the inside, a Windows-style desktop on the outside, with AI as a first-class S-rank kernel subsystem."* Three pillars: Pillar A (Linux ethical-hacking), Pillar B (Windows-like desktop), Pillar C (S-rank AI subsystem). This file informs one of the three Pillars: its content is preserved as substrate. See `README.md` and `mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` (v2.0).

# GUI Pointer Research and Design Notes

## Scope

GUI-0002 uses a software cursor in the 800x600 Bochs framebuffer and receives host input through QEMU's VirtIO-MMIO keyboard and relative mouse devices.

## Research conclusions

### Linux input semantics

Linux input devices report `struct input_event`-compatible records. Relative movement is delivered as `EV_REL` with `REL_X` and `REL_Y`; a `SYN_REPORT` (`EV_SYN`, code 0) commits one logical input frame. Relative values are signed 32-bit quantities. The GUI therefore accumulates deltas until synchronization, then applies one pointer update.

Sources:

- Linux input documentation: https://www.kernel.org/doc/html/latest/input/input.html
- Virtio 1.2 input device specification: https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html

### Desktop pointer behavior

Windows distinguishes processed client mouse messages from raw relative input. Raw input preserves physical deltas; ordinary desktop pointers apply a user-facing acceleration/ballistics policy. Linux libinput similarly separates normalized motion from adaptive or flat acceleration profiles.

The kernel GUI uses a deterministic integer adaptive curve: small movements remain 1:1, medium movement receives a 1.5x multiplier, and fast movement receives a 2x multiplier. It is intentionally a first-kernel profile, not a claim of compatibility with Windows or libinput's exact curves. The transform is applied per `SYN_REPORT` frame so queue depth cannot change sensitivity.

Sources:

- Windows `RAWMOUSE`: https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawmouse
- Windows mouse input overview: https://learn.microsoft.com/en-us/windows/win32/inputdev/about-mouse-input
- libinput relative-motion normalization: https://wayland.freedesktop.org/libinput/doc/latest/normalization-of-relative-motion.html
- libinput pointer acceleration: https://wayland.freedesktop.org/libinput/doc/latest/pointer-acceleration.html

### Capture semantics

A relative VirtIO mouse needs host pointer capture. QEMU's GTK backend supports `grab-on-hover=on`; `Ctrl+Alt+G` toggles capture manually and `Ctrl+Alt` releases it. The launch profile keeps `virtio-mouse-device` as required and hides the host cursor so the guest software pointer is unambiguous.

Source:

- QEMU GTK display options and input behavior: https://www.qemu.org/docs/master/system/invocation.html

## Implementation decisions

- Keep `virtio-keyboard-device` and `virtio-mouse-device` simultaneously enabled.
- Keep separate VirtIO-MMIO queues and probes.
- Decode signed `REL_X`/`REL_Y`, left-button events, and `SYN_REPORT`.
- Batch movement by synchronization frame before rendering.
- Apply acceleration once per complete frame, then coalesce transformed frames.
- Preserve incomplete frames across polling iterations rather than applying them opportunistically.
- Use a high-contrast arrow with a `(0,0)` hotspot.
- Capture the framebuffer underlay around the cursor and restore it before moving the cursor; use a two-pass outline/interior sprite renderer.
- Clamp the hotspot so the complete 16x16 pointer remains inside the framebuffer.
- Keep arrow-key movement as a fallback.
- Use a full scene redraw only when status changes, such as button press/release; pointer-only motion updates the saved underlay region.

## Validation

The GUI target builds successfully. Fresh QEMU validation confirmed device detection, queue readiness, relative-event routing, left-button routing, keyboard fallback events, and no panic. Visual acceptance remains: focus the QEMU window, move the host mouse, click, test the edges, and use `Ctrl+Alt+G` if the host platform does not honor hover capture.
