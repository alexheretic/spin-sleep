use std::{mem, sync::OnceLock, time::Duration};
use winapi::{
    shared::minwindef::UINT,
    um::{
        mmsystem::{TIMECAPS, TIMERR_NOERROR},
        timeapi::{timeBeginPeriod, timeEndPeriod, timeGetDevCaps},
    },
};

#[inline]
pub fn native_sleep(duration: Duration) {
    let min_time_period = win_min_time_period();
    unsafe {
        timeBeginPeriod(min_time_period);
        std::thread::sleep(duration);
        timeEndPeriod(min_time_period);
    }
}

fn win_min_time_period() -> UINT {
    static MIN_TIME_PERIOD: OnceLock<UINT> = OnceLock::new();

    *MIN_TIME_PERIOD.get_or_init(|| {
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
