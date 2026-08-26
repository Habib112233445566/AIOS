> **V2 AMENDMENT (2026-08-20):** The product vision has been restated. AIOS is *"a Linux system for ethical hacking on the inside, a Windows-style desktop on the outside, with AI as a first-class S-rank kernel subsystem."* Three pillars: Pillar A (Linux ethical-hacking), Pillar B (Windows-like desktop), Pillar C (S-rank AI subsystem). This file informs one of the three Pillars: its content is preserved as substrate. See `README.md` and `mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` (v2.0).

# AINOS Hardware Bring-Up Notes

## Supported Platforms

| Platform | SoC | Cores | Status |
|----------|-----|-------|--------|
| QEMU virt (`-machine virt -cpu rv64`) | — | 1–8 | ✅ Fully supported |
| SiFive HiFive Unmatched | SiFive FU740 | 4×U74 + 1×S7 | 🚧 PLIC driver ready |
| VisionFive 2 | JH7110 | 4×U74 | 🚧 PLIC driver ready |
| StarFive VisionFive 1 | JH7100 | 2×U74 | ⏳ Untested |

## Boot Flow

```
OpenSBI (M-mode)
  ↓
kernel _start (S-mode, 0x80200000)
  ↓
kernel_main(hart_id, fdt_addr)
  ↓
init_memory(fdt_addr)  ← parses FDT for RAM regions
  ↓
init_interrupts()      ← stvec, sscratch, sie.STIE
  ↓
...smoke tests...
  ↓
init_smp()             ← starts secondary harts via SBI HSM
  ↓
init_timer()           ← arms timer ISR
  ↓
enter_user()           ← sret to U-mode
```

## PLIC (Platform-Level Interrupt Controller)

The PLIC driver (`kernel/src/plic.rs`) handles external interrupts on real hardware.
On QEMU virt, the PLIC is present at `0x0C00_0000` but VirtIO interrupts are
edge-triggered and the SBI fw_base handles forwarding. The driver is ready for
real hardware but not yet wired into the trap handler.

To enable on real hardware:
1. Call `crate::plic::plic_init()` after `init_interrupts()` in `kernel_main`.
2. Add `SCAUSE_EXTERNAL_INTR` handling to `handle_trap`.
3. Call `plic_claim(hart_id)` to get the IRQ, handle it, then `plic_complete(hart_id, irq)`.

## UART

On QEMU virt, OpenSBI initializes the 16550 UART and the kernel uses `sbi::putchar()`.
On real hardware:
- **HiFive Unmatched**: NS16550 at 0x10010000
- **VisionFive 2**: NS16550 at 0x10000000

A native UART driver would replace `sbi::putchar()` for lower latency.

## Device Tree (FDT)

The kernel receives the FDT address in `a1` (second argument to `kernel_main`).
Memory regions are parsed by `init_memory(fdt_addr)`. For real hardware:
- Ensure the bootloader (U-Boot + OpenSBI) passes a valid FDT.
- The kernel's `#address-cells` and `#size-cells` parser handles `ns16550` nodes.
- RAM is detected from `/memory` node's `reg` property.

## Known Limitations

1. **No PLIC forwarding**: External interrupts (UART, disk) are not handled yet.
2. **No CLINT/mtimecmp**: Timer interrupts go through SBI, not direct CLINT access.
3. **No PMP**: Physical Memory Protection is not configured — all physical memory
   is accessible from S-mode.
4. **No MMU isolation between processes**: All processes share one Sv39 page table
   root (single-address-space MVP).
5. **Hardcoded PLIC base**: Not parsed from FDT; uses QEMU virt default (0x0C00_0000).

## Building for Hardware

```bash
# Build the kernel
cd kernel
cargo build --target riscv64gc-unknown-none-elf --release

# Convert to binary (for U-Boot)
rust-objcopy -O binary \
  target/riscv64gc-unknown-none-elf/release/ainos-kernel \
  ainos-kernel.bin

# Copy to SD card (VisionFive 2 example)
# Mount the boot partition and replace the kernel image
```
