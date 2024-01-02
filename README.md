spin_sleep
[![crates.io](https://img.shields.io/crates/v/spin_sleep.svg)](https://crates.io/crates/spin_sleep)
[![Documentation](https://docs.rs/spin_sleep/badge.svg)](https://docs.rs/spin_sleep)
==========

Accurate sleeping. Only use native sleep as far as it can be trusted, then spin.

The problem with `thread::sleep` is it isn't always very accurate, and this accuracy varies
on platform and state. Spinning is as accurate as we can get, but consumes the CPU
rather ungracefully.

This library adds a middle ground, using a configurable native accuracy setting allowing
thread::sleep to wait the bulk of a sleep time, and spin the final section to guarantee
accuracy.

### SpinSleeper
The simplest usage with default native accuracy is a drop in replacement for `thread::sleep`.
```rust
spin_sleep::sleep(Duration::new(1, 12_550_000));
```

#### Configure
More advanced usage, including setting a custom native accuracy, can be achieved by
constructing a `SpinSleeper`.
```rust
// Create a new sleeper that trusts native thread::sleep with 100μs accuracy
let spin_sleeper = spin_sleep::SpinSleeper::new(100_000)
    .with_spin_strategy(spin_sleep::SpinStrategy::YieldThread);

// Sleep for 1.01255 seconds, this will:
//  - thread:sleep for 1.01245 seconds, i.e., 100μs less than the requested duration
//  - spin until total 1.01255 seconds have elapsed
spin_sleeper.sleep(Duration::new(1, 12_550_000));
```

Sleep can also be requested in `f64` seconds or `u64` nanoseconds
(useful when used with `time` crate)

```rust
spin_sleeper.sleep_s(1.01255);
spin_sleeper.sleep_ns(1_012_550_000);
```

OS-specific default settings should be good enough for most cases.
```rust
let sleeper = SpinSleeper::default();
```

### Windows Accuracy
Windows (>= Windows 10, version 1803) will use a high resolution waitable timer, similar to sleep in rust std >= 1.75.

Earlier versions of Windows have particularly poor accuracy by default (~15ms), `spin_sleep` will automatically
select the best accuracy on Windows generally achieving ~1-2ms native sleep accuracy.

## Minimum supported rust compiler
This crate is maintained with [latest stable rust](https://gist.github.com/alexheretic/d1e98d8433b602e57f5d0a9637927e0c).
