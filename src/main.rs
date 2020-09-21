#![no_std]
#![no_main]

extern crate rlibc;

mod vga_buffer;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // entry point
    println!("Hello world! {}", " :>");

    panic!("Test panic message");

    //loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}
