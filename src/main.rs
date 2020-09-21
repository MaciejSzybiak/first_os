#![no_std]
#![no_main]

extern crate rlibc;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // entry point
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
