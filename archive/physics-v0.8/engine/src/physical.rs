//! Reduced, nondimensional physics and chemistry. This is not atomistic chemistry.
//! Conserved material pools, reaction kinetics, D2Q9 fluid, and material mechanics.
use crate::autonomous::{Bond, Cell, Division, Evolution, ACTIONS, MAX_CELLS};
use crate::body::V3;
use crate::*;

const DX: f64 = 64. / GRID as f64;
const Q: f64 = 3.; // chemical energy per unit of reduced organic matter
const FLUID_MASS: f64 = 20.; // solvent mass in each lattice site
const WATER_DENSITY: f64 = 3.;
const TAU: f64 = 0.9;
const NU: f64 = (TAU - 0.5) / 3. * DX * DX / DT as f64;
const C: [(i32, i32); 9] = [
    (0, 0),
    (1, 0),
    (0, 1),
    (-1, 0),
    (0, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
    (1, -1),
];
const W: [f64; 9] = [
    4. / 9.,
    1. / 9.,
    1. / 9.,
    1. / 9.,
    1. / 9.,
    1. / 36.,
    1. / 36.,
    1. / 36.,
    1. / 36.,
];
const OPP: [usize; 9] = [0, 3, 4, 1, 2, 7, 8, 5, 6];
const DIFFUSIVITY: [f64; 5] = [0.12, 0.18, 0.06, 0.3, 0.3];
pub const SYNTHESIS_NAMES: [&str; ACTIONS] = [
    "Photosynthetic catalyst synthesis",
    "Nutrient transporter synthesis",
    "Adhesive polymer synthesis",
    "Mineral-binding catalyst synthesis",
    "Contractile polymer synthesis",
    "Signal A catalyst synthesis",
    "Signal B catalyst synthesis",
    "Replication catalyst synthesis",
    "Contact transporter synthesis",
    "Digestive catalyst synthesis",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub biomass: f64,
    pub fuel: f64,
    pub mineral: f64,
    pub proteins: [f64; ACTIONS],
}
impl Inventory {
    pub fn founder() -> Self {
        let mut proteins = [0.; ACTIONS];
        for k in [0, 1, 2, 7] {
            proteins[k] = 0.002;
        }
        Self {
            biomass: 0.22,
            fuel: 0.04,
            mineral: 0.,
            proteins,
        }
    }
    pub fn organic(&self) -> f64 {
        self.biomass + self.fuel + self.proteins.iter().sum::<f64>()
    }
    pub fn mass(&self) -> f64 {
        self.organic() + self.mineral
    }
    pub fn radius(&self) -> f32 {
        // Organic and mineral densities differ; precipitation increases inertia.
        (0.22 * ((self.organic() + self.mineral * 0.3) / 0.268).cbrt()) as f32
    }
    pub fn activity(&self, k: usize) -> f32 {
        (self.proteins[k] / (0.015 * self.biomass + self.proteins[k]).max(1e-12)) as f32
    }
    fn split(&mut self) -> Self {
        self.biomass *= 0.5;
        self.fuel *= 0.5;
        self.mineral *= 0.5;
        for p in &mut self.proteins {
            *p *= 0.5;
        }
        self.clone()
    }
    fn valid(&self) -> bool {
        [self.biomass, self.fuel, self.mineral]
            .iter()
            .chain(self.proteins.iter())
            .all(|x| x.is_finite() && *x >= 0. && *x < 1e6)
            && self.biomass > 0.
            && self.mass() > 0.
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fluid {
    pub populations: Vec<[f64; 9]>,
}
impl Fluid {
    fn equilibrium(rho: f64, u: [f64; 2]) -> [f64; 9] {
        let mut f = [0.; 9];
        for q in 0..9 {
            let cu = C[q].0 as f64 * u[0] + C[q].1 as f64 * u[1];
            f[q] = W[q] * rho * (1. + 3. * cu + 4.5 * cu * cu - 1.5 * (u[0] * u[0] + u[1] * u[1]));
        }
        f
    }
    pub fn still() -> Self {
        Self {
            populations: vec![W; GRID * GRID],
        }
    }
    pub fn new() -> Self {
        let mut f = Self::still();
        // One low-Mach, approximately divergence-free initial vortex; no forcing
        // or nutrient steering sustains it. No-slip walls dissipate its momentum.
        for z in 0..GRID {
            for x in 0..GRID {
                let a = std::f64::consts::PI * (x as f64 + 0.5) / GRID as f64;
                let b = std::f64::consts::PI * (z as f64 + 0.5) / GRID as f64;
                let u = [
                    0.018 * a.sin().powi(2) * (2. * b).sin(),
                    -0.018 * b.sin().powi(2) * (2. * a).sin(),
                ];
                f.populations[z * GRID + x] = Self::equilibrium(1., u);
            }
        }
        f
    }
    pub fn lattice_velocity(&self, site: usize) -> [f64; 2] {
        let f = &self.populations[site];
        let rho = f.iter().sum::<f64>();
        let mut u = [0.; 2];
        for q in 0..9 {
            u[0] += f[q] * C[q].0 as f64 / rho;
            u[1] += f[q] * C[q].1 as f64 / rho;
        }
        u
    }
    pub fn velocity(&self, site: usize) -> [f64; 2] {
        let u = self.lattice_velocity(site);
        [u[0] * DX / DT as f64, u[1] * DX / DT as f64]
    }
    pub fn momentum(&self) -> [f64; 2] {
        let mut p = [0.; 2];
        for f in &self.populations {
            for q in 0..9 {
                p[0] += f[q] * C[q].0 as f64 * FLUID_MASS * DX / DT as f64;
                p[1] += f[q] * C[q].1 as f64 * FLUID_MASS * DX / DT as f64;
            }
        }
        p
    }
    fn impulse(&mut self, site: usize, impulse: [f64; 2]) {
        // Linear moment update has zero zeroth moment and the exact requested
        // first moment. It is the reaction to drag, not a prescribed current.
        let j = [
            impulse[0] / FLUID_MASS * DT as f64 / DX,
            impulse[1] / FLUID_MASS * DT as f64 / DX,
        ];
        for q in 0..9 {
            self.populations[site][q] += 3. * W[q] * (C[q].0 as f64 * j[0] + C[q].1 as f64 * j[1]);
        }
    }
    pub fn step(&mut self) {
        let mut next = vec![[0.; 9]; GRID * GRID];
        for z in 0..GRID {
            for x in 0..GRID {
                let site = z * GRID + x;
                let f = self.populations[site];
                let rho = f.iter().sum();
                let eq = Self::equilibrium(rho, self.lattice_velocity(site));
                for q in 0..9 {
                    let post = f[q] + (eq[q] - f[q]) / TAU;
                    let xx = x as i32 + C[q].0;
                    let zz = z as i32 + C[q].1;
                    if xx < 0 || zz < 0 || xx >= GRID as i32 || zz >= GRID as i32 {
                        next[site][OPP[q]] += post; // halfway no-slip bounce-back
                    } else {
                        next[zz as usize * GRID + xx as usize][q] += post;
                    }
                }
            }
        }
        self.populations = next;
    }
    fn valid(&self) -> bool {
        self.populations.len() == GRID * GRID
            && self.populations.iter().enumerate().all(|(i, f)| {
                f.iter().all(|v| v.is_finite() && *v > 0. && *v < 2.)
                    && (0.5..1.5).contains(&f.iter().sum::<f64>())
                    && self.lattice_velocity(i).iter().all(|v| v.abs() < 0.18)
            })
    }
}

/// A bending interaction between two mineral-bearing adhesive contacts.
/// Created from actual contact geometry, not a supplied skeleton/body plan.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialJoint {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub rest: f32,
    pub target: f32,
    pub angle: f32,
    pub flexion: f32,
    pub extension: f32,
    pub work: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Habitat {
    #[serde(default)]
    pub joints: Vec<MaterialJoint>,
    #[serde(default)]
    pub passive: bool,
    /// Reduced fuel, oxidized waste, dissolved mineral, signal A, signal B.
    /// Amounts per equal-volume site, not clamped normalized signals.
    pub solutes: Vec<[f64; 5]>,
    pub fluid: Fluid,
    pub initial_organic: f64,
    pub initial_mineral: f64,
    pub initial_energy: f64,
    pub photon_energy: f64,
    pub heat: f64,
    pub mechanical_input: f64,
    pub bound_events: u32,
    pub dissociations: u32,
}
impl Habitat {
    pub fn new(core: &Core, cells: &mut [Cell]) -> Self {
        let mut solutes = vec![[0.; 5]; GRID * GRID];
        for (i, s) in solutes.iter_mut().enumerate() {
            s[0] = core.field[i] as f64;
            s[1] = 0.3;
            let x = (i % GRID) as f32 * DX as f32 - 32.;
            let z = (i / GRID) as f32 * DX as f32 - 32.;
            s[2] = core.minerals_at(x, z) as f64 * 0.4;
        }
        for cell in cells.iter_mut() {
            cell.material = Some(Inventory::founder());
        }
        let mut h = Self {
            joints: Vec::new(),
            passive: core.config.physical_assay == "passive",
            solutes,
            fluid: Fluid::new(),
            initial_organic: 0.,
            initial_mineral: 0.,
            initial_energy: 0.,
            photon_energy: 0.,
            heat: 0.,
            mechanical_input: 0.,
            bound_events: 0,
            dissociations: 0,
        };
        let (organic, mineral, energy) = h.totals(cells);
        h.initial_organic = organic;
        h.initial_mineral = mineral;
        h.initial_energy = energy;
        h
    }
    pub fn totals(&self, cells: &[Cell]) -> (f64, f64, f64) {
        let mut organic = 0.;
        let mut mineral = 0.;
        let mut reduced = 0.;
        let mut atp = 0.;
        for s in &self.solutes {
            organic += s[0] + s[1] + s[3] + s[4];
            reduced += s[0] + s[3] + s[4];
            mineral += s[2];
        }
        for c in cells {
            if let Some(m) = &c.material {
                organic += m.organic();
                reduced += m.organic();
                mineral += m.mineral;
                atp += c.energy as f64;
            }
        }
        (organic, mineral, reduced * Q + atp)
    }
    pub fn errors(&self, cells: &[Cell]) -> [f64; 3] {
        let (o, m, e) = self.totals(cells);
        [
            o - self.initial_organic,
            m - self.initial_mineral,
            e + self.heat + self.mechanical_input - self.initial_energy - self.photon_energy,
        ]
    }
    pub fn valid(&self, cells: &[Cell]) -> bool {
        let ids: BTreeSet<_> = cells.iter().map(|c| c.id).collect();
        let mut keys = BTreeSet::new();
        self.joints.len() <= MAX_CELLS * 3
            && self.joints.iter().all(|j| {
                ids.contains(&j.a)
                    && ids.contains(&j.b)
                    && ids.contains(&j.c)
                    && j.a < j.c
                    && j.a != j.b
                    && j.b != j.c
                    && keys.insert((j.a, j.b, j.c))
                    && [j.rest, j.target, j.angle]
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=std::f32::consts::PI).contains(v))
                    && [j.flexion, j.extension]
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=1.).contains(v))
                    && j.work.is_finite()
                    && j.work >= 0.
            })
            && self.solutes.len() == GRID * GRID
            && self
                .solutes
                .iter()
                .flatten()
                .all(|v| v.is_finite() && *v >= 0. && *v < 1e6)
            && self.fluid.valid()
            && cells
                .iter()
                .all(|c| c.material.as_ref().is_some_and(Inventory::valid))
            && [
                self.initial_organic,
                self.initial_mineral,
                self.initial_energy,
                self.photon_energy,
                self.heat,
                self.mechanical_input,
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.)
            && self
                .errors(cells)
                .iter()
                .all(|v| v.abs() < 1e-6 * (1. + self.initial_energy))
    }
    pub fn transport(&mut self, dt: f64) {
        // Conservative donor-cell advection + central Fickian diffusion.
        // Four substeps keep outgoing coefficients below one for the validated
        // low-Mach domain. No concentration clamp can erase/create material.
        let velocity: Vec<_> = (0..GRID * GRID).map(|i| self.fluid.velocity(i)).collect();
        for _ in 0..4 {
            let old = &self.solutes;
            let mut delta = vec![[0.; 5]; GRID * GRID];
            for z in 0..GRID {
                for x in 0..GRID {
                    let i = z * GRID + x;
                    for (j, axis) in [
                        (if x + 1 < GRID { Some(i + 1) } else { None }, 0),
                        (if z + 1 < GRID { Some(i + GRID) } else { None }, 1),
                    ] {
                        if let Some(j) = j {
                            let u = (velocity[i][axis] + velocity[j][axis]) * 0.5;
                            for k in 0..5 {
                                let donor = if u >= 0. { old[i][k] } else { old[j][k] };
                                let flux = dt / 4.
                                    * (u / DX * donor
                                        + DIFFUSIVITY[k] / (DX * DX) * (old[i][k] - old[j][k]));
                                delta[i][k] -= flux;
                                delta[j][k] += flux;
                            }
                        }
                    }
                }
            }
            for i in 0..GRID * GRID {
                for k in 0..5 {
                    self.solutes[i][k] += delta[i][k];
                }
            }
        }
        for s in &mut self.solutes {
            for k in [3, 4] {
                let decayed = s[k] * (1. - (-0.015 * dt).exp());
                s[k] -= decayed;
                s[1] += decayed;
                self.heat += decayed * Q;
            }
        }
    }
    fn spend(&mut self, c: &mut Cell, amount: f64, mechanical: bool) -> f64 {
        let before = c.energy as f64;
        c.energy = (before - amount.min(before)).max(0.) as f32;
        let paid = before - c.energy as f64;
        if mechanical {
            self.mechanical_input += paid;
        } else {
            self.heat += paid;
        }
        paid
    }
    fn physiology(&mut self, e: &mut Evolution, core: &Core) {
        let dt = DT as f64;
        let mut degree = BTreeMap::<u32, usize>::new();
        for b in &e.bonds {
            *degree.entry(b.a).or_default() += 1;
            *degree.entry(b.b).or_default() += 1;
        }
        for c in &mut e.cells {
            let site = field_index(c.p.x, c.p.z);
            let s = &mut self.solutes[site];
            let m = c.material.as_mut().unwrap();
            c.chemicals = [(s[3] / (0.03 + s[3])) as f32, (s[4] / (0.03 + s[4])) as f32];
            c.expression = c.genome.express([
                (s[0] / (0.5 + s[0])) as f32,
                (s[2] / (0.1 + s[2])) as f32,
                (c.energy / 10.).min(1.),
                (degree.get(&c.id).copied().unwrap_or(0) as f32 / 6.).min(1.),
                c.chemicals[0],
                c.chemicals[1],
                (c.age / 80.).min(1.),
                (c.v.len() / 2.).min(1.),
            ]);
            let activity: Vec<_> = (0..ACTIONS).map(|k| m.activity(k) as f64).collect();
            // Transport moves actual reduced carbon into the cell.
            let uptake = (dt * 0.35 * activity[1] * core.config.solvent as f64)
                .min(s[0])
                .min((m.biomass * 1.5 - m.fuel).max(0.));
            s[0] -= uptake;
            m.fuel += uptake;
            // W + light -> reduced C. This is a deliberately abstract redox
            // reaction. Its chemical energy is supplied by tracked photons.
            let photo = (dt * 0.06 * activity[0] * core.config.sunlight as f64)
                .min(s[1])
                .min((m.biomass * 1.5 - m.fuel).max(0.));
            s[1] -= photo;
            m.fuel += photo;
            self.photon_energy += photo * Q;
            let burn = (dt * (0.035 + 0.12 * activity[1]))
                .min(m.fuel)
                .min(((9. - c.energy as f64) / Q).max(0.));
            m.fuel -= burn;
            s[1] += burn;
            let before = c.energy as f64;
            c.energy = (before + burn * Q) as f32;
            self.heat += burn * Q - (c.energy as f64 - before);
            e.food_energy += burn * Q;
            // Gene output is a production rate. Mechanical effects use actual
            // protein/mineral inventories, never promoter output directly.
            for k in 0..ACTIONS {
                let amount = (dt * 0.012 * c.expression[k] as f64)
                    .min(m.fuel)
                    .min(c.energy as f64);
                m.fuel -= amount;
                m.proteins[k] += amount;
                let before = c.energy as f64;
                c.energy = (before - amount) as f32;
                self.heat += before - c.energy as f64;
                let turnover = m.proteins[k] * (1. - (-0.025 * dt).exp());
                m.proteins[k] -= turnover;
                m.fuel += turnover;
            }
            let growth = (dt * 0.04 * activity[7] * (1. - m.biomass / 0.5).max(0.))
                .min(m.fuel)
                .min(c.energy as f64 / 0.4);
            m.fuel -= growth;
            m.biomass += growth;
            let before = c.energy as f64;
            c.energy = (before - growth * 0.4) as f32;
            self.heat += before - c.energy as f64;
            // Reversible mineral association: k_on * catalyst * M - k_off * B.
            let bound = (dt * 0.025 * activity[3] * s[2])
                .min(s[2])
                .min(c.energy as f64 / 0.2);
            s[2] -= bound;
            m.mineral += bound;
            let before = c.energy as f64;
            c.energy = (before - bound * 0.2) as f32;
            self.heat += before - c.energy as f64;
            let released = m.mineral * (1. - (-0.006 * dt).exp());
            m.mineral -= released;
            s[2] += released;
            for k in 0..2 {
                let secreted = (dt * 0.015 * activity[5 + k])
                    .min(m.fuel)
                    .min(c.energy as f64 / 0.3);
                m.fuel -= secreted;
                s[3 + k] += secreted;
                let before = c.energy as f64;
                c.energy = (before - secreted * 0.3) as f32;
                self.heat += before - c.energy as f64;
            }
            let upkeep = dt * (0.055 + c.genome.genes.len() as f64 * 0.004 + m.organic() * 0.015);
            self.spend(c, upkeep, false);
            c.age += DT;
            c.cooldown = (c.cooldown - DT).max(0.);
        }
    }
    fn contacts(&mut self, e: &mut Evolution, core: &mut Core) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        // Keep exact inventory-dependent radii, refreshing only after digestion
        // moves matter. Broad-phase candidates need not repeatedly compute cbrt.
        let mut radii: Vec<_> = e.cells.iter().map(Cell::radius).collect();
        let mut degree = vec![0usize; e.cells.len()];
        let ids = e.index();
        let mut bonded = BTreeSet::new();
        for b in &e.bonds {
            degree[ids[&b.a]] += 1;
            degree[ids[&b.b]] += 1;
            bonded.insert((b.a, b.b));
        }
        // Broad phase by spatial bins. Only geometric contact enables binding,
        // digestion and exchange; no prey direction or supplied kin rule.
        let mut bins = vec![Vec::new(); GRID * GRID];
        for (i, c) in e.cells.iter().enumerate() {
            bins[field_index(c.p.x, c.p.z)].push(i);
        }
        for i in 0..e.cells.len() {
            let site = field_index(e.cells[i].p.x, e.cells[i].p.z);
            let (x, z) = (site % GRID, site / GRID);
            for zz in z.saturating_sub(1)..=(z + 1).min(GRID - 1) {
                for xx in x.saturating_sub(1)..=(x + 1).min(GRID - 1) {
                    for &j in &bins[zz * GRID + xx] {
                        if j <= i {
                            continue;
                        }
                        let d = e.cells[j].p.sub(e.cells[i].p).len();
                        if d > radii[i] + radii[j] + 0.12 {
                            continue;
                        }
                        pairs.push((i, j));
                        let key = (
                            e.cells[i].id.min(e.cells[j].id),
                            e.cells[i].id.max(e.cells[j].id),
                        );
                        let adhesion = (e.cells[i].activity(2) * e.cells[j].activity(2)) as f64;
                        if !bonded.contains(&key)
                            && degree[i] < 6
                            && degree[j] < 6
                            && (core.random() as f64) < 1. - (-0.8 * adhesion * DT as f64).exp()
                        {
                            e.bonds.push(Bond {
                                a: key.0,
                                b: key.1,
                                rest: d.max(0.1),
                                integrity: 1.,
                                activation: 0.,
                                stiffness: 0.,
                            });
                            degree[i] += 1;
                            degree[j] += 1;
                            bonded.insert(key);
                            self.bound_events += 1;
                        }
                        for (a, b) in [(i, j), (j, i)] {
                            let enzyme = e.cells[a].activity(9) as f64;
                            if enzyme > 0. && e.cells[a].energy > 0.1 {
                                let prey = e.cells[b].material.as_mut().unwrap();
                                let extracted = (DT as f64 * 0.015 * enzyme
                                    / (1. + prey.mineral / prey.biomass * 4.))
                                    .min((prey.biomass - 0.005).max(0.));
                                prey.biomass -= extracted;
                                e.cells[a].material.as_mut().unwrap().fuel += extracted;
                                self.spend(&mut e.cells[a], extracted * 0.2, false);
                                if extracted > 0. {
                                    radii[a] = e.cells[a].radius();
                                    radii[b] = e.cells[b].radius();
                                    e.meals += 1;
                                }
                            }
                            if e.cells[a].activity(8) > 0.1
                                && e.cells[a].energy > 0.2
                                && e.cells[a].genome.id != e.cells[b].genome.id
                                && e.cells[a].genome.genes.len() < autonomous::MAX_GENES
                                && core.random() < 0.002
                            {
                                let donor = e.cells[b].genome.genes[(core.random()
                                    * e.cells[b].genome.genes.len() as f32)
                                    as usize]
                                    .clone();
                                e.cells[a].genome.genes.push(donor);
                                e.cells[a].genome.id = e.next_genome;
                                e.next_genome += 1;
                                self.spend(&mut e.cells[a], 0.1, false);
                                e.exchanges += 1;
                            }
                        }
                    }
                }
            }
        }
        pairs
    }
    #[cfg(test)]
    fn drag(&mut self, c: &mut Cell, dt: f64) {
        let m = c.material.as_ref().unwrap();
        let mass = m.mass();
        let gamma = 6. * std::f64::consts::PI * WATER_DENSITY * NU * m.radius() as f64;
        self.drag_cached(c, mass, gamma * dt / mass);
    }
    fn drag_cached(&mut self, c: &mut Cell, mass: f64, ratio: f64) {
        let site = field_index(c.p.x, c.p.z);
        let u = self.fluid.velocity(site);
        let old = c.v;
        c.v.x = ((old.x as f64 + ratio * u[0]) / (1. + ratio)) as f32;
        c.v.z = ((old.z as f64 + ratio * u[1]) / (1. + ratio)) as f32;
        c.v.y = (old.y as f64 / (1. + ratio)) as f32;
        self.fluid.impulse(
            site,
            [
                -mass * (c.v.x as f64 - old.x as f64),
                -mass * (c.v.z as f64 - old.z as f64),
            ],
        );
    }
    fn sync_joints(&mut self, e: &Evolution) {
        let ids = e.index();
        let mut neighbors = vec![Vec::new(); e.cells.len()];
        let mineral_fraction = |i: usize| {
            let m = e.cells[i].material.as_ref().unwrap();
            (m.mineral / m.mass()) as f32
        };
        for b in &e.bonds {
            let (a, c) = (ids[&b.a], ids[&b.b]);
            if b.integrity > 0. && mineral_fraction(a).min(mineral_fraction(c)) > 0.025 {
                neighbors[a].push(b.b);
                neighbors[c].push(b.a);
            }
        }
        let previous: BTreeMap<_, _> = std::mem::take(&mut self.joints)
            .into_iter()
            .map(|j| ((j.a, j.b, j.c), j))
            .collect();
        for (b, ns) in neighbors.iter_mut().enumerate() {
            ns.sort_unstable();
            // Bound bending constraints to three adjacent pairs per cell.
            for ends in ns.windows(2).take(3) {
                let (a, c) = (ends[0], ends[1]);
                let key = (a, e.cells[b].id, c);
                let angle = crate::articulation::angle_and_gradients(
                    e.cells[ids[&a]].p,
                    e.cells[b].p,
                    e.cells[ids[&c]].p,
                )
                .0;
                let joint = previous.get(&key).cloned().unwrap_or(MaterialJoint {
                    a,
                    b: e.cells[b].id,
                    c,
                    rest: angle,
                    target: angle,
                    angle,
                    flexion: 0.,
                    extension: 0.,
                    work: 0.,
                });
                self.joints.push(joint);
            }
        }
    }
    fn joint_k(e: &Evolution, a: usize, b: usize, c: usize) -> f32 {
        [a, b, c]
            .into_iter()
            .map(|i| {
                let m = e.cells[i].material.as_ref().unwrap();
                (m.mineral / m.mass()) as f32
            })
            .fold(1., f32::min)
            * 0.08
    }
    fn prepare_joint_motors(&mut self, e: &mut Evolution) {
        let ids = e.index();
        // Temporarily take the vector so ATP accounting can borrow the habitat.
        let mut joints = std::mem::take(&mut self.joints);
        for j in &mut joints {
            let (a, b, c) = (ids[&j.a], ids[&j.b], ids[&j.c]);
            let angle =
                crate::articulation::angle_and_gradients(e.cells[a].p, e.cells[b].p, e.cells[c].p)
                    .0;
            let cell = &e.cells[b];
            let motor = if self.passive { 0. } else { cell.activity(4) };
            let flexion = motor * cell.chemicals[0];
            let extension = motor * cell.chemicals[1];
            let next = (j.rest + (extension - flexion) * 0.45).clamp(0., std::f32::consts::PI);
            let k = Self::joint_k(e, a, b, c) as f64;
            let increase =
                (0.5 * k * ((angle - next).powi(2) - (angle - j.target).powi(2)) as f64).max(0.);
            let required = increase + DT as f64 * 0.001 * (flexion + extension) as f64;
            let fraction = if required > 0. {
                (e.cells[b].energy as f64 / required).min(1.)
            } else {
                1.
            };
            j.target += (next - j.target) * fraction as f32;
            j.flexion = flexion * fraction as f32;
            j.extension = extension * fraction as f32;
            let paid = self.spend(&mut e.cells[b], required * fraction, true);
            j.work += paid;
            e.motor_work += paid;
            j.angle = angle;
        }
        self.joints = joints;
    }
    fn mechanics(&mut self, e: &mut Evolution, core: &mut Core, pairs: &[(usize, usize)]) {
        self.sync_joints(e);
        self.prepare_joint_motors(e);
        let ids = e.index();
        for b in &mut e.bonds {
            let (a, c) = (ids[&b.a], ids[&b.b]);
            let (left, right) = e.cells.split_at_mut(c.max(a));
            let (x, y) = if a < c {
                (&mut left[a], &mut right[0])
            } else {
                (&mut right[0], &mut left[c])
            };
            // Contact transport conservatively redistributes fuel molecules.
            let mobility = (x.activity(8) + y.activity(8)) as f64 * 0.5;
            let (xm, ym) = (x.material.as_mut().unwrap(), y.material.as_mut().unwrap());
            let flow = ((xm.fuel - ym.fuel) * 0.08 * mobility).clamp(-ym.fuel, xm.fuel);
            xm.fuel -= flow;
            ym.fuel += flow;
            b.stiffness = ((xm.mineral / xm.mass() + ym.mineral / ym.mass()) * 0.5).min(1.) as f32;
            let catalyst = (x.activity(4) + y.activity(4)) * 0.5;
            let drive = (x.chemicals[0] + y.chemicals[0])
                * 0.5
                * (1. - (x.chemicals[1] + y.chemicals[1]) * 0.5);
            let next = if self.passive {
                0.
            } else {
                b.activation + (catalyst * drive - b.activation) * 0.2
            };
            let d = x.p.sub(y.p).len() as f64;
            let k = Self::spring_k(b);
            let old_energy =
                0.5 * k * (d - b.rest as f64 * (1. - 0.22 * b.activation as f64)).powi(2);
            let new_energy = 0.5 * k * (d - b.rest as f64 * (1. - 0.22 * next as f64)).powi(2);
            let required = (new_energy - old_energy).max(0.) + DT as f64 * 0.001 * next as f64;
            let fraction = if required > 0. {
                ((x.energy.min(y.energy) as f64 * 2.) / required).min(1.)
            } else {
                1.
            };
            b.activation += (next - b.activation) * fraction as f32;
            let paid = self.spend(x, required * fraction * 0.5, true)
                + self.spend(y, required * fraction * 0.5, true);
            e.motor_work += paid;
        }
        let step = DT / 20.;
        // Material inventories do not change within the mechanical substeps.
        // Avoid repeated cbrt, mass sums and tree lookups; preserve force order.
        let properties: Vec<_> = e
            .cells
            .iter()
            .map(|c| {
                let m = c.material.as_ref().unwrap();
                let mass = m.mass();
                let radius = m.radius();
                let volume = 4. / 3. * std::f64::consts::PI * (radius as f64).powi(3);
                let weight = 0.3 * (mass - WATER_DENSITY * volume) as f32;
                let gamma = 6. * std::f64::consts::PI * WATER_DENSITY * NU * radius as f64;
                (mass, radius, weight, gamma * step as f64 / mass)
            })
            .collect();
        let springs: Vec<_> = e
            .bonds
            .iter()
            .map(|b| {
                (
                    ids[&b.a],
                    ids[&b.b],
                    b.rest * (1. - b.activation * 0.22),
                    Self::spring_k(b) as f32,
                )
            })
            .collect();
        let angular: Vec<_> = self
            .joints
            .iter()
            .map(|j| {
                let (a, b, c) = (ids[&j.a], ids[&j.b], ids[&j.c]);
                (a, b, c, j.target, Self::joint_k(e, a, b, c))
            })
            .collect();
        for _ in 0..20 {
            let mut force = vec![V3::default(); e.cells.len()];
            for &(a, c, target, k) in &springs {
                let delta = e.cells[c].p.sub(e.cells[a].p);
                let d = delta.len().max(1e-6);
                let f = delta.mul((d - target) * k / d);
                force[a] = force[a].add(f);
                force[c] = force[c].sub(f);
            }
            for &(a, b, c, target, k) in &angular {
                let (angle, gradients) = crate::articulation::angle_and_gradients(
                    e.cells[a].p,
                    e.cells[b].p,
                    e.cells[c].p,
                );
                let torque = (-(angle - target) * k).clamp(-0.04, 0.04);
                for (i, grad) in [a, b, c].into_iter().zip(gradients) {
                    force[i] = force[i].add(grad.mul(torque));
                }
            }
            for &(i, j) in pairs {
                let delta = e.cells[j].p.sub(e.cells[i].p);
                let d = delta.len().max(1e-6);
                let overlap = properties[i].1 + properties[j].1 - d;
                if overlap > 0. {
                    let f = delta.mul(overlap * 4. / d);
                    force[i] = force[i].sub(f);
                    force[j] = force[j].add(f);
                }
            }
            for (i, c) in e.cells.iter_mut().enumerate() {
                let (mass, radius, weight, drag_ratio) = properties[i];
                force[i].y -= weight;
                c.v = c.v.add(force[i].mul(step / mass as f32));
                self.drag_cached(c, mass, drag_ratio);
                c.p = c.p.add(c.v.mul(step));
                for axis in [0, 2] {
                    let (p, v) = if axis == 0 {
                        (&mut c.p.x, &mut c.v.x)
                    } else {
                        (&mut c.p.z, &mut c.v.z)
                    };
                    if *p < -30. || *p > 30. {
                        *p = (*p).clamp(-30., 30.);
                        *v = 0.;
                    }
                }
                let floor = body::habitat_y(c.p.x, c.p.z, radius, core.config.seed);
                if c.p.y < floor {
                    c.p.y = floor;
                    c.v.y = c.v.y.max(0.);
                    let friction = (1. - step * 3.).max(0.);
                    c.v.x *= friction;
                    c.v.z *= friction;
                }
            }
        }
        e.bonds.retain(|b| {
            let x = &e.cells[ids[&b.a]];
            let y = &e.cells[ids[&b.b]];
            let strain = (x.p.sub(y.p).len() / b.rest - 1.).abs();
            let rate =
                0.004 * (1. + strain as f64 * 8.) / (0.1 + (x.activity(2) * y.activity(2)) as f64);
            let survive = strain < 2. && (core.random() as f64) >= 1. - (-rate * DT as f64).exp();
            if !survive {
                self.dissociations += 1;
            }
            survive
        });
    }
    fn spring_k(b: &Bond) -> f64 {
        (1. + b.stiffness as f64 * 60.) * b.integrity as f64
    }
    fn divide(&mut self, e: &mut Evolution, core: &mut Core) {
        let n = e.cells.len();
        let mut daughters = Vec::new();
        let mut degree = BTreeMap::<u32, usize>::new();
        for b in &e.bonds {
            *degree.entry(b.a).or_default() += 1;
            *degree.entry(b.b).or_default() += 1;
        }
        for i in 0..n {
            if n + daughters.len() >= MAX_CELLS.min(core.config.capacity) {
                break;
            }
            let c = &e.cells[i];
            if c.material.as_ref().unwrap().biomass < 0.38
                || c.energy < 5.5
                || c.cooldown > 0.
                || core.random() > c.activity(7) * DT * 0.5
            {
                continue;
            }
            self.spend(&mut e.cells[i], 1., false);
            let mut child = e.cells[i].clone();
            child.id = e.next_cell;
            e.next_cell += 1;
            child.parent = e.cells[i].id;
            child.generation += 1;
            child.age = 0.;
            child.cooldown = 3.;
            let edits = child.genome.mutate(core);
            if edits.iter().sum::<u32>() > 0 {
                child.genome.id = e.next_genome;
                e.next_genome += 1;
            }
            e.point_edits += edits[0];
            e.duplications += edits[1];
            e.deletions += edits[2];
            e.innovations += edits[3];
            child.material = Some(e.cells[i].material.as_mut().unwrap().split());
            child.energy = e.cells[i].energy * 0.5;
            e.cells[i].energy = child.energy;
            e.cells[i].cooldown = 3.;
            let angle = core.random() * std::f32::consts::TAU;
            let dist = child.radius() + e.cells[i].radius();
            let delta = V3::new(angle.cos() * dist, 0., angle.sin() * dist);
            child.p = child.p.add(delta.mul(0.5));
            e.cells[i].p = e.cells[i].p.sub(delta.mul(0.5));
            child.p.x = child.p.x.clamp(-30., 30.);
            child.p.z = child.p.z.clamp(-30., 30.);
            e.cells[i].p.x = e.cells[i].p.x.clamp(-30., 30.);
            e.cells[i].p.z = e.cells[i].p.z.clamp(-30., 30.);
            let attached = degree.get(&child.parent).copied().unwrap_or(0) < 6
                && core.random() < child.activity(2) * e.cells[i].activity(2);
            if attached {
                e.bonds.push(Bond {
                    a: child.parent,
                    b: child.id,
                    rest: dist,
                    integrity: 1.,
                    activation: 0.,
                    stiffness: 0.,
                });
                e.attached_divisions += 1;
            }
            e.divisions += 1;
            e.peak_generation = e.peak_generation.max(child.generation);
            e.history.push(Division {
                time: core.time,
                parent: child.parent,
                child: child.id,
                genome: child.genome.id,
                generation: child.generation,
                attached,
                edits: edits.iter().sum(),
            });
            if e.history.len() > 256 {
                e.history.remove(0);
            }
            daughters.push(child);
        }
        e.cells.extend(daughters);
    }
    fn recycle(&mut self, e: &mut Evolution) {
        e.cells.retain(|c| {
            let m = c.material.as_ref().unwrap();
            let alive = c.energy > 0.0001 && m.biomass >= 0.02;
            if !alive {
                let site = field_index(c.p.x, c.p.z);
                self.solutes[site][0] += m.organic();
                self.solutes[site][2] += m.mineral;
                self.heat += c.energy as f64;
                e.deaths += 1;
            }
            alive
        });
        let alive: BTreeSet<_> = e.cells.iter().map(|c| c.id).collect();
        e.bonds
            .retain(|b| alive.contains(&b.a) && alive.contains(&b.b));
    }
    pub fn tick(&mut self, e: &mut Evolution, core: &mut Core) {
        core.time += DT;
        self.fluid.step();
        self.transport(DT as f64);
        self.physiology(e, core);
        let pairs = self.contacts(e, core);
        self.mechanics(e, core, &pairs);
        self.divide(e, core);
        self.recycle(e);
        self.sync_joints(e); // remove broken contacts/dead endpoints before save/display
        let ids = e.index();
        for j in &mut self.joints {
            j.angle = crate::articulation::angle_and_gradients(
                e.cells[ids[&j.a]].p,
                e.cells[ids[&j.b]].p,
                e.cells[ids[&j.c]].p,
            )
            .0;
        }
        for (i, s) in self.solutes.iter().enumerate() {
            core.field[i] = s[0] as f32;
        } // display adapter only
        e.peak_body = e
            .peak_body
            .max(e.groups().iter().map(Vec::len).max().unwrap_or(0));
        core.births = e.divisions;
        core.deaths = e.deaths;
        core.mating = e.exchanges;
        core.feedings = e.meals;
        if core.time - core.history_clock >= 3. {
            let groups = e.groups();
            core.history.push(History {
                time: core.time,
                population: groups.len(),
                cells: e.cells.len() as u32,
                colonies: groups.iter().filter(|g| g.len() > 1).count(),
                diversity: e
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
    pub fn view(&self, cells: &[Cell]) -> serde_json::Value {
        let (organic, mineral, energy) = self.totals(cells);
        let errors = self.errors(cells);
        let speed = (0..GRID * GRID)
            .map(|i| {
                let u = self.fluid.velocity(i);
                (u[0] * u[0] + u[1] * u[1]).sqrt()
            })
            .fold(0., f64::max);
        serde_json::json!({"assay":if self.passive {"passive"}else{"full"},"joints":self.joints.len(),"angularWork":self.joints.iter().map(|j|j.work).sum::<f64>(),"organic":organic,"mineral":mineral,"chemicalEnergy":energy,"photonEnergy":self.photon_energy,"heat":self.heat,
            "mechanicalInput":self.mechanical_input,"massError":errors[0],"mineralError":errors[1],"energyError":errors[2],
            "fluidSpeed":speed,"flow":(0..GRID*GRID).step_by(4).map(|i|{let u=self.fluid.velocity(i);[i as f64,u[0],u[1]]}).collect::<Vec<_>>(),"boundEvents":self.bound_events,"dissociations":self.dissociations,
            "proteinMass":cells.iter().map(|c|c.material.as_ref().unwrap().proteins.iter().sum::<f64>()).sum::<f64>(),
            "boundMineral":cells.iter().map(|c|c.material.as_ref().unwrap().mineral).sum::<f64>()})
    }
}

impl Cell {
    pub fn radius(&self) -> f32 {
        self.material.as_ref().map_or(0.22, Inventory::radius)
    }
    pub fn activity(&self, k: usize) -> f32 {
        self.material
            .as_ref()
            .map_or(self.expression[k], |m| m.activity(k))
    }
}
impl Core {
    pub fn start_physical(&mut self) {
        self.start_autonomous();
        self.config.name = "Physical cellular habitat".into();
        let mut e = self.autonomous.take().unwrap();
        e.physical = Some(Habitat::new(self, &mut e.cells));
        self.autonomous = Some(e);
        self.event("world","Physical cellular habitat: finite conserved chemical pools, diffusion and fluid transport. Genes regulate material synthesis; bonds and contraction follow material stores. Chemistry is a reduced model, not molecular abiogenesis.".into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn joint_fixture(passive: bool) -> (Core, Evolution, Habitat) {
        let mut core = Core::new(Config {
            initial_cells: 3,
            physical_assay: if passive {
                "passive".into()
            } else {
                "full".into()
            },
            ..Config::default()
        })
        .unwrap();
        let mut e = Evolution::new(&mut core);
        let mut h = Habitat::new(&core, &mut e.cells);
        for (c, p) in e.cells.iter_mut().zip([
            V3::new(0., 5., 0.),
            V3::new(0.5, 5., 0.),
            V3::new(0.5, 5., 0.5),
        ]) {
            c.p = p;
            c.material.as_mut().unwrap().mineral = 0.08;
            c.material.as_mut().unwrap().proteins[4] = 0.03;
            c.chemicals = [1., 0.];
        }
        for (a, b) in [(1, 2), (2, 3)] {
            e.bonds.push(Bond {
                a,
                b,
                rest: 0.5,
                integrity: 1.,
                activation: 0.,
                stiffness: 0.,
            });
        }
        (h.initial_organic, h.initial_mineral, h.initial_energy) = h.totals(&e.cells);
        (core, e, h)
    }
    #[test]
    fn material_joints_require_deposition_and_intact_contacts() {
        let (_, mut e, mut h) = joint_fixture(false);
        h.sync_joints(&e);
        assert_eq!(h.joints.len(), 1);
        let rest = h.joints[0].rest;
        e.cells[2].p.z += 0.1;
        h.sync_joints(&e);
        assert_eq!(h.joints[0].rest, rest);
        e.bonds.pop();
        h.sync_joints(&e);
        assert!(h.joints.is_empty());
        let (_, mut e, mut h) = joint_fixture(false);
        e.cells[1].material.as_mut().unwrap().mineral = 0.;
        h.sync_joints(&e);
        assert!(h.joints.is_empty());
    }
    #[test]
    fn angular_actuation_is_paid_passive_and_atp_gated() {
        for (passive, energy, active) in [(false, 5., true), (true, 5., false), (false, 0., false)]
        {
            let (_, mut e, mut h) = joint_fixture(passive);
            for c in &mut e.cells {
                c.energy = energy;
            }
            (h.initial_organic, h.initial_mineral, h.initial_energy) = h.totals(&e.cells);
            h.sync_joints(&e);
            h.prepare_joint_motors(&mut e);
            assert_eq!(h.joints[0].work > 0., active);
            assert_eq!(h.joints[0].flexion > 0., active);
            assert!(h.errors(&e.cells).iter().all(|v| v.abs() < 1e-9));
            if active {
                assert!(h.joints[0].target < h.joints[0].rest);
            }
        }
    }
    #[test]
    fn absent_contractile_material_never_actuates_a_sharp_rest_angle() {
        let (_, mut e, mut h) = joint_fixture(false);
        e.cells[2].p = V3::new(0., 5., 0.0001);
        for c in &mut e.cells {
            c.material.as_mut().unwrap().proteins[4] = 0.;
        }
        h.sync_joints(&e);
        let rest = h.joints[0].rest;
        h.prepare_joint_motors(&mut e);
        assert_eq!(h.joints[0].target, rest);
        assert_eq!(h.joints[0].work, 0.);
    }
    #[test]
    fn angular_internal_forces_have_no_net_force_or_torque() {
        let (_, e, mut h) = joint_fixture(false);
        h.sync_joints(&e);
        let ps: [V3; 3] = [e.cells[0].p, e.cells[1].p, e.cells[2].p];
        let (_, grad) = crate::articulation::angle_and_gradients(ps[0], ps[1], ps[2]);
        let sum = grad.into_iter().fold(V3::default(), |sum, g| sum.add(g));
        assert!(sum.len() < 1e-5);
        let cross = |a: V3, b: V3| {
            V3::new(
                a.y * b.z - a.z * b.y,
                a.z * b.x - a.x * b.z,
                a.x * b.y - a.y * b.x,
            )
        };
        let torque = ps
            .into_iter()
            .zip(grad)
            .fold(V3::default(), |sum, (p, g)| sum.add(cross(p, g)));
        assert!(torque.len() < 1e-5);
    }
    #[test]
    fn deposited_joint_world_resumes_and_rejects_invalid_joint_contacts() {
        let (mut core, mut e, h) = joint_fixture(false);
        e.physical = Some(h);
        core.organisms.clear();
        core.autonomous = Some(e);
        core.step(1);
        assert!(core.autonomous.as_ref().unwrap().valid());
        let mut copy = Core::load(&serde_json::to_string(&core).unwrap()).unwrap();
        core.step(3);
        copy.step(3);
        assert_eq!(
            serde_json::to_string(&core).unwrap(),
            serde_json::to_string(&copy).unwrap()
        );
        let mut bad: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&core).unwrap()).unwrap();
        if !bad["autonomous"]["physical"]["joints"]
            .as_array()
            .unwrap()
            .is_empty()
        {
            bad["autonomous"]["physical"]["joints"][0]["c"] = 99999.into();
            assert!(Core::load(&bad.to_string()).is_err());
        }
    }
    #[test]
    fn configured_population_and_fission_exceed_the_old_1024_ceiling() {
        let mut core = Core::new(Config {
            initial_cells: 1100,
            capacity: 1200,
            ..Config::default()
        })
        .unwrap();
        let mut e = Evolution::new(&mut core);
        let mut h = Habitat::new(&core, &mut e.cells);
        assert_eq!(e.cells.len(), 1100);
        assert_eq!(e.next_cell, 1101);
        for c in &mut e.cells {
            let m = c.material.as_mut().unwrap();
            m.biomass = 0.4;
            m.proteins[7] = 0.1;
            c.energy = 6.;
            c.cooldown = 0.;
        }
        (h.initial_organic, h.initial_mineral, h.initial_energy) = h.totals(&e.cells);
        h.divide(&mut e, &mut core);
        assert_eq!(e.cells.len(), 1200);
        assert!(e.cells.iter().any(|c| c.generation == 1));
        let count = e.cells.len();
        h.divide(&mut e, &mut core);
        assert_eq!(count, e.cells.len());
        assert!(h.errors(&e.cells).iter().all(|v| v.abs() < 1e-7));
        e.physical = Some(h);
        assert!(e.valid());
        core.organisms.clear();
        core.autonomous = Some(e);
        let restored = Core::load(&serde_json::to_string(&core).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_string(&core).unwrap(),
            serde_json::to_string(&restored).unwrap()
        );
    }
    #[test]
    fn setup_bounds_allow_64000_cells_and_keep_old_save_defaults() {
        assert!(Config {
            capacity: 64000,
            initial_cells: 8192,
            ..Config::default()
        }
        .validate()
        .is_ok());
        assert!(Config {
            capacity: 64001,
            ..Config::default()
        }
        .validate()
        .is_err());
        assert!(Config {
            initial_cells: 8193,
            ..Config::default()
        }
        .validate()
        .is_err());
        let mut old = serde_json::to_value(Config::default()).unwrap();
        old.as_object_mut().unwrap().remove("initialCells");
        old.as_object_mut().unwrap().remove("physicalAssay");
        let c: Config = serde_json::from_value(old).unwrap();
        assert_eq!(c.initial_cells, 64);
        assert_eq!(c.physical_assay, "full");
    }
    #[test]
    fn larger_world_ticks_conserve_material_and_stay_finite() {
        let mut core = Core::new(Config {
            initial_cells: 1536,
            capacity: 8192,
            ..Config::default()
        })
        .unwrap();
        core.start_physical();
        core.step(4);
        let e = core.autonomous.as_ref().unwrap();
        assert!(e.valid());
        assert!(e
            .physical
            .as_ref()
            .unwrap()
            .errors(&e.cells)
            .iter()
            .all(|v| v.abs() < 1e-7));
    }
    #[test]
    fn unforced_vortex_dissipates_and_stays_in_low_mach_domain() {
        let mut f = Fluid::new();
        let energy = |fluid: &Fluid| -> f64 {
            (0..GRID * GRID)
                .map(|i| {
                    let u = fluid.lattice_velocity(i);
                    u[0] * u[0] + u[1] * u[1]
                })
                .sum()
        };
        let before = energy(&f);
        let mass = f.populations.iter().flatten().sum::<f64>();
        for _ in 0..160 {
            f.step();
            assert!(f.valid());
        }
        assert!(energy(&f) < before * 0.8);
        assert!((f.populations.iter().flatten().sum::<f64>() - mass).abs() < 1e-9);
    }
    #[test]
    fn mineral_binding_requires_both_catalyst_material_and_local_mineral() {
        let mut core = Core::new(Config::default()).unwrap();
        core.start_physical();
        let mut e = core.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        e.cells.truncate(1);
        e.cells[0].genome.genes.push(autonomous::Gene {
            action: 3,
            promoter: [0.; 8],
            bias: 1.,
            strength: 1.,
        });
        let site = field_index(e.cells[0].p.x, e.cells[0].p.z);
        h.solutes[site][2] = 0.;
        for _ in 0..20 {
            h.physiology(&mut e, &core);
        }
        assert!(e.cells[0].material.as_ref().unwrap().proteins[3] > 0.);
        assert_eq!(e.cells[0].material.as_ref().unwrap().mineral, 0.);
        h.solutes[site][2] = 0.5;
        let before = h.solutes[site][2];
        h.physiology(&mut e, &core);
        let m = e.cells[0].material.as_ref().unwrap();
        assert!(m.mineral > 0.);
        assert!((h.solutes[site][2] + m.mineral - before).abs() < 1e-12);
    }
    #[test]
    fn material_contraction_is_paid_and_empty_atp_prevents_activation() {
        let mut core = Core::new(Config::default()).unwrap();
        core.start_physical();
        let mut e = core.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        e.cells.truncate(2);
        h.fluid = Fluid::still();
        for (i, c) in e.cells.iter_mut().enumerate() {
            c.p = V3::new(i as f32 * 0.6, 10., 0.);
            c.v = V3::default();
            c.chemicals = [1., 0.];
            c.material.as_mut().unwrap().proteins[4] = 0.05;
        }
        e.bonds.push(Bond {
            a: 1,
            b: 2,
            rest: 0.6,
            integrity: 1.,
            activation: 0.,
            stiffness: 0.,
        });
        let mut empty = e.clone();
        for c in &mut empty.cells {
            c.energy = 0.;
        }
        let before = e.cells.iter().map(|c| c.energy as f64).sum::<f64>();
        h.mechanics(&mut e, &mut core, &[]);
        assert!(e.bonds[0].activation > 0. && e.motor_work > 0.);
        let after = e.cells.iter().map(|c| c.energy as f64).sum::<f64>();
        assert!((before - after - e.motor_work).abs() < 1e-12);
        h.mechanics(&mut empty, &mut core, &[]);
        assert_eq!(empty.bonds[0].activation, 0.);
        assert_eq!(empty.motor_work, 0.);
    }
    #[test]
    fn closed_fluid_preserves_mass_and_rest_equilibrium() {
        let mut f = Fluid::still();
        let before = f.populations.iter().flatten().sum::<f64>();
        for _ in 0..80 {
            f.step();
        }
        assert!((f.populations.iter().flatten().sum::<f64>() - before).abs() < 1e-9);
        assert!(f.momentum().iter().all(|v| v.abs() < 1e-9));
        assert!(f.valid());
    }
    #[test]
    fn diffusion_spreads_a_pulse_without_loss_or_negative_concentrations() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_physical();
        let e = c.autonomous.as_mut().unwrap();
        let h = e.physical.as_mut().unwrap();
        h.fluid = Fluid::still();
        h.solutes = vec![[0.; 5]; GRID * GRID];
        let site = 20 * GRID + 20;
        h.solutes[site][0] = 1.;
        for _ in 0..100 {
            h.transport(DT as f64);
        }
        assert!((h.solutes.iter().map(|s| s[0]).sum::<f64>() - 1.).abs() < 1e-12);
        assert!(h.solutes[site][0] < 0.5 && h.solutes[site + 1][0] > 0.);
        assert!(h.solutes.iter().flatten().all(|v| *v >= 0.));
    }
    #[test]
    fn drag_transfers_equal_opposite_momentum_to_solvent() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_physical();
        let mut e = c.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        h.fluid = Fluid::still();
        let cell = &mut e.cells[0];
        cell.v = V3::new(0.2, 0., -0.1);
        let mass = cell.material.as_ref().unwrap().mass();
        let before = [mass * cell.v.x as f64, mass * cell.v.z as f64];
        h.drag(cell, 0.01);
        let solvent = h.fluid.momentum();
        assert!((mass * cell.v.x as f64 + solvent[0] - before[0]).abs() < 1e-12);
        assert!((mass * cell.v.z as f64 + solvent[1] - before[1]).abs() < 1e-12);
        assert!(cell.v.x < 0.2 && cell.v.z.abs() < 0.1);
    }
    #[test]
    fn expression_requires_material_production_before_mechanical_activity() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_physical();
        let mut e = c.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        let cell = &mut e.cells[0];
        cell.genome.genes.push(autonomous::Gene {
            action: 3,
            promoter: [0.; 8],
            bias: 1.,
            strength: 1.,
        });
        assert_eq!(cell.activity(3), 0.);
        h.physiology(&mut e, &c);
        assert!(e.cells[0].material.as_ref().unwrap().proteins[3] > 0.);
        assert!(h.errors(&e.cells).iter().all(|v| v.abs() < 1e-8));
    }
    #[test]
    fn fission_splits_existing_material_and_copies_with_mutation() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_physical();
        let mut e = c.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        for cell in &mut e.cells {
            cell.material.as_mut().unwrap().biomass = 0.4;
            cell.energy = 8.;
            cell.cooldown = 0.;
        }
        let before = h.totals(&e.cells);
        for _ in 0..100 {
            h.divide(&mut e, &mut c);
        }
        let after = h.totals(&e.cells);
        assert!(e.divisions > 0);
        assert!((after.0 - before.0).abs() < 1e-10);
        assert!((after.1 - before.1).abs() < 1e-10);
        assert!((after.2 + h.heat - before.2).abs() < 1e-9);
    }
    #[test]
    fn coupled_world_conserves_matter_energy_and_resumes_exactly() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_physical();
        c.step(120);
        let e = c.autonomous.as_ref().unwrap();
        let h = e.physical.as_ref().unwrap();
        assert!(h.valid(&e.cells), "{:?}", h.errors(&e.cells));
        let save = serde_json::to_string(&c).unwrap();
        let mut other = Core::load(&save).unwrap();
        c.step(8);
        other.step(8);
        assert_eq!(
            serde_json::to_string(&c).unwrap(),
            serde_json::to_string(&other).unwrap()
        );
    }
    #[test]
    fn corrupted_chemical_store_is_rejected() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_physical();
        c.autonomous
            .as_mut()
            .unwrap()
            .physical
            .as_mut()
            .unwrap()
            .solutes[0][0] += 1.;
        assert!(Core::load(&serde_json::to_string(&c).unwrap()).is_err());
    }
}
