#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]

#![no_std]

#[cfg(not(windows))]
use libc::{c_long, time_t};
use num_traits::Num;

#[cfg(windows)]
#[derive(Debug, Clone, Copy)]
pub struct MonoTime(u64);

#[cfg(not(windows))]
#[derive(Debug, Clone, Copy)]
pub struct MonoTime { s: time_t, ms: i16 }

impl MonoTime {
    pub fn delta_ms_u8(self, prev: MonoTime) -> Option<u8> { self.delta_ms(prev) }

    pub fn delta_ms_u16(self, prev: MonoTime) -> Option<u16> { self.delta_ms(prev) }

    pub fn delta_ms_u32(self, prev: MonoTime) -> Option<u32> { self.delta_ms(prev) }

    pub fn delta_ms_u64(self, prev: MonoTime) -> Option<u64> { self.delta_ms(prev) }

    pub fn delta_ms_u128(self, prev: MonoTime) -> Option<u128> { self.delta_ms(prev) }

    #[cfg(windows)]
    #[inline]
    fn delta_ms<T>(self, prev: MonoTime) -> Option<T> where u64: TryInto<T> {
        self.0.checked_sub(prev.0).unwrap().try_into().ok()
    }

    #[cfg(not(windows))]
    #[inline]
    fn delta_ms<T: Num + num_traits::Bounded + Ord + Copy>(
        self,
        prev: MonoTime
    ) -> Option<T> where i16: TryInto<T>, time_t: TryInto<T> {
        let mut s = self.s.checked_sub(prev.s).unwrap();
        assert!(s >= 0);
        let mut ms = self.ms - prev.ms;
        if ms < 0 {
            s = s.checked_sub(1).unwrap();
            ms += 1000;
            debug_assert!(ms >= 0);
        }
        let ms: T = ms.try_into().ok()?;
        let s: T = s.try_into().ok()?;
        let thousand: T = if let Ok(thousand) = 1000i16.try_into() {
            thousand
        } else {
            if s != T::zero() { return None; }
            T::one()
        };
        if T::max_value() / thousand < s { return None; }
        let s = s * thousand;
        if T::max_value() - s < ms { return None; }
        Some(s + ms)
    }

    #[cfg(windows)]
    pub fn get() -> Self {
        use winapi::um::sysinfoapi::GetTickCount64;

        MonoTime(unsafe { GetTickCount64() })
    }

    #[cfg(not(windows))]
    pub fn get() -> Self {
        use core::mem::MaybeUninit;
        use libc::{CLOCK_MONOTONIC, clock_gettime};

        let mut time = MaybeUninit::uninit();
        assert_eq!(unsafe { clock_gettime(CLOCK_MONOTONIC, time.as_mut_ptr()) }, 0);
        let time = unsafe { time.assume_init() };
        MonoTime { s: time.tv_sec, ms: (time.tv_nsec / 1_000_000) as i16 }
    }
}

pub fn sleep_ms_u8(ms: u8) { sleep_ms(ms); }

pub fn sleep_ms_u16(ms: u16) { sleep_ms(ms); }

pub fn sleep_ms_u32(ms: u32) { sleep_ms(ms); }

pub fn sleep_ms_u64(ms: u64) { sleep_ms(ms); }

pub fn sleep_ms_u128(ms: u128) { sleep_ms(ms); }

#[cfg(windows)]
#[inline]
fn sleep_ms<T: Num + Copy>(mut ms: T) where u32: TryInto<T>, T: TryInto<u32> {
    use winapi::um::synchapi::Sleep;

    while ms != T::zero() {
        let (sleep, last) = if let Ok(ms_u32) = ms.try_into() {
            (ms_u32, T::zero())
        } else {
            (u32::MAX, ms - u32::MAX.try_into().unwrap_or_else(|_| unreachable!()))
        };
        unsafe { Sleep(sleep); }
        ms = last;
    }
}

#[cfg(not(windows))]
#[inline]
fn sleep_ms<T: Num + Copy>(
    mut ms: T
) where u16: TryInto<T>, T: TryInto<time_t>, time_t: TryInto<T>, T: TryInto<c_long> {
    use core::ptr::null_mut;
    use libc::{nanosleep, timespec};

    while ms != T::zero() {
        let (sleep_s, sleep_ms, last): (time_t, c_long, T) = if let Ok(thousand) = 1000u16.try_into() {
            let s = ms / thousand;
            if let Ok(s_time_t) = s.try_into() {
                (
                    s_time_t,
                    (ms % thousand).try_into().unwrap_or_else(|_| unreachable!()),
                    T::zero()
                )
            } else {
                (
                    time_t::MAX,
                    (ms % thousand).try_into().unwrap_or_else(|_| unreachable!()),
                    (s - time_t::MAX.try_into().unwrap_or_else(|_| unreachable!())) * thousand 
                )
            }
        } else {
            (0, ms.try_into().unwrap_or_else(|_| unreachable!()), T::zero())
        };
        let sleep = timespec { tv_sec: sleep_s, tv_nsec: sleep_ms * 1_000_000 };
        unsafe { nanosleep(&sleep as *const _, null_mut()); }
        ms = last;
    }
}
