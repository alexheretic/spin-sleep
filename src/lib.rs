//! Accurate sleeping. Only use native sleep as far as it can be trusted, then spin.
//!
//! The problem with `thread::sleep` is it isn't always very accurate, and this accuracy varies
//! on platform and state. Spinning is as accurate as we can get, but consumes the CPU
//! rather ungracefully.
//!
//! This library adds a middle ground, using a configurable native accuracy setting allowing
//! `thread::sleep` to wait the bulk of a sleep time, and spin the final section to guarantee
//! accuracy.
//!
//! # Example: Replace `thread::sleep`
//!
//! The simplest usage with default native accuracy is a drop in replacement for `thread::sleep`.
//! ```no_run
//! # use std::time::Duration;
//! spin_sleep::sleep(Duration::new(1, 12_550_000));
//! ```
//!
//! # Example: Configure
//! More advanced usage, including setting a custom native accuracy, can be achieved by
//! constructing a `SpinSleeper`.
//! ```no_run
//! # use std::time::Duration;
//! // Create a new sleeper that trusts native thread::sleep with 100μs accuracy
//! let spin_sleeper = spin_sleep::SpinSleeper::new(100_000)
//!     .with_spin_strategy(spin_sleep::SpinStrategy::YieldThread);
//!
//! // Sleep for 1.01255 seconds, this will:
//! //  - thread:sleep for 1.01245 seconds, i.e., 100μs less than the requested duration
//! //  - spin until total 1.01255 seconds have elapsed
//! spin_sleeper.sleep(Duration::new(1, 12_550_000));
//! ```
//!
//! Sleep can also be requested in `f64` seconds or `u64` nanoseconds
//! (useful when used with `time` crate)
//!
//! ```no_run
//! # use std::time::Duration;
//! # let spin_sleeper = spin_sleep::SpinSleeper::new(100_000);
//! spin_sleeper.sleep_s(1.01255);
//! spin_sleeper.sleep_ns(1_012_550_000);
//! ```
//!
//! OS-specific default settings should be good enough for most cases.
//! ```
//! # use spin_sleep::SpinSleeper;
//! let sleeper = SpinSleeper::default();
//! # let _ = sleeper;
//! ```
mod loop_helper;

pub use crate::loop_helper::*;
use std::{
    thread,
    time::{Duration, Instant},
};

/// Marker alias to show the meaning of a `f64` in certain methods.
pub type Seconds = f64;
/// Marker alias to show the meaning of a `f64` in certain methods.
pub type RatePerSecond = f64;
/// Marker alias to show the meaning of a `u64` in certain methods.
pub type Nanoseconds = u64;
/// Marker alias to show the meaning of a `u32` in certain methods.
pub type SubsecondNanoseconds = u32;

/// Accuracy container for spin sleeping. See [crate docs](index.html).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpinSleeper {
    native_accuracy_ns: u32,
    spin_strategy: SpinStrategy,
}

#[cfg(not(windows))]
const DEFAULT_NATIVE_SLEEP_ACCURACY: SubsecondNanoseconds = 125_000;

/// Asks the OS to put the current thread to sleep for at least the specified amount of time.
/// **Does not spin.**
///
/// Equivalent to [`std::thread::sleep`], with the following exceptions:
/// * **Windows**: Automatically selects the best native sleep accuracy generally achieving ~1ms
/// native sleep accuracy, instead of default ~16ms.
#[cfg(not(windows))]
#[inline]
pub fn native_sleep(duration: Duration) {
    thread::sleep(duration)
}

#[cfg(windows)]
static MIN_TIME_PERIOD: once_cell::sync::Lazy<winapi::shared::minwindef::UINT> =
    once_cell::sync::Lazy::new(|| unsafe {
        use std::mem;
        use winapi::um::{mmsystem::*, timeapi::timeGetDevCaps};

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
    });

/// Asks the OS to put the current thread to sleep for at least the specified amount of time.
///
/// Equivalent to [`std::thread::sleep`], with the following exceptions:
/// * **Windows**: Automatically selects the best native sleep accuracy generally achieving ~1ms
/// native sleep accuracy, instead of default ~16ms.
#[cfg(windows)]
#[inline]
pub fn native_sleep(duration: Duration) {
    unsafe {
        use winapi::um::timeapi::{timeBeginPeriod, timeEndPeriod};
        timeBeginPeriod(*MIN_TIME_PERIOD);
        thread::sleep(duration);
        timeEndPeriod(*MIN_TIME_PERIOD);
    }
}

impl Default for SpinSleeper {
    /// Constructs new SpinSleeper with defaults suiting the current OS
    #[inline]
    fn default() -> Self {
        #[cfg(windows)]
        let accuracy = *MIN_TIME_PERIOD * 1_000_000;
        #[cfg(not(windows))]
        let accuracy = DEFAULT_NATIVE_SLEEP_ACCURACY;

        SpinSleeper::new(accuracy)
    }
}

impl SpinSleeper {
    /// Constructs new SpinSleeper with the input native sleep accuracy.
    /// The lower the `native_accuracy_ns` the more we effectively trust the accuracy of the
    /// [`native_sleep`] function.
    #[inline]
    pub fn new(native_accuracy_ns: SubsecondNanoseconds) -> SpinSleeper {
        SpinSleeper {
            native_accuracy_ns,
            spin_strategy: <_>::default(),
        }
    }

    /// Returns configured native_accuracy_ns.
    pub fn native_accuracy_ns(self) -> SubsecondNanoseconds {
        self.native_accuracy_ns
    }

    /// Returns configured spin strategy.
    pub fn spin_strategy(self) -> SpinStrategy {
        self.spin_strategy
    }

    /// Returns a spin sleeper with the given [`SpinStrategy`].
    ///
    /// # Example
    /// ```no_run
    /// use spin_sleep::{SpinSleeper, SpinStrategy};
    ///
    /// let sleeper = SpinSleeper::default().with_spin_strategy(SpinStrategy::SpinLoopHint);
    /// ```
    pub fn with_spin_strategy(mut self, strategy: SpinStrategy) -> Self {
        self.spin_strategy = strategy;
        self
    }

    /// Puts the [current thread to sleep](fn.native_sleep.html) for the duration less the
    /// configured native accuracy. Then spins until the specified duration has elapsed.
    pub fn sleep(self, duration: Duration) {
        let start = Instant::now();
        let accuracy = Duration::new(0, self.native_accuracy_ns);
        if duration > accuracy {
            native_sleep(duration - accuracy);
        }
        // spin the rest of the duration
        while start.elapsed() < duration {
            match self.spin_strategy {
                SpinStrategy::YieldThread => thread::yield_now(),
                SpinStrategy::SpinLoopHint => std::hint::spin_loop(),
            }
        }
    }

    /// Puts the [current thread to sleep](fn.native_sleep.html) for the give seconds-duration
    /// less the configured native accuracy. Then spins until the specified duration has elapsed.
    pub fn sleep_s(self, seconds: Seconds) {
        if seconds > 0.0 {
            self.sleep(Duration::from_secs_f64(seconds));
        }
    }

    /// Puts the [current thread to sleep](fn.native_sleep.html) for the give nanoseconds-duration
    /// less the configured native accuracy. Then spins until the specified duration has elapsed.
    pub fn sleep_ns(self, nanoseconds: Nanoseconds) {
        let subsec_ns = (nanoseconds % 1_000_000_000) as u32;
        let seconds = nanoseconds / 1_000_000_000;
        self.sleep(Duration::new(seconds, subsec_ns))
    }
}

/// Puts the [current thread to sleep](fn.native_sleep.html) for the duration less the
/// default native accuracy. Then spins until the specified duration has elapsed.
///
/// Convenience function for `SpinSleeper::default().sleep(duration)`. Can directly take the
/// place of `thread::sleep`.
pub fn sleep(duration: Duration) {
    SpinSleeper::default().sleep(duration);
}

/// What to do while spinning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SpinStrategy {
    /// Call [`std::thread::yield_now`] while spinning.
    YieldThread,
    /// Call [`std::hint::spin_loop`] while spinning.
    SpinLoopHint,
}

/// Per-OS default strategy.
/// * Windows  `SpinLoopHint`
/// * !Windows `YieldThread`
impl Default for SpinStrategy {
    #[inline]
    #[cfg(windows)]
    fn default() -> Self {
        Self::SpinLoopHint
    }
    #[inline]
    #[cfg(not(windows))]
    fn default() -> Self {
        Self::YieldThread
    }
}

// Not run unless specifically enabled with `cargo test --features "nondeterministic_tests"`
// Travis does not do well with these tests, as they require a certain CPU priority.
#[cfg(feature = "nondeterministic_tests")]
#[cfg(test)]
mod spin_sleep_test {
    use super::*;

    // The worst case error is unbounded even when spinning, but this accuracy is reasonable
    // for most platforms.
    const ACCEPTABLE_DELTA_NS: SubsecondNanoseconds = 50_000;

    // Since on spin performance is not guaranteed it suffices that the assertions are valid
    // 'most of the time'. This macro should avoid most 1-off failures.
    macro_rules! passes_eventually {
        ($test:expr) => {{
            let mut error = None;
            for _ in 0..50 {
                match ::std::panic::catch_unwind(|| $test) {
                    Ok(_) => break,
                    Err(err) => {
                        // test is failing, maybe due to spin unreliability
                        error = error.or(Some(err));
                        thread::sleep(Duration::new(0, 1000));
                    }
                }
            }
            assert!(
                error.is_none(),
                "Test failed 50/50 times: {:?}",
                error.unwrap()
            );
        }};
    }

    #[test]
    fn sleep_small() {
        passes_eventually!({
            let ns_duration = 12_345_678;

            let ps = SpinSleeper::new(20_000_000);
            ps.sleep(Duration::new(0, 1000)); // warm up

            let before = Instant::now();
            ps.sleep(Duration::new(0, ns_duration));
            let elapsed = before.elapsed();

            println!("Actual: {:?}", elapsed);
            assert!(elapsed <= Duration::new(0, ns_duration + ACCEPTABLE_DELTA_NS));
            assert!(elapsed >= Duration::new(0, ns_duration - ACCEPTABLE_DELTA_NS));
        });
    }

    #[test]
    fn sleep_big() {
        passes_eventually!({
            let ns_duration = 212_345_678;

            let ps = SpinSleeper::new(20_000_000);
            ps.sleep(Duration::new(0, 1000)); // warm up

            let before = Instant::now();
            ps.sleep(Duration::new(1, ns_duration));
            let elapsed = before.elapsed();

            println!("Actual: {:?}", elapsed);
            assert!(elapsed <= Duration::new(1, ns_duration + ACCEPTABLE_DELTA_NS));
            assert!(elapsed >= Duration::new(1, ns_duration - ACCEPTABLE_DELTA_NS));
        });
    }

    #[test]
    fn sleep_s() {
        passes_eventually!({
            let ns_duration = 12_345_678_f64;

            let ps = SpinSleeper::new(20_000_000);
            ps.sleep_s(0.000001); // warm up

            let before = Instant::now();
            ps.sleep_s(ns_duration / 1_000_000_000_f64);
            let elapsed = before.elapsed();

            println!("Actual: {:?}", elapsed);
            assert!(elapsed <= Duration::new(0, ns_duration.round() as u32 + ACCEPTABLE_DELTA_NS));
            assert!(elapsed >= Duration::new(0, ns_duration.round() as u32 - ACCEPTABLE_DELTA_NS));
        });
    }

    #[test]
    fn sleep_ns() {
        passes_eventually!({
            let ns_duration: u32 = 12_345_678;

            let ps = SpinSleeper::new(20_000_000);
            ps.sleep_ns(1000); // warm up

            let before = Instant::now();
            ps.sleep_ns(ns_duration as u64);
            let elapsed = before.elapsed();

            println!("Actual: {:?}", elapsed);
            assert!(elapsed <= Duration::new(0, ns_duration + ACCEPTABLE_DELTA_NS));
            assert!(elapsed >= Duration::new(0, ns_duration - ACCEPTABLE_DELTA_NS));
        });
    }
}
