use std::{mem, ops::Neg, ptr::null, sync::OnceLock, time::Duration};
use windows_sys::Win32::{
    Foundation::{CloseHandle, FALSE},
    Media::{timeBeginPeriod, timeEndPeriod, timeGetDevCaps, TIMECAPS, TIMERR_NOERROR},
    System::Threading::{
        CreateWaitableTimerExW, SetWaitableTimer, WaitForSingleObject,
        CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, INFINITE, TIMER_ALL_ACCESS,
    },
};

#[inline]
pub fn native_sleep(duration: Duration) {
    if high_res_sleep(&duration).is_err() {
        // fallback for OS earlier than Windows 10, version 1803.
        let min_time_period = min_time_period();
        unsafe {
            timeBeginPeriod(min_time_period);
            std::thread::sleep(duration);
            timeEndPeriod(min_time_period);
        }
    }
}

#[inline]
pub(crate) fn sleep_accuracy() -> u32 {
    if HIGH_RES_TIMER.with(|t| t.is_ok()) {
        // high resolution timer is fast on average but has high maximums
        // e.g. `average: 154.7µs, best: 2.6µs, worst: 729.5µs`
        //
        // 500-1000µs accuracy should mostly eliminate over-sleeps except under load.
        700_000
    } else {
        min_time_period() * 1_000_000
    }
}

/// Minimum time period for use with `timeBeginPeriod` & `timeEndPeriod`.
fn min_time_period() -> u32 {
    static MIN_TIME_PERIOD: OnceLock<u32> = OnceLock::new();

    *MIN_TIME_PERIOD.get_or_init(|| unsafe {
        let tc_size = mem::size_of::<TIMECAPS>() as u32;
        let mut tc = TIMECAPS {
            wPeriodMin: 0,
            wPeriodMax: 0,
        };

        if timeGetDevCaps(&mut tc as *mut TIMECAPS, tc_size) == TIMERR_NOERROR {
            tc.wPeriodMin
        } else {
            1
        }
    })
}

thread_local! {
    static HIGH_RES_TIMER: Result<WaitableTimer, ()> = WaitableTimer::try_high_resolution();
}

#[inline]
fn high_res_sleep(duration: &Duration) -> Result<(), ()> {
    HIGH_RES_TIMER.with(|timer| {
        let timer = timer.as_ref().map_err(|_| ())?;
        timer.set(duration)?;
        timer.wait()
    })
}

struct WaitableTimer {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

impl WaitableTimer {
    /// Create a high-resolution timer. Will fail before Windows 10, version 1803.
    fn try_high_resolution() -> Result<Self, ()> {
        let handle = unsafe {
            CreateWaitableTimerExW(
                null(),
                null(),
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                TIMER_ALL_ACCESS,
            )
        };
        match handle.is_null() {
            true => Err(()),
            _ => Ok(Self { handle }),
        }
    }

    fn set(&self, duration: &Duration) -> Result<(), ()> {
        // Convert the Duration to a format similar to FILETIME.
        // Negative values are relative times whereas positive values are absolute.
        // Therefore we negate the relative duration.
        let time = checked_dur2intervals(duration).ok_or(())?.neg();
        match unsafe { SetWaitableTimer(self.handle, &time, 0, None, null(), FALSE) } {
            0 => Err(()),
            _ => Ok(()),
        }
    }

    fn wait(&self) -> Result<(), ()> {
        match unsafe { WaitForSingleObject(self.handle, INFINITE) } {
            windows_sys::Win32::Foundation::WAIT_FAILED => Err(()),
            _ => Ok(()),
        }
    }
}

impl Drop for WaitableTimer {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle) };
    }
}

fn checked_dur2intervals(dur: &Duration) -> Option<i64> {
    const NANOS_PER_SEC: u64 = 1_000_000_000;
    const INTERVALS_PER_SEC: u64 = NANOS_PER_SEC / 100;

    dur.as_secs()
        .checked_mul(INTERVALS_PER_SEC)?
        .checked_add(dur.subsec_nanos() as u64 / 100)?
        .try_into()
        .ok()
}
