use super::*;
use std::time::{Instant, Duration};
use std::f64;

#[cfg(target_os = "windows")]
pub const DEFAULT_NATIVE_SLEEP_ACCURACY: SubsecondNanoseconds = 15_000_000;
#[cfg(not(target_os = "windows"))]
pub const DEFAULT_NATIVE_SLEEP_ACCURACY: SubsecondNanoseconds = 125_000;

trait ToF64Seconds {
    fn to_f64_secs(&self) -> Seconds;
}

trait FromF64Seconds<T> {
    fn from_f64_secs(seconds: Seconds) -> T;
}

impl ToF64Seconds for Duration {
    fn to_f64_secs(&self) -> Seconds {
        let whole_seconds = self.as_secs() as f64;
        let subsec_nanos = self.subsec_nanos() as f64 / 1_000_000_000_f64;
        whole_seconds + subsec_nanos
    }
}

impl FromF64Seconds<Duration> for Duration {
    fn from_f64_secs(seconds: Seconds) -> Duration {
        let whole_seconds = seconds.round() as u64;
        let subsec_nanos = (seconds.fract() * 1_000_000_000_f64).round() as u32;
        Duration::new(whole_seconds, subsec_nanos)
    }
}

/// Tool for loop rate reporting and control.
///
/// Can report mean rate per second of a loop over a configured
/// report interval with [`LoopHelper::report_rate`](struct.LoopHelper.html#method.report_rate).
///
/// Can limit a loop rate to a desired target using
/// [`LoopHelper::loop_sleep`](struct.LoopHelper.html#method.loop_sleep).
///
/// # Example
///
/// ```no_run
/// use spin_sleep::LoopHelper;
///
/// let mut loop_helper = LoopHelper::builder()
///     .report_interval_s(0.5) // report every half a second
///     .build_with_target_rate(250.0); // limit to 250 FPS if possible
///
/// let mut current_fps = None;
///
/// loop {
///     let delta = loop_helper.loop_start(); // or .loop_start_s() for f64 seconds
///
///     // compute_something(delta);
///
///     if let Some(fps) = loop_helper.report_rate() {
///         current_fps = Some(fps.round());
///     }
///
///     // render_fps(current_fps);
///
///     loop_helper.loop_sleep(); // sleeps to acheive a 250 FPS rate
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopHelper {
    target_delta: Duration,
    report_interval: Duration,
    sleeper: SpinSleeper,

    last_loop_start: Instant,
    last_report: Instant,
    delta_sum: Duration,
    delta_count: u32,
}

/// Builds [`LoopHelper`](struct.LoopHelper.html).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopHelperBuilder {
    report_interval: Option<Duration>,
    sleeper: Option<SpinSleeper>,
}

impl LoopHelperBuilder {
    /// Sets the interval between
    /// [`LoopHelper::report_rate`](/struct.LoopHelper.html#method.report_rate) reports in seconds.
    pub fn report_interval_s(mut self, seconds: Seconds) -> Self {
        let whole_seconds = seconds.round() as u64;
        let subsec_nanos = (seconds.fract() * 1_000_000_000_f64).round() as u32;
        self.report_interval = Some(Duration::new(whole_seconds, subsec_nanos));
        self
    }

    /// Sets the interval between
    /// [`LoopHelper::report_rate`](/struct.LoopHelper.html#method.report_rate) reports.
    pub fn report_interval(mut self, duration: Duration) -> Self {
        self.report_interval = Some(duration);
        self
    }

    /// Sets the native sleep accuracy.
    /// See [`SpinSleeper::new`](struct.SpinSleeper.html#method.new) for details.
    pub fn native_accuracy_ns(mut self, accuracy: SubsecondNanoseconds) -> Self {
        self.sleeper = Some(SpinSleeper::new(accuracy));
        self
    }

    /// Builds a [`LoopHelper`](struct.LoopHelper.html) without targeting a rate.
    /// This means all calls to
    /// [`LoopHelper::loop_sleep`](struct.LoopHelper.html#method.loop_sleep) will simply return
    /// immediately. Normally used when only interested in the LoopHelper rate reporting.
    pub fn build_without_target_rate(self) -> LoopHelper {
        self.build_with_target_rate(f64::INFINITY)
    }

    /// Builds a [`LoopHelper`](struct.LoopHelper.html) targeting an input `target_rate`.
    /// Note: The `target_rate` only affects
    /// [`LoopHelper::loop_sleep`](struct.LoopHelper.html#method.loop_sleep).
    pub fn build_with_target_rate(self, target_rate: RatePerSecond) -> LoopHelper {
        let now = Instant::now();
        let interval = self.report_interval.unwrap_or_else(|| Duration::from_secs(1));
        LoopHelper {
            target_delta: Duration::from_f64_secs(1.0 / target_rate),
            report_interval: interval,
            sleeper: self.sleeper.unwrap_or_else(|| SpinSleeper::new(DEFAULT_NATIVE_SLEEP_ACCURACY)),

            last_report: now - interval,
            last_loop_start: now,
            delta_sum: Duration::from_secs(0),
            delta_count: 0,
        }
    }
}

impl LoopHelper {
    /// Returns a [`LoopHelperBuilder`](struct.LoopHelperBuilder.html) with which to build a
    /// `LoopHelper`.
    pub fn builder() -> LoopHelperBuilder {
        LoopHelperBuilder {
            report_interval: None,
            sleeper: None,
        }
    }

    /// Notifies the helper that a new loop has begun.
    /// Returns the delta, the duration since the last call to `loop_start` or `loop_start_s`.
    pub fn loop_start(&mut self) -> Duration {
        let it_start = Instant::now();
        let delta = it_start.duration_since(self.last_loop_start);
        self.last_loop_start = it_start;
        self.delta_sum += delta;
        self.delta_count += 1;
        delta
    }

    /// Notifies the helper that a new loop has begun.
    /// Returns the delta, the seconds since the last call to `loop_start` or `loop_start_s`.
    pub fn loop_start_s(&mut self) -> Seconds {
        self.loop_start().to_f64_secs()
    }

    /// Generally called at the end of a loop to sleep until the desired delta (configured with
    /// [`build_with_target_rate`](struct.LoopHelperBuilder.html#method.build_with_target_rate))
    /// has elapsed. Uses a [`SpinSleeper`](struct.SpinSleeper.html) to sleep the thread to provide
    /// improved accuracy. If the delta has already elapsed this method returns immediately.
    pub fn loop_sleep(&mut self) {
        let elapsed = self.last_loop_start.elapsed();
        if elapsed < self.target_delta {
            self.sleeper.sleep(self.target_delta - elapsed);
        }
    }

    /// Returns the mean rate per second recorded since the last report. Returns `None` if
    /// the last report was within the configured `report_interval`.
    pub fn report_rate(&mut self) -> Option<RatePerSecond> {
        let now = Instant::now();
        if now.duration_since(self.last_report) > self.report_interval && self.delta_count > 0 {
            let report = Some(1.0 / (self.delta_sum.to_f64_secs() / self.delta_count as f64));
            self.delta_sum = Duration::from_secs(0);
            self.delta_count = 0;
            self.last_report = now;
            report
        }
        else { None }
    }
}

#[cfg(test)]
mod loop_helper_test {
    use super::*;
    use std::thread;

    #[test]
    fn duration_f64_conversion() {
        let duration = Duration::new(123, 234_345_456);

        assert_relative_eq!(duration.to_f64_secs(), 123.234345456);
        // Note: f64 -> duration is not expected to be perfect in all cases
        assert_eq!(Duration::from_f64_secs(123.234345456), duration);
    }

    #[test]
    fn rate_reporting_using_duration() {
        let mut loop_helper = LoopHelper::builder()
            .report_interval_s(0.0)
            .build_without_target_rate();

        let loops = 10;
        let mut deltas = vec![];
        for _ in 0..loops {
            deltas.push(loop_helper.loop_start());
            thread::sleep(Duration::new(0, 1000));
        }

        let reported_rate = loop_helper.report_rate().expect("missing report");
        let expected_rate = 1.0 / (deltas.iter().sum::<Duration>().to_f64_secs() / loops as f64);

        assert_relative_eq!(reported_rate, expected_rate);
    }

    #[test]
    fn rate_reporting_using_seconds() {
        let mut loop_helper = LoopHelper::builder()
            .report_interval_s(0.0)
            .build_without_target_rate();

        let loops = 10;
        let mut deltas = vec![];
        for _ in 0..loops {
            deltas.push(loop_helper.loop_start_s());
            thread::sleep(Duration::new(0, 1000));
        }

        let reported_rate = loop_helper.report_rate().expect("missing report");
        let expected_rate = 1.0 / (deltas.iter().fold(0.0, |sum, n| sum + n) / loops as f64);

        assert_relative_eq!(reported_rate, expected_rate, epsilon = 1e-9);
    }

    #[test]
    fn loop_sleep_already_past_target() {
        let mut loop_helper = LoopHelper::builder()
            .report_interval_s(0.0)
            .build_with_target_rate(f64::INFINITY);

        loop_helper.loop_start();

        loop_helper.loop_sleep(); // should not panic
    }
}
