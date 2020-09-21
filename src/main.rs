#![no_std]
#![no_main]

extern crate rlibc;

use core::panic::PanicInfo;

static HELLO_MESSAGE: &[u8] = b"Hello World! :)";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // entry point

    let vga_buffer = 0xb8000 as *mut u8; // vga text buffer location

    // print the hello message
    for (i, &byte) in HELLO_MESSAGE.iter().enumerate() {
        // raw pointers can be used only in unsafe block
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte; // character
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb; // color byte: cyan
        }
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
