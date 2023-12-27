use std::{time::Duration, io::{self, Error}};

use nix::{time::{ClockId, clock_gettime}, sys::time::TimeSpec};
pub struct RtClock(TimeSpec);

const CLOCK: ClockId = ClockId::CLOCK_MONOTONIC_RAW;
const PRIO_DEFAULT: i32 = 10;

/// Clock based on `clock_gettime` in Linux.
/// Precision: ~1ns (interpolated); Performance: <100ns/call
#[cfg(target_os = "linux")]
impl RtClock {
    pub fn now() -> Self {
        let res = clock_gettime(CLOCK).unwrap();
        RtClock(res)
    }

    pub fn elapsed(&self) -> u64 {
        let ts = clock_gettime(CLOCK).unwrap();
        ts.tv_nsec() as u64 - self.0.tv_nsec() as u64
    }
}

pub fn schedule_self_fifo() -> io::Result<()> {
    scheduler::set_self_policy(scheduler::Policy::Fifo, PRIO_DEFAULT).map_err(|_| {
        let error = Error::last_os_error();
        io::Error::new(error.kind(), error)
    })
}

fn main() {
    sudo::escalate_if_needed().unwrap();
    schedule_self_fifo().expect("Not able to set FIFO scheduler");
    // Create a new sleeper that trusts native thread::sleep with 100μs accuracy
    let spin_sleeper = spin_sleep::SpinSleeper::new(500_000)
        .with_spin_strategy(spin_sleep::SpinStrategy::SpinLoopHint);
    loop {
        let clk = RtClock::now();
        spin_sleeper.sleep(Duration::new(1, 12_550_000));
        let elapsed = clk.elapsed();
        println!("Sleeped {}us (diff: {}us)", elapsed / 1000, (elapsed as i64 - 12_550_000) / 1000);
    }
}