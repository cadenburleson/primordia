//! Deliberately supplied mechanism fixtures, never ordinary-founder outcomes.
use crate::{
    autonomous::{Bond, Evolution, Gene, ACTIONS, INPUTS},
    body::V3,
    development::*,
    physical::Habitat,
    *,
};

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
        c.energy = 8.;
        c.cooldown = 0.;
    }
    rebase(&mut h, &e);
    (core, e, h)
}
fn rebase(h: &mut Habitat, e: &Evolution) {
    h.rebase(&e.cells);
    h.initial_energy += h.heat + h.mechanical_input - h.photon_energy;
}
fn network_fixture() -> (Core, Evolution, Habitat) {
    let (core, mut e, mut h) = fixture();
    for c in &mut e.cells {
        for k in [PHOTIC, SYNAPSE, physiology::CHANNEL, 4] {
            c.material.as_mut().unwrap().proteins[k] = 0.05;
        }
        let mut promoter = [0.; INPUTS];
        promoter[8] = 0.5;
        c.genome.genes.push(Gene {
            action: SYNAPSE,
            promoter,
            bias: 1.,
            strength: 1.,
            fates: 15,
        });
    }
    e.bonds.push(Bond {
        a: 1,
        b: 2,
        rest: 0.4,
        integrity: 1.,
        activation: 0.,
        stiffness: 0.,
    });
    e.cells[0].state.excitation = 0.8;
    e.cells[1].state.excitation = 0.1;
    rebase(&mut h, &e);
    (core, e, h)
}
fn gamete_fixture() -> (Core, Evolution, Habitat) {
    let (core, mut e, mut h) = fixture();
    for (i, c) in e.cells.iter_mut().enumerate() {
        c.id = i as u32 + 3;
        c.parent = i as u32 + 1;
        c.lifecycle.root = c.parent;
        c.lifecycle.phase = Phase::Gamete;
        c.material.as_mut().unwrap().proteins[FUSOGEN] = 0.08;
    }
    e.next_cell = 5;
    rebase(&mut h, &e);
    (core, e, h)
}
#[test]
fn ordinary_founders_have_no_supplied_adult_or_sexual_program() {
    let (_, e, h) = fixture();
    assert!(e.cells.iter().all(|c| c
        .genome
        .genes
        .iter()
        .all(|g| [0, 1, 2, 7, crate::physical::RESPIRE].contains(&g.action))
        && c.material.as_ref().unwrap().proteins[16..crate::physical::RESPIRE]
            .iter()
            .all(|v| *v == 0.)
        && c.lifecycle.homolog.is_none()));
    assert_eq!(h.development.zygotes, 0);
    assert!(h.development.synapses.is_empty());
}
#[test]
fn rare_random_rounding_cannot_select_a_gene_or_crossover_past_the_end() {
    assert_eq!((u32::MAX as f64 / 4294967296.) as f32, 1.);
    let draw = unit_fraction(u32::MAX);
    assert!(draw < 1. && draw >= 0.);
    for size in [INPUTS, ACTIONS, autonomous::MAX_GENES] {
        assert!(((draw * size as f32) as usize) < size);
        assert!(((draw * (size + 1) as f32) as usize) <= size);
    }
    assert_eq!(unit_fraction(0), 0.);
}
#[test]
fn directional_light_is_local_material_and_energy_gated() {
    let (_, mut e, mut h) = network_fixture();
    h.photoreception(&mut e);
    let forward = e.cells[0].state.light;
    assert!(forward > 0. && h.development.sensory_work > 0.);
    h.planet.as_mut().unwrap().sun_direction = [0., -1., 0.];
    h.photoreception(&mut e);
    assert_eq!(e.cells[0].state.light, 0.);
    h.planet.as_mut().unwrap().sun_direction = [0., 1., 0.];
    e.cells[0].energy = 0.;
    e.cells[1].material.as_mut().unwrap().proteins[PHOTIC] = 0.;
    rebase(&mut h, &e);
    h.photoreception(&mut e);
    assert!(e.cells.iter().all(|c| c.state.light == 0.));
    assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
}
#[test]
fn contact_signals_adapt_and_drive_excitation_using_paid_energy() {
    let (_, mut e, mut h) = network_fixture();
    h.neural_contacts(&mut e);
    let original = h.development.synapses.clone();
    assert_eq!(original.len(), 2);
    assert!(e.cells[1].state.neural_input > 0.);
    let mut control = e.clone();
    let mut hc = h.clone();
    for c in &mut control.cells {
        c.state.neural_input = 0.;
    }
    // A nerve fires and is quiet again within a few moments, so what
    // matters is the most it was excited, not how it was left.
    let (mut fired, mut quiet, mut pulled, mut slack) = (0f32, 0f32, 0f32, 0f32);
    for _ in 0..12 {
        h.neural_contacts(&mut e);
        h.excitable_signals(&mut e);
        hc.excitable_signals(&mut control);
        fired = fired.max(e.cells[1].state.excitation);
        quiet = quiet.max(control.cells[1].state.excitation);
        pulled = pulled.max(e.cells[1].motor_signals()[0]);
        slack = slack.max(control.cells[1].motor_signals()[0]);
    }
    assert!(
        h.development.adaptations > 0
            && h.development.transmissions > 0
            && h.development.neural_work > 0.
    );
    assert_ne!(h.development.synapses[0].weight, original[0].weight);
    assert!(fired > quiet, "excited {fired} against {quiet}");
    assert!(pulled > slack, "muscle drive {pulled} against {slack}");
    assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
    e.bonds.clear();
    h.prune_synapses(&e);
    assert!(h.development.synapses.is_empty());
}
#[test]
fn channels_without_adaptive_material_and_unpowered_material_do_not_signal() {
    for no_material in [true, false] {
        let (_, mut e, mut h) = network_fixture();
        for c in &mut e.cells {
            if no_material {
                c.material.as_mut().unwrap().proteins[SYNAPSE] = 0.;
            } else {
                c.energy = 0.;
            }
        }
        h.neural_contacts(&mut e);
        assert!(e.cells.iter().all(|c| c.state.neural_input == 0.));
        assert_eq!(h.development.adaptations, 0);
        assert_eq!(h.development.neural_work, 0.);
    }
}
#[test]
fn gametic_budding_splits_existing_stores_and_gametes_cannot_fission() {
    let (mut core, mut e, mut h) = fixture();
    e.cells.truncate(1);
    let c = &mut e.cells[0];
    c.material.as_mut().unwrap().biomass = 0.45;
    for k in [7, GAMETIC, FUSOGEN] {
        c.material.as_mut().unwrap().proteins[k] = 0.08;
    }
    // Its gene for gametes is at work now, not only its leftover protein.
    c.expression[GAMETIC] = 1.;
    c.lifecycle.homolog = Some(c.genome.clone());
    rebase(&mut h, &e);
    for _ in 0..300 {
        h.divide(&mut e, &mut core);
        if h.development.gametes > 0 {
            break;
        }
    }
    assert_eq!(h.development.gametes, 1);
    assert_eq!(h.development.meioses, 1);
    let gamete = e
        .cells
        .iter()
        .find(|c| c.lifecycle.phase == Phase::Gamete)
        .unwrap();
    assert!(gamete.lifecycle.homolog.is_none());
    assert!(!e.bonds.iter().any(|b| b.a == gamete.id || b.b == gamete.id));
    assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
    e.cells.retain(|c| c.lifecycle.phase == Phase::Gamete);
    e.cells[0].material.as_mut().unwrap().biomass = 0.45;
    e.cells[0].energy = 8.;
    e.cells[0].cooldown = 0.;
    for _ in 0..300 {
        h.divide(&mut e, &mut core);
    }
    assert_eq!(e.cells.len(), 1);
}
#[test]
fn a_gamete_shed_from_a_body_keeps_that_bodys_origin() {
    let (mut core, mut e, mut h) = fixture();
    e.cells.truncate(1);
    let c = &mut e.cells[0];
    c.material.as_mut().unwrap().biomass = 0.45;
    for k in [7, GAMETIC, FUSOGEN] {
        c.material.as_mut().unwrap().proteins[k] = 0.08;
    }
    c.expression[GAMETIC] = 1.;
    // A part of a body: it was tied to others when it last looked.
    c.state.tied = true;
    c.lifecycle.root = c.id;
    let origin = c.id;
    rebase(&mut h, &e);
    for _ in 0..300 {
        h.divide(&mut e, &mut core);
        if h.development.gametes > 0 {
            break;
        }
    }
    assert_eq!(h.development.gametes, 1);
    // Loose in the water, the gamete is read again. It must still count as
    // its body's, or its siblings could fertilise it.
    h.physiology(&mut e, &core);
    let gamete = e
        .cells
        .iter()
        .find(|c| c.lifecycle.phase == Phase::Gamete)
        .unwrap();
    assert_eq!(gamete.body_origin(), origin);
}
#[test]
fn fertilization_inherits_two_chromosomes_conserves_matter_and_resets_memory() {
    let (mut core, mut e, mut h) = gamete_fixture();
    e.cells[1].genome.genes.push(Gene {
        action: 4,
        promoter: [0.; INPUTS],
        bias: 1.,
        strength: 1.,
        fates: 15,
    });
    e.cells[1].genome.id = 2;
    e.next_genome = 3;
    e.cells[0].state.excitation = 0.9;
    e.cells[0].state.nutrient_memory = 0.6;
    let organic = h.totals(&e.cells).0;
    for _ in 0..300 {
        h.fertilize(&mut e, &mut core, &[(0, 1)]);
        if h.development.zygotes > 0 {
            break;
        }
    }
    assert_eq!(e.cells.len(), 1);
    let z = &e.cells[0];
    assert_eq!(z.lifecycle.phase, Phase::Zygote);
    assert_eq!(z.parent, 1);
    assert_eq!(z.lifecycle.mate, 2);
    assert!(z
        .lifecycle
        .homolog
        .as_ref()
        .unwrap()
        .genes
        .iter()
        .any(|g| g.action == 4));
    assert_eq!(z.state.excitation, 0.);
    assert_eq!(z.state.nutrient_memory, 0.);
    assert_eq!(z.lifecycle.depth, 0);
    assert!((h.totals(&e.cells).0 - organic).abs() < 1e-10);
    assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
    assert_eq!(h.development.ancestry[0].mate, 2);
    assert!(z.lifecycle.valid(z));
}
#[test]
fn fertilization_requires_contact_compatibility_different_origin_fusogen_and_atp() {
    for condition in 0..6 {
        let (mut core, mut e, mut h) = gamete_fixture();
        match condition {
            0 => e.cells[1].p.x = 10.,
            1 => e.cells[1].lifecycle.root = e.cells[0].lifecycle.root,
            2 => e.cells[0].material.as_mut().unwrap().proteins[FUSOGEN] = 0.,
            3 => e.cells[0].energy = 0.,
            4 => e.cells[1].lifecycle.phase = Phase::Somatic,
            _ => {
                e.cells[1].genome.genes = vec![Gene {
                    action: 10,
                    promoter: [0.; INPUTS],
                    bias: 1.,
                    strength: 1.,
                    fates: 15,
                }];
            }
        }
        for _ in 0..300 {
            h.fertilize(&mut e, &mut core, &[(0, 1)]);
        }
        assert_eq!(h.development.zygotes, 0, "condition {condition}");
        assert_eq!(e.cells.len(), 2);
    }
}
#[test]
fn diploid_expression_and_crossover_preserve_inherited_programs() {
    let (mut core, mut e, _) = fixture();
    let c = &mut e.cells[0];
    let mut other = c.genome.clone();
    other.genes[0].action = 4;
    other.id = 2;
    c.lifecycle.homolog = Some(other);
    let input = [0.5; INPUTS];
    let out = c.cellular_expression(input);
    assert!(out[0] > 0. && out[4] > 0.);
    let mut combinations = BTreeSet::new();
    for _ in 0..100 {
        let genes = c.gamete_chromosome(&mut core).genes;
        assert_eq!(genes.len(), 5);
        combinations.insert(genes[0].action);
    }
    assert_eq!(combinations, BTreeSet::from([0, 4]));
}
#[test]
fn zygote_growth_copies_both_chromosomes_and_keeps_the_reproductive_origin() {
    let (mut core, mut e, mut h) = gamete_fixture();
    for _ in 0..300 {
        h.fertilize(&mut e, &mut core, &[(0, 1)]);
        if h.development.zygotes > 0 {
            break;
        }
    }
    let origin = e.cells[0].id;
    e.cells[0].energy = 8.;
    e.cells[0].cooldown = 0.;
    e.cells[0].material.as_mut().unwrap().proteins[7] = 0.08;
    // Its division genes are switched on.
    e.cells[0].expression[7] = 1.;
    rebase(&mut h, &e);
    for _ in 0..300 {
        h.divide(&mut e, &mut core);
        if e.cells.len() == 2 {
            break;
        }
    }
    assert_eq!(e.cells.len(), 2);
    assert!(e.cells.iter().all(|c| c.lifecycle.homolog.is_some()
        && c.lifecycle.root == origin
        && c.lifecycle.phase == Phase::Somatic
        && c.lifecycle.valid(c)));
    assert_eq!(e.cells[1].parent, origin);
    assert_eq!(e.cells[1].lifecycle.depth, 1);
    assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
    e.physical = Some(h);
    assert!(e.valid());
}
#[test]
fn produced_polar_scaffolds_change_division_geometry_without_an_adult_template() {
    let (mut core, mut e, mut h) = fixture();
    e.cells[0].heading = 0.;
    e.cells[0].material.as_mut().unwrap().biomass = 0.45;
    e.cells[0].material.as_mut().unwrap().proteins[7] = 0.08;
    e.cells[0].material.as_mut().unwrap().proteins[POLARITY] = 0.2;
    rebase(&mut h, &e);
    for _ in 0..300 {
        h.divide(&mut e, &mut core);
        if e.cells.len() == 3 {
            break;
        }
    }
    assert_eq!(e.cells.len(), 3);
    let delta = e.cells[2].p.sub(e.cells[0].p);
    assert!(delta.x > 0. && delta.z.abs() + delta.y.abs() < delta.x.abs() * 0.2);
    assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
}
#[test]
fn developmental_and_chemical_inputs_allow_local_differentiation_without_templates() {
    let (_, mut e, h) = fixture();
    e.cells[0].lifecycle.depth = 16;
    e.cells[0].chemicals = [0.8, 0.2];
    let input = e.cells[0].physical_inputs(&h.solutes, 0, h.domain);
    assert_eq!(input[18], 0.5);
    assert!((input[19] - 0.6).abs() < 1e-6);
    let mut g = e.cells[0].genome.clone();
    let mut promoter = [0.; INPUTS];
    promoter[18] = 1.;
    g.genes.push(Gene {
        action: 4,
        promoter,
        bias: -0.2,
        strength: 1.,
        fates: 15,
    });
    assert!(g.express(input)[4] > 0.);
    e.cells[1].lifecycle.depth = 0;
    assert_eq!(
        g.express(e.cells[1].physical_inputs(&h.solutes, 0, h.domain))[4],
        0.
    );
}
#[test]
fn new_states_round_trip_exactly_and_invalid_network_or_homolog_is_rejected() {
    let (mut core, mut e, mut h) = network_fixture();
    h.neural_contacts(&mut e);
    e.physical = Some(h);
    core.autonomous = Some(e);
    let save = serde_json::to_string(&core).unwrap();
    let mut copy: Core = serde_json::from_str(&save).unwrap();
    assert!(copy.autonomous.as_ref().unwrap().valid());
    assert_eq!(save, serde_json::to_string(&copy).unwrap());
    core.step(4);
    copy.step(4);
    assert_eq!(
        serde_json::to_string(&core).unwrap(),
        serde_json::to_string(&copy).unwrap()
    );
    let e = copy.autonomous.as_mut().unwrap();
    e.physical.as_mut().unwrap().development.synapses[0].weight = 2.;
    assert!(!e.valid());
    e.physical.as_mut().unwrap().development.synapses[0].weight = 0.;
    e.cells[0].lifecycle.homolog = Some(e.cells[0].genome.clone());
    e.cells[0].lifecycle.homolog.as_mut().unwrap().genes[0].action = ACTIONS;
    assert!(!e.valid());
}
