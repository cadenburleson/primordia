//! Cell-local regulatory genomes. No tissue stages, target body size or gait.
use crate::*;
use body::{Cell as RenderCell, Link, V3};

pub const MAX_CELLS: usize = 64000;
pub const MAX_FOUNDERS: usize = 8192;
pub const MAX_GENES: usize = 48;
pub const ACTIONS: usize = 10;
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
];
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Gene {
    pub action: usize,
    pub promoter: [f32; 8],
    pub bias: f32,
    pub strength: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Genome {
    pub id: u32,
    pub genes: Vec<Gene>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell {
    #[serde(default)]
    pub material: Option<crate::physical::Inventory>,
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
    }
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
                    promoter: [0.4, 0., -0.1, 0., 0., 0., 0., 0.],
                    bias: 0.7,
                    strength: 0.55,
                },
                Gene {
                    action: 1,
                    promoter: [0., 0.4, -0.1, 0., 0., 0., 0., 0.],
                    bias: 0.7,
                    strength: 0.55,
                },
                Gene {
                    action: 7,
                    promoter: [0., 0., 1., 0., 0., 0., 0., 0.],
                    bias: -0.2,
                    strength: 0.75,
                },
                Gene {
                    action: 2,
                    promoter: [0., 0., 0., 0., 0., 0., 0., 0.],
                    bias: 0.2,
                    strength: 0.12,
                },
            ],
        }
    }
    pub fn express(&self, inputs: [f32; 8]) -> [f32; ACTIONS] {
        let mut out = [0.; ACTIONS];
        for g in &self.genes {
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
    fn valid(&self) -> bool {
        self.id > 0
            && !self.genes.is_empty()
            && self.genes.len() <= MAX_GENES
            && self.genes.iter().all(Gene::valid)
    }
    pub fn mutate(&mut self, core: &mut Core) -> [u32; 4] {
        let mut edits = [0; 4];
        for gene in &mut self.genes {
            if core.random() < 0.025 {
                let slot = (core.random() * 10.) as usize;
                let delta = (core.random() - 0.5) * 0.8;
                if slot < 8 {
                    gene.promoter[slot] = (gene.promoter[slot] + delta).clamp(-2., 2.);
                } else if slot == 8 {
                    gene.bias = (gene.bias + delta).clamp(-2., 2.);
                } else {
                    gene.strength = (gene.strength + delta).clamp(0., 2.);
                }
                edits[0] += 1;
            }
        }
        if self.genes.len() < MAX_GENES && core.random() < 0.04 {
            let gene = self.genes[(core.random() * self.genes.len() as f32) as usize].clone();
            self.genes.push(gene);
            edits[1] += 1;
        }
        if self.genes.len() > 1 && core.random() < 0.025 {
            let i = (core.random() * self.genes.len() as f32) as usize;
            self.genes.remove(i);
            edits[2] += 1;
        }
        if self.genes.len() < MAX_GENES && core.random() < 0.08 {
            let mut promoter = [0.; 8];
            for w in &mut promoter {
                *w = (core.random() - 0.5) * 2.;
            }
            self.genes.push(Gene {
                action: (core.random() * ACTIONS as f32) as usize,
                promoter,
                bias: (core.random() - 0.5) * 1.2,
                strength: core.random() * 0.7,
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
            let x = angle.cos() * r;
            let z = angle.sin() * r;
            cells.push(Cell {
                material: None,
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
            if let Some(previous) = genomes.insert(cell.genome.id, &cell.genome.genes) {
                if previous != &cell.genome.genes {
                    return false;
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
            && self.next_genome > self.cells.iter().map(|c| c.genome.id).max().unwrap_or(0)
            && self.bonds.len() <= MAX_CELLS * 3
            && self.history.len() <= 256
            && self.food_energy.is_finite()
            && self.food_energy >= 0.
            && self.motor_work.is_finite()
            && self.motor_work >= 0.
            && self.peak_body <= MAX_CELLS
            && self.cells.iter().all(|c| {
                c.id > 0
                    && c.parent < c.id
                    && c.founder > 0
                    && c.founder <= MAX_FOUNDERS as u32
                    && c.genome.valid()
                    && [
                        c.p.x, c.p.y, c.p.z, c.v.x, c.v.y, c.v.z, c.energy, c.age, c.cooldown,
                        c.heading,
                    ]
                    .iter()
                    .all(|v| v.is_finite())
                    && c.p.x.abs() <= 31.
                    && c.p.z.abs() <= 31.
                    && c.p.y.abs() < 100.
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
                h.joints.iter().all(|j| {
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
                        role: if c.activity(4) > 0.1 {
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
                        contact: true,
                        health: 1.,
                        reserve: c.energy,
                        signal: [c.chemicals[0], c.chemicals[1], 0.],
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
                body.contacts = group.len() as u32;
                let mut genes = [0.; GENE_COUNT];
                genes[PHOTO] = expressions[0];
                genes[CHEMO] = expressions[1];
                genes[ADHESION] = expressions[2];
                genes[SKELETON] = expressions[3];
                genes[MUSCLE] = expressions[4];
                genes[PREDATION] = expressions[9];
                genes[PIGMENT] = (first.founder as f32 * 0.618).fract();
                if group.len() == 1 {
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
                    meals: 0,
                }
            })
            .collect()
    }
    pub fn metadata(&self, core: &Core, selected: u32) -> String {
        let projections = self.project();
        let selected_body = projections.iter().find(|o| o.id == selected);
        let selected_cells = self
            .groups()
            .into_iter()
            .find(|g| self.cells[g[0]].id == selected)
            .unwrap_or_default();
        let mut lineages = BTreeMap::<u32, serde_json::Value>::new();
        for o in &projections {
            let l=lineages.entry(o.lineage).or_insert_with(||serde_json::json!({"id":o.lineage,"parent":0,"name":format!("Genome {}",o.lineage),"born":0.,"generation":o.generation,"color":"#b7df9c","ancestor":o.genes,"peak":0,"count":0,"representative":o.id}));
            l["count"] = (l["count"].as_u64().unwrap() + 1).into();
        }
        let mut selected_value=selected_body.map(|o|serde_json::json!({"id":o.id,"lineage":o.lineage,"name":format!("Cell body {}",o.id),"generation":o.generation,"age":o.age,"cells":o.cells,"energy":o.energy,"genes":o.genes,"parent":o.parent,"mate":0,"family":0,"offspring":0,"development":o.development,"regulation":null,"anatomy":null,"body":o.body,"meals":0,"feedingCooldown":0.,"x":o.x,"z":o.z}));
        let programs:Vec<_>=selected_cells.iter().take(64).map(|&i|{let c=&self.cells[i];serde_json::json!({"id":c.id,"parent":c.parent,"genome":c.genome.id,"generation":c.generation,"genes":c.genome.genes,"energy":c.energy,"chemicals":c.chemicals,"expression":c.expression,"material":c.material})}).collect();
        if let Some(v) = &mut selected_value {
            v["programCells"] = serde_json::json!(programs);
        }
        let gene_counts: Vec<_> = self.cells.iter().map(|c| c.genome.genes.len()).collect();
        let genomes = self
            .cells
            .iter()
            .map(|c| c.genome.id)
            .collect::<BTreeSet<_>>()
            .len();
        let expressed: Vec<_> = (0..ACTIONS)
            .map(|k| self.cells.iter().filter(|c| c.expression[k] > 0.1).count())
            .collect();
        serde_json::json!({"config":core.config,"lab":null,"study":false,"competition":null,"autonomous":{"actions":if self.physical.is_some(){crate::physical::SYNTHESIS_NAMES}else{ACTION_NAMES},"physics":self.physical.as_ref().map(|h|h.view(&self.cells)),"expressed":expressed,"divisions":self.divisions,"attachedDivisions":self.attached_divisions,"pointEdits":self.point_edits,"duplications":self.duplications,"deletions":self.deletions,"innovations":self.innovations,"exchanges":self.exchanges,"foodEnergy":self.food_energy,"motorWork":self.motor_work,"peakBody":self.peak_body,"peakGeneration":self.peak_generation,"genomeMin":gene_counts.iter().min().copied().unwrap_or(0),"genomeMax":gene_counts.iter().max().copied().unwrap_or(0),"genomes":genomes,"recent":self.history.iter().rev().take(8).collect::<Vec<_>>()},"selected":selected_value,"lineages":lineages.into_values().collect::<Vec<_>>(),"events":core.events,"history":core.history,"stats":{"time":core.time,"population":projections.len(),"cells":self.cells.len(),"colonies":projections.iter().filter(|o|o.cells>1).count(),"diversity":genomes,"generation":self.peak_generation,"births":self.divisions,"deaths":self.deaths,"mating":self.exchanges,"feedings":self.meals,"meanGenes":vec![0.;GENE_COUNT],"nutrient":core.field.iter().sum::<f32>()/core.field.len()as f32,"capped":self.cells.len()>=MAX_CELLS.min(core.config.capacity),"computeMs":0.,"cellBudget":MAX_CELLS.min(core.config.capacity),"cellBudgetReached":self.cells.len()>=MAX_CELLS.min(core.config.capacity),"regulated":projections.len(),"injured":0,"articulated":projections.iter().filter(|o|!o.body.joints.is_empty()).count(),"jointed":projections.iter().filter(|o|!o.body.joints.is_empty()).count(),"contractile":projections.iter().filter(|o|o.body.links.iter().any(|b|b.activation>0.01)).count()}}).to_string()
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
                promoter: [0.; 8],
                bias: 1.,
                strength: 1.,
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
