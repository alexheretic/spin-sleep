use spin_sleep::SpinStrategy;
use std::time::{Duration, Instant};

fn main() {
    if cfg!(debug_assertions) {
        eprintln!("Should run with `--release`");
        std::process::exit(1);
    }

    if std::env::args().nth(1).as_deref() == Some("load") {
        let cpus = num_cpus::get();
        eprintln!("Simulating {cpus} thread load");
        for _ in 0..cpus {
            std::thread::spawn(|| {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                while rng.gen::<u64>() > 0 {}
            });
        }

        std::thread::sleep(Duration::from_secs(1));
    }

    // warmup
    eprintln!("warming up...");
    for _ in 0..200 {
        let before = Instant::now();
        while before.elapsed() < Duration::from_millis(5) {}
    }

    for duration in [
        Duration::from_millis(5),
        Duration::from_micros(900),
        Duration::from_micros(5),
        Duration::from_nanos(100),
    ] {
        for strategy in [
            None,
            Some(SpinStrategy::SpinLoopHint),
            Some(SpinStrategy::YieldThread),
        ] {
            let mut sum = Duration::from_secs(0);
            let mut spins = 0_u32;

            for _ in 0..100 {
                let before = Instant::now();
                while before.elapsed() < duration {
                    match strategy {
                        Some(SpinStrategy::YieldThread) => std::thread::yield_now(),
                        Some(SpinStrategy::SpinLoopHint) => std::hint::spin_loop(),
                        None => {}
                    }
                    spins += 1;
                }
                let elapsed = before.elapsed();
                sum += elapsed;
            }
            println!(
                "{duration:?}-{}\tavg-spins: {:<7} avg-actual: {:?}",
                strategy
                    .map(|s| format!("{s:?}:"))
                    .unwrap_or_else(|| "None:        ".into()),
                spins / 100,
                Duration::from_nanos((sum.subsec_nanos() / 100).into()),
            );
        }
    }
}
