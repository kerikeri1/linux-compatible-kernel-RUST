#![no_std]
#![no_main]

mod gdt;
mod idt;
mod serial;

use core::arch::asm;
use core::panic::PanicInfo;
use limine::BaseRevision;

// Tells Limine which protocol revision we support.
// The .requests section is collected by the linker script.
#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

/// Recurses forever, consuming stack until it runs out.
/// The 512-byte buffer makes each frame big enough to hit the limit quickly,
/// and black_box stops the compiler from optimizing the buffer away.
#[allow(unconditional_recursion)]
fn overflow(n: u64) -> u64 {
    let buf = [n as u8; 512];
    core::hint::black_box(&buf);
    overflow(n + 1) + buf[0] as u64
}

#[no_mangle]
extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    serial::init();
    println!("Hello from ferrux!");

    // Install our own GDT/TSS first, then the IDT that references it
    gdt::init();
    idt::init();
    println!("GDT and IDT loaded");

    // Exception tests: enable ONE line at a time.

    // Double fault (#8): stack overflow, handled on the IST1 stack
    // overflow(0);

    // unsafe { asm!("ud2") };                                                   // invalid opcode (#6)
    // unsafe { core::ptr::read_volatile(0xdead_beef as *const u8); }            // page fault (#14)
    // unsafe { asm!("xor edx, edx", "div edx", out("eax") _, out("edx") _) };   // divide error (#0)

    // Deterministic double fault: break RSP, then raise any exception.
    // The CPU can't push the exception frame, so the fault escalates to #8.
    unsafe { asm!("mov rsp, 0", "ud2") };

    hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {info}");
    hlt_loop()
}

fn hlt_loop() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}
