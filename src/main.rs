#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(first_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use first_os::println;
use bootloader::{BootInfo, entry_point};

entry_point!(kernel_entry);

fn kernel_entry(_boot_info: &'static BootInfo) -> ! {
    // entry point
    println!("Hello world!");

    first_os::init();

    #[cfg(test)]
    test_main();

    first_os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    first_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    first_os::test_panic_handler(info);
}

#[test_case]
fn test_assertion() {
    assert_eq!(1, 1);
}
