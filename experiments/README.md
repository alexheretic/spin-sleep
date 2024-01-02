# spin_sleep experiments
Experiments to measure latency all machine specific & non-deterministic but are used to determine
good default settings for _spin_sleep_.

## native_sleep_accuracy
Call OS native sleep for **1ns, 1µs & 1ms** and see how long it actually takes.

```sh
cargo run --bin native_sleep_accuracy --release
```

**Linux example output** *
```
==> sleep 1ns
average: 54.0µs, best: 8.7µs, worst: 94.1µs
==> sleep 1µs
average: 55.1µs, best: 8.3µs, worst: 60.4µs
==> sleep 1ms
average: 1.055ms, best: 1.054ms, worst: 1.058ms
```

**Windows example output** *
```
==> sleep 1ns
average: 2.0µs, best: 1.3µs, worst: 13.9µs
==> sleep 1µs
average: 446.7µs, best: 2.3µs, worst: 725.8µs
==> sleep 1ms
average: 1.775ms, best: 1.502ms, worst: 2.012ms
```

### Under high load
Do the same measurement as above but while all cores are being stressed.
```sh
cargo run --bin native_sleep_accuracy --release -- load
```

**Linux example output** *
Generally similar to no load, but more likely to produce occasional high latency.
```
Simulating 16 thread load
==> sleep 1ns
average: 53.8µs, best: 7.1µs, worst: 231.6µs
==> sleep 1µs
average: 58.0µs, best: 7.6µs, worst: 3.3ms
==> sleep 1ms
average: 1.054ms, best: 1.054ms, worst: 1.055ms
```

**Windows example output** *
High latency is fairly common.
```
Simulating 16 thread load
==> sleep 1ns
average: 39.3µs, best: 1.8µs, worst: 36.8ms
==> sleep 1µs
average: 14.9ms, best: 2.1µs, worst: 46.9ms
==> sleep 1ms
average: 16.025ms, best: 2.004ms, worst: 30.071ms
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
5ms    None          avg-spins: 231633   avg-actual: 5.000039ms
5ms    SpinLoopHint  avg-spins: 168571   avg-actual: 5.000041ms
5ms    YieldThread   avg-spins: 8431     avg-actual: 5.000323ms
900µs  None          avg-spins: 41194    avg-actual: 900.039µs
900µs  SpinLoopHint  avg-spins: 30094    avg-actual: 900.044µs
900µs  YieldThread   avg-spins: 1527     avg-actual: 900.349µs
5µs    None          avg-spins: 231      avg-actual: 5.033µs
5µs    SpinLoopHint  avg-spins: 167      avg-actual: 5.063µs
5µs    YieldThread   avg-spins: 9        avg-actual: 5.229µs
100ns  None          avg-spins: 4        avg-actual: 129ns
100ns  SpinLoopHint  avg-spins: 3        avg-actual: 132ns
100ns  YieldThread   avg-spins: 1        avg-actual: 625ns
```

**Windows example output** *
```
warming up...
5ms    None          avg-spins: 176820   avg-actual: 5ms
5ms    SpinLoopHint  avg-spins: 164060   avg-actual: 5ms
5ms    YieldThread   avg-spins: 31789    avg-actual: 5.000064ms
900µs  None          avg-spins: 31791    avg-actual: 900µs
900µs  SpinLoopHint  avg-spins: 29406    avg-actual: 900.021µs
900µs  YieldThread   avg-spins: 5700     avg-actual: 900.063µs
5µs    None          avg-spins: 139      avg-actual: 5µs
5µs    SpinLoopHint  avg-spins: 160      avg-actual: 5µs
5µs    YieldThread   avg-spins: 31       avg-actual: 5.09µs
100ns  None          avg-spins: 0        avg-actual: 100ns
100ns  SpinLoopHint  avg-spins: 0        avg-actual: 100ns
100ns  YieldThread   avg-spins: 0        avg-actual: 172ns
```

### Under high load
Do the same measurement as above but while all cores are being stressed.

```sh
cargo run --bin spin_strategy_latency --release -- load
```

**Linux example output** *
```
Simulating 16 thread load
warming up...
5ms    None          avg-spins: 170998   avg-actual: 5.374337ms
5ms    SpinLoopHint  avg-spins: 110830   avg-actual: 5.385263ms
5ms    YieldThread   avg-spins: 6457     avg-actual: 5.000448ms
900µs  None          avg-spins: 34035    avg-actual: 900.045µs
900µs  SpinLoopHint  avg-spins: 21661    avg-actual: 900.051µs
900µs  YieldThread   avg-spins: 1132     avg-actual: 900.54µs
5µs    None          avg-spins: 186      avg-actual: 5.18µs
5µs    SpinLoopHint  avg-spins: 117      avg-actual: 5.124µs
5µs    YieldThread   avg-spins: 6        avg-actual: 5.621µs
100ns  None          avg-spins: 3        avg-actual: 128ns
100ns  SpinLoopHint  avg-spins: 2        avg-actual: 131ns
100ns  YieldThread   avg-spins: 1        avg-actual: 898ns
```

**Windows example output** *
```
Simulating 16 thread load
warming up...
5ms    None          avg-spins: 140709   avg-actual: 5.604986ms
5ms    SpinLoopHint  avg-spins: 108241   avg-actual: 5.81583ms
5ms    YieldThread   avg-spins: 3        avg-actual: 32.039572ms
900µs  None          avg-spins: 27701    avg-actual: 902.595µs
900µs  SpinLoopHint  avg-spins: 20202    avg-actual: 1.210891ms
900µs  YieldThread   avg-spins: 1        avg-actual: 11.297962ms
5µs    None          avg-spins: 153      avg-actual: 5µs
5µs    SpinLoopHint  avg-spins: 110      avg-actual: 5µs
5µs    YieldThread   avg-spins: 1        avg-actual: 13.948654ms
100ns  None          avg-spins: 0        avg-actual: 100ns
100ns  SpinLoopHint  avg-spins: 0        avg-actual: 100ns
100ns  YieldThread   avg-spins: 0        avg-actual: 2.882577ms
```

\* _Measured 2023-01-02 with a AMD 5800X_.
