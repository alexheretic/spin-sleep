//! Call OS native sleep for **1ns** and see how long it actually takes.
use std::time::{Duration, Instant};

fn main() {
    if cfg!(debug_assertions) {
        eprintln!("Should run with `--release`");
        std::process::exit(1);
    }

    const ITS: u32 = 1000;

    let mut best = Duration::from_secs(100);
    let mut sum = Duration::from_secs(0);
    let mut worst = Duration::from_secs(0);

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
        "average: {:?}, best : {best:?}, worst: {worst:?}",
        sum / ITS
    );
}
