#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]

#![no_std]

#[cfg(target_os="dos")]
use core::arch::asm;
use core::mem::replace;
use educe::Educe;
#[cfg(all(not(target_os="dos"), not(windows)))]
use libc::{c_long, time_t};
use num_traits::Num;
#[cfg(target_os="dos")]
use pc_timer::Timer;

#[cfg(not(target_os="dos"))]
pub struct MonoClock(());

#[cfg(target_os="dos")]
pub struct MonoClock(Timer);

impl MonoClock {
    /// # Safety
    ///
    /// This function may not be called while another [`MonoClock`] instance is alive.
    ///
    /// Also, if compiled with `cfg(target_os="dos")` it should be guaranteed that
    /// it is executing on an effectively single-core processor.
    pub unsafe fn new() -> Self {
        Self::new_raw()
    }

    #[cfg(not(target_os="dos"))]
    unsafe fn new_raw() -> Self {
        MonoClock(())
    }

    #[cfg(target_os="dos")]
    unsafe fn new_raw() -> Self {
        MonoClock(Timer::new(125))
    }

    pub fn sleep_ms_u8(&self, ms: u8) { self.sleep_ms(ms); }

    pub fn sleep_ms_u16(&self, ms: u16) { self.sleep_ms(ms); }

    pub fn sleep_ms_u32(&self, ms: u32) { self.sleep_ms(ms); }

    pub fn sleep_ms_u64(&self, ms: u64) { self.sleep_ms(ms); }

    #[cfg(target_os="dos")]
    pub fn time(&self) -> MonoTime {
        MonoTime {
            ticks: self.0.ticks(),
            clock: self
        }
    }

    #[cfg(all(not(target_os="dos"), windows))]
    pub fn time(&self) -> MonoTime {
        use winapi::um::sysinfoapi::GetTickCount64;

        MonoTime {
            ticks: unsafe { GetTickCount64() },
            clock: self
        }
    }

    #[cfg(all(not(target_os="dos"), not(windows)))]
    pub fn time(&self) -> MonoTime {
        use core::mem::MaybeUninit;
        use libc::{CLOCK_MONOTONIC, clock_gettime};

        let mut time = MaybeUninit::uninit();
        assert_eq!(unsafe { clock_gettime(CLOCK_MONOTONIC, time.as_mut_ptr()) }, 0);
        let time = unsafe { time.assume_init() };
        MonoTime {
            s: time.tv_sec,
            ms: (time.tv_nsec / 1_000_000) as i16,
            clock: self
        }
    }

    #[cfg(target_os="dos")]
    #[inline]
    fn sleep_ms<T: Num + Copy>(&self, mut ms: T) where u64: TryInto<T>, T: TryInto<u64> {
        while ms != T::zero() {
            let (sleep, last) = if let Ok(ms_u64) = ms.try_into() {
                (ms_u64, T::zero())
            } else {
                (u64::MAX, ms - u64::MAX.try_into().unwrap_or_else(|_| unreachable!()))
            };
            let start = self.time();
            loop {
                if self.time().delta_ms_u64(start).unwrap() > sleep { break; }
                for _ in 0 .. 64 {
                    unsafe { asm!("nop"); }
                }
            }
            ms = last;
        }
    }

    #[cfg(all(not(target_os="dos"), windows))]
    #[inline]
    fn sleep_ms<T: Num + Copy>(&self, mut ms: T) where u32: TryInto<T>, T: TryInto<u32> {
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

    #[cfg(all(not(target_os="dos"), not(windows)))]
    #[inline]
    fn sleep_ms<T: Num + Copy>(
        &self,
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
}

#[cfg(any(target_os="dos", windows))]
#[derive(Educe, Clone, Copy)]
#[educe(Debug)]
pub struct MonoTime<'a> {
    ticks: u64,
    #[educe(Debug(ignore))]
    clock: &'a MonoClock,
}

#[cfg(all(not(target_os="dos"), not(windows)))]
#[derive(Educe, Clone, Copy)]
#[educe(Debug)]
pub struct MonoTime<'a> {
    s: time_t,
    ms: i16,
    #[educe(Debug(ignore))]
    clock: &'a MonoClock,
}

impl<'a> MonoTime<'a> {
    pub fn delta_ms_u8(self, prev: MonoTime) -> Option<u8> { self.delta_ms(prev) }

    pub fn delta_ms_u16(self, prev: MonoTime) -> Option<u16> { self.delta_ms(prev) }

    pub fn delta_ms_u32(self, prev: MonoTime) -> Option<u32> { self.delta_ms(prev) }

    pub fn delta_ms_u64(self, prev: MonoTime) -> Option<u64> { self.delta_ms(prev) }

    pub fn delta_ms_u128(self, prev: MonoTime) -> Option<u128> { self.delta_ms(prev) }

    pub fn split_ms_u8(&mut self) -> Option<u8> {
        let prev = replace(self, self.clock.time());
        self.delta_ms_u8(prev)
    }

    pub fn split_ms_u16(&mut self) -> Option<u16> {
        let prev = replace(self, self.clock.time());
        self.delta_ms_u16(prev)
    }

    pub fn split_ms_u32(&mut self) -> Option<u32> {
        let prev = replace(self, self.clock.time());
        self.delta_ms_u32(prev)
    }

    pub fn split_ms_u64(&mut self) -> Option<u64> {
        let prev = replace(self, self.clock.time());
        self.delta_ms_u64(prev)
    }

    pub fn split_ms_u128(&mut self) -> Option<u128> {
        let prev = replace(self, self.clock.time());
        self.delta_ms_u128(prev)
    }

    #[cfg(target_os="dos")]
    #[inline]
    fn delta_ms<T>(self, prev: MonoTime) -> Option<T> where u64: TryInto<T> {
        self.ticks.wrapping_sub(prev.ticks).wrapping_mul(8).try_into().ok()
    }

    #[cfg(all(not(target_os="dos"), windows))]
    #[inline]
    fn delta_ms<T>(self, prev: MonoTime) -> Option<T> where u64: TryInto<T> {
        self.ticks.checked_sub(prev.ticks).unwrap().try_into().ok()
    }

    #[cfg(all(not(target_os="dos"), not(windows)))]
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
}
