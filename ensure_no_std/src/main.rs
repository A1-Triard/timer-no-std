#![feature(start)]

#![deny(warnings)]

#![no_std]

use core::panic::PanicInfo;
use exit_no_std::exit;
use timer_no_std::{get_monotonic_milliseconds, sleep_milliseconds};

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

#[panic_handler]
pub extern fn panic(_info: &PanicInfo) -> ! {
    exit(99)
}

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let _ = get_monotonic_milliseconds();
    sleep_milliseconds(100);
    exit(0);
}
