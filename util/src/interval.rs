use spin_sleep::SpinSleeper;
use std::time::{Duration, Instant};

/// Creates new [`Interval`] that spin-sleeps with interval of `period`. The first
/// tick returns immediately. The default [`MissedTickBehavior`] is
/// [`Skip`](MissedTickBehavior::Skip).
///
/// This function is equivalent to
/// [`interval_at(Instant::now(), period)`](interval_at).
///
/// # Panics
///
/// This function panics if `period` is zero.
#[track_caller]
pub fn interval(period: Duration) -> Interval {
    interval_at(Instant::now(), period)
}

/// Creates new [`Interval`] that spin-sleeps with interval of `period` with the
/// first tick returning at `start`. The default [`MissedTickBehavior`] is
/// [`Skip`](MissedTickBehavior::Skip).
///
/// # Panics
///
/// This function panics if `period` is zero.
#[track_caller]
pub fn interval_at(start: Instant, period: Duration) -> Interval {
    assert!(period > Duration::ZERO, "`period` must be non-zero.");
    Interval {
        next_tick: start,
        period,
        missed_tick_behavior: <_>::default(),
        sleeper: <_>::default(),
    }
}

/// Interval returned by [`interval`] and [`interval_at`].
#[derive(Debug)]
pub struct Interval {
    next_tick: Instant,
    period: Duration,
    missed_tick_behavior: MissedTickBehavior,
    sleeper: SpinSleeper,
}

impl Interval {
    /// Use [`SpinSleeper`] to sleep until the next scheduled tick.
    ///
    /// If the tick is in the past will return without sleeping
    /// computing the next tick based on the configured [`MissedTickBehavior`].
    ///
    /// Returns the tick time.
    pub fn tick(&mut self) -> Instant {
        let tick = self.next_tick;
        let now = Instant::now();

        if now > tick {
            // missed tick
            self.next_tick = self.missed_tick_behavior.next_tick(tick, now, self.period);
            return tick;
        }

        self.sleeper.sleep(tick - now);

        self.next_tick = tick + self.period;
        tick
    }

    /// Resets the interval to complete one period after the current time.
    pub fn reset(&mut self) {
        self.next_tick = Instant::now() + self.period;
    }

    /// Returns the [`MissedTickBehavior`] strategy currently being used.
    ///
    /// # Example
    /// ```
    /// use spin_sleep_util::{interval, MissedTickBehavior};
    /// # use std::time::Duration;
    ///
    /// let i = interval(Duration::from_millis(20));
    /// assert_eq!(i.missed_tick_behavior(), MissedTickBehavior::Skip);
    /// ```
    pub fn missed_tick_behavior(&self) -> MissedTickBehavior {
        self.missed_tick_behavior
    }

    /// Returns the period of the interval.
    ///
    /// # Example
    /// ```
    /// use spin_sleep_util::interval;
    /// # use std::time::Duration;
    ///
    /// let i = interval(Duration::from_millis(20));
    /// assert_eq!(i.period(), Duration::from_millis(20));
    /// ```
    pub fn period(&self) -> Duration {
        self.period
    }

    /// Sets the [`MissedTickBehavior`] strategy that should be used.
    ///
    /// # Example
    /// ```
    /// use spin_sleep_util::{interval, MissedTickBehavior};
    /// # use std::time::Duration;
    ///
    /// let mut i = interval(Duration::from_millis(20));
    /// i.set_missed_tick_behavior(MissedTickBehavior::Burst);
    /// assert_eq!(i.missed_tick_behavior(), MissedTickBehavior::Burst);
    /// ```
    pub fn set_missed_tick_behavior(&mut self, behavior: MissedTickBehavior) {
        self.missed_tick_behavior = behavior;
    }

    /// Returns `Self` with the specified [`MissedTickBehavior`] strategy.
    ///
    /// # Example
    /// ```
    /// use spin_sleep_util::{interval, MissedTickBehavior};
    /// # use std::time::Duration;
    ///
    /// let i =
    ///     interval(Duration::from_millis(20)).with_missed_tick_behavior(MissedTickBehavior::Burst);
    /// assert_eq!(i.missed_tick_behavior(), MissedTickBehavior::Burst);
    /// ```
    pub fn with_missed_tick_behavior(mut self, behavior: MissedTickBehavior) -> Self {
        self.missed_tick_behavior = behavior;
        self
    }

    /// Returns the configured [`SpinSleeper`].
    ///
    /// # Example
    /// ```
    /// use spin_sleep::SpinSleeper;
    /// use spin_sleep_util::interval;
    /// # use std::time::Duration;
    ///
    /// let i = interval(Duration::from_millis(20));
    /// assert_eq!(i.spin_sleeper(), SpinSleeper::default());
    /// ```
    pub fn spin_sleeper(&self) -> SpinSleeper {
        self.sleeper
    }

    /// Sets the [`SpinSleeper`] used for accurate sleeping.
    ///
    /// # Example
    /// ```
    /// # use spin_sleep::SpinSleeper;
    /// use spin_sleep_util::interval;
    /// # use std::time::Duration;
    /// # let custom_sleeper = SpinSleeper::new(123456);
    ///
    /// let mut i = interval(Duration::from_millis(20));
    /// i.set_spin_sleeper(custom_sleeper);
    /// assert_eq!(i.spin_sleeper(), custom_sleeper);
    /// ```
    pub fn set_spin_sleeper(&mut self, sleeper: SpinSleeper) {
        self.sleeper = sleeper;
    }

    /// Returns `Self` with the specified [`SpinSleeper`].
    ///
    /// # Example
    /// ```
    /// # use spin_sleep::SpinSleeper;
    /// use spin_sleep_util::interval;
    /// # use std::time::Duration;
    /// # let custom_sleeper = SpinSleeper::new(123456);
    ///
    /// let i = interval(Duration::from_millis(20)).with_spin_sleeper(custom_sleeper);
    /// assert_eq!(i.spin_sleeper(), custom_sleeper);
    /// ```
    pub fn with_spin_sleeper(mut self, sleeper: SpinSleeper) -> Self {
        self.sleeper = sleeper;
        self
    }
}

/// Defines the behavior of an [`Interval`] when it misses a tick.
///
/// Generally, a tick is missed if too much time is spent without calling
/// [`Interval::tick()`].
///
/// Default [`Skip`](MissedTickBehavior::Skip).
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissedTickBehavior {
    /// Ticks as fast as possible until caught up.
    ///
    /// When this strategy is used, [`Interval`] schedules ticks "normally" (the
    /// same as it would have if the ticks hadn't been delayed), which results
    /// in it firing ticks as fast as possible until it is caught up in time to
    /// where it should be. Unlike [`Delay`] and [`Skip`], the ticks yielded
    /// when `Burst` is used (the [`Instant`]s that [`tick`](Interval::tick)
    /// yields) aren't different than they would have been if a tick had not
    /// been missed. Like [`Skip`], and unlike [`Delay`], the ticks may be
    /// shortened.
    ///
    /// This looks something like this:
    /// ```text
    /// Expected ticks: |     1     |     2     |     3     |     4     |     5     |     6     |
    /// Actual ticks:   | work -----|          delay          | work | work | work -| work -----|
    /// ```
    ///
    /// In code:
    ///
    /// ```no_run
    /// use spin_sleep_util::{interval, MissedTickBehavior};
    /// use std::time::Duration;
    /// # fn task_that_takes_200_millis() {}
    ///
    /// let mut interval =
    ///     interval(Duration::from_millis(50)).with_missed_tick_behavior(MissedTickBehavior::Burst);
    ///
    /// // First tick resolves immediately after creation
    /// interval.tick();
    ///
    /// task_that_takes_200_millis();
    /// // The `Interval` has missed a tick
    ///
    /// // Since we have exceeded our timeout, this will resolve immediately
    /// interval.tick();
    ///
    /// // Since we are more than 100ms after the start of `interval`, this will
    /// // also resolve immediately.
    /// interval.tick();
    ///
    /// // Also resolves immediately, because it was supposed to resolve at
    /// // 150ms after the start of `interval`
    /// interval.tick();
    ///
    /// // Resolves immediately
    /// interval.tick();
    ///
    /// // Since we have gotten to 200ms after the start of `interval`, this
    /// // will resolve after 50ms
    /// interval.tick();
    /// ```
    ///
    /// [`Delay`]: MissedTickBehavior::Delay
    /// [`Skip`]: MissedTickBehavior::Skip
    Burst,

    /// Tick at multiples of `period` from when [`tick`] was called, rather than
    /// from `start`.
    ///
    /// When this strategy is used and [`Interval`] has missed a tick, instead
    /// of scheduling ticks to fire at multiples of `period` from `start` (the
    /// time when the first tick was fired), it schedules all future ticks to
    /// happen at a regular `period` from the point when [`tick`] was called.
    /// Unlike [`Burst`] and [`Skip`], ticks are not shortened, and they aren't
    /// guaranteed to happen at a multiple of `period` from `start` any longer.
    ///
    /// This looks something like this:
    /// ```text
    /// Expected ticks: |     1     |     2     |     3     |     4     |     5     |     6     |
    /// Actual ticks:   | work -----|          delay          | work -----| work -----| work -----|
    /// ```
    ///
    /// In code:
    ///
    /// ```no_run
    /// use spin_sleep_util::{interval, MissedTickBehavior};
    /// use std::time::Duration;
    /// # fn task_that_takes_more_than_50_millis() {}
    ///
    /// let mut interval =
    ///     interval(Duration::from_millis(50)).with_missed_tick_behavior(MissedTickBehavior::Delay);
    ///
    /// task_that_takes_more_than_50_millis();
    /// // The `Interval` has missed a tick
    ///
    /// // Since we have exceeded our timeout, this will resolve immediately
    /// interval.tick();
    ///
    /// // But this one, rather than also resolving immediately, as might happen
    /// // with the `Burst` or `Skip` behaviors, will not resolve until
    /// // 50ms after the call to `tick` up above. That is, in `tick`, when we
    /// // recognize that we missed a tick, we schedule the next tick to happen
    /// // 50ms (or whatever the `period` is) from right then, not from when
    /// // were *supposed* to tick
    /// interval.tick();
    /// ```
    ///
    /// [`Burst`]: MissedTickBehavior::Burst
    /// [`Skip`]: MissedTickBehavior::Skip
    /// [`tick`]: Interval::tick
    Delay,

    /// Skips missed ticks and tick on the next multiple of `period` from
    /// `start`.
    ///
    /// When this strategy is used, [`Interval`] schedules the next tick to fire
    /// at the next-closest tick that is a multiple of `period` away from
    /// `start` (the point where [`Interval`] first ticked). Like [`Burst`], all
    /// ticks remain multiples of `period` away from `start`, but unlike
    /// [`Burst`], the ticks may not be *one* multiple of `period` away from the
    /// last tick. Like [`Delay`], the ticks are no longer the same as they
    /// would have been if ticks had not been missed, but unlike [`Delay`], and
    /// like [`Burst`], the ticks may be shortened to be less than one `period`
    /// away from each other.
    ///
    /// This looks something like this:
    /// ```text
    /// Expected ticks: |     1     |     2     |     3     |     4     |     5     |     6     |
    /// Actual ticks:   | work -----|          delay          | work ---| work -----| work -----|
    /// ```
    ///
    /// In code:
    ///
    /// ```no_run
    /// use spin_sleep_util::{interval, MissedTickBehavior};
    /// use std::time::Duration;
    /// # fn task_that_takes_75_millis() {}
    ///
    /// let mut interval = interval(Duration::from_millis(50));
    ///
    /// task_that_takes_75_millis();
    /// // The `Interval` has missed a tick
    ///
    /// // Since we have exceeded our timeout, this will resolve immediately
    /// interval.tick();
    ///
    /// // This one will resolve after 25ms, 100ms after the start of
    /// // `interval`, which is the closest multiple of `period` from the start
    /// // of `interval` after the call to `tick` up above.
    /// interval.tick();
    /// ```
    ///
    /// [`Burst`]: MissedTickBehavior::Burst
    /// [`Delay`]: MissedTickBehavior::Delay
    #[default]
    Skip,
}

impl MissedTickBehavior {
    /// If a tick is missed, this method is called to determine when the next tick should happen.
    fn next_tick(&self, missed_tick: Instant, now: Instant, period: Duration) -> Instant {
        match self {
            Self::Burst => missed_tick + period,
            Self::Delay => now + period,
            Self::Skip => {
                now + period
                    - Duration::from_nanos(
                        ((now - missed_tick).as_nanos() % period.as_nanos())
                            .try_into()
                            // This operation is practically guaranteed not to
                            // fail, as in order for it to fail, `period` would
                            // have to be longer than `now - timeout`, and both
                            // would have to be longer than 584 years.
                            //
                            // If it did fail, there's not a good way to pass
                            // the error along to the user, so we just panic.
                            .expect(
                                "too much time has elapsed since the interval was supposed to tick",
                            ),
                    )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn at_zero_period() {
        interval_at(Instant::now(), Duration::ZERO);
    }

    #[test]
    #[should_panic]
    fn zero_period() {
        interval(Duration::ZERO);
    }
}
