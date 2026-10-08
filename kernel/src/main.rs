#![no_std]
#![no_main]

mod serial;

use core::arch::asm;
use core::panic::PanicInfo;
use limine::BaseRevision;

#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[no_mangle]
extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());
    serial::init();
    println!("Hello from ferrux!");
    hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {info}");
    hlt_loop()
}

fn hlt_loop() -> ! {
    loop { unsafe { asm!("hlt") } }
}
