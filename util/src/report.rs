use std::time::{Duration, Instant};

/// Helper for regularly reporting a rate per second, like fps.
///
/// # Example
/// ```no_run
/// # use std::time::Duration;
/// # fn compute_something() {}
/// # fn update_fps(fps: f64) {}
/// let mut reporter = spin_sleep_util::RateReporter::new(Duration::from_secs(1));
///
/// loop {
///     compute_something();
///
///     if let Some(fps) = reporter.increment_and_report() {
///         update_fps(fps);
///     }
/// }
/// ```
#[derive(Debug)]
pub struct RateReporter {
    report_period: Duration,
    start: Instant,
    rate_count: u32,
}

impl RateReporter {
    /// Returns a new [`RateReporter`] with the given `report_period` minimum
    /// duration to return reports and aggregate inside.
    pub fn new(report_period: Duration) -> Self {
        Self {
            report_period,
            start: Instant::now(),
            rate_count: 0,
        }
    }

    /// Increment the rate count for the next report.
    pub fn increment(&mut self) {
        self.rate_count = self.rate_count.saturating_add(1);
    }

    /// If at least `report_period` has elapsed since the last report returns the mean rate per second
    /// and resets the rate count to zero and start to now. Otherwise returns `None`.
    pub fn report(&mut self) -> Option<f64> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.start);
        if elapsed < self.report_period {
            return None;
        }

        let report = f64::from(self.rate_count) / elapsed.as_secs_f64();
        self.rate_count = 0;
        self.start = now;
        Some(report)
    }

    /// [`Self::increment`] and [`Self::report`].
    pub fn increment_and_report(&mut self) -> Option<f64> {
        self.increment();
        self.report()
    }

    /// Reset rate count to zero & report start to now.
    pub fn reset(&mut self) {
        self.rate_count = 0;
        self.start = Instant::now();
    }
}
