//! Native, render-free execution of the same physical cellular engine.
use primordia_engine::{Config, Core};
use std::{env, fs, process, time::Instant};
fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().skip(1).collect();
    let mut config = Config::default();
    let mut cycles = 100.;
    let mut output = None;
    let mut input = None;
    let mut steering = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--help" {
            println!("physical_batch [--founders 256] [--cells 8192] [--cycles 100] [--size 128] [--shape sphere] [--spread global] [--layout patchy] [--seed 8042] [--input save.json] [--output save.json] [--steer <creator command JSON, repeatable>]\nSame fixed-dt physics as the browser, without rendering. Input resumes all chemical, fluid and genome state. The cell budget can be raised on resume.");
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
            "--layout" => config.resource_layout = value.clone(),
            "--seed" => config.seed = value.parse().map_err(|_| "Invalid seed")?,
            "--cycles" => cycles = value.parse::<f64>().map_err(|_| "Invalid cycle count")?,
            "--output" => output = Some(value.clone()),
            "--input" => input = Some(value.clone()),
            // Repeatable creator command, e.g. '{"kind":"mutation","scale":4}'.
            "--steer" => steering.push(value.clone()),
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
    for command in &steering {
        eprintln!("{}", world.steer(command)?);
    }
    // How many cells of each starter species the run begins with, after
    // anything it was asked to release: what the ledger below is read against.
    let mut began: std::collections::BTreeMap<String, u32> = Default::default();
    for c in &world.autonomous.as_ref().unwrap().cells {
        *began.entry(c.collection_sources.first().cloned().unwrap_or_default()).or_default() += 1;
    }
    let start = Instant::now();
    let initial_time = world.time;
    let ticks = (cycles * 4.).ceil() as u32;
    for tick in 0..ticks {
        world.step(1);
        if tick % 400 == 399 {
            let e = world.autonomous.as_ref().unwrap();
            let n = e.cells.len().max(1) as f32;
            let mean = |f: &dyn Fn(&primordia_engine::autonomous::Cell) -> f32| {
                e.cells.iter().map(f).sum::<f32>() / n
            };
            let mouths = e
                .cells
                .iter()
                .filter(|c| c.activity(primordia_engine::physical::ENGULF) > 0.25)
                .count();
            let eating = e.cells.iter().filter(|c| c.state.flux.eater()).count();
            let fed = e.cells.iter().filter(|c| c.state.flux.fed > 0.01).count();
            eprintln!(
                "{:.1} cycles, {} cells · mouths {} eating {} fed {} · energy {:.2} biomass {:.3} injury {:.2} exposure {:.2} age {:.0} · lost [starved, wasted, injured] {:?}",
                world.time,
                e.cells.len(),
                mouths,
                eating,
                fed,
                mean(&|c| c.energy),
                mean(&|c| c.material.as_ref().map_or(0., |m| m.biomass as f32)),
                mean(&|c| c.state.injury),
                mean(&|c| c.state.exposure),
                mean(&|c| c.age),
                e.physical.as_ref().unwrap().steering.losses
            );
        }
    }
    let seconds = start.elapsed().as_secs_f64();
    let e = world.autonomous.as_ref().unwrap();
    if !e.valid() {
        // Keep the evidence: write the invalid state where asked, then fail.
        if let Some(path) = &output {
            let _ = fs::write(path, serde_json::to_string(&world).unwrap_or_default());
        }
        let h = e.physical.as_ref().unwrap();
        eprintln!(
            "invalid: balance {:?} · steering {} · ledger {} · stores {} · flux {}",
            h.errors(&e.cells),
            h.steering.valid(),
            h.physiology.valid(),
            e.cells.iter().all(|c| c.material.as_ref().is_some_and(|m| m.proteins.iter().all(|p| p.is_finite() && *p >= 0.) && m.biomass >= 0. && m.fuel >= 0.)),
            e.cells.iter().all(|c| c.state.valid())
        );
        return Err("Final simulation state failed validation".into());
    }
    println!(
        "{}",
        serde_json::json!({"cycles":world.time-initial_time,"cells":e.cells.len(),"cellBudget":world.config.capacity,"divisions":e.divisions,"maxGeneration":e.peak_generation,"seconds":seconds,"actualSpeed":(world.time-initial_time)as f64/seconds,"balanceErrors":e.physical.as_ref().unwrap().errors(&e.cells)})
    );
    let view = e.physical.as_ref().unwrap().steering_view(e, &world);
    for rung in view["ladder"].as_array().unwrap() {
        eprintln!(
            "{:>9}  stage {}  genes {:>5.1}%  built {:>5.1}%",
            rung["key"].as_str().unwrap(),
            rung["stage"],
            rung["genes"].as_f64().unwrap() * 100.,
            rung["built"].as_f64().unwrap() * 100.
        );
    }
    eprintln!(
        "diet [light, soup, remains, vents, herbivore, omnivore, carnivore] {} · complex-cell captures {}\nkinds [microbe, plant-like, fungus-like, animal-like] {}\nspecies {}\noxygen {:.2} breath {:.2} restocked {}\nsky impacts {} airbursts {} air {:.2} rock nitrogen {:.0} ozone {:.2} hard rays {:.2}",
        view["diet"], view["captures"], view["kinds"], view["species"],
        view["oxygen"].as_f64().unwrap_or(0.), view["breath"].as_f64().unwrap_or(0.), view["restocked"],
        view["sky"]["impacts"], view["sky"]["airbursts"], view["sky"]["airColumn"].as_f64().unwrap_or(0.), view["sky"]["rockNitrogen"].as_f64().unwrap_or(0.), view["sky"]["ozone"].as_f64().unwrap_or(0.), view["sky"]["hardRays"].as_f64().unwrap_or(0.)
    );
    eprintln!("land {}", view["land"]);
    // The family tree of each starter species: its living lines, what each
    // gained or lost against the line it arose from, and when.
    let names = primordia_engine::steering::TRAIT_NAMES;
    let list = |mask: u64| (0..names.len()).filter(|b| mask >> b & 1 == 1).map(|b| names[b]).collect::<Vec<_>>().join(", ");
    for tree in view["trees"].as_array().unwrap() {
        eprintln!("tree {} ({} cells, {} lines)", tree["species"].as_str().unwrap(), tree["cells"], tree["lines"]);
        for b in tree["branches"].as_array().unwrap() {
            let t = b["traits"].as_u64().unwrap();
            match b["from"].as_u64() {
                None => eprintln!("  root: {} · {} cells", list(t), b["cells"]),
                Some(f) => eprintln!(
                    "  line: gained [{}] lost [{}] · {} cells · {}",
                    list(t & !f), list(f & !t), b["cells"],
                    b["since"].as_f64().map_or("before records began".to_string(), |s| format!("since cycle {s:.0}"))
                ),
            }
        }
    }
    // What each starter species died of during this run: starved, wasted
    // away, then each kind of injury.
    let h = e.physical.as_ref().unwrap();
    let causes: Vec<&str> =
        ["starved", "wasted"].into_iter().chain(primordia_engine::physiology::HARMS).collect();
    // One line a species for the whole run: cells at the start and the end,
    // born and died, and the yearly-interest figure of it: growth per hundred
    // cycles, which is what says whether a species is holding.
    let mut ended: std::collections::BTreeMap<String, u32> = Default::default();
    for c in &e.cells {
        *ended.entry(c.collection_sources.first().cloned().unwrap_or_default()).or_default() += 1;
    }
    let span = (world.time - initial_time).max(1e-6) as f64;
    for (name, &from) in &began {
        let to = ended.get(name).copied().unwrap_or(0);
        let born = h.births.get(name).copied().unwrap_or(0);
        let died: u32 = h.deaths.get(name).map_or(0, |r| r.iter().sum());
        let rate = if from > 0 && to > 0 { ((to as f64 / from as f64).ln() * 100. / span).exp() - 1. } else { -1. };
        eprintln!(
            "tally {:<14} {:>6} -> {:<6} born {:>6} died {:>6}  {:>+7.1}% a hundred cycles",
            if name.is_empty() { "(evolved)" } else { name },
            from, to, born, died, rate * 100.
        );
    }
    for (name, row) in &h.deaths {
        let all: u32 = row.iter().sum();
        let mut parts: Vec<(u32, &str)> = row.iter().copied().zip(causes.iter().copied()).filter(|p| p.0 > 0).collect();
        parts.sort_by(|a, b| b.0.cmp(&a.0));
        eprintln!(
            "died {:<14} {:>6}: {}",
            if name.is_empty() { "(evolved)" } else { name },
            all,
            parts.iter().map(|(n, c)| format!("{c} {n}")).collect::<Vec<_>>().join(", ")
        );
    }
    let mut sizes: Vec<usize> = e.groups().iter().map(Vec::len).collect();
    sizes.sort_unstable();
    let at_least = |n: usize| sizes.iter().filter(|s| **s >= n).count();
    let in_bodies: usize = sizes.iter().filter(|s| **s >= 4).sum();
    // And who ate each of them, by how much living tissue was taken.
    let mut eaten: std::collections::BTreeMap<&str, Vec<(f64, &str)>> = Default::default();
    for ((prey, eater), amount) in &h.taken {
        eaten.entry(prey).or_default().push((*amount, eater));
    }
    for (prey, mut by) in eaten {
        by.sort_by(|a, b| b.0.total_cmp(&a.0));
        let all: f64 = by.iter().map(|p| p.0).sum();
        eprintln!(
            "eaten {:<14} {:>7.1}: {}",
            if prey.is_empty() { "(evolved)" } else { prey },
            all,
            by.iter().take(6).map(|(n, c)| format!("{} {:.0}%", if c.is_empty() { "(evolved)" } else { c }, n * 100. / all)).collect::<Vec<_>>().join(", ")
        );
    }
    // The same from the eater's side: what each species lived on.
    let mut ate: std::collections::BTreeMap<&str, Vec<(f64, &str)>> = Default::default();
    for ((prey, eater), amount) in &h.taken {
        ate.entry(eater).or_default().push((*amount, prey));
    }
    for (eater, mut of) in ate {
        of.sort_by(|a, b| b.0.total_cmp(&a.0));
        let all: f64 = of.iter().map(|p| p.0).sum();
        eprintln!(
            "ate {:<14} {:>7.1}: {}",
            if eater.is_empty() { "(evolved)" } else { eater },
            all,
            of.iter().take(8).map(|(n, c)| format!("{} {:.0}%", if c.is_empty() { "(evolved)" } else { c }, n * 100. / all)).collect::<Vec<_>>().join(", ")
        );
    }
    eprintln!(
        "bodies largest {} · of 4+ {} · of 10+ {} · of 30+ {} · cells in bodies of 4+ {:.0}%",
        sizes.last().copied().unwrap_or(0), at_least(4), at_least(10), at_least(30),
        in_bodies as f64 * 100. / e.cells.len().max(1) as f64
    );
    let cycles = e.physical.as_ref().unwrap().cycles(e);
    let part = |pool: &str, keys: &[&str]| {
        keys.iter()
            .map(|k| format!("{k} {:.1}", cycles[pool][*k].as_f64().unwrap_or(0.)))
            .collect::<Vec<_>>()
            .join(" ")
    };
    eprintln!(
        "carbon {}\nnitrogen {} short {:.2}\nlevels {}\nefficiency {}",
        part("carbon", &["dioxide", "dissolved", "living", "remains"]),
        part("nitrogen", &["air", "dissolved", "living", "remains"]),
        e.cells.iter().map(|c| c.state.nitrogen_short as f64).sum::<f64>() / e.cells.len().max(1) as f64,
        cycles["levels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| format!(
                "{} {} cells {:.1}C in {:.3}",
                l["name"].as_str().unwrap(),
                l["cells"],
                l["carbon"].as_f64().unwrap(),
                l["intake"].as_f64().unwrap()
            ))
            .collect::<Vec<_>>()
            .join(" | "),
        cycles["efficiency"]
    );
    eprintln!(
        "breeding: {} rounds, {} cells removed, last round '{}'",
        view["rounds"], view["culled"], view["note"].as_str().unwrap()
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
