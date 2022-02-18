use std::time::{Duration, Instant};

fn main() {
    if cfg!(debug_assertions) {
        eprintln!("Should run with `--release`");
        std::process::exit(1);
    }

    let mut best = Duration::from_secs(100);
    let mut sum = Duration::from_secs(0);
    let mut worst = Duration::from_secs(0);

    for _ in 0..100 {
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
        "average: {:?}, best : {:?}, worst: {:?}",
        Duration::from_nanos((sum.subsec_nanos() / 100).into()),
        best,
        worst,
    );
}
