// Global Descriptor Table + Task State Segment.
//
// In 64-bit mode segmentation is mostly disabled, but we still need:
//  - a code segment with the L (long mode) bit set,
//  - a data segment,
//  - a TSS, which holds the IST stacks used by exceptions
//    that must run on a known-good stack (e.g. double fault).

use core::arch::asm;
use core::mem::size_of;

/// 64-bit Task State Segment (104 bytes).
#[repr(C, packed)]
struct Tss {
    _r0: u32,
    rsp: [u64; 3],   // stacks for privilege level changes (RSP0..RSP2)
    _r1: u64,
    ist: [u64; 7],   // Interrupt Stack Table (IST1..IST7)
    _r2: u64,
    _r3: u16,
    iomap_base: u16, // set to size_of::<Tss>() = "no I/O bitmap"
}

/// Pointer structure loaded by the `lgdt` instruction.
#[repr(C, packed)]
struct GdtPtr {
    limit: u16, // size of the table minus 1
    base: u64,  // linear address of the table
}

// Layout: null, kernel code, kernel data, TSS (a TSS descriptor takes 2 slots)
static mut GDT: [u64; 5] = [
    0,
    0x00AF_9A00_0000_FFFF, // kernel code: present, ring 0, executable, L=1
    0x00CF_9200_0000_FFFF, // kernel data: present, ring 0, writable
    0,                     // TSS descriptor (low half), filled in init()
    0,                     // TSS descriptor (high half), filled in init()
];

static mut TSS: Tss = Tss {
    _r0: 0,
    rsp: [0; 3],
    _r1: 0,
    ist: [0; 7],
    _r2: 0,
    _r3: 0,
    iomap_base: size_of::<Tss>() as u16,
};

/// Dedicated stack for the double fault handler (16 KiB, 16-byte aligned).
#[repr(align(16))]
struct Stack([u8; 16 * 1024]);
static mut DOUBLE_FAULT_STACK: Stack = Stack([0; 16 * 1024]);

// Segment selectors = index in the GDT * 8
pub const KERNEL_CS: u16 = 0x08;
pub const KERNEL_DS: u16 = 0x10;
const TSS_SEL: u16 = 0x18;

pub fn init() {
    unsafe {
        // IST1 -> top of the double fault stack (stacks grow downwards)
        let stack_top = core::ptr::addr_of!(DOUBLE_FAULT_STACK) as u64 + 16 * 1024;
        let ist_ptr = core::ptr::addr_of_mut!(TSS.ist) as *mut u64;
        ist_ptr.write_unaligned(stack_top);

        // Build the 16-byte TSS descriptor
        let base = core::ptr::addr_of!(TSS) as u64;
        let limit = (size_of::<Tss>() - 1) as u64;
        let low = (limit & 0xFFFF)
            | ((base & 0xFF_FFFF) << 16)
            | (0x89u64 << 40)                 // present, type = 64-bit TSS (available)
            | (((limit >> 16) & 0xF) << 48)
            | (((base >> 24) & 0xFF) << 56);
        let high = base >> 32;
        GDT[3] = low;
        GDT[4] = high;

        // Load the new GDT
        let ptr = GdtPtr {
            limit: (size_of::<[u64; 5]>() - 1) as u16,
            base: core::ptr::addr_of!(GDT) as u64,
        };
        asm!("lgdt [{}]", in(reg) &ptr as *const GdtPtr, options(readonly, nostack));

        // Reload CS with a far return (CS cannot be written with `mov`),
        // then reload the data segment registers.
        asm!(
            "push {cs}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",                // pops RIP and CS
            "2:",
            "mov ds, {ds:x}",
            "mov es, {ds:x}",
            "mov ss, {ds:x}",
            "xor {tmp:e}, {tmp:e}", // FS/GS = null selector
            "mov fs, {tmp:x}",
            "mov gs, {tmp:x}",
            cs = in(reg) KERNEL_CS as u64,
            ds = in(reg) KERNEL_DS as u64,
            tmp = out(reg) _,
        );

        // Load the task register with the TSS selector
        asm!("ltr {0:x}", in(reg) TSS_SEL, options(nostack));
    }
}
