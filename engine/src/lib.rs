use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use wasm_bindgen::prelude::*;

pub mod articulation;
pub mod autonomous;
pub mod body;
pub mod development;
#[cfg(test)]
mod development_tests;
#[cfg(test)]
mod foodweb_tests;
pub mod domain;
pub mod experiment;
pub mod observations;
pub mod specimens;
pub mod bestiary;
pub mod champions;
pub mod trophic;
pub mod steering;
pub mod interventions;
pub mod organics;
pub mod physical;
pub mod physiology;
pub mod planet;
pub mod population;
pub mod surface;
pub mod tissue;
use articulation::Anatomy;
use body::{default_development, Body, DEVELOPMENT};
use experiment::Experiment;
use population::Competition;
use tissue::{Regulation, CELL_BUDGET, MAX_BODY_CELLS};

pub const GRID: usize = 40;
pub const GENE_COUNT: usize = 16;
pub const MAX_POPULATION: usize = 12000;
const DT: f32 = 0.25;
const PHOTO: usize = 0;
const CHEMO: usize = 1;
const MOBILITY: usize = 2;
const ADHESION: usize = 3;
const ARMOR: usize = 4;
const THERMAL: usize = 5;
const FERTILITY: usize = 6;
const SEX: usize = 7;
const SHAPE: usize = 8;
const BRANCHING: usize = 9;
const SPECIALIZATION: usize = 10;
const PIGMENT: usize = 11;
const MUSCLE: usize = 12;
const SKELETON: usize = 13;
const SENSING: usize = 14;
const PREDATION: usize = 15;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub seed: u32,
    pub kind: String,
    pub name: String,
    pub sunlight: f32,
    pub nutrients: f32,
    pub minerals: f32,
    pub temperature: f32,
    pub radiation: f32,
    pub solvent: f32,
    pub mutation: f32,
    pub capacity: usize,
    #[serde(default = "default_world_size", rename = "worldSize")]
    pub world_size: u16,
    #[serde(default, rename = "worldShape")]
    pub world_shape: domain::Shape,
    #[serde(default = "default_founder_spread", rename = "founderSpread")]
    pub founder_spread: String,
    #[serde(default = "default_initial_cells", rename = "initialCells")]
    pub initial_cells: usize,
    #[serde(default = "default_physical_assay", rename = "physicalAssay")]
    pub physical_assay: String,
    #[serde(default = "default_resource_layout", rename = "resourceLayout")]
    pub resource_layout: String,
    pub origin: String,
}
fn default_resource_layout() -> String {
    "diffuse".into()
}
fn default_founder_spread() -> String {
    "local".into()
}
fn default_world_size() -> u16 {
    64
}
fn default_initial_cells() -> usize {
    64
}
fn default_physical_assay() -> String {
    "full".into()
}
impl Default for Config {
    fn default() -> Self {
        Self {
            seed: 8042,
            kind: "tidal".into(),
            name: "The tidal garden".into(),
            sunlight: 0.75,
            nutrients: 0.7,
            minerals: 0.45,
            temperature: 0.48,
            radiation: 0.18,
            solvent: 0.85,
            mutation: 0.09,
            capacity: 4000,
            world_size: default_world_size(),
            world_shape: domain::Shape::Planar,
            founder_spread: default_founder_spread(),
            initial_cells: default_initial_cells(),
            physical_assay: default_physical_assay(),
            resource_layout: default_resource_layout(),
            origin: "seeded".into(),
        }
    }
}
impl Config {
    fn validate(&self) -> Result<(), String> {
        if !["diffuse", "patchy"].contains(&self.resource_layout.as_str()) {
            return Err("Invalid resource layout".into());
        }
        for v in [
            self.sunlight,
            self.nutrients,
            self.minerals,
            self.temperature,
            self.radiation,
            self.solvent,
            self.mutation,
        ] {
            if !v.is_finite() || !(0.0..=1.0).contains(&v) {
                return Err("World parameters must be between 0 and 1".into());
            }
        }
        if !["local", "global"].contains(&self.founder_spread.as_str()) {
            return Err("Invalid founder distribution".into());
        }
        if ![64, 128, 256, 512, 1024, 8192, 64000].contains(&self.world_size) {
            return Err("Unsupported habitat span".into());
        }
        if self.world_size > 256 && self.world_shape != domain::Shape::Sphere {
            return Err("Large spans currently require spherical geometry".into());
        }
        if !(100..=autonomous::MAX_CELLS).contains(&self.capacity) {
            return Err("Living-cell budget must be 100–64000".into());
        }
        if !(1..=autonomous::MAX_FOUNDERS).contains(&self.initial_cells)
            || self.initial_cells > self.capacity
        {
            return Err("Starting cells must be 1–8192 and fit within the cell budget".into());
        }
        if !["full", "passive", "scarcity"].contains(&self.physical_assay.as_str()) {
            return Err("Invalid physical assay".into());
        }
        if !["tidal", "volcanic", "alien", "lab"].contains(&self.kind.as_str())
            || !["seeded", "chemistry"].contains(&self.origin.as_str())
            || self.name.len() > 100
        {
            return Err("Invalid world definition".into());
        }
        Ok(())
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Organism {
    pub id: u32,
    pub parent: u32,
    pub mate: u32,
    #[serde(default)]
    pub family: u32,
    #[serde(default)]
    pub offspring: u32,
    pub lineage: u32,
    pub generation: u32,
    pub x: f32,
    pub z: f32,
    pub vx: f32,
    pub vz: f32,
    pub heading: f32,
    pub energy: f32,
    pub age: f32,
    pub cells: u32,
    pub genes: [f32; GENE_COUNT],
    #[serde(default = "default_development")]
    pub development: [f32; DEVELOPMENT],
    #[serde(default)]
    pub regulation: Option<Regulation>,
    #[serde(default)]
    pub anatomy: Option<Anatomy>,
    #[serde(default)]
    pub body: Body,
    pub cooldown: f32,
    pub alive: bool,
    #[serde(default)]
    pub feeding_cooldown: f32,
    #[serde(default)]
    pub meals: u32,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Lineage {
    pub id: u32,
    pub parent: u32,
    pub name: String,
    pub born: f32,
    pub generation: u32,
    pub color: String,
    pub ancestor: [f32; GENE_COUNT],
    pub peak: usize,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct LifeEvent {
    pub time: f32,
    pub text: String,
    pub r#type: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct History {
    pub time: f32,
    pub population: usize,
    pub cells: u32,
    pub colonies: usize,
    pub diversity: usize,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Core {
    #[serde(default)]
    pub autonomous: Option<autonomous::Evolution>,
    pub version: u32,
    #[serde(default)]
    pub study: bool,
    #[serde(default)]
    pub competition: Option<Competition>,
    pub config: Config,
    pub rng: u32,
    pub time: f32,
    pub next_id: u32,
    pub next_lineage: u32,
    pub births: u32,
    pub deaths: u32,
    pub mating: u32,
    #[serde(default)]
    pub feedings: u32,
    pub organisms: Vec<Organism>,
    pub lineages: BTreeMap<u32, Lineage>,
    pub field: Vec<f32>,
    pub events: Vec<LifeEvent>,
    pub history: Vec<History>,
    pub milestones: BTreeSet<String>,
    pub history_clock: f32,
    #[serde(default)]
    pub lab: Option<Experiment>,
}
fn bounded(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}
fn noise(x: f32, z: f32, seed: u32) -> f32 {
    // Stable inexpensive integer lattice hashing, independent of platform RNG.
    let ix = x.floor() as i32;
    let iz = z.floor() as i32;
    let mut h = (ix as u32).wrapping_mul(374761393) ^ (iz as u32).wrapping_mul(668265263) ^ seed;
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    (h ^ (h >> 16)) as f32 / u32::MAX as f32
}
pub fn field_index(x: f32, z: f32) -> usize {
    let ix = (((x + 32.0) / 64.0 * GRID as f32) as usize).min(GRID - 1);
    let iz = (((z + 32.0) / 64.0 * GRID as f32) as usize).min(GRID - 1);
    iz * GRID + ix
}
fn genome_distance(a: &[f32; GENE_COUNT], b: &[f32; GENE_COUNT]) -> f32 {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f32>() / GENE_COUNT as f32
}
impl Core {
    pub fn new(config: Config) -> Result<Self, String> {
        config.validate()?;
        let mut core = Self {
            autonomous: None,
            version: 4,
            study: false,
            competition: None,
            rng: config.seed,
            config,
            time: 0.0,
            next_id: 1,
            next_lineage: 1,
            births: 0,
            deaths: 0,
            mating: 0,
            feedings: 0,
            organisms: Vec::with_capacity(MAX_POPULATION),
            lineages: BTreeMap::new(),
            field: vec![0.0; GRID * GRID],
            events: vec![],
            history: vec![],
            milestones: BTreeSet::new(),
            history_clock: 0.0,
            lab: None,
        };
        for i in 0..GRID * GRID {
            core.field[i] = core.field_capacity(i);
        }
        if core.config.origin == "seeded" {
            core.seed(160);
        }
        core.record_history();
        Ok(core)
    }
    fn random(&mut self) -> f32 {
        self.rng = self.rng.wrapping_add(0x6D2B79F5);
        let mut t = self.rng;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        unit_fraction(t ^ (t >> 14))
    }
    fn event(&mut self, kind: &str, text: String) {
        self.events.push(LifeEvent {
            time: self.time,
            text,
            r#type: kind.into(),
        });
        if self.events.len() > 60 {
            self.events.remove(0);
        }
    }
    fn milestone(&mut self, key: &str, kind: &str, text: &str) {
        if self.milestones.insert(key.into()) {
            self.event(kind, text.into());
        }
    }
    fn field_capacity(&self, i: usize) -> f32 {
        let patch = noise(
            (i % GRID) as f32 / 4.0,
            (i / GRID) as f32 / 4.0,
            self.config.seed,
        );
        (0.25 + patch * 0.75) * self.config.nutrients * 18.0
    }
    fn minerals_at(&self, x: f32, z: f32) -> f32 {
        let vent = noise(
            (x + 32.0) / 7.0,
            (z + 32.0) / 7.0,
            self.config.seed.wrapping_add(31),
        );
        self.config.minerals * (0.35 + vent * 0.65)
    }
    fn new_lineage(&mut self, genes: [f32; GENE_COUNT], parent: u32, generation: u32) -> u32 {
        let id = self.next_lineage;
        self.next_lineage += 1;
        let roots = [
            "Luma", "Vela", "Nera", "Auri", "Soma", "Cyra", "Orbi", "Thala",
        ];
        let suffix = [
            "viridis",
            "lucens",
            "mineralis",
            "ramosa",
            "gelida",
            "rubra",
            "spiralis",
            "nova",
        ];
        let r = (80.0 + genes[PIGMENT] * 150.0) as u8;
        let g = (100.0 + genes[PHOTO] * 140.0) as u8;
        let b = (115.0 + genes[CHEMO] * 130.0) as u8;
        self.lineages.insert(
            id,
            Lineage {
                id,
                parent,
                name: format!(
                    "{} {} · {}",
                    roots[(id as usize - 1) % 8],
                    suffix[((genes[SHAPE] * 7.99) as usize).min(7)],
                    id
                ),
                born: self.time,
                generation,
                color: format!("#{r:02x}{g:02x}{b:02x}"),
                ancestor: genes,
                peak: 0,
            },
        );
        id
    }
    pub fn seed(&mut self, count: usize) {
        if self.autonomous.is_some() {
            return;
        }
        if self.competition.is_some() {
            self.seed_competitors(count.min(96) as u32);
            return;
        }
        let count = count
            .min(self.config.capacity.saturating_sub(self.organisms.len()))
            .min(CELL_BUDGET.saturating_sub(self.organisms.iter().map(|o| o.cells as usize).sum()))
            .min(1000);
        if count == 0 {
            return;
        }
        // Founders vary in metabolism; every founder begins as a single cell.
        for group in 0..count.min(8) {
            let mut genes = [0.0; GENE_COUNT];
            for gene in &mut genes {
                *gene = 0.04 + self.random() * 0.35;
            }
            genes[PHOTO] = 0.15 + self.random() * 0.75;
            genes[CHEMO] = 0.15 + self.random() * 0.75;
            genes[THERMAL] = self.random();
            genes[SHAPE] = self.random();
            genes[PIGMENT] = self.random();
            genes[SEX] = 0.02 + self.random() * 0.12;
            genes[PREDATION] = self.random() * 0.12;
            let mut development = default_development();
            for value in &mut development {
                *value = bounded(*value + (self.random() - 0.5) * 0.45);
            }
            let lineage = self.new_lineage(genes, 0, 0);
            let mut regulation = Regulation::default();
            for value in regulation
                .rules
                .iter_mut()
                .flatten()
                .chain(regulation.growth.iter_mut())
            {
                *value = bounded(*value + (self.random() - 0.5) * 0.3);
            }
            for _ in (group..count).step_by(8) {
                let a = self.random() * std::f32::consts::TAU;
                let radius = self.random().sqrt() * 25.0;
                let o = Organism {
                    id: self.next_id,
                    parent: 0,
                    mate: 0,
                    family: 0,
                    offspring: 0,
                    lineage,
                    generation: 0,
                    x: a.cos() * radius,
                    z: a.sin() * radius,
                    vx: 0.0,
                    vz: 0.0,
                    heading: a,
                    energy: 5.0 + self.random() * 4.0,
                    age: 0.0,
                    cells: 1,
                    genes,
                    development,
                    regulation: Some(regulation.clone()),
                    anatomy: Some(Anatomy::default()),
                    body: Body::default(),
                    cooldown: self.random() * 8.0,
                    alive: true,
                    feeding_cooldown: 0.,
                    meals: 0,
                };
                self.next_id += 1;
                self.organisms.push(o);
            }
        }
        self.event(
            "origin",
            format!(
                "{count} protocells introduced in {} independent founder lineages.",
                count.min(8)
            ),
        );
    }
    pub fn pulse(&mut self) {
        for i in 0..self.field.len() {
            self.field[i] = (self.field[i] + 4.0).min(self.field_capacity(i) + 4.0);
        }
        self.event("world", "A nutrient pulse entered the basin.".into());
    }
    pub fn step(&mut self, count: u32) {
        for _ in 0..count.min(400) {
            self.tick();
        }
    }
    fn tick(&mut self) {
        if let Some(mut evolution) = self.autonomous.take() {
            evolution.tick(self);
            self.autonomous = Some(evolution);
            self.sustain_species();
            return;
        }
        if let Some(mut lab) = self.lab.take() {
            lab.tick(self);
            self.lab = Some(lab);
            return;
        }
        self.time += DT;
        for i in 0..GRID * GRID {
            let cap = self.field_capacity(i);
            self.field[i] = (self.field[i]
                + DT * self.config.nutrients
                    * self.competition.as_ref().map_or(0.035, |c| c.renewal))
            .min(cap.max(self.field[i] - 0.004));
        }
        // Abiogenesis is explicitly a configurable toy rule, not a molecular simulation.
        if self.competition.is_none() && self.time % 3.0 < DT && self.organisms.len() < 8 {
            let chance =
                self.config.solvent * (self.config.nutrients + self.config.minerals) * 0.055;
            if self.random() < chance {
                self.seed(8);
                if let Some(event) = self.events.last_mut() {
                    event.text = "Local chemistry produced eight candidate replicators (speculative origin rule).".into();
                }
                self.milestone(
                    "origin",
                    "origin",
                    "Local chemistry produced a self-replicating protocell candidate.",
                );
            }
        }
        let n = self.organisms.len();
        // Spatial hashing limits collision, mating and predation searches to local neighborhoods.
        let mut buckets = vec![Vec::<usize>::new(); GRID * GRID];
        for (i, o) in self.organisms.iter().enumerate() {
            buckets[field_index(o.x, o.z)].push(i);
        }
        let mut newborns = Vec::new();
        let mut living_cells: usize = self.organisms.iter().map(|o| o.cells as usize).sum();
        for i in 0..n {
            if !self.organisms[i].alive {
                continue;
            }
            let mut o = self.organisms[i].clone();
            let g = o.genes;
            if o.cells > 1 {
                o.body.anatomy = o.anatomy.clone();
            }
            let idx = field_index(o.x, o.z);
            let cells = o.cells as f32;
            let specialized = if o.cells >= 4 { g[SPECIALIZATION] } else { 0.0 };
            if o.cells > 1 && o.regulation.is_some() {
                o.body.regulated = true;
                o.body.sync(
                    o.cells as usize,
                    &g,
                    &o.development,
                    o.x,
                    o.z,
                    o.heading,
                    self.config.seed,
                    false,
                );
                o.body.balance_reserves(o.energy);
            }
            let thermal_stress = (g[THERMAL] - self.config.temperature).abs();
            let protection = g[ARMOR] * 0.65 + g[SKELETON] * specialized * 0.25;
            let surface = cells.powf(0.68)
                * if o.cells > 1 && g[SHAPE] > 0.68 {
                    1.0 + g[BRANCHING] * 0.16
                } else {
                    1.0
                };
            let photo = g[PHOTO] * self.config.sunlight * (0.7 + specialized * 0.6);
            let chemo = g[CHEMO] * self.minerals_at(o.x, o.z) * (0.7 + specialized * 0.5);
            let efficiency = (photo + chemo + 0.16)
                * self.config.solvent
                * (1.0 - thermal_stress * 0.8)
                * (1.0
                    + cells.ln()
                        * g[ADHESION]
                        * (0.25 + self.config.minerals * 0.35 + self.config.nutrients * 0.15));
            // Distributed cells forage at their actual positions, sharing body energy.
            let sites: Vec<usize> = if o.body.nodes.len() > 1 {
                o.body
                    .nodes
                    .iter()
                    .map(|node| field_index(node.p.x, node.p.z))
                    .collect()
            } else {
                vec![idx]
            };
            let demand = DT * surface * efficiency * 1.6 / sites.len() as f32;
            let mut uptake = 0.;
            let mut acquired = Vec::with_capacity(sites.len());
            let mut cues = Vec::with_capacity(sites.len());
            for (cell, site) in sites.into_iter().enumerate() {
                cues.push((self.field[site] / 4.).min(1.));
                let factor = if o.regulation.is_some() && o.cells > 1 {
                    o.body.uptake_factor(cell)
                } else {
                    1.
                };
                let amount = (demand * factor).min(self.field[site]);
                self.field[site] -= amount;
                uptake += amount;
                acquired.push(amount * 3.);
            }
            if let Some(c) = &mut self.competition {
                c.food_energy += (uptake * 3.) as f64;
            }
            let investment = g[ARMOR]
                + (g[MUSCLE] + g[SKELETON]) * specialized
                + g[SENSING] * 0.6
                + g[PREDATION];
            let cost = DT
                * (cells.powf(0.78) * (0.18 + investment * 0.045 + thermal_stress * 0.38)
                    + self.config.radiation * (1.0 - protection) * 0.14
                    + o.vx.hypot(o.vz) * 0.035
                    + (g[PHOTO] + g[CHEMO]) * surface * 0.075);
            if let Some(program) = &o.regulation {
                if o.cells > 1 {
                    o.body.regulate(DT, &g, program, &cues);
                    o.body.maintain_tissues(DT, program, &acquired, cost);
                    o.energy = o.body.reserve_energy();
                } else {
                    o.energy += uptake * 3.0 - cost;
                }
            } else {
                o.energy += uptake * 3.0 - cost;
            }
            o.age += DT;
            o.cooldown = (o.cooldown - DT).max(0.0);
            o.feeding_cooldown = (o.feeding_cooldown - DT).max(0.);

            let mut neighbors = Vec::with_capacity(24);
            let bx = idx % GRID;
            let bz = idx / GRID;
            let perception = 1 + (g[SENSING] * specialized * 3.) as usize;
            for zz in bz.saturating_sub(perception)..=(bz + perception).min(GRID - 1) {
                for xx in bx.saturating_sub(perception)..=(bx + perception).min(GRID - 1) {
                    for &j in buckets[zz * GRID + xx].iter().take(12) {
                        if i != j && self.organisms[j].alive && neighbors.len() < 256 {
                            neighbors.push(j);
                        }
                    }
                }
            }
            let mut fx = 0.0;
            let mut fz = 0.0;
            for &j in &neighbors {
                let other = &self.organisms[j];
                let dx = o.x - other.x;
                let dz = o.z - other.z;
                let dist = dx.hypot(dz).max(0.01);
                let body = 0.13 * (cells.sqrt() + (other.cells as f32).sqrt());
                if dist < body {
                    let force = (body - dist) * 0.8;
                    fx += dx / dist * force;
                    fz += dz / dist * force;
                }
            }
            // Contractile cells improve active motion; sensors follow real resource gradients.
            let gradient_x =
                self.field[field_index(o.x + 1.0, o.z)] - self.field[field_index(o.x - 1.0, o.z)];
            let gradient_z =
                self.field[field_index(o.x, o.z + 1.0)] - self.field[field_index(o.x, o.z - 1.0)];
            let turn = (self.random() - 0.5) * 0.55;
            o.heading += turn;
            let directional_sensing = self
                .competition
                .as_ref()
                .is_none_or(|c| c.directional_sensing);
            if directional_sensing && g[SENSING] > 0.4 && gradient_x.abs() + gradient_z.abs() > 0.1
            {
                let desired = gradient_z.atan2(gradient_x);
                let delta = (desired - o.heading)
                    .sin()
                    .atan2((desired - o.heading).cos());
                o.heading += delta * g[SENSING] * 0.15;
            }
            // Cells sample local chemistry; differentiated sensory bodies can
            // perceive more distant patches, edible neighbors and compatible mates.
            // Signals bias internal motors. They never move the body directly.
            let mut signal = (gradient_x, gradient_z);
            if directional_sensing && g[SENSING] > 0.2 {
                let reach = 2. + g[SENSING] * specialized * 8.;
                let mut best = self.field[idx];
                let mut direction = (0., 0.);
                for ray in 0..8 {
                    let angle = ray as f32 * std::f32::consts::TAU / 8.;
                    let (dx, dz) = (angle.cos() * reach, angle.sin() * reach);
                    let food = self.field[field_index(o.x + dx, o.z + dz)];
                    if food > best {
                        best = food;
                        direction = (dx / reach, dz / reach);
                    }
                }
                signal.0 += direction.0 * (best - self.field[idx]).min(3.);
                signal.1 += direction.1 * (best - self.field[idx]).min(3.);
                let hungry = o.energy < 12. + cells * 5.;
                let mut target = None;
                let mut distance = reach;
                for &j in &neighbors {
                    let other = &self.organisms[j];
                    let r = (other.x - o.x).hypot(other.z - o.z);
                    let edible = hungry
                        && g[PREDATION] > 0.02
                        && o.cells > other.cells
                        && other.lineage != o.lineage;
                    let mating = g[SEX] > 0.5
                        && other.genes[SEX] > 0.5
                        && other.cooldown <= 0.
                        && genome_distance(&g, &other.genes) < 0.07;
                    if (edible || mating) && r < distance {
                        distance = r;
                        target = Some((other.x - o.x, other.z - o.z));
                    }
                }
                if let Some((dx, dz)) = target {
                    let distance = dx.hypot(dz).max(0.1);
                    signal.0 += dx / distance * 3.;
                    signal.1 += dz / distance * 3.;
                }
            }
            if !directional_sensing {
                signal = (0., 0.);
            }
            let signal_size = signal.0.hypot(signal.1).max(1.);
            signal.0 /= signal_size;
            signal.1 /= signal_size;
            if o.cells > 1 {
                o.body.sync(
                    o.cells as usize,
                    &g,
                    &o.development,
                    o.x,
                    o.z,
                    o.heading,
                    self.config.seed,
                    false,
                );
                // Colony locomotion comes solely from internal contraction and
                // environmental reaction forces; mobility supplies no thrust.
                let before = o.body.center();
                let work = o.body.advance(
                    DT,
                    &g,
                    &o.development,
                    self.config.seed,
                    false,
                    signal,
                    (fx, fz),
                );
                let center = o.body.center();
                o.x = center.x;
                o.z = center.z;
                o.vx = (center.x - before.x) / DT;
                o.vz = (center.z - before.z) / DT;
                if o.regulation.is_some() {
                    o.body.pay_motor_work(work);
                    o.energy = o.body.reserve_energy();
                    if o.body.differentiations > 0 {
                        self.milestone("regulation","tissue","Local food, stress and neighboring signals changed cell functions through inherited regulatory rules.");
                    }
                    if o.body.repair_work > 0.1 {
                        self.milestone("repair","tissue","A colony spent stored cellular energy repairing tissue and physical connections.");
                    }
                } else {
                    o.energy -= work;
                }
                if o.body.nodes.iter().any(|node| {
                    !node.p.x.is_finite() || !node.p.y.is_finite() || !node.p.z.is_finite()
                }) {
                    o.alive = false;
                }
                if o.body.links.iter().any(|l| l.kind == 2)
                    || o.body.joints.iter().any(|j| j.work > 0.)
                {
                    self.milestone(
                        "actuation",
                        "motion",
                        "A differentiated colony contracts physical links between cells.",
                    );
                }
                if o.body.steps > 8
                    && o.body.path > 0.5
                    && o.body.contacts > 0
                    && o.body.work > 0.5
                    && (o.body.links.iter().any(|l| l.kind == 2)
                        || o.body.joints.iter().any(|j| j.work > 0.))
                {
                    self.milestone("ground-motion","motion","A colony moves through contraction and repeated ground contacts. This is physical locomotion, not a walking unlock.");
                }
            } else {
                let propulsion = g[MOBILITY] * 0.3;
                fx += o.heading.cos() * propulsion;
                fz += o.heading.sin() * propulsion;
                o.vx = o.vx * 0.76 + fx * DT;
                o.vz = o.vz * 0.76 + fz * DT;
                o.x += o.vx * DT;
                o.z += o.vz * DT;
                let radius = o.x.hypot(o.z);
                if radius > 29.0 {
                    o.x *= 29.0 / radius;
                    o.z *= 29.0 / radius;
                    o.vx *= -0.6;
                    o.vz *= -0.6;
                }
            }

            if o.cells > 1
                && g[PREDATION] > 0.02
                && o.feeding_cooldown <= 0.
                && o.energy < 12. + cells * 5.
            {
                for &j in &neighbors {
                    let prey = &self.organisms[j];
                    if !prey.alive || prey.lineage == o.lineage || prey.cells >= o.cells {
                        continue;
                    }
                    // Consume an exposed terminal cell. The body graph remains
                    // contiguous, and actual cell-surface proximity is required.
                    let (point, radius) = if let Some(node) = prey.body.nodes.last() {
                        (node.p, node.radius)
                    } else {
                        let radius = 0.25 + prey.genes[SHAPE] * 0.1;
                        (
                            body::V3::new(
                                prey.x,
                                body::habitat_y(prey.x, prey.z, radius, self.config.seed),
                                prey.z,
                            ),
                            radius,
                        )
                    };
                    let contact = o.body.nodes.iter().enumerate().find_map(|(index, node)| {
                        let dx = node.p.x - point.x;
                        let dy = node.p.y - point.y;
                        let dz = node.p.z - point.z;
                        let eligible = o.regulation.is_none() || node.role == 0 || node.role == 4;
                        (eligible
                            && node.health > 0.25
                            && dx * dx + dy * dy + dz * dz < (node.radius + radius + 0.1).powi(2))
                        .then_some(index)
                    });
                    let Some(feeder) = contact else {
                        continue;
                    };
                    let defense = prey.genes[ARMOR] * 0.55
                        + prey.genes[SKELETON]
                            * if prey.cells >= 4 {
                                prey.genes[SPECIALIZATION] * 0.3
                            } else {
                                0.
                            };
                    let advantage = (o.cells - prey.cells) as f32 / o.cells as f32;
                    let probability = DT
                        * (0.12 + g[PREDATION] * 2.)
                        * advantage
                        * (1. - defense).max(0.1)
                        * if o.regulation.is_some() {
                            o.body.nodes[feeder].health
                                * if o.body.nodes[feeder].role == 4 {
                                    1.6
                                } else {
                                    0.7
                                }
                        } else {
                            1.
                        };
                    if self.random() >= probability {
                        continue;
                    }
                    let prey = &mut self.organisms[j];
                    let stored = if prey.regulation.is_some() && prey.cells > 1 {
                        prey.body.nodes.last().unwrap().reserve
                    } else {
                        prey.energy.max(0.) / prey.cells as f32
                    };
                    let food = 2.4 + stored;
                    let efficiency = 0.25 + g[PREDATION] * 0.55;
                    o.energy += food * efficiency;
                    if o.regulation.is_some() {
                        o.body.nodes[feeder].reserve += food * efficiency;
                    }
                    o.meals += 1;
                    self.feedings += 1;
                    if let Some(c) = &mut self.competition {
                        if let Some(f) = c.families.iter_mut().find(|f| f.id == o.family) {
                            f.meals += 1;
                        }
                    }
                    o.feeding_cooldown = 2. + (1. - g[PREDATION]) * 4.;
                    prey.energy -= stored;
                    self.field[field_index(point.x, point.z)] += food * (1. - efficiency) / 3.;
                    if prey.cells == 1 {
                        living_cells = living_cells.saturating_sub(1);
                        prey.alive = false;
                        self.deaths += 1;
                    } else {
                        living_cells = living_cells.saturating_sub(1);
                        if prey.regulation.is_some() {
                            let terminal = prey.cells as usize - 1;
                            let adjacent: Vec<_> = prey
                                .body
                                .links
                                .iter()
                                .filter(|l| l.purpose == 0 && (l.a == terminal || l.b == terminal))
                                .map(|l| if l.a == terminal { l.b } else { l.a })
                                .collect();
                            for cell in adjacent {
                                prey.body.injure(cell, 0.2);
                            }
                        }
                        prey.cells -= 1;
                        prey.body.sync(
                            prey.cells as usize,
                            &prey.genes,
                            &prey.development,
                            prey.x,
                            prey.z,
                            prey.heading,
                            self.config.seed,
                            false,
                        );
                        let center = prey.body.center();
                        prey.x = center.x;
                        prey.z = center.z;
                        // The final surviving cell immediately returns to the
                        // single-cell representation, even if prey already ticked.
                        if prey.cells == 1 {
                            prey.body = Body::default();
                        }
                    }
                    self.milestone("ingestion","feeding","A larger colony consumed a smaller organism's exposed cell. Digested energy can fund division; remaining material returns to the habitat.");
                    break;
                }
            }
            let lifespan = 170.0 + g[ARMOR] * 230.0 + specialized * 80.0;
            if !o.alive || o.energy <= 0.0 || o.age > lifespan || self.config.solvent < 0.05 {
                living_cells = living_cells.saturating_sub(o.cells as usize);
                o.alive = false;
                self.deaths += 1;
                self.field[idx] += cells * 0.65;
                self.organisms[i] = o;
                continue;
            }
            let reproductive = if o.regulation.is_some() {
                o.body
                    .nodes
                    .iter()
                    .filter(|n| n.role == 5 && n.health > 0.5)
                    .count() as f32
                    / cells
            } else {
                0.
            };
            let threshold =
                (10.0 + cells * 2.2) * (1.25 - (g[FERTILITY] + reproductive * 0.3).min(1.) * 0.4);
            if o.energy > threshold && o.cooldown <= 0.0 {
                let max_cells = o
                    .regulation
                    .as_ref()
                    .map(|p| p.target_cells(&g, &o.body).min(MAX_BODY_CELLS) as u32)
                    .unwrap_or(
                        ((2.0 + g[ADHESION] * 8.0 + g[SKELETON] * specialized * 14.0) as u32)
                            .min(24),
                    );
                let retention = o
                    .regulation
                    .as_ref()
                    .map(|p| g[ADHESION] * (0.55 + p.growth[1] * 0.4))
                    .unwrap_or(g[ADHESION] * 0.78);
                if o.cells < max_cells && living_cells < CELL_BUDGET && self.random() < retention {
                    // Daughter retention grows a body from actual cell division.
                    o.cells += 1;
                    living_cells += 1;
                    o.energy -= 3.0 + cells * 0.4;
                    o.cooldown = 3.0;
                    self.milestone(
                        "colony",
                        "colony",
                        "A daughter cell stayed attached. The first colony formed.",
                    );
                    if o.cells >= 4 && specialized > 0.35 {
                        self.milestone(
                            "roles",
                            "colony",
                            "A colony expresses distinct cell functions.",
                        );
                    }
                    if o.cells >= 4 && g[MUSCLE] > 0.55 {
                        self.milestone(
                            "muscles",
                            "colony",
                            "A growing colony has inherited the potential for contractile tissue.",
                        );
                    }
                    if o.cells >= 6 && g[SKELETON] > 0.55 {
                        self.milestone(
                            "skeleton",
                            "colony",
                            "Structural cells support a larger body plan.",
                        );
                    }
                } else if n + newborns.len() < self.config.capacity
                    && (o.cells > 3 || living_cells < CELL_BUDGET)
                {
                    let mut mate = None;
                    if g[SEX] > 0.5 {
                        for &j in &neighbors {
                            let partner = &self.organisms[j];
                            if partner.alive
                                && partner.energy > threshold * 0.55
                                && partner.cooldown <= 0.0
                                && partner.genes[SEX] > 0.5
                                && genome_distance(&g, &partner.genes) < 0.07
                                && (partner.x - o.x).hypot(partner.z - o.z) < 1.8
                            {
                                mate = Some(j);
                                break;
                            }
                        }
                    }
                    // High-sex genomes wait for a mate; mixed strategies can still divide.
                    if mate.is_some() || g[SEX] < 0.8 {
                        let mut child = o.clone();
                        child.id = self.next_id;
                        self.next_id += 1;
                        child.parent = o.id;
                        child.mate = 0;
                        child.generation += 1;
                        child.age = 0.0;
                        child.feeding_cooldown = 0.;
                        child.meals = 0;
                        child.offspring = 0;
                        child.cooldown = 5.0;
                        child.vx *= -0.5;
                        child.vz *= -0.5;
                        child.x = (o.x + (self.random() - 0.5) * 0.8).clamp(-29.0, 29.0);
                        child.z = (o.z + (self.random() - 0.5) * 0.8).clamp(-29.0, 29.0);
                        child.cells = if o.cells > 3 { (o.cells / 3).max(1) } else { 1 };
                        child.body = Body::default();
                        if let Some(j) = mate {
                            child.mate = self.organisms[j].id;
                            for k in 0..GENE_COUNT {
                                if self.random() < 0.5 {
                                    child.genes[k] = self.organisms[j].genes[k];
                                }
                            }
                            for k in 0..DEVELOPMENT {
                                if self.random() < 0.5 {
                                    child.development[k] = self.organisms[j].development[k];
                                }
                            }
                            let partner_anatomy = self.organisms[j].anatomy.clone();
                            if let (Some(child_anatomy), Some(partner_anatomy)) =
                                (&mut child.anatomy, partner_anatomy)
                            {
                                for (target, source) in
                                    child_anatomy.values_mut().zip(partner_anatomy.values())
                                {
                                    if self.random() < 0.5 {
                                        *target = *source;
                                    }
                                }
                            }
                            let partner_program = self.organisms[j].regulation.clone();
                            if let (Some(child_program), Some(partner_program)) =
                                (&mut child.regulation, partner_program)
                            {
                                for (target, source) in child_program
                                    .rules
                                    .iter_mut()
                                    .flatten()
                                    .chain(child_program.growth.iter_mut())
                                    .zip(
                                        partner_program
                                            .rules
                                            .iter()
                                            .flatten()
                                            .chain(&partner_program.growth),
                                    )
                                {
                                    if self.random() < 0.5 {
                                        *target = *source;
                                    }
                                }
                            }
                            self.organisms[j].energy *= 0.78;
                            if self.organisms[j].regulation.is_some() && self.organisms[j].cells > 1
                            {
                                let energy = self.organisms[j].energy;
                                self.organisms[j].body.balance_reserves(energy);
                            }
                            self.organisms[j].cooldown = 6.0;
                            self.mating += 1;
                            self.organisms[j].offspring += 1;
                            self.milestone(
                                "sex",
                                "sex",
                                "Compatible neighbors exchanged hereditary material.",
                            );
                        }
                        for gene in &mut child.genes {
                            if self.random()
                                < self.config.mutation * (1.0 + self.config.radiation * 0.6)
                            {
                                *gene = bounded(*gene + (self.random() - 0.5) * 0.42);
                            }
                        }
                        for value in &mut child.development {
                            if self.random()
                                < self.config.mutation * (1. + self.config.radiation * 0.6)
                            {
                                *value = bounded(*value + (self.random() - 0.5) * 0.42);
                            }
                        }
                        if let Some(program) = &mut child.regulation {
                            for value in program
                                .rules
                                .iter_mut()
                                .flatten()
                                .chain(program.growth.iter_mut())
                            {
                                if self.random()
                                    < self.config.mutation * (1. + self.config.radiation * 0.6)
                                {
                                    *value = bounded(*value + (self.random() - 0.5) * 0.42);
                                }
                            }
                        }
                        if let Some(anatomy) = &mut child.anatomy {
                            for value in anatomy.values_mut() {
                                if self.random()
                                    < self.config.mutation * (1. + self.config.radiation * 0.6)
                                {
                                    *value = bounded(*value + (self.random() - 0.5) * 0.42);
                                }
                            }
                        }
                        let ancestor = self.lineages[&o.lineage].ancestor;
                        if genome_distance(&child.genes, &ancestor) > 0.018 {
                            child.lineage =
                                self.new_lineage(child.genes, o.lineage, child.generation);
                        }
                        child.energy = o.energy * 0.42;
                        o.energy *= 0.48;
                        o.cooldown = 4.0;
                        if o.cells > 3 {
                            o.cells -= child.cells;
                        } else {
                            living_cells += 1;
                        }
                        self.births += 1;
                        if child.cells > 1 {
                            child.body.regulated = child.regulation.is_some();
                            child.body.anatomy = child.anatomy.clone();
                            child.body.sync(
                                child.cells as usize,
                                &child.genes,
                                &child.development,
                                child.x,
                                child.z,
                                child.heading,
                                self.config.seed,
                                false,
                            );
                            let center = child.body.center();
                            child.x = center.x;
                            child.z = center.z;
                            if child.regulation.is_some() {
                                child.body.balance_reserves(child.energy);
                            }
                        }
                        o.offspring += 1;
                        if let Some(c) = &mut self.competition {
                            c.birth(&child, self.time);
                        }
                        newborns.push(child);
                    }
                }
            }
            if o.cells > 1 {
                o.body.regulated = o.regulation.is_some();
                o.body.anatomy = o.anatomy.clone();
                o.body.sync(
                    o.cells as usize,
                    &o.genes,
                    &o.development,
                    o.x,
                    o.z,
                    o.heading,
                    self.config.seed,
                    false,
                );
                let center = o.body.center();
                o.x = center.x;
                o.z = center.z;
                if o.regulation.is_some() {
                    o.body.balance_reserves(o.energy);
                }
            } else {
                o.body = Body::default();
            }
            self.organisms[i] = o;
        }
        if let Some(c) = &mut self.competition {
            for o in self.organisms.iter().filter(|o| !o.alive) {
                if let Some(f) = c.families.iter_mut().find(|f| f.id == o.family) {
                    f.deaths += 1;
                }
            }
        }
        self.organisms.retain(|o| o.alive);
        self.organisms.extend(newborns);
        if let Some(c) = &mut self.competition {
            c.peak_jointed = c.peak_jointed.max(
                self.organisms
                    .iter()
                    .filter(|o| o.body.joints.iter().any(|j| j.enabled))
                    .count(),
            );
            c.largest_body = c
                .largest_body
                .max(self.organisms.iter().map(|o| o.cells).max().unwrap_or(0));
        }
        if n > 0 && self.organisms.is_empty() {
            self.event(
                "extinction",
                if self.competition.is_some() {"The competing population went extinct. No replacement founders are introduced automatically.".into()} else {"The last organism died. The basin is lifeless; chemistry may begin again.".into()},
            );
        }
        self.history_clock += DT;
        if self.history_clock >= 3.0 {
            self.history_clock = 0.0;
            self.record_history();
        }
    }
    fn counts(&self) -> BTreeMap<u32, usize> {
        let mut counts = BTreeMap::new();
        for o in &self.organisms {
            *counts.entry(o.lineage).or_insert(0) += 1;
        }
        counts
    }
    fn record_history(&mut self) {
        let counts = self.counts();
        for (&id, &count) in &counts {
            if let Some(l) = self.lineages.get_mut(&id) {
                l.peak = l.peak.max(count);
            }
        }
        self.history.push(History {
            time: self.time,
            population: self.organisms.len(),
            cells: self.organisms.iter().map(|o| o.cells).sum(),
            colonies: self.organisms.iter().filter(|o| o.cells > 1).count(),
            diversity: counts.len(),
        });
        if self.history.len() > 240 {
            self.history.remove(0);
        }
    }
    pub fn metadata(&self, selected: u32) -> String {
        if let Some(e) = &self.autonomous {
            return e.metadata(self, selected);
        }
        let counts = self.counts();
        let mut representatives = BTreeMap::new();
        let mut means = [0.0; GENE_COUNT];
        let mut cells = 0;
        let mut colonies = 0;
        let mut generation = 0;
        for o in &self.organisms {
            representatives.entry(o.lineage).or_insert(o.id);
            cells += o.cells;
            if o.cells > 1 {
                colonies += 1;
            }
            generation = generation.max(o.generation);
            for (k, v) in means.iter_mut().enumerate() {
                *v += o.genes[k];
            }
        }
        for v in &mut means {
            *v /= self.organisms.len().max(1) as f32;
        }
        let mut lineages: Vec<_> = counts
            .iter()
            .map(|(id, count)| {
                let mut value = serde_json::to_value(&self.lineages[id]).unwrap();
                value["count"] = (*count).into();
                value["representative"] = representatives[id].into();
                value
            })
            .collect();
        lineages.sort_by_key(|l| std::cmp::Reverse(l["count"].as_u64().unwrap()));
        let selected = self.organisms.iter().find(|o|o.id==selected).map(|o| {
            serde_json::json!({"id":o.id,"lineage":o.lineage,"name":self.lineages[&o.lineage].name,
                "generation":o.generation,"age":o.age,"cells":o.cells,"energy":o.energy,"genes":o.genes,
                "parent":o.parent,"mate":o.mate,"family":o.family,"offspring":o.offspring,"x":o.x,"z":o.z,
                "development":o.development,"regulation":o.regulation,"anatomy":o.anatomy,"body":o.body,"meals":o.meals,"feedingCooldown":o.feeding_cooldown})
        });
        serde_json::json!({"config":self.config,"stats":{"time":self.time,"population":self.organisms.len(),
            "cells":cells,"colonies":colonies,"diversity":counts.len(),"generation":generation,"births":self.births,
            "deaths":self.deaths,"mating":self.mating,"feedings":self.feedings,"meanGenes":means,"nutrient":self.field.iter().sum::<f32>() / self.field.len() as f32,
            "capped":self.organisms.len()>=self.config.capacity,"computeMs":0,
            "cellBudget":CELL_BUDGET,"cellBudgetReached":cells as usize>=CELL_BUDGET,
            "regulated":self.organisms.iter().filter(|o|o.regulation.is_some()).count(),
            "injured":self.organisms.iter().filter(|o|o.body.nodes.iter().any(|n|n.health<0.8)).count(),
            "articulated":self.organisms.iter().filter(|o|o.body.links.iter().any(|l|l.kind==1)).count(),
            "jointed":self.organisms.iter().filter(|o|o.body.joints.iter().any(|j|j.enabled)).count(),
            "contractile":self.organisms.iter().filter(|o|o.body.links.iter().any(|l|l.kind==2)||o.body.joints.iter().any(|j|j.work>0.)).count()},
            "lineages":lineages,"events":self.events,"history":self.history,"selected":selected,
            "competition":self.competition.as_ref().map(|c|c.metadata(self)),"study":self.study,"lab":self.lab.as_ref().map(|l|l.metadata(self))}).to_string()
    }
    pub fn render_data(&self) -> Vec<f32> {
        if let Some(e) = &self.autonomous {
            let display = Core {
                organisms: e.project(),
                autonomous: None,
                ..Core::new(Config {
                    origin: "chemistry".into(),
                    ..self.config.clone()
                })
                .unwrap()
            };
            return display.render_data();
        }
        let mut data = Vec::with_capacity(self.organisms.len() * 12);
        for o in &self.organisms {
            data.extend_from_slice(&[
                o.x,
                o.z,
                o.heading,
                o.cells as f32,
                o.genes[SHAPE],
                o.genes[PIGMENT],
                o.genes[PHOTO],
                o.genes[CHEMO],
                o.genes[BRANCHING],
                o.genes[SPECIALIZATION],
                o.genes[MUSCLE],
                o.genes[SKELETON],
            ]);
        }
        data
    }
    // Transfer-friendly global particle records: organism index, xyz, radius, role, contact,
    // health, reserve, and for a rod its length in widths and the axis it lies along.
    pub fn cell_data(&self) -> Vec<f32> {
        if let Some(e) = &self.autonomous {
            let display = Core {
                organisms: e.project(),
                autonomous: None,
                ..Core::new(Config {
                    origin: "chemistry".into(),
                    ..self.config.clone()
                })
                .unwrap()
            };
            return display.cell_data();
        }
        let mut out = Vec::new();
        for (i, o) in self.organisms.iter().enumerate() {
            for n in &o.body.nodes {
                out.extend_from_slice(&[
                    i as f32,
                    n.p.x,
                    n.p.y,
                    n.p.z,
                    n.radius,
                    n.role as f32,
                    if n.contact { 1. } else { 0. },
                    n.health,
                    n.reserve,
                    n.long[0],
                    n.long[1],
                    n.long[2],
                    n.long[3],
                ]);
            }
        }
        out
    }
    // Link endpoints reference the packed particle buffer; kind and activation follow.
    pub fn link_data(&self) -> Vec<f32> {
        if let Some(e) = &self.autonomous {
            let display = Core {
                organisms: e.project(),
                autonomous: None,
                ..Core::new(Config {
                    origin: "chemistry".into(),
                    ..self.config.clone()
                })
                .unwrap()
            };
            return display.link_data();
        }
        let mut out = Vec::new();
        let mut offset = 0;
        for o in &self.organisms {
            for l in &o.body.links {
                out.extend_from_slice(&[
                    (offset + l.a) as f32,
                    (offset + l.b) as f32,
                    l.kind as f32,
                    l.activation,
                    l.strain,
                    l.integrity,
                ]);
            }
            offset += o.body.nodes.len();
        }
        out
    }
    pub fn joint_data(&self) -> Vec<f32> {
        if let Some(e) = &self.autonomous {
            let display = Core {
                organisms: e.project(),
                autonomous: None,
                ..Core::new(Config {
                    origin: "chemistry".into(),
                    ..self.config.clone()
                })
                .unwrap()
            };
            return display.joint_data();
        }
        let mut data = Vec::new();
        let mut offset = 0;
        for o in &self.organisms {
            for j in &o.body.joints {
                if !j.enabled {
                    continue;
                }
                data.extend_from_slice(&[
                    (offset + j.a) as f32,
                    (offset + j.b) as f32,
                    (offset + j.c) as f32,
                    j.min,
                    j.max,
                    j.angle,
                    j.flexion,
                    j.extension,
                    j.limit_error,
                ]);
            }
            offset += o.body.nodes.len();
        }
        data
    }
    pub fn start_lab(&mut self) {
        let config = Config {
            seed: self.config.seed,
            kind: "lab".into(),
            name: "The locomotion laboratory".into(),
            origin: "chemistry".into(),
            ..Config::default()
        };
        *self = Self::new(config).unwrap();
        let lab = Experiment::default();
        lab.spawn(self);
        self.lab = Some(lab);
        self.event("world","Controlled locomotion assay: a supplied single-cell precursor develops under gravity. Morphology and motor programs vary; net travel minus work determines artificial selection.".into());
    }
    pub fn start_joint_lab(&mut self) {
        self.start_lab();
        self.config.name = "The jointed locomotion laboratory".into();
        let mut lab = Experiment::default();
        lab.jointed = true;
        lab.program.anatomy = Some(Anatomy::default());
        lab.elite = lab.program.clone();
        lab.spawn(self);
        self.lab = Some(lab);
        self.event("world","Jointed locomotion assay: supplied single-cell precursor, mineralized segments, angular limits and paired motors controlled by heritable recurrent units. Selection measures travel after growth and settling; it does not certify walking.".into());
    }
    pub fn start_tissue_study(&mut self) {
        let config = Config {
            seed: self.config.seed,
            name: "The tissue ecology study".into(),
            origin: "chemistry".into(),
            sunlight: 0.9,
            nutrients: 0.9,
            minerals: 0.6,
            solvent: 1.,
            temperature: 0.48,
            radiation: 0.1,
            mutation: 0.04,
            capacity: 1000,
            ..Config::default()
        };
        *self = Self::new(config).unwrap();
        self.study = true;
        self.seed(80);
        let mut genes = self.organisms[0].genes;
        for (index, value) in [
            (PHOTO, 0.85),
            (CHEMO, 0.8),
            (ADHESION, 0.99),
            (MUSCLE, 0.8),
            (SKELETON, 0.9),
            (SPECIALIZATION, 0.9),
            (SENSING, 0.7),
            (FERTILITY, 0.7),
            (SEX, 0.1),
            (PREDATION, 0.65),
            (THERMAL, 0.48),
        ] {
            genes[index] = value;
        }
        let lineage = self.new_lineage(genes, 0, 0);
        let precursor = &mut self.organisms[0];
        precursor.genes = genes;
        precursor.lineage = lineage;
        precursor.development = default_development();
        precursor.regulation = Some(Regulation::default());
        precursor.x = 0.;
        precursor.z = 0.;
        precursor.heading = 0.;
        precursor.energy = 9.;
        precursor.cooldown = 0.;
        self.event("world","Controlled tissue study: 80 single cells, including one supplied precursor with strong tissue potentials. Growth, feeding, transport, damage, repair and reproduction use ecosystem rules; no adult is introduced.".into());
    }
    pub fn load(json: &str) -> Result<Self, String> {
        let mut core: Self =
            serde_json::from_str(json).map_err(|_| "Invalid Primordia save file")?;
        core.config.validate()?;
        if core.autonomous.as_ref().is_some_and(|e| {
            !e.valid()
                || e.physical.as_ref().is_some_and(|h| {
                    h.passive != (core.config.physical_assay == "passive")
                        || (h.domain.size() != core.config.world_size as f32
                            || h.domain.shape != core.config.world_shape)
                })
                || core.lab.is_some()
                || core.competition.is_some()
                || core.study
                || !core.organisms.is_empty()
        }) {
            return Err("Invalid autonomous cellular state".into());
        }
        if ![1, 2, 3, 4].contains(&core.version)
            || core.field.len() != GRID * GRID
            || core.organisms.len() > MAX_POPULATION
            || core.organisms.iter().map(|o| o.cells as u64).sum::<u64>() > CELL_BUDGET as u64
            || core.lineages.len() > 100000
            || core.history.len() > 240
            || core.events.len() > 60
            || !core.time.is_finite()
            || core.time < 0.0
            || !core.history_clock.is_finite()
            || core
                .competition
                .as_ref()
                .is_some_and(|c| !c.valid(core.next_id) || core.lab.is_some() || core.study)
            || core.lab.as_ref().is_some_and(|l| !l.valid())
            || (core.lab.is_some() && (core.organisms.len() != 1 || core.config.kind != "lab"))
            || (core.config.kind == "lab" && core.lab.is_none())
            || core
                .field
                .iter()
                .any(|v| !v.is_finite() || *v < 0.0 || *v > 100000.0)
            || core.next_id <= core.organisms.iter().map(|o| o.id).max().unwrap_or(0)
            || core.next_lineage <= core.lineages.keys().max().copied().unwrap_or(0)
        {
            return Err("Save exceeds simulation limits".into());
        }
        let mut ids = BTreeSet::new();
        for o in &core.organisms {
            if core
                .competition
                .as_ref()
                .is_some_and(|c| o.family != 0 && !c.families.iter().any(|f| f.id == o.family))
                || !ids.insert(o.id)
                || !core.lineages.contains_key(&o.lineage)
                || !(1..=MAX_BODY_CELLS as u32).contains(&o.cells)
                || o.regulation.as_ref().is_some_and(|p| !p.valid())
                || o.anatomy.as_ref().is_some_and(|p| !p.valid())
                || (core.version >= 4
                    && o.body.anatomy
                        != if !o.body.nodes.is_empty() {
                            o.anatomy.clone()
                        } else {
                            None
                        })
                || !o
                    .genes
                    .iter()
                    .all(|g| g.is_finite() && (0.0..=1.0).contains(g))
                || !o
                    .development
                    .iter()
                    .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
                || !o.body.valid(o.cells as usize)
                || (core.version >= 2 && o.cells > 1 && o.body.nodes.len() != o.cells as usize)
                || (core.version >= 3
                    && o.body.regulated != (o.cells > 1 && o.regulation.is_some()))
                || [
                    o.x,
                    o.z,
                    o.vx,
                    o.vz,
                    o.heading,
                    o.energy,
                    o.age,
                    o.cooldown,
                    o.feeding_cooldown,
                ]
                .iter()
                .any(|v| !v.is_finite())
                || o.x.abs() > 32.0
                || o.z.abs() > 32.0
                || o.energy.abs() > 100000.0
                || o.vx.abs() > 100.0
                || o.vz.abs() > 100.0
            {
                return Err("Save contains invalid organisms".into());
            }
        }
        for (&id, l) in &core.lineages {
            if id != l.id
                || l.name.len() > 100
                || l.color.len() != 7
                || !l.color.starts_with('#')
                || !l.color[1..].bytes().all(|b| b.is_ascii_hexdigit())
                || l.ancestor
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            {
                return Err("Save contains invalid lineages".into());
            }
        }
        let legacy = core.version == 1;
        core.version = 4;
        // Older version-one saves acquire developmental defaults. Hydrate once on
        // load; all subsequent positions, controllers and RNG are saved exactly.
        if legacy && core.lab.is_none() {
            for o in &mut core.organisms {
                if o.cells > 1 && o.body.nodes.is_empty() {
                    o.body.sync(
                        o.cells as usize,
                        &o.genes,
                        &o.development,
                        o.x,
                        o.z,
                        o.heading,
                        core.config.seed,
                        false,
                    );
                }
            }
        }
        Ok(core)
    }
}
// Float rounding must not turn a half-open random interval into an array's
// one-past-the-end index during a long evolutionary run.
fn unit_fraction(value: u32) -> f32 {
    ((value as f64 / 4294967296.0) as f32).min(f32::from_bits(0x3f7fffff))
}

#[wasm_bindgen]
pub struct World {
    core: Core,
}
#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new(json: &str) -> Result<World, JsValue> {
        let config = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Invalid world: {e}")))?;
        Ok(Self {
            core: Core::new(config).map_err(|e| JsValue::from_str(&e))?,
        })
    }
    pub fn step(&mut self, count: u32) {
        self.core.step(count);
    }
    pub fn seed(&mut self, count: usize) {
        self.core.seed(count);
    }
    pub fn pulse(&mut self) {
        self.core.pulse();
    }
    pub fn intervene(&mut self, json: &str) -> Result<String, JsValue> {
        self.core.intervene(json).map_err(|e| JsValue::from_str(&e))
    }
    /// Creator steering: mutation rate, breeding goals, favoured lineages and
    /// disasters. Changes selection and variation, never genes or materials.
    pub fn steer(&mut self, json: &str) -> Result<String, JsValue> {
        self.core.steer(json).map_err(|e| JsValue::from_str(&e))
    }
    /// The leading living body on each measure, as a JSON list.
    pub fn champions(&self) -> String {
        self.core.champions()
    }
    /// One founder of a starter species, as an organism snapshot.
    pub fn species_specimen(&self, species: &str) -> Result<String, JsValue> {
        self.core
            .species_specimen(species)
            .map(|s| s.1)
            .map_err(|e| JsValue::from_str(&e))
    }
    /// Scatter copies of an organism around painted points; returns how many fitted.
    pub fn paint_life(
        &mut self,
        json: &str,
        points: &[f32],
        radius: f32,
        per_point: u32,
    ) -> Result<u32, JsValue> {
        self.core
            .paint_life(json, points, radius, per_point)
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn founder_specimen(&self) -> Result<String, JsValue> {
        self.core.founder_specimen().map_err(|e| JsValue::from_str(&e))
    }
    pub fn configure(&mut self, json: &str) -> Result<(), JsValue> {
        if self.core.autonomous.is_some() {
            return Err(JsValue::from_str("Autonomous conditions are fixed for this run. Create a new world to change the habitat."));
        }
        let mut value = serde_json::to_value(&self.core.config).unwrap();
        let patch: serde_json::Value =
            serde_json::from_str(json).map_err(|_| JsValue::from_str("Invalid parameters"))?;
        if let Some(map) = patch.as_object() {
            for (k, v) in map {
                value[k] = v.clone();
            }
        }
        let config: Config =
            serde_json::from_value(value).map_err(|_| JsValue::from_str("Invalid parameters"))?;
        config.validate().map_err(|e| JsValue::from_str(&e))?;
        if self
            .core
            .autonomous
            .as_ref()
            .and_then(|e| e.physical.as_ref())
            .is_some_and(|h| {
                h.domain.size() != config.world_size as f32 || h.domain.shape != config.world_shape
            })
        {
            return Err(JsValue::from_str(
                "Habitat size applies to new worlds; existing material is not rescaled",
            ));
        }
        self.core.config = config;
        Ok(())
    }
    /// Engineering budget only: does not change chemistry, genes or timestep.
    pub fn set_cell_budget(&mut self, capacity: usize) -> Result<(), JsValue> {
        if !(100..=autonomous::MAX_CELLS).contains(&capacity) {
            return Err(JsValue::from_str("Living-cell budget must be 100–64000"));
        }
        let Some(e) = self.core.autonomous.as_ref() else {
            return Err(JsValue::from_str(
                "Cell budget applies to the physical habitat",
            ));
        };
        if capacity < e.cells.len().max(self.core.config.initial_cells) {
            return Err(JsValue::from_str(
                "Budget must fit the living population and starting population",
            ));
        }
        self.core.config.capacity = capacity;
        Ok(())
    }
    pub fn metadata(&self, selected: u32) -> String {
        self.core.metadata(selected)
    }
    /// Reuse the body projection already prepared for this display snapshot.
    pub fn display_metadata(&self, selected: u32, display: &World) -> String {
        if let Some(e) = &self.core.autonomous {
            e.metadata_projected(&self.core, selected, &display.core.organisms)
        } else { self.core.metadata(selected) }
    }
    pub fn display_world(&self) -> World {
        let core = if let Some(e) = &self.core.autonomous {
            Core {
                organisms: e.project(),
                autonomous: None,
                ..Core::new(Config {
                    origin: "chemistry".into(),
                    ..self.core.config.clone()
                })
                .unwrap()
            }
        } else {
            self.core.clone()
        };
        World { core }
    }
    pub fn render_data(&self) -> Vec<f32> {
        self.core.render_data()
    }
    pub fn cell_data(&self) -> Vec<f32> {
        self.core.cell_data()
    }
    pub fn link_data(&self) -> Vec<f32> {
        self.core.link_data()
    }
    pub fn joint_data(&self) -> Vec<f32> {
        self.core.joint_data()
    }
    pub fn start_lab(&mut self) {
        self.core.start_lab();
    }
    pub fn start_joint_lab(&mut self) {
        self.core.start_joint_lab();
    }
    pub fn start_tissue_study(&mut self) {
        self.core.start_tissue_study();
    }
    pub fn start_autonomous(&mut self) {
        self.core.start_autonomous();
    }
    pub fn observation_control(&mut self, control: &str) -> Result<(), JsValue> {
        if !["baseline", "motors", "transport", "signals"].contains(&control) {
            return Err(JsValue::from_str("Invalid comparison control"));
        }
        let Some(h) = self
            .core
            .autonomous
            .as_mut()
            .and_then(|e| e.physical.as_mut())
        else {
            return Err(JsValue::from_str("A physical world is required"));
        };
        h.ablation = if control == "baseline" {
            String::new()
        } else {
            control.into()
        };
        h.passive = control == "motors";
        self.core.config.physical_assay = if h.passive {
            "passive".into()
        } else {
            "full".into()
        };
        Ok(())
    }
    pub fn simulation_time(&self) -> f32 {
        self.core.time
    }
    pub fn start_physical(&mut self) {
        self.core.start_physical();
    }
    pub fn capture_specimen(&self, selected:u32, name:&str)->Result<String,JsValue>{
        self.core.capture_specimen(selected,name).map_err(|e|JsValue::from_str(&e))
    }
    pub fn inspect_specimen(&self,json:&str)->Result<String,JsValue>{
        specimens::inspect(json).map_err(|e|JsValue::from_str(&e))
    }
    pub fn place_specimen(&mut self,json:&str,x:f32,z:f32)->Result<u32,JsValue>{
        self.core.place_specimen(json,x,z).map_err(|e|JsValue::from_str(&e))
    }
    pub fn start_population(&mut self, count: u32) {
        self.core.start_population(count);
    }
    pub fn seed_competitors(&mut self, count: u32) {
        self.core.seed_competitors(count);
    }
    pub fn set_directional_sensing(&mut self, enabled: bool) {
        if let Some(c) = &mut self.core.competition {
            c.directional_sensing = enabled;
        }
    }
    pub fn replay_best(&mut self, enabled: bool) {
        if let Some(mut lab) = self.core.lab.take() {
            lab.replay(&mut self.core, enabled);
            self.core.lab = Some(lab);
        }
    }
    pub fn set_motors_enabled(&mut self, enabled: bool) {
        if let Some(mut lab) = self.core.lab.take() {
            lab.set_motors(&mut self.core, enabled);
            self.core.lab = Some(lab);
        }
    }
    pub fn ids(&self) -> Vec<u32> {
        if let Some(e) = &self.core.autonomous {
            return e.groups().iter().map(|g| e.cells[g[0]].id).collect();
        }
        self.core.organisms.iter().map(|o| o.id).collect()
    }
    pub fn field(&self) -> Vec<f32> {
        self.core
            .autonomous
            .as_ref()
            .and_then(|e| e.physical.as_ref())
            .map_or_else(
                || self.core.field.clone(),
                |h| h.solutes.iter().map(|s| s[0] as f32).collect(),
            )
    }
    pub fn save(&self) -> String {
        serde_json::to_string(&self.core).unwrap()
    }
    pub fn load(&mut self, json: &str) -> Result<(), JsValue> {
        self.core = Core::load(json).map_err(|e| JsValue::from_str(&e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn feeding_world(prey_cells: u32, kin: bool) -> Core {
        let mut a = Core::new(Config::default()).unwrap();
        let mut prey = a.organisms[20].clone();
        a.organisms.truncate(1);
        a.config.nutrients = 0.;
        a.config.sunlight = 0.;
        a.config.minerals = 0.;
        a.config.radiation = 0.;
        a.config.mutation = 0.;
        a.field.fill(0.);
        let predator = &mut a.organisms[0];
        predator.regulation = None;
        predator.anatomy = None;
        predator.genes.fill(0.);
        predator.genes[THERMAL] = a.config.temperature;
        predator.genes[PREDATION] = 1.;
        predator.genes[SKELETON] = 1.;
        predator.genes[SPECIALIZATION] = 1.;
        predator.genes[ADHESION] = 1.;
        predator.cells = 8;
        predator.energy = 10.;
        predator.cooldown = 1000.;
        predator.x = 0.;
        predator.z = 0.;
        predator.body.sync(
            8,
            &predator.genes,
            &predator.development,
            0.,
            0.,
            0.,
            a.config.seed,
            false,
        );
        for _ in 0..40 {
            predator.body.advance(
                DT,
                &predator.genes,
                &predator.development,
                a.config.seed,
                false,
                (0., 0.),
                (0., 0.),
            );
        }
        let center = predator.body.center();
        predator.x = center.x;
        predator.z = center.z;
        prey.genes.fill(0.);
        prey.regulation = None;
        prey.anatomy = None;
        prey.genes[THERMAL] = a.config.temperature;
        prey.energy = 100.;
        prey.cooldown = 1000.;
        prey.cells = prey_cells;
        prey.body = Body::default();
        if kin {
            prey.lineage = predator.lineage;
        }
        a.organisms.push(prey);
        place_prey_at_contact(&mut a);
        a
    }
    fn place_prey_at_contact(a: &mut Core) {
        if a.organisms.len() < 2 {
            return;
        }
        let point = a.organisms[0].body.nodes[0].p;
        let prey = &mut a.organisms[1];
        prey.x = point.x;
        prey.z = point.z;
        prey.vx = 0.;
        prey.vz = 0.;
        if prey.cells > 1 {
            prey.body.sync(
                prey.cells as usize,
                &prey.genes,
                &prey.development,
                prey.x,
                prey.z,
                0.,
                a.config.seed,
                false,
            );
            let last = prey.body.nodes.last().unwrap().p;
            for node in &mut prey.body.nodes {
                node.p.x += point.x - last.x;
                node.p.y += point.y - last.y;
                node.p.z += point.z - last.z;
                node.v = body::V3::default();
            }
            let center = prey.body.center();
            prey.x = center.x;
            prey.z = center.z;
        }
    }
    #[test]
    fn ingestion_transfers_energy_and_can_fund_retained_division() {
        let mut a = feeding_world(1, false);
        for _ in 0..40 {
            place_prey_at_contact(&mut a);
            a.step(1);
            if a.feedings > 0 {
                break;
            }
        }
        assert_eq!(a.feedings, 1);
        assert_eq!(a.deaths, 1);
        assert_eq!(a.organisms.len(), 1);
        assert_eq!(a.organisms[0].meals, 1);
        assert!(a.organisms[0].energy > 70.);
        assert!(a.field.iter().sum::<f32>() > 5.);
        assert!(a.events.iter().any(|e| e.r#type == "feeding"));
        a.field.fill(0.);
        a.organisms[0].cooldown = 0.;
        a.rng = 0; // Fix the retention choice so this isolates energy-funded growth.
        for _ in 0..80 {
            a.step(1);
            if a.organisms.iter().any(|o| o.cells > 8) {
                break;
            }
        }
        assert!(
            a.organisms.iter().any(|o| o.cells > 8),
            "digestion must fund growth without environmental uptake"
        );
    }
    #[test]
    fn feeding_requires_contact_and_smaller_nonkin_prey() {
        for (count, kin) in [(1, true), (8, false), (12, false)] {
            let mut a = feeding_world(count, kin);
            a.organisms[0].energy = 30.;
            for _ in 0..40 {
                place_prey_at_contact(&mut a);
                a.step(1);
            }
            assert_eq!(a.feedings, 0);
            assert_eq!(a.organisms[1].cells, count);
        }
        let mut a = feeding_world(1, false);
        a.organisms[0].energy = 30.;
        a.organisms[1].x += 10.;
        a.step(40);
        assert_eq!(a.feedings, 0);
    }
    #[test]
    fn eating_a_terminal_cell_preserves_the_prey_body_graph() {
        let mut a = feeding_world(4, false);
        for _ in 0..40 {
            place_prey_at_contact(&mut a);
            a.step(1);
            if a.feedings > 0 {
                break;
            }
        }
        assert_eq!(a.feedings, 1);
        assert_eq!(a.deaths, 0);
        assert_eq!(a.organisms[1].cells, 3);
        assert!(a.organisms[1].body.valid(3));
        Core::load(&serde_json::to_string(&a).unwrap()).unwrap();
    }
    #[test]
    fn ingestion_to_one_cell_is_valid_after_the_prey_already_ticked() {
        let mut a = feeding_world(2, false);
        a.organisms[1].regulation = Some(Regulation::default());
        a.organisms[1].body.regulated = true;
        let prey_energy = a.organisms[1].energy;
        a.organisms[1].body.balance_reserves(prey_energy);
        let predator = a.organisms[0].id;
        a.organisms.swap(0, 1);
        for _ in 0..40 {
            a.organisms.swap(0, 1);
            place_prey_at_contact(&mut a);
            a.organisms.swap(0, 1);
            a.step(1);
            if a.feedings > 0 {
                break;
            }
        }
        assert_eq!(a.feedings, 1);
        let prey = a.organisms.iter().find(|o| o.id != predator).unwrap();
        assert_eq!(prey.cells, 1);
        assert!(prey.body.nodes.is_empty());
        assert!(!prey.body.regulated);
        Core::load(&serde_json::to_string(&a).unwrap()).unwrap();
    }
    #[test]
    fn colonies_use_physical_nodes_and_inherit_development() {
        let mut a = Core::new(Config::default()).unwrap();
        for o in &mut a.organisms {
            o.genes[ADHESION] = 1.;
            o.energy = 100.;
            o.cooldown = 0.;
        }
        a.step(20);
        assert!(a.organisms.iter().any(|o| o.body.nodes.len() > 1));
        assert!(a.organisms.iter().all(|o| o.body.valid(o.cells as usize)));
        assert!(!a.cell_data().is_empty());
        assert!(!a.link_data().is_empty());
        let mut single = Core::new(Config::default()).unwrap();
        single.organisms.truncate(1);
        let mut latent = single.clone();
        latent.organisms[0].genes[MUSCLE] = 1.;
        single.step(1);
        latent.step(1);
        assert_eq!(single.organisms[0].x, latent.organisms[0].x);
    }
    #[test]
    fn laboratory_selects_measured_locomotion_and_resumes_exactly() {
        let mut a = Core::new(Config::default()).unwrap();
        a.start_lab();
        a.step(33);
        let mut b = Core::load(&serde_json::to_string(&a).unwrap()).unwrap();
        a.step(400);
        b.step(400);
        assert_eq!(a.cell_data(), b.cell_data());
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        for _ in 0..6 {
            a.step(400);
        }
        let lab = a.lab.as_ref().unwrap();
        assert!(lab.generation >= 2);
        assert!(lab.champion.as_ref().unwrap().result.viable);
        assert!(lab.champion.as_ref().unwrap().result.distance > 0.05);
        assert!(
            lab.champion.as_ref().unwrap().result.score >= lab.baseline.as_ref().unwrap().score
        );
        assert!(lab
            .progress
            .windows(2)
            .all(|p| p[1].score + 1e-5 >= p[0].score));
    }
    #[test]
    fn legacy_saves_migrate_without_discarding_life() {
        let a = Core::new(Config::default()).unwrap();
        let mut value = serde_json::to_value(&a).unwrap();
        value["version"] = 1.into();
        value.as_object_mut().unwrap().remove("lab");
        for o in value["organisms"].as_array_mut().unwrap() {
            o.as_object_mut().unwrap().remove("body");
            o.as_object_mut().unwrap().remove("development");
        }
        let b = Core::load(&value.to_string()).unwrap();
        assert_eq!(b.version, 4);
        assert_eq!(b.organisms.len(), 160);
    }
    #[test]
    fn jointed_assay_develops_and_resumes_heritable_controllers() {
        let mut a = Core::new(Config::default()).unwrap();
        a.start_joint_lab();
        assert_eq!(a.organisms[0].cells, 1);
        a.step(80);
        let result = a.lab.as_ref().unwrap().baseline.as_ref().unwrap();
        println!("jointed baseline: {:?}", result);
        assert!(result.viable && result.joints == 8 && result.work > 0. && result.distance > 0.1);
        let mut b = Core::load(&serde_json::to_string(&a).unwrap()).unwrap();
        a.step(35);
        b.step(35);
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
    }
    #[test]
    fn same_seed_is_deterministic() {
        let mut a = Core::new(Config::default()).unwrap();
        let mut b = a.clone();
        a.step(400);
        b.step(400);
        assert_eq!(a.render_data(), b.render_data());
        assert_eq!(a.metadata(0), b.metadata(0));
    }
    #[test]
    fn save_roundtrip_preserves_future() {
        let mut a = Core::new(Config::default()).unwrap();
        a.step(300);
        let mut b = Core::load(&serde_json::to_string(&a).unwrap()).unwrap();
        a.step(200);
        b.step(200);
        assert_eq!(a.render_data(), b.render_data());
        assert_eq!(a.rng, b.rng);
    }
    #[test]
    fn division_inherits_with_no_mutation() {
        let mut c = Config::default();
        c.mutation = 0.0;
        c.radiation = 0.0;
        let mut a = Core::new(c).unwrap();
        let inherited: BTreeMap<_, _> = a
            .organisms
            .iter()
            .map(|o| (o.lineage, (o.development, o.anatomy.clone())))
            .collect();
        a.step(400);
        assert!(a.births > 0);
        for o in &a.organisms {
            assert_eq!(o.development, inherited[&o.lineage].0);
            assert_eq!(o.anatomy, inherited[&o.lineage].1);
            assert_eq!(o.genes, a.lineages[&o.lineage].ancestor);
        }
    }
    #[test]
    fn finite_resources_are_consumed_and_death_recycles() {
        let mut a = Core::new(Config::default()).unwrap();
        let before = a.field.iter().sum::<f32>();
        a.step(1);
        assert!(a.field.iter().sum::<f32>() < before);
        for o in &mut a.organisms {
            o.energy = -10.0;
        }
        let before = a.field.iter().sum::<f32>();
        a.step(1);
        assert_eq!(a.organisms.len(), 0);
        assert!(a.field.iter().sum::<f32>() > before);
        assert_eq!(a.deaths, 160);
    }
    #[test]
    fn capacity_is_enforced() {
        let mut c = Config::default();
        c.capacity = 200;
        let mut a = Core::new(c).unwrap();
        a.seed(1000);
        a.step(400);
        assert!(a.organisms.len() <= 200);
        assert!(a.field.iter().all(|v| v.is_finite() && *v >= 0.0));
    }
    #[test]
    fn incompatible_environment_can_go_extinct() {
        let mut c = Config::default();
        c.solvent = 0.0;
        let mut a = Core::new(c).unwrap();
        a.step(1);
        assert!(a.organisms.is_empty());
    }
    #[test]
    fn adhesion_grows_colonies_without_time_unlocks() {
        let mut a = Core::new(Config::default()).unwrap();
        for o in &mut a.organisms {
            o.genes[ADHESION] = 1.0;
            o.genes[SKELETON] = 0.8;
            o.genes[SPECIALIZATION] = 0.9;
            o.energy = 100.0;
            o.cooldown = 0.0;
        }
        a.step(80);
        assert!(a.organisms.iter().any(|o| o.cells > 1));
    }
    #[test]
    fn compatible_mating_recombines_genes() {
        let mut a = Core::new(Config::default()).unwrap();
        a.organisms.truncate(2);
        for o in &mut a.organisms {
            o.x = 0.0;
            o.z = 0.0;
            o.energy = 100.0;
            o.cooldown = 0.0;
            o.genes = [0.5; GENE_COUNT];
            o.genes[SEX] = 1.0;
            o.genes[ADHESION] = 0.0;
        }
        a.config.mutation = 0.0;
        a.organisms[1].genes[PHOTO] = 0.6;
        a.step(1);
        assert!(a.mating > 0);
        assert!(a.organisms.iter().any(|o| o.mate > 0));
    }
    #[test]
    fn rejects_corrupt_saves_and_invalid_config() {
        let mut a = Core::new(Config::default()).unwrap();
        a.organisms[0].cells = 1000;
        assert!(Core::load(&serde_json::to_string(&a).unwrap()).is_err());
        let mut c = Config::default();
        c.capacity = 500000;
        assert!(Core::new(c).is_err());
    }
    #[test]
    fn mineral_and_sun_worlds_select_different_metabolisms() {
        let mut c = Config::default();
        c.sunlight = 1.0;
        c.minerals = 0.0;
        let mut light = Core::new(c.clone()).unwrap();
        c.sunlight = 0.0;
        c.minerals = 1.0;
        let mut dark = Core::new(c).unwrap();
        // Control all other traits to isolate the environmental selection pressure.
        for w in [&mut light, &mut dark] {
            for (i, o) in w.organisms.iter_mut().enumerate() {
                o.genes = [0.2; GENE_COUNT];
                o.genes[THERMAL] = 0.48;
                o.genes[PHOTO] = if i % 2 == 0 { 0.9 } else { 0.1 };
                o.genes[CHEMO] = if i % 2 == 0 { 0.1 } else { 0.9 };
            }
        }
        for _ in 0..8 {
            light.step(400);
            dark.step(400);
        }
        let mean = |w: &Core, g: usize| {
            w.organisms.iter().map(|o| o.genes[g]).sum::<f32>() / w.organisms.len().max(1) as f32
        };
        assert!(
            mean(&light, PHOTO) > mean(&dark, PHOTO),
            "light {} dark {}",
            mean(&light, PHOTO),
            mean(&dark, PHOTO)
        );
        assert!(mean(&dark, CHEMO) > mean(&light, CHEMO));
    }
}
