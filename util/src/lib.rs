//! Utils using spin_sleep.
//!
//! # Example: Frame limiter
//! [`Interval`] may be used to limit a loop to a max fps by calling [`Interval::tick`]
//! at the start or end of each loop.
//! ```no_run
//! # use std::time::Duration;
//! # fn compute_something() {}
//! // Create an interval to tick 144 times each second
//! let mut interval = spin_sleep_util::interval(Duration::from_secs(1) / 144);
//! loop {
//!     compute_something(); // do loop work
//!
//!     // tick: sleep using a SpinSleeper until next tick.
//!     // The default `Skip` missed ticke behaviour is appropriate for a frame limiter
//!     interval.tick();
//! }
//! ```

mod interval;
mod report;

pub use interval::*;
pub use report::*;
