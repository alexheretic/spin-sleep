//! Call OS native sleep for **1ns** and see how long it actually takes.
use std::time::{Duration, Instant};

fn main() {
    if cfg!(debug_assertions) {
        eprintln!("Should run with `--release`");
        std::process::exit(1);
    }

    if std::env::args().nth(1).as_deref() == Some("load") {
        let cpus = std::thread::available_parallelism().unwrap().into();
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

    const ITS: u32 = 1000;

    let mut best = Duration::MAX;
    let mut sum = Duration::ZERO;
    let mut worst = Duration::ZERO;

    for _ in 0..ITS {
        let before = Instant::now();
        spin_sleep::native_sleep(Duration::new(0, 1));
        let elapsed = before.elapsed();
        sum += elapsed;
        if elapsed < best {
            best = elapsed;
        }
        if elapsed > worst {
            worst = elapsed;
        }
    }

    println!(
        "average: {:.1?}, best: {best:.1?}, worst: {worst:.1?}",
        sum / ITS
    );
}
