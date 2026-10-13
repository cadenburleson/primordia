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
        // And able to breathe, whatever its membrane is given over to.
        c.state.mitochondria = 1.;
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
    e.cells[1].energy = 0.01; // a drive, but nothing to pay with:
    // no fuel, and no other machinery to break down for it
    let m = e.cells[1].material.as_mut().unwrap();
    m.fuel = 0.;
    for k in (0..m.proteins.len()).filter(|&k| k != PROPEL) {
        m.proteins[k] = 0.;
    }
    rebase(&mut h, &e);
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
fn a_swimmer_pointed_at_the_sky_does_not_leave_the_water() {
    let (mut core, mut e, mut h) = fixture();
    e.cells.truncate(1);
    build(&mut e, 0, PROPEL);
    e.cells[0].genome.genes.push(always(PROPEL));
    rebase(&mut h, &e);
    let start = e.cells[0].p.y;
    let mut top = start;
    for _ in 0..400 {
        // Held nose up and kept fed: all it can do is try to climb.
        e.cells[0].state.pitch = 1.;
        e.cells[0].energy = 8.;
        h.tick(&mut e, &mut core);
        top = top.max(e.cells[0].p.y);
    }
    assert!(e.cells[0].state.motor_work > 0., "its motor ran");
    assert!(top < start + 0.5, "it rose {} with no water over it", top - start);
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
fn light_feeders_free_oxygen_the_air_carries_it_and_breathing_gives_fifteen_times_what_fermenting_does() {
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
    assert!(fermenting > 0. && (breathing / fermenting - 15.).abs() < 1.5, "{breathing} against {fermenting}");

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
    while e.cells[0].state.symbiont == 0. && meals < 2_000_000 {
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

#[test]
fn breathing_takes_machinery_a_mouth_crowds_out_and_a_kept_breather_restores() {
    use crate::physical::{RESPIRE, SIZE};
    // One cell with fuel to burn in water full of oxygen. Returns the share
    // of its fuel it burned right down.
    let breath = |machinery: bool, mouth: bool, kept: f32, big: bool| {
        let (mut core, mut e, mut h) = fixture();
        e.cells.truncate(1);
        let c = &mut e.cells[0];
        c.state.mitochondria = kept;
        c.energy = 1.;
        if big {
            c.genome.genes = vec![Gene { action: SIZE, promoter: [0.; INPUTS], bias: 1., strength: 1., fates: 15 }];
        }
        let m = c.material.as_mut().unwrap();
        m.proteins = [0.; crate::autonomous::ACTIONS];
        if big {
            m.biomass *= 4.;
        }
        m.fuel = 0.3;
        if machinery {
            m.proteins[RESPIRE] = 0.5 * m.biomass;
        }
        if mouth {
            m.proteins[ENGULF] = 0.5 * m.biomass;
        }
        rebase(&mut h, &e);
        for _ in 0..4 {
            h.tick(&mut e, &mut core);
        }
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        e.cells[0].state.breath
    };
    let (none, own) = (breath(false, false, 0., false), breath(true, false, 0., false));
    let (crowded, restored) = (breath(true, true, 0., false), breath(false, true, 1., false));
    let (outsized, housed) = (breath(true, false, 0., true), breath(false, false, 1., true));
    eprintln!("share of fuel breathed: no machinery {none}, its own {own}, with a mouth {crowded}, a mouth and a kept breather {restored}, four times the size {outsized}, that size with kept breathers {housed}");
    assert!(none < 0.02, "oxygen is no use without the machinery: {none}");
    assert!(own > 0.9, "with it a cell breathes: {own}");
    assert!(crowded < 0.3, "a mouth leaves little membrane to breathe through: {crowded}");
    assert!(restored > 0.9, "a breather kept inside needs none: {restored}");
    assert!(outsized < own * 0.75 && outsized > 0.4, "a big cell has too little membrane for its bulk: {outsized}");
    assert!(housed > 0.9, "kept breathers fill a cell of any size: {housed}");
}

#[test]
fn nitrogen_is_fixed_from_the_air_at_a_price_and_only_out_of_reach_of_oxygen() {
    use crate::physical::FIX;
    // One cell short of nitrogen, for a cycle. Returns what it fixed and the
    // energy it was left with.
    let fixed = |air: f64, wall: bool, light: bool, supply: bool| {
        let (mut core, mut e, mut h) = fixture();
        e.cells.truncate(1);
        h.oxygenate(air);
        for s in &mut h.solutes {
            s[1] = 2.;
            s[crate::physical::NITROGEN] = 0.;
        }
        if !supply {
            h.air_nitrogen = 0.;
        }
        e.cells[0].energy = 3.;
        let m = e.cells[0].material.as_mut().unwrap();
        m.proteins = [0.; crate::autonomous::ACTIONS];
        m.proteins[FIX] = 0.02;
        m.nitrogen = 0.;
        m.fuel = 0.;
        if wall {
            m.proteins[WALL] = 0.2;
        }
        if light {
            m.proteins[0] = 0.02;
        }
        rebase(&mut h, &e);
        let air_before = h.air_nitrogen;
        for _ in 0..4 {
            h.tick(&mut e, &mut core);
        }
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        let got = h.physiology.nitrogen_fixed;
        assert!((air_before - h.air_nitrogen - got).abs() < 1e-6, "what a cell fixes comes out of the air");
        (got, e.cells[0].energy as f64)
    };
    let (anoxic, bare, walled, lit) = (
        fixed(0., false, false, true),
        fixed(1., false, false, true),
        fixed(1., true, false, true),
        fixed(0., false, true, true),
    );
    // The same cell with no nitrogen in the air to fix keeps what fixing costs.
    let price = (fixed(0., false, false, false).1 - anoxic.1) / anoxic.0;
    eprintln!("fixed in a cycle: no oxygen {:.5} at {price:.0} energy a unit; in oxygen bare {:.5}, walled {:.5}; harvesting light {:.5}", anoxic.0, bare.0, walled.0, lit.0);
    assert!(anoxic.0 > 0., "with no oxygen about, the enzyme works");
    assert!((price - 80.).abs() < 8., "and it is dear: {price} a unit");
    assert!(anoxic.0 > 0., "with no oxygen about, the enzyme works");
    assert!(bare.0 < anoxic.0 * 0.1, "oxygen stops it: {} against {}", bare.0, anoxic.0);
    assert!(walled.0 > anoxic.0 * 0.5, "a wall keeps oxygen off it: {} against {}", walled.0, anoxic.0);
    assert!(lit.0 < anoxic.0 * 0.2, "a cell making oxygen of its own cannot fix beside it: {} against {}", lit.0, anoxic.0);
}

#[test]
fn a_swallowed_breather_can_be_kept_and_the_hosts_descendants_inherit_it() {
    use crate::physical::RESPIRE;
    // Keep offering an engulfer fresh breathing prey until one is kept.
    let (mut core, mut e, mut h) = fixture();
    build(&mut e, 0, ENGULF);
    e.cells[0].state.mitochondria = 0.;
    e.cells[0].material.as_mut().unwrap().biomass *= 2.;
    e.cells[1].p = V3::new(e.cells[0].radius() + e.cells[1].radius(), 5., 0.);
    build(&mut e, 1, RESPIRE);
    let mut meals = 0;
    while e.cells[0].state.mitochondria == 0. && meals < 5000 {
        e.cells[0].material.as_mut().unwrap().fuel = 0.;
        e.cells[0].energy = 8.;
        e.cells[1].material.as_mut().unwrap().biomass = 0.22;
        e.cells[1].state.injury = 0.;
        h.contacts(&mut e, &mut core);
        meals += 1;
    }
    assert!(e.cells[0].state.mitochondria > 0.25, "no breather kept in {meals} meals");
    assert!(meals > 1, "keeping one is a rare event, not the rule");
    assert_eq!(h.steering.captures, 1);
    // Prey with no breathing machinery leaves nothing to keep.
    let (mut core, mut e, mut h) = fixture();
    build(&mut e, 0, ENGULF);
    e.cells[0].state.mitochondria = 0.;
    e.cells[0].material.as_mut().unwrap().biomass *= 2.;
    e.cells[1].p = V3::new(e.cells[0].radius() + e.cells[1].radius(), 5., 0.);
    e.cells[1].material.as_mut().unwrap().proteins[RESPIRE] = 0.;
    for _ in 0..2000 {
        e.cells[0].material.as_mut().unwrap().fuel = 0.;
        e.cells[0].energy = 8.;
        e.cells[1].material.as_mut().unwrap().biomass = 0.22;
        e.cells[1].state.injury = 0.;
        h.contacts(&mut e, &mut core);
    }
    assert_eq!(e.cells[0].state.mitochondria, 0.);

    // Daughters inherit the breathers their mother carried.
    let mut world = Core::new(Config {
        initial_cells: 16,
        ..Config::default()
    })
    .unwrap();
    world.start_physical();
    for c in &mut world.autonomous.as_mut().unwrap().cells {
        c.state.mitochondria = 0.5;
    }
    world.step(1200);
    let e = world.autonomous.as_ref().unwrap();
    // (A descendant that has since grown a mouth may have swallowed and
    // kept a better breather, which only ever raises what it carries.)
    assert!(e.divisions > 0 && e.cells.iter().all(|c| c.state.mitochondria >= 0.5));
}

/// A hunter of the given bulk beside one prey cell that has `held` bonds to
/// bystanders. Returns the biomass the prey lost in one tick.
fn bite(hunter_cells: usize, held: usize) -> f64 {
    bite_by(hunter_cells, hunter_cells, held, 1.)
}

/// The same, where only the first `mouths` cells behind the leading one are
/// mouths like it and the rest of the hunter is plain body, and the prey
/// cell is `size` times the usual.
fn bite_by(hunter_cells: usize, mouths: usize, held: usize, size: f64) -> f64 {
    bite_with(hunter_cells, mouths, held, size, true, 0.)
}

/// The same, with or without muscle in the mouth to open it, and with
/// `matrix` laid down around the prey cell. With `mouths` of `usize::MAX`
/// the hunter digests from outside with enzymes and has no mouth at all.
fn bite_with(hunter_cells: usize, mouths: usize, held: usize, size: f64, muscle: bool, matrix: f64) -> f64 {
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
    build(&mut e, 0, if mouths == usize::MAX { 9 } else { ENGULF });
    if muscle {
        build(&mut e, 0, 4);
    }
    e.cells[1].material.as_mut().unwrap().body.matrix = matrix;
    e.bonds.clear();
    // The rest of the hunter's body trails behind the mouth.
    for k in 0..hunter_cells {
        let i = 2 + k;
        e.cells[i].p = V3::new(-0.44 * (k + 1) as f32, 5., 0.);
        if k < mouths && mouths != usize::MAX {
            build(&mut e, i, ENGULF);
        }
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
    e.cells[1].material.as_mut().unwrap().biomass *= size;
    h.contacts(&mut e, &mut core);
    e.cells[1].state.flux.lost_to_predators
}

#[test]
fn what_a_mouth_can_take_is_set_by_the_mouth_and_not_by_the_body_behind_it() {
    // One mouth cell on a body of eight plain cells bites no harder than a
    // mouth cell on its own. Eight mouth cells side by side are a big mouth.
    let (alone, small_mouth, big_mouth) = (bite(0, 3), bite_by(8, 0, 3, 1.), bite(8, 3));
    assert!(
        (small_mouth - alone).abs() < alone * 0.05 && big_mouth > small_mouth * 2.,
        "one mouth cell alone {alone}, on a big body {small_mouth}; a mouth of nine {big_mouth}"
    );
    // Prey half again the size of a mouth cell is too big for one, whatever
    // body is behind it, and not for a mouth of three.
    assert_eq!(bite_by(8, 0, 0, 1.5), 0., "too big for a small mouth on a big body");
    assert!(bite_by(8, 2, 0, 1.5) > 0., "a mouth of three takes it");
    // A body of two cells goes whole into a mouth that is bigger than both
    // together: holding on to each other does not save them. Against a mouth
    // no bigger than one of them it does.
    let (loose, pair_small, pair_big) = (bite(0, 0), bite(0, 1), bite(8, 1));
    assert!(pair_small < loose * 0.6, "a pair resists a mouth its own size: {pair_small} vs {loose}");
    assert!(pair_big > loose, "and is swallowed whole by a big one: {pair_big} vs {loose}");
}

#[test]
fn a_cell_in_a_body_is_held_by_the_body_however_big_the_mouth_that_pulls_at_it() {
    // A mouth three times the size of one cell cannot take the body that
    // cell belongs to, and gets the cell no faster for its size: what it
    // pulls against is the body. Prey here is a body of four, each cell a
    // third the size of the mouth.
    let (loose, held_small) = (bite(0, 0), bite(0, 3));
    let (loose_big, held_big) = (bite(2, 0), bite(2, 3));
    assert!(held_small < loose * 0.15, "three ties hold: {held_small} of {loose}");
    assert!(held_big < loose_big * 0.15, "against a mouth of three as well: {held_big} of {loose_big}");
    // A mouth that can take all four at once does.
    assert!(bite(8, 3) > loose, "and a mouth of nine swallows the lot");
    // Matrix between the cells holds them harder still.
    let bound = bite_with(0, 0, 3, 1., true, 0.05);
    assert!(bound < held_small * 0.7, "matrix holds harder: {bound} vs {held_small}");
}

#[test]
fn a_mouth_of_many_cells_opens_only_by_muscle() {
    // Nine mouth cells with muscle among them are one big mouth and take
    // prey half again the size of any one of them. Without muscle they are
    // nine small mouths, and it is too big for every one.
    assert!(bite_with(8, 8, 0, 1.5, true, 0.) > 0.);
    assert_eq!(bite_with(8, 8, 0, 1.5, false, 0.), 0.);
    // What one cell can swallow it takes either way.
    assert!(bite_with(8, 8, 0, 0.5, false, 0.) > 0.);
}

#[test]
fn a_wide_mouth_cannot_keep_hold_of_prey_far_smaller_than_itself() {
    // A mouth of nine cells, opened by muscle, loses a loose cell a
    // twentieth of its own bulk with the water, and keeps one a quarter of
    // it. One mouth cell on its own takes the small one.
    assert_eq!(bite_by(8, 8, 0, 0.45), 0., "it slips out of a mouth of nine");
    assert!(bite_by(8, 8, 0, 2.2) > 0., "a mouth of nine holds prey a quarter its size");
    assert!(bite(0, 0) > 0. && bite_by(0, 0, 0, 0.45) > 0., "one cell wraps round the small one");
    // A small body that goes in whole is judged by the whole of it, and a
    // piece torn from a body too big to swallow is held by the bite.
    assert!(bite_by(8, 8, 1, 0.45) > 0., "two small cells together are a mouthful");
}

#[test]
fn the_matrix_of_a_tissue_keeps_enzymes_out() {
    let (bare, bound) = (bite_with(0, usize::MAX, 0, 1., false, 0.), bite_with(0, usize::MAX, 0, 1., false, 0.05));
    assert!(bare > 0. && bound < bare * 0.3, "digested through matrix at {bound} of {bare}");
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
        } else {
            // Kin here are two cells of one individual.
            e.cells[1].lifecycle.root = e.cells[0].id;
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
    // With the vents shut there is no hydrogen, and the catalyst makes nothing.
    let (starved, _) = fed(true, 0.);
    assert_eq!((none, no_input), (0., 0.));
    assert_eq!(starved, 0.);
    assert!(natural > 0. && input > 0.);
    assert!(active > natural * 1.3, "{active} vs {natural}");
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
    // Such a world had no breathing gene: take it and its machinery out.
    for c in &mut core.autonomous.as_mut().unwrap().cells {
        c.genome.genes.retain(|g| g.action < 32);
        let m = c.material.as_mut().unwrap();
        let p = std::mem::take(&mut m.proteins[crate::physical::RESPIRE]);
        m.fuel += p;
        m.nitrogen += p * crate::physical::N_RATIO;
    }
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
#[ignore = "the one-way fin grip this rested on is gone (docs/ANIMAL_BODIES_PLAN.md); a body that only squeezes should go nowhere, fins or not"]
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
    // Each cell that touches a cell covers a twelfth of its surface: one
    // neighbour costs it little, four cost a third.
    let (mut core, mut e, mut h) = row(5, vec![always(0)], &[]);
    let hub_id = e.cells[0].id;
    for (k, b) in e.bonds.iter_mut().enumerate() {
        (b.a, b.b) = (hub_id, e.cells[k + 1].id);
    }
    let middle = e.cells[0].p;
    for (k, at) in [(0.44, 0.), (-0.44, 0.), (0., 0.44), (0., -0.44)].into_iter().enumerate() {
        e.cells[k + 1].p = V3::new(middle.x + at.0, middle.y, middle.z + at.1);
    }
    // What touches a cell is found in one moment and felt in the next.
    h.tick(&mut e, &mut core);
    h.tick(&mut e, &mut core);
    let open = |i: usize| e.cells.iter().find(|c| c.id == hub_id + i as u32).unwrap().state.exposure;
    assert!(open(1) > 0.9 && (open(0) - 0.667).abs() < 0.01, "arm {}, hub {}", open(1), open(0));
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
#[ignore = "the one-way fin grip this rested on is gone (docs/ANIMAL_BODIES_PLAN.md); to be rewritten with the rebuilt bodies"]
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


/// Chains of bones of several lengths, set in a zigzag so every joint is at
/// a wide angle, with muscle and a beat-maker at each joint and fins on
/// every bone: how far each goes in eighty cycles. Nothing depends on it.
#[test]
#[ignore]
fn probe_jointed_chain() {
    let bend: f32 = std::env::var("BEND").ok().and_then(|v| v.parse().ok()).unwrap_or(0.114);
    eprintln!("CHAIN each joint set {bend} to one side and the other in turn (0 is a straight line)");
    for (n, fins_on) in [(3, "ends"), (5, "ends"), (5, "all"), (9, "all"), (15, "all"), (15, "ends")] {
        let (mut core, mut e, mut h) = row(n, vec![always(0)], &[]);
        let first = e.cells[0].p;
        for (i, c) in e.cells.iter_mut().enumerate() {
            c.p = V3::new(first.x + i as f32 * 0.425, first.y, first.z + if i % 2 == 0 { bend } else { 0. });
            let m = c.material.as_mut().unwrap();
            m.mineral = 0.12 * m.mass();
            let end = i == 0 || i == n - 1;
            if !end {
                m.proteins[4] = 0.05;
                m.proteins[crate::physiology::CHANNEL] = 0.05;
                c.genome.genes.extend([always(4), always(crate::physiology::CHANNEL)]);
            }
            if end || fins_on == "all" {
                m.proteins[FIN] = 0.05;
                c.genome.genes.push(always(FIN));
            }
            c.genome.id = 150 + u32::from(end);
        }
        rebase(&mut h, &e);
        let centre = |e: &Evolution| e.cells.iter().fold(V3::default(), |s, c| s.add(c.p)).mul(1. / e.cells.len() as f32);
        let start = centre(&e);
        let (mut least, mut most) = (f32::MAX, 0f32);
        for _ in 0..80 {
            for s in &mut h.solutes {
                s[3] = 0.045;
            }
            travel(&mut core, &mut e, &mut h, 4);
            for j in &h.joints {
                least = least.min(j.angle);
                most = most.max(j.angle);
            }
        }
        let went = centre(&e).sub(start);
        let length = (n - 1) as f32 * 0.44;
        eprintln!(
            "CHAIN of {n} bones, fins on {fins_on}: went {:.2} along and {:.2} across in 80 cycles: {:.3} units a cycle, {:.3} body lengths a cycle; joints {} swinging {:.2} rad; ties left {} of {}; work {:.1}",
            went.x, went.z, went.x / 80., went.x / 80. / length, h.joints.len(), most - least, e.bonds.len(), n - 1,
            h.joints.iter().map(|j| j.work).sum::<f64>(),
        );
    }
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
fn many_mouths_pumping_together_draw_from_farther_than_one() {
    // A small free cell two and a half units from the end of a row of
    // mouths: farther than one mouth's current carries.
    let drawn = |mouths: usize| {
        let (mut core, mut e, mut h) = row(mouths + 1, vec![always(ENGULF)], &[ENGULF]);
        for c in &mut e.cells[..mouths] {
            for k in [0, 1] {
                c.material.as_mut().unwrap().proteins[k] = 0.;
            }
        }
        e.bonds.truncate(mouths - 1);
        let last = e.cells[mouths - 1].p;
        let prey = &mut e.cells[mouths];
        prey.genome.id = 300;
        // Small enough for any one of them, and it stays so: with no
        // muscle to open them together they are so many small mouths, as a
        // sponge's are.
        prey.genome.genes = vec![always(1)];
        for k in [0, 2, ENGULF] {
            prey.material.as_mut().unwrap().proteins[k] = 0.;
        }
        prey.material.as_mut().unwrap().biomass = 0.06;
        prey.p = V3::new(last.x + 2.5, last.y, last.z);
        rebase(&mut h, &e);
        let start = e.cells[mouths].p.sub(e.cells[mouths - 1].p).len();
        let mut nearest = start;
        for _ in 0..80 {
            h.tick(&mut e, &mut core);
            if e.cells.len() <= mouths {
                break;
            }
            nearest = nearest.min(e.cells[mouths].p.sub(e.cells[mouths - 1].p).len());
        }
        start - nearest
    };
    let (one, eight) = (drawn(1), drawn(8));
    eprintln!("a free cell two and a half units off came {one} closer to one mouth and {eight} closer to eight");
    assert!(one < 0.5, "it is beyond the reach of one mouth: {one}");
    assert!(eight > 1., "eight pumping together bring it in: {eight}");
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

    // The same cell alone and packed in on every side, with twelve cells
    // touching it: what it pays to stay alive.
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
        e.cells[0].state.touching = (bonds * 2) as u8;
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
        let (mut harvest, mut divided, mut grown) = (0., None, None);
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
            // Big enough to divide: when it then does is partly chance.
            if grown.is_none()
                && e.cells[0].material.as_ref().unwrap().biomass >= 0.38 * e.cells[0].stature()
            {
                grown = Some((k + 1) as f32 / 4.);
            }
        }
        assert!(balanced(&h, &e), "{:?}", h.errors(&e.cells));
        let size = e.cells.iter().map(|c| c.material.as_ref().unwrap().biomass).fold(0., f64::max);
        assert!(divided.is_some(), "it bred");
        (start, harvest, grown.unwrap_or(f32::INFINITY), size, e.cells[0].state.stature)
    };
    let (small, usual, big) = (lived(0.1), lived(0.5), lived(0.9));
    for (name, r) in [("small", small), ("usual", usual), ("big", big)] {
        eprintln!("{name}: starts at {:.3}, harvests {:.4} a cycle ({:.3} per unit of itself), is big enough to divide after {} cycles, grows to {:.3}", r.0, r.1, r.1 / r.0, r.2, r.3);
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

#[test]
fn a_young_planet_is_struck_and_its_air_burns_up_more_of_what_falls_as_it_thickens() {
    // The same sky over the same planet twice: once as it is, once with
    // nearly all of its air taken away.
    let run = |air: f64| {
        let mut core = Core::new(Config {
            world_size: 64,
            world_shape: crate::domain::Shape::Sphere,
            initial_cells: 16,
            ..Config::default()
        })
        .unwrap();
        core.start_physical();
        core.steer(r#"{"kind":"bombardment","scale":8}"#).unwrap();
        let h = core.autonomous.as_mut().unwrap().physical.as_mut().unwrap();
        let rock = h.rock_nitrogen;
        // Taken out of the world altogether, books and all.
        let gone = h.air_nitrogen * (1. - air) + h.rock_nitrogen * (1. - air) + h.air_carbon * (1. - air);
        h.initial_nitrogen -= (h.air_nitrogen + h.rock_nitrogen) * (1. - air);
        h.initial_organic -= h.air_carbon * (1. - air);
        h.air_nitrogen *= air;
        h.rock_nitrogen *= air;
        h.air_carbon *= air;
        let start = (h.air_column(), h.burned_up(), h.hard_rays());
        let nitrogen = h.initial_nitrogen;
        core.step(1600);
        let e = core.autonomous.as_ref().unwrap();
        let h = e.physical.as_ref().unwrap();
        assert!(e.valid(), "books {:?}", h.errors(&e.cells));
        assert!(rock > 0. && gone >= 0.);
        (start, h.impacts, h.airbursts, h.initial_nitrogen - nitrogen, (h.air_column(), h.burned_up(), h.hard_rays()))
    };
    let (thick, thin) = (run(1.), run(0.05));
    eprintln!("as it is {thick:?}; nearly airless {thin:?}");
    let (start, impacts, airbursts, gained, end) = thick;
    assert!(impacts + airbursts >= 4, "bodies arrive: {impacts} landed, {airbursts} burned up");
    assert_eq!(impacts + airbursts, thin.1 + thin.2, "the same bodies arrive whatever the air");
    assert!(impacts <= thin.1, "and thicker air never lets more of them land: {impacts} against {}", thin.1);
    assert!(gained > 0., "what falls is the planet's from then on: {gained}");
    // A newborn planet's air is thin. It thickens as the rock and the sea
    // give up their gas, and stops more as it does.
    assert!(end.0 > start.0 * 2., "the air thickens: {} to {}", start.0, end.0);
    assert!(start.1 < 0.05 && end.1 > 0.5, "share of falling rock burned up: {} then {}", start.1, end.1);
    assert!(start.2 > 0.1 && end.2 < 0.01, "hard rays reaching the ground: {} then {}", start.2, end.2);
}

#[test]
fn nothing_falls_from_an_empty_sky() {
    let mut core = Core::new(Config {
        world_size: 64,
        world_shape: crate::domain::Shape::Sphere,
        initial_cells: 16,
        ..Config::default()
    })
    .unwrap();
    core.start_physical();
    core.steer(r#"{"kind":"bombardment","scale":0}"#).unwrap();
    core.step(1600);
    let h = core.autonomous.as_ref().unwrap().physical.as_ref().unwrap();
    assert_eq!((h.impacts, h.airbursts), (0, 0));
    assert!(core.steer(r#"{"kind":"bombardment","scale":9}"#).is_err());
}

#[test]
fn ozone_from_free_oxygen_and_the_water_over_a_cell_keep_ultraviolet_off_it() {
    // One bare cell under a bright star: at the surface or two units down,
    // with no free oxygen in the air or with all of it free.
    let harmed = |oxygen: f64, depth: f32| {
        let (mut core, mut e, mut h) = fixture();
        e.cells.truncate(1);
        h.oxygenate(oxygen);
        h.steering.light = 6.;
        rebase(&mut h, &e);
        let mut dose = 0f32;
        e.cells[0].state.depth = depth;
        for _ in 0..192 {
            e.cells[0].energy = 8.;
            h.tick(&mut e, &mut core);
            if e.cells.is_empty() {
                return (1., dose);
            }
            // Held where it was put.
            e.cells[0].state.depth = depth;
            dose = e.cells[0].state.ultraviolet;
        }
        (e.cells[0].state.injury, dose)
    };
    let (bare, full) = harmed(0., 0.);
    let (screened, little) = harmed(1., 0.);
    let (deep, less) = harmed(0., 2.);
    eprintln!("injury and peak ultraviolet: no ozone at the surface {bare} {full}; under ozone {screened} {little}; no ozone, two units down {deep} {less}");
    assert!(bare > 0.5, "with no ozone the surface is deadly: {bare} from {full}");
    assert!(screened < 0.02, "ozone takes nearly all of it: {screened} from {little}");
    assert!(deep < bare * 0.2, "and so does water: {deep} from {less}");
}

#[test]
#[ignore = "the one-way fin grip this rested on is gone (docs/ANIMAL_BODIES_PLAN.md); to be rewritten with the rebuilt bodies"]
fn muscle_and_bristles_carry_a_body_a_good_way_for_less_than_cilia_cost() {
    // A row of twelve cells, driven for 40 cycles by a coat of cilia or by
    // muscle, nerve and bristles.
    let (mut core, mut e, mut h) = row(12, vec![always(PROPEL)], &[PROPEL]);
    let swum = travel(&mut core, &mut e, &mut h, 160);
    let spent: f64 = e.cells.iter().map(|c| c.state.motor_work).sum();
    let materials = [4, crate::physiology::CHANNEL, FIN];
    let genes = vec![always(4), always(crate::physiology::CHANNEL), always(FIN)];
    let (mut core, mut e, mut h) = row(12, genes, &materials);
    let (mut beats, mut was, mut rowed) = (0, 0., 0.);
    for _ in 0..160 {
        // Enough of signal A in the water to keep the nerves firing.
        for s in &mut h.solutes {
            s[3] = 1.;
        }
        rowed += travel(&mut core, &mut e, &mut h, 1);
        let now = e.bonds[0].activation;
        beats += usize::from(was < 0.4 && now >= 0.4);
        was = now;
    }
    let worked: f64 = e.cells.iter().map(|c| c.state.motor_work).sum();
    eprintln!("twelve cells over 40 cycles: {swum} by cilia for {spent} energy, {rowed} by {beats} beats of muscle for {worked}");
    assert!(beats >= 20, "a nerve fires most of once a cycle: {beats}");
    assert!(rowed > 4., "a tenth of a unit a cycle or more: {rowed}");
    assert!(worked < spent, "for less than the cilia cost: {worked} against {spent}");
    assert!(swum > rowed, "though at this size cilia are still the faster: {swum}");
}

#[test]
fn a_wing_holds_weight_up_near_the_ground_and_not_far_above_it() {
    // A cell kept moving fast through the air, set down a unit and a half
    // above the ground or twelve, with a wing or without: how far it has
    // dropped after two cycles, and the highest it was over the ground in twenty.
    let flown = |wings: bool, height: f32| {
        let (mut core, mut e, mut h) = fixture();
        e.cells.truncate(1);
        if wings {
            build(&mut e, 0, crate::physical::WING);
            e.cells[0].genome.genes.push(always(crate::physical::WING));
        }
        let c = &mut e.cells[0];
        let ground = h.domain.ground(c.p.x, c.p.z, core.config.seed)
            + h.planet.as_ref().map_or(0., |p| p.relief_at(c.p.x, c.p.z));
        c.p.y = ground + height;
        rebase(&mut h, &e);
        let start = e.cells[0].p.y;
        let (mut dropped, mut highest) = (0., 0f32);
        for tick in 0..80 {
            let c = &mut e.cells[0];
            c.energy = 8.;
            // Flung forward along its heading, whatever else has happened to it.
            let ahead = h.domain.tangent(c.p, c.heading);
            c.v = ahead.mul(3.).add(V3::new(0., c.v.y, 0.));
            h.tick(&mut e, &mut core);
            if tick == 7 {
                dropped = start - e.cells[0].p.y;
            }
            highest = highest.max(e.cells[0].state.height);
        }
        (dropped, highest)
    };
    let (held, highest) = flown(true, 1.5);
    let (fell, _) = flown(false, 1.5);
    let (thin, _) = flown(true, 12.);
    eprintln!("moving fast for two cycles: with a wing it dropped {held}, without one {fell}, and with a wing twelve units up {thin}; in twenty cycles the wing never had it higher than {highest} over the ground");
    assert!(fell > 0.05, "with no wing it falls: {fell}");
    assert!(held < fell * 0.75, "a wing holds its weight and more: {held} against {fell}");
    assert!(thin > 0.3, "far above the ground the air is too thin to: {thin}");
    // A wing turns speed into height and adds nothing of its own: the
    // cell is never higher than its speed could throw it, which at three
    // units a cycle is fifteen units above where it began.
    assert!(highest < 1.5 + 9. / (2. * 0.3) + 1., "so a wing does not carry it away: {highest}");
}

#[test]
fn a_line_of_cells_meets_less_water_end_on_than_broadside() {
    // The same line of nine, shoved along itself and shoved sideways. How
    // much of its speed each keeps after one moment.
    let kept = |along: bool| {
        let (mut core, mut e, mut h) = row(9, vec![always(0)], &[]);
        for c in &mut e.cells {
            c.v = if along { V3::new(1., 0., 0.) } else { V3::new(0., 0., 1.) };
        }
        h.tick(&mut e, &mut core);
        let v = e.cells.iter().fold(V3::default(), |s, c| s.add(c.v)).mul(1. / 9.);
        if along { v.x } else { v.z }
    };
    let (ahead, across) = (kept(true), kept(false));
    // The water's hold is as the loss of speed, taken as a rate.
    let ratio = ahead.ln() / across.ln();
    eprintln!("a line of nine after one moment: {ahead} of its speed end-on, {across} broadside; hold {ratio} as strong end-on");
    assert!(ahead > across && (0.4..0.75).contains(&ratio), "end-on it meets about half the water: {ratio}");
}

/// A backbone as a tadpole has one: a line of `n` bones, each with muscle
/// and a fin, and a nerve cell riding on each side of every bone. The nerve
/// cells toward the head are the quicker, each holds down the one across
/// from it, and each bone's joint is pulled from its two sides. With
/// `together` the nerve cells along each side are all alike, so each side
/// fires as one and the body bends as a whole, one way and then the other.
/// Without `fins` the bones are bare.
/// Returns the world and, for each tick's upkeep, the nerve cells' levels.
fn backbone(n: usize, together: bool, fins: bool) -> (Core, Evolution, Habitat, Vec<(usize, f64)>) {
    use crate::development::Synapse;
    let (core, mut e, mut h) = row(3 * n, vec![always(0)], &[]);
    // These fixtures' sea is a film. Sink its bed, and set the body down
    // half-way between the top and the bed over the deepest stretch, so
    // that neither is part of how it swims.
    for r in &mut h.planet.as_mut().unwrap().relief {
        *r -= 4.;
    }
    let bed = |x: f32, z: f32| (-8..24).map(|k| h.domain.ground(x + k as f32 * 0.5, z, core.config.seed) - 4.).fold(f32::MIN, f32::max);
    let (mut first, mut deepest) = (e.cells[0].p, f32::MAX);
    for x in -16..10 {
        for z in -16..17 {
            let high = bed(x as f32, z as f32);
            if high < deepest {
                deepest = high;
                first = V3::new(x as f32, -1., z as f32);
            }
        }
    }
    assert!(deepest < -3., "there is water deep enough to swim in: the bed is at {deepest}");
    let ids: Vec<u32> = e.cells.iter().map(|c| c.id).collect();
    let mut channels = Vec::new();
    for (i, c) in e.cells.iter_mut().enumerate() {
        let (along, side) = (i % n, i / n);
        // The nerve cells ride on top of the bones, a little to each side,
        // as a nerve cord lies on the rod beneath it.
        c.p = V3::new(first.x + along as f32 * 0.44, first.y + if side == 0 { 0. } else { 0.3 }, first.z + [0., 0.2, -0.2][side]);
        let m = c.material.as_mut().unwrap();
        if side == 0 {
            // Half bone: stiff enough for its joints to follow their muscle.
            // It sinks as it swims, as bone does, and has the depth to.
            m.mineral = m.mass();
            m.proteins[4] = 0.05;
            c.genome.id = 150;
            c.genome.genes.push(always(4));
            if fins {
                m.proteins[FIN] = 0.05;
                c.genome.genes.push(always(FIN));
            }
        } else {
            // A quarter of the channel a nerve usually has, for a beat the
            // body can follow, and a little less of it from head to tail.
            let fall = if together { 0. } else { 0.06 };
            let level = 0.25 * (1. - fall * along as f64 / (n - 1) as f64);
            channels.push((i, level * crate::physiology::CHANNEL_SET * m.biomass));
            m.proteins[8] = 0.05;
            m.proteins[crate::development::SYNAPSE] = 0.05;
            c.genome.id = 160;
            // No gene for the channel itself: its level is set by hand each
            // moment, where a grown animal would set it by the cell's place.
            c.genome.genes.extend([always(8), always(crate::development::SYNAPSE)]);
        }
    }
    e.bonds.clear();
    let mut tie = |a: usize, b: usize, rest: f32| {
        e.bonds.push(Bond { a: ids[a].min(ids[b]), b: ids[a].max(ids[b]), rest, integrity: 1., activation: 0., stiffness: 0. });
    };
    for i in 0..n {
        tie(i, n + i, 0.36);
        tie(i, 2 * n + i, 0.36);
        tie(n + i, 2 * n + i, 0.4);
        if i + 1 < n {
            tie(i, i + 1, 0.44);
            tie(n + i, n + i + 1, 0.44);
            tie(2 * n + i, 2 * n + i + 1, 0.44);
        }
    }
    // The junctions between the nerve cells: the one across is held down,
    // the ones along the line are left to the ties between them.
    for i in 0..n {
        // (A quarter of the hardest hold there is: a hard hold makes each
        // side wait too long for the other.)
        for (a, b, weight) in [(n + i, 2 * n + i, -0.25), (2 * n + i, n + i, -0.25)] {
            h.development.synapses.push(Synapse { from: ids[a], to: ids[b], weight, pre: 0., post: 0. });
        }
        if i + 1 < n {
            for side in [n, 2 * n] {
                for (a, b) in [(side + i, side + i + 1), (side + i + 1, side + i)] {
                    h.development.synapses.push(Synapse { from: ids[a], to: ids[b], weight: 0., pre: 0., post: 0. });
                }
            }
        }
    }
    // One side starts first and the other has just fired: two sides begun
    // alike stay alike, firing together and pulling against each other.
    for i in 0..n {
        e.cells[n + i].state.excitation = 0.6;
        e.cells[2 * n + i].state.recovery = 0.5;
    }
    rebase(&mut h, &e);
    (core, e, h, channels)
}

#[test]
#[ignore = "the paired-nerve wiring swims, but how far swings from a tenth of a body length to seven tenths with the strength of the hold across and with chance; a probe until that is understood. The single line of bones below is the swimmer that is held to"]
fn a_backbone_swims_head_first_by_a_wave_and_its_fins_are_what_carry_it() {
    let n = 9;
    let swum = |together: bool, fins: bool| {
        let (mut core, mut e, mut h, channels) = backbone(n, together, fins);
        let length = (n - 1) as f32 * 0.44;
        let (mut gone, mut beats, mut was, mut widest) = (0., 0, 0f32, 0f32);
        let (mut left, mut right) = (Vec::new(), Vec::new());
        let (mut torn, mut seen, mut fired) = (0, [0f32; 3], [Vec::new(), Vec::new(), Vec::new()]);
        let mut path: Vec<String> = Vec::new();
        for tick in 0..300 {
            for s in &mut h.solutes {
                s[3] = 1.;
            }
            for &(i, level) in &channels {
                e.cells[i].material.as_mut().unwrap().proteins[crate::physiology::CHANNEL] = level;
            }
            let tied = e.bonds.len();
            let before = e.cells.iter().map(|c| c.p.x).sum::<f32>() / e.cells.len() as f32;
            for c in &mut e.cells {
                c.energy = 8.;
                c.state.injury = 0.;
            }
            let kept = e.bonds.clone();
            h.tick(&mut e, &mut core);
            torn += tied - e.bonds.len().min(tied);
            for old in kept {
                match e.bonds.iter_mut().find(|b| b.a == old.a && b.b == old.b) {
                    Some(b) => b.integrity = 1.,
                    None => e.bonds.push(Bond { integrity: 1., ..old }),
                }
            }
            gone += e.cells.iter().map(|c| c.p.x).sum::<f32>() / e.cells.len() as f32 - before;
            for (k, &i) in [n, n + n / 2, 2 * n - 1].iter().enumerate() {
                let now = e.cells[i].state.excitation;
                if seen[k] < 0.65 && now >= 0.65 && fired[k].len() < 8 {
                    fired[k].push(tick);
                }
                seen[k] = now;
            }
            if !together && tick < 0 {
                let c = |i: usize| (e.cells[i].state.excitation, e.cells[i].state.neural_input, e.cells[i].activity(crate::physiology::CHANNEL));
                let ((lu, li, lg), (ru, ri, _)) = (c(n), c(2 * n));
                eprintln!("firing {tick:3}: left {lu:.2} (junctions {li:+.3}, channel {lg:.2})  right {ru:.2} ({ri:+.3})");
            }
            if tick % 60 == 59 {
                let mid = e.cells[..n].iter().fold(V3::default(), |s, c| s.add(c.p)).mul(1. / n as f32);
                let nose = e.cells[0].p.sub(e.cells[n - 1].p);
                let speed = e.cells[..n].iter().fold(V3::default(), |s, c| s.add(c.v)).mul(1. / n as f32);
                let water = h.fluid.velocity(h.domain.locate(mid));
                path.push(format!("({:.1},{:.1}) nose ({:.1},{:.1}) water ({:.2},{:.2}) speed ({:.2},{:.2})", mid.x, mid.z, nose.x, nose.z, water[0], water[1], speed.x, speed.z));
            }
            let (l, r) = (e.cells[n].state.excitation, e.cells[2 * n].state.excitation);
            if was < 0.65 && l >= 0.65 {
                beats += 1;
                left.push(tick);
            }
            if r >= 0.65 && right.last().map_or(true, |&t| tick > t + 3) {
                right.push(tick);
            }
            was = l;
            for j in &h.joints {
                widest = widest.max(j.bend.abs());
            }
        }
        eprintln!(
            "backbone of {n}, nerves {}, {}: went {gone} along its line ({} lengths), {beats} beats, joints {} of which {} a backbone's, widest bend {widest}, bonds {}; left head fired at {:?}, right at {:?}",
            if together { "each side as one" } else { "in a wave" },
            if fins { "fins" } else { "no fins" },
            gone / length,
            h.joints.len(),
            h.joints.iter().filter(|j| j.straight).count(),
            e.bonds.len(),
            &left[..left.len().min(6)],
            &right[..right.len().min(6)],
        );
        eprintln!("   every 60 ticks, where its middle is and where its nose points: {}", path.join(" | "));
        eprintln!(
            "   height now {:.2} to {:.2}, on the ground {} of {}, under water {:.2}, drift across {:.2}",
            e.cells.iter().map(|c| c.p.y).fold(f32::MAX, f32::min),
            e.cells.iter().map(|c| c.p.y).fold(f32::MIN, f32::max),
            e.cells.iter().filter(|c| c.state.ground).count(),
            e.cells.len(),
            e.cells.iter().map(|c| c.state.under).sum::<f32>() / e.cells.len() as f32,
            e.cells.iter().map(|c| c.p.z).sum::<f32>() / e.cells.len() as f32,
        );
        eprintln!(
            "   junctions across the head pair: {:?}",
            h.development.synapses.iter().filter(|s| (s.from == e.cells[n].id && s.to == e.cells[2 * n].id) || (s.to == e.cells[n].id && s.from == e.cells[2 * n].id)).map(|s| s.weight).collect::<Vec<_>>(),
        );
        eprintln!(
            "   ties torn and mended {torn}; left line fired at head {:?}, middle {:?}, tail {:?}; bends now {:?}",
            fired[0], fired[1], fired[2],
            h.joints.iter().map(|j| (j.straight, (j.bend * 100.).round() / 100.)).collect::<Vec<_>>(),
        );
        (gone, beats)
    };
    // Each side firing as one still makes headway, since the tail of a
    // springy body trails its head whatever its nerves do. It is printed, not
    // asserted on.
    swum(true, true);
    let (bare, _) = swum(false, false);
    let (went, beats) = swum(false, true);
    // The head is at the low end of the line, so head first is backward along it.
    assert!(beats >= 4, "the nerves keep a beat: {beats}");
    assert!(-went > 0.5, "the wave carries it head first: {went}");
    assert!(-went > 2. * bare.abs(), "and it is the fins that the water pushes on: {went} with them, {bare} without");
}

#[test]
fn a_single_line_of_bones_swims_by_its_own_nerves_and_nowhere_when_they_fire_in_step() {
    // The simplest swimmer: nine bones in a line, each with muscle, a fin
    // and a nerve channel of its own, a little less of it toward the tail.
    let n = 9;
    let swum = |fall: f64| {
        let (mut core, mut e, mut h) = row(n, vec![always(0)], &[]);
        for r in &mut h.planet.as_mut().unwrap().relief {
            *r -= 4.;
        }
        let mut channels = Vec::new();
        for (i, c) in e.cells.iter_mut().enumerate() {
            c.p = V3::new(8. + i as f32 * 0.44, -1., -16.);
            let m = c.material.as_mut().unwrap();
            // A quarter of the channel a nerve usually has: a beat of about
            // twenty ticks, which a body can follow.
            let level = 0.25 * (1. - fall * i as f64 / (n - 1) as f64);
            channels.push(level * crate::physiology::CHANNEL_SET * m.biomass);
            m.mineral = m.mass();
            for k in [4, 8, FIN] {
                m.proteins[k] = 0.05;
            }
            c.genome.id = 150;
            c.genome.genes.extend([always(4), always(8), always(FIN)]);
        }
        rebase(&mut h, &e);
        let length = (n - 1) as f32 * 0.44;
        let (mut gone, mut beats, mut was, mut work) = (0., 0, 0f32, 0.);
        for _ in 0..300 {
            for s in &mut h.solutes {
                s[3] = 1.;
            }
            for (c, &level) in e.cells.iter_mut().zip(&channels) {
                c.material.as_mut().unwrap().proteins[crate::physiology::CHANNEL] = level;
            }
            gone += travel(&mut core, &mut e, &mut h, 1);
            let now = e.cells[0].state.excitation;
            beats += usize::from(was < 0.65 && now >= 0.65);
            was = now;
        }
        for j in &h.joints {
            work += j.work;
        }
        eprintln!(
            "single line of {n}, channels falling {fall}: went {gone} along its line ({} lengths) in 75 cycles, {beats} beats, {} of {} joints a backbone's, joint work {work:.3}, height {:.2}",
            gone / length,
            h.joints.iter().filter(|j| j.straight).count(),
            h.joints.len(),
            e.cells[0].p.y,
        );
        (gone, beats)
    };
    let (still, _) = swum(0.);
    let (went, beats) = swum(0.1);
    // The head is at the low end of the line, so head first is backward along it.
    assert!(beats >= 4, "the nerves keep a beat: {beats}");
    assert!(-went > 1., "the wave carries it head first: {went}");
    assert!(still.abs() < 0.25 * -went, "all in step it goes nowhere: {still} against {went}");
}
