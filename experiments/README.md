# spin_sleep experiments
Experiments to measure latency all machine specific & non-deterministic but are used to determine
good default settings for _spin_sleep_.

## native_sleep_accuracy
Call OS native sleep for **1ns** and see how long it actually takes.

```sh
cargo run --bin native_sleep_accuracy --release
```

**Linux example output** *
```
average: 53.04µs, best : 7.95µs, worst: 85.238µs
```

**Windows example output** *
```
average: 2.012432ms, best : 2.0069ms, worst: 2.1455ms
```

## spin_strategy_latency
Measure `SpinStrategy` latencies and spin counts across various wait durations
_5ms, 900µs, 5µs, 100ns_.

```sh
cargo run --bin spin_strategy_latency --release
```

**Linux example output** *
```
warming up...
5ms    None          avg-spins: 191610   avg-actual: 5.000044ms
5ms    SpinLoopHint  avg-spins: 176594   avg-actual: 5.000045ms
5ms    YieldThread   avg-spins: 38366    avg-actual: 5.000105ms
900µs  None          avg-spins: 34340    avg-actual: 900.05µs
900µs  SpinLoopHint  avg-spins: 31633    avg-actual: 900.052µs
900µs  YieldThread   avg-spins: 6843     avg-actual: 900.104µs
5µs    None          avg-spins: 186      avg-actual: 5.04µs
5µs    SpinLoopHint  avg-spins: 173      avg-actual: 5.048µs
5µs    YieldThread   avg-spins: 38       avg-actual: 5.075µs
100ns  None          avg-spins: 3        avg-actual: 135ns
100ns  SpinLoopHint  avg-spins: 3        avg-actual: 132ns
100ns  YieldThread   avg-spins: 1        avg-actual: 181ns
```

**Windows example output** *
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

**Linux example output** *
```
Simulating 16 thread load
warming up...
5ms    None          avg-spins: 159018   avg-actual: 5.000058ms
5ms    SpinLoopHint  avg-spins: 122263   avg-actual: 5.000065ms
5ms    YieldThread   avg-spins: 23265    avg-actual: 5.000327ms
900µs  None          avg-spins: 27748    avg-actual: 938.427µs
900µs  SpinLoopHint  avg-spins: 21727    avg-actual: 900.062µs
900µs  YieldThread   avg-spins: 4054     avg-actual: 901.31µs
5µs    None          avg-spins: 157      avg-actual: 5.055µs
5µs    SpinLoopHint  avg-spins: 122      avg-actual: 5.057µs
5µs    YieldThread   avg-spins: 23       avg-actual: 5.07µs
100ns  None          avg-spins: 2        avg-actual: 147ns
100ns  SpinLoopHint  avg-spins: 1        avg-actual: 135ns
100ns  YieldThread   avg-spins: 1        avg-actual: 278ns
```

**Windows example output** *
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

\* _Measured 2022-02-18 with a AMD 5800X_.
