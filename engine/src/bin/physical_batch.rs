//! Native, render-free execution of the same physical cellular engine.
use primordia_engine::{Config, Core};
use std::{env, fs, process, time::Instant};
fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().skip(1).collect();
    let mut config = Config::default();
    let mut cycles = 100.;
    let mut output = None;
    let mut input = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--help" {
            println!("physical_batch [--founders 256] [--cells 8192] [--cycles 100] [--size 128] [--shape sphere] [--spread global] [--seed 8042] [--input save.json] [--output save.json]\nSame fixed-dt physics as the browser, without rendering. Input resumes all chemical, fluid and genome state. The cell budget can be raised on resume.");
            return Ok(());
        }
        let value = args.get(i + 1).ok_or("Each option requires a value")?;
        match args[i].as_str() {
            "--founders" => {
                config.initial_cells = value.parse().map_err(|_| "Invalid founder count")?
            }
            "--cells" => config.capacity = value.parse().map_err(|_| "Invalid cell ceiling")?,
            "--size" => config.world_size = value.parse().map_err(|_| "Invalid world size")?,
            "--shape" => {
                config.world_shape = match value.as_str() {
                    "sphere" => primordia_engine::domain::Shape::Sphere,
                    "planar" => primordia_engine::domain::Shape::Planar,
                    _ => return Err("Shape must be sphere or planar".into()),
                }
            }
            "--spread" => config.founder_spread = value.clone(),
            "--seed" => config.seed = value.parse().map_err(|_| "Invalid seed")?,
            "--cycles" => cycles = value.parse::<f64>().map_err(|_| "Invalid cycle count")?,
            "--output" => output = Some(value.clone()),
            "--input" => input = Some(value.clone()),
            _ => return Err(format!("Unknown option: {}", args[i])),
        }
        i += 2;
    }
    if !cycles.is_finite() || cycles <= 0. || cycles > 1_000_000. {
        return Err("Cycles must be greater than zero and at most 1000000".into());
    }
    let mut world = if let Some(path) = input {
        let mut c = Core::load(&fs::read_to_string(path).map_err(|e| e.to_string())?)?;
        let e = c
            .autonomous
            .as_ref()
            .ok_or("Input must contain a physical habitat")?;
        if e.physical.is_none() {
            return Err("Input must contain a physical habitat".into());
        }
        if args.iter().any(|a| {
            ["--size", "--shape", "--spread", "--founders", "--seed"].contains(&a.as_str())
        }) {
            return Err("Geometry, founders and seed apply only to new worlds".into());
        }
        if args.iter().any(|a| a == "--cells") {
            if !(100..=64000).contains(&config.capacity)
                || config.capacity < e.cells.len().max(c.config.initial_cells)
            {
                return Err(
                    "Requested budget must fit the living and starting population, up to 64000"
                        .into(),
                );
            }
            c.config.capacity = config.capacity;
        }
        c
    } else {
        let mut c = Core::new(config)?;
        c.start_physical();
        c
    };
    let start = Instant::now();
    let initial_time = world.time;
    let ticks = (cycles * 4.).ceil() as u32;
    for tick in 0..ticks {
        world.step(1);
        if tick % 400 == 399 {
            eprintln!(
                "{:.1} cycles, {} cells",
                world.time,
                world.autonomous.as_ref().unwrap().cells.len()
            );
        }
    }
    let seconds = start.elapsed().as_secs_f64();
    let e = world.autonomous.as_ref().unwrap();
    if !e.valid() {
        return Err("Final simulation state failed validation".into());
    }
    println!(
        "{}",
        serde_json::json!({"cycles":world.time-initial_time,"cells":e.cells.len(),"cellBudget":world.config.capacity,"divisions":e.divisions,"maxGeneration":e.peak_generation,"seconds":seconds,"actualSpeed":(world.time-initial_time)as f64/seconds,"balanceErrors":e.physical.as_ref().unwrap().errors(&e.cells)})
    );
    if let Some(path) = output {
        fs::write(
            path,
            serde_json::to_string(&world).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}
