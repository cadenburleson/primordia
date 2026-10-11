//! Cell-local regulatory genomes. No tissue stages, target body size or gait.
use crate::*;
use body::{Cell as RenderCell, Link, V3};

pub const MAX_CELLS: usize = 64000;
pub const MAX_FOUNDERS: usize = 8192;
pub const MAX_GENES: usize = 48;
pub const ACTIONS: usize = 45;
pub const INPUTS: usize = 32;
/// Vocabulary sizes of earlier engine versions, padded with zeros on load.
const OLD_ACTIONS: [usize; 8] = [10, 16, 21, 32, 38, 42, 43, 44];
const OLD_INPUTS: [usize; 4] = [8, 16, 20, 28];
/// The last promoter inputs are signed left-minus-right senses.
pub const LATERAL: usize = 4;
// Accept precisely the old vocabulary or the new one. Padding is a migration,
// not permission to silently truncate a malformed genome or material store.
fn migrated_array<
    'de,
    D: serde::Deserializer<'de>,
    T: Default + Copy + Deserialize<'de>,
    const N: usize,
>(
    d: D,
    earlier: &[usize],
) -> Result<[T; N], D::Error> {
    let values = Vec::<T>::deserialize(d)?;
    if values.len() != N && !earlier.contains(&values.len()) {
        return Err(serde::de::Error::custom(
            "Invalid cellular vocabulary length",
        ));
    }
    let mut result = [T::default(); N];
    result[..values.len()].copy_from_slice(&values);
    Ok(result)
}
/// serde implements arrays only up to 32 entries; the gene vocabulary is larger.
pub(crate) fn as_list<S: serde::Serializer, T: Serialize, const N: usize>(
    values: &[T; N],
    s: S,
) -> Result<S::Ok, S::Error> {
    values.as_slice().serialize(s)
}
fn promoter_array<'de, D: serde::Deserializer<'de>>(d: D) -> Result<[f32; INPUTS], D::Error> {
    migrated_array(d, &OLD_INPUTS)
}
pub(crate) fn expression_array<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<[f32; ACTIONS], D::Error> {
    migrated_array(d, &OLD_ACTIONS)
}
pub(crate) fn protein_array<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<[f64; ACTIONS], D::Error> {
    migrated_array(d, &OLD_ACTIONS)
}
// Physical/chemical primitives, not tissue identities.
pub const ACTION_NAMES: [&str; ACTIONS] = [
    "Light uptake",
    "Mineral uptake",
    "Adhesion",
    "Material stiffness",
    "Contraction",
    "Chemical A secretion",
    "Chemical B secretion",
    "Division investment",
    "Membrane transport",
    "Contact digestion",
    "Lipid storage",
    "Local receptors",
    "Excitable channels",
    "Matrix repair",
    "Protective membrane",
    "Detritus breakdown",
    "Photoreception",
    "Adaptive contact signaling",
    "Gametic division",
    "Membrane fusion",
    "Division polarity",
    "Extracellular matrix",
    "Hydraulic pumping",
    "Gas exchange",
    "Mechanical sensing",
    "Membrane recognition",
    "Replication fidelity",
    "Matrix digestion",
    "Fiber crosslinking",
    "Mineral remodeling",
    "Polymer assembly",
    "Membrane lysis",
    "Propulsion",
    "Steering",
    "Engulfing",
    "Venom",
    "Spines",
    "Chemosynthesis",
    "Fate A commitment",
    "Fate B commitment",
    "Fate C commitment",
    "Cell wall",
    "Fins",
    "Sideways division",
    "Cell size",
];
/// A cell is a stem cell (fate 0) or committed to one of three fates.
pub const FATES: u8 = 4;
fn all_fates() -> u8 {
    (1 << FATES) - 1
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Gene {
    pub action: usize,
    #[serde(deserialize_with = "promoter_array")]
    pub promoter: [f32; INPUTS],
    pub bias: f32,
    pub strength: f32,
    /// Fates in which this gene is switched on, one bit each. This is how
    /// cells with the same genome come to do different jobs.
    #[serde(default = "all_fates")]
    pub fates: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Genome {
    pub id: u32,
    pub genes: Vec<Gene>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell {
    #[serde(default)]
    pub collection_sources: Vec<String>,
    #[serde(default)]
    pub material: Option<crate::physical::Inventory>,
    #[serde(default)]
    pub state: crate::physiology::CellState,
    #[serde(default)]
    pub lifecycle: crate::development::Lifecycle,
    pub id: u32,
    pub parent: u32,
    pub founder: u32,
    pub generation: u32,
    pub genome: Genome,
    pub p: V3,
    pub v: V3,
    pub energy: f32,
    pub age: f32,
    pub cooldown: f32,
    pub chemicals: [f32; 2],
    #[serde(deserialize_with = "expression_array", serialize_with = "as_list")]
    pub expression: [f32; ACTIONS],
    pub heading: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bond {
    pub a: u32,
    pub b: u32,
    pub rest: f32,
    pub integrity: f32,
    pub activation: f32,
    pub stiffness: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Division {
    pub time: f32,
    pub parent: u32,
    pub child: u32,
    pub genome: u32,
    pub generation: u32,
    pub attached: bool,
    pub edits: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Evolution {
    #[serde(default)]
    pub physical: Option<crate::physical::Habitat>,
    pub cells: Vec<Cell>,
    pub bonds: Vec<Bond>,
    pub next_cell: u32,
    pub next_genome: u32,
    pub divisions: u32,
    pub attached_divisions: u32,
    pub deaths: u32,
    pub exchanges: u32,
    pub meals: u32,
    pub point_edits: u32,
    pub duplications: u32,
    pub deletions: u32,
    pub innovations: u32,
    pub food_energy: f64,
    pub motor_work: f64,
    pub history: Vec<Division>,
    pub peak_body: usize,
    pub peak_generation: u32,
}
impl Gene {
    fn valid(&self) -> bool {
        self.action < ACTIONS
            && self
                .promoter
                .iter()
                .chain([&self.bias, &self.strength])
                .all(|x| x.is_finite() && x.abs() <= 2.)
            && self.strength >= 0.
            && (1..=all_fates()).contains(&self.fates)
    }
}
fn founder_promoter(values: &[f32]) -> [f32; INPUTS] {
    let mut result = [0.; INPUTS];
    result[..values.len()].copy_from_slice(values);
    result
}
impl Genome {
    pub fn founder() -> Self {
        // Only uptake, division and weak daughter adhesion. No support, motors,
        // secretion, transport or digestion genes are supplied.
        Self {
            id: 1,
            genes: vec![
                Gene {
                    action: 0,
                    promoter: founder_promoter(&[
                        0.4, 0., -0.1, 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
                        0., 0.,
                    ]),
                    bias: 0.7,
                    strength: 0.55,
                    fates: 15,
                },
                Gene {
                    action: 1,
                    promoter: founder_promoter(&[
                        0., 0.4, -0.1, 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
                        0., 0.,
                    ]),
                    bias: 0.7,
                    strength: 0.55,
                    fates: 15,
                },
                Gene {
                    action: 7,
                    promoter: founder_promoter(&[
                        0., 0., 1., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
                        0.,
                    ]),
                    bias: -0.2,
                    strength: 0.75,
                    fates: 15,
                },
                Gene {
                    action: 2,
                    promoter: [0.; INPUTS],
                    bias: 0.2,
                    strength: 0.12,
                    fates: 15,
                },
            ],
        }
    }
    pub fn express<const N: usize>(&self, inputs: [f32; N]) -> [f32; ACTIONS] {
        self.express_in(inputs, 0)
    }
    /// Expression in a cell of the given fate: genes masked off for that
    /// fate are silent, whatever their promoters say.
    pub fn express_in<const N: usize>(&self, inputs: [f32; N], fate: u8) -> [f32; ACTIONS] {
        let mut out = [0.; ACTIONS];
        for g in self.genes.iter().filter(|g| g.fates >> fate & 1 == 1) {
            let drive = g.bias
                + g.promoter
                    .iter()
                    .zip(inputs)
                    .map(|(w, x)| w * x)
                    .sum::<f32>();
            out[g.action] += drive.clamp(0., 1.) * g.strength;
        }
        for x in &mut out {
            *x = x.clamp(0., 1.);
        }
        out
    }
    /// Signed turn command: each steering gene weighs the left-minus-right
    /// senses with its own promoter weights. Positive turns left.
    pub fn steer(&self, lateral: [f32; LATERAL], fate: u8) -> f32 {
        self.genes
            .iter()
            .filter(|g| g.action == crate::physical::STEER && g.fates >> fate & 1 == 1)
            .map(|g| {
                g.strength
                    * g.promoter[INPUTS - LATERAL..]
                        .iter()
                        .zip(lateral)
                        .map(|(w, x)| w * x)
                        .sum::<f32>()
            })
            .sum::<f32>()
            .clamp(-1., 1.)
    }
    pub(crate) fn valid(&self) -> bool {
        self.id > 0
            && !self.genes.is_empty()
            && self.genes.len() <= MAX_GENES
            && self.genes.iter().all(Gene::valid)
    }
    pub fn mutate(&mut self, core: &mut Core) -> [u32; 4] {
        self.mutate_protected(core, 0.)
    }
    pub(crate) fn mutate_protected(&mut self, core: &mut Core, fidelity: f32) -> [u32; 4] {
        self.mutate_scaled(core, fidelity, 1.)
    }
    /// `scale` multiplies every copying-error probability; 1 is the natural rate.
    pub(crate) fn mutate_scaled(&mut self, core: &mut Core, fidelity: f32, scale: f32) -> [u32; 4] {
        self.mutate_toward(core, fidelity, scale, &[])
    }
    /// As `mutate_scaled`, with half of all new genes drawn from `favoured`.
    /// The gene still arrives by a copying error, with random regulation, and
    /// its material still has to be built and paid for.
    pub(crate) fn mutate_toward(
        &mut self,
        core: &mut Core,
        fidelity: f32,
        scale: f32,
        favoured: &[usize],
    ) -> [u32; 4] {
        let rate = (1. - fidelity.clamp(0., 1.) * 0.98).max(0.02) * scale.clamp(0., 16.);
        let mut edits = [0; 4];
        for gene in &mut self.genes {
            if core.random() < 0.025 * rate {
                let slot = (core.random() * (INPUTS + 2) as f32) as usize;
                let delta = (core.random() - 0.5) * 0.8;
                if slot < INPUTS {
                    gene.promoter[slot] = (gene.promoter[slot] + delta).clamp(-2., 2.);
                } else if slot == INPUTS {
                    gene.bias = (gene.bias + delta).clamp(-2., 2.);
                } else {
                    gene.strength = (gene.strength + delta).clamp(0., 2.);
                }
                edits[0] += 1;
            }
            // Restrict or widen the fates a gene is switched on in.
            if core.random() < 0.02 * rate {
                gene.fates ^= 1 << (core.random() * FATES as f32) as u8;
                if gene.fates == 0 {
                    gene.fates = all_fates();
                }
                edits[0] += 1;
            }
        }
        if self.genes.len() < MAX_GENES && core.random() < 0.04 * rate {
            let gene = self.genes[(core.random() * self.genes.len() as f32) as usize].clone();
            self.genes.push(gene);
            edits[1] += 1;
        }
        if self.genes.len() > 1 && core.random() < 0.025 * rate {
            let i = (core.random() * self.genes.len() as f32) as usize;
            self.genes.remove(i);
            edits[2] += 1;
        }
        if self.genes.len() < MAX_GENES && core.random() < 0.08 * rate {
            let mut promoter = [0.; INPUTS];
            for w in &mut promoter {
                *w = (core.random() - 0.5) * 2.;
            }
            self.genes.push(Gene {
                action: if !favoured.is_empty() && core.random() < 0.5 {
                    favoured[((core.random() * favoured.len() as f32) as usize).min(favoured.len() - 1)]
                } else {
                    (core.random() * ACTIONS as f32) as usize
                },
                promoter,
                bias: (core.random() - 0.5) * 1.2,
                strength: core.random() * 0.7,
                // Most new genes work everywhere; some arrive fate-specific.
                fates: if core.random() < 0.25 {
                    1 + (core.random() * (all_fates() as f32 - 0.001)) as u8
                } else {
                    all_fates()
                },
            });
            edits[3] += 1;
        }
        edits
    }
}
impl Evolution {
    pub fn new(core: &mut Core) -> Self {
        let mut cells = Vec::new();
        for id in 1..=core.config.initial_cells as u32 {
            let angle = core.random() * std::f32::consts::TAU;
            let r = core.random().sqrt() * 13.;
            let (mut x, mut z) = (angle.cos() * r, angle.sin() * r);
            if core.config.founder_spread == "global" {
                let width = core.config.world_size as f32;
                if core.config.world_shape == crate::domain::Shape::Sphere {
                    x = (core.random() - 0.5) * width;
                    z = (core.random() * 2. - 1.).asin() * width / std::f32::consts::TAU;
                } else {
                    x = (core.random() - 0.5) * (width - 6.);
                    z = (core.random() - 0.5) * (width - 6.);
                }
            }

            cells.push(Cell {
                collection_sources: vec![],
                material: None,
                state: crate::physiology::CellState::default(),
                lifecycle: crate::development::Lifecycle::default(),
                id,
                parent: 0,
                founder: id,
                generation: 0,
                genome: Genome::founder(),
                p: V3::new(x, body::habitat_y(x, z, 0.22, core.config.seed), z),
                v: V3::default(),
                energy: 5.,
                age: 0.,
                cooldown: core.random() * 3.,
                chemicals: [0.; 2],
                expression: [0.; ACTIONS],
                heading: angle,
            });
        }
        Self {
            physical: None,
            cells,
            bonds: vec![],
            next_cell: core.config.initial_cells as u32 + 1,
            next_genome: 2,
            divisions: 0,
            attached_divisions: 0,
            deaths: 0,
            exchanges: 0,
            meals: 0,
            point_edits: 0,
            duplications: 0,
            deletions: 0,
            innovations: 0,
            food_energy: 0.,
            motor_work: 0.,
            history: vec![],
            peak_body: 1,
            peak_generation: 0,
        }
    }
    pub(crate) fn index(&self) -> BTreeMap<u32, usize> {
        self.cells
            .iter()
            .enumerate()
            .map(|(i, c)| (c.id, i))
            .collect()
    }
    pub fn groups(&self) -> Vec<Vec<usize>> {
        let ids = self.index();
        let mut root: Vec<usize> = (0..self.cells.len()).collect();
        fn find(r: &mut [usize], mut x: usize) -> usize {
            let start = x;
            while r[x] != x {
                x = r[x];
            }
            let result = x;
            x = start;
            while r[x] != x {
                let next = r[x];
                r[x] = result;
                x = next;
            }
            result
        }
        for b in &self.bonds {
            if b.integrity <= 0. {
                continue;
            }
            if let (Some(&a), Some(&c)) = (ids.get(&b.a), ids.get(&b.b)) {
                let a = find(&mut root, a);
                let c = find(&mut root, c);
                if a != c {
                    let (lo, hi) = if a < c { (a, c) } else { (c, a) };
                    root[hi] = lo;
                }
            }
        }
        let mut groups = BTreeMap::<usize, Vec<usize>>::new();
        for i in 0..root.len() {
            groups.entry(find(&mut root, i)).or_default().push(i);
        }
        groups.into_values().collect()
    }
    pub fn valid(&self) -> bool {
        let ids = self.index();
        let mut genomes = BTreeMap::<u32, &Vec<Gene>>::new();
        for cell in &self.cells {
            for genome in std::iter::once(&cell.genome).chain(cell.lifecycle.homolog.as_ref()) {
                if let Some(previous) = genomes.insert(genome.id, &genome.genes) {
                    if previous != &genome.genes {
                        return false;
                    }
                }
            }
        }
        let mut pairs = BTreeSet::new();
        self.physical
            .as_ref()
            .map_or(true, |h| h.valid(&self.cells))
            && self
                .cells
                .iter()
                .all(|c| c.material.is_some() == self.physical.is_some())
            && self.cells.len() <= MAX_CELLS
            && ids.len() == self.cells.len()
            && self.next_cell > self.cells.iter().map(|c| c.id).max().unwrap_or(0)
            && self.next_genome > genomes.keys().copied().max().unwrap_or(0)
            && self.bonds.len() <= MAX_CELLS * 3
            && self.history.len() <= 256
            && self.food_energy.is_finite()
            && self.food_energy >= 0.
            && self.motor_work.is_finite()
            && self.motor_work >= 0.
            && self.peak_body <= MAX_CELLS
            && self.cells.iter().all(|c| {
                c.collection_sources.len() <= 16
                    && c.collection_sources
                        .iter()
                        .all(|s| !s.is_empty() && s.len() <= 80)
                    && c.id > 0
                    && c.parent < c.id
                    && c.founder > 0
                    && c.founder <= MAX_FOUNDERS as u32
                    && c.genome.valid()
                    && c.state.valid()
                    && c.lifecycle.valid(c)
                    && [
                        c.p.x, c.p.y, c.p.z, c.v.x, c.v.y, c.v.z, c.energy, c.age, c.cooldown,
                        c.heading,
                    ]
                    .iter()
                    .all(|v| v.is_finite())
                    && c.p.x.abs() <= self.physical.as_ref().map_or(31., |h| h.domain.half() - 1.)
                    && c.p.z.abs() <= self.physical.as_ref().map_or(31., |h| h.domain.half() - 1.)
                    && c.p.y.abs()
                        < self.physical.as_ref().map_or(100., |h| {
                            if h.domain.spherical() {
                                h.domain.half() + 100.
                            } else {
                                100.
                            }
                        })
                    && c.v.len() < 100.
                    && c.energy >= 0.
                    && c.energy < 10000.
                    && c.age >= 0.
                    && c.cooldown >= 0.
                    && c.chemicals
                        .iter()
                        .chain(&c.expression)
                        .all(|x| x.is_finite() && (0.0..=1.).contains(x))
            })
            && self.bonds.iter().all(|b| {
                b.a < b.b
                    && ids.contains_key(&b.a)
                    && ids.contains_key(&b.b)
                    && pairs.insert((b.a, b.b))
                    && b.rest.is_finite()
                    && (0.1..=2.).contains(&b.rest)
                    && [b.integrity, b.activation, b.stiffness]
                        .iter()
                        .all(|x| x.is_finite() && (0.0..=1.).contains(x))
            })
            && self.physical.as_ref().is_none_or(|h| {
                h.development
                    .synapses
                    .iter()
                    .all(|s| pairs.contains(&(s.from.min(s.to), s.from.max(s.to))))
                    && h.development
                        .ancestry
                        .iter()
                        .all(|b| b.child < self.next_cell)
                    && h.joints.iter().all(|j| {
                        pairs.contains(&(j.a.min(j.b), j.a.max(j.b)))
                            && pairs.contains(&(j.b.min(j.c), j.b.max(j.c)))
                    })
            })
            && self.history.iter().all(|h| {
                h.time.is_finite()
                    && h.time >= 0.
                    && h.parent > 0
                    && h.parent < h.child
                    && h.child < self.next_cell
                    && h.genome > 0
                    && h.genome < self.next_genome
            })
    }
    pub fn tick(&mut self, core: &mut Core) {
        if let Some(mut habitat) = self.physical.take() {
            habitat.tick(self, core);
            self.physical = Some(habitat);
            return;
        }
        core.time += DT;
        for i in 0..core.field.len() {
            core.field[i] = (core.field[i] + DT * core.config.nutrients * 0.16)
                .min(core.field_capacity(i).max(core.field[i] - 0.004));
        }
        let ids = self.index();
        let n = self.cells.len();
        let mut buckets = vec![Vec::new(); GRID * GRID];
        for (i, c) in self.cells.iter().enumerate() {
            buckets[field_index(c.p.x, c.p.z)].push(i);
        }
        let old = self.cells.clone();
        let mut neighbors = vec![Vec::new(); n];
        let mut degree = vec![0usize; n];
        for b in &self.bonds {
            degree[ids[&b.a]] += 1;
            degree[ids[&b.b]] += 1;
        }
        for i in 0..n {
            let c = &old[i];
            let site = field_index(c.p.x, c.p.z);
            let (x, z) = (site % GRID, site / GRID);
            let mut chemical = [0.; 2];
            let mut count = 0.;
            for zz in z.saturating_sub(1)..=(z + 1).min(GRID - 1) {
                for xx in x.saturating_sub(1)..=(x + 1).min(GRID - 1) {
                    for &j in &buckets[zz * GRID + xx] {
                        if i != j && (old[j].p.sub(c.p)).len() < 1.5 {
                            neighbors[i].push(j);
                            for (k, v) in chemical.iter_mut().enumerate() {
                                *v += old[j].chemicals[k];
                            }
                            count += 1.;
                        }
                    }
                }
            }
            for (k, v) in chemical.iter_mut().enumerate() {
                *v = if count > 0. { *v / count } else { 0. };
                *v = 0.7 * c.chemicals[k] + 0.3 * *v;
            }
            let mineral = core.minerals_at(c.p.x, c.p.z);
            let inputs = [
                (core.field[site] / 3.).clamp(0., 1.),
                mineral,
                (c.energy / 10.).clamp(0., 1.),
                (degree[i] as f32 / 6.).min(1.),
                chemical[0],
                chemical[1],
                (c.age / 80.).min(1.),
                (c.v.len() / 2.).min(1.),
            ];
            let expr = c.genome.express(inputs);
            let demand = DT
                * 0.8
                * (expr[0] * core.config.sunlight + expr[1] * mineral)
                * core.config.solvent;
            let uptake = demand.min(core.field[site]);
            core.field[site] -= uptake;
            self.food_energy += (uptake * 3.) as f64;
            let cost = DT
                * (0.07
                    + c.genome.genes.len() as f32 * 0.004
                    + expr.iter().sum::<f32>() * 0.026
                    + expr[3] * mineral * 0.03);
            let cell = &mut self.cells[i];
            cell.expression = expr;
            cell.energy = (cell.energy + uptake * 3. - cost).max(0.);
            cell.age += DT;
            cell.cooldown = (cell.cooldown - DT).max(0.);
            for k in 0..2 {
                cell.chemicals[k] = (chemical[k] * 0.9 + expr[5 + k] * 0.1).clamp(0., 1.);
            }
            // Motility of isolated cells is passive environmental agitation. Colonies
            // have no prescribed forward movement or nutrient-gradient steering.
        }
        // Energy transport and contraction occur only along real retained-division bonds.
        for b in &mut self.bonds {
            let (a, c) = (ids[&b.a], ids[&b.b]);
            let (left, right) = self.cells.split_at_mut(c.max(a));
            let (x, y) = if a < c {
                (&mut left[a], &mut right[0])
            } else {
                (&mut right[0], &mut left[c])
            };
            let flow = (x.energy - y.energy) * 0.08 * (x.expression[8] + y.expression[8]).min(1.);
            let flow = flow.clamp(-y.energy, x.energy);
            x.energy -= flow;
            y.energy += flow;
            b.activation = (x.expression[4] + y.expression[4]) * 0.5;
            b.stiffness = (x.expression[3] + y.expression[3]) * 0.5;
            let work = DT * b.activation * 0.03;
            let paid = work.min(x.energy).min(y.energy);
            x.energy -= paid;
            y.energy -= paid;
            self.motor_work += (paid * 2.) as f64;
            if paid < work {
                b.activation *= if work > 0. { paid / work } else { 0. };
            }
            if x.energy < 0.05 || y.energy < 0.05 {
                b.integrity = (b.integrity - DT * 0.04).max(0.);
            }
        }
        // Semi-implicit particle dynamics: equal and opposite spring/contraction
        // forces, ground reaction and friction; no body plans are synthesized.
        let step = DT / 10.;
        for _ in 0..10 {
            let mut forces = vec![V3::new(0., -0.3, 0.); n];
            for b in &self.bonds {
                let (a, c) = (ids[&b.a], ids[&b.b]);
                let delta = self.cells[c].p.sub(self.cells[a].p);
                let dist = delta.len().max(0.001);
                let target = b.rest * (1. - b.activation * 0.22);
                let f = delta.mul(
                    ((dist - target) * (8. + b.stiffness * 32.) * b.integrity).clamp(-10., 10.)
                        / dist,
                );
                forces[a] = forces[a].add(f);
                forces[c] = forces[c].sub(f);
            }
            for i in 0..n {
                for &j in &neighbors[i] {
                    if j <= i {
                        continue;
                    }
                    let delta = self.cells[j].p.sub(self.cells[i].p);
                    let dist = delta.len().max(0.001);
                    if dist < 0.44 {
                        let f = delta.mul((0.44 - dist) * 6. / dist);
                        forces[i] = forces[i].sub(f);
                        forces[j] = forces[j].add(f);
                    }
                }
            }
            for (i, c) in self.cells.iter_mut().enumerate() {
                c.v = c.v.add(forces[i].mul(step)).mul(0.96);
                if c.v.len() > 5. {
                    c.v = c.v.normalized().mul(5.);
                }
                c.p = c.p.add(c.v.mul(step));
                c.p.x = c.p.x.clamp(-30., 30.);
                c.p.z = c.p.z.clamp(-30., 30.);
                let floor = body::habitat_y(c.p.x, c.p.z, 0.22, core.config.seed);
                if c.p.y < floor {
                    c.p.y = floor;
                    c.v.y = c.v.y.max(0.);
                    c.v.x *= 0.8;
                    c.v.z *= 0.8;
                }
            }
        }
        // Contact-mediated exchange copies a gene, paid by the receiver. It is not
        // required for ordinary cellular reproduction.
        for i in 0..n {
            if self.cells[i].expression[8] > 0.1
                && self.cells[i].energy > 0.1
                && core.random() < 0.005
            {
                if let Some(&j) = neighbors[i].iter().find(|&&j| {
                    self.cells[j].genome.id != self.cells[i].genome.id
                        && (self.cells[j].p.sub(self.cells[i].p)).len() < 0.55
                }) {
                    let donor = self.cells[j].genome.genes
                        [(core.random() * self.cells[j].genome.genes.len() as f32) as usize]
                        .clone();
                    if self.cells[i].genome.genes.len() < MAX_GENES {
                        self.cells[i].genome.genes.push(donor);
                        self.cells[i].genome.id = self.next_genome;
                        self.next_genome += 1;
                        self.cells[i].energy -= 0.1;
                        self.exchanges += 1;
                    }
                }
            }
        }
        // Digestive chemistry is a paid contact action. Stiffer membranes reduce
        // extraction. No predefined predator species or kill upgrade.
        for i in 0..n {
            if self.cells[i].energy <= 0. {
                continue;
            }
            let strength = self.cells[i].expression[9];
            if strength <= 0. {
                continue;
            }
            for &j in &neighbors[i] {
                if self.cells[j].founder == self.cells[i].founder || self.cells[j].energy <= 0. {
                    continue;
                }
                if self.cells[j].p.sub(self.cells[i].p).len() < 0.49 {
                    let extracted = (DT * strength * 0.4 / (1. + self.cells[j].expression[3] * 3.))
                        .min(self.cells[j].energy);
                    self.cells[j].energy -= extracted;
                    self.cells[i].energy += extracted * 0.65;
                    core.field[field_index(self.cells[j].p.x, self.cells[j].p.z)] +=
                        extracted * 0.35 / 3.;
                    if extracted > 0. {
                        self.meals += 1;
                    }
                    break;
                }
            }
        }
        let mut daughters = Vec::new();
        let limit = MAX_CELLS.min(core.config.capacity);
        for i in 0..n {
            let c = self.cells[i].clone();
            let threshold = 5. + c.genome.genes.len() as f32 * 0.12;
            if c.energy < threshold
                || c.cooldown > 0.
                || c.expression[7] <= 0.05
                || n + daughters.len() >= limit
            {
                continue;
            }
            if core.random() > c.expression[7] * DT * 0.55 {
                continue;
            }
            let mut child = c.clone();
            child.id = self.next_cell;
            self.next_cell += 1;
            child.parent = c.id;
            child.generation += 1;
            child.age = 0.;
            child.cooldown = 3.;
            let edits = child.genome.mutate(core);
            if edits.iter().sum::<u32>() > 0 {
                child.genome.id = self.next_genome;
                self.next_genome += 1;
            }
            self.point_edits += edits[0];
            self.duplications += edits[1];
            self.deletions += edits[2];
            self.innovations += edits[3];
            let angle = core.random() * std::f32::consts::TAU;
            let delta = V3::new(angle.cos() * 0.45, 0.08, angle.sin() * 0.45);
            child.p = child.p.add(delta);
            child.p.x = child.p.x.clamp(-30., 30.);
            child.p.z = child.p.z.clamp(-30., 30.);
            child.heading = angle;
            child.energy = (c.energy - 1.0) * 0.5;
            self.cells[i].energy = child.energy;
            self.cells[i].cooldown = 3.;
            let attached = degree[i] < 6 && core.random() < c.expression[2];
            if attached {
                self.bonds.push(Bond {
                    a: c.id,
                    b: child.id,
                    rest: delta.len(),
                    integrity: 1.,
                    activation: 0.,
                    stiffness: 0.,
                });
                self.attached_divisions += 1;
            }
            self.peak_generation = self.peak_generation.max(child.generation);
            self.divisions += 1;
            self.history.push(Division {
                time: core.time,
                parent: c.id,
                child: child.id,
                genome: child.genome.id,
                generation: child.generation,
                attached,
                edits: edits.iter().sum(),
            });
            if self.history.len() > 256 {
                self.history.remove(0);
            }
            daughters.push(child);
        }
        self.cells.extend(daughters);
        self.cells.retain(|c| {
            let alive = c.energy > 0.0001 && c.age < 160.;
            if !alive {
                self.deaths += 1;
                core.field[field_index(c.p.x, c.p.z)] += 0.35;
            }
            alive
        });
        let alive: BTreeSet<_> = self.cells.iter().map(|c| c.id).collect();
        self.bonds.retain(|b| {
            alive.contains(&b.a)
                && alive.contains(&b.b)
                && b.integrity > 0.
                && self
                    .cells
                    .iter()
                    .find(|c| c.id == b.a)
                    .unwrap()
                    .p
                    .sub(self.cells.iter().find(|c| c.id == b.b).unwrap().p)
                    .len()
                    < b.rest * 3.
        });
        self.peak_body = self
            .peak_body
            .max(self.groups().iter().map(Vec::len).max().unwrap_or(0));
        core.births = self.divisions;
        core.deaths = self.deaths;
        core.mating = self.exchanges;
        core.feedings = self.meals;
        if core.time - core.history_clock >= 3. {
            let groups = self.groups();
            core.history.push(History {
                time: core.time,
                population: groups.len(),
                cells: self.cells.len() as u32,
                colonies: groups.iter().filter(|g| g.len() > 1).count(),
                diversity: self
                    .cells
                    .iter()
                    .map(|c| c.genome.id)
                    .collect::<BTreeSet<_>>()
                    .len(),
            });
            if core.history.len() > 240 {
                core.history.remove(0);
            }
            core.history_clock = core.time;
        }
    }
    pub fn project(&self) -> Vec<Organism> {
        let groups = self.groups();
        let ids = self.index();
        let mut owner = vec![0usize; self.cells.len()];
        for (g, group) in groups.iter().enumerate() {
            for &i in group {
                owner[i] = g;
            }
        }
        let mut group_bonds = vec![Vec::new(); groups.len()];
        for b in &self.bonds {
            group_bonds[owner[ids[&b.a]]].push(b);
        }
        let mut group_joints = vec![Vec::new(); groups.len()];
        if let Some(h) = &self.physical {
            for j in &h.joints {
                group_joints[owner[ids[&j.b]]].push(j);
            }
        }
        groups
            .iter()
            .enumerate()
            .map(|(g, group)| {
                let first = &self.cells[group[0]];
                let mut body = Body::default();
                let mut expressions = [0.; ACTIONS];
                let mut energy = 0.;
                let mut center = V3::default();
                let local: BTreeMap<_, _> = group
                    .iter()
                    .enumerate()
                    .map(|(i, &j)| (self.cells[j].id, i))
                    .collect();
                for &j in group {
                    let c = &self.cells[j];
                    for (k, v) in expressions.iter_mut().enumerate() {
                        *v += c.activity(k) / group.len() as f32;
                    }
                    energy += c.energy;
                    center = center.add(c.p.mul(1. / group.len() as f32));
                    body.nodes.push(RenderCell {
                        p: c.p,
                        v: c.v,
                        rest: c.p,
                        radius: c.radius(),
                        role: if c.material.is_some() {
                            // Display flags: bit 6 the player's chosen lineage,
                            // bit 5 a complex cell carrying a symbiont, bit 7 a
                            // cell committed to a fate.
                            c.material_role()
                                | if c.state.favoured { 64 } else { 0 }
                                | if c.state.symbiont > 0. { 32 } else { 0 }
                                | if c.state.fate > 0 { 128 } else { 0 }
                        } else if c.activity(4) > 0.1 {
                            2
                        } else if c
                            .material
                            .as_ref()
                            .map_or(c.expression[3], |m| (m.mineral / m.mass()) as f32)
                            > 0.1
                        {
                            1
                        } else {
                            0
                        },
                        contact: if c.material.is_some() {
                            c.state.ground
                        } else {
                            true
                        },
                        health: 1. - c.state.injury,
                        reserve: c.energy,
                        signal: [c.chemicals[0], c.chemicals[1], c.state.excitation],
                    });
                }
                for b in &group_bonds[g] {
                    if let (Some(&a), Some(&c)) = (local.get(&b.a), local.get(&b.b)) {
                        let dist = self.cells[ids[&b.a]].p.sub(self.cells[ids[&b.b]].p).len();
                        body.links.push(Link {
                            a,
                            b: c,
                            rest: b.rest,
                            target: b.rest * (1. - b.activation * 0.22),
                            kind: if b.activation > 0.01 {
                                2
                            } else if b.stiffness > 0.1 {
                                1
                            } else {
                                0
                            },
                            activation: b.activation,
                            strain: (dist / b.rest - 1.).abs(),
                            integrity: b.integrity,
                            purpose: 0,
                        });
                    }
                }
                for j in &group_joints[g] {
                    let (Some(&a), Some(&b), Some(&c)) =
                        (local.get(&j.a), local.get(&j.b), local.get(&j.c))
                    else {
                        continue;
                    };
                    body.joints.push(crate::articulation::Joint {
                        a,
                        b,
                        c,
                        segments: [0, 0],
                        enabled: true,
                        min: (j.rest - 0.6).max(0.1),
                        max: (j.rest + 0.6).min(3.1),
                        angle: j.angle,
                        flexion: j.flexion,
                        extension: j.extension,
                        state: [0., 0.],
                        work: j.work as f32,
                        limit_error: 0.,
                    });
                }
                body.contacts = body.nodes.iter().filter(|c| c.contact).count() as u32;
                body.work = group
                    .iter()
                    .map(|&i| self.cells[i].state.motor_work as f32)
                    .sum();
                let mut genes = [0.; GENE_COUNT];
                genes[PHOTO] = expressions[0];
                genes[CHEMO] = expressions[1];
                genes[ADHESION] = expressions[2];
                genes[SKELETON] = expressions[3];
                genes[MUSCLE] = expressions[4];
                genes[PREDATION] = expressions[9];
                genes[PIGMENT] = (first.founder as f32 * 0.618).fract();
                if group.len() == 1 && self.physical.is_none() {
                    body = Body::default();
                }
                Organism {
                    id: first.id,
                    parent: first.parent,
                    mate: 0,
                    family: 0,
                    offspring: 0,
                    lineage: first.genome.id,
                    generation: group
                        .iter()
                        .map(|&i| self.cells[i].generation)
                        .max()
                        .unwrap_or(0),
                    x: center.x,
                    z: center.z,
                    vx: first.v.x,
                    vz: first.v.z,
                    heading: first.heading,
                    energy,
                    age: first.age,
                    cells: group.len() as u32,
                    genes,
                    development: default_development(),
                    regulation: None,
                    anatomy: None,
                    body,
                    cooldown: first.cooldown,
                    alive: true,
                    feeding_cooldown: 0.,
                    meals: group.iter().map(|&i| self.cells[i].state.meals).sum(),
                }
            })
            .collect()
    }
    pub fn metadata(&self, core: &Core, selected: u32) -> String {
        self.metadata_projected(core, selected, &self.project())
    }
    pub fn metadata_projected(
        &self,
        core: &Core,
        selected: u32,
        projections: &[Organism],
    ) -> String {
        let selected_body = projections.iter().find(|o| o.id == selected);
        let selected_cells = if selected_body.is_some() {
            self.groups()
                .into_iter()
                .find(|g| self.cells[g[0]].id == selected)
                .unwrap_or_default()
        } else {
            vec![]
        };
        let mut lineages = BTreeMap::<u32, serde_json::Value>::new();
        for o in projections {
            let l=lineages.entry(o.lineage).or_insert_with(||serde_json::json!({"id":o.lineage,"parent":0,"name":format!("Genome {}",o.lineage),"born":0.,"generation":o.generation,"color":"#b7df9c","ancestor":o.genes,"peak":0,"count":0,"representative":o.id}));
            l["count"] = (l["count"].as_u64().unwrap() + 1).into();
        }
        let mut selected_value=selected_body.map(|o| {
            let mut inspected = o.body.clone();
            inspected.nodes.truncate(256);
            inspected.links.retain(|l|l.a<256 && l.b<256);
            inspected.joints.retain(|j|j.a<256 && j.b<256 && j.c<256);
            serde_json::json!({"id":o.id,"lineage":o.lineage,"name":format!("Cell body {}",o.id),"generation":o.generation,"age":o.age,"cells":o.cells,"energy":o.energy,"genes":o.genes,"parent":o.parent,"mate":0,"family":0,"offspring":0,"development":o.development,"regulation":null,"anatomy":null,"body":inspected,"meals":o.meals,"feedingCooldown":0.,"x":o.x,"z":o.z})});
        if let Some(v) = &mut selected_value {
            v["mate"] = self
                .cells
                .iter()
                .find(|c| c.id == selected)
                .map_or(0, |c| c.lifecycle.mate)
                .into();
        }
        let programs:Vec<_>=selected_cells.iter().take(64).map(|&i|{let c=&self.cells[i];serde_json::json!({"id":c.id,"parent":c.parent,"genome":c.genome.id,"generation":c.generation,"genes":c.genome.genes,"energy":c.energy,"chemicals":c.chemicals,"expression":c.expression.as_slice(),"material":c.material,"state":c.state,"lifecycle":c.lifecycle})}).collect();
        if let Some(v) = &mut selected_value {
            v["programCells"] = serde_json::json!(programs);
            v["collectionSources"] = serde_json::json!(selected_cells
                .iter()
                .flat_map(|&i| self.cells[i].collection_sources.iter())
                .collect::<BTreeSet<_>>());
            // A body descended from a starter species carries its name.
            if let Some(species) = selected_cells
                .first()
                .and_then(|&i| self.cells[i].collection_sources.first())
            {
                v["name"] = format!("{species} {}", v["id"]).into();
            }
            if let Some(h) = &self.physical {
                v["observation"] = h.observation(self, &selected_cells);
                v["bearing"] = h.bearing(self, &selected_cells, core.config.seed);
                let members: Vec<&Cell> = selected_cells.iter().map(|&i| &self.cells[i]).collect();
                let n = members.len().max(1) as f32;
                let mean = |f: &dyn Fn(&Cell) -> f32| members.iter().map(|c| f(c)).sum::<f32>() / n;
                v["feeding"] = serde_json::json!({
                    "level": crate::trophic::NAMES[crate::physical::Habitat::body_level(&members)],
                    "steps": mean(&|c| c.state.trophic),
                    "scavenged": mean(&|c| c.state.scavenged),
                    "nitrogenShort": mean(&|c| c.state.nitrogen_short),
                    "phosphorusShort": mean(&|c| c.state.phosphorus_short),
                    "stature": mean(&|c| c.stature() as f32),
                    "bone": mean(&|c| c.material.as_ref().map_or(0., |m| m.bone() as f32)),
                    "fat": mean(&|c| c.material.as_ref().map_or(0., |m| m.sheath() as f32)),
                });
            }
        }
        let development = self.physical.as_ref().map(|h| h.development_view(self));
        let gene_counts: Vec<_> = self
            .cells
            .iter()
            .map(|c| {
                c.genome.genes.len() + c.lifecycle.homolog.as_ref().map_or(0, |g| g.genes.len())
            })
            .collect();
        let genomes = self
            .cells
            .iter()
            .map(|c| c.genome.id)
            .collect::<BTreeSet<_>>()
            .len();
        let expressed: Vec<_> = (0..ACTIONS)
            .map(|k| self.cells.iter().filter(|c| c.expression[k] > 0.1).count())
            .collect();
        let mut metadata = serde_json::json!({"config":core.config,"lab":null,"study":false,"competition":null,"autonomous":{"actions":if self.physical.is_some(){crate::physical::SYNTHESIS_NAMES.as_slice()}else{ACTION_NAMES.as_slice()},"physics":self.physical.as_ref().map(|h|h.view(&self.cells)),"expressed":expressed,"divisions":self.divisions,"attachedDivisions":self.attached_divisions,"pointEdits":self.point_edits,"duplications":self.duplications,"deletions":self.deletions,"innovations":self.innovations,"exchanges":self.exchanges,"foodEnergy":self.food_energy,"motorWork":self.motor_work,"peakBody":self.peak_body,"peakGeneration":self.peak_generation,"genomeMin":gene_counts.iter().min().copied().unwrap_or(0),"genomeMax":gene_counts.iter().max().copied().unwrap_or(0),"genomes":genomes,"recent":self.history.iter().rev().take(8).collect::<Vec<_>>()},"selected":selected_value,"lineages":lineages.into_values().collect::<Vec<_>>(),"events":core.events,"history":core.history,"stats":{"time":core.time,"population":projections.len(),"cells":self.cells.len(),"colonies":projections.iter().filter(|o|o.cells>1).count(),"diversity":genomes,"generation":self.peak_generation,"births":self.divisions,"deaths":self.deaths,"mating":self.physical.as_ref().map_or(self.exchanges,|h|h.development.zygotes),"feedings":self.meals,"meanGenes":vec![0.;GENE_COUNT],"nutrient":core.field.iter().sum::<f32>()/core.field.len()as f32,"capped":self.cells.len()>=MAX_CELLS.min(core.config.capacity),"computeMs":0.,"cellBudget":MAX_CELLS.min(core.config.capacity),"cellBudgetReached":self.cells.len()>=MAX_CELLS.min(core.config.capacity),"regulated":projections.len(),"injured":self.cells.iter().filter(|c|c.state.injury>0.05).count(),"articulated":projections.iter().filter(|o|!o.body.joints.is_empty()).count(),"jointed":projections.iter().filter(|o|!o.body.joints.is_empty()).count(),"contractile":projections.iter().filter(|o|o.body.links.iter().any(|b|b.activation>0.01)).count()}});
        metadata["autonomous"]["development"] = serde_json::json!(development);
        if let Some(h) = &self.physical {
            metadata["autonomous"]["systems"] = h.systems_view(self);
            metadata["autonomous"]["steering"] = h.steering_view(self, core);
            metadata["autonomous"]["physiology"] = h.physiology_view(self);
            metadata["autonomous"]["planet"] = h.planet.as_ref().map(|p| p.view(core.time)).into();
            metadata["autonomous"]["resources"] = serde_json::json!({"detritus":h.solutes.iter().map(|s|s[5]).collect::<Vec<_>>(),"mineral":h.solutes.iter().map(|s|s[2]).collect::<Vec<_>>(),"waste":h.solutes.iter().map(|s|s[1]).collect::<Vec<_>>(),"light":(0..h.domain.sites()).map(|i|h.planet.as_ref().map_or(1.,|p|p.illumination(i))*h.light_fraction(i)*core.config.sunlight as f64).collect::<Vec<_>>(),"area":(0..h.domain.sites()).map(|i|h.domain.area(i)).collect::<Vec<_>>()});
            // Each meal with the body its eater now belongs to, so a player
            // can jump to it; 0 once the eater is dead.
            let mut body_of = BTreeMap::new();
            for g in self.groups() {
                for &i in &g {
                    body_of.insert(self.cells[i].id, self.cells[g[0]].id);
                }
            }
            metadata["autonomous"]["feeding"] = h
                .meals
                .iter()
                .map(|m| {
                    let mut v = serde_json::json!(m);
                    v["attacker_body"] = body_of.get(&m.attacker).copied().unwrap_or(0).into();
                    v
                })
                .collect::<Vec<_>>()
                .into();
            metadata["autonomous"]["observationControl"] = serde_json::json!(h.ablation);
            metadata["autonomous"]["detritus"] = h.solutes.iter().map(|s| s[5]).sum::<f64>().into();
            metadata["autonomous"]["waste"] = h.solutes.iter().map(|s| s[1]).sum::<f64>().into();
        }
        metadata.to_string()
    }
}
impl Core {
    pub fn start_autonomous(&mut self) {
        let mut config = self.config.clone();
        config.name = "Autonomous cellular evolution".into();
        config.origin = "chemistry".into();
        config.kind = if config.kind == "lab" {
            "tidal".into()
        } else {
            config.kind
        };
        *self = Core::new(config).unwrap();
        self.config.origin = "seeded".into();
        self.events.clear();
        self.history.clear();
        let evolution = Evolution::new(self);
        self.autonomous = Some(evolution);
        self.event("world",format!("{} identical supplied replicating cells, four basic genes. Variable genomes and local signals drive actual cell division and attachment. No tissue stages, adult bodies, optimizer or automatic replacement.", self.config.initial_cells));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn founder_has_only_basic_replication_and_no_specialized_actions() {
        let g = Genome::founder();
        let e = g.express([1.; 8]);
        for k in [3, 4, 5, 6, 8, 9] {
            assert_eq!(e[k], 0.);
        }
        assert!(g.valid());
    }
    #[test]
    fn regulatory_context_changes_expression_without_changing_genome() {
        let g = Genome::founder();
        let before = g.clone();
        assert_ne!(g.express([0.; 8]), g.express([1.; 8]));
        assert_eq!(g, before);
    }
    #[test]
    fn copying_mutation_can_change_gene_count_and_preserves_bounds() {
        let mut core = Core::new(Config::default()).unwrap();
        let mut g = Genome::founder();
        let mut seen = [0; 4];
        for _ in 0..1000 {
            let edits = g.mutate(&mut core);
            for k in 0..4 {
                seen[k] += edits[k];
            }
            assert!(g.valid());
        }
        assert!(seen.iter().all(|v| *v > 0));
    }
    #[test]
    fn cell_division_and_graphs_resume_exactly() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_autonomous();
        c.step(80);
        assert!(c.autonomous.as_ref().unwrap().divisions > 0);
        assert!(c.autonomous.as_ref().unwrap().valid());
        let saved = serde_json::to_string(&c).unwrap();
        let mut other = Core::load(&saved).unwrap();
        c.step(8);
        other.step(8);
        assert_eq!(
            serde_json::to_string(&c).unwrap(),
            serde_json::to_string(&other).unwrap()
        );
    }
    #[test]
    fn paid_internal_contraction_preserves_horizontal_momentum_in_free_space() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_autonomous();
        let mut e = c.autonomous.take().unwrap();
        e.cells.truncate(2);
        for (i, cell) in e.cells.iter_mut().enumerate() {
            cell.p = V3::new(i as f32 * 0.7, 10., 0.);
            cell.v = V3::default();
            cell.energy = 4.;
            cell.cooldown = 100.;
            cell.genome.genes.push(Gene {
                action: 4,
                promoter: [0.; INPUTS],
                bias: 1.,
                strength: 1.,
                fates: 15,
            });
        }
        e.bonds.push(Bond {
            a: 1,
            b: 2,
            rest: 0.7,
            integrity: 1.,
            activation: 0.,
            stiffness: 0.,
        });
        e.tick(&mut c);
        assert!(e.motor_work > 0.);
        assert!((e.cells[0].v.x + e.cells[1].v.x).abs() < 1e-6);
        assert!(e.cells[0].v.x.abs() > 0.);
        assert!(e.valid());
    }
    #[test]
    fn starvation_does_not_supply_new_genes_or_rescue_extinction() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_autonomous();
        c.config.solvent = 0.;
        c.step(400);
        c.step(400);
        assert!(c.autonomous.as_ref().unwrap().cells.is_empty());
        let divisions = c.births;
        c.step(100);
        assert!(c.autonomous.as_ref().unwrap().cells.is_empty());
        assert_eq!(c.births, divisions);
    }
    #[test]
    fn corrupt_programs_and_bonds_are_rejected_without_replacing_the_world() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_autonomous();
        let mut bad = c.clone();
        bad.autonomous.as_mut().unwrap().cells[0].genome.genes[0].action = ACTIONS;
        assert!(Core::load(&serde_json::to_string(&bad).unwrap()).is_err());
        bad = c.clone();
        bad.autonomous.as_mut().unwrap().bonds.push(Bond {
            a: 1,
            b: 999,
            rest: 0.4,
            integrity: 1.,
            activation: 0.,
            stiffness: 0.,
        });
        assert!(Core::load(&serde_json::to_string(&bad).unwrap()).is_err());
    }
}
