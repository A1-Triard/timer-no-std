#![feature(start)]

#![deny(warnings)]

#![no_std]

use core::panic::PanicInfo;
use exit_no_std::exit;
use timer_no_std::MonoClock;

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(99)
}

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let clock = unsafe { MonoClock::new() };
    let _ = clock.time();
    clock.sleep_ms_u8(100);
    exit(0);
}
