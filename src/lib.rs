#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]

#![no_std]

#[cfg(windows)]
pub fn sleep_milliseconds(mut ms: u128) {
    use winapi::um::synchapi::Sleep;

    while ms != 0 {
        let last = ms.saturating_sub(u32::MAX as u128);
        unsafe { Sleep((ms - last) as u32); }
        ms = last;
    }
}

#[cfg(not(windows))]
pub fn sleep_milliseconds(mut ms: u128) {
    use core::ptr::null_mut;
    use libc::{nanosleep, timespec, time_t};

    while ms != 0 {
        let last = ms.saturating_sub(time_t::MAX as i128 as u128 * 1000);
        let wait = ms - last;
        let wait = timespec {
            tv_sec: (wait / 1000) as i128 as time_t,
            tv_nsec: ((wait % 1000) * 1_000_000) as i128 as _
        };
        unsafe { nanosleep(&wait as *const _, null_mut()); }
        ms = last;
    }
}

#[cfg(windows)]
pub fn get_monotonic_milliseconds() -> u128 {
    use winapi::um::sysinfoapi::GetTickCount64;

    (unsafe { GetTickCount64() }) as u128
}

#[cfg(not(windows))]
pub fn get_monotonic_milliseconds() -> u128 {
    use core::mem::MaybeUninit;
    use libc::{CLOCK_MONOTONIC, clock_gettime};

    let mut time = MaybeUninit::uninit();
    assert_eq!(unsafe { clock_gettime(CLOCK_MONOTONIC, time.as_mut_ptr()) }, 0);
    let time = unsafe { time.assume_init() };
    (time.tv_sec as i128 as u128) * 1_000 + (time.tv_nsec as i128 as u128) / 1_000_000
}
