use primordia_engine::{Config, Core};
use std::time::Instant;
fn main() {
    for population in [1000, 4000, 12000] {
        let mut config = Config::default();
        config.capacity = population;
        let mut world = Core::new(config).unwrap();
        while world.organisms.len() < population {
            world.seed(1000);
        }
        let start = Instant::now();
        world.step(200);
        let elapsed = start.elapsed();
        println!(
            "{population} initial organisms: {:.2} ms/tick; {} survivors, {} births, {} colonies",
            elapsed.as_secs_f64() * 1000.0 / 200.0,
            world.organisms.len(),
            world.births,
            world.organisms.iter().filter(|o| o.cells > 1).count()
        );
    }
}
