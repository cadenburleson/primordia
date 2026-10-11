//! Starter species: founders designed from the ordinary gene set and released
//! into a world like any other introduced organism. Nothing here is a special
//! case in the physics. Each starts as one cell with a founder's stores and
//! has to build, feed and breed with the genes it was given, and those genes
//! mutate from the first division on.
use crate::physical::{CHEMO, ELONGATE, HOLDFAST, SHELL, TOXIN};
use crate::{
    autonomous::{Cell, Gene, Genome, ACTIONS, INPUTS},
    body::V3,
    development::{FUSOGEN, GAMETIC, POLARITY},
    physical::{Inventory, ENGULF, FIN, PROPEL, SIZE, SPINE, STEER, WALL},
    physiology::{BARRIER, CHANNEL},
    specimens::Specimen,
    *,
};

/// Key and display name of every starter species.
pub const SPECIES: [(&str, &str); 26] = [
    ("drifter", "Drifter"),
    ("alga", "Alga"),
    ("swimmer", "Swimmer"),
    ("grazer", "Grazer"),
    ("hunter", "Hunter"),
    ("thornback", "Thornback"),
    ("rower", "Rower"),
    ("courter", "Courter"),
    ("decomposer", "Decomposer"),
    ("plankton", "Plankton"),
    ("leviathan", "Leviathan"),
    ("volvox", "Volvox"),
    ("ulothrix", "Ulothrix"),
    ("lettuce", "Sea lettuce"),
    ("diatom", "Diatom"),
    ("cyanobacterium", "Cyanobacterium"),
    ("bacillus", "Bacillus"),
    ("archaeon", "Archaeon"),
    ("euglena", "Euglena"),
    ("paramecium", "Paramecium"),
    ("dinoflagellate", "Dinoflagellate"),
    ("heliozoan", "Heliozoan"),
    ("radiolarian", "Radiolarian"),
    ("vorticella", "Vorticella"),
    ("seaweed", "Seaweed"),
    ("hydra", "Hydra"),
];
/// Promoter inputs that carry a left-minus-right sense: fuel, light, scent, remains.
const TOWARD_LIGHT: usize = INPUTS - 3;
const TOWARD_SCENT: usize = INPUTS - 2;
/// Promoter input that rises with the number of neighbours a cell is tied to.
const BONDED: usize = 3;
/// Promoter input that rises with each division since the cell its body began as: a thirty-second a division.
const DIVISIONS: usize = 18;

fn gene(action: usize, bias: f32, strength: f32, senses: &[(usize, f32)]) -> Gene {
    let mut promoter = [0.; INPUTS];
    for &(input, weight) in senses {
        promoter[input] = weight;
    }
    Gene {
        action,
        promoter,
        bias,
        strength,
        fates: 15,
    }
}

/// The genes a species starts with, or `None` for an unknown key.
fn design(species: &str) -> Option<Vec<Gene>> {
    // What every founder has: light harvest, uptake, copying, a little glue.
    let basic = Genome::founder().genes;
    let without_glue = || basic.iter().filter(|g| g.action != 2).cloned().collect::<Vec<_>>();
    // An animal lives by eating. It harvests no light, since a cell that
    // does cannot take prey, and it does not soak up dissolved fuel either:
    // an eater that could would multiply on the open sea and strip it bare.
    let animal = || {
        without_glue().into_iter().filter(|g| g.action > 1).collect::<Vec<_>>()
    };
    let motor = || gene(PROPEL, 0.6, 0.6, &[]);
    // How large the cell grows before it divides: a third of the usual size
    // for the small, twice it for a grazer, half as large again for a hunter
    // (whose mouth then doubles it).
    let small = || gene(SIZE, 1., 0.1, &[]);
    // A light-sensing spot. What feeds on light lives where the light is,
    // so an eater that swims toward it from afar finds its food; the scent
    // of life takes over close by.
    let eye = || gene(crate::development::PHOTIC, 1., 0.5, &[]);
    let mut genes = match species {
        "drifter" => basic.clone(),
        // Plant-like: a strong light harvester in a wall. Sisters stay cemented.
        "alga" => {
            let mut g = without_glue();
            g.extend([gene(0, 1., 0.4, &[]), gene(WALL, 1., 0.6, &[])]);
            g
        }
        // A thread one cell wide. Walled cells divide along the body's own
        // axis and stay cemented end to end, and only a cell at an end, with
        // one neighbour or none, divides at all. The walls hold the line
        // straight. A daughter that fails to stay cemented starts a new thread.
        "ulothrix" => {
            let mut g: Vec<Gene> = without_glue().into_iter().filter(|g| g.action != 7).collect();
            g.extend([
                gene(0, 1., 0.4, &[]),
                gene(WALL, 1., 0.9, &[]),
                gene(7, 0.6, 2., &[(BONDED, -2.)]),
                // The second gene makes nothing; it cancels the turn the
                // first would give each new cell, so the thread runs straight.
                gene(POLARITY, 1., 0.9, &[]),
                gene(POLARITY, -1., 0.9, &[]),
            ]);
            g
        }
        // A blade one cell thick. As the thread, but each division goes off
        // at a slant to one side or the other, and any cell with fewer than
        // three neighbours divides: the body spreads at its edge into a sheet.
        "lettuce" => {
            let mut g: Vec<Gene> = without_glue().into_iter().filter(|g| g.action != 7).collect();
            g.extend([
                gene(0, 1., 0.4, &[]),
                gene(WALL, 1., 0.9, &[]),
                gene(7, 1., 2., &[(BONDED, -2.)]),
                gene(POLARITY, 1., 0.9, &[]),
                gene(POLARITY, -1., 0.9, &[]),
                gene(crate::physical::BROADEN, 0.5, 1., &[]),
            ]);
            g
        }
        // A single light-feeder in a shell of glass, laid down from silica
        // dissolved in the water. The shell is cheap, makes the cell hard to
        // swallow or digest, and weighs it down; and the cell cannot divide
        // until each daughter's half is made whole, so where the silica runs
        // out, diatoms stop.
        "diatom" => {
            let mut g = without_glue();
            g.extend([gene(0, 1., 0.4, &[]), gene(SHELL, 1., 0.8, &[])]);
            g
        }
        // The sea's first light-feeders: bacteria, small and rod-shaped.
        "cyanobacterium" => {
            let mut g = without_glue();
            g.extend([gene(0, 1., 0.4, &[]), small(), gene(ELONGATE, 1., 0.5, &[])]);
            g
        }
        // A rod-shaped bacterium of decay. A rod has more surface for its
        // size than a ball, and it is through its surface that it feeds.
        "bacillus" => {
            let mut g = animal();
            g.extend([gene(15, 1., 0.8, &[]), small(), gene(ELONGATE, 1., 0.9, &[])]);
            g
        }
        // A microbe of the vents. It needs neither light nor food, only the
        // chemistry of hot rock, and that is in the rock: it holds on to the
        // bed and grows there as a mat.
        "archaeon" => {
            let mut g: Vec<Gene> = without_glue().into_iter().filter(|g| g.action != 0).collect();
            g.extend([gene(CHEMO, 1., 0.8, &[]), small(), gene(HOLDFAST, 1., 0.8, &[])]);
            g
        }
        // A long green swimmer with an eye-spot: it makes its food in the
        // light, swims to find it, and soaks up what is dissolved in the dark.
        "euglena" => {
            let mut g = without_glue();
            g.extend([
                gene(0, 1., 0.4, &[]),
                motor(),
                eye(),
                gene(STEER, 0.5, 1., &[(TOWARD_LIGHT, 1.5)]),
                gene(ELONGATE, 1., 0.7, &[]),
            ]);
            g
        }
        // A slipper-shaped swimmer whose beating draws bacteria and the
        // smallest algae into its mouth.
        "paramecium" => {
            let mut g = animal();
            g.extend([
                gene(ENGULF, 2., 1., &[]),
                gene(SIZE, 1., 0.6, &[]),
                motor(),
                gene(STEER, 0.5, 1., &[(TOWARD_SCENT, 1.5)]),
                gene(ELONGATE, 1., 0.6, &[]),
            ]);
            g
        }
        // A swimming light-feeder that makes poison: in numbers, a red tide.
        "dinoflagellate" => {
            let mut g = without_glue();
            g.extend([
                gene(0, 1., 0.4, &[]),
                motor(),
                eye(),
                gene(STEER, 0.5, 1., &[(TOWARD_LIGHT, 1.5)]),
                gene(TOXIN, 1., 0.5, &[]),
            ]);
            g
        }
        // A ball of spines that drifts and swallows whatever blunders into it.
        "heliozoan" => {
            let mut g = animal();
            g.extend([gene(ENGULF, 2., 1., &[]), gene(SIZE, 1., 0.65, &[]), gene(SPINE, 1., 0.5, &[])]);
            g
        }
        // As the heliozoan, with a skeleton of glass.
        "radiolarian" => {
            let mut g = animal();
            g.extend([
                gene(ENGULF, 2., 1., &[]),
                gene(SIZE, 1., 0.65, &[]),
                gene(SPINE, 1., 0.5, &[]),
                gene(SHELL, 1., 0.5, &[]),
            ]);
            g
        }
        // A mouth on a stalk. It grips the bottom and its beating draws the
        // small life of the water to it.
        "vorticella" => {
            let mut g = animal();
            g.extend([gene(ENGULF, 2., 1., &[]), gene(SIZE, 1., 0.6, &[]), gene(HOLDFAST, 1., 0.8, &[])]);
            g
        }
        // A thread of walled cells, as Ulothrix, that holds on to the bottom.
        "seaweed" => {
            let mut g: Vec<Gene> = without_glue().into_iter().filter(|g| g.action != 7).collect();
            g.extend([
                gene(0, 1., 0.4, &[]),
                gene(WALL, 1., 0.9, &[]),
                gene(7, 0.6, 2., &[(BONDED, -2.)]),
                gene(POLARITY, 1., 0.9, &[]),
                gene(POLARITY, -1., 0.9, &[]),
                gene(HOLDFAST, 1., 0.8, &[]),
            ]);
            g
        }
        // A small animal fixed to the bottom: cells that stay together, grip
        // the bed, sting what touches them and swallow it.
        "hydra" => {
            let mut g = animal();
            g.extend([
                gene(2, 1., 0.8, &[]),
                gene(ENGULF, 2., 1., &[]),
                gene(TOXIN, 1., 0.4, &[]),
                gene(HOLDFAST, 1., 0.8, &[]),
            ]);
            g
        }
        // A light feeder that swims toward the light.
        "swimmer" => {
            let mut g = without_glue();
            g.extend([motor(), eye(), gene(STEER, 0.5, 1., &[(TOWARD_LIGHT, 1.5)])]);
            g
        }
        // An egg that becomes a body. Alone, the cell grows three times the
        // usual size. Each neighbour it is tied to turns its size gene down,
        // to nothing at three, so each daughter is already big enough to
        // divide again, and again: the egg cleaves, without growing, into a
        // ball of small cells that stay together and live on light. A cell
        // that comes loose grows into the next egg.
        //
        // It also counts. Its drive to divide falls with every division since
        // the egg and stops at the sixth, so the body ends at a few dozen
        // cells. Late in that count, a cell left with a single neighbour
        // commits to a fate in which the glue gene is switched off; it comes
        // away from the body, swims off, and alone it is the next egg.
        "volvox" => {
            let mut g: Vec<Gene> = without_glue().into_iter().filter(|g| g.action != 7).collect();
            g.extend([
                gene(0, 1., 0.4, &[]),
                gene(7, 0.375, 2., &[(DIVISIONS, -2.)]),
                Gene { fates: 13, ..gene(2, 2., 1., &[]) },
                gene(SIZE, 1., 0.9, &[(BONDED, -2.)]),
                gene(crate::physical::BECOME[0], 0.6, 1., &[(DIVISIONS, 2.), (BONDED, -2.)]),
                // A loose cell swims, which carries the egg clear of its
                // parent; tied to two neighbours the motor is off.
                gene(PROPEL, 0.6, 0.6, &[(BONDED, -2.)]),
            ]);
            g
        }
        // A hunter of hunters: three times the usual size before its mouth
        // doubles it, and skinned, which also makes it no kin of the smaller
        // hunters it can therefore swallow. Slow to breed, as big things are.
        "leviathan" => {
            let mut g = animal();
            g.extend([
                gene(ENGULF, 2., 1., &[]),
                gene(SIZE, 1., 0.9, &[]),
                gene(crate::physiology::BARRIER, 1., 0.5, &[]),
                gene(crate::physiology::REPAIR, 1., 0.4, &[]),
                motor(),
                eye(),
                gene(STEER, 0.5, 1., &[(TOWARD_SCENT, 1.5), (TOWARD_LIGHT, 0.8)]),
            ]);
            g
        }
        // A free cell living in the dark on what the dead leave, as the sea's
        // bacteria do. It breaks remains down and gives back the nitrogen
        // bound in them.
        "decomposer" => {
            let mut g = animal();
            g.extend([gene(15, 1., 0.8, &[]), small()]);
            g
        }
        // The sea's small free light-feeders: no wall, no glue, a third the
        // usual size. Small cells feed and breed fast for their size, and
        // anything with a mouth can swallow them whole.
        "plankton" => {
            let mut g = without_glue();
            g.extend([gene(0, 1., 0.4, &[]), small()]);
            g
        }
        // An animal: it has given up the light, digests whatever it touches,
        // walls and all, and follows the scent of life.
        "grazer" => {
            let mut g = animal();
            g.extend([
                gene(9, 1., 0.8, &[]),
                gene(SIZE, 1., 0.75, &[]),
                motor(),
                eye(),
                gene(STEER, 0.5, 1., &[(TOWARD_SCENT, 1.5), (TOWARD_LIGHT, 0.8)]),
            ]);
            g
        }
        // An animal that swallows smaller cells whole and chases their scent.
        "hunter" => {
            let mut g = animal();
            g.extend([
                gene(ENGULF, 2., 1., &[]),
                gene(SIZE, 1., 0.65, &[]),
                motor(),
                eye(),
                gene(STEER, 0.5, 1., &[(TOWARD_SCENT, 1.5), (TOWARD_LIGHT, 0.8)]),
            ]);
            g
        }
        // A drifter in spines and skin.
        "thornback" => {
            let mut g = basic.clone();
            g.extend([gene(SPINE, 1., 0.6, &[]), gene(BARRIER, 1., 0.5, &[])]);
            g
        }
        // A body that grows along its heading, beats, and rows or crawls on fins.
        "rower" => {
            let mut g = without_glue();
            g.extend([
                gene(2, 1., 0.6, &[]),
                gene(POLARITY, 1., 0.6, &[]),
                gene(4, 1., 0.6, &[]),
                gene(CHANNEL, 1., 0.6, &[]),
                gene(5, 1., 0.6, &[]),
                gene(FIN, 1., 0.6, &[]),
            ]);
            g
        }
        // A swimmer some of whose daughters are gametes: they seek out others
        // by scent and fuse with a gamete from a different body.
        "courter" => {
            let mut g = without_glue();
            g.extend([
                motor(),
                gene(STEER, 0.5, 1., &[(TOWARD_SCENT, 1.5)]),
                // Weakly driven on purpose: about half its daughters are gametes.
                gene(GAMETIC, 2., 0.04, &[]),
                gene(FUSOGEN, 1., 0.6, &[]),
            ]);
            g
        }
        _ => return None,
    };
    genes.truncate(crate::autonomous::MAX_GENES);
    Some(genes)
}

/// A sustained species is restocked when it falls below this many cells.
const FLOOR: usize = 8;
/// Cycles between checks on sustained species.
const WATCH: f32 = 25.;

impl Core {
    /// Keep the species the player asked for from dying out: when one falls
    /// below a handful of cells, a few more founders are released. Worlds
    /// with nothing sustained are untouched.
    pub(crate) fn sustain_species(&mut self) {
        if (self.time / WATCH).floor() == ((self.time - DT) / WATCH).floor() {
            return;
        }
        let Some(h) = self.autonomous.as_ref().and_then(|e| e.physical.as_ref()) else {
            return;
        };
        if h.steering.sustained.is_empty() || self.config.physical_assay != "full" {
            return;
        }
        let living = h.species(self.autonomous.as_ref().unwrap());
        let short: Vec<String> = h
            .steering
            .sustained
            .iter()
            .filter(|key| {
                let name = SPECIES.iter().find(|s| s.0 == key.as_str()).unwrap().1;
                living.iter().find(|l| l.0 == name).map_or(0, |l| l.1) < FLOOR
            })
            .cloned()
            .collect();
        for key in short {
            if self.seed_species(&key, 4).is_ok() {
                let h = self.autonomous.as_mut().unwrap().physical.as_mut().unwrap();
                h.steering.restocked += 4;
            }
        }
    }
    /// One founder of a starter species as a portable organism: its display
    /// name and the snapshot any placement accepts.
    pub fn species_specimen(&self, species: &str) -> Result<(&'static str, String), String> {
        let genes = design(species).ok_or("Unknown starter species")?;
        let name = SPECIES.iter().find(|s| s.0 == species).unwrap().1;
        // A founder is born with the working parts its genes call for and no others.
        let mut material = Inventory::founder();
        for (k, p) in material.proteins.iter_mut().enumerate() {
            if !genes.iter().any(|g| g.action == k) {
                *p = 0.;
            }
        }
        // A hunter is released full grown: a mouth has to be bigger than
        // what it swallows, and a newborn one is not.
        if genes.iter().any(|g| g.action == ENGULF) {
            material.biomass = 0.6;
            material.fuel = 0.1;
        }
        // The catalyst a vent cell lives by is part of what it is born with,
        // as a light-feeder's is.
        if genes.iter().any(|g| g.action == CHEMO) {
            material.proteins[CHEMO] = 0.002;
        }
        // An egg is laid ready to hold its first daughters to it.
        if species == "volvox" {
            material.proteins[2] = 0.02;
        }
        // What lives fixed to the bottom is set down ready to grip it.
        if genes.iter().any(|g| g.action == HOLDFAST) {
            material.proteins[HOLDFAST] = 0.02;
        }
        // Everything a founder holds is in proportion to the size its genes set.
        let stature = genes
            .iter()
            .find(|g| g.action == SIZE)
            .map_or(1., |g| 4f64.powf(2. * (g.bias.clamp(0., 1.) * g.strength).clamp(0., 1.) as f64 - 1.));
        material.biomass *= stature;
        material.fuel *= stature;
        material.nitrogen *= stature;
        material.phosphorus *= stature;
        for p in &mut material.proteins {
            *p *= stature;
        }
        let founder = Cell {
            collection_sources: vec![],
            material: Some(material),
            // Until it has fed, a founder is counted where its genes place it.
            state: crate::physiology::CellState {
                trophic: if genes.iter().any(|g| g.action == 0) { 0. } else { 1. },
                scavenged: if species == "decomposer" { 1. } else { 0. },
                stature: stature as f32,
                ..Default::default()
            },
            lifecycle: crate::development::Lifecycle::default(),
            id: 1,
            parent: 0,
            founder: 1,
            generation: 0,
            genome: Genome { id: 1, genes },
            p: V3::default(),
            v: V3::default(),
            energy: 5. * stature as f32,
            age: 0.,
            cooldown: 0.,
            chemicals: [0.; 2],
            expression: [0.; ACTIONS],
            heading: 0.,
        };
        let json = serde_json::to_string(&Specimen {
            format: "primordia-organism".into(),
            version: 1,
            name: name.into(),
            source_body: 1,
            source_seed: self.config.seed,
            captured_cycle: 0.,
            cells: vec![founder],
            bonds: vec![],
            joints: vec![],
            synapses: vec![],
        })
        .map_err(|e| e.to_string())?;
        Ok((name, json))
    }
    /// Release `count` single-celled founders of a starter species at open
    /// sites around the world. External input, like any introduced organism.
    pub fn seed_species(&mut self, species: &str, count: u32) -> Result<String, String> {
        if design(species).is_none() {
            return Err("Unknown starter species".into());
        }
        if !(1..=64).contains(&count) {
            return Err("Release between 1 and 64 founders at a time".into());
        }
        let (name, json) = self.species_specimen(species)?;
        // What lives on the rock's chemistry is set down where the rock is
        // hottest, not where the light is.
        let vents = design(species).is_some_and(|g| {
            g.iter().any(|g| g.action == CHEMO) && !g.iter().any(|g| g.action == 0)
        });
        let mut released = 0;
        for _ in 0..count * 12 {
            if released == count {
                break;
            }
            // Of a few spots of open water, the best lit: life gathers
            // where the energy is, and an eater set down in the dark between
            // the lit patches never finds its food.
            let mut best: Option<(f32, f32, f64)> = None;
            for _ in 0..if vents { 16 } else { 4 } {
                let d = self
                    .autonomous
                    .as_ref()
                    .and_then(|e| e.physical.as_ref())
                    .ok_or("Starter species need the current cellular world")?
                    .domain;
                let width = d.half() * 2.;
                let (a, b) = (self.random(), self.random());
                let (x, z) = if d.spherical() {
                    ((a - 0.5) * width, (b * 2. - 1.).asin() * width / std::f32::consts::TAU)
                } else {
                    ((a - 0.5) * (width - 6.), (b - 0.5) * (width - 6.))
                };
                let h = self.autonomous.as_ref().unwrap().physical.as_ref().unwrap();
                let site = d.index(x, z);
                let wet = h
                    .planet
                    .as_ref()
                    .is_none_or(|p| !d.spherical() || p.surface_depth(site) > 0.5);
                let light = if vents {
                    h.planet.as_ref().map_or(0., |p| p.vent(site))
                } else {
                    h.light_fraction(site)
                };
                if wet && best.is_none_or(|spot| light > spot.2) {
                    best = Some((x, z, light));
                }
            }
            let (x, z, wet) = best.map_or((0., 0., false), |spot| (spot.0, spot.1, true));
            // Every starter species begins in the water, as life did.
            if wet && self.place_specimen(&json, x, z).is_ok() {
                released += 1;
            }
        }
        if released == 0 {
            return Err("No open water with room for them was found".into());
        }
        Ok(format!(
            "{released} {name}{} released into the water.",
            if released == 1 { "" } else { "s" }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> Core {
        let mut c = Core::new(Config {
            initial_cells: 24,
            ..Config::default()
        })
        .unwrap();
        c.start_physical();
        // A planet whose air is already breathable.
        c.autonomous.as_mut().unwrap().physical.as_mut().unwrap().oxygenate(1.);
        c
    }
    /// Cells descended from a starter species.
    fn of<'a>(c: &'a Core, name: &'a str) -> impl Iterator<Item = &'a Cell> {
        c.autonomous
            .as_ref()
            .unwrap()
            .cells
            .iter()
            .filter(move |cell| cell.collection_sources.first().is_some_and(|s| s == name))
    }

    #[test]
    fn every_starter_species_is_released_with_its_own_genes_and_nothing_else_changes() {
        let mut c = world();
        let before = c.autonomous.as_ref().unwrap().cells.len();
        for (key, name) in SPECIES {
            let note = c
                .steer(&format!(r#"{{"kind":"seed","species":"{key}","count":6}}"#))
                .unwrap();
            assert!(note.starts_with('6'), "{note}");
            assert_eq!(of(&c, name).count(), 6);
            let genes = design(key).unwrap();
            assert!(of(&c, name).all(|cell| cell.genome.genes == genes && cell.genome.valid()));
        }
        assert!(c.steer(r#"{"kind":"seed","species":"dragon","count":6}"#).is_err());
        assert!(c.steer(r#"{"kind":"seed","species":"hunter","count":0}"#).is_err());
        assert!(c.steer(r#"{"kind":"seed","species":"hunter","count":900}"#).is_err());
        let e = c.autonomous.as_ref().unwrap();
        assert_eq!(e.cells.len(), before + 6 * SPECIES.len());
        assert!(e.valid(), "the world's books still balance");
        let census = e.physical.as_ref().unwrap().species(e);
        assert_eq!(census.len(), SPECIES.len());
        assert!(census.iter().all(|s| s.1 == 6));
    }

    #[test]
    fn starter_species_grow_into_what_they_were_designed_to_be() {
        let mut c = world();
        // Kept from dying out: in a pond this small the eaters and the eaten
        // do not always both last, and this test is about what each one builds.
        for (key, _) in SPECIES {
            c.steer(&format!(r#"{{"kind":"seed","species":"{key}","count":10,"sustain":true}}"#)).unwrap();
        }
        for _ in 0..3 {
            c.step(400);
        }
        // Of the cells old enough to have built anything.
        let share = |name: &str, action: usize| {
            let (mut living, mut built) = (0, 0);
            for cell in of(&c, name).filter(|cell| cell.age > 10.) {
                living += 1;
                built += usize::from(cell.activity(action) > 0.25);
            }
            assert!(living > 0, "no grown {name} within 300 cycles");
            built as f32 / living as f32
        };
        assert!(share("Alga", WALL) > 0.5, "algae are walled");
        assert!(share("Swimmer", PROPEL) > 0.5, "swimmers have motors");
        assert!(share("Grazer", 9) > 0.5, "grazers make enzymes");
        assert!(share("Hunter", ENGULF) > 0.5, "hunters have mouths");
        assert!(share("Thornback", SPINE) > 0.5, "thornbacks have spines");
        assert!(share("Rower", 4) > 0.5 && share("Rower", FIN) > 0.5, "rowers have muscle and fins");
        assert!(share("Courter", FUSOGEN) > 0.5, "courters can fuse");
        assert!(c.autonomous.as_ref().unwrap().valid());
    }

    #[test]
    fn an_egg_cleaves_into_a_body_of_small_cells_without_growing_first() {
        let mut c = world();
        c.steer(r#"{"kind":"light","scale":6}"#).unwrap();
        c.steer(r#"{"kind":"seed","species":"volvox","count":1}"#).unwrap();
        let egg = of(&c, "Volvox").next().unwrap();
        let (start, size) = (egg.material.as_ref().unwrap().biomass, egg.stature());
        assert!(size > 2.5, "alone, the cell is built large: {size}");
        // Watch it: the cell count, and the carbon in all of them together.
        let mut history = Vec::new();
        for _ in 0..30 {
            c.step(40);
            let cells: Vec<_> = of(&c, "Volvox").collect();
            history.push((cells.len(), cells.iter().map(|x| x.material.as_ref().unwrap().biomass).sum::<f64>()));
        }
        eprintln!("egg of {start:.2}: cells and their total biomass every ten cycles {history:?}");
        let e = c.autonomous.as_ref().unwrap();
        let body = e
            .groups()
            .into_iter()
            .filter(|g| e.cells[g[0]].collection_sources.first().is_some_and(|s| s == "Volvox"))
            .max_by_key(Vec::len)
            .unwrap();
        let small = body.iter().filter(|&&i| e.cells[i].stature() < 0.6).count();
        eprintln!("largest body {} cells, {small} of them small", body.len());
        assert!(body.len() >= 8, "the egg became a body: {}", body.len());
        assert!(small * 5 >= body.len() * 4, "of small cells: {small} of {}", body.len());
        // Cleavage: by the time there were eight cells they held little more
        // than the egg did, where eight ordinary cells would hold several times it.
        let eight = history.iter().find(|h| h.0 >= 8).unwrap();
        assert!(eight.1 < start * 3.5, "eight cells from one egg's worth: {} from {start}", eight.1);
        assert!(e.valid());
        // It stops: a body ends at a few dozen cells, and sheds loose cells
        // that become the next eggs and the next bodies.
        let mut sizes: Vec<usize> = e
            .groups()
            .into_iter()
            .filter(|g| e.cells[g[0]].collection_sources.first().is_some_and(|s| s == "Volvox"))
            .map(|g| g.len())
            .collect();
        sizes.sort_unstable();
        eprintln!("bodies after {} cycles: {sizes:?}", c.time);
        let bodies: Vec<usize> = sizes.iter().copied().filter(|s| *s >= 8).collect();
        let eggs = sizes.iter().filter(|s| **s == 1).count();
        assert!(bodies.len() >= 5 && eggs >= 5, "its loose daughters swim off and grow into more bodies: {} bodies, {eggs} loose cells", bodies.len());
        // Most bodies end at a few dozen cells. (Kin bodies that touch can
        // still join, so the largest in a crowded pond is bigger.)
        assert!(bodies[bodies.len() / 2] <= 80, "a body stops growing: median {}", bodies[bodies.len() / 2]);
    }

    /// The largest body of a species: how many neighbours its most tied
    /// cell has, and how far it reaches along its three principal axes.
    fn shape(c: &Core, name: &str) -> (usize, usize, [f32; 3]) {
        let e = c.autonomous.as_ref().unwrap();
        let body = e
            .groups()
            .into_iter()
            .filter(|g| e.cells[g[0]].collection_sources.first().is_some_and(|s| s == name))
            .max_by_key(Vec::len)
            .unwrap();
        let ids = e.index();
        let mut ties = vec![0usize; e.cells.len()];
        for b in &e.bonds {
            ties[ids[&b.a]] += 1;
            ties[ids[&b.b]] += 1;
        }
        let n = body.len() as f32;
        let middle = body.iter().fold(V3::default(), |sum, &i| sum.add(e.cells[i].p.mul(1. / n)));
        let offsets: Vec<V3> = body.iter().map(|&i| e.cells[i].p.sub(middle)).collect();
        // Each axis in turn is the direction the body spreads furthest along,
        // once the axes already found are taken out.
        let mut axes: Vec<V3> = Vec::new();
        let mut reach = [0.; 3];
        for extent in &mut reach {
            let mut axis = V3::new(0.37, 0.51, 0.77);
            for _ in 0..60 {
                let mut next = V3::default();
                for d in &offsets {
                    let d = axes.iter().fold(*d, |d, a| d.sub(a.mul(d.dot(*a))));
                    next = next.add(d.mul(d.dot(axis)));
                }
                axis = next.normalized();
            }
            *extent = (offsets.iter().map(|d| d.dot(axis).powi(2)).sum::<f32>() / n).sqrt();
            axes.push(axis);
        }
        (body.len(), body.iter().map(|&i| ties[i]).max().unwrap(), reach)
    }

    #[test]
    fn walled_cells_that_divide_along_an_axis_grow_into_a_thread_or_a_sheet() {
        let grown = |key: &str, name: &str, ticks: u32| {
            let mut c = world();
            c.steer(r#"{"kind":"light","scale":6}"#).unwrap();
            c.steer(&format!(r#"{{"kind":"seed","species":"{key}","count":1}}"#)).unwrap();
            // The largest body it grows to in that time, looked for every
            // twenty cycles: in a pond bodies also break up and are eaten.
            let mut found = (0, 0, [0.; 3]);
            for _ in 0..ticks / 80 {
                c.step(80);
                let now = shape(&c, name);
                if now.0 > found.0 {
                    found = now;
                }
            }
            assert!(c.autonomous.as_ref().unwrap().valid());
            eprintln!("{name}: {} cells, at most {} neighbours, reach {:.2?}", found.0, found.1, found.2);
            found
        };
        // A clump, for comparison: an alga's daughters go any way they like.
        let (_, _, clump) = grown("alga", "Alga", 640);
        // A thread: a line of cells with one neighbour at each end and two
        // everywhere else, many times longer than it is wide.
        let (cells, ties, reach) = grown("ulothrix", "Ulothrix", 640);
        assert!(cells >= 8, "a thread of some length: {cells}");
        assert!(ties <= 3, "a thread does not branch: {ties}");
        assert!(reach[0] > 3. * reach[1], "long and thin: {reach:?}");
        assert!(
            reach[0] / reach[1] > 2. * clump[0] / clump[1],
            "far thinner for its length than a clump: {reach:?} against {clump:?}"
        );
        // A sheet: broad two ways and one cell thick.
        let (cells, _, reach) = grown("lettuce", "Sea lettuce", 640);
        assert!(cells >= 15, "a sheet of some size: {cells}");
        assert!(reach[1] > 0.4 * reach[0], "broad both ways: {reach:?}");
        assert!(reach[2] < 0.4 * reach[1], "and thin: {reach:?}");
        assert!(
            reach[2] / reach[1] < 0.6 * clump[2] / clump[1],
            "far flatter than a clump: {reach:?} against {clump:?}"
        );
    }

    #[test]
    fn single_cells_build_what_marks_them_out_and_silicon_is_never_made_or_lost() {
        let grown = |key: &str, ticks: u32| {
            let mut c = world();
            c.steer(r#"{"kind":"light","scale":6}"#).unwrap();
            c.steer(&format!(r#"{{"kind":"seed","species":"{key}","count":10}}"#)).unwrap();
            let silicon = |c: &Core| {
                let e = c.autonomous.as_ref().unwrap();
                e.physical.as_ref().unwrap().silicon_total(&e.cells)
            };
            let before = silicon(&c);
            c.step(ticks);
            assert!(c.autonomous.as_ref().unwrap().valid());
            // Whatever was built, died, sank and dissolved, the planet holds
            // exactly the silicon it began with.
            assert!((silicon(&c) - before).abs() < 1e-6 * before, "{key}: {} from {before}", silicon(&c));
            c
        };
        let mean = |c: &Core, name: &str, of_cell: &dyn Fn(&Cell) -> f32| {
            let cells: Vec<&Cell> = of(c, name).filter(|cell| cell.age > 10.).collect();
            assert!(cells.len() >= 10, "{name} multiplied: {}", cells.len());
            cells.iter().map(|cell| of_cell(cell)).sum::<f32>() / cells.len() as f32
        };
        // A diatom lives in a whole shell of glass, taken out of the water.
        let c = grown("diatom", 720);
        let shell = mean(&c, "Diatom", &|cell| cell.material.as_ref().unwrap().shell() as f32);
        assert!(shell > 0.5, "diatoms are shelled: {shell}");
        let e = c.autonomous.as_ref().unwrap();
        let held: f64 = of(&c, "Diatom").map(|cell| cell.material.as_ref().unwrap().silica).sum();
        assert!(held > 1., "and the glass is real silica: {held}");
        assert!(e.cells.iter().filter(|cell| cell.material_role() == 19).count() >= 10);
        // Bacteria are rods; a plain cell is round.
        let c = grown("cyanobacterium", 480);
        let long = mean(&c, "Cyanobacterium", &|cell| cell.state.aspect);
        assert!(long > 2., "a cyanobacterium is a rod: {long}");
        assert!(of(&c, "Cyanobacterium").all(|cell| cell.outline() >= 1.));
        let c = grown("plankton", 480);
        assert!(mean(&c, "Plankton", &|cell| cell.state.aspect) < 1.01);
        // Seaweed holds on to the bottom.
        let c = grown("seaweed", 720);
        let held = mean(&c, "Seaweed", &|cell| if cell.state.anchored { 1. } else { 0. });
        assert!(held > 0.3, "seaweed grips the bed: {held}");
    }

    #[test]
    fn a_holdfast_keeps_a_cell_where_the_water_would_carry_it_off() {
        // Founders of two kinds in the same stirred water. Those with a
        // holdfast grip the bed where they first touch it and are still
        // there sixty cycles later; the others have drifted on.
        let drift = |key: &str, name: &str| {
            let mut c = world();
            c.steer(r#"{"kind":"mixing","scale":3}"#).unwrap();
            c.steer(&format!(r#"{{"kind":"seed","species":"{key}","count":8}}"#)).unwrap();
            for cell in &mut c.autonomous.as_mut().unwrap().cells {
                cell.cooldown = 1e6; // none divides: this is about staying put
            }
            c.step(120);
            let held = of(&c, name).any(|cell| cell.state.anchored);
            let start: Vec<(u32, V3)> = of(&c, name)
                .filter(|cell| cell.state.anchored == held)
                .map(|cell| (cell.id, cell.p))
                .collect();
            c.step(240);
            let moved: Vec<f32> = of(&c, name)
                .filter_map(|cell| start.iter().find(|s| s.0 == cell.id).map(|s| cell.p.sub(s.1).len()))
                .collect();
            assert!(!moved.is_empty());
            (moved.iter().sum::<f32>() / moved.len() as f32, start.len())
        };
        let (held, gripping) = drift("vorticella", "Vorticella");
        let (loose, _) = drift("heliozoan", "Heliozoan");
        eprintln!("moved in 60 cycles: {gripping} gripping {held:.2}, loose {loose:.2}");
        assert!(gripping >= 4, "most of them have hold of the bed: {gripping}");
        assert!(held < 0.3, "a held cell stays where it gripped: {held}");
        assert!(loose > 0.5, "a loose one drifts: {loose}");
    }

    #[test]
    fn a_sustained_species_is_restocked_and_an_unsustained_one_is_left_to_die() {
        let gone = |sustain: bool| {
            let mut c = world();
            c.steer(&format!(
                r#"{{"kind":"seed","species":"hunter","count":6,"sustain":{sustain}}}"#
            ))
            .unwrap();
            // A plague takes every hunter; then the watch comes round.
            for cell in &mut c.autonomous.as_mut().unwrap().cells {
                if !cell.collection_sources.is_empty() {
                    cell.state.injury = 1.;
                }
            }
            c.step(4 * 26);
            let h = c.autonomous.as_ref().unwrap().physical.as_ref().unwrap();
            (of(&c, "Hunter").count(), h.steering.restocked)
        };
        let (living, restocked) = gone(true);
        assert!(living >= 4 && restocked == 4, "{living} living, {restocked} restocked");
        assert_eq!(gone(false), (0, 0));
        let mut c = world();
        c.steer(r#"{"kind":"seed","species":"alga","count":6}"#).unwrap();
        c.steer(r#"{"kind":"sustain","on":true}"#).unwrap();
        let kept = |c: &Core| c.autonomous.as_ref().unwrap().physical.as_ref().unwrap().steering.sustained.clone();
        assert_eq!(kept(&c), vec!["alga".to_string()]);
        c.steer(r#"{"kind":"sustain","on":false}"#).unwrap();
        assert!(kept(&c).is_empty());
    }

    #[test]
    fn courters_make_gametes_that_find_each_other_and_fuse() {
        let mut c = world();
        for _ in 0..2 {
            c.steer(r#"{"kind":"seed","species":"courter","count":40}"#).unwrap();
        }
        for _ in 0..3 {
            c.step(400);
        }
        let e = c.autonomous.as_ref().unwrap();
        let d = &e.physical.as_ref().unwrap().development;
        assert!(d.gametes > 0, "gametes were made");
        assert!(d.zygotes > 0, "and some fused: {} gametes, {} zygotes", d.gametes, d.zygotes);
        assert!(of(&c, "Courter").count() > 20, "the species is still here");
    }
}
