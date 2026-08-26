# Research Findings — AI-Native Operating System

> **v2 amendment (2026-08-20):** The product vision is restated:
> *"a Linux system for ethical hacking on the inside, a Windows-style desktop
> on the outside, with AI as a first-class S-rank kernel subsystem that
> controls the whole system."* The original notes that follow informed the v1
> framing; they remain as **research substrate** for the userspace capability /
> IPC / scheduler designs but are no longer the shipping path. The shipping
> path is built on a hardened Linux host + KDE Plasma 6 + Wine/Proton +
> MCP-native S-rank AI subsystem.
>
> New authoritative sources cited as v2 reference: [Kali Linux tools](https://www.kali.org/tools/),
> [KDE Plasma](https://en.wikipedia.org/wiki/KDE_Plasma), [Wine](https://en.wikipedia.org/wiki/Wine_(software)),
> [Proton](https://en.wikipedia.org/wiki/Proton_(software)), [MCP introduction](https://modelcontextprotocol.io/introduction),
> [Wayland protocol](https://en.wikipedia.org/wiki/Wayland_(display_server_protocol)),
> [AI agent OS integration](https://en.wikipedia.org/wiki/AI_agent),
> [Kali Linux (Wikipedia)](https://en.wikipedia.org/wiki/Kali_Linux).
>
- https://os.phil-opp.com/ – "Writing an OS in Rust" (second edition, last updated ~2023, still canonical)
- https://github.com/phil-opp/blog_os/issues/1387 – soft-float ABI fix
- https://github.com/phil-opp/blog_os/issues/1292 – data-layout fix
- https://github.com/rust-osdev/bootloader/issues/236 – PageAlreadyMapped with x86_64-unknown-none
- https://github.com/phil-opp/blog_os/pull/1425 – target-c-int-width must be number
- https://docs.rs/x86_64/0.15.5/x86_64/ – latest x86_64 crate API docs

---

## Project Status

- **Posts 1–4 (Minimal Kernel → Testing):** Complete, fully working with custom target spec
- **Post 5 (CPU Exceptions):** Complete – breakpoint handler via `lazy_static!` IDT, tested via `int3`
- **Post 6 (Double Faults):** Complete – GDT + TSS with IST stack, double fault handler, tested via `stack_overflow`
- **Post 7 (Hardware Interrupts):** Complete – 8259 PIC init at offsets 32/48, timer IRQ0 handler (EOI), `hlt` idle loop
- **Post 8 (Paging Introduction):** Foundation set – `paging.rs` with `OffsetPageTable` init + `EmptyFrameAllocator`; `BootInfoFrameAllocator` not yet implemented
- **Post 9–10 (Heap Allocation & Beyond):** Not started
- **Environment:** Windows 10 x86_64, 8 GB RAM, Nightly Rust (x86_64-pc-windows-gnu)
- **QEMU emulation:** Direct kernel boot (no full VM), bootimage runner v0.10.4

---

## Key Decisions / Lessons

1. **Custom target spec required** – built-in `x86_64-unknown-none` causes `PageAlreadyMapped` panic at runtime with bootloader 0.9.x. Custom `x86_64-blog_os.json` avoids this.
2. **`cargo bootimage` is the build command** – plain `cargo build` fails because `build-std` is configured (it needs `-Z build-std` which bootimage provides).
3. **Rust 1.95+ de-stabilised JSON target specs** (PR #150151, Jan 2026) – custom `.json` targets require `[unstable] json-target-spec = true` in `.cargo/config.toml`. `bootimage` 0.10.4 handles `-Zjson-target-spec` passthrough.
4. **Dependency versions (as of July 2026):**
   - `bootloader = "0.9.35"` – latest 0.9.x (compatible with blog, works with custom target)
   - `x86_64 = "0.15.5"` – latest stable (v2.0 exists but is a breaking rewrite)
   - `uart_16550 = "0.3.2"` – stable (v0.6.0 is a full rewrite with incompatible API)
   - `spin = "0.9"` – stable (v0.10+ exists but breaks `Mutex` API)
   - `volatile = "0.2.6"` – stable (v0.4+/0.6+ use `VolatilePtr` instead of `Volatile`)
   - `lazy_static = "1.4"` – stable (resolves to 1.5.0)
   - `bootimage = "0.10.4"` – latest (handles `json-target-spec` passthrough)

---

## Current File State

### x86_64-blog_os.json
```json
{
    "llvm-target": "x86_64-unknown-none",
    "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
    "arch": "x86_64",
    "target-endian": "little",
    "target-pointer-width": 64,
    "target-c-int-width": 32,
    "os": "none",
    "executables": true,
    "linker-flavor": "ld.lld",
    "linker": "rust-lld",
    "panic-strategy": "abort",
    "disable-redzone": true,
    "features": "-mmx,-sse,+soft-float",
    "rustc-abi": "softfloat"
}
```

### .cargo/config.toml
```toml
[unstable]
json-target-spec = true
build-std-features = ["compiler-builtins-mem"]
build-std = ["core", "compiler_builtins"]

[build]
target = "x86_64-blog_os.json"

[target.'cfg(target_os = "none")']
runner = "bootimage runner"
```

### Cargo.toml
```toml
[package]
name = "blog_os"
version = "0.1.0"
edition = "2021"

[dependencies]
bootloader = "0.9.35"
lazy_static = { version = "1.4", features = ["spin_no_std"] }
spin = "0.9"
uart_16550 = "0.3.2"
volatile = "0.2.6"
x86_64 = "0.15.5"

[package.metadata.bootimage]
test-args = ["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04", "-serial", "stdio", "-display", "none"]
test-success-exit-code = 33
test-timeout = 300

[[test]]
name = "basic_boot"
harness = true

[[test]]
name = "should_panic"
harness = false
```

### src/main.rs
```rust
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::println;
use bootloader::entry_point;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    blog_os::init();
    println!("Hello World!");

    #[cfg(test)]
    test_main();

    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}
```

### src/lib.rs
```rust
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

pub mod interrupts;
pub mod serial;
pub mod vga;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn init() {
    serial::init();
    interrupts::init_idt();
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
```

### src/interrupts.rs
```rust
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[cfg(test)]
#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
```

### src/vga.rs
Full VGA text mode driver with `Writer` struct, `Color` enum, `WRITER` static via `lazy_static!` + `spin::Mutex`, and `println!`/`print!` macros. Uses `volatile::Volatile` wrapper for buffer cell accesses.

### src/serial.rs
Serial port driver via `uart_16550::SerialPort` at COM1 (0x3F8). `SERIAL1` static via `lazy_static!` + `spin::Mutex`. Exports `serial_print!`/`serial_println!` macros.

### tests/basic_boot.rs
Integration test: one `test_println` test case. Uses `blog_os::println` and `blog_os::test_runner`.

### tests/should_panic.rs
Integration test: triggers `assert_eq!(0, 1)` and expects panic handler to call `exit_qemu(Success)`.

---

## Remaining Work

### Post 8 – Paging (in progress)
- [ ] Implement `BootInfoFrameAllocator` (uses memory map from BootInfo for real frame allocation)
- [ ] Add `mapper.map_to()` test (map a page to a frame)
- [ ] Verify page translation works end-to-end

### Post 9 – Heap Allocation
- [ ] Implement linked list / bump allocator
- [ ] Create kernel heap region via page mapping
- [ ] Add `alloc` crate support

### Post 10+ – Advanced Topics
- [ ] Async/await (cooperative multitasking)
- [ ] Keyboard/PS2 driver
- [ ] Filesystem / storage
- [ ] User-space / syscalls (ambitious)
- [ ] AI integration (per user's vision)

---

## Learning Resources (Running Collection)

### SMP / Multicore (~27 sources)

1. OSDev Wiki SMP — wiki.osdev.org/SMP
2. OSDev Wiki Symmetric Multiprocessing — wiki.osdev.org/Symmetric_Multiprocessing
3. OSDev Wiki APIC — wiki.osdev.org/APIC
4. OSDev Wiki MP Specification — wiki.osdev.org/MP_Specification
5. OSDev Wiki Multiprocessing — wiki.osdev.org/Multiprocessing
6. OSDev User:Shikhin/Tutorial SMP — wiki.osdev.org/User:Shikhin/Tutorial_SMP
7. OSDev.wiki SMP — osdev.wiki/wiki/Symmetric_Multiprocessing
8. Hakutaku: SMP on x86-64 (Codetector) — codetector.org/post/hakutaku/smp/
9. Bringing SMP to Your UP OS (cheesecake.org) — cheesecake.org/sac/smp.html
10. Bringing SMP to Your UP OS (kos.enix.org mirror) — kos.enix.org/pub/how_to_bring_smp.html
11. Multiprocessing Support for Hobby OSes (osdever.net) — osdever.net/tutorials/view/multiprocessing-support-for-hobby-oses-explained
12. Linux kernel smpboot.c (arch/x86/kernel/smpboot.c) — github.com/torvalds/linux/blob/master/arch/x86/kernel/smpboot.c
13. Stack Overflow: Using APIC to create IPIs to wake APs — stackoverflow.com/questions/16364817
14. OSDev Forum: SMP initialization — forum.osdev.org/viewtopic.php?t=40408
15. OSDev Forum: Initializing Multi-Core Processor and Hyper-Threading — forum.osdev.org/viewtopic.php?t=51213
16. OSDev Forum: Simplest way to start extra CPUs — forum.osdev.org/viewtopic.php?t=13752
17. Linked List Blog: Maestro SMP with ACPI (Rust) — linkedlist.org/2026/01/14/maestro-acpi
18. rCore-Tutorial v3 (RISC-V SMP) — github.com/RumuCG/rCore-Tutorial-v3
19. tg-rcore-tutorial SMP (T2L9 multicore) — github.com/cg24-THU/tg-rcore-tutorial
20. Building a microkernel in Rust Part 3 (AArch64 SMP concepts) — blog.desigeek.com/post/2026/03/building-microkernel-part3-concurrency-preemption/
21. FreeRTOS SMP Tutorial (RP2040) — learnembeddedsystems.co.uk/freertos-smp-tutorial
22. TLDP: Linux i386 SMP Boot Code HOWTO — tldp.org/HOWTO/Linux-i386-Boot-Code-HOWTO/smpboot.html
23. Intel SDM Vol 3 Ch 8: MP Initialization (via xem.github.io) — xem.github.io/minix86/manual/intel-x86-and-64-manual-vol3/ (pages 276, 309)
24. Writing a Linux-style OS From Scratch (Toyix series) — coderancher.us/2026/06/20/writing-a-linux-style-operating-system-from-scratch/
25. willothy/goose (hobby OS, Rust, SMP goals) — github.com/willothy/goose
26. kimbethstonehouse/multicore-support (InfOS SMP) — github.com/kimbethstonehouse/multicore-support
27. OpenRISC SMP Linux on De0 Nano — openrisc.io/tutorials/docs/linux-on-de0nano-multicore.html

**Running total: ~252 sources**

---

### ACPI Parsing (~30 sources)

1. OSDev Wiki RSDP — wiki.osdev.org/RSDP
2. OSDev Wiki RSDT — wiki.osdev.org/RSDT
3. OSDev Wiki XSDT — wiki.osdev.org/XSDT
4. OSDev Wiki MADT — wiki.osdev.org/MADT
5. OSDev Wiki ACPI — wiki.osdev.org/ACPI
6. OSDev Wiki ACPICA — wiki.osdev.org/ACPICA
7. OSDev.wiki RSDP — osdev.wiki/wiki/RSDP
8. OSDev.wiki RSDT — osdev.wiki/wiki/RSDT
9. OSDev.wiki XSDT — osdev.wiki/wiki/XSDT
10. OSDev.wiki MADT — osdev.wiki/wiki/MADT
11. ACPI Specification 6.5 (uefi.org) — uefi.org/specs/ACPI/6.5_A/
12. Osdev-Notes: ACPI Tables — baponkar.github.io/Osdev-Notes/02_Architecture/06_ACPITables.html
13. Osdev-Notes GitHub (dreamportdev) — github.com/dreamportdev/Osdev-Notes/blob/master/02_Architecture/06_ACPITables.md
14. Hakutaku: ACPI & SMP — codetector.org/post/hakutaku/smp/
15. Dhairya's Notes: x86 Device Enumeration (ACPI + PCIe) — notes.guptadhairya.com/Semesters/Spring-2026-Semester/CS-378---Multicore-Operating-Systems/x86-Device-Enumeration
16. mittos64 ACPI driver (C) — github.com/thomasloven/mittos64/blob/master/src/kernel/drivers/acpi.c
17. pdoane/osdev ACPI parser (C) — github.com/pdoane/osdev/blob/master/acpi/acpi.c
18. CellOS ACPI parser (C) — github.com/CoryXie/CellOS/blob/master/arch/x64/acpi.c
19. rust-osdev/acpi crate (Rust) — github.com/rust-osdev/acpi
20. docs.rs/acpi (Rust API docs) — docs.rs/acpi/latest/acpi/
21. rust-osdev/acpi PR #246 — rewrite of entire AML interpreter (v6.0.0) — github.com/rust-osdev/acpi/pull/246
22. crates.io/acpi v5.2.0 → v6.1.1 — crates.io/crates/acpi
23. ACPI tables mapping (Rust, HackMD) — hackmd.io/@royhuang/ACPI
24. Linux kernel ACPI Rust abstractions (2026 patches) — ratatoskr.run/linux-acpi/2026/01/3362409
25. Linux kernel Rust ACPI match table (LKML v3) — lkml.iu.edu/2506.0/03803.html
26. Linux kernel Rust ACPI PRP0001 fix (LKML v5) — yhbt.net/lore/lkml/20260420-rust_acpi_prp0001-v5-1-f77869b18b9f@posteo.de/T/
27. Linux kernel ACPI Rust DeviceId (v6) — lists.openwall.net/linux-kernel/2025/06/13/979
28. YouTube: How ACPI Works (various channels)
29. Intel SDM Vol 3 Ch 10: APIC — referenced throughout APIC/ACPI resources
30. MultiProcessor Specification (Intel) — referenced throughout, foundation for MP boot

**Running total: ~282 sources**

---

### PCI / PCIe Enumeration (~25 sources)

1. OSDev Wiki PCI — wiki.osdev.org/PCI
2. OSDev Wiki PCI Express — wiki.osdev.org/PCI_Express
3. OSDev.wiki PCI Express — osdev.wiki/wiki/PCI_Express
4. OSDev Wiki Detecting Hardware — wiki.osdev.org/Detecting_Hardware
5. OSDev Wiki UDI Device Enumeration — wiki.osdev.org/UDI_Device_Enumeration
6. OSDev Forum: PCIe enumeration — forum.osdev.org/viewtopic.php?t=33329
7. OSDev Forum: PCIe devices info — forum.osdev.org/viewtopic.php?t=57361
8. OSDev Forum: MMIO_Starting_Physical_Address — forum.osdev.org/viewtopic.php?t=50560
9. OSDev Forum: What is meant by enumerating PCI bus — forum.osdev.org/viewtopic.php?t=25862
10. Stack Overflow: Accessing PCI space through toy kernel — stackoverflow.com/questions/26808895
11. Stack Overflow: Does modern OS use MCFG for xHCI? — stackoverflow.com/questions/66924486
12. Linux kernel documentation: ACPI for PCI host bridges — docs.kernel.org/PCI/acpi-info.html
13. Linux kernel pci_mcfg.c — github.com/torvalds/linux/blob/master/drivers/acpi/pci_mcfg.c
14. Dhairya's Notes: x86 Device Enumeration — notes.guptadhairya.com (PCIe ECAM + recursive scan)
15. Northeastern University: PCI Enumeration (xv6 lab) — khoury.northeastern.edu/~pjd/cs7680/homework/pci-enumeration.html
16. Medium: PCIe Enumeration Is Not Plug-and-Play — medium.com/@shankar_ravi_v
17. VLSI Trainers: PCIe Enumeration Explained — vlsitrainers.com/pcie-enumeration/
18. Reversing Engineering for the Soul: PCIe Tutorial Part 1 (Windows/WinDbg) — ctf.re/windows/kernel/pcie/tutorial/2023/02/14/pcie-part-1/
19. ChaiOS PCIe driver (C++) — github.com/ChaiSoft/ChaiOS/blob/master/Chaikrnl/pciexpress.cpp
20. PeachOS PCI/ECAM driver (C) — github.com/nibblebits/PeachOS64BitModuleTwo (Lecture 178)
21. asterinas PCI ECAM (Rust) — github.com/asterinas/asterinas/pull/2914
22. Haiku PCI ECAM reimplementation — github.com/davidkaroly/haiku/commit/1a88b57
23. FreeBSD PCI host generic ACPI — leidinger.net/FreeBSD/dox/dev_pci/html/
24. szhou42/osdev PCI enumeration (C) — github.com/szhou42/osdev/blob/master/src/kernel/drivers/pci.c
25. CellOS PCI driver — github.com/CoryXie/CellOS (PCI enumeration via legacy 0xCF8/0xCFC)

**Running total: ~307 sources**

---

### USB / xHCI / EHCI / UHCI / OHCI (~30 sources)

1. OSDev Wiki Universal Serial Bus — wiki.osdev.org/Universal_Serial_Bus
2. OSDev.wiki USB — osdev.wiki/wiki/Universal_Serial_Bus
3. OSDev Wiki xHCI — wiki.osdev.org/EXtensible_Host_Controller_Interface
4. OSDev Wiki EHCI — wiki.osdev.org/EHCI
5. OSDev Wiki Enhanced Host Controller Interface — wiki.osdev.org/Enhanced_Host_Controller_Interface
6. OSDev Wiki OHCI — wiki.osdev.org/OHCI
7. OSDev Wiki UHCI — wiki.osdev.org/Universal_Host_Controller_Interface
8. OSDev.wiki UHCI — osdev.wiki/wiki/Universal_Host_Controller_Interface
9. OSDev Forum: Writing xHCI Driver — forum.osdev.org/viewtopic.php?t=57746
10. YouTube: From Zero to a Native xHCI Driver (seL4 summit) — youtube.com/watch?v=Jsp2VMq3uDI
11. FlareCoding YouTube playlist: xHCI driver tutorial — youtube.com/playlist?list=PLATP7rOKo3E82tBnMp90B4zejpWeAKlxn
12. xHCI Specification 1.2 (Intel) — intel.com/content/www/us/en/products/docs/io/universal-serial-bus/extensible-host-controller-interface-usb-xhci.html
13. Linux xHCI driver (drivers/usb/host/xhci.c) — github.com/torvalds/linux/blob/master/drivers/usb/host/xhci.c
14. illumos xHCI driver — github.com/illumos/illumos-gate/blob/master/usr/src/uts/common/io/usb/hcd/xhci/xhci.c
15. HelenOS USB/xHCI documentation (PDF) — helenos.org/doc/helenos-usb3-doc.pdf
16. ChaiOS xHCI driver (C++) — github.com/ChaiSoft/ChaiOS/blob/master/Chaikrnl/xhci.cpp
17. suhteevah/xhci-nostd (Rust no_std xHCI + HID keyboard) — github.com/suhteevah/xhci-nostd
18. drivercraft/CrabUSB (Rust async xHCI) — github.com/drivercraft/CrabUSB
19. rust-osdev/xhci (Rust xHCI structs crate) — github.com/rust-osdev/xhci
20. usb-oxide crate (Rust xHCI + MSC + HID) — crates.io/crates/usb-oxide
21. crab-usb crate (Rust async USB host) — docs.rs/crate/crab-usb/latest
22. ez_xhci crate (Rust xHCI library) — github.com/ChocolateLoverRaj/ez_xhci
23. RustOS-Dev/RustOS (xHCI + FAT32 + VFS) — github.com/RustOS-Dev/RustOS
24. UEFI EDK2 USB Driver Design Guidelines — tianocore-docs.github.io/edk2-UefiDriverWritersGuide/draft/19_usb_driver_design_guidelines/
25. USB 2.0 Specification (usb.org) — usb.org/documents
26. USB 3.2 Specification (usb.org) — usb.org/documents
27. AmigaOS 4 xHCI driver (derfsss/usb3-amigaos4) — github.com/derfsss/usb3-amigaos4
28. Fennix OS xHCI driver — git.enderice2.com/enderice2/Fennix/commit/7a648cea5c
29. ormastes/simple xHCI driver (SPL language) — github.com/ormastes/simple/commit/00859e1
30. USB in a Nutshell (beyondlogic.org) — beyondlogic.org/usbnutshell/usb-in-a-nutshell/

**Running total: ~337 sources**

---

### Networking / TCP/IP Stack & NIC Drivers (~30 sources)

1. OSDev Wiki Network Stack — wiki.osdev.org/Network_Stack
2. OSDev Wiki RTL8139 — wiki.osdev.org/RTL8139
3. OSDev Wiki Intel 8254x (E1000) — wiki.osdev.org/Intel_8254x
4. OSDev Wiki Ethernet devices — wiki.osdev.org/Ethernet_devices
5. OSDev Forum: Network card driver in real mode — forum.osdev.org/viewtopic.php?t=29229
6. smoltcp (Rust no_std TCP/IP stack) — github.com/smoltcp-rs/smoltcp
7. lwIP (lightweight TCP/IP stack) — savannah.nongnu.org/projects/lwip/
8. uIP (micro IP stack) — github.com/adamdunkels/uip
9. Let's code a TCP/IP stack (saminiir) — saminiir.com/lets-code-tcp-ip-stack-1-ethernet-arp/ (series: Ethernet/ARP, IPv4/ICMP, UDP, TCP)
10. On writing a network stack (William Durand, ArvernOS) — williamdurand.fr/2022/02/17/on-writing-a-network-stack-part-1/
11. TCP/IP Illustrated (W. Richard Stevens) — canonical reference
12. MIT 6.S081 xv6 Networking Lab — pdos.csail.mit.edu/6.S081/2020/labs/net.html
13. MIT 6.1810 xv6 Networking Lab (2025) — pdos.csail.mit.edu/6.1810/2025/labs/net.html
14. MIT JOS Lab 6: Network Driver — pdos.csail.mit.edu/6.1810/2018/labs/lab6/
15. Harvey Mudd CS134 Lab 9: Network Driver — cs.hmc.edu/~rhodes/cs134/labs/lab9.html
16. xv6 Rust Networking (Alessandro Ferrari) — alessandroferrari.live/rust-networking-inside-xv6/
17. xv6 E1000 + UDP stack (IslamTayeb) — github.com/IslamTayeb/xv6-networking-project
18. RPi4 OS Part 15: TCP/IP Web Server — rpi4os.com/part15-tcpip-webserver/
19. Network stack from scratch (Nim) — kostandyan.xyz/blog/networking/
20. Let's write TCP/IP from scratch (Medium) — medium.com/@skaiuijing/lets-write-a-tcp-ip-stack-from-scratch-part1-ethernet-frame-i-o
21. TanayK07/networking-from-scratch (289 lessons, C/Python) — github.com/TanayK07/networking-from-scratch
22. PKU SOAR lab-netstack (Ethernet/IP/TCP) — github.com/SOAR-PKU/lab-netstack
23. Linux e1000 driver — github.com/torvalds/linux/tree/master/drivers/net/ethernet/intel/e1000
24. Linux smc91x driver — github.com/torvalds/linux/blob/master/drivers/net/ethernet/smsc/smc91x.c
25. szhou42 RTL8139 driver (C) — github.com/szhou42/osdev/blob/master/src/kernel/drivers/rtl8139.c
26. Intel E1000 Software Developer's Manual — intel.com content/dam/doc/manual/
27. Building a OS Lab 10: Networking Part 1 (xiayingp.gitbook.io) — xiayingp.gitbook.io/build_a_os/labs/lab-10-networking-part-1
28. QEMU user-mode network stack documentation — qemu.readthedocs.io
29. WireShark / tcpdump — packet analysis tools for debugging
30. Ultibo SMC91X driver (Pascal) — ultibo.org/wiki/Unit_SMC91X

**Running total: ~367 sources**

---

### ELF Loader, User Mode & Syscalls (~20 sources)

1. OSDev Wiki ELF — wiki.osdev.org/ELF
2. OSDev Wiki System Calls — wiki.osdev.org/System_Calls
3. Wasil Zafar Phase 9: ELF Loading & Executables — wasilzafar.com/pages/series/kernel-development/kernel-dev-phase-09-elf.html
4. EuraliOS journal: Userspace (ELF load, user mode, syscalls) — github.com/bendudson/EuraliOS/blob/main/doc/journal/02-userspace.org
5. Hux kernel Wiki 14: User Mode Execution — github.com/josehu07/hux-kernel/wiki/14.-User-Mode-Execution
6. Hux kernel Wiki 15: System Calls API — github.com/josehu07/hux-kernel/wiki/15.-System-Calls-API-Setup
7. Fusion OS: ELF Loader (Khaled Hammouda, Nim) — 0xc0ffee.netlify.app/osdev/21-elf-loader-p1
8. Fusion OS: System Call Interface / SYSCALL/SYSRET — 0xc0ffee.netlify.app/osdev/
9. SkelixOS Tutorial 09: System Call & Executing Programs — skelix.net/skelixos/tutorial09_en.html
10. Stanford Pintos Project 2: User Programs (ELF + syscalls) — scs.stanford.edu/26sp-cs212/pintos/pintos_3.html
11. Linux x86-64 Syscall Table (rchapman) — blog.rchapman.org/posts/Linux_System_Call_Table_for_x86_64/
12. Linux Syscall Table (filippo.io) — filippo.io/linux-syscall-table/
13. Intel SDM Vol 2: SYSCALL/SYSRET instructions
14. Intel SDM Vol 3A Ch 4: Paging and user-mode access control
15. Rust `object` crate (ELF parsing) — crates.io/crates/object
16. goblin crate (ELF/mach-o/PE) — crates.io/crates/goblin
17. xmas-elf crate — crates.io/crates/xmas-elf
18. Redox OS ELF loader — gitlab.redox-os.org/redox-os/kernel
19. Theseus OS ELF loading (Rust) — github.com/theseus-os/Theseus
20. Tock OS ELF loading (Rust, embedded) — github.com/tock/tock

### GPU / Framebuffer / Display Drivers (~15 sources)

1. OSDev Wiki VGA Hardware — wiki.osdev.org/VGA_Hardware
2. OSDev Wiki VESA Framebuffer — wiki.osdev.org/VESA
3. OSDev Wiki Drawing In Protected Mode — wiki.osdev.org/Drawing_In_Protected_Mode
4. OSDev Wiki GOP (UEFI Graphics Output Protocol) — wiki.osdev.org/GOP
5. Linux vesafb documentation — docs.kernel.org/fb/vesafb.html
6. Linux simpledrm driver (DRM for simplefb) — lists.freedesktop.org/archives/dri-devel/2021-April/303293.html
7. Linux efidrm + vesadrm drivers (2025) — lists.freedesktop.org/archives/dri-devel/2025-March/496651.html
8. Linux corebootdrm driver (2026) — lwn.net/Articles/1053324/
9. Linux DRM GPU Driver Developer's Guide — docs.kernel.org/gpu/
10. Rust framebuffer driver for Linux (2026) — lists.freedesktop.org/archives/dri-devel/2026-January/546829.html
11. Phoronix: Verisilicon DC8200 & Coreboot Framebuffer — phoronix.com/news/Linux-7.1-DC8200-Coreboot-FB
12. Limine boot protocol framebuffer — github.com/limine-bootloader/limine
13. RustOS GOP framebuffer driver — github.com/RustOS-Dev/RustOS (docs/FRAMEBUFFER_IMPLEMENTATION.md)
14. UEFI GOP specification — uefi.org/specifications
15. VBE/VESA BIOS Extensions Core 3.0 — vesa.org

**Running total: ~402 sources**

---

### UEFI Boot (~20 sources)

1. OSDev Wiki UEFI — wiki.osdev.org/UEFI
2. OSDev Wiki Bare Bones with Rust (UEFI) — wiki.osdev.org/Rust_Bare_Bones#UEFI
3. Rust UEFI Book (tutorial series) — rust-osdev.github.io/uefi-rs/
4. rust-osdev/uefi-rs crate & docs — github.com/rust-osdev/uefi-rs, docs.rs/uefi
5. Atharva Pandey: Lesson 10 — Writing a Bootloader (UEFI in Rust) — atharvapandey.com/post/rust/rust-sys-bootloader/
6. Tony Huang: Rust-based OS booting with UEFI (series) — cnwzhjs.github.io/rust-based-os-botting-with-uefi-1/
7. HackMD: Rust OS Dev Journey Pt.1 — basic UEFI application — hackmd.io/@eA9qJeBySNywR3Oi6h1T0Q/BkBQiJ2id
8. malware.re: OS Experiment in Rust — Creating a UEFI Loader (parts 1–2) — blog.malware.re/2023/08/20/rust-os-part1/
9. OSDev Wiki POSIX-UEFI — wiki.osdev.org/POSIX-UEFI
10. TianoCore EDK2/OVMF UEFI firmware — github.com/tianocore/edk2
11. uefi-run (cargo tool for testing UEFI in QEMU) — crates.io/crates/uefi-run
12. uefi-services crate — crates.io/crates/uefi-services
13. Rust UEFI Application Template — github.com/rust-osdev/uefi-rs/tree/main/template
14. Rust std for UEFI (ongoing effort) — github.com/rust-osdev/uefi-rs#std-implementation
15. Fullerene OS (Rust UEFI kernel + scheduler) — github.com/p14c31355/fullerene
16. echOS-x64 (Rust UEFI + SMP + networking) — github.com/asosyal04440/echOS-x64
17. ChronoOS (Rust no_std x86_64, UEFI loader) — github.com/VanshajPoonia/chronosapien
18. rOSt (BIOS + UEFI, microkernel, round-robin) — github.com/ComicalCache/rOSt
19. QEMU with OVMF usage guide — wiki.qemu.org/Testing_with_OVMF
20. Intel SDM Vol 3: UEFI boot path (Sec 8.10, 9A)

### Multitasking / Scheduler & Context Switching (~20 sources)

1. OSDev Wiki Multitasking — wiki.osdev.org/Multitasking
2. OSDev Wiki Context Switching — wiki.osdev.org/Context_Switching
3. OSDev Wiki Scheduling Algorithms — wiki.osdev.org/Scheduling_Algorithms
4. OSDev Wiki Round Robin — wiki.osdev.org/Round-robin
5. Nikos Filippakis: Rust-OS Kernel Task Scheduler (round-robin) — nfil.dev/kernel/rust/coding/rust-kernel-task-scheduler/
6. rou2exOS Scheduler docs (PIT @ 100 Hz, round-robin, sleep, states) — r2.krusty.space/multitasking/scheduler/
7. Amit Bahree: Building a microkernel in Rust — Part 2 (IPC, cooperative scheduler) — blog.desigeek.com/post/2026/03/building-microkernel-part2-communication-ipc/
8. Amit Bahree: Building a microkernel in Rust — Part 3 (preemption, timer IRQ, context switch) — blog.desigeek.com/post/2026/03/building-microkernel-part3-concurrency-preemption/
9. 0XLUC4/rust-x86_64-kernel (preemptive scheduler, ring-3, CoW fork, round-robin) — github.com/0XLUC4/rust-x86_64-kernel
10. thatmagicalcat/magicalos (preemptive, round-robin, sync, async executor) — github.com/thatmagicalcat/magicalos
11. Fullerene kernel scheduler (preemptive round-robin, UEFI) — github.com/p14c31355/fullerene
12. echOS-x64 task/scheduler (CFS, RT, deadline, SMP work-steal) — github.com/asosyal04440/echOS-x64
13. MagicalOS threads (kernel + user threads, context switch, scheduler) — github.com/thatmagicalcat/magicalos
14. Philipp Oppermann: Async/Await post (cooperative scheduling) — os.phil-opp.com/async-await/
15. xv6 scheduler source (MIT, educational) — github.com/mit-pdos/xv6-riscv
16. Roslyn McConnell: Systems Programming with Rust (round-robin, time-slicing) — rozmichelle.com/systems-programming-with-rust/
17. TAOS Labs: Events & Scheduling (async/await in kernel) — taos-labs.dev/posts/events/
18. Intel SDM Vol 3 Ch 7, 8: Task Management, MWAIT/HLT
19. Theseus OS scheduler (Rust, TLS + work-stealing) — github.com/theseus-os/Theseus
20. Tock OS scheduling (cooperative, round-robin, Rust embedded) — github.com/tock/tock

### PS/2 Keyboard Driver (~15 sources)

1. OSDev Wiki PS/2 Controller — wiki.osdev.org/PS/2_Controller (formerly Keyboard_Controller)
2. OSDev Wiki PS/2 Keyboard — wiki.osdev.org/PS/2_Keyboard
3. OSDev Wiki Scancode Sets — wiki.osdev.org/Scancode_Sets
3. docs.rs/pc-keyboard (Rust scancode decoder, layouts, EventDecoder) — docs.rs/pc-keyboard
4. docs.rs/ps2 (Rust low-level PS/2 controller) — docs.rs/ps2
5. docs.rs/polished_ps2 (Rust PS/2 init library) — docs.rs/polished_ps2
6. Philipp Oppermann: Hardware Interrupts (keyboard interrupt handler + pc-keyboard) — os.phil-opp.com/hardware-interrupts/#keyboard-input
7. baponkar.github.io/Osdev-Notes: Keyboard Interrupt Handling (IOAPIC + set 1/2) — baponkar.github.io/Osdev-Notes/02_Architecture/10_Keyboard_Interrupt_Handling.html
8. emk/toyos-rs keyboard.rs (PS/2 driver with modifiers) — github.com/emk/toyos-rs/blob/master/src/arch/x86_64/keyboard.rs
9. Julia Evans: Day 37 — After 5 days, my OS doesn't crash when I press a key — jvns.ca/blog/2013/12/04/day-37-how-a-keyboard-works/
10. Random Hacks: Bare Metal Rust — Configure your PIC + keyboard — randomhacks.net/2015/11/16/bare-metal-rust-configure-your-pic-interrupts/
11. krnldev.nerdnextdoor.net: PS/2 driver guide (scancode set 1, setup, IRQ unmask) — krnldev.nerdnextdoor.net/x86_64/ps2/
12. Adam Chapweske's PS/2 write-ups (detailed low-level reference)
13. Andries Brouwer: Keyboard scancodes
14. VanshajPoonia/chronosapien keyboard.rs (IRQ + polling fallback) — github.com/VanshajPoonia/chronosapien
15. MagicalOS keyboard implementation (async + interrupt-driven) — github.com/thatmagicalcat/magicalos

### Lock-Free Programming, Atomics & Synchronization (~12 sources)

1. Rust embedded WG: How to use atomics (in-depth explanation) — docs.rust-embedded.org/book/concurrency/
2. Mara Bos: Rust Atomics and Locks (O'Reilly book / online chapters) — mara.lgbt/atomics.html
3. core::sync::atomic reference (Rust std) — doc.rust-lang.org/core/sync/atomic/
4. Rust spin crate (spinlock, Mutex, RwLock, Once, Lazy) — crates.io/crates/spin
5. Linux kernel LKMM (Linux Kernel Memory Model) — kernel.org/doc/Documentation/atomic_t.txt
6. Linux Unreliable Guide to Locking (spinlocks, mutexes, atomic_t, IRQ contexts) — kernel.org/doc/html/latest/kernel-hacking/locking.html
7. rust-osdev/spinlock crate (custom spinlock for OS dev) — github.com/rust-osdev/spinlock
8. Linux kernel Rust sync module (SpinLock, Mutex, atomic) — rust-for-linux.github.io/docs/kernel/sync/
9. atomic-wait crate (portable wait/wake primitives for building locks) — crates.io/crates/atomic-wait
10. Rust std::hint::spin_loop (PAUSE instruction wrapper) — doc.rust-lang.org/std/hint/fn.spin_loop.html
11. Intel SDM Vol 3 Ch 8: Memory Ordering, LOCK prefix, MESI protocol
12. Memory Barriers: a Hardware View for Software Hackers (Paul McKenney)

### Async/Await Executor & Runtime (~12 sources)

1. Philipp Oppermann: Async/Await — os.phil-opp.com/async-await/ (full tutorial: futures, executor, waker, async keyboard)
2. Rust Async Book Ch 2: Executors & Wakers — rust-lang.github.io/async-book/02_execution/04_executor.html
3. maitake (Rust no_std async runtime building blocks: task, scheduler, timer) — mnemos.dev/doc/maitake/
4. Embassy framework (no_std async executor + HAL, includes waker integration) — embassy.dev
5. TAOS Labs: Events & Scheduling (kernel async/await, waker, process preempt) — taos-labs.dev/posts/events/
6. Linux kernel Rust async/await executor — rust-for-linux.github.io/docs/kernel/kasync/executor/
7. m561247/awkernel (realtime OS with async/await in kernel space) — github.com/m561247/awkernel
8. RingCore (minimal io_uring-based async runtime in Rust) — github.com/sumant1122/ringcore
9. Blake Hildebrand: Async Rust on Cortex-M (executor, waker, interrupts) — interrupt.memfault.com/blog/embedded-async-rust
10. Fuchsia async executor (common.rs source) — fuchsia-docs.firebaseapp.com/rust/src/fuchsia_async/
11. OpenVMM pal_async local executor — openvmm.dev/rustdoc/linux/src/pal_async/local.rs.html
12. Tock OS async (Rust embedded, no_std futures) — github.com/tock/tock

### PC Speaker & Audio (~10 sources)

1. OSDev Wiki PC Speaker — wiki.osdev.org/PC_Speaker, osdev.wiki/wiki/PC_Speaker
2. OSDev Wiki Sound — wiki.osdev.org/Sound
3. docs.rs/pc-beeper (Rust PC speaker crate) — docs.rs/pc-beeper
4. krnldev.nerdnextdoor.net: PC Speaker — krnldev.nerdnextdoor.net/x86_64/pcspkr/
5. HoShiMin/BeeSynth (Rust PC speaker synthesizer, FFT, PCM audio) — github.com/HoShiMin/BeeSynth
6. BleskOS PC speaker driver (ASM example) — github.com/Klaykap/BleskOS (blesk.asm)
7. Intrinsic PIT driver (Speaker + timer) — github.com/kagerouttepaso/intrinsic (pckbd module)
8. VanshajPoonia/chronosapien (PC speaker tones in kernel) — github.com/VanshajPoonia/chronosapien
9. MOROS (AC97 + SB16 sound card drivers, Rust) — github.com/vinc/moros
10. TTWNO/hdaudio-uefi (Intel HD Audio, Rust no_std port of Redox driver) — github.com/TTWNO/hdaudio-uefi

---

### Filesystem Journaling & Crash Consistency (~12 sources)

1. OSDev Wiki ext2 — wiki.osdev.org/ext2 (inode, block groups, directory entries)
2. OSDev Wiki ext3 — wiki.osdev.org/ext3
3. Linux kernel docs: ext4 journal (jbd2) — docs.kernel.org/filesystems/ext4/journal.html
4. Wasil Zafar Phase 7: Disk Access & Filesystems (ATA + FAT + VFS) — wasilzafar.com/pages/series/kernel-development/kernel-dev-phase-07-filesystem.html
5. am-fs-ext4: pure-Rust ext2/3/4 driver w/ JBD2 journal — github.com/christhomas/rust-fs-ext4
6. fs-ext4 crate (Rust, JBD2, mkfs, fsck) — crates.io/crates/fs-ext4
7. KarpelesLab/fstool (Rust FS tool: ext2/3/4, FAT32, XFS, JBD2, journal replay) — github.com/karpeleslab/fstool
8. fat32-raw crate (Rust low-level FAT32 r/w, LFN) — github.com/meowrch/fat32-raw
9. Remzi Arpaci-Dusseau: OSTEP Ch 42 — Crash Consistency: FSCK and Journaling — pages.cs.wisc.edu/~remzi/OSFEP/file-journaling.pdf
10. Linux kernel random.c (ChaCha20 CRNG, input_pool, fast_pool) — drivers/char/random.c in torvalds/linux
11. docs.rs/ext2 (Rust ext2 library) — docs.rs/ext2
12. MOROS (Rust OS with FAT32 + VFS) — github.com/vinc/moros

### Random Number Generator / Entropy (~8 sources)

1. OSDev Wiki Random Number Generator (RDRAND, RDSEED, TSC entropy, jitter) — wiki.osdev.org/Random_Number_Generator
2. Intel SDM Vol 2: RDRAND, RDSEED instructions
3. Intel SDM Vol 1 Ch 17: RNG / Secure Key
4. Linux kernel random.c architecture (ChaCha20 CRNG, Blake2s input_pool, per-CPU DRNGs) — torvalds/linux/drivers/char/random.c
5. BSI Analysis: Linux Random Number Generator (comprehensive deep dive) — bsi.bund.de
6. Cloudflare Blog: Ensuring Randomness with Linux's RNG — blog.cloudflare.com/ensuring-randomness-with-linuxs-random-number-generator/
7. ArchWiki Random Number Generation — wiki.archlinux.org/title/Random_number_generation
8. rust-osdev/rand crate (no_std RNG for kernel use) — crates.io/crates/rand

---

### CPUID / CPU Feature Detection (~10 sources)

1. OSDev Wiki CPUID — wiki.osdev.org/CPUID
2. docs.rs/raw-cpuid (Rust no_std CPUID parsing, all leaves) — docs.rs/raw-cpuid
3. docs.rs/x86::cpuid (Rust x86 crate CPUID module) — docs.rs/x86/latest/x86/cpuid/
4. Rust stdarch core_arch cpuid implementation (__cpuid, __cpuid_count) — rust-lang.github.io/stdarch/x86_64/core_arch/x86/cpuid/
5. Rust std_detect x86 runtime feature detection — doc.rust-lang.org/beta/src/std_detect/detect/os/x86.rs.html
6. cpufeatures crate (lightweight no_std CPU feature detection for x86, aarch64) — lib.rs/crates/cpufeatures
7. Intel SDM Vol 2A: CPUID instruction reference and leaf tables
8. Intel SDM Vol 3 Ch 3: MTRRs, feature flags, cache/TLB info via CPUID
9. Philipp Oppermann: Entering Long Mode (CPUID usage in bootloader) — os.phil-opp.com/entering-longmode/
10. Wikipedia: CPUID instruction summary — en.wikipedia.org/wiki/CPUID

### ATA / SATA / AHCI / NVMe Storage (~15 sources)

1. OSDev Wiki ATA PIO Mode — wiki.osdev.org/ATA_PIO_Mode
2. OSDev Wiki ATA — wiki.osdev.org/ATA (Master/Slave, IRQ14/15, task file, commands)
3. OSDev Wiki SATA — wiki.osdev.org/SATA
4. OSDev Wiki AHCI — wiki.osdev.org/AHCI (ABAR, command list, PRDT, FIS, port init)
5. OSDev Wiki PCI IDE Controller — wiki.osdev.org/PCI_IDE_Controller
6. suhteevah/ahci-nostd (Rust no_std AHCI/SATA driver) — github.com/suhteevah/ahci-nostd
7. crates.io/nvme-driver (Rust NVMe 1.4 driver, rdif-block, PRP, queues) — crates.io/crates/nvme-driver
8. crates.io/nvme-oxide (Rust bare-metal NVMe driver, init, admin/IO queues, trim) — crates.io/crates/nvme-oxide
9. asterinas NVMe driver PR (Rust, identify/read/write/flush, ktests) — github.com/asterinas/asterinas/pull/1984
10. V.E.L.O.C.I.T.Y.-OS: PCI + NVMe + FAT32 bare-metal drivers (Rust) — dev.to/unitbuilds_cc/velocity-os-writing-bare-metal-drivers-pci-nvme-fat32-part-9-46k1
11. blraaz.me: Building an AHCI Driver (kush-os, detailed walkthrough) — blraaz.me/osdev/2021/06/29/building-ahci-driver.html
12. Tuomas Pirhonen: Writing an NVMe Driver in Rust (thesis, vroom, user space, SPDK) — db.in.tum.de/people/sites/ellmann/theses/finished/24/pirhonen_writing_an_nvme_driver_in_rust.pdf
13. AHCI 1.3.1 Specification (Intel) — intel.com/content/www/us/en/io/serial-ata/serial-ata-ahci-spec-rev1-3-1.html
14. NVMe Specification 1.4 — nvmexpress.org
15. Wasil Zafar Phase 7: Disk Access & Filesystems (ATA + FAT + VFS) — wasilzafar.com/pages/series/kernel-development/kernel-dev-phase-07-filesystem.html

### IOMMU (Intel VT-d / AMD-Vi) (~10 sources)

1. OSDev Wiki Intel VT-d — wiki.osdev.org/Intel_VT-d
2. OSDev Wiki AMD-Vi IOMMU — wiki.osdev.org/AMD-Vi_IOMMU
3. Intel VT-d Architecture Specification (ID D51397)
4. AMD I/O Virtualization Technology (IOMMU) Specification (ID 48882)
5. Linux kernel IOMMU subsystem docs — docs.kernel.org/arch/x86/iommu.html
6. Linux kernel IOMMU architecture internals (VT-d, AMD-Vi, domain, groups) — kernel-internals.org/iommu/iommu-arch/
7. Linux kernel IOMMU userspace API (iommufd, VFIO, nested translation) — docs.kernel.org/userspace-api/iommufd.html
8. KVM: How to assign devices with VT-d — linux-kvm.org/page/How_to_assign_devices_with_VT-d_in_KVM
9. Lenovo: Introduction to IOMMU Infrastructure in Linux Kernel — lenovopress.lenovo.com/lp1467.pdf
10. Xen VT-d HowTo — wiki.xenproject.org/wiki/VTd_HowTo

---

### Kernel Security: SMEP/SMAP/NX/KASLR (~10 sources)

1. OSDev Wiki Supervisor Memory Protection (SMEP + SMAP) — wiki.osdev.org/Supervisor_Memory_Protection
2. Intel SDM Vol 3 Ch 4: Paging (NX bit, U/S bit, SMEP/SMAP enforcement via page faults)
3. Linux kernel CR4 pinning (SMEP/SMAP/UMIP/FSGSBASE bits) — commit 679cd5ce, Linux 5.1
4. Breaking Bits: SMEP exploit bypass (ret2usr → kernel ROP → commit_creds) — breaking-bits.gitbook.io
5. KernelSight: SMEP/SMAP internals (Windows/Linux, CR4 bits, bypass history) — splintersfury.github.io/KernelSight/mitigations/smep-smap/
6. Kernel Pwn Primer: ret2usr, SMEP/SMAP, KASLR (Yunolay) — yunolay.com/kernel-pwn-primer-ret2usr-smep-smap-and-kaslr/
7. Efiens: Learning Linux Kernel Exploitation (SMEP, KPTI, SMAP, ROP bypass) — blog.efiens.com/post/midas/linux-kernel-pwn-2/
8. ret2dir: Rethinking Kernel Isolation (USENIX Security 2014) — usenix.org/system/files/conference/usenixsecurity14/sec14-paper-kemerlis.pdf
9. Wikipedia: Supervisor Mode Access Prevention (SMAP) — en.wikipedia.org/wiki/Supervisor_Mode_Access_Prevention
10. Philipp Oppermann: KASLR / kernel ASLR overview

### Intel VT-x / Virtualization (~10 sources)

1. Intel SDM Vol 3C Ch 23-33: VMX architecture, VMCS, VM entries/exits, EPT
2. Intel VT-x Architecture Specification (C97063-002) — kib.kiev.ua/x86docs/Intel/VT-x/
3. Writing Hypervisor in Zig (30-chapter series, type-1 hypervisor from scratch) — hv.smallkirby.com
4. COMP 530H Lab 6b: Paravirtual VMM (Intel VT-x, EPT, VMCS, vmcall) — cs.unc.edu/~porter/courses/comp530/f18/lab6bh.html
5. Hypervisor From Scratch (Rayanfam / kiwi) — kiwids.me/posts/Hypervisor-From-Scratch/
6. Jonathan M. McCune: The Basics of Intel VT-x Extensions — research.meekolab.com/the-basics-of-intel-vt-x-extensions
7. Linux KVM VMX driver source (arch/x86/kvm/vmx/vmx.c) — github.com/torvalds/linux
8. Linux KVM nested VMX documentation — docs.kernel.org/virt/kvm/x86/nested-vmx.html
9. hyperkit/moby VMX implementation (macOS Hypervisor.framework) — github.com/moby/hyperkit
10. Xen VT-d/VT-x HowTo — wiki.xenproject.org

### CMOS / Real-Time Clock (RTC) (~10 sources)

1. OSDev Wiki RTC — wiki.osdev.org/RTC
2. OSDev Wiki CMOS — wiki.osdev.org/CMOS
3. OSDev Wiki Time And Date — wiki.osdev.org/Time_And_Date
4. OSDev.wiki RTC (IRQ 8, status registers A/B/C, BCD, periodic interrupt) — osdev.wiki/wiki/RTC
5. Linux kernel rtc-cmos driver (drivers/rtc/rtc-cmos.c) — github.com/torvalds/linux
6. Linux arch/x86/kernel/rtc.c (mach_get_cmos_time, rtc_cmos_read/write, PNP detection) — github.com/torvalds/linux
7. ToaruOS cmos.c (RTC → Unix timestamp, TSC calibration, wall clock) — github.com/klange/toaruos
8. OSDev Forum: Basic RTC Driver (C, IRQ 8 handler, BCD conversion, century handling) — f.osdev.org/viewtopic.php?t=17433
9. Intel SDM: PIT, RTC, CMOS NVRAM references
10. MC146818 RTC datasheet (original chip spec for PC/AT)

---

### Debugging & QEMU Tools (~10 sources)

1. QEMU Monitor Protocol (QMP) — qemu.readthedocs.io/en/latest/interop/qemu-qmp-ref.html
2. QEMU `-d int,cpu_reset,guest_errors` logging (interrupt tracing, CPU state dumps)
3. QEMU `-gdb tcp::1234` + GDB cross-debugging (target remote, layout asm, info registers)
4. QEMU `-no-reboot` + `-no-shutdown` for catching triple faults
5. GDB `target remote localhost:1234`, `symbol-file target/x86_64-blog_os/debug/blog_os`
6. Bochs internal debugger (magic breakpoint `xchg bx, bx`, `#ifdef BX_DEBUGGER` break)
7. Rust `RUST_BACKTRACE=1` + `#[panic_handler]` custom backtrace walk (frame pointer, DWARF)
8. `cargo-objdump` / `llvm-objdump -d` for disassembly verification (e.g. `objdump -d -S blog_os.elf`)
9. `cargo-readobj` / `llvm-readobj -h -s -S` for ELF section/segment inspection
10. `cargo-nm` / `llvm-nm` for symbol table inspection (verify linker script layout)

### Heap/Buddy/Slab Memory Allocators (~10 sources)

1. OSDev Wiki: Memory Allocation — wiki.osdev.org/Memory_Allocation
2. Linux `mm/slub.c` – SLUB allocator (percpu freelist, kmem_cache, NUMA-aware) — github.com/torvalds/linux
3. Linux `mm/slab.c` – SLAB allocator (colour-caching, per-CPU arrays, kmem_bufctl_t)
4. Linux `mm/page_alloc.c` – Buddy allocator (MAX_ORDER=11, `__alloc_pages_nodemask`, buddy merging)
5. Doug Lea's `dlmalloc` + `ptmalloc2` (glibc `malloc`) — g.oswego.edu/dl/html/malloc.html
6. Rust `linked_list_allocator` crate (bump + linked-list free, `Heap::alloc`/`dealloc`) — docs.rs/linked_list_allocator
7. Rust `buddy_alloc` crate (buddy system, `BuddyAllocParam` config, non-`alloc` API) — docs.rs/buddy_alloc
8. slab allocator for kernel objects (fixed-size caches, constructor caching) — wiki.osdev.org/Slab_Allocator
9. ToaruOS kernel malloc (buddy + slab, `malloc.c` / `slab.c`) — github.com/klange/toaruos
10. Theseus OS region-based allocator — github.com/theseus-os/Theseus

### Power Management: APM / ACPI P-CPU / C-CPU / S-States (~10 sources)

1. ACPI Specification 6.5 (Ch 3: System & Power Management, Ch 8: Processors, P/C/T-states) — uefi.org/specifications
2. APM 1.2 BIOS Interface Specification (Intel/Microsoft, 1996) — wiki.osdev.org/APM
3. OSDev Wiki APM — wiki.osdev.org/APM
4. OSDev Wiki ACPI — wiki.osdev.org/ACPI
5. ACPI P-State (Performance State): `_PCT`, `_PSS`, `_PPC`, `CPPC` — uefi.org/specifications
6. ACPI C-State (Idle State): `_CST`, MWAIT/MONITOR hints, `CPUID.05H`
7. Linux `drivers/acpi/processor_idle.c` (C-state entry via MWAIT or HLT, tickless idle)
8. Linux `drivers/cpufreq/acpi-cpufreq.c` (P-state via `PERF_CTL` MSR 0x199, freq transitions)
9. Intel SDM Vol 3B Ch 14: Power Management (P-state HW coordination, MSR-based P-state)
10. Intel SDM Vol 3B Ch 15: C-State Management (MWAIT/MONITOR, C1E auto-demotion)

### System Calls / Signal Handling (~10 sources)

1. Intel SDM Vol 3B Ch 6: Interrupt 0x80 (Linux syscall gate, legacy `int 0x80`)
2. `sysenter` / `sysexit` (Intel, MSR `SYSENTER_CS/EIP/ESP` 174h–176h)
3. `syscall` / `sysret` (AMD + Intel, `STAR`/`LSTAR`/`CSTAR` MSRs C000_0081h–C000_0083h, IA32_EFER.SCE)
4. Linux `arch/x86/entry/entry_64.S` – `entry_SYSCALL_64`, `pt_regs`, SYSCALL vector 0x80
5. Redox OS syscall ABI (Rust `syscall` crate, TRAP frame, `context_switch`) — gitlab.redox-os.org/redox-os/kernel
6. Theseus OS syscall model (inter-process notification, capability-based) — theseus-os.github.io/Theseus/book
7. Signal delivery: `sigframe` construction, `sighand_struct`, do_signal() — linux kernel
8. POSIX signal semantics: pending/masked, `SA_RESTART`, `SA_SIGINFO`, real-time signals
9. Linux `arch/x86/kernel/signal.c` – `setup_rt_frame`, `restore_sigcontext`, signal trampoline
10. Writing a syscall handler in Rust: `#[naked]` wrapper, `asm!` preserve callee-saved regs, return via `sysret`

### Partitioning (GPT/MBR) & Boot Protocols (~10 sources)

1. UEFI Spec Ch 5: GPT partition table (protective MBR, partition entry array, LBA 1–33)
2. MBR partition table (legacy, 4 primary entries, EBR logicals, CHS vs LBA)
3. OSDev Wiki GPT — wiki.osdev.org/GPT
4. Multiboot2 Specification v1.0 (boot info tags, ELF sections, memory map, RSDP) — gnu.org/software/grub/manual/multiboot2
5. Limine Boot Protocol v6 (page tables, HHDM, SMP wakeup, stivale2) — github.com/limine-bootloader/limine
6. STIVALE2 Boot Protocol (Limine's simpler protocol, `stivale2_struct`, MMAP tags) — github.com/stivale/stivale
7. Bootloader crate `0.9.x` vs `0.11.x` differences: `BootInfo` tag structure, `map_physical_memory` rename
8. initramfs/initrd creation: `cpio` newc format, embedded in kernel ELF section
9. `objcopy` embedding files as ELF symbols (`.incbin` in linker, `extern` symbols in Rust)
10. Theseus OS PXE + network boot chain (UDP, TFTP, DHCP, wake-on-LAN) — theseus-os.github.io/Theseus/book

---

### PIT / HPET / TSC (Timers) (~10 sources)

1. OSDev Wiki PIT (8253/8254, counter 0–2, Mode 2/3, IRQ 0, I/O 0x40–0x43) — wiki.osdev.org/PIT
2. OSDev Wiki HPET (ACPI HPET table, main counter, comparators, IRQ routing, 10+ MHz) — wiki.osdev.org/HPET
3. Intel SDM Vol 3B Ch 17: Time-Stamp Counter (TSC, `RDTSC`/`RDTSCP`, `IA32_TSC` MSR 10H, invariant TSC)
4. Linux kernel `clocksource` framework (`clocksource` struct, rating/rank, `timekeeping_init`) — kernel.org/doc/Documentation/timers
5. Linux `drivers/clocksource/acpi_pm.c` (PMTMR, ACPI FADT PM_TMR_BLK, 3.58 MHz)
6. QEMU `-icount shift=auto` + `-rtc base=localtime` / `-no-reboot` for deterministic TSC emulation
7. OSDev Wiki RDTSC — wiki.osdev.org/RDTSC
8. x86 `LAPIC timer` (programmable one-shot/periodic at bus freq, `divide configuration register`)
9. Theseus OS HPET-based scheduler tick (oneshot HPET comparator reprogrammed per tick)
10. Write an OS: HPET & Time (using HPET as monotonic system timer) — taylor.town/write-an-os-hpet

### VFS / Device Model / Driver Model (~10 sources)

1. Linux VFS architecture: superblock, inode, dentry, file operations (`file_operations`, `inode_operations`) — tldp.org/LDP/khg/HyperNews/get/fs/vfstour.html
2. Linux `struct file_operations` (`open`, `read`, `write`, `mmap`, `ioctl`, `release`)
3. Linux `struct device_driver` + `struct bus_type` + `struct class` (driver core, sysfs integration)
4. Linux `platform_driver` framework (`probe`/`remove`, DT match, ACPI match, hotplug)
5. Redox OS `Scheme`/`SchemeBlock` (VFS interface, `SYS_OPEN`/`SYS_READ`/`SYS_WRITE` via kernel channels) — doc.redox-os.org
6. Theseus OS device manager (PCI enumeration → device tree, driver binding by VID/PID)
7. FreeBSD `device_attach()` / `driver_t` / `devclass_t` / NEWBUS (hierarchical device tree, resource management)
8. Writing a driver model in Rust (trait-based, type-safe, `DriverOps` trait for probe/bind):
9. `ioctl` implementation (cmd direction `_IOR`/`_IOW`/`_IOWR`, struct sizing via `_IOC_SIZE`)
10. Theseus OS `Device` trait + `DeviceIo` (memory-mapped I/O, port I/O abstraction) — theseus-os.github.io/Theseus/book

### Kernel Modules / Dynamic Loading (~10 sources)

1. Linux loadable kernel module (LKM) framework: `init_module`, `cleanup_module`, `MODULE_LICENSE`, `.ko` ELF
2. Linux `insmod` / `modprobe` internals: `finit_module`, `sys_init_module`, kernel module loader
3. OSDev Wiki Kernel Module — wiki.osdev.org/Kernel_Module
4. ELF object file relocation at load time (`rela`/`rel` sections, `SHT_RELA`, `DT_NEEDED`)
5. Linux `module_init` / `__init` macro, `MODULE_DEVICE_TABLE`, `module_param`
6. Writing a kernel module system (Rust): `#[module_init]` attribute, `extern "C"`, `ModuleOps` trait
7. Redox OS daemon + scheme model (userspace drivers via namespace, no kernel LKM needed)
8. Theseus OS loaded-cell model (cell relocation via `load_cell`, `link_cell`, borrow-checked) — theseus-os.github.io/Theseus/book
9. ToaruOS kernel module loader (ELF symbols, module list, `/dev/module`) — github.com/klange/toaruos
10. GNU ld `-r` (relocatable link) for combining kernel + modules, `objcopy` for symbol export

### Terminal / Console / Framebuffer Console (~10 sources)

1. OSDev Wiki VGA Text Mode (ports 0x3D4/0x3D5, cursor shape/position, page flipping) — wiki.osdev.org/Text_Mode_Cursor
2. OSDev Wiki VGA Hardware (sequencer, CRT controller, graphics controller, attribute controller)
3. ANSI escape sequences reference (CSI `ESC[`, SGR, cursor movement, `ED`/`EL`) — gnu.org/software/screen/manual/ansi-escape
4. VT100/xterm terminal emulation: `RIS`, `DECSET`, `DECRST`, alternate screen buffer — vt100.net
5. Linux `vt.c` (virtual console, keyboard translation, scrollback, `con_write`) — drivers/tty/vt/
6. Linux `fbcon.c` (framebuffer console, `fbcon_ops`, font rendering, scroll acceleration)
7. Linux `linux/kd.h` / `KDSETMODE` / `KD_TEXT` / `KD_GRAPHICS` (console mode switch)
8. `lat9x16-12` / `sun12x22` font formats (PSF v1/v2, Linux console font loader) — wiki.osdev.org/PC_Screen_Font
9. Theseus OS terminal driver (TTY abstraction, line discipline, `write`/`read` buffering)
10. ToaruOS terminal emulator (`terminal.c`, PSF font, ANSI sequences, window manager integration)

**Running total: ~666 sources**
