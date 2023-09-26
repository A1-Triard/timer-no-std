#![feature(extern_types)]

#![deny(warnings)]

#![windows_subsystem="console"]
#![no_std]
#![no_main]

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


    #[panic_handler]
    fn panic_handler(info: &core::panic::PanicInfo) -> ! { panic_no_std::panic(info, b'P') }
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
    let mut seconds = 0;
    let start = clock.time();
    loop {
        let now = clock.time();
        let wait = now.delta_ms_u16(start)
            .and_then(|x| i16::try_from(x).ok())
            .map_or(0, |passed| max(0, seconds * 1000 - passed));
        if wait == 0 {
            if seconds == 10 {
                break;
            } else {
                println!("{}!", 10 - seconds);
            }
            seconds += 1;
        } else {
            clock.sleep_ms_u16(wait.try_into().unwrap());
        }
    }
    0
}
