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
//! # Examples
//!
//! Simplest usage with default native accuracy is a drop in replacement for `thread::sleep`.
//! ```no_run
//! # use std::time::Duration;
//! spin_sleep::sleep(Duration::new(1, 12_550_000));
//! ```
//!
//! More advanced usage, including setting a custom native accuracy, can be achieved by
//! constructing a `SpinSleeper`.
//! ```no_run
//! # use std::time::Duration;
//! // Create a new sleeper that trusts native thread::sleep with 100μs accuracy
//! let spin_sleeper = spin_sleep::SpinSleeper::new(100_000);
//!
//! // Sleep for 1.01255 seconds, this will:
//! //  - thread:sleep for 1.01245 seconds, ie 100μs less than the requested duration
//! //  - spin until total 1.01255 seconds have elapsed
//! spin_sleeper.sleep(Duration::new(1, 12_550_000));
//! ```
//!
//! Sleep can also requested in `f64` seconds or `u64` nanoseconds
//! (useful when used with `time` crate)
//!
//! ```no_run
//! # use std::time::Duration;
//! # let spin_sleeper = spin_sleep::SpinSleeper::new(100_000);
//! spin_sleeper.sleep_s(1.01255);
//! spin_sleeper.sleep_ns(1_012_550_000);
//! ```
//!
//! OS-specific default accuracy settings should be good enough for most cases.
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

/*
I would highly suggest *not* using floats to represent time.  This is just a mess
of issues due to variations on float accuracy and performance issues to be avoided.
I would also suggest changing Nanoseconds to be:
#[repr(transparent)]
struct Nanoseconds {
    duration: u64
}
This enforces type safety, allows impl From's for conversions and of course makes
it compatible with C ABI's if you ever attempt to wrap this up and expose it to C.

Also of note, I might suggest just using Microseconds as the exposed granularity
since you are unlikely to ever get better than +-1 ms accuracy anyway.
 */

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
}

#[cfg(not(windows))]
const DEFAULT_NATIVE_SLEEP_ACCURACY: SubsecondNanoseconds = 125_000;

#[cfg(not(windows))]
#[inline]
pub(crate) fn thread_sleep(duration: Duration) {
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

#[cfg(windows)]
#[inline]
pub(crate) fn thread_sleep(duration: Duration) {
    unsafe {
        use winapi::um::timeapi::{timeBeginPeriod, timeEndPeriod};

        /*
        This is a *BIG* no no..  The problem here is that timeBeginPeriod has
        global impact on the entire OS.  The overhead of changing this value
        regularly is ... not good, since all processes (not just threads) in the
        system will now have different accuracy for sleeps, system interrupt
        rates are increased, etc etc.  Not to mention that the cost of the pair
        of calls is exceptionally high.  From the docs:

        "Use caution when calling timeBeginPeriod, as frequent calls can
        significantly affect the system clock, system power usage, and the scheduler."

        To keep using this, only do it once a frame at most and even that is
        not really a great idea since it can trigger a number of bugs in the
        overall system where timers change accuracy unexpectedly in completely
        unrelated processes.
         */
        timeBeginPeriod(*MIN_TIME_PERIOD);
        thread::sleep(duration);
        timeEndPeriod(*MIN_TIME_PERIOD);

        /*
        The option to replace this is a spin wait using _mm_pause.  This is a
        low level instruction you can find in:
        https://doc.rust-lang.org/1.29.1/core/arch/x86_64/fn._mm_pause.html

        There are two problems with this item though, one building on the other.
        Intel, in their infinite wisdom, decided to change the behavior of the
        silicon on Skylake and broke a lot of things thanks to the change.
        Everything prior to Skylake used a delay of between 6 and 14 CPI,
        Skylake cranked the hell out of it and it now delays the CPU by 140 CPI.

        The second problem follows from the change: for tight multicore work that
        targets multiple CPU's now have to detect if the CPU is Skylake to
        adjust spin wait repetitions or risk massive wait time behavior changes.

        Anyway, you would end up with something like:
        loop {
            let count = 32; // Pre-Skylake
            for i in 0..count {
                std::arch::x86_64::_mm_pause;
            }

            // Test current time.
            // As a note, calling time functions is also slow, so
            // generally you want a linear backoff where the count
            // above is about 1/2 the "guessed" at length of the
            // desired sleep period the first time through, then
            // use half of that again for the next loop etc till
            // you get "close enough".
        }
         */
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
    /// `thread::sleep` function.
    #[inline]
    pub fn new(native_accuracy_ns: SubsecondNanoseconds) -> SpinSleeper {
        SpinSleeper { native_accuracy_ns }
    }

    /// Returns configured native_accuracy_ns
    pub fn native_accuracy_ns(self) -> SubsecondNanoseconds {
        self.native_accuracy_ns
    }

    /// Puts the current thread to sleep, if duration is long enough, then spins until the
    /// specified duration has elapsed.
    ///
    /// **Windows**: Automatically selects the best native sleep accuracy generally achieving ~1ms
    /// native sleep accuracy, instead of default ~16ms.
    pub fn sleep(self, duration: Duration) {
        /* likely bug here, at least for windozer.
        The accuracy is not measured from point of call, it is a granularity measure.  In other
        words, when you call sleep the accuracy is not "from the time you make that call" it is
        rather until the next "tick" of the system clock.  So, if you call sleep(16'ms') and it is
        half way between system ticks you will end up not waking up for 8 ms till next tick, which
        is too early so it keeps sleeping till the next increment of 16 ms, meaning 24ms of actual
        sleep time.

        All said and done, you should use OS Sleep as much as possible even with all the problems.
        Basically you take the desired sleep duration and chop it up by accuracy granularity, so
        if the desired sleep is 30ms and accuracy is ~16ms, go ahead and sleep(16) then recheck the
        timer, if >30ms, return obviously, if less "then" start using the spin loops to get closer
        to the desired time.
        */

        let start = Instant::now();
        let accuracy = Duration::new(0, self.native_accuracy_ns);
        if duration > accuracy {
            thread_sleep(duration - accuracy);
        }
        // spin the rest of the duration
        while start.elapsed() < duration {
            thread::yield_now();
        }
    }

    /// Puts the current thread to sleep, if duration is long enough, then spins until the
    /// specified second duration has elapsed.
    ///
    /// **Windows**: Automatically selects the best native sleep accuracy generally achieving ~1ms
    /// native sleep accuracy, instead of default ~16ms.
    pub fn sleep_s(self, seconds: Seconds) {
        if seconds > 0.0 {
            self.sleep(Duration::from_secs_f64(seconds));
        }
    }

    /// Puts the current thread to sleep, if duration is long enough, then spins until the
    /// specified nanosecond duration has elapsed.
    ///
    /// **Windows**: Automatically selects the best native sleep accuracy generally achieving ~1ms
    /// native sleep accuracy, instead of default ~16ms.
    pub fn sleep_ns(self, nanoseconds: Nanoseconds) {
        let subsec_ns = (nanoseconds % 1_000_000_000) as u32;
        let seconds = nanoseconds / 1_000_000_000;
        self.sleep(Duration::new(seconds, subsec_ns))
    }
}

/// Puts the current thread to sleep, if duration is long enough, then spins until the
/// specified nanosecond duration has elapsed.
///
/// Uses default native accuracy.
///
/// Convenience function for `SpinSleeper::default().sleep(duration)`. Can directly take the
/// place of `thread::sleep`.
///
/// **Windows**: Automatically selects the best native sleep accuracy generally achieving ~1ms
/// native sleep accuracy, instead of default ~16ms.
pub fn sleep(duration: Duration) {
    SpinSleeper::default().sleep(duration);
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
        ($test:stmt) => {{
            let mut error = None;
            for _ in 0..50 {
                match ::std::panic::catch_unwind(|| {
                    $test;
                }) {
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
            let after = Instant::now();

            println!("Actual: {:?}", after.duration_since(before));
            assert!(
                after.duration_since(before) <= Duration::new(0, ns_duration + ACCEPTABLE_DELTA_NS)
            );
            assert!(
                after.duration_since(before) >= Duration::new(0, ns_duration - ACCEPTABLE_DELTA_NS)
            );
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
            let after = Instant::now();

            println!("Actual: {:?}", after.duration_since(before));
            assert!(
                after.duration_since(before) <= Duration::new(1, ns_duration + ACCEPTABLE_DELTA_NS)
            );
            assert!(
                after.duration_since(before) >= Duration::new(1, ns_duration - ACCEPTABLE_DELTA_NS)
            );
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
            let after = Instant::now();

            println!("Actual: {:?}", after.duration_since(before));
            assert!(
                after.duration_since(before)
                    <= Duration::new(0, ns_duration.round() as u32 + ACCEPTABLE_DELTA_NS)
            );
            assert!(
                after.duration_since(before)
                    >= Duration::new(0, ns_duration.round() as u32 - ACCEPTABLE_DELTA_NS)
            );
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
            let after = Instant::now();

            println!("Actual: {:?}", after.duration_since(before));
            assert!(
                after.duration_since(before) <= Duration::new(0, ns_duration + ACCEPTABLE_DELTA_NS)
            );
            assert!(
                after.duration_since(before) >= Duration::new(0, ns_duration - ACCEPTABLE_DELTA_NS)
            );
        });
    }
}

#[test]
#[ignore]
fn print_estimated_thread_sleep_accuracy() {
    let mut best = Duration::from_secs(100);
    let mut sum = Duration::from_secs(0);
    let mut worst = Duration::from_secs(0);

    for _ in 0..100 {
        let before = Instant::now();
        thread_sleep(Duration::new(0, 1));
        let elapsed = before.elapsed();
        sum += elapsed;
        if elapsed < best {
            best = elapsed;
        }
        if elapsed > worst {
            worst = elapsed;
        }
    }

    println!(
        "average: {:?}, best : {:?}, worst: {:?}",
        Duration::from_nanos((sum.subsec_nanos() / 100).into()),
        best,
        worst,
    );

    panic!("Manual use only, ignore when done");
}
