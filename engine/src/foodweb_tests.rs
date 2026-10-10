//! Supplied-mechanism fixtures for movement, attack, defence and chemosynthesis.
//! They check what each material does once a cell has it, never whether
//! ordinary founders evolve it.
use crate::{
    autonomous::{Bond, Evolution, Gene, INPUTS, LATERAL},
    body::V3,
    physical::{Habitat, CHEMO, ENGULF, PROPEL, SPINE, STEER, TOXIN},
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
        }];
    }
    e.next_genome = 200;
    e.bonds.clear();
    rebase(&mut h, &e);
    (core, e, h)
}
fn rebase(h: &mut Habitat, e: &Evolution) {
    (h.initial_organic, h.initial_mineral, h.initial_energy) = h.totals(&e.cells);
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
        (e.cells[0].p.x - start, e.cells[0].state.motor_work, e.cells[0].energy)
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
    assert!(e.bonds[0].integrity > 0.9, "bond integrity {}", e.bonds[0].integrity);
    assert!(moved.0 > 0.1 && moved.1 > 0.1, "both travelled: {moved:?}");
    assert!((moved.0 - moved.1).abs() < 0.2, "and stayed together: {moved:?}");
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
    assert!(crude > 0. && crude < turned(1., true), "steering alone senses crudely: {crude}");
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
    assert!(equal > 0. && equal < eaten * 0.5, "an equal goes down slowly: {equal} vs {eaten}");
    assert!(eaten > 0.02 && gained > 0., "bigger cell ate {eaten}");
    assert!(blunted < eaten * 0.5, "spines: {blunted} vs {eaten}");
    assert!(pricked > unhurt, "spines wound the eater");
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
        assert!(harm(true) < harm(false) * 0.05, "weapon {weapon} spares its own kind");
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
    assert!(e.cells[0].state.symbiont > 0.25, "no capture in {meals} meals");
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
    assert!(harvest(0.9) > harvest(0.) * 1.5, "{} vs {}", harvest(0.9), harvest(0.));

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
    assert!(loose > 0. && held < loose * 0.4, "held prey resists: {held} vs {loose}");
    let big = bite(8, 3);
    assert!(big > held * 2., "a big hunter tears it off: {big} vs {held}");
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
    assert!(harm(true) > 0., "carrying a mouth is no shield against a stranger");
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
    assert!(biomass(&e, 0) > 0.6, "mouth cell reached {}", biomass(&e, 0));
    assert!(biomass(&e, 1) <= 0.5 + 1e-9, "ordinary cell stays at {}", biomass(&e, 1));

    // Sticky cells bond on contact. A mouth will not bond to an unarmed
    // stranger, but it will to unarmed kin: that is how a body gets a mouth.
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
    assert!(bonded(false, true), "unarmed strangers still stick");
    assert!(!bonded(true, true), "a mouth does not glue itself to a stranger");
    assert!(bonded(true, false), "a mouth bonds to its unarmed kin");
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
        (fuel(0), fuel(1), e.cells[0].state.flux.shared, e.cells[1].state.flux.fed)
    };
    // After a meal the mouth pours fuel into its neighbour until they level.
    let (mouth, fed_cell, shared, fed) = share(true, true, 0.3, 0.);
    assert!(fed_cell > 0.1 && (mouth - fed_cell).abs() < 0.05, "{mouth} {fed_cell}");
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
        cell["material"]["proteins"].as_array_mut().unwrap().truncate(32);
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
