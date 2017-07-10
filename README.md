spin_sleep
<a href="https://crates.io/crates/spin_sleep">
  <img src="http://img.shields.io/crates/v/spin_sleep.svg">
</a>
<a href="https://docs.rs/spin_sleep">
  <img src="https://docs.rs/spin_sleep/badge.svg">
</a>
==========
Accurate sleeping. Only use native sleep as far as it can be trusted, then spin.

The problem with `thread::sleep` is it isn't always very accurate, and this error can varies
on platform and state. Spinning is as accurate as we can get, but consumes the CPU
rather ungracefully.

This library adds a middle ground, using a configurable native accuracy setting allowing
thread::sleep to wait the bulk of a sleep time, and spin the final section to guarantee
accuracy.

### Examples
```rust
extern crate spin_sleep;

// Create a new sleeper that trusts native thread::sleep with 100μs accuracy
let spin_sleeper = spin_sleep::SpinSleeper::new(100_000);

// Sleep for 1.01255 seconds, this will:
//  - thread:sleep for 1.01245 seconds, ie 100μs less than the requested duration
//  - spin until total 1.01255 seconds have elapsed
spin_sleeper.sleep(Duration::new(1, 12_550_000));
```

Sleep can also requested in f64 seconds (useful when used with `time` crate)
```rust
spin_sleeper.sleep_s(1.01255);
```
