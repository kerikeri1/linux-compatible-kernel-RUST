// Interrupt Descriptor Table and CPU exception handling (vectors 0-31).
//
// Each vector gets a tiny assembly stub that normalizes the stack layout
// (pushes a fake error code when the CPU does not push one), then jumps to a
// common routine that saves all general purpose registers and calls the Rust
// handler with a pointer to the saved frame.

use core::arch::{asm, global_asm};
use core::mem::size_of;

use crate::gdt::KERNEL_CS;

/// One 16-byte IDT entry (interrupt gate).
#[repr(C)]
#[derive(Clone, Copy)]
struct Entry {
    offset_low: u16,
    selector: u16,
    ist: u8,        // IST index (0 = don't switch stack)
    flags: u8,      // present, DPL, gate type
    offset_mid: u16,
    offset_high: u32,
    _zero: u32,
}

impl Entry {
    const MISSING: Entry = Entry {
        offset_low: 0, selector: 0, ist: 0, flags: 0,
        offset_mid: 0, offset_high: 0, _zero: 0,
    };

    fn new(handler: u64, ist: u8) -> Entry {
        Entry {
            offset_low: handler as u16,
            selector: KERNEL_CS,
            ist,
            flags: 0x8E, // present, ring 0, 64-bit interrupt gate
            offset_mid: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            _zero: 0,
        }
    }
}

/// Pointer structure loaded by the `lidt` instruction.
#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

static mut IDT: [Entry; 256] = [Entry::MISSING; 256];

// Assembly stubs.
// STUB_NOERR: the CPU pushes no error code -> we push a dummy 0 so that the
//             stack layout is identical for every vector.
// STUB_ERR:   the CPU already pushed an error code.
// In both cases we then push the vector number and jump to isr_common.
global_asm!(
    r#"
    .macro STUB_NOERR n
    .global isr_stub_\n
    isr_stub_\n:
        push 0
        push \n
        jmp isr_common
    .endm

    .macro STUB_ERR n
    .global isr_stub_\n
    isr_stub_\n:
        push \n
        jmp isr_common
    .endm

    STUB_NOERR 0
    STUB_NOERR 1
    STUB_NOERR 2
    STUB_NOERR 3
    STUB_NOERR 4
    STUB_NOERR 5
    STUB_NOERR 6
    STUB_NOERR 7
    STUB_ERR   8
    STUB_NOERR 9
    STUB_ERR   10
    STUB_ERR   11
    STUB_ERR   12
    STUB_ERR   13
    STUB_ERR   14
    STUB_NOERR 15
    STUB_NOERR 16
    STUB_ERR   17
    STUB_NOERR 18
    STUB_NOERR 19
    STUB_NOERR 20
    STUB_ERR   21
    STUB_NOERR 22
    STUB_NOERR 23
    STUB_NOERR 24
    STUB_NOERR 25
    STUB_NOERR 26
    STUB_NOERR 27
    STUB_NOERR 28
    STUB_ERR   29
    STUB_ERR   30
    STUB_NOERR 31

    isr_common:
        // Save all general purpose registers (order must match `Frame`)
        push rax
        push rbx
        push rcx
        push rdx
        push rsi
        push rdi
        push rbp
        push r8
        push r9
        push r10
        push r11
        push r12
        push r13
        push r14
        push r15
        mov rdi, rsp          // first argument: pointer to the saved frame
        cld                   // the SysV ABI requires DF = 0
        call exception_handler
    .Lhang:                   // exceptions are fatal for now: halt forever
        cli
        hlt
        jmp .Lhang
    "#
);

extern "C" {
    fn isr_stub_0(); fn isr_stub_1(); fn isr_stub_2(); fn isr_stub_3();
    fn isr_stub_4(); fn isr_stub_5(); fn isr_stub_6(); fn isr_stub_7();
    fn isr_stub_8(); fn isr_stub_9(); fn isr_stub_10(); fn isr_stub_11();
    fn isr_stub_12(); fn isr_stub_13(); fn isr_stub_14(); fn isr_stub_15();
    fn isr_stub_16(); fn isr_stub_17(); fn isr_stub_18(); fn isr_stub_19();
    fn isr_stub_20(); fn isr_stub_21(); fn isr_stub_22(); fn isr_stub_23();
    fn isr_stub_24(); fn isr_stub_25(); fn isr_stub_26(); fn isr_stub_27();
    fn isr_stub_28(); fn isr_stub_29(); fn isr_stub_30(); fn isr_stub_31();
}

pub fn init() {
    let stubs: [unsafe extern "C" fn(); 32] = [
        isr_stub_0, isr_stub_1, isr_stub_2, isr_stub_3, isr_stub_4, isr_stub_5,
        isr_stub_6, isr_stub_7, isr_stub_8, isr_stub_9, isr_stub_10, isr_stub_11,
        isr_stub_12, isr_stub_13, isr_stub_14, isr_stub_15, isr_stub_16, isr_stub_17,
        isr_stub_18, isr_stub_19, isr_stub_20, isr_stub_21, isr_stub_22, isr_stub_23,
        isr_stub_24, isr_stub_25, isr_stub_26, isr_stub_27, isr_stub_28, isr_stub_29,
        isr_stub_30, isr_stub_31,
    ];

    unsafe {
        for (i, s) in stubs.iter().enumerate() {
            // Double fault (vector 8) runs on IST1, a separate known-good stack
            let ist = if i == 8 { 1 } else { 0 };
            IDT[i] = Entry::new(*s as usize as u64, ist);
        }

        let ptr = IdtPtr {
            limit: (size_of::<[Entry; 256]>() - 1) as u16,
            base: core::ptr::addr_of!(IDT) as u64,
        };
        asm!("lidt [{}]", in(reg) &ptr as *const IdtPtr, options(readonly, nostack));
    }
}

/// Stack layout seen by the Rust handler, from the lowest address upwards:
/// the registers we pushed (r15 was pushed last, so it comes first),
/// then the vector and error code from the stub,
/// then the frame pushed by the CPU (rip, cs, rflags, rsp, ss).
#[repr(C)]
pub struct Frame {
    r15: u64, r14: u64, r13: u64, r12: u64, r11: u64, r10: u64, r9: u64, r8: u64,
    rbp: u64, rdi: u64, rsi: u64, rdx: u64, rcx: u64, rbx: u64, rax: u64,
    vector: u64,
    error: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

const NAMES: [&str; 32] = [
    "Divide Error", "Debug", "NMI", "Breakpoint", "Overflow", "Bound Range",
    "Invalid Opcode", "Device Not Available", "Double Fault", "Coprocessor Segment",
    "Invalid TSS", "Segment Not Present", "Stack-Segment Fault",
    "General Protection Fault", "Page Fault", "Reserved", "x87 FPU Error",
    "Alignment Check", "Machine Check", "SIMD FP Exception", "Virtualization",
    "Control Protection", "Reserved", "Reserved", "Reserved", "Reserved",
    "Reserved", "Reserved", "Hypervisor Injection", "VMM Communication",
    "Security Exception", "Reserved",
];

/// Called from the assembly stub. For now every exception is fatal:
/// we dump the CPU state on the serial port and the stub halts the CPU.
#[no_mangle]
extern "C" fn exception_handler(f: &Frame) {
    let name = NAMES.get(f.vector as usize).copied().unwrap_or("?");
    crate::println!("\n=== EXCEPTION {}: {} ===", f.vector, name);
    crate::println!(
        "error={:#x} rip={:#x} cs={:#x} rflags={:#x}",
        f.error, f.rip, f.cs, f.rflags
    );
    crate::println!("rsp={:#x} ss={:#x}", f.rsp, f.ss);

    // On a page fault, CR2 holds the faulting virtual address
    if f.vector == 14 {
        let cr2: u64;
        unsafe { asm!("mov {}, cr2", out(reg) cr2) };
        crate::println!("cr2 (fault address) = {:#x}", cr2);
    }

    crate::println!(
        "rax={:#x} rbx={:#x} rcx={:#x} rdx={:#x}\nrsi={:#x} rdi={:#x} rbp={:#x}\nr8={:#x} r9={:#x} r10={:#x} r11={:#x}\nr12={:#x} r13={:#x} r14={:#x} r15={:#x}",
        f.rax, f.rbx, f.rcx, f.rdx, f.rsi, f.rdi, f.rbp,
        f.r8, f.r9, f.r10, f.r11, f.r12, f.r13, f.r14, f.r15
    );
}
