#![feature(extern_types)]

//#![deny(warnings)]

#![windows_subsystem="console"]
#![no_std]
#![no_main]

extern crate dos_errno_and_panic;
extern crate pc_atomics;
extern crate rlibc;

mod no_std {
    #[no_mangle]
    extern "C" fn _aulldiv() -> ! { panic!("10") }
    #[no_mangle]
    extern "C" fn _aullrem() -> ! { panic!("11") }
    #[no_mangle]
    extern "C" fn _chkstk() { }
    #[no_mangle]
    extern "C" fn _fltused() -> ! { panic!("13") }
    #[no_mangle]
    extern "C" fn strlen() -> ! { panic!("14") }
}

extern {
    type PEB;
}

use core::cmp::max;
use dos_cp::println;
use timer_no_std::MonoClock;

#[allow(non_snake_case)]
#[no_mangle]
extern "stdcall" fn mainCRTStartup(_: *const PEB) -> u64 {
    let clock = unsafe { MonoClock::new() };
    let mut seconds = 1;
    let start = clock.time();
    loop {
        let now = clock.time();
        let wait = now.delta_ms_u16(start)
            .and_then(|x| i16::try_from(x).ok())
            .map_or(0, |passed| max(0, seconds * 1000 - passed));
        if wait == 0 {
            if seconds == 11 {
                break;
            } else {
                println!("{}!", 11 - seconds);
            }
            seconds += 1;
        } else {
            //println!("{wait}");
            clock.sleep_ms_u16(wait.try_into().unwrap());
            panic!("XXX");
        }
    }
    0
}
