//! Supplied-mechanism fixtures for movement, attack, defence and chemosynthesis.
//! They check what each material does once a cell has it, never whether
//! ordinary founders evolve it.
use crate::{
    autonomous::{Bond, Evolution, Gene, INPUTS, LATERAL},
    body::V3,
    physical::{Habitat, CHEMO, ENGULF, FIN, PROPEL, SPINE, STEER, TOXIN, WALL},
    physiology::RECEPTOR,
    *,
};

/// Two free cells in open water, touching, with full energy.
fn fixture() -> (Core, Evolution, Habitat) {
    let mut core = Core::new(Config {
        initial_cells: 2,
        ..Config::default()
    })
    .unwrap();
    let mut e = Evolution::new(&mut core);
    let mut h = Habitat::new(&core, &mut e.cells);
    for (i, c) in e.cells.iter_mut().enumerate() {
        c.p = V3::new(i as f32 * 0.4, 5., 0.);
        c.v = V3::default();
        c.heading = 0.;
        c.energy = 8.;
        c.cooldown = 1e6; // these fixtures are about behaviour, not division
                          // Not sticky: two cells that bond become one body, and a body does
                          // not attack itself.
        c.material.as_mut().unwrap().proteins[2] = 0.;
        c.genome.id = 100 + i as u32;
        // One silent gene: a genome may not be empty.
        c.genome.genes = vec![Gene {
            action: 10,
            promoter: [0.; INPUTS],
            bias: -2.,
            strength: 0.,
            fates: 15,
        }];
    }
    e.next_genome = 200;
    e.bonds.clear();
    h.oxygenate(1.); // these fixtures breathe freely
    rebase(&mut h, &e);
    (core, e, h)
}
fn rebase(h: &mut Habitat, e: &Evolution) {
    h.rebase(&e.cells);
    h.initial_energy += h.heat + h.mechanical_input - h.photon_energy - h.geothermal_energy;
}
fn build(e: &mut Evolution, cell: usize, material: usize) {
    // Enough for near-full activity without changing the cell's size much.
    e.cells[cell].material.as_mut().unwrap().proteins[material] = 0.02;
}
fn always(action: usize) -> Gene {
    Gene {
        action,
        promoter: [0.; INPUTS],
        bias: 2.,
        strength: 1.,
        fates: 15,
    }
}
fn biomass(e: &Evolution, cell: usize) -> f64 {
    e.cells[cell].material.as_ref().unwrap().biomass
}
fn balanced(h: &Habitat, e: &Evolution) -> bool {
    h.errors(&e.cells).iter().all(|x| x.abs() < 1e-6)
}

#[test]
fn propulsion_moves_a_cell_along_its_heading_and_is_paid_for() {
    // The same cell in the same water, with and without a driven motor.
    let run = |motor: bool| {
        let (mut core, mut e, mut h) = fixture();
        e.cells[1].p = V3::new(0., 5., 6.); // well apart: no contact
        if motor {
            build(&mut e, 0, PROPEL);
            e.cells[0].genome.genes.push(always(PROPEL));
        }
        rebase(&mut h, &e);
        let start = e.cells[0].p.x;
        for _ in 0..24 {
            h.tick(&mut e, &mut core);
        }
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        assert!(h.valid(&e.cells), "fluid and stores stay in range");
        (
            e.cells[0].p.x - start,
            e.cells[0].state.motor_work,
            e.cells[0].energy,
        )
    };
    let (drift, idle_work, idle_energy) = run(false);
    let (swam, work, energy) = run(true);
    assert!(swam > drift + 0.25, "swam {swam}, drifted {drift}");
    assert!(work > 0. && idle_work == 0.);
    assert!(energy < idle_energy, "thrust is paid for");
}

#[test]
fn a_bodys_motors_push_the_whole_body_without_tearing_it() {
    // A swimmer bonded to a passive cell: both travel, and the bond survives.
    let (mut core, mut e, mut h) = fixture();
    build(&mut e, 0, PROPEL);
    e.cells[0].genome.genes.push(always(PROPEL));
    for c in &mut e.cells {
        c.material.as_mut().unwrap().proteins[2] = 0.05; // adhesion keeps a bond alive
    }
    e.bonds.push(Bond {
        a: e.cells[0].id,
        b: e.cells[1].id,
        rest: 0.4,
        integrity: 1.,
        activation: 0.,
        stiffness: 0.,
    });
    let start = (e.cells[0].p.x, e.cells[1].p.x);
    for _ in 0..40 {
        h.tick(&mut e, &mut core);
    }
    let moved = (e.cells[0].p.x - start.0, e.cells[1].p.x - start.1);
    assert_eq!(e.bonds.len(), 1, "the body holds together");
    assert!(
        e.bonds[0].integrity > 0.9,
        "bond integrity {}",
        e.bonds[0].integrity
    );
    assert!(moved.0 > 0.1 && moved.1 > 0.1, "both travelled: {moved:?}");
    assert!(
        (moved.0 - moved.1).abs() < 0.2,
        "and stayed together: {moved:?}"
    );
    assert!(e.cells[0].state.injury < 0.05 && e.cells[1].state.injury < 0.05);
}

#[test]
fn a_motor_without_its_gene_drive_or_without_energy_does_nothing() {
    let (mut core, mut e, mut h) = fixture();
    e.cells[1].p = V3::new(0., 5., 6.);
    build(&mut e, 0, PROPEL); // material, but no gene expressing a drive
    build(&mut e, 1, PROPEL);
    e.cells[1].genome.genes.push(always(PROPEL));
    e.cells[1].energy = 0.01; // a drive, but nothing to pay with
    let start = (e.cells[0].p.x, e.cells[1].p.x);
    for _ in 0..12 {
        h.tick(&mut e, &mut core);
    }
    assert!((e.cells[0].p.x - start.0).abs() < 0.05);
    assert!(e.cells.len() < 2 || (e.cells[1].p.x - start.1).abs() < 0.05);
    // Disabling actuation for a comparison also stops swimming.
    let (mut core, mut e, mut h) = fixture();
    e.cells[1].p = V3::new(0., 5., 6.);
    build(&mut e, 0, PROPEL);
    e.cells[0].genome.genes.push(always(PROPEL));
    h.passive = true;
    let start = e.cells[0].p.x;
    for _ in 0..12 {
        h.tick(&mut e, &mut core);
    }
    assert!((e.cells[0].p.x - start).abs() < 0.05);
}

#[test]
fn steering_turns_toward_or_away_from_what_the_cell_senses() {
    let turned = |weight: f32, receptor: bool| {
        let (mut core, mut e, mut h) = fixture();
        e.cells[1].p = V3::new(20., 5., 20.);
        // Rich fuel to the left of a cell heading along +x (left is +z).
        for i in 0..h.domain.sites() {
            let (_, z) = h.domain.point(i);
            h.solutes[i][0] = if z > 0. { 6. } else { 0.2 };
        }
        e.cells[0].p = V3::new(0., 5., 0.);
        build(&mut e, 0, STEER);
        if receptor {
            build(&mut e, 0, RECEPTOR);
        }
        let mut promoter = [0.; INPUTS];
        promoter[INPUTS - LATERAL] = weight;
        e.cells[0].genome.genes.push(Gene {
            action: STEER,
            promoter,
            bias: 0.,
            strength: 1.,
            fates: 15,
        });
        for _ in 0..8 {
            h.tick(&mut e, &mut core);
        }
        let heading = e.cells[0].heading;
        // Signed turn in (-pi, pi].
        if heading > std::f32::consts::PI {
            heading - std::f32::consts::TAU
        } else {
            heading
        }
    };
    assert!(turned(1., true) > 0.15, "attracted: {}", turned(1., true));
    assert!(turned(-1., true) < -0.15, "repelled: {}", turned(-1., true));
    let crude = turned(1., false);
    assert!(
        crude > 0. && crude < turned(1., true),
        "steering alone senses crudely: {crude}"
    );
}

#[test]
fn engulfing_needs_a_size_advantage_and_spines_blunt_and_punish_it() {
    let meal = |grow: bool, spines: bool| {
        let (mut core, mut e, mut h) = fixture();
        build(&mut e, 0, ENGULF);
        if grow {
            e.cells[0].material.as_mut().unwrap().biomass *= 2.;
            e.cells[1].p = V3::new(e.cells[0].radius() + e.cells[1].radius(), 5., 0.);
        }
        if spines {
            build(&mut e, 1, SPINE);
        }
        rebase(&mut h, &e);
        let before = biomass(&e, 1);
        h.tick(&mut e, &mut core);
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        let _ = before;
        let prey = e.cells.iter().find(|c| c.genome.id == 101);
        (
            prey.map_or(before, |c| c.state.flux.lost_to_predators),
            e.cells[0].state.injury,
            e.cells[0].state.flux.prey,
        )
    };
    let (equal, _, _) = meal(false, false);
    // Prey a quarter larger than the hunter cannot be swallowed at all.
    let oversized = {
        let (mut core, mut e, mut h) = fixture();
        build(&mut e, 0, ENGULF);
        e.cells[1].material.as_mut().unwrap().biomass *= 1.3;
        e.cells[1].p = V3::new(e.cells[0].radius() + e.cells[1].radius(), 5., 0.);
        h.tick(&mut e, &mut core);
        e.cells[1].state.flux.lost_to_predators
    };
    assert_eq!(oversized, 0.);
    let (eaten, unhurt, gained) = meal(true, false);
    let (blunted, pricked, _) = meal(true, true);
    assert!(
        equal > 0. && equal < eaten * 0.5,
        "an equal goes down slowly: {equal} vs {eaten}"
    );
    assert!(eaten > 0.02 && gained > 0., "bigger cell ate {eaten}");
    assert!(blunted < eaten * 0.5, "spines: {blunted} vs {eaten}");
    assert!(pricked > unhurt, "spines wound the eater");
}

#[test]
fn a_cell_that_harvests_light_cannot_take_prey_until_it_gives_the_light_up() {
    // The same hunter and the same grazer, with and without light-harvesting machinery.
    for weapon in [ENGULF, 9] {
        let taken = |light: f64| {
            let (mut core, mut e, mut h) = fixture();
            build(&mut e, 0, weapon);
            let hunter = e.cells[0].material.as_mut().unwrap();
            hunter.proteins[0] = light;
            hunter.biomass *= 2.;
            e.cells[1].p = V3::new(e.cells[0].radius() + e.cells[1].radius(), 5., 0.);
            rebase(&mut h, &e);
            h.tick(&mut e, &mut core);
            assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
            e.cells[0].state.flux.prey
        };
        let (animal, founder, plant) = (taken(0.), taken(0.002), taken(0.05));
        eprintln!("weapon {weapon}: prey taken with no light harvest {animal}, a founder's {founder}, a plant's {plant}");
        assert!(animal > 0., "an animal eats");
        assert!(founder < animal * 0.75, "even a little light harvest costs some of the bite: {founder} against {animal}");
        assert!(plant < animal * 0.02, "a plant cannot attack: {plant} against {animal}");
    }
}

#[test]
fn light_feeders_free_oxygen_the_air_carries_it_and_breathing_doubles_what_fuel_gives() {
    // One cell with fuel to burn, in air with all of its oxygen free or none.
    let gained = |air: f64| {
        let (mut core, mut e, mut h) = fixture();
        h.oxygenate(air);
        for c in &mut e.cells {
            c.energy = 1.;
            let m = c.material.as_mut().unwrap();
            m.fuel = 0.3;
            m.proteins[0] = 0.; // nothing here makes oxygen of its own
        }
        rebase(&mut h, &e);
        for _ in 0..4 {
            h.tick(&mut e, &mut core);
        }
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        // Energy captured for each unit of fuel burned.
        let burned: f64 = e.cells.iter().map(|c| c.state.flux.waste).sum();
        (e.food_energy / burned, e.cells[0].state.breath)
    };
    let ((breathing, full), (fermenting, none)) = (gained(1.), gained(0.));
    eprintln!("energy from the same fuel: {breathing} breathing, {fermenting} without oxygen");
    assert!(full > 0.9 && none < 0.05, "breath {full} and {none}");
    assert!(fermenting > 0. && (breathing / fermenting - 2.).abs() < 0.2, "{breathing} against {fermenting}");

    // A light-feeder in air with no free oxygen makes some, and the air
    // carries it to seas where nothing grows.
    let (mut core, mut e, mut h) = fixture();
    h.oxygenate(0.);
    e.cells[0].genome.genes = vec![always(0)];
    build(&mut e, 0, 0);
    for s in &mut h.solutes {
        s[1] = 1.;
    }
    rebase(&mut h, &e);
    for _ in 0..40 {
        h.tick(&mut e, &mut core);
    }
    let here = h.domain.locate(e.cells[0].p);
    let far = (here + h.domain.sites() / 2) % h.domain.sites();
    assert!(e.cells[0].state.flux.photosynthesis > 0.);
    assert!(h.oxygen() > 0., "oxygen was freed");
    assert!(h.systems.gas[far][0] > 0., "and reached the far side: {:?}", h.systems.gas[far]);
    assert!(h.systems.gas_error(&e.cells).abs() < 1e-9, "none was made from nothing");
    assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
}

#[test]
fn a_swimmer_covers_body_lengths_for_little_fuel_in_thin_water() {
    let (mut core, mut e, mut h) = fixture();
    e.cells[1].p = V3::new(0., 5., 9.);
    build(&mut e, 0, PROPEL);
    e.cells[0].genome.genes.push(always(PROPEL));
    rebase(&mut h, &e);
    let start = e.cells[0].p;
    for _ in 0..40 {
        h.tick(&mut e, &mut core);
    }
    let (gone, paid) = (e.cells[0].p.sub(start).len(), e.cells[0].state.motor_work);
    eprintln!("ten cycles of swimming: {gone} units for {paid} energy");
    assert!(gone > 2., "more than four body lengths in ten cycles: {gone}");
    assert!(paid < gone as f64 * 0.25, "at a quarter of a unit of energy a unit travelled, or less: {paid}");
}

#[test]
fn the_sea_spreads_free_cells_apart_and_leaves_bodies_and_land_cells_where_they_are() {
    // A free cell, and a bonded pair, all at rest in still water.
    let wandered = |mixing: f32| {
        let (mut core, mut e, mut h) = row(3, vec![always(0)], &[]);
        h.steering.mixing = mixing;
        e.bonds.truncate(1); // cells 0 and 1 stay a body; cell 2 is free
        // Open sea: the first stretch of deep water along the habitat's middle.
        let seed = core.config.seed;
        let x = (-28..28)
            .map(|x| x as f32)
            .find(|&x| (0..5).all(|k| crate::body::ground(x + k as f32 * 0.5, 0., seed, false) < -0.8))
            .expect("the habitat has a sea");
        for (i, c) in e.cells.iter_mut().enumerate() {
            c.p = V3::new(x + [0., 0.44, 2.][i], -0.3, 0.);
        }
        let start: Vec<V3> = e.cells.iter().map(|c| c.p).collect();
        let mut path = 0.;
        for _ in 0..40 {
            let before = e.cells[2].p;
            h.mechanics(&mut e, &mut core, &[]);
            path += e.cells[2].p.sub(before).len();
        }
        let moved = |i: usize| e.cells[i].p.sub(start[i]).len();
        (moved(2), path, moved(0).max(moved(1)))
    };
    let (free, path, body) = wandered(1.);
    eprintln!("after 10 cycles: a free cell is {free} from where it began after wandering {path}; a body moved {body}");
    assert!(path > 2. && free > 0.05, "a free cell is jostled about: {free} after {path}");
    assert!(free < path * 0.6, "and it wanders rather than travels");
    assert!(body < free * 0.2, "a body holds its place: {body}");
    assert!(wandered(4.).1 > path * 1.8, "stirred seas jostle harder");
}

#[test]
fn venom_and_digestion_act_on_other_bodies_never_on_their_own() {
    for weapon in [TOXIN, 9] {
        let outcome = |bonded: bool| {
            let (mut core, mut e, mut h) = fixture();
            build(&mut e, 0, weapon);
            if bonded {
                e.bonds.push(Bond {
                    a: e.cells[0].id,
                    b: e.cells[1].id,
                    rest: 0.4,
                    integrity: 1.,
                    activation: 0.,
                    stiffness: 0.,
                });
            }
            rebase(&mut h, &e);
            h.tick(&mut e, &mut core);
            assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
            e.cells[1].state.injury
        };
        assert!(outcome(false) > 0., "weapon {weapon} harms a stranger");
        assert_eq!(outcome(true), 0., "weapon {weapon} spares its own body");
    }
}

#[test]
fn a_weapon_does_not_work_on_a_cell_that_carries_the_same_weapon() {
    for weapon in [TOXIN, 9, ENGULF] {
        let harm = |armed_prey: bool| {
            let (mut core, mut e, mut h) = fixture();
            build(&mut e, 0, weapon);
            e.cells[0].material.as_mut().unwrap().biomass *= 2.;
            e.cells[1].p = V3::new(e.cells[0].radius() + e.cells[1].radius(), 5., 0.);
            if armed_prey {
                e.cells[1].material.as_mut().unwrap().proteins[weapon] = 5.;
            }
            rebase(&mut h, &e);
            h.tick(&mut e, &mut core);
            e.cells[1].state.injury
        };
        assert!(harm(false) > 0., "weapon {weapon} harms the unarmed");
        assert!(
            harm(true) < harm(false) * 0.05,
            "weapon {weapon} spares its own kind"
        );
    }
}

#[test]
fn a_swallowed_light_feeder_can_be_kept_and_its_host_and_descendants_keep_harvesting_light() {
    // Keep offering an engulfer fresh light-feeding prey until one is kept.
    let (mut core, mut e, mut h) = fixture();
    build(&mut e, 0, ENGULF);
    e.cells[0].material.as_mut().unwrap().biomass *= 2.;
    e.cells[1].p = V3::new(e.cells[0].radius() + e.cells[1].radius(), 5., 0.);
    build(&mut e, 1, 0);
    let mut meals = 0;
    while e.cells[0].state.symbiont == 0. && meals < 5000 {
        e.cells[0].material.as_mut().unwrap().fuel = 0.;
        e.cells[0].energy = 8.;
        e.cells[1].material.as_mut().unwrap().biomass = 0.22;
        e.cells[1].state.injury = 0.;
        h.contacts(&mut e, &mut core);
        meals += 1;
    }
    assert!(
        e.cells[0].state.symbiont > 0.25,
        "no capture in {meals} meals"
    );
    assert!(meals > 1, "capture is a rare event, not the rule");
    assert_eq!(h.steering.captures, 1);

    // A hunter's own membrane harvests little light; a captive makes up for it.
    let harvest = |symbiont: f32| {
        let (mut core, mut e, mut h) = fixture();
        e.cells[1].p = V3::new(0., 5., 6.);
        build(&mut e, 0, ENGULF);
        build(&mut e, 0, 0);
        e.cells[0].state.symbiont = symbiont;
        for site in &mut h.solutes {
            site[1] = 2.;
        }
        rebase(&mut h, &e);
        for _ in 0..8 {
            h.tick(&mut e, &mut core);
        }
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        e.cells[0].state.flux.photosynthesis
    };
    assert!(
        harvest(0.9) > harvest(0.) * 1.5,
        "{} vs {}",
        harvest(0.9),
        harvest(0.)
    );

    // Daughters inherit the captive.
    let mut world = Core::new(Config {
        initial_cells: 16,
        ..Config::default()
    })
    .unwrap();
    world.start_physical();
    for c in &mut world.autonomous.as_mut().unwrap().cells {
        c.state.symbiont = 0.5;
    }
    world.step(400);
    let e = world.autonomous.as_ref().unwrap();
    assert!(e.divisions > 0 && e.cells.iter().all(|c| c.state.symbiont == 0.5));
}

/// A hunter of the given bulk beside one prey cell that has `held` bonds to
/// bystanders. Returns the biomass the prey lost in one tick.
fn bite(hunter_cells: usize, held: usize) -> f64 {
    let mut core = Core::new(Config {
        initial_cells: 2 + hunter_cells + held,
        ..Config::default()
    })
    .unwrap();
    let mut e = Evolution::new(&mut core);
    let mut h = Habitat::new(&core, &mut e.cells);
    for (i, c) in e.cells.iter_mut().enumerate() {
        c.p = V3::new(20. + i as f32 * 3., 5., 20.); // parked apart
        c.v = V3::default();
        c.energy = 8.;
        c.cooldown = 1e6;
        c.material.as_mut().unwrap().proteins[2] = 0.;
    }
    let bond = |a: &autonomous::Cell, b: &autonomous::Cell| Bond {
        a: a.id.min(b.id),
        b: a.id.max(b.id),
        rest: 0.44,
        integrity: 1.,
        activation: 0.,
        stiffness: 0.,
    };
    // Cell 0 is the mouth, cell 1 the prey touching it.
    e.cells[0].p = V3::new(0., 5., 0.);
    e.cells[1].p = V3::new(0.44, 5., 0.);
    build(&mut e, 0, ENGULF);
    e.bonds.clear();
    // The rest of the hunter's body trails behind the mouth.
    for k in 0..hunter_cells {
        let i = 2 + k;
        e.cells[i].p = V3::new(-0.44 * (k + 1) as f32, 5., 0.);
        build(&mut e, i, ENGULF);
        let link = bond(&e.cells[i - if k == 0 { 2 } else { 1 }], &e.cells[i]);
        e.bonds.push(link);
    }
    // Bystanders hold the prey from the far side, out of the hunter's reach.
    for k in 0..held {
        let i = 2 + hunter_cells + k;
        e.cells[i].p = V3::new(0.88, 5. + 0.44 * (k as f32 - 1.), 0.3 * k as f32);
        let link = bond(&e.cells[1], &e.cells[i]);
        e.bonds.push(link);
    }
    h.contacts(&mut e, &mut core);
    e.cells[1].state.flux.lost_to_predators
}

#[test]
fn a_body_is_a_refuge_from_small_hunters_and_not_from_big_ones() {
    let (loose, held) = (bite(0, 0), bite(0, 3));
    assert!(
        loose > 0. && held < loose * 0.4,
        "held prey resists: {held} vs {loose}"
    );
    let big = bite(8, 3);
    assert!(
        big > held * 2.,
        "a big hunter tears it off: {big} vs {held}"
    );
}

#[test]
fn a_weapon_spares_armed_kin_but_not_an_armed_stranger() {
    let harm = |stranger: bool| {
        let (mut core, mut e, mut h) = fixture();
        build(&mut e, 0, ENGULF);
        build(&mut e, 1, ENGULF);
        // Kin share their working genes; a stranger has a different set.
        e.cells[0].genome.genes = vec![always(ENGULF), always(0), always(1), always(7)];
        e.cells[1].genome.genes = if stranger {
            vec![always(ENGULF), always(3), always(14), always(21)]
        } else {
            vec![always(ENGULF), always(0), always(1), always(7)]
        };
        e.cells[0].material.as_mut().unwrap().biomass *= 2.;
        e.cells[1].p = V3::new(e.cells[0].radius() + e.cells[1].radius(), 5., 0.);
        h.contacts(&mut e, &mut core);
        e.cells[1].state.flux.lost_to_predators
    };
    assert_eq!(harm(false), 0., "armed kin are spared");
    assert!(
        harm(true) > 0.,
        "carrying a mouth is no shield against a stranger"
    );
}

#[test]
fn a_mouth_grows_larger_than_its_prey_and_does_not_glue_itself_to_it() {
    let (mut core, mut e, mut h) = fixture();
    e.cells[1].p = V3::new(0., 5., 6.);
    build(&mut e, 0, ENGULF);
    for c in &mut e.cells {
        // Both can take up dissolved fuel and invest in growth.
        c.genome.genes = vec![always(7), always(1)];
        c.material.as_mut().unwrap().fuel = 0.3;
    }
    e.cells[0].genome.genes.push(always(ENGULF));
    e.cells[0].genome.id = 300;
    for s in &mut h.solutes {
        s[0] = 6.;
    }
    for _ in 0..600 {
        h.tick(&mut e, &mut core);
    }
    assert!(
        biomass(&e, 0) > 0.52,
        "mouth cell reached {}",
        biomass(&e, 0)
    );
    assert!(
        biomass(&e, 1) <= 0.5 + 1e-9,
        "ordinary cell stays at {}",
        biomass(&e, 1)
    );

    // Sticky cells bond on contact only with kin. A mouth bonds to unarmed
    // kin too: that is how a body gets a mouth.
    let bonded = |mouth: bool, stranger: bool| {
        let (mut core, mut e, mut h) = fixture();
        for c in &mut e.cells {
            c.material.as_mut().unwrap().proteins[2] = 0.05;
            c.genome.genes = vec![always(0), always(1), always(7)];
        }
        if stranger {
            e.cells[1].genome.genes = vec![always(3), always(14), always(21)];
        }
        if mouth {
            build(&mut e, 0, ENGULF);
            e.cells[1].material.as_mut().unwrap().biomass *= 3.; // too big to be eaten
            e.cells[1].p = V3::new(e.cells[0].radius() + e.cells[1].radius(), 5., 0.);
        }
        for _ in 0..40 {
            h.contacts(&mut e, &mut core);
        }
        !e.bonds.is_empty()
    };
    assert!(
        !bonded(false, true),
        "strangers do not stick: a body is a family"
    );
    assert!(
        !bonded(true, true),
        "least of all a mouth to a stranger it could swallow"
    );
    assert!(bonded(false, false), "kin stick");
    assert!(bonded(true, false), "and a mouth bonds to its unarmed kin");
}

#[test]
fn a_steering_cell_senses_which_side_other_life_is_on() {
    let side = |z: f32| {
        let (mut core, mut e, mut h) = fixture();
        build(&mut e, 0, STEER);
        e.cells[0].p = V3::new(0., 5., 0.);
        e.cells[1].p = V3::new(0., 5., z); // left of a cell heading along +x is +z
        h.contacts(&mut e, &mut core);
        e.cells[0].state.nearby
    };
    assert!(side(1.) > 0.1 && side(-1.) < -0.1);
    assert_eq!(side(3.), 0., "beyond one field spacing nothing is felt");
}

#[test]
fn a_mouth_feeds_the_body_it_is_bonded_to_and_is_fed_by_it_in_turn() {
    // (mouth's fuel, neighbour's fuel) after sharing, plus what was recorded.
    let share = |mouth: bool, bonded: bool, mouth_fuel: f64, other_fuel: f64| {
        let (_, mut e, mut h) = fixture();
        if mouth {
            build(&mut e, 0, ENGULF);
        }
        if bonded {
            e.bonds.push(Bond {
                a: e.cells[0].id,
                b: e.cells[1].id,
                rest: 0.4,
                integrity: 1.,
                activation: 0.,
                stiffness: 0.,
            });
        }
        e.cells[0].material.as_mut().unwrap().fuel = mouth_fuel;
        e.cells[1].material.as_mut().unwrap().fuel = other_fuel;
        rebase(&mut h, &e);
        for _ in 0..40 {
            h.tissue_transport(&mut e);
        }
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        let fuel = |i: usize| e.cells[i].material.as_ref().unwrap().fuel;
        (
            fuel(0),
            fuel(1),
            e.cells[0].state.flux.shared,
            e.cells[1].state.flux.fed,
        )
    };
    // After a meal the mouth pours fuel into its neighbour until they level.
    let (mouth, fed_cell, shared, fed) = share(true, true, 0.3, 0.);
    assert!(
        fed_cell > 0.1 && (mouth - fed_cell).abs() < 0.05,
        "{mouth} {fed_cell}"
    );
    assert!((shared - fed).abs() < 1e-12 && fed > 0.1);
    // Between meals the body keeps its mouth alive; that is not being fed by a mouth.
    let (mouth, body_cell, shared, fed) = share(true, true, 0., 0.3);
    assert!(mouth > 0.1 && body_cell < 0.2);
    assert_eq!((shared, fed), (0., 0.));
    // No bond, or no mouth and no transport tissue: nothing moves.
    assert_eq!(share(true, false, 0.3, 0.).1, 0.);
    assert_eq!(share(false, true, 0.3, 0.).1, 0.);
    // A gut fills; it does not burst.
    let (_, full, _, _) = share(true, true, 0.33, 0.32);
    assert!(full <= 0.22 * 1.5 + 1e-9, "{full}");
}

/// One stem cell bonded to a neighbour, carrying the given genes.
fn in_a_body(genes: Vec<Gene>) -> (Core, Evolution, Habitat) {
    let (core, mut e, mut h) = fixture();
    for c in &mut e.cells {
        c.material.as_mut().unwrap().proteins[2] = 0.05;
    }
    // Both keep making adhesion, or the bond between them would not last.
    e.cells[0].genome.genes = genes;
    e.cells[0].genome.genes.push(always(2));
    e.cells[1].genome.genes = vec![always(2)];
    e.bonds.push(Bond {
        a: e.cells[0].id,
        b: e.cells[1].id,
        rest: 0.4,
        integrity: 1.,
        activation: 0.,
        stiffness: 0.,
    });
    rebase(&mut h, &e);
    (core, e, h)
}
/// Bonds break at random; these fixtures are about what happens inside a
/// body, so the original pair is tied back together before every tick.
fn step_as_a_body(h: &mut Habitat, e: &mut Evolution, core: &mut Core) {
    let (a, b) = (100, 101);
    let ids: Vec<u32> = e
        .cells
        .iter()
        .filter(|c| c.genome.id == a || c.genome.id == b)
        .filter(|c| c.parent == 0)
        .map(|c| c.id)
        .collect();
    if ids.len() == 2
        && !e
            .bonds
            .iter()
            .any(|x| (x.a, x.b) == (ids[0].min(ids[1]), ids[0].max(ids[1])))
    {
        e.bonds.push(Bond {
            a: ids[0].min(ids[1]),
            b: ids[0].max(ids[1]),
            rest: 0.4,
            integrity: 1.,
            activation: 0.,
            stiffness: 0.,
        });
    }
    h.tick(e, core);
}
fn only_in(action: usize, fates: u8) -> Gene {
    Gene {
        fates,
        ..always(action)
    }
}

#[test]
fn a_bonded_stem_cell_commits_to_a_fate_and_a_lone_cell_is_always_a_stem_cell() {
    use crate::physical::BECOME;
    let (mut core, mut e, mut h) = in_a_body(vec![always(0), always(BECOME[1])]);
    h.tick(&mut e, &mut core);
    assert_eq!(e.cells[0].state.fate, 2, "driven past half, it commits");
    assert_eq!(e.cells[1].state.fate, 0, "its neighbour has no such gene");
    assert_eq!(
        e.cells[0].material.as_ref().unwrap().proteins[BECOME[1]],
        0.,
        "regulation builds nothing"
    );
    // Cut loose, it is its own organism again.
    e.bonds.clear();
    h.tick(&mut e, &mut core);
    assert_eq!(e.cells[0].state.fate, 0);
    // A weakly driven commitment gene does nothing.
    let weak = Gene {
        strength: 0.4,
        ..always(BECOME[0])
    };
    let (mut core, mut e, mut h) = in_a_body(vec![always(0), weak]);
    h.tick(&mut e, &mut core);
    assert_eq!(e.cells[0].state.fate, 0);
}

#[test]
fn a_gene_masked_off_for_a_fate_is_silent_there_and_its_machinery_is_dismantled() {
    use crate::physical::BECOME;
    // A motor gene switched on only in stem cells (fate 0), in a cell that commits to fate 1.
    let genes = |commit: bool| {
        let mut genes = vec![always(0), always(1), only_in(PROPEL, 0b0001)];
        if commit {
            genes.push(always(BECOME[0]));
        }
        genes
    };
    let motor_after = |commit: bool| {
        let (mut core, mut e, mut h) = in_a_body(genes(commit));
        build(&mut e, 0, PROPEL);
        for s in &mut h.solutes {
            s[0] = 4.;
        }
        for _ in 0..200 {
            step_as_a_body(&mut h, &mut e, &mut core);
        }
        (
            e.cells[0].state.fate,
            e.cells[0].expression[PROPEL],
            e.cells[0].activity(PROPEL),
        )
    };
    let (fate, drive, built) = motor_after(false);
    assert_eq!(fate, 0);
    assert!(
        drive > 0.5 && built > 0.5,
        "a stem cell keeps its motor: {drive} {built}"
    );
    let (fate, drive, built) = motor_after(true);
    assert_eq!(fate, 1);
    assert_eq!(drive, 0., "silent in the committed cell");
    assert!(built < 0.1, "and the motor is taken apart: {built}");
}

#[test]
fn a_committed_cell_that_cannot_divide_is_a_worker_and_daughters_inherit_fate() {
    use crate::physical::BECOME;
    // Replication switched on in stem cells only: committing makes a worker.
    let (mut core, mut e, mut h) = in_a_body(vec![
        always(0),
        always(1),
        only_in(7, 0b0001),
        always(BECOME[2]),
    ]);
    for s in &mut h.solutes {
        s[0] = 6.;
    }
    for c in &mut e.cells {
        c.cooldown = 0.;
    }
    for _ in 0..400 {
        step_as_a_body(&mut h, &mut e, &mut core);
    }
    let worker = e.cells.iter().find(|c| c.genome.id == 100).unwrap();
    assert_eq!(worker.state.fate, 3);
    assert!(
        worker.activity(7) < 0.05,
        "no replication machinery: {}",
        worker.activity(7)
    );
    assert!(
        e.cells.iter().all(|c| c.parent != worker.id),
        "a worker never divides"
    );
    // A fate is passed to daughters that stay attached.
    let (mut core, mut e, mut h) = in_a_body(vec![always(0), always(1), always(7)]);
    e.cells[0].state.fate = 2;
    for s in &mut h.solutes {
        s[0] = 6.;
    }
    for c in &mut e.cells {
        c.cooldown = 0.;
    }
    let mother = e.cells[0].id;
    // Bonds break and re-form at random; a cell that was ever cut loose started again as a stem cell.
    let mut loosened = std::collections::HashSet::new();
    for _ in 0..600 {
        step_as_a_body(&mut h, &mut e, &mut core);
        for c in &e.cells {
            if !e.bonds.iter().any(|b| b.a == c.id || b.b == c.id) {
                loosened.insert(c.id);
            }
        }
    }
    let daughters = || e.cells.iter().filter(|c| c.parent == mother);
    let kept = || daughters().filter(|c| !loosened.contains(&c.id));
    assert!(
        kept().count() > 0,
        "the mother divided and daughters stayed attached"
    );
    assert!(
        kept().all(|c| c.state.fate == 2),
        "attached daughters inherit the fate"
    );
    assert!(
        daughters()
            .filter(|c| loosened.contains(&c.id))
            .all(|c| c.state.fate == 0),
        "a daughter cut loose is a stem cell again"
    );
}

#[test]
fn a_wall_stops_swallowing_both_ways_and_stops_swimming_but_not_enzymes() {
    // What the prey loses to a mouth twice its size, or to a digesting neighbour.
    let lost = |weapon: usize, walled_prey: bool, walled_hunter: bool| {
        let (mut core, mut e, mut h) = fixture();
        build(&mut e, 0, weapon);
        e.cells[0].material.as_mut().unwrap().biomass *= 2.;
        e.cells[1].p = V3::new(e.cells[0].radius() + e.cells[1].radius(), 5., 0.);
        if walled_prey {
            build(&mut e, 1, WALL);
        }
        if walled_hunter {
            build(&mut e, 0, WALL);
        }
        rebase(&mut h, &e);
        h.tick(&mut e, &mut core);
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        e.cells
            .iter()
            .find(|c| c.genome.id == 101)
            .unwrap()
            .state
            .flux
            .lost_to_predators
    };
    let eaten = lost(ENGULF, false, false);
    assert!(eaten > 0.02);
    assert!(
        lost(ENGULF, true, false) < eaten * 0.15,
        "a walled cell is not swallowed"
    );
    assert!(
        lost(ENGULF, false, true) < eaten * 0.25,
        "a walled cell cannot swallow"
    );
    let digested = lost(9, false, false);
    assert!(
        digested > 0. && lost(9, true, false) > digested * 0.9,
        "enzymes still eat walled life"
    );
    // The same motor, with and without a wall around it.
    let swam = |walled: bool| {
        let (mut core, mut e, mut h) = fixture();
        e.cells[1].p = V3::new(0., 5., 6.);
        build(&mut e, 0, PROPEL);
        e.cells[0].genome.genes.push(always(PROPEL));
        if walled {
            build(&mut e, 0, WALL);
            e.cells[0].genome.genes.push(always(WALL));
        }
        rebase(&mut h, &e);
        let start = e.cells[0].p.x;
        for _ in 0..24 {
            h.tick(&mut e, &mut core);
        }
        e.cells[0].p.x - start
    };
    assert!(
        swam(true) < swam(false) * 0.4,
        "{} vs {}",
        swam(true),
        swam(false)
    );
}

#[test]
fn walled_sisters_stay_cemented_without_any_glue() {
    let attached = |walled: bool| {
        let (mut core, mut e, mut h) = fixture();
        e.cells[1].p = V3::new(0., 5., 9.);
        e.cells[0].genome.genes = vec![always(0), always(1), always(7)];
        if walled {
            e.cells[0].genome.genes.push(always(WALL));
            build(&mut e, 0, WALL);
        }
        e.cells[0].cooldown = 0.;
        // Copies stay true, so no daughter invents glue or a wall of its own.
        h.steering.mutation_scale = 0.;
        for s in &mut h.solutes {
            s[0] = 6.;
        }
        rebase(&mut h, &e);
        for _ in 0..600 {
            h.tick(&mut e, &mut core);
        }
        (e.divisions, e.attached_divisions, e.bonds.len())
    };
    let (born, joined, bonds) = attached(true);
    assert!(
        born > 2 && joined * 2 > born && bonds > 0,
        "walled: {born} born, {joined} joined, {bonds} bonds"
    );
    let (born, joined, _) = attached(false);
    assert!(
        born > 2 && joined == 0,
        "bare cells with no glue part: {born} born, {joined} joined"
    );
}

#[test]
fn chemosynthesis_feeds_a_cell_in_the_dark_and_its_energy_is_accounted() {
    let fed = |catalyst: bool, vents: f32| {
        let (mut core, mut e, mut h) = fixture();
        core.config.sunlight = 0.;
        e.cells[1].p = V3::new(0., 5., 6.);
        h.steering.vents = vents;
        for s in &mut h.solutes {
            s[0] = 0.;
            s[1] = 2.;
            s[2] = 0.5;
        }
        if catalyst {
            build(&mut e, 0, CHEMO);
        }
        rebase(&mut h, &e);
        for _ in 0..8 {
            h.tick(&mut e, &mut core);
        }
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        (e.cells[0].state.flux.chemosynthesis, h.geothermal_energy)
    };
    let (none, no_input) = fed(false, 1.);
    let (natural, input) = fed(true, 1.);
    let (active, _) = fed(true, 4.);
    assert_eq!((none, no_input), (0., 0.));
    assert!(natural > 0. && input > 0.);
    assert!(active > natural * 3., "{active} vs {natural}");
}

#[test]
fn founders_carry_none_of_the_new_abilities_and_older_saves_still_load() {
    let mut core = Core::new(Config::default()).unwrap();
    core.start_physical();
    let e = core.autonomous.as_ref().unwrap();
    for c in &e.cells {
        for k in [PROPEL, STEER, ENGULF, TOXIN, SPINE, CHEMO] {
            assert!(c.genome.genes.iter().all(|g| g.action != k));
            assert_eq!(c.material.as_ref().unwrap().proteins[k], 0.);
        }
    }
    // A save written with the 32-gene, 28-sense vocabulary pads with zeros.
    let mut save: serde_json::Value = serde_json::to_value(&core).unwrap();
    for cell in save["autonomous"]["cells"].as_array_mut().unwrap() {
        cell["expression"].as_array_mut().unwrap().truncate(32);
        cell["material"]["proteins"]
            .as_array_mut()
            .unwrap()
            .truncate(32);
        for gene in cell["genome"]["genes"].as_array_mut().unwrap() {
            gene["promoter"].as_array_mut().unwrap().truncate(28);
        }
    }
    let mut old = Core::load(&save.to_string()).unwrap();
    old.step(40);
    core.step(40);
    assert_eq!(
        serde_json::to_string(&old).unwrap(),
        serde_json::to_string(&core).unwrap()
    );
}

/// A row of bonded cells along the heading, in still open water, kept whole:
/// these fixtures measure a stroke, not how long bonds last.
fn row(n: usize, genes: Vec<Gene>, materials: &[usize]) -> (Core, Evolution, Habitat) {
    let mut core = Core::new(Config {
        initial_cells: n,
        ..Config::default()
    })
    .unwrap();
    let mut e = Evolution::new(&mut core);
    let mut h = Habitat::new(&core, &mut e.cells);
    h.fluid = crate::physical::Fluid::still();
    for (i, c) in e.cells.iter_mut().enumerate() {
        c.p = V3::new(i as f32 * 0.44, 12., 0.);
        c.v = V3::default();
        c.heading = 0.;
        c.energy = 8.;
        c.cooldown = 1e6;
        c.genome.id = 100;
        c.genome.genes = genes.clone();
        c.genome.genes.push(always(2));
        let m = c.material.as_mut().unwrap();
        m.proteins[2] = 0.05;
        for &k in materials {
            m.proteins[k] = 0.05;
        }
    }
    e.next_genome = 200;
    e.bonds.clear();
    for i in 0..n - 1 {
        e.bonds.push(Bond {
            a: e.cells[i].id,
            b: e.cells[i + 1].id,
            rest: 0.44,
            integrity: 1.,
            activation: 0.,
            stiffness: 0.,
        });
    }
    h.oxygenate(1.); // these fixtures breathe freely
    rebase(&mut h, &e);
    (core, e, h)
}
/// Run a row for `ticks`, re-tying it, and return how far it travelled along its heading.
fn travel(core: &mut Core, e: &mut Evolution, h: &mut Habitat, ticks: usize) -> f32 {
    let centre = |e: &Evolution| e.cells.iter().map(|c| c.p.x).sum::<f32>() / e.cells.len() as f32;
    let start = centre(e);
    for _ in 0..ticks {
        let tied = e.bonds.clone();
        for c in &mut e.cells {
            c.energy = 8.;
            c.state.injury = 0.;
        }
        h.tick(e, core);
        for old in tied {
            match e.bonds.iter_mut().find(|b| b.a == old.a && b.b == old.b) {
                Some(b) => b.integrity = 1.,
                None => e.bonds.push(Bond { integrity: 1., ..old }),
            }
        }
    }
    centre(e) - start
}

#[test]
fn a_beating_muscle_goes_nowhere_without_fins_and_swims_with_them() {
    // Muscle, excitable channels, and enough of signal A in the water to keep
    // the channels firing: a body that beats. Only the fins differ.
    let swum = |fins: bool| {
        let mut materials = vec![4, crate::physiology::CHANNEL];
        let mut genes = vec![always(0), always(4), always(crate::physiology::CHANNEL)];
        if fins {
            materials.push(FIN);
            genes.push(always(FIN));
        }
        let (mut core, mut e, mut h) = row(4, genes, &materials);
        let mut beats = 0;
        let mut was = 0.;
        let mut gone = 0.;
        for _ in 0..80 {
            for s in &mut h.solutes {
                s[3] = 1.;
            }
            gone += travel(&mut core, &mut e, &mut h, 4);
            // The signal holds the muscle half tight; each firing pulls it the rest of the way.
            let now = e.bonds[0].activation;
            beats += usize::from(was < 0.7 && now >= 0.7);
            was = now;
        }
        (gone, beats, e.cells.iter().map(|c| c.state.motor_work).sum::<f64>())
    };
    let (drifted, beats, work) = swum(false);
    assert!(beats >= 4 && work > 0., "the muscle beats on its own: {beats} beats, work {work}");
    let (swam, _, _) = swum(true);
    eprintln!("beating row of four over 80 cycles: {swam} with fins, {drifted} without, {beats} beats");
    assert!(drifted.abs() < 0.1, "no fins, no headway: {drifted}");
    assert!(swam > drifted.abs() * 3. + 0.2, "fins turn the beat into travel: {swam} against {drifted}");
}

#[test]
fn a_fin_makes_no_thrust_of_its_own() {
    let (mut core, mut e, mut h) = row(3, vec![always(0), always(FIN)], &[FIN]);
    // Shoved backward, the fins catch the water; nothing is created or lost.
    for c in &mut e.cells {
        c.v = V3::new(-0.5, 0., 0.);
    }
    for _ in 0..8 {
        h.tick(&mut e, &mut core);
    }
    assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
    assert!(h.valid(&e.cells), "stores and fluid stay in range");
    let gone = travel(&mut core, &mut e, &mut h, 160);
    assert!(gone.abs() < 0.1, "a body that does not beat stays put: {gone}");
    assert_eq!(e.cells.iter().map(|c| c.state.motor_work).sum::<f64>(), 0.);
}

#[test]
fn motors_buried_in_a_body_have_no_water_to_beat_against() {
    // A cross of five cells with one driven motor: on an arm, or on the hub
    // with a neighbour on every side.
    let swum = |at: usize| {
        let (mut core, mut e, mut h) = row(5, vec![always(0)], &[]);
        let hub = e.cells[0].p;
        for (k, offset) in [(1, (0.44, 0.)), (2, (-0.44, 0.)), (3, (0., 0.44)), (4, (0., -0.44))] {
            e.cells[k].p = V3::new(hub.x + offset.0, hub.y, hub.z + offset.1);
        }
        let hub_id = e.cells[0].id;
        for (k, b) in e.bonds.iter_mut().enumerate() {
            (b.a, b.b) = (hub_id, e.cells[k + 1].id);
        }
        e.cells[at].genome.id = 150;
        e.cells[at].genome.genes.push(always(PROPEL));
        build(&mut e, at, PROPEL);
        // A heading turns with the body it is part of and in no other way.
        let bearing = |e: &Evolution| {
            let d = e.cells[1].p.sub(e.cells[2].p);
            d.z.atan2(d.x)
        };
        let (heading, lay) = (e.cells[at].heading, bearing(&e));
        let gone = travel(&mut core, &mut e, &mut h, 40);
        let turned = |a: f32| (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        let (swung, rolled) = (turned(e.cells[at].heading - heading), turned(bearing(&e) - lay));
        assert!((swung - rolled).abs() < 0.25, "a cell held in a body does not tumble: heading {swung}, body {rolled}");
        gone
    };
    let (arm, hub) = (swum(1), swum(0));
    assert!(arm > 0.1 && hub > 0., "arm {arm}, hub {hub}");
    // One neighbour costs a cell little of its surface; four cost nearly half.
    let (mut core, mut e, mut h) = row(5, vec![always(0)], &[]);
    let hub_id = e.cells[0].id;
    for (k, b) in e.bonds.iter_mut().enumerate() {
        (b.a, b.b) = (hub_id, e.cells[k + 1].id);
    }
    h.tick(&mut e, &mut core);
    let open = |i: usize| e.cells.iter().find(|c| c.id == hub_id + i as u32).unwrap().state.exposure;
    assert!(open(1) > 0.95 && (open(0) - 0.556).abs() < 0.01, "arm {}, hub {}", open(1), open(0));
    assert!(hub < arm * 0.75, "the buried motor pushes less: {hub} against {arm}");
}

#[test]
fn touching_cells_trade_genes_only_when_the_player_allows_it() {
    let traded = |sharing: f32| {
        let (mut core, mut e, mut h) = fixture();
        // Two strangers, each carrying one silent gene the other lacks.
        let silent = |action: usize| Gene {
            strength: 0.,
            ..always(action)
        };
        e.cells[0].genome.genes.push(silent(PROPEL));
        e.cells[1].genome.genes.push(silent(STEER));
        h.steering.sharing = sharing;
        for _ in 0..200 {
            h.tick(&mut e, &mut core);
        }
        assert_eq!(e.cells.len(), 2);
        let carries = |c: usize, action: usize| e.cells[c].genome.genes.iter().any(|g| g.action == action);
        (
            carries(0, STEER) || carries(1, PROPEL),
            e.cells.iter().map(|c| c.genome.id).max().unwrap(),
            e.exchanges,
            e.cells[0].genome.id != e.cells[1].genome.id && h.valid(&e.cells),
        )
    };
    let (gained, newest, exchanges, sound) = traded(8.);
    assert!(gained && newest >= 200 && exchanges > 0, "a gene crossed over: {newest} {exchanges}");
    assert!(sound, "each changed genome has its own id and the world stays in range");
    let (gained, newest, exchanges, _) = traded(0.);
    assert!(!gained && newest < 200 && exchanges == 0, "off means off");
}

#[test]
fn strong_glue_holds_a_body_together_and_a_body_that_stops_making_it_falls_apart() {
    let left = |strong: bool| {
        let (mut core, mut e, mut h) = row(8, vec![always(0)], &[]);
        if !strong {
            // A trace of glue left over, and none being made.
            for c in &mut e.cells {
                c.genome.genes = vec![always(0)];
                c.material.as_mut().unwrap().proteins[2] = 0.002;
            }
        }
        for _ in 0..400 {
            for c in &mut e.cells {
                c.energy = 8.;
            }
            h.tick(&mut e, &mut core);
        }
        e.bonds.len()
    };
    let (strong, weak) = (left(true), left(false));
    assert!(strong >= 6, "seven bonds, a hundred cycles, strong glue: {strong} left");
    assert!(weak <= 3, "without fresh glue the body comes apart: {weak} left");
}

#[test]
fn a_polarised_cell_divides_along_its_heading_or_across_it() {
    use crate::{development::POLARITY, physical::BROADEN};
    // Where the first daughter lands, as (along the heading, across it).
    let placed = |broaden: bool| {
        let (mut core, mut e, mut h) = fixture();
        e.cells[1].p = V3::new(0., 5., 9.);
        e.cells[0].genome.genes = vec![always(0), always(1), always(7), always(POLARITY)];
        if broaden {
            e.cells[0].genome.genes.push(always(BROADEN));
        }
        build(&mut e, 0, POLARITY);
        e.cells[0].material.as_mut().unwrap().proteins[POLARITY] = 0.2;
        e.cells[0].cooldown = 0.;
        for s in &mut h.solutes {
            s[0] = 6.;
        }
        let mother = e.cells[0].id;
        for _ in 0..400 {
            let before = e.cells.len();
            let at = e.cells[0].p;
            h.divide(&mut e, &mut core);
            if e.cells.len() > before {
                let child = e.cells.iter().find(|c| c.parent == mother).unwrap();
                let d = child.p.sub(at);
                assert_eq!(child.material.as_ref().unwrap().proteins[BROADEN], 0., "regulation builds nothing");
                return (d.x.abs(), d.z.abs());
            }
            h.tick(&mut e, &mut core);
        }
        panic!("the cell never divided");
    };
    let (along, across) = placed(false);
    assert!(along > across * 3., "lengthening: {along} along, {across} across");
    let (along, across) = placed(true);
    assert!(across > along * 3., "widening: {along} along, {across} across");
}

#[test]
fn a_cells_heading_turns_with_the_body_it_is_part_of() {
    let (mut core, mut e, mut h) = row(2, vec![always(0)], &[]);
    // Spin the pair about its middle.
    e.cells[0].v = V3::new(0., 0., -1.5);
    e.cells[1].v = V3::new(0., 0., 1.5);
    for _ in 0..12 {
        h.tick(&mut e, &mut core);
    }
    assert_eq!(e.bonds.len(), 1);
    let d = e.cells[1].p.sub(e.cells[0].p);
    let swing = d.z.atan2(d.x);
    assert!(swing.abs() > 0.1, "the body turned: {swing}");
    for c in &e.cells {
        let heading = (c.heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        assert!((heading - swing).abs() < 0.05, "heading {heading} against {swing}");
    }
}

#[test]
fn a_beating_body_resting_on_the_ground_crawls_on_its_feet() {
    let crawled = |feet: bool| {
        let mut materials = vec![4, crate::physiology::CHANNEL];
        let mut genes = vec![always(0), always(4), always(crate::physiology::CHANNEL)];
        if feet {
            materials.push(FIN);
            genes.push(always(FIN));
        }
        let (mut core, mut e, mut h) = row(4, genes, &materials);
        for c in &mut e.cells {
            c.p.y = 4.6; // just above the bed: it settles within a few cycles
        }
        let mut gone = 0.;
        let mut resting = 0;
        for k in 0..100 {
            for s in &mut h.solutes {
                s[3] = 1.;
            }
            let step = travel(&mut core, &mut e, &mut h, 4);
            if k >= 20 {
                gone += step;
                resting += e.cells.iter().filter(|c| c.p.y < 4.5).count();
            }
        }
        (gone, resting)
    };
    let (slid, resting) = crawled(false);
    assert!(resting > 300, "the body lies on the bed: {resting} of 320 cell-samples");
    let (crawled, _) = crawled(true);
    eprintln!("on the ground over 80 cycles: {crawled} with feet, {slid} without");
    assert!(crawled > slid.abs() * 3. + 0.1, "feet turn the beat into a crawl: {crawled} against {slid}");
}

#[test]
fn muscle_at_a_bony_joint_swings_a_limb_and_feet_make_it_a_stride() {
    // Two bones meeting at a wide angle, muscle and a beat-maker at the joint.
    // The ends of the bones lie fore and aft; only the feet on them differ.
    let walked = |feet: bool| {
        let (mut core, mut e, mut h) = row(3, vec![always(0)], &[]);
        let joint = e.cells[1].p;
        e.cells[0].p = V3::new(joint.x - 0.425, joint.y, joint.z + 0.114);
        e.cells[2].p = V3::new(joint.x + 0.425, joint.y, joint.z + 0.114);
        let knee = e.cells[1].id;
        for (i, c) in e.cells.iter_mut().enumerate() {
            let m = c.material.as_mut().unwrap();
            m.mineral = 0.12 * m.mass();
            if i == 1 {
                m.proteins[4] = 0.05;
                m.proteins[crate::physiology::CHANNEL] = 0.05;
                c.genome.id = 150;
                c.genome.genes.extend([always(4), always(crate::physiology::CHANNEL)]);
            } else if feet {
                m.proteins[FIN] = 0.05;
                c.genome.id = 160;
                c.genome.genes.push(always(FIN));
            }
        }
        let (mut gone, mut least, mut most) = (0., f32::MAX, 0f32);
        for _ in 0..80 {
            // A little of signal A: enough to keep the beat-maker firing
            // without holding the flexor tight.
            for s in &mut h.solutes {
                s[3] = 0.045;
            }
            gone += travel(&mut core, &mut e, &mut h, 4);
            assert_eq!(e.bonds.len(), 2, "the two ends never touch");
            if let Some(j) = h.joints.iter().find(|j| j.b == knee) {
                least = least.min(j.angle);
                most = most.max(j.angle);
            }
        }
        (gone, most - least, h.joints.iter().map(|j| j.work).sum::<f64>())
    };
    let (stood, swing, work) = walked(false);
    assert!(swing > 0.2 && work > 0., "the joint swings under its own muscle: {swing} rad, work {work}");
    let (strode, _, _) = walked(true);
    eprintln!("a two-bone limb over 80 cycles: {strode} with feet, {stood} without, swing {swing} rad");
    assert!(strode > stood.abs() * 3. + 0.1, "feet turn the swing into a stride: {strode} against {stood}");
}


#[test]
fn nitrogen_is_taken_from_the_water_by_light_feeders_passed_on_in_food_and_never_made_or_lost() {
    use crate::physical::{NITROGEN, N_RATIO, REMAINS_N};
    // The same fed cell, in a sea with nitrogen and in one with none.
    let grown = |sea: bool| {
        let (mut core, mut e, mut h) = fixture();
        e.cells[1].p = V3::new(0., 5., 9.);
        for c in &mut e.cells {
            c.genome.genes = vec![always(7)];
            let m = c.material.as_mut().unwrap();
            m.fuel = 0.3;
            m.nitrogen = 0.;
        }
        if !sea {
            for s in &mut h.solutes {
                s[NITROGEN] = 0.;
            }
        }
        rebase(&mut h, &e);
        let before = biomass(&e, 0);
        for _ in 0..80 {
            h.tick(&mut e, &mut core);
        }
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        (biomass(&e, 0) - before, e.cells[0].state.nitrogen_short)
    };
    let ((fed, eased), (starved, short)) = (grown(true), grown(false));
    eprintln!("growth with nitrogen {fed}, without {starved}");
    assert!(fed > 0.05 && starved < fed * 0.2, "{fed} against {starved}");
    assert!(short > 0.5 && eased < 0.2, "shortage {short} and {eased}");

    // An animal has no way to take nitrogen from the water. It gets it by
    // eating, keeps what it can build with and gives the rest off.
    let (mut core, mut e, mut h) = fixture();
    for k in [0, 1] {
        e.cells[0].material.as_mut().unwrap().proteins[k] = 0.;
    }
    e.cells[0].material.as_mut().unwrap().nitrogen = 0.;
    rebase(&mut h, &e);
    for _ in 0..8 {
        h.tick(&mut e, &mut core);
    }
    assert_eq!(e.cells[0].state.flux.nitrogen, 0., "an animal took nitrogen from the water");
    build(&mut e, 0, 9);
    e.cells[0].genome.genes = vec![always(9)];
    e.cells[1].genome.id = 300; // a stranger, not kin
    e.cells[1].genome.genes = vec![always(0), always(7), always(1)];
    rebase(&mut h, &e);
    let total = h.nitrogen_total(&e.cells);
    for _ in 0..240 {
        for (i, c) in e.cells.iter_mut().enumerate() {
            c.p = V3::new(i as f32 * 0.4, 5., 0.);
            c.v = V3::default();
            c.state.injury = 0.;
        }
        h.tick(&mut e, &mut core);
    }
    let eater = &e.cells[0];
    let droppings: f64 = h.solutes.iter().map(|s| s[REMAINS_N]).sum();
    eprintln!(
        "eater ate {}, holds {} loose nitrogen, gave off {}, droppings hold {}",
        eater.state.flux.prey, eater.material.as_ref().unwrap().nitrogen,
        eater.state.flux.nitrogen_given, droppings
    );
    assert!(eater.state.flux.prey > 0.02, "the eater fed");
    assert_eq!(eater.state.flux.nitrogen, 0.);
    assert!(eater.material.as_ref().unwrap().nitrogen > 0. && droppings > 0.);
    assert!(eater.state.flux.nitrogen_given > 0., "spare nitrogen goes back to the water");
    assert!((h.nitrogen_total(&e.cells) - total).abs() < 1e-9, "nitrogen was made or lost");
    assert!(N_RATIO > 0.1 && balanced(&h, &e), "{:?}", h.errors(&e.cells));
}

#[test]
fn a_cell_feeds_one_level_above_what_it_is_built_from() {
    // A light-feeder with no other food is a producer.
    let (mut core, mut e, mut h) = fixture();
    e.cells[1].p = V3::new(0., 5., 9.);
    for s in &mut h.solutes {
        s[0] = 0.;
        s[1] = 1.;
    }
    e.cells[0].genome.genes = vec![always(0)];
    build(&mut e, 0, 0);
    e.cells[0].state.trophic = 1.;
    e.cells[0].material.as_mut().unwrap().fuel = 0.;
    rebase(&mut h, &e);
    for _ in 0..120 {
        e.cells[0].state.thirst = 0.;
        h.tick(&mut e, &mut core);
    }
    eprintln!("light-feeder: {} steps above the producers", e.cells[0].state.trophic);
    assert_eq!(e.cells[0].state.level(), 0);
    assert!(h.web.flow[0] > 0., "carbon entered the producers");

    // What eats a producer is a plant-eater; what eats that is a meat-eater.
    for (prey_steps, level) in [(0., 1), (1., 2)] {
        let (mut core, mut e, mut h) = fixture();
        for k in [0, 1] {
            e.cells[0].material.as_mut().unwrap().proteins[k] = 0.;
        }
        build(&mut e, 0, 9);
        e.cells[0].genome.genes = vec![always(9)];
        e.cells[1].genome.id = 300;
        e.cells[1].state.trophic = prey_steps;
        rebase(&mut h, &e);
        for _ in 0..160 {
            for (i, c) in e.cells.iter_mut().enumerate() {
                c.p = V3::new(i as f32 * 0.4, 5., 0.);
                c.v = V3::default();
                c.state.injury = 0.;
            }
            e.cells[1].material.as_mut().unwrap().biomass = 0.22;
            e.cells[1].state.trophic = prey_steps;
            rebase(&mut h, &e);
            h.tick(&mut e, &mut core);
        }
        eprintln!("eater of level {prey_steps}: {} steps above the producers", e.cells[0].state.trophic);
        assert_eq!(e.cells[0].state.level(), level);
        assert!(h.web.flow[level] > 0.);
    }

    // A cell living on remains is a decomposer, whatever the remains once were.
    let (mut core, mut e, mut h) = fixture();
    for k in [0, 1] {
        e.cells[0].material.as_mut().unwrap().proteins[k] = 0.;
    }
    build(&mut e, 0, 15);
    e.cells[0].genome.genes = vec![always(15)];
    e.cells[0].material.as_mut().unwrap().fuel = 0.;
    e.cells[1].p = V3::new(0., 5., 9.);
    for s in &mut h.solutes {
        s[5] = 2.;
        s[crate::physical::REMAINS_N] = 0.3;
    }
    rebase(&mut h, &e);
    for _ in 0..160 {
        h.tick(&mut e, &mut core);
    }
    let rotter = &e.cells[0];
    assert_eq!(rotter.state.level(), crate::trophic::LEVELS, "a decomposer");
    assert!(h.web.dead > 0. && rotter.state.flux.nitrogen_given > 0., "it frees the nitrogen in remains");
    assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
}

#[test]
fn on_land_a_thick_soft_body_is_crushed_at_its_base_and_bone_or_a_wall_holds_it_up() {
    // Six cells standing on one of them. Returns the load on the footing and
    // how much more the foot was hurt than a cell it carries.
    let stand = |prop: &str, wet: f32| {
        let (_, mut e, mut h) = row(6, vec![always(0)], &[]);
        for c in &mut e.cells {
            c.state.under = wet;
        }
        e.cells[0].state.ground = true;
        match prop {
            "bone" => e.cells[0].material.as_mut().unwrap().mineral = 0.05,
            "wall" => build(&mut e, 0, WALL),
            "fibre" => e.cells[0].material.as_mut().unwrap().body.fibers = 0.05,
            _ => {}
        }
        for _ in 0..40 {
            h.land(&mut e);
        }
        (e.cells[0].state.load, e.cells[0].state.injury - e.cells[1].state.injury)
    };
    let (load, hurt) = stand("", 0.);
    eprintln!("soft body on land: load {load}, base hurt {hurt}");
    assert!(load > 1.5 && hurt > 0.2, "soft tissue was not crushed: {load}, {hurt}");
    for prop in ["bone", "wall", "fibre"] {
        let (load, hurt) = stand(prop, 0.);
        eprintln!("standing on {prop}: load {load}, base hurt {hurt}");
        assert!(load < 1. && hurt.abs() < 1e-6, "{prop} did not hold: {load}, {hurt}");
    }
    let (load, hurt) = stand("", 1.);
    assert!(load < 0.05 && hurt.abs() < 1e-6, "water carries a soft body: {load}");
}

#[test]
fn dry_air_kills_a_bare_cell_and_skin_or_wet_ground_spares_it() {
    let dried = |skin: bool, wet: f32, rain: bool| {
        let (_, mut e, mut h) = fixture();
        if skin {
            build(&mut e, 0, crate::physiology::BARRIER);
        }
        let site = h.domain.locate(e.cells[0].p);
        h.planet.as_mut().unwrap().water[site] = if rain { 1. } else { 0. };
        for _ in 0..40 {
            e.cells[0].state.under = wet;
            h.land(&mut e);
        }
        (e.cells[0].state.injury, e.cells[0].state.thirst)
    };
    let (bare, thirst) = dried(false, 0., false);
    let (skinned, _) = dried(true, 0., false);
    eprintln!("ten cycles in dry air: bare {bare}, with skin {skinned}");
    assert!(bare > 0.1 && thirst > 0.99, "{bare}, {thirst}");
    assert!(skinned < bare * 0.4, "skin keeps water in: {skinned}");
    assert_eq!(dried(false, 1., false), (0., 0.), "a cell under water does not dry");
    assert_eq!(dried(false, 0., true), (0., 0.), "nor does one on wet ground");
}

#[test]
fn fat_sheathed_nerves_carry_a_signal_further_along_a_body() {
    // A row of nerve cells, the first one stimulated. How far the signal gets.
    let reach = |fat: f64| {
        let (mut core, mut e, mut h) = row(
            6,
            vec![always(crate::physiology::CHANNEL), always(8)],
            &[crate::physiology::CHANNEL, 8],
        );
        for c in &mut e.cells {
            c.material.as_mut().unwrap().lipid = fat;
        }
        rebase(&mut h, &e);
        let mut far = 0f32;
        for _ in 0..12 {
            e.cells[0].state.excitation = 1.;
            h.excitable_signals(&mut e);
            far = far.max(e.cells[3].state.excitation);
        }
        let _ = &mut core;
        far
    };
    let (lean, fat) = (reach(0.), reach(0.3));
    eprintln!("signal reaching the fourth cell: lean {lean}, fat-sheathed {fat}");
    assert!(fat > lean * 1.5 && fat > 0.05, "{lean} against {fat}");
}

#[test]
fn matter_is_counted_in_carbon_and_fat_holds_more_energy_at_less_weight() {
    use crate::physical::{CO2, FAT_ENERGY, FAT_WEIGHT};
    // A fed cell lays fat down. The carbon it does not keep leaves as carbon
    // dioxide, and no energy is made or lost in the exchange.
    let (_, mut e, mut h) = fixture();
    build(&mut e, 0, crate::physiology::LIPID);
    e.cells[0].material.as_mut().unwrap().fuel = 0.4;
    rebase(&mut h, &e);
    let (fuel, dioxide) = (0.4, h.solutes.iter().map(|s| s[CO2]).sum::<f64>());
    for _ in 0..40 {
        h.reserves_and_repair(&mut e.cells[0]);
    }
    let m = e.cells[0].material.as_ref().unwrap();
    let (fat, spent) = (m.lipid, fuel - m.fuel);
    let given_off = h.solutes.iter().map(|s| s[CO2]).sum::<f64>() - dioxide;
    eprintln!("{spent} of sugar became {fat} of fat and {given_off} of carbon dioxide");
    assert!(fat > 0.01 && (fat * FAT_ENERGY - spent).abs() < 1e-9, "the same energy in fewer carbons");
    assert!((fat + given_off - spent).abs() < 1e-9, "every carbon accounted for");
    assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));

    // The same carbon as fat weighs under half what it does as tissue, and a
    // starving cell gets the extra energy back when it burns it.
    let lean = e.cells[1].material.as_ref().unwrap().mass();
    e.cells[1].material.as_mut().unwrap().lipid = 0.1;
    let gained = e.cells[1].material.as_ref().unwrap().mass() - lean;
    assert!((gained - 0.1 * FAT_WEIGHT).abs() < 1e-12, "fat weighs {gained} for 0.1 of carbon");
    build(&mut e, 1, crate::physiology::LIPID);
    e.cells[1].energy = 1.;
    rebase(&mut h, &e);
    let before = e.cells[1].energy;
    for _ in 0..8 {
        h.reserves_and_repair(&mut e.cells[1]);
    }
    let m = e.cells[1].material.as_ref().unwrap();
    assert!(m.lipid < 0.1 && e.cells[1].energy > before, "burning fat pays at once");
    assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
}

#[test]
fn a_wall_is_built_of_carbon_alone_and_needs_no_nitrogen() {
    use crate::physical::NITROGEN;
    // Two cells with fuel and no nitrogen anywhere: one told to build a wall,
    // the other to build a motor.
    let (mut core, mut e, mut h) = fixture();
    e.cells[1].p = V3::new(0., 5., 9.);
    for s in &mut h.solutes {
        s[NITROGEN] = 0.;
    }
    for (c, action) in e.cells.iter_mut().zip([WALL, PROPEL]) {
        c.genome.genes = vec![always(action)];
        let m = c.material.as_mut().unwrap();
        m.fuel = 0.3;
        m.nitrogen = 0.;
        m.proteins = [0.; crate::autonomous::ACTIONS];
    }
    rebase(&mut h, &e);
    for _ in 0..40 {
        h.tick(&mut e, &mut core);
    }
    let (wall, motor) = (
        e.cells[0].material.as_ref().unwrap(),
        e.cells[1].material.as_ref().unwrap(),
    );
    eprintln!("with no nitrogen: wall built {}, motor built {}", wall.proteins[WALL], motor.proteins[PROPEL]);
    assert!(wall.proteins[WALL] > 0.01, "a wall needs no nitrogen");
    assert!(motor.proteins[PROPEL] < 1e-9, "a motor is protein and cannot be built without it");
    assert!(wall.bound_nitrogen() < crate::physical::N_RATIO * wall.biomass + 1e-12, "a wall holds none");
    assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
}

#[test]
fn a_feeding_current_draws_free_small_cells_to_a_body_of_mouths_and_spares_the_bonded() {
    // A row of four mouths, and one small free cell a little way off.
    let drawn = |mouths: bool, held: bool| {
        let genes = if mouths { vec![always(ENGULF)] } else { vec![always(0)] };
        let materials: &[usize] = if mouths { &[ENGULF] } else { &[] };
        let (mut core, mut e, mut h) = row(5, genes, materials);
        for c in &mut e.cells[..4] {
            for k in [0, 1] {
                c.material.as_mut().unwrap().proteins[k] = 0.; // an animal harvests no light
            }
        }
        // The fifth cell is prey: a stranger, small, a body length from the row.
        e.bonds.truncate(3);
        let prey = &mut e.cells[4];
        prey.genome.id = 300;
        prey.genome.genes = vec![always(0), always(7), always(1)];
        prey.material.as_mut().unwrap().proteins[2] = 0.;
        prey.material.as_mut().unwrap().proteins[ENGULF] = 0.;
        prey.material.as_mut().unwrap().biomass = 0.12;
        prey.p = V3::new(3. * 0.44 + 1.1, 12., 0.);
        if held {
            // The same cell, held in a body of its own kind.
            let id = e.cells[4].id;
            let mut partner = e.cells[4].clone();
            partner.id = id + 100;
            partner.p = V3::new(3. * 0.44 + 1.5, 12., 0.);
            e.cells.push(partner);
            e.bonds.push(Bond { a: id, b: id + 100, rest: 0.4, integrity: 1., activation: 0., stiffness: 0. });
        }
        rebase(&mut h, &e);
        let start = e.cells[4].p.sub(e.cells[3].p).len();
        let mut nearest = start;
        for _ in 0..24 {
            h.tick(&mut e, &mut core);
            if e.cells.len() < 5 {
                break;
            }
            nearest = nearest.min(e.cells[4].p.sub(e.cells[3].p).len());
        }
        let eaten: f64 = e.cells[..4].iter().map(|c| c.state.flux.prey).sum();
        (start - nearest, eaten)
    };
    let (pulled, eaten) = drawn(true, false);
    let (drifted, none) = drawn(false, false);
    let (spared, _) = drawn(true, true);
    eprintln!("free cell came {pulled} closer to mouths (ate {eaten}), {drifted} closer to a body with none, {spared} when held in a body");
    assert!(pulled > 0.4 && eaten > 0.02, "the current brought it in and it was eaten: {pulled}, {eaten}");
    assert!(drifted < 0.2 && none == 0., "no mouths, no current: {drifted}");
    assert!(spared < pulled * 0.4, "a cell held in a body is not drawn: {spared}");
}

#[test]
fn a_bigger_compact_body_swims_faster_and_its_inner_cells_cost_less_to_run() {
    // A square sheet of `n` by `n` motor cells, all heading the same way.
    let swum = |n: usize| {
        let (mut core, mut e, mut h) = row(n * n, vec![always(PROPEL)], &[PROPEL]);
        e.bonds.clear();
        let id: Vec<u32> = e.cells.iter().map(|c| c.id).collect();
        for k in 0..n * n {
            let (x, z) = (k % n, k / n);
            e.cells[k].p = V3::new(x as f32 * 0.44, 12., z as f32 * 0.44);
            for (other, near) in [(k + 1, x + 1 < n), (k + n, z + 1 < n)] {
                if near {
                    e.bonds.push(Bond { a: id[k].min(id[other]), b: id[k].max(id[other]), rest: 0.44, integrity: 1., activation: 0., stiffness: 0. });
                }
            }
        }
        rebase(&mut h, &e);
        let centre = |e: &Evolution| e.cells.iter().fold(V3::default(), |s, c| s.add(c.p)).mul(1. / e.cells.len() as f32);
        let start = centre(&e);
        for _ in 0..40 {
            for c in &mut e.cells {
                c.energy = 8.; // this is about speed, not endurance
            }
            h.tick(&mut e, &mut core);
        }
        (centre(&e).sub(start).x, e.cells.len())
    };
    let ((one, _), (nine, _), (many, kept)) = (swum(1), swum(3), swum(5));
    eprintln!("ten cycles of swimming: one cell {one}, a body of nine {nine}, a body of twenty-five {many}");
    assert_eq!(kept, 25, "the body held together");
    assert!(nine > one * 1.3 && many > nine, "a bigger body is faster: {one}, {nine}, {many}");

    // The same cell alone and wrapped in six neighbours: what it pays to stay alive.
    let upkeep = |bonds: usize| {
        let (mut core, mut e, mut h) = row(7, vec![always(7)], &[]);
        e.bonds.truncate(0);
        let id: Vec<u32> = e.cells.iter().map(|c| c.id).collect();
        for k in 1..=bonds {
            e.bonds.push(Bond { a: id[0].min(id[k]), b: id[0].max(id[k]), rest: 0.44, integrity: 1., activation: 0., stiffness: 0. });
        }
        for c in &mut e.cells {
            c.material.as_mut().unwrap().fuel = 0.;
            c.material.as_mut().unwrap().proteins = [0.; crate::autonomous::ACTIONS];
            c.energy = 8.;
        }
        rebase(&mut h, &e);
        h.physiology(&mut e, &core);
        let before = e.cells[0].energy;
        h.physiology(&mut e, &core);
        let _ = &mut core;
        before - e.cells[0].energy
    };
    let (alone, enclosed) = (upkeep(0), upkeep(6));
    eprintln!("upkeep a tick: alone {alone}, enclosed {enclosed}");
    assert!(enclosed < alone * 0.6 && enclosed > alone * 0.4, "{alone} against {enclosed}");
}

#[test]
fn the_planets_own_chemistry_moves_matter_between_air_water_and_ground_and_makes_none() {
    use crate::physical::{CO2, IRON, REMAINS_N, SALT};
    let sum = |h: &Habitat, k: usize| h.solutes.iter().map(|s| s[k]).sum::<f64>();
    // Carbon dioxide: a sea with none takes it from the air until they balance.
    let (_, e, mut h) = fixture();
    for s in &mut h.solutes {
        s[CO2] = 0.;
    }
    rebase(&mut h, &e);
    let air = h.air_carbon;
    for _ in 0..400 {
        h.planet_chemistry(0.25);
    }
    let dissolved = sum(&h, CO2);
    eprintln!("air held {air}; after a hundred cycles the air holds {} and the water {dissolved}", h.air_carbon);
    assert!(dissolved > air * 0.3 && h.air_carbon < air * 0.7, "the sea took carbon dioxide from the air");
    assert!((h.air_carbon + dissolved - air).abs() < 1e-9, "and none was made");

    // Remains are silted over; the planet's heat burns buried carbon back
    // out with free oxygen. Carbon, nitrogen and energy all still balance.
    let (_, e, mut h) = fixture();
    for s in &mut h.solutes {
        s[5] = 1.;
        s[REMAINS_N] = 0.1;
    }
    rebase(&mut h, &e);
    let lying = sum(&h, 5);
    for _ in 0..400 {
        h.planet_chemistry(0.25);
    }
    eprintln!("of {lying} remains, {} still lie and {} are buried", sum(&h, 5), h.buried_carbon);
    assert!(sum(&h, 5) < lying * 0.8 && h.buried_carbon > lying * 0.15 && h.buried_nitrogen > 0.);
    assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
    // With nothing left lying to bury, more comes out than goes in.
    for s in &mut h.solutes {
        s[5] = 0.;
        s[REMAINS_N] = 0.;
    }
    rebase(&mut h, &e);
    let (buried, oxygen) = (h.buried_carbon, h.oxygen());
    h.steering.vents = 8.;
    for _ in 0..400 {
        h.planet_chemistry(0.25);
    }
    assert!(h.buried_carbon < buried && h.oxygen() < oxygen, "outgassing burns buried carbon with free oxygen");
    assert!(balanced(&h, &e) && h.systems.gas_error(&e.cells).abs() < 1e-7, "{:?}", h.errors(&e.cells));

    // Iron stays dissolved in a sea without oxygen and rusts out of one with it.
    let rusted = |air: f64| {
        let (_, e, mut h) = fixture();
        h.oxygenate(air);
        let (iron, oxygen) = (sum(&h, IRON), h.oxygen());
        for _ in 0..400 {
            h.planet_chemistry(0.25);
        }
        assert!((sum(&h, IRON) + h.rusted_iron - iron).abs() < 1e-9, "iron is neither made nor lost");
        assert!(h.systems.gas_error(&e.cells).abs() < 1e-7);
        (h.rusted_iron / iron, oxygen - h.oxygen())
    };
    let ((none, _), (most, taken)) = (rusted(0.), rusted(1.));
    eprintln!("iron rusted: {none} without oxygen, {most} with it, taking {taken} of the free oxygen");
    assert!(none == 0. && most > 0.9 && taken > 0.);
    assert!(sum(&h, SALT) > 0., "every world has salt");
}

#[test]
fn phosphorus_comes_from_rock_is_lent_by_bone_and_passes_on_in_food() {
    use crate::physical::P_RATIO;
    // A fed cell with nitrogen to spare, in a sea with dissolved mineral or
    // with none, and with or without bone of its own.
    let grown = |sea: f64, bone: f64| {
        let (mut core, mut e, mut h) = fixture();
        e.cells[1].p = V3::new(0., 5., 9.);
        for s in &mut h.solutes {
            s[2] = sea;
        }
        for c in &mut e.cells {
            c.genome.genes = vec![always(7)];
            let m = c.material.as_mut().unwrap();
            m.fuel = 0.3;
            m.nitrogen = 0.1;
            m.phosphorus = 0.;
            m.mineral = bone;
        }
        rebase(&mut h, &e);
        let before = biomass(&e, 0);
        for _ in 0..80 {
            h.tick(&mut e, &mut core);
        }
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        (biomass(&e, 0) - before, e.cells[0].state.phosphorus_short)
    };
    let ((fed, _), (starved, short), (lent, _)) = (grown(0.2, 0.), grown(0., 0.), grown(0., 0.02));
    eprintln!("growth with dissolved mineral {fed}, with none {starved}, with none but bone {lent}");
    assert!(fed > 0.05 && starved < fed * 0.2 && short > 0.5, "{fed}, {starved}, {short}");
    assert!(lent > starved * 3. && lent > 0.02, "bone lends its phosphorus: {lent}");

    // An eater gets the phosphorus of what it digests.
    let (mut core, mut e, mut h) = fixture();
    for k in [0, 1] {
        e.cells[0].material.as_mut().unwrap().proteins[k] = 0.;
    }
    e.cells[0].material.as_mut().unwrap().phosphorus = 0.;
    build(&mut e, 0, 9);
    e.cells[0].genome.genes = vec![always(9)];
    e.cells[1].genome.id = 300;
    rebase(&mut h, &e);
    for _ in 0..40 {
        for (i, c) in e.cells.iter_mut().enumerate() {
            c.p = V3::new(i as f32 * 0.4, 5., 0.);
            c.v = V3::default();
            c.state.injury = 0.;
        }
        h.tick(&mut e, &mut core);
    }
    let eater = e.cells[0].material.as_ref().unwrap();
    assert!(e.cells[0].state.flux.prey > 0.01 && eater.phosphorus > 0., "{}", eater.phosphorus);
    assert!(P_RATIO > 0. && balanced(&h, &e), "{:?}", h.errors(&e.cells));
}

#[test]
fn a_cells_size_is_inherited_and_sets_how_it_feeds_breeds_and_what_it_costs() {
    use crate::physical::SIZE;
    // One light-feeding cell of each size, alone in the same lit sea.
    let lived = |drive: f32| {
        let (mut core, mut e, mut h) = fixture();
        e.cells.truncate(1);
        for s in &mut h.solutes {
            s[0] = 0.;
            s[1] = 1.;
        }
        let stature = 4f64.powf(2. * drive as f64 - 1.);
        h.steering.light = 6.; // a bright star, as new planets have
        h.steering.mutation_scale = 0.; // this is about one size, not about size evolving
        let c = &mut e.cells[0];
        c.genome.genes = vec![always(0), always(7), Gene { action: SIZE, promoter: [0.; INPUTS], bias: 1., strength: drive, fates: 15 }];
        c.cooldown = 0.;
        let m = c.material.as_mut().unwrap();
        m.biomass *= stature;
        m.nitrogen = 1.;
        m.phosphorus = 1.;
        m.fuel = 0.;
        for p in &mut m.proteins {
            *p *= stature;
        }
        m.proteins[0] = 0.02 * stature;
        c.energy = 4. * stature as f32;
        rebase(&mut h, &e);
        let start = e.cells[0].material.as_ref().unwrap().biomass;
        let (mut harvest, mut divided) = (0., None);
        for k in 0..400 {
            for c in &mut e.cells {
                c.state.thirst = 0.;
            }
            h.tick(&mut e, &mut core);
            if k == 7 {
                harvest = e.cells[0].state.flux.photosynthesis / 2.;
            }
            if divided.is_none() && e.cells.len() > 1 {
                divided = Some((k + 1) as f32 / 4.);
            }
        }
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        let size = e.cells.iter().map(|c| c.material.as_ref().unwrap().biomass).fold(0., f64::max);
        (start, harvest, divided.unwrap_or(f32::INFINITY), size, e.cells[0].state.stature)
    };
    let (small, usual, big) = (lived(0.1), lived(0.5), lived(0.9));
    for (name, r) in [("small", small), ("usual", usual), ("big", big)] {
        eprintln!("{name}: starts at {:.3}, harvests {:.4} a cycle ({:.3} per unit of itself), first divides after {} cycles, grows to {:.3}", r.0, r.1, r.1 / r.0, r.2, r.3);
    }
    assert!((usual.4 - 1.).abs() < 1e-6 && small.4 < 0.4 && big.4 > 2.5, "size follows the gene");
    assert!(big.1 > usual.1 && usual.1 > small.1, "a bigger cell harvests more");
    assert!(small.1 / small.0 > usual.1 / usual.0 && usual.1 / usual.0 > big.1 / big.0, "but less for its size");
    assert!(small.2 < usual.2 && usual.2 < big.2, "and a smaller cell breeds sooner: {} {} {}", small.2, usual.2, big.2);
    assert!(small.3 < usual.3 * 0.5 && big.3 > usual.3 * 2., "cells stay the size their genes set");
}

#[test]
fn a_mid_sized_planet_runs_and_reloads() {
    let mut c = Core::new(Config {
        initial_cells: 24,
        world_size: 512,
        world_shape: crate::domain::Shape::Sphere,
        ..Config::default()
    })
    .unwrap();
    c.start_physical();
    c.step(40);
    let e = c.autonomous.as_ref().unwrap();
    let h = e.physical.as_ref().unwrap();
    assert_eq!((h.domain.size(), h.domain.grid), (512., 160));
    assert!(e.valid(), "{:?}", h.errors(&e.cells));
    assert!(Core::load(&serde_json::to_string(&c).unwrap()).is_ok());
}
