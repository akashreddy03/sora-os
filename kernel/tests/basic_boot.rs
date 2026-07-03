#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sora_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use sora_os::test_panic_handler;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[test_case]
fn test_after_boot() {
    assert_eq!(1, 1);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}
