# spin_sleep experiments
Experiments to measure latency, all machine specific & non-deterministic.

## native_sleep_accuracy
Call OS native sleep for **1ns** and see how long it actually takes.

```sh
cargo run --bin native_sleep_accuracy --release
```

**Linux example output**
```
average: 53.04µs, best : 7.95µs, worst: 85.238µs
```

**Windows example output**
```
average: 2.012432ms, best : 2.0069ms, worst: 2.1455ms
```

## spin_strategy_latency
Measure `SpinStrategy` latencies and spin counts across various wait durations
_5ms, 900µs, 5µs, 100ns_.

```sh
cargo run --bin spin_strategy_latency --release
```

**Linux example output**
```
warming up...
5ms-None:               avg-spins: 190879  avg-actual: 5.000048ms
5ms-SpinLoopHint:       avg-spins: 168775  avg-actual: 5.000052ms
5ms-YieldThread:        avg-spins: 39381   avg-actual: 5.000103ms
900µs-None:             avg-spins: 34525   avg-actual: 900.046µs
900µs-SpinLoopHint:     avg-spins: 30454   avg-actual: 900.047µs
900µs-YieldThread:      avg-spins: 7046    avg-actual: 900.098µs
5µs-None:               avg-spins: 191     avg-actual: 5.042µs
5µs-SpinLoopHint:       avg-spins: 166     avg-actual: 5.041µs
5µs-YieldThread:        avg-spins: 39      avg-actual: 5.074µs
100ns-None:             avg-spins: 3       avg-actual: 128ns
100ns-SpinLoopHint:     avg-spins: 3       avg-actual: 135ns
100ns-YieldThread:      avg-spins: 1       avg-actual: 176ns
```

**Windows example output**
```
warming up...
5ms    None          avg-spins: 158591   avg-actual: 5ms
5ms    SpinLoopHint  avg-spins: 134568   avg-actual: 5ms
5ms    YieldThread   avg-spins: 50380    avg-actual: 5.000039ms
900µs  None          avg-spins: 28491    avg-actual: 900µs
900µs  SpinLoopHint  avg-spins: 24128    avg-actual: 900.002µs
900µs  YieldThread   avg-spins: 9070     avg-actual: 900.033µs
5µs    None          avg-spins: 155      avg-actual: 5µs
5µs    SpinLoopHint  avg-spins: 133      avg-actual: 5µs
5µs    YieldThread   avg-spins: 49       avg-actual: 5.042µs
100ns  None          avg-spins: 0        avg-actual: 100ns
100ns  SpinLoopHint  avg-spins: 0        avg-actual: 100ns
100ns  YieldThread   avg-spins: 1        avg-actual: 102ns
```

## spin_strategy_latency under load
Do the same measurement as above but while all cores are being stressed.

```sh
cargo run --bin spin_strategy_latency --release -- load
```

**Linux example output**
```
Simulating 16 thread load
warming up...
5ms-None:               avg-spins: 158992  avg-actual: 5.000057ms
5ms-SpinLoopHint:       avg-spins: 121884  avg-actual: 5.000243ms
5ms-YieldThread:        avg-spins: 23072   avg-actual: 5.000157ms
900µs-None:             avg-spins: 28287   avg-actual: 911.087µs
900µs-SpinLoopHint:     avg-spins: 21785   avg-actual: 906.957µs
900µs-YieldThread:      avg-spins: 4070    avg-actual: 914.979µs
5µs-None:               avg-spins: 158     avg-actual: 6.049µs
5µs-SpinLoopHint:       avg-spins: 121     avg-actual: 5.795µs
5µs-YieldThread:        avg-spins: 23      avg-actual: 24.089µs
100ns-None:             avg-spins: 2       avg-actual: 148ns
100ns-SpinLoopHint:     avg-spins: 2       avg-actual: 136ns
100ns-YieldThread:      avg-spins: 1       avg-actual: 274ns
```

**Windows example output**
```
Simulating 16 thread load
warming up...
5ms    None          avg-spins: 105568   avg-actual: 5.838449ms
5ms    SpinLoopHint  avg-spins: 79548    avg-actual: 5.608363ms
5ms    YieldThread   avg-spins: 1        avg-actual: 17.526351ms
900µs  None          avg-spins: 19461    avg-actual: 1.127537ms
900µs  SpinLoopHint  avg-spins: 14578    avg-actual: 1.326708ms
900µs  YieldThread   avg-spins: 1        avg-actual: 17.526448ms
5µs    None          avg-spins: 108      avg-actual: 5µs
5µs    SpinLoopHint  avg-spins: 79       avg-actual: 6.298µs
5µs    YieldThread   avg-spins: 1        avg-actual: 11.417271ms
100ns  None          avg-spins: 1        avg-actual: 101ns
100ns  SpinLoopHint  avg-spins: 0        avg-actual: 102ns
100ns  YieldThread   avg-spins: 0        avg-actual: 7.716038ms
```