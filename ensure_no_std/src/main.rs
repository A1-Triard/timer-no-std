#![feature(start)]

#![deny(warnings)]

#![no_std]

use core::panic::PanicInfo;
use exit_no_std::exit;
use timer_no_std::{MonoTime, sleep_ms_u8};

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

#[panic_handler]
pub extern fn panic(_info: &PanicInfo) -> ! {
    exit(99)
}

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let _ = MonoTime::get();
    sleep_ms_u8(100);
    exit(0);
}
