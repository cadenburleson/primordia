//! Reduced, nondimensional physics and chemistry. This is not atomistic chemistry.
//! Conserved material pools, reaction kinetics, D2Q9 fluid, and material mechanics.
use crate::autonomous::{Bond, Cell, Division, Evolution, ACTIONS, INPUTS, LATERAL, MAX_CELLS};
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
const DIFFUSIVITY: [f64; 6] = [0.12, 0.18, 0.06, 0.3, 0.3, 0.01];
fn solute_array<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<[f64; 6]>, D::Error> {
    Vec::<Vec<f64>>::deserialize(d)?
        .into_iter()
        .map(|values| {
            if values.len() != 5 && values.len() != 6 {
                return Err(serde::de::Error::custom("Invalid solute vocabulary"));
            }
            let mut s = [0.; 6];
            s[..values.len()].copy_from_slice(&values);
            Ok(s)
        })
        .collect()
}
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
    "Lipid storage catalyst synthesis",
    "Local receptor synthesis",
    "Excitable channel synthesis",
    "Matrix repair catalyst synthesis",
    "Protective membrane synthesis",
    "Detritus-breakdown catalyst synthesis",
    "Photoreceptor synthesis",
    "Adaptive contact polymer synthesis",
    "Gametic division catalyst synthesis",
    "Membrane fusogen synthesis",
    "Polar scaffold synthesis",
    "Extracellular matrix secretion",
    "Hydraulic pump synthesis",
    "Gas exchanger synthesis",
    "Mechanosensitive channel synthesis",
    "Membrane recognition synthesis",
    "Replication repair synthesis",
    "Matrix digesting enzyme synthesis",
    "Fiber crosslinker synthesis",
    "Mineral remodeling enzyme synthesis",
    "Sequence polymer assembler synthesis",
    "Membrane lysis catalyst synthesis",
    "Propulsion motor synthesis",
    "Steering apparatus synthesis",
    "Engulfing membrane synthesis",
    "Venom synthesis",
    "Spine synthesis",
    "Chemosynthetic catalyst synthesis",
];
pub const PROPEL: usize = 32;
pub const STEER: usize = 33;
pub const ENGULF: usize = 34;
pub const TOXIN: usize = 35;
pub const SPINE: usize = 36;
pub const CHEMO: usize = 37;
/// Share of a cell's own feeding (light, vents, dissolved fuel) given up by a
/// fully built engulfing membrane. Low enough that a hunter between meals can
/// limp along on light, as many single-celled hunters do.
const MOUTH_COST: f64 = 0.85;
/// Thrust of a fully built, fully driven propulsion motor.
const THRUST: f64 = 2.5;
/// Fastest turn of a fully built steering apparatus, radians per cycle.
const TURN_RATE: f32 = 1.2;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub biomass: f64,
    pub fuel: f64,
    pub mineral: f64,
    #[serde(default)]
    pub lipid: f64,
    #[serde(default)]
    pub body: crate::organics::BodyStores,
    #[serde(
        deserialize_with = "crate::autonomous::protein_array",
        serialize_with = "crate::autonomous::as_list"
    )]
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
            lipid: 0.,
            body: crate::organics::BodyStores::default(),
            proteins,
        }
    }
    pub fn organic(&self) -> f64 {
        self.biomass
            + self.fuel
            + self.lipid
            + self.proteins.iter().sum::<f64>()
            + self.body.organic()
    }
    pub fn reduced(&self) -> f64 {
        self.organic() - self.body.solutes[1]
    }
    pub fn mineral_total(&self) -> f64 {
        self.mineral + self.body.solutes[2]
    }
    pub fn mass(&self) -> f64 {
        self.organic() + self.mineral_total() + self.body.water * 3.
    }
    pub fn radius(&self) -> f32 {
        // Organic and mineral densities differ; precipitation increases inertia.
        (0.22
            * ((self.organic() + self.lipid * 0.6 + self.mineral * 0.3 + self.body.water * 3.)
                / 0.268)
                .cbrt()) as f32
    }
    pub fn activity(&self, k: usize) -> f32 {
        (self.proteins[k] / (0.015 * self.biomass + self.proteins[k]).max(1e-12)) as f32
    }
    fn split(&mut self) -> Self {
        self.biomass *= 0.5;
        self.fuel *= 0.5;
        self.mineral *= 0.5;
        self.lipid *= 0.5;
        for p in &mut self.proteins {
            *p *= 0.5;
        }
        self.body.scale(0.5);
        self.clone()
    }
    pub(crate) fn valid(&self) -> bool {
        [self.biomass, self.fuel, self.mineral, self.lipid]
            .iter()
            .chain(self.proteins.iter())
            .all(|x| x.is_finite() && *x >= 0. && *x < 1e6)
            && self.biomass > 0.
            && self.mass() > 0.
            && self.body.valid()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fluid {
    #[serde(default)]
    pub domain: crate::domain::Domain,
    pub populations: Vec<[f64; 9]>,
    #[serde(default)]
    pub surface: Option<crate::surface::SurfaceFlow>,
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
        Self::still_in(crate::domain::Domain::default())
    }
    pub fn still_in(domain: crate::domain::Domain) -> Self {
        Self {
            domain,
            populations: if domain.spherical() {
                vec![]
            } else {
                vec![W; domain.sites()]
            },
            surface: domain
                .spherical()
                .then(|| crate::surface::SurfaceFlow::new(domain)),
        }
    }
    pub fn new() -> Self {
        Self::new_in(crate::domain::Domain::default())
    }
    pub fn new_in(domain: crate::domain::Domain) -> Self {
        let mut f = Self::still_in(domain);
        if domain.spherical() {
            return f;
        }
        let grid = domain.grid;
        // One low-Mach, approximately divergence-free initial vortex; no forcing
        // or nutrient steering sustains it. No-slip walls dissipate its momentum.
        for z in 0..grid {
            for x in 0..grid {
                let a = std::f64::consts::PI * (x as f64 + 0.5) / grid as f64;
                let b = std::f64::consts::PI * (z as f64 + 0.5) / grid as f64;
                let u = [
                    0.018 * a.sin().powi(2) * (2. * b).sin(),
                    -0.018 * b.sin().powi(2) * (2. * a).sin(),
                ];
                f.populations[z * grid + x] = Self::equilibrium(1., u);
            }
        }
        f
    }
    pub fn lattice_velocity(&self, site: usize) -> [f64; 2] {
        if self.domain.spherical() {
            let v = self.surface.as_ref().unwrap().velocities[site];
            let (x, z) = self.domain.point(site);
            let p = self.domain.position(x, z, 0.);
            return [
                v.dot(self.domain.tangent(p, 0.)) as f64 * DT as f64 / DX,
                v.dot(self.domain.tangent(p, std::f32::consts::FRAC_PI_2)) as f64 * DT as f64 / DX,
            ];
        }
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
        if self.domain.spherical() {
            return;
        }
        let mut next = vec![[0.; 9]; self.domain.sites()];
        for z in 0..self.domain.rows() {
            for x in 0..self.domain.grid {
                let site = z * self.domain.grid + x;
                let f = self.populations[site];
                let rho = f.iter().sum();
                let eq = Self::equilibrium(rho, self.lattice_velocity(site));
                for q in 0..9 {
                    let post = f[q] + (eq[q] - f[q]) / TAU;
                    let xx = x as i32 + C[q].0;
                    let zz = z as i32 + C[q].1;
                    if xx < 0
                        || zz < 0
                        || xx >= self.domain.grid as i32
                        || zz >= self.domain.grid as i32
                    {
                        next[site][OPP[q]] += post; // halfway no-slip bounce-back
                    } else {
                        next[zz as usize * self.domain.grid + xx as usize][q] += post;
                    }
                }
            }
        }
        self.populations = next;
    }
    fn valid(&self) -> bool {
        if self.domain.spherical() {
            return self.domain.valid()
                && self.populations.is_empty()
                && self.surface.as_ref().is_some_and(|f| f.valid(self.domain));
        }
        if self.surface.is_some() {
            return false;
        }
        self.domain.valid()
            && self.populations.len() == self.domain.sites()
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
    pub ablation: String,
    #[serde(default)]
    pub meals: Vec<crate::observations::Meal>,
    #[serde(default)]
    pub shade: Vec<f32>,
    #[serde(default)]
    pub domain: crate::domain::Domain,
    #[serde(default)]
    pub planet: Option<crate::planet::Planet>,
    #[serde(default)]
    pub physiology: crate::physiology::Ledger,
    #[serde(default)]
    pub development: crate::development::Development,
    #[serde(default)]
    pub systems: crate::organics::SystemLedger,
    #[serde(default)]
    pub steering: crate::steering::Steering,
    #[serde(default)]
    pub joints: Vec<MaterialJoint>,
    #[serde(default)]
    pub passive: bool,
    /// Fuel, oxidized waste, mineral, signal A/B, and organic detritus.
    #[serde(deserialize_with = "solute_array")]
    pub solutes: Vec<[f64; 6]>,
    pub fluid: Fluid,
    pub initial_organic: f64,
    pub initial_mineral: f64,
    pub initial_energy: f64,
    pub photon_energy: f64,
    /// Chemical energy supplied by the planet's interior to chemosynthesis.
    #[serde(default)]
    pub geothermal_energy: f64,
    pub heat: f64,
    pub mechanical_input: f64,
    pub bound_events: u32,
    pub dissociations: u32,
}
impl Habitat {
    pub fn new(core: &Core, cells: &mut [Cell]) -> Self {
        let mut domain = crate::domain::Domain::from_size(core.config.world_size);
        domain.shape = core.config.world_shape;
        let mut solutes = vec![[0.; 6]; domain.sites()];
        for (i, s) in solutes.iter_mut().enumerate() {
            s[0] = if domain == crate::domain::Domain::default() {
                core.field[i] as f64
            } else {
                (0.25
                    + noise(
                        (i % domain.grid) as f32 / 4.,
                        (i / domain.grid) as f32 / 4.,
                        core.config.seed,
                    ) * 0.75) as f64
                    * core.config.nutrients as f64
                    * 18.
            };
            s[1] = 0.3;
            let (x, z) = domain.point(i);
            let density = domain.area(i) / (DX * DX);
            s[0] *= density;
            s[1] *= density;
            s[2] = core.minerals_at(x, z) as f64 * 0.4 * density;
        }
        let mut shade = Vec::new();
        if core.config.resource_layout == "patchy" {
            let before: f64 = solutes.iter().map(|s| s[0]).sum();
            shade = (0..domain.sites())
                .map(|i| {
                    let (x, z) = domain.point(i);
                    let n = if domain.spherical() {
                        let n = domain.normal(x, z);
                        noise(
                            n.x * 25. + 20.,
                            n.y * 25. + 20.,
                            core.config.seed.wrapping_add(314),
                        )
                    } else {
                        noise(
                            x * 0.25 + 20.,
                            z * 0.25 + 20.,
                            core.config.seed.wrapping_add(314),
                        )
                    };
                    let patch = ((n - 0.5) * 4.).clamp(0., 1.);
                    solutes[i][0] *= 0.001 + patch as f64;
                    0.02 + 0.98 * patch
                })
                .collect();
            let after: f64 = solutes.iter().map(|s| s[0]).sum();
            if after > 0. {
                for s in &mut solutes {
                    s[0] *= before / after;
                }
            }
        }
        for cell in cells.iter_mut() {
            cell.material = Some(Inventory::founder());
            let ground = domain.ground(cell.p.x, cell.p.z, core.config.seed);
            let height = if ground > 0.12 {
                ground + cell.radius()
            } else {
                0.12 + cell.radius() * (1. - 2. * 0.78)
            };
            cell.p = domain.position(cell.p.x, cell.p.z, height);
        }
        let mut h = Self {
            ablation: String::new(),
            meals: Vec::new(),
            shade,
            domain,
            planet: Some({
                let mut p = crate::planet::Planet::new_in(domain);
                p.initialize_surface(core.config.seed);
                p
            }),
            physiology: crate::physiology::Ledger::default(),
            development: crate::development::Development::default(),
            systems: crate::organics::SystemLedger::default(),
            steering: crate::steering::Steering::default(),
            joints: Vec::new(),
            passive: core.config.physical_assay == "passive",
            solutes,
            fluid: Fluid::new_in(domain),
            initial_organic: 0.,
            initial_mineral: 0.,
            initial_energy: 0.,
            photon_energy: 0.,
            geothermal_energy: 0.,
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
            organic += s[0] + s[1] + s[3] + s[4] + s[5];
            reduced += s[0] + s[3] + s[4] + s[5];
            mineral += s[2];
        }
        for c in cells {
            if let Some(m) = &c.material {
                organic += m.organic();
                reduced += m.reduced();
                mineral += m.mineral_total();
                atp += c.energy as f64;
            }
        }
        for particle in &self.systems.particles {
            let s = particle.solutes;
            organic += s[0] + s[1] + s[3] + s[4] + s[5];
            reduced += s[0] + s[3] + s[4] + s[5];
            mineral += s[2];
        }
        (organic, mineral, reduced * Q + atp)
    }
    pub fn errors(&self, cells: &[Cell]) -> [f64; 3] {
        let (o, m, e) = self.totals(cells);
        [
            o - self.initial_organic,
            m - self.initial_mineral - self.planet.as_ref().map_or(0., |p| p.mineral_input),
            e + self.heat + self.mechanical_input
                - self.initial_energy
                - self.photon_energy
                - self.geothermal_energy,
        ]
    }
    pub fn valid(&self, cells: &[Cell]) -> bool {
        let ids: BTreeSet<_> = cells.iter().map(|c| c.id).collect();
        let mut keys = BTreeSet::new();
        ["", "motors", "transport", "signals"].contains(&self.ablation.as_str())
            && self.steering.valid()
            && self.meals.len() <= 24
            && self.meals.iter().all(|m| {
                m.time.is_finite() && m.time >= 0. && m.amount.is_finite() && m.amount >= 0.
            })
            && (self.shade.is_empty()
                || self.shade.len() == self.domain.sites()
                    && self
                        .shade
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=1.).contains(v)))
            && self.domain.valid()
            && self.fluid.domain == self.domain
            && self.planet.as_ref().is_none_or(|p| p.domain == self.domain)
            && self.physiology.valid()
            && self.development.valid(cells, &self.joints)
            && self.systems.valid(self.domain, cells)
            && self.planet.as_ref().is_none_or(|p| {
                (p.biological_water
                    - cells
                        .iter()
                        .map(|c| c.material.as_ref().map_or(0., |m| m.body.water))
                        .sum::<f64>())
                .abs()
                    < 1e-7 * (1. + p.initial_water)
            })
            && self
                .planet
                .as_ref()
                .is_none_or(crate::planet::Planet::valid)
            && self.joints.len() <= MAX_CELLS * 3
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
            && self.solutes.len() == self.domain.sites()
            && self
                .solutes
                .iter()
                .flatten()
                .all(|v| v.is_finite() && *v >= 0. && *v < 1e12)
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
        let diffusivity = self.diffusivity(DIFFUSIVITY);
        if self.domain.spherical() {
            crate::surface::SurfaceFlow::diffuse(self.domain, &mut self.solutes, dt, diffusivity);
            self.decay_signals(dt);
            return;
        }
        // Conservative donor-cell advection + central Fickian diffusion.
        // Four substeps keep outgoing coefficients below one for the validated
        // low-Mach domain. No concentration clamp can erase/create material.
        let velocity: Vec<_> = (0..self.domain.sites())
            .map(|i| self.fluid.velocity(i))
            .collect();
        for _ in 0..4 {
            let old = &self.solutes;
            let mut delta = vec![[0.; 6]; self.domain.sites()];
            for z in 0..self.domain.rows() {
                for x in 0..self.domain.grid {
                    let i = z * self.domain.grid + x;
                    for (j, axis) in [
                        (
                            if x + 1 < self.domain.grid {
                                Some(i + 1)
                            } else {
                                None
                            },
                            0,
                        ),
                        (
                            if z + 1 < self.domain.grid {
                                Some(i + self.domain.grid)
                            } else {
                                None
                            },
                            1,
                        ),
                    ] {
                        if let Some(j) = j {
                            let u = (velocity[i][axis] + velocity[j][axis]) * 0.5;
                            for k in 0..6 {
                                let donor = if u >= 0. { old[i][k] } else { old[j][k] };
                                let flux = dt / 4.
                                    * (u / DX * donor
                                        + diffusivity[k] / (DX * DX) * (old[i][k] - old[j][k]));
                                delta[i][k] -= flux;
                                delta[j][k] += flux;
                            }
                        }
                    }
                }
            }
            for i in 0..self.domain.sites() {
                for k in 0..6 {
                    self.solutes[i][k] += delta[i][k];
                }
            }
        }
        self.decay_signals(dt);
    }
    fn decay_signals(&mut self, dt: f64) {
        for s in &mut self.solutes {
            for k in [3, 4] {
                let decayed = s[k] * (1. - (-0.015 * dt).exp());
                s[k] -= decayed;
                s[1] += decayed;
                self.heat += decayed * Q;
            }
        }
    }
    pub(crate) fn spend(&mut self, c: &mut Cell, amount: f64, mechanical: bool) -> f64 {
        let before = c.energy as f64;
        c.energy = (before - amount.min(before)).max(0.) as f32;
        let paid = before - c.energy as f64;
        if mechanical {
            self.mechanical_input += paid;
            c.state.motor_work += paid;
        } else {
            self.heat += paid;
        }
        paid
    }
    /// Left-minus-right differences sampled one field spacing to either side
    /// of the heading: dissolved fuel, light, the waste living cells give off,
    /// and organic remains. Each needs the matching receptor material.
    fn lateral_senses(&self, c: &Cell, core: &Core) -> [f32; LATERAL] {
        // A steering apparatus carries a crude chemical sense of its own;
        // dedicated receptors sharpen it.
        let receptor = c
            .activity(crate::physiology::RECEPTOR)
            .max(0.35 * c.activity(STEER));
        let photic = c.activity(crate::development::PHOTIC);
        if receptor <= 0.01 && photic <= 0.01 {
            return [0.; LATERAL];
        }
        let reach = self.domain.spacing().max(c.radius());
        let side = self
            .domain
            .tangent(c.p, c.heading + std::f32::consts::FRAC_PI_2)
            .mul(reach);
        let (l, r) = (
            self.domain.locate(c.p.add(side)),
            self.domain.locate(c.p.sub(side)),
        );
        let level = |i: usize, k: usize| self.solutes[i][k] * 2.56 / self.domain.area(i);
        let contrast = |a: f64, b: f64, half: f64| ((a - b) / (half + a + b)) as f32;
        let light = |i: usize| {
            self.planet.as_ref().map_or(1., |p| p.illumination(i))
                * self.light_fraction(i)
                * core.config.sunlight as f64
        };
        [
            contrast(level(l, 0), level(r, 0), 0.5) * receptor,
            contrast(light(l), light(r), 0.05) * photic,
            // The scent of other life: the waste it gives off across the
            // field, and whatever is alive close by.
            // A cell's own colony is the strongest source of waste around it,
            // so the far field counts for less than what is close by.
            ((0.3 * contrast(level(l, 1), level(r, 1), 0.5) + c.state.nearby) * receptor)
                .clamp(-1., 1.),
            contrast(level(l, 5), level(r, 5), 0.5) * receptor,
        ]
    }
    fn physiology(&mut self, e: &mut Evolution, core: &Core) {
        let mut degree = BTreeMap::<u32, usize>::new();
        for b in &e.bonds {
            *degree.entry(b.a).or_default() += 1;
            *degree.entry(b.b).or_default() += 1;
        }
        for c in &mut e.cells {
            let site = self.domain.locate(c.p);
            c.chemicals = [
                (self.solutes[site][3]
                    / (0.03 * self.domain.area(site) / 2.56 + self.solutes[site][3]))
                    as f32,
                (self.solutes[site][4]
                    / (0.03 * self.domain.area(site) / 2.56 + self.solutes[site][4]))
                    as f32,
            ];
            let body = &c.material.as_ref().unwrap().body;
            if body.water > 1e-9 {
                let fraction = (body.water / (0.02 + body.water)) as f32;
                for k in 0..2 {
                    let concentration = body.solutes[3 + k] / body.water;
                    let internal = (concentration / (0.03 + concentration)) as f32;
                    c.chemicals[k] = c.chemicals[k] * (1. - fraction) + internal * fraction;
                }
            }
            let inputs = c.physical_inputs(
                &self.solutes,
                degree.get(&c.id).copied().unwrap_or(0),
                self.domain,
            );
            let mut inputs = inputs;
            let lateral = self.lateral_senses(c, core);
            inputs[INPUTS - LATERAL..].copy_from_slice(&lateral);
            c.expression = c.cellular_expression(inputs);
            c.state.turn = {
                let turn = c.genome.steer(lateral);
                c.lifecycle
                    .homolog
                    .as_ref()
                    .map_or(turn, |other| (turn + other.steer(lateral)) * 0.5)
            };
            self.reserves_and_repair(c);
            let dt = DT as f64 * self.planet.as_ref().map_or(1., |p| p.kinetics(site));
            let illumination = self.planet.as_ref().map_or(1., |p| p.illumination(site))
                * self.light_fraction(site)
                * self.steering.light as f64;
            let s = &mut self.solutes[site];
            let m = c.material.as_mut().unwrap();
            let activity: Vec<_> = (0..ACTIONS)
                .map(|k| m.activity(k) as f64 * (1. - c.state.injury * 0.75) as f64)
                .collect();
            // Transport moves actual reduced carbon into the cell.
            // A membrane given over to swallowing prey has little room left for
            // soaking up dissolved fuel, harvesting light or vent chemistry: a
            // cell is mostly a feeder or mostly a hunter.
            let autotrophy = 1. - MOUTH_COST * activity[ENGULF];
            let uptake = (dt
                * 0.35
                * activity[1]
                * autotrophy
                * core.config.solvent as f64
                * c.state.exposure as f64)
                .min(s[0])
                    .min((m.biomass * 1.5 - m.fuel).max(0.));
            s[0] -= uptake;
            m.fuel += uptake * 0.9;
            s[5] += uptake * 0.1; // unassimilated organic excretion, not oxidized waste
            self.physiology.excreted += uptake * 0.1;
            c.state.flux.fuel += uptake * 0.9;
            c.state.flux.organic_waste += uptake * 0.1;
            let decomposed = (dt * 0.05 * activity[15] * s[5] / (0.05 + s[5]))
                .min(s[5])
                .min((m.biomass * 1.5 - m.fuel).max(0.))
                .min(c.energy as f64 / 0.15);
            s[5] -= decomposed;
            m.fuel += decomposed;
            self.physiology.decomposed += decomposed;
            c.state.flux.detritus += decomposed;
            let before = c.energy as f64;
            c.energy = (before - decomposed * 0.15) as f32;
            self.heat += before - c.energy as f64;
            // W + light -> reduced C. This is a deliberately abstract redox
            // reaction. Its chemical energy is supplied by tracked photons.
            // A captive symbiont harvests light inside the host, whatever the
            // host's own membrane is given over to.
            let harvest = (activity[0] * autotrophy).max(c.state.symbiont as f64);
            let photo = (dt
                * 0.06
                * harvest
                * core.config.sunlight as f64
                * illumination
                * c.state.exposure as f64)
                .min(s[1])
                .min((m.biomass * 1.5 - m.fuel).max(0.));
            s[1] -= photo;
            m.fuel += photo;
            self.photon_energy += photo * Q;
            c.state.flux.photosynthesis += photo;
            // W + mineral chemistry -> reduced C. Works in the dark; the energy is
            // supplied by the planet's interior and tracked like sunlight.
            let mineral = s[2] * 2.56 / self.domain.area(site);
            let vent = self.planet.as_ref().map_or(0., |p| p.heat[site] as f64);
            let chemo = (dt
                * 0.02
                * activity[CHEMO]
                * (mineral / (0.1 + mineral))
                * (1. + 4. * vent)
                * autotrophy
                * self.steering.vents as f64
                * c.state.exposure as f64)
                .min(s[1])
                .min((m.biomass * 1.5 - m.fuel).max(0.));
            s[1] -= chemo;
            m.fuel += chemo;
            self.geothermal_energy += chemo * Q;
            c.state.flux.chemosynthesis += chemo;
            let burn = (dt * (0.035 + 0.12 * activity[1]))
                .min(m.fuel)
                .min(((9. - c.energy as f64) / Q).max(0.));
            m.fuel -= burn;
            s[1] += burn;
            c.state.flux.waste += burn;
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
                c.state.flux.proteins += amount;
                let before = c.energy as f64;
                c.energy = (before - amount) as f32;
                self.heat += before - c.energy as f64;
                let turnover = m.proteins[k] * (1. - (-0.025 * dt).exp());
                m.proteins[k] -= turnover;
                m.fuel += turnover;
            }
            // An engulfing cell grows up to twice the size of an ordinary one,
            // and has to before it divides: bigger than its prey, slower to breed.
            let growth = (dt
                * 0.04
                * activity[7]
                * (1. - m.biomass / (0.5 * (1. + activity[ENGULF]))).max(0.))
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
            c.state.flux.minerals += bound;
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
                if k == 0 {
                    c.state.flux.signal_a += secreted;
                } else {
                    c.state.flux.signal_b += secreted;
                }
                if m.body.water > 1e-9 {
                    m.body.solutes[3 + k] += secreted;
                } else {
                    s[3 + k] += secreted;
                }
                let before = c.energy as f64;
                c.energy = (before - secreted * 0.3) as f32;
                self.heat += before - c.energy as f64;
            }
            let upkeep = dt
                * (0.055
                    + (c.genome.genes.len()
                        + c.lifecycle.homolog.as_ref().map_or(0, |g| g.genes.len()))
                        as f64
                        * 0.004
                    + m.organic() * 0.015
                    + c.state.symbiont as f64 * 0.01);
            self.spend(c, upkeep, false);
            if let Some(p) = &self.planet {
                c.state.injury =
                    (c.state.injury + DT * (p.heat[site] - 0.55).max(0.) * 0.05).min(1.);
            }
            c.age += DT;
            c.cooldown = (c.cooldown - DT).max(0.);
        }
    }
    pub(crate) fn contacts(&mut self, e: &mut Evolution, core: &mut Core) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        // Keep exact inventory-dependent radii, refreshing only after digestion
        // moves matter. Broad-phase candidates need not repeatedly compute cbrt.
        let mut radii: Vec<_> = e.cells.iter().map(Cell::radius).collect();
        // Attacks act between bodies: a body does not eat or sting itself.
        // Bulk is living structure (summed biomass), not the fuel a cell happens
        // to be carrying: a lean hunter is still bigger than a fat prey cell.
        let armed = e
            .cells
            .iter()
            .any(|c| c.activity(9) > 0. || c.activity(ENGULF) > 0. || c.activity(TOXIN) > 0.);
        // Kin are cells whose sets of working genes are at least four-fifths
        // the same. Weapons spare armed kin; they do not spare a stranger just
        // because it carries the same weapon.
        let steering = e.cells.iter().any(|c| c.activity(STEER) > 0.01);
        let genes: Vec<u64> = if armed || steering {
            e.cells
                .iter()
                .map(|c| {
                    std::iter::once(&c.genome)
                        .chain(c.lifecycle.homolog.as_ref())
                        .flat_map(|g| &g.genes)
                        .filter(|g| g.strength > 0.05)
                        .fold(0u64, |set, g| set | 1 << g.action)
                })
                .collect()
        } else {
            Vec::new()
        };
        let kin = |a: usize, b: usize| {
            (genes[a] & genes[b]).count_ones() * 5 >= (genes[a] | genes[b]).count_ones() * 4
        };
        let mut body = Vec::new();
        let mut bulk = Vec::new();
        if armed {
            body = vec![0usize; e.cells.len()];
            for (g, members) in e.groups().iter().enumerate() {
                bulk.push(
                    members
                        .iter()
                        .map(|&i| e.cells[i].material.as_ref().unwrap().biomass)
                        .sum::<f64>(),
                );
                for &i in members {
                    body[i] = g;
                }
            }
        }
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
        let width = (radii.iter().copied().fold(0., f32::max) * 2. + 0.12).max(1.6);
        let bin = |p: V3| {
            (
                (p.x / width).floor() as i32,
                (p.y / width).floor() as i32,
                (p.z / width).floor() as i32,
            )
        };
        let mut bins = BTreeMap::<(i32, i32, i32), Vec<usize>>::new();
        for (i, c) in e.cells.iter().enumerate() {
            bins.entry(bin(c.p)).or_default().push(i);
        }
        for i in 0..e.cells.len() {
            let (x, y, z) = bin(e.cells[i].p);
            // Flatten the 27 adjacent Cartesian bins into a deterministic list.
            let mut neighbors = Vec::new();
            e.cells[i].state.nearby = 0.;
            for yy in y - 1..=y + 1 {
                for zz in z - 1..=z + 1 {
                    for xx in x - 1..=x + 1 {
                        if let Some(ids) = bins.get(&(xx, yy, zz)) {
                            neighbors.extend(ids.iter().copied());
                        }
                    }
                }
            }
            // A cell that can steer feels which side other life is on, within
            // one field spacing: close-range scent. Engulfers ignore their kin.
            if e.cells[i].activity(STEER) > 0.01 {
                let me = &e.cells[i];
                let left = self
                    .domain
                    .tangent(me.p, me.heading + std::f32::consts::FRAC_PI_2);
                let mouth = me.activity(ENGULF);
                let mut pull = 0.;
                for &j in &neighbors {
                    let other = &e.cells[j];
                    let key = (me.id.min(other.id), me.id.max(other.id));
                    if j == i || bonded.contains(&key) {
                        continue;
                    }
                    let delta = other.p.sub(me.p);
                    let d = delta.len();
                    if d < 1.6 && d > 1e-4 {
                        let own = mouth > 0.25 && other.activity(ENGULF) > 0.25 && kin(i, j);
                        if !own {
                            pull += delta.dot(left) / d * (1. - d / 1.6);
                        }
                    }
                }
                e.cells[i].state.nearby = (pull / (1. + pull.abs())).clamp(-1., 1.);
            }
            for j in neighbors {
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
                let material_affinity = |c: &Cell| {
                    c.material
                        .as_ref()
                        .unwrap()
                        .body
                        .properties(c.material.as_ref().unwrap().biomass)[0]
                };
                let adhesion = ((e.cells[i].activity(2) + material_affinity(&e.cells[i]) * 0.2)
                    .min(1.)
                    * (e.cells[j].activity(2) + material_affinity(&e.cells[j]) * 0.2).min(1.)
                    * e.cells[i].recognition(&e.cells[j])
                    // A mouth does not glue itself to its food: engulfers bind
                    // each other, not the cells they could swallow.
                    * (1. - (e.cells[i].activity(ENGULF) - e.cells[j].activity(ENGULF)).abs()))
                    as f64;
                if !bonded.contains(&key)
                    && e.cells[i].lifecycle.phase != crate::development::Phase::Gamete
                    && e.cells[j].lifecycle.phase != crate::development::Phase::Gamete
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
                    let gate = e.cells[a].activity(crate::organics::MATRIX_DIGEST) as f64;
                    if gate > 0. && e.cells[a].energy > 0.1 {
                        let (left, right) = e.cells.split_at_mut(a.max(b));
                        let (attacker, prey) = if a < b {
                            (&mut left[a], &mut right[0])
                        } else {
                            (&mut right[0], &mut left[b])
                        };
                        let protection = 1. + prey.activity(crate::physiology::BARRIER) as f64 * 5.;
                        let (am, pm) = (
                            attacker.material.as_mut().unwrap(),
                            prey.material.as_mut().unwrap(),
                        );
                        let total = pm.body.matrix + pm.body.fibers;
                        let extracted = (DT as f64 * 0.008 * gate / protection)
                            .min(total)
                            .min((am.biomass * 1.5 - am.fuel).max(0.) / 0.85)
                            .min(attacker.energy as f64 / 0.25);
                        let potential = (DT as f64 * 0.008 * gate)
                            .min(total)
                            .min((am.biomass * 1.5 - am.fuel).max(0.) / 0.85)
                            .min(attacker.energy as f64 / 0.25);
                        prey.state.flux.defense_blocked += (potential - extracted).max(0.);
                        if prey.state.flux.eater() {
                            attacker.state.flux.prey_animal += extracted * 0.85;
                        }
                        attacker.state.flux.prey += extracted * 0.85;
                        prey.state.flux.lost_to_predators += extracted;
                        attacker.state.flux.organic_waste += extracted * 0.15;
                        self.record_meal(
                            core.time,
                            attacker.id,
                            prey.id,
                            attacker.genome.id,
                            prey.genome.id,
                            "matrix / fibers",
                            extracted,
                        );
                        let fraction = extracted / total.max(1e-12);
                        pm.body.matrix *= 1. - fraction;
                        pm.body.fibers *= 1. - fraction;
                        am.fuel += extracted * 0.85;
                        self.solutes[self.domain.locate(attacker.p)][5] += extracted * 0.15;
                        self.physiology.excreted += extracted * 0.15;
                        self.systems.matrix_meals += extracted;
                        self.spend(attacker, extracted * 0.25, false);
                        if extracted > 0. {
                            radii[a] = attacker.radius();
                            radii[b] = prey.radius();
                            e.meals += 1;
                            attacker.state.meals += 1;
                        }
                    }
                    let enzyme = e.cells[a].activity(9) as f64;
                    if enzyme > 0. && e.cells[a].energy > 0.1 && body[a] != body[b] {
                        let barrier = e.cells[b].activity(crate::physiology::BARRIER) as f64;
                        // A cell is protected from the enzyme it makes itself, so
                        // digesters can live beside their own kind and eat others.
                        let immune = if kin(a, b) {
                            (e.cells[b].activity(9) as f64 * 4.).min(1.)
                        } else {
                            0.
                        };
                        let capacity = (e.cells[a].material.as_ref().unwrap().biomass * 1.5
                            - e.cells[a].material.as_ref().unwrap().fuel)
                            .max(0.);
                        let available = e.cells[a].energy as f64 / 0.2;
                        let prey = e.cells[b].material.as_mut().unwrap();
                        let extracted = (DT as f64 * 0.015 * enzyme * (1. - immune)
                            / (1. + prey.mineral / prey.biomass * 4. + barrier * 5.))
                            .min((prey.biomass - 0.005).max(0.))
                            .min(capacity / 0.85)
                            .min(available);
                        let potential = (DT as f64 * 0.015 * enzyme)
                            .min((prey.biomass - 0.005).max(0.))
                            .min(capacity / 0.85)
                            .min(available);
                        prey.biomass -= extracted;
                        let damage = (extracted / prey.biomass.max(0.02)) as f32;
                        e.cells[b].state.flux.defense_blocked += (potential - extracted).max(0.);
                        e.cells[b].state.flux.lost_to_predators += extracted;
                        if e.cells[b].state.flux.eater() {
                            e.cells[a].state.flux.prey_animal += extracted * 0.85;
                        }
                        e.cells[a].state.flux.prey += extracted * 0.85;
                        e.cells[a].state.flux.organic_waste += extracted * 0.15;
                        self.record_meal(
                            core.time,
                            e.cells[a].id,
                            e.cells[b].id,
                            e.cells[a].genome.id,
                            e.cells[b].genome.id,
                            "living biomass",
                            extracted,
                        );
                        e.cells[b].state.injury = (e.cells[b].state.injury + damage).min(1.);
                        e.cells[a].material.as_mut().unwrap().fuel += extracted * 0.85;
                        self.solutes[self.domain.locate(e.cells[a].p)][5] += extracted * 0.15;
                        self.physiology.digested += extracted;
                        self.physiology.excreted += extracted * 0.15;
                        self.spend(&mut e.cells[a], extracted * 0.2, false);
                        if extracted > 0. {
                            radii[a] = e.cells[a].radius();
                            radii[b] = e.cells[b].radius();
                            e.meals += 1;
                            e.cells[a].state.meals += 1;
                        }
                    }
                    // Engulfing is fast and takes one cell at a time. A cell held
                    // by its neighbours is hard for a hunter its own size to
                    // pull off and easy for a much bigger one, so living as a
                    // body is a refuge from small hunters and a body's interior
                    // is out of reach altogether. Skin does not help once
                    // swallowed; spines and mineral armour do, and spines wound
                    // whatever tries.
                    let mouth = e.cells[a].activity(ENGULF) as f64;
                    if mouth > 0.
                        && e.cells[a].energy > 0.1
                        && body[a] != body[b]
                        && bulk[body[a]] > 0.8 * e.cells[b].material.as_ref().unwrap().biomass
                    {
                        // The bigger the hunter is than its prey, the faster it
                        // gets it down: nothing against prey a quarter larger
                        // than itself, full speed at half again its size.
                        let outsize =
                            bulk[body[a]] / e.cells[b].material.as_ref().unwrap().biomass;
                        let advantage = ((outsize - 0.8) / 0.7).clamp(0., 1.);
                        let grip = degree[b] as f64 * 1.5 / outsize.clamp(1., 8.);
                        let spines = e.cells[b].activity(SPINE) as f64;
                        // Engulfers recognise and spare other engulfers.
                        // Engulfers spare armed kin, even wounded or half-built.
                        let spared = if kin(a, b) {
                            (e.cells[b].activity(ENGULF) as f64 * 4.).min(1.)
                        } else {
                            0.
                        };
                        let capacity = (e.cells[a].material.as_ref().unwrap().biomass * 1.5
                            - e.cells[a].material.as_ref().unwrap().fuel)
                            .max(0.);
                        let available = e.cells[a].energy as f64 / 0.2;
                        let prey = e.cells[b].material.as_mut().unwrap();
                        let potential = (DT as f64 * 0.3 * mouth * advantage * (1. - spared))
                            .min((prey.biomass - 0.005).max(0.))
                            .min(capacity / 0.85)
                            .min(available);
                        let extracted =
                            potential / (1. + spines * 6. + prey.mineral / prey.biomass * 4. + grip);
                        // Swallowing takes the prey's fuel store along with its body.
                        let swallowed = (prey.fuel * extracted / prey.biomass.max(1e-9))
                            .min(prey.fuel)
                            .min((capacity - extracted * 0.85).max(0.));
                        prey.fuel -= swallowed;
                        prey.biomass -= extracted;
                        let damage = (extracted / prey.biomass.max(0.02)) as f32;
                        e.cells[b].state.flux.defense_blocked += potential - extracted;
                        e.cells[b].state.flux.lost_to_predators += extracted;
                        e.cells[b].state.injury = (e.cells[b].state.injury + damage).min(1.);
                        if e.cells[b].state.flux.eater() {
                            e.cells[a].state.flux.prey_animal += extracted * 0.85 + swallowed;
                        }
                        // Now and then a swallowed light-feeder is not digested
                        // but kept alive inside: the host gains its captive's
                        // light harvesting, and its descendants inherit it.
                        let captive = e.cells[b].activity(0);
                        if extracted > 0.
                            && captive > 0.25
                            && captive > e.cells[a].state.symbiont
                            && core.random() < 0.01
                        {
                            if e.cells[a].state.symbiont == 0. {
                                self.steering.captures += 1;
                            }
                            e.cells[a].state.symbiont = captive.min(1.);
                        }
                        e.cells[a].state.flux.prey += extracted * 0.85 + swallowed;
                        e.cells[a].state.flux.organic_waste += extracted * 0.15;
                        self.record_meal(
                            core.time,
                            e.cells[a].id,
                            e.cells[b].id,
                            e.cells[a].genome.id,
                            e.cells[b].genome.id,
                            "engulfed",
                            extracted + swallowed,
                        );
                        e.cells[a].material.as_mut().unwrap().fuel += extracted * 0.85 + swallowed;
                        self.solutes[self.domain.locate(e.cells[a].p)][5] += extracted * 0.15;
                        self.physiology.digested += extracted;
                        self.physiology.excreted += extracted * 0.15;
                        self.spend(&mut e.cells[a], extracted * 0.2, false);
                        if extracted > 0. {
                            // Caught prey is held: the pair move as one from here
                            // on, sharing their momentum.
                            let (ma, mb) = (
                                e.cells[a].material.as_ref().unwrap().mass() as f32,
                                e.cells[b].material.as_ref().unwrap().mass() as f32,
                            );
                            let shared = e.cells[a]
                                .v
                                .mul(ma / (ma + mb))
                                .add(e.cells[b].v.mul(mb / (ma + mb)));
                            e.cells[a].v = shared;
                            e.cells[b].v = shared;
                        }
                        let pricked = (DT as f64 * 0.05 * spines * mouth) as f32;
                        e.cells[a].state.injury = (e.cells[a].state.injury + pricked).min(1.);
                        e.cells[b].state.flux.wounded += pricked as f64;
                        if extracted > 0. {
                            radii[a] = e.cells[a].radius();
                            radii[b] = e.cells[b].radius();
                            e.meals += 1;
                            e.cells[a].state.meals += 1;
                        }
                    }
                    // Venom wounds any other body it touches: a weapon, and a
                    // reason not to bite. Skin blunts it.
                    let venom = e.cells[a].activity(TOXIN) as f64;
                    if venom > 0. && e.cells[a].energy > 0.05 && body[a] != body[b] {
                        let wanted = DT as f64 * 0.01 * venom;
                        let paid = self.spend(&mut e.cells[a], wanted, false);
                        let skin = e.cells[b].activity(crate::physiology::BARRIER) as f64;
                        // Venomous cells are immune to venom.
                        let immune = if kin(a, b) {
                            (e.cells[b].activity(TOXIN) as f64 * 4.).min(1.)
                        } else {
                            0.
                        };
                        let dose = (DT as f64 * 0.03 * venom * (1. - immune) * (paid / wanted).min(1.)
                            / (1. + skin * 4.)) as f32;
                        e.cells[b].state.injury = (e.cells[b].state.injury + dose).min(1.);
                        e.cells[a].state.flux.wounded += dose as f64;
                    }
                    if e.cells[a].activity(8) > 0.1
                        && core.random() >= e.cells[a].state.germline * 0.95
                        && e.cells[a].lifecycle.phase != crate::development::Phase::Gamete
                        && e.cells[b].lifecycle.phase != crate::development::Phase::Gamete
                        && e.cells[a].energy > 0.2
                        && e.cells[a].genome.id != e.cells[b].genome.id
                        && e.cells[a].genome.genes.len() < autonomous::MAX_GENES
                        && core.random() < 0.002
                    {
                        let donor = e.cells[b].genome.genes
                            [(core.random() * e.cells[b].genome.genes.len() as f32) as usize]
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
        pairs
    }
    #[cfg(test)]
    fn drag(&mut self, c: &mut Cell, dt: f64) {
        let m = c.material.as_ref().unwrap();
        let mass = m.mass();
        let gamma = 6. * std::f64::consts::PI * WATER_DENSITY * NU * m.radius() as f64;
        let site = self.domain.locate(c.p);
        self.drag_cached(c, mass, gamma * dt / mass, site);
    }
    fn drag_cached(&mut self, c: &mut Cell, mass: f64, ratio: f64, site: usize) {
        if let Some(flow) = &mut self.fluid.surface {
            let water = self.planet.as_ref().unwrap().water[site];
            if water < 1e-9 {
                return;
            }
            let fluid_mass = water * WATER_DENSITY;
            let up = self.domain.up(c.p);
            let tangent = c.v.sub(up.mul(c.v.dot(up)));
            let uv = flow.velocities[site];
            let uv = uv.sub(up.mul(uv.dot(up)));
            let delta = uv
                .sub(tangent)
                .mul((ratio / (1. + ratio * (1. + mass / fluid_mass))) as f32);
            c.v = c.v.add(delta);
            flow.velocities[site] =
                flow.velocities[site].sub(delta.mul((mass / fluid_mass) as f32));
            let normal = up.mul(c.v.dot(up));
            c.v = c.v.sub(normal).add(normal.mul((1. / (1. + ratio)) as f32));
            return;
        }
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
            let signals = cell.motor_signals();
            let flexion = motor * signals[0];
            let extension = motor * signals[1];
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
            let (xm, ym) = (x.material.as_ref().unwrap(), y.material.as_ref().unwrap());
            let polymer =
                (xm.body.properties(xm.biomass)[2] + ym.body.properties(ym.biomass)[2]) * 0.5;
            let scaffold = (x.scaffold() + y.scaffold()) * 0.5;
            b.stiffness = (((xm.mineral / xm.mass() + ym.mineral / ym.mass()) * 0.5) as f32
                + scaffold * 0.08
                + polymer * 0.05)
                .min(1.);
            let fiber_axis = x.p.sub(y.p).normalized();
            let alignment = (fiber_axis.dot(self.domain.tangent(x.p, x.heading)).abs()
                + fiber_axis.dot(self.domain.tangent(y.p, y.heading)).abs())
                * 0.5;
            let fiber_strength = ((xm.body.fibers / (0.02 * xm.biomass + xm.body.fibers)) as f32
                + (ym.body.fibers / (0.02 * ym.biomass + ym.body.fibers)) as f32)
                * 0.5;
            b.stiffness = (b.stiffness + fiber_strength * alignment * 0.1).min(1.);
            let catalyst = (x.activity(4) + y.activity(4)) * 0.5;
            let xs = x.motor_signals();
            let ys = y.motor_signals();
            let drive = (xs[0] + ys[0]) * 0.5 * (1. - (xs[1] + ys[1]) * 0.5);
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
                let wet = if self.domain.spherical() {
                    let p = self.planet.as_ref().unwrap();
                    let site = self.domain.locate(c.p);
                    let (x, z) = self.domain.coords(c.p);
                    let surface = self.domain.ground(x, z, core.config.seed)
                        + p.relief_at(x, z)
                        + p.surface_depth(site) as f32;
                    ((surface - (c.p.len() - self.domain.radius()) + radius) / (2. * radius))
                        .clamp(0., 1.) as f64
                } else {
                    1.
                };
                let weight = 0.3 * (mass - WATER_DENSITY * volume * wet) as f32;
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
        for c in &mut e.cells {
            c.state.strain = 0.;
        }
        // Self-propulsion. A motor pushes the cell along its heading and the
        // medium the other way; the work against drag is paid in ATP. Steering
        // turns the heading by the cell's own sensed left-right differences.
        let mut thrust = vec![V3::default(); e.cells.len()];
        // Crawlers meet the ground's resistance where swimmers meet the water's.
        let mut crawling = vec![false; e.cells.len()];
        if !self.passive {
            for (i, c) in e.cells.iter_mut().enumerate() {
                let steer = c.activity(STEER);
                if steer > 0.01 && c.state.turn != 0. {
                    let wanted = DT as f64 * 0.01 * (steer * c.state.turn.abs()) as f64;
                    let paid = self.spend(c, wanted, false);
                    c.heading = (c.heading
                        + TURN_RATE * DT * steer * c.state.turn * (paid / wanted).min(1.) as f32)
                        .rem_euclid(std::f32::consts::TAU);
                }
                let motor = (c.activity(PROPEL) * c.expression[PROPEL]) as f64;
                c.state.wet = 0.;
                if motor <= 1e-4 {
                    continue;
                }
                // A free cell tumbles; this alone gives an undirected search.
                if !c.state.ground {
                    c.heading = (c.heading + (core.random() - 0.5) * 0.5 * DT.sqrt())
                        .rem_euclid(std::f32::consts::TAU);
                }
                let site = self.domain.locate(c.p);
                let water = self
                    .planet
                    .as_ref()
                    .filter(|_| self.domain.spherical())
                    .map_or(f64::INFINITY, |p| p.water[site]);
                // A swimmer can push off only as much water as there is: full
                // thrust in open water, almost none in a film. A cell resting on
                // the ground crawls at half strength instead. Off the ground and
                // out of water there is nothing to push on.
                let (mass, _, _, drag_ratio) = properties[i];
                let fluid_mass = water * WATER_DENSITY;
                let swim = if !self.domain.spherical() {
                    1.
                } else {
                    fluid_mass / (fluid_mass + mass)
                };
                let medium = if c.state.ground && swim < 0.5 {
                    crawling[i] = true;
                    0.5
                } else {
                    swim
                };
                c.state.wet = medium as f32;
                let gamma = drag_ratio * mass / step as f64;
                let wanted = THRUST * motor * medium;
                let cost = (wanted * wanted / gamma + 0.01 * motor) * DT as f64;
                let paid = self.spend(c, cost, true);
                e.motor_work += paid;
                let force = wanted * (paid / cost).min(1.);
                if force <= 0. {
                    continue;
                }
                let direction = self.domain.tangent(c.p, c.heading);
                thrust[i] = direction.mul(force as f32);
                let impulse = force * DT as f64;
                if let Some(flow) = &mut self.fluid.surface {
                    if !crawling[i] && water > 1e-9 {
                        let kick = (impulse / fluid_mass).min(0.2) as f32;
                        flow.velocities[site] = flow.velocities[site].sub(direction.mul(kick));
                    }
                } else {
                    self.fluid.impulse(
                        site,
                        [-(direction.x as f64) * impulse, -(direction.z as f64) * impulse],
                    );
                }
            }
        }
        // Terrain and the field site under a cell change little within one
        // tick, so they are sampled on every fourth substep and reused between.
        let mut sites: Vec<usize> = e.cells.iter().map(|c| self.domain.locate(c.p)).collect();
        let mut terrain = vec![(0f32, false); e.cells.len()];
        for substep in 0..20 {
            let mut force = thrust.clone();
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
                let gravity = if self.domain.spherical() {
                    (self.domain.radius() / c.p.len().max(1e-4)).powi(2)
                } else {
                    1.
                };
                force[i] = force[i].sub(self.domain.up(c.p).mul(weight * gravity));
                c.v = c.v.add(force[i].mul(step / mass as f32));
                self.drag_cached(c, mass, drag_ratio, sites[i]);
                if crawling[i] {
                    let up = self.domain.up(c.p);
                    let normal = up.mul(c.v.dot(up));
                    c.v = normal.add(c.v.sub(normal).mul((1. / (1. + drag_ratio)) as f32));
                }
                c.p = c.p.add(c.v.mul(step));
                if !self.domain.spherical() {
                    for axis in [0, 2] {
                        let (p, v) = if axis == 0 {
                            (&mut c.p.x, &mut c.v.x)
                        } else {
                            (&mut c.p.z, &mut c.v.z)
                        };
                        if p.abs() > self.domain.wall() {
                            *p = p.clamp(-self.domain.wall(), self.domain.wall());
                            *v = 0.;
                        }
                    }
                }
                let p = self.planet.as_ref().unwrap();
                if substep % 4 == 0 {
                    let (x, z) = self.domain.coords(c.p);
                    sites[i] = self.domain.index(x, z);
                    terrain[i] = (
                        p.floor(x, z, radius, core.config.seed),
                        if self.domain.spherical() {
                            p.surface_depth(sites[i]) < 0.1
                        } else {
                            self.domain.ground(x, z, core.config.seed) + p.relief_at(x, z) > 0.12
                        },
                    );
                }
                let (floor, firm) = terrain[i];
                let altitude = if self.domain.spherical() {
                    c.p.len() - self.domain.radius()
                } else {
                    c.p.y
                };
                let ground = altitude <= floor + 0.001 && firm;
                if ground != c.state.ground {
                    self.physiology.contact_changes += 1;
                }
                c.state.ground = ground;
                if altitude < floor {
                    let up = self.domain.up(c.p);
                    c.p = c.p.add(up.mul(floor - altitude));
                    let contact_normal = if ground && self.domain.spherical() {
                        let east = self.domain.tangent(c.p, 0.);
                        let north = self.domain.tangent(c.p, std::f32::consts::FRAC_PI_2);
                        let height = |q: V3| {
                            let (a, b) = self.domain.coords(q);
                            self.domain.ground(a, b, core.config.seed) + p.relief_at(a, b)
                        };
                        let slope_e = (height(c.p.add(east.mul(0.03)))
                            - height(c.p.sub(east.mul(0.03))))
                            / 0.06;
                        let slope_n = (height(c.p.add(north.mul(0.03)))
                            - height(c.p.sub(north.mul(0.03))))
                            / 0.06;
                        up.sub(east.mul(slope_e))
                            .sub(north.mul(slope_n))
                            .normalized()
                    } else {
                        up
                    };
                    let normal_speed = c.v.dot(contact_normal);
                    if normal_speed < 0. {
                        c.v = c.v.sub(contact_normal.mul(normal_speed));
                    }
                    let friction =
                        (1. - step * (3. + c.activity(crate::physiology::REPAIR) * 3.)).max(0.);
                    let normal = contact_normal.mul(c.v.dot(contact_normal));
                    c.v = normal.add(c.v.sub(normal).mul(friction));
                }
            }
        }
        e.bonds.retain_mut(|b| {
            let (a, c) = (ids[&b.a], ids[&b.b]);
            let (left, right) = e.cells.split_at_mut(a.max(c));
            let (x, y) = if a < c {
                (&mut left[a], &mut right[0])
            } else {
                (&mut right[0], &mut left[c])
            };
            let strain = (x.p.sub(y.p).len() / b.rest - 1.).abs();
            x.state.strain = x.state.strain.max(strain.min(2.));
            y.state.strain = y.state.strain.max(strain.min(2.));
            let toughness = 0.4 + (x.scaffold() + y.scaffold()) * 0.2;
            let brittleness = 1. + b.stiffness * 0.4;
            let damage = DT * 0.03 * (strain - toughness / brittleness).max(0.);
            for cell in [&mut *x, &mut *y] {
                cell.state.injury = (cell.state.injury
                    + damage / (1. + cell.activity(crate::physiology::BARRIER) * 4.))
                    .min(1.);
            }
            let repair = (x.activity(crate::physiology::REPAIR)
                + y.activity(crate::physiology::REPAIR))
                * 0.5;
            let requested = (DT * 0.02 * repair * (1. - b.integrity))
                .min((x.energy.min(y.energy) * 100.).max(0.));
            let paid = self.spend(x, requested as f64 * 0.005, false)
                + self.spend(y, requested as f64 * 0.005, false);
            b.integrity = (b.integrity - damage + paid as f32 / 0.01).clamp(0., 1.);
            let remodel =
                (x.activity(crate::organics::REMODEL) + y.activity(crate::organics::REMODEL)) * 0.5;
            let distance = x.p.sub(y.p).len();
            let change = (distance - b.rest) * DT * 0.004 * remodel;
            let required = change.abs() as f64 * 0.1;
            let available = (x.energy.min(y.energy) as f64 * 2.).min(required);
            let fraction = if required > 0. {
                available / required
            } else {
                0.
            };
            self.spend(x, available * 0.5, false);
            self.spend(y, available * 0.5, false);
            b.rest = (b.rest + change * fraction as f32).clamp(0.1, 2.);
            let rate =
                0.004 * (1. + strain as f64 * 8.) / (0.1 + (x.activity(2) * y.activity(2)) as f64);
            let survive = b.integrity > 0.02
                && strain < 2.
                && (core.random() as f64) >= 1. - (-rate * DT as f64).exp();
            if !survive {
                self.dissociations += 1;
            }
            survive
        });
    }
    fn spring_k(b: &Bond) -> f64 {
        (1. + b.stiffness as f64 * 60.) * b.integrity as f64
    }
    pub(crate) fn divide(&mut self, e: &mut Evolution, core: &mut Core) {
        let n = e.cells.len();
        let variation = self.mutation_factor(core.time);
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
            if c.lifecycle.phase == crate::development::Phase::Gamete
                || c.material.as_ref().unwrap().biomass < 0.38 * (1. + c.activity(ENGULF)) as f64
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
            child.state.motor_work = 0.;
            child.state.flux = crate::observations::Flux::default();
            child.state.meals = 0;
            child.state.neural_input = 0.;
            child.state.light = 0.;
            child.state.light_contrast = 0.;
            child.lifecycle.root = e.cells[i].body_origin();
            child.lifecycle.depth = child.lifecycle.depth.saturating_add(1).min(1_000_000);
            let gamete = child.activity(crate::development::GAMETIC) > 0.25
                && child.activity(crate::development::FUSOGEN) > 0.1
                && core.random() < child.activity(crate::development::GAMETIC);
            if gamete {
                child.genome = child.gamete_chromosome(core);
                if child.lifecycle.homolog.is_some() {
                    self.development.meioses += 1;
                }
                child.lifecycle.homolog = None;
                child.lifecycle.phase = crate::development::Phase::Gamete;
                self.development.gametes += 1;
            } else {
                child.lifecycle.phase = crate::development::Phase::Somatic;
            }
            e.cells[i].lifecycle.phase = crate::development::Phase::Somatic;
            let fidelity = e.cells[i].state.germline;
            let requested = fidelity as f64 * 0.15;
            let paid = self.spend(&mut e.cells[i], requested, false);
            self.systems.fidelity_work += paid;
            let fidelity = if requested > 0. {
                fidelity * (paid / requested).min(1.) as f32
            } else {
                0.
            };
            let mut edits = child.genome.mutate_scaled(core, fidelity, variation);
            if edits.iter().sum::<u32>() > 0 || gamete {
                child.genome.id = e.next_genome;
                e.next_genome += 1;
            }
            if let Some(other) = &mut child.lifecycle.homolog {
                let extra = other.mutate_scaled(core, fidelity, variation);
                if extra.iter().sum::<u32>() > 0 {
                    if edits.iter().sum::<u32>() == 0 {
                        child.genome.id = e.next_genome;
                        e.next_genome += 1;
                    }
                    other.id = e.next_genome;
                    e.next_genome += 1;
                }
                for (a, b) in edits.iter_mut().zip(extra) {
                    *a += b;
                }
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
            // A three-dimensional division plane with no limb or adult template.
            let vertical = core.random() * 2. - 1.;
            let horizontal = (1. - vertical * vertical).sqrt();
            let direction = self.domain.tangent(child.p, angle);
            let up = self.domain.up(child.p);
            let random_direction = direction
                .mul(horizontal * dist)
                .add(up.mul(vertical * dist));
            let polarity = child.activity(crate::development::POLARITY);
            let turn = child
                .genome
                .genes
                .iter()
                .chain(child.lifecycle.homolog.iter().flat_map(|g| &g.genes))
                .filter(|g| g.action == crate::development::POLARITY)
                .map(|g| {
                    g.strength
                        * (g.bias * 0.15
                            + g.promoter[19] * (child.chemicals[0] - child.chemicals[1]) * 0.25)
                })
                .sum::<f32>();
            child.heading = (child.heading + turn * polarity).rem_euclid(std::f32::consts::TAU);
            let delta = random_direction
                .mul(1. - polarity)
                .add(
                    self.domain
                        .tangent(child.p, child.heading)
                        .mul(dist * polarity),
                )
                .normalized()
                .mul(dist);
            child.p = child.p.add(delta.mul(0.5));
            e.cells[i].p = e.cells[i].p.sub(delta.mul(0.5));
            if !self.domain.spherical() {
                for c in [&mut child, &mut e.cells[i]] {
                    c.p.x = c.p.x.clamp(-self.domain.wall(), self.domain.wall());
                    c.p.z = c.p.z.clamp(-self.domain.wall(), self.domain.wall());
                }
            }
            let attached = !gamete
                && degree.get(&child.parent).copied().unwrap_or(0) < 6
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
            let alive = c.energy > 0.0001 && m.biomass >= 0.02 && c.state.injury < 0.98;
            if !alive {
                let cause = if c.state.injury >= 0.98 {
                    2
                } else if c.energy <= 0.0001 {
                    0
                } else {
                    1
                };
                self.steering.losses[cause] += 1;
                let site = self.domain.locate(c.p);
                self.solutes[site][0] += m.fuel;
                let detritus = m.organic() - m.fuel - m.body.dissolved_organic();
                let fraction = if c.state.culled || self.systems.particles.len() >= 4096 {
                    0.
                } else {
                    0.2
                };
                // Remains need a decomposer; a culled body is returned as fuel.
                let pool = if c.state.culled { 0 } else { 5 };
                self.solutes[site][pool] += detritus * (1. - fraction);
                self.solutes[site][2] += m.mineral * (1. - fraction);
                if fraction > 0. {
                    self.systems
                        .particles
                        .push(crate::organics::MatterParticle {
                            p: c.p,
                            v: c.v,
                            solutes: [0., 0., m.mineral * fraction, 0., 0., detritus * fraction],
                        });
                }
                for k in 0..6 {
                    self.solutes[site][k] += m.body.solutes[k];
                }
                if !self.systems.gas.is_empty() {
                    for k in 0..2 {
                        self.systems.gas[site][k] += m.body.gas[k];
                    }
                }
                if let Some(p) = &mut self.planet {
                    p.water[site] += m.body.water;
                    p.biological_water -= m.body.water;
                }
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
        let mut planet = self
            .planet
            .take()
            .unwrap_or_else(|| crate::planet::Planet::new_in(self.domain));
        planet.tick(core, &mut self.solutes);
        if let Some(flow) = &mut self.fluid.surface {
            flow.step(
                self.domain,
                &mut planet,
                &mut self.solutes,
                core.config.seed,
            );
        }
        self.planet = Some(planet);
        self.fluid.step();
        self.transport(DT as f64);
        if self.ablation != "transport" {
            self.tissue_transport(e);
        }
        self.body_chemistry(e, core);
        if self.ablation != "transport" {
            self.internal_circulation(e);
        }
        self.particle_ecology(e, core);
        self.photoreception(e);
        self.physiology(e, core);
        if self.ablation != "signals" {
            self.neural_contacts(e);
            self.excitable_signals(e);
        } else {
            for c in &mut e.cells {
                c.state.neural_input = 0.;
                c.state.excitation = 0.;
                c.state.recovery = 0.;
            }
        }
        let pairs = self.contacts(e, core);
        self.mechanics(e, core, &pairs);
        self.fertilize(e, core, &pairs);
        self.divide(e, core);
        self.steer(e, core);
        self.recycle(e);
        if let Some(p) = &mut self.planet {
            p.biological_water = e
                .cells
                .iter()
                .map(|c| c.material.as_ref().unwrap().body.water)
                .sum();
        }
        self.prune_synapses(e);
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
        // The physical export reads this habitat directly.
        for i in 0..GRID * GRID {
            let x = (i % GRID) as f32 * 1.6 - 31.2;
            let z = (i / GRID) as f32 * 1.6 - 31.2;
            core.field[i] = self.solutes[self.domain.index(x, z)][0] as f32;
        }
        e.peak_body = e
            .peak_body
            .max(e.groups().iter().map(Vec::len).max().unwrap_or(0));
        core.births = e.divisions;
        core.deaths = e.deaths;
        core.mating = self.development.zygotes;
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
        let speed = (0..self.domain.sites())
            .map(|i| {
                let u = self.fluid.velocity(i);
                (u[0] * u[0] + u[1] * u[1]).sqrt()
            })
            .fold(0., f64::max);
        serde_json::json!({"domain":{"width":self.domain.size(),"grid":self.domain.grid,"rows":self.domain.rows(),"sites":self.domain.sites(),"spacing":self.domain.spacing(),"radius":if self.domain.spherical(){self.domain.radius()}else{0.},"shape":self.domain.shape,"area":(0..self.domain.sites()).map(|i|self.domain.area(i)).sum::<f64>(),"boundary":if self.domain.spherical(){"closed sphere"}else{"closed planar"}},"worldFlow":self.fluid.surface.as_ref().map(|f|(0..self.domain.sites()).step_by((self.domain.sites()/400).max(1)).map(|i|{let(x,z)=self.domain.point(i);let p=self.domain.position(x,z,0.12);let v=f.velocities[i];[p.x,p.y,p.z,v.x,v.y,v.z]}).collect::<Vec<_>>()),"assay":if self.passive {"passive"}else{"full"},"joints":self.joints.len(),"angularWork":self.joints.iter().map(|j|j.work).sum::<f64>(),"organic":organic,"mineral":mineral,"chemicalEnergy":energy,"photonEnergy":self.photon_energy,"heat":self.heat,
            "mechanicalInput":self.mechanical_input,"massError":errors[0],"mineralError":errors[1],"energyError":errors[2],
            "fluidSpeed":speed,"flow":(0..self.domain.sites()).step_by(4).map(|i|{let u=self.fluid.velocity(i);[i as f64,u[0],u[1]]}).collect::<Vec<_>>(),"boundEvents":self.bound_events,"dissociations":self.dissociations,
            "proteinMass":cells.iter().map(|c|c.material.as_ref().unwrap().proteins.iter().sum::<f64>()).sum::<f64>(),
            "boundMineral":cells.iter().map(|c|c.material.as_ref().unwrap().mineral).sum::<f64>()})
    }
}

impl Cell {
    pub fn radius(&self) -> f32 {
        self.material.as_ref().map_or(0.22, Inventory::radius)
    }
    pub fn activity(&self, k: usize) -> f32 {
        self.material.as_ref().map_or(self.expression[k], |m| {
            m.activity(k) * (1. - self.state.injury * 0.75)
        })
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
            promoter: [0.; crate::autonomous::INPUTS],
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
        h.solutes = vec![[0.; 6]; GRID * GRID];
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
            promoter: [0.; crate::autonomous::INPUTS],
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

    #[test]
    fn storage_repair_and_fission_use_existing_material() {
        let (mut core, mut e, mut h) = joint_fixture(false);
        e.bonds.clear();
        h.joints.clear();
        for c in &mut e.cells {
            c.energy = 8.;
            let m = c.material.as_mut().unwrap();
            m.fuel = 0.4;
            m.proteins[crate::physiology::LIPID] = 0.04;
            m.proteins[crate::physiology::REPAIR] = 0.04;
        }
        (h.initial_organic, h.initial_mineral, h.initial_energy) = h.totals(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        h.reserves_and_repair(&mut e.cells[0]);
        assert!(e.cells[0].material.as_ref().unwrap().lipid > 0.);
        e.cells[0].state.injury = 0.4;
        let biomass = e.cells[0].material.as_ref().unwrap().biomass;
        h.reserves_and_repair(&mut e.cells[0]);
        assert!(
            e.cells[0].state.injury < 0.4
                && e.cells[0].material.as_ref().unwrap().biomass > biomass
        );
        // Moving ATP out is an explicit sink in this controlled starvation test.
        h.spend(&mut e.cells[0], 6., false);
        let lipid = e.cells[0].material.as_ref().unwrap().lipid;
        h.reserves_and_repair(&mut e.cells[0]);
        assert!(e.cells[0].material.as_ref().unwrap().lipid < lipid);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
        e.cells[0].material.as_mut().unwrap().biomass = 0.4;
        e.cells[0].material.as_mut().unwrap().lipid = 0.08;
        e.cells[0].material.as_mut().unwrap().proteins[7] = 0.1;
        e.cells[0].energy = 8.;
        e.cells[0].cooldown = 0.;
        for c in &mut e.cells[1..] {
            c.cooldown = 100.;
        }
        (h.initial_organic, h.initial_mineral, h.initial_energy) = h.totals(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        for _ in 0..100 {
            h.divide(&mut e, &mut core);
            if e.cells.len() > 3 {
                break;
            }
        }
        assert_eq!(e.cells.len(), 4);
        assert_eq!(e.cells[0].material.as_ref().unwrap().lipid, 0.04);
        assert_eq!(e.cells[3].material.as_ref().unwrap().lipid, 0.04);
        assert_eq!(e.cells[0].state.excitation, e.cells[3].state.excitation);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
    }
    #[test]
    fn attached_carriers_share_fuel_and_atp_without_creating_energy() {
        let (_, mut e, mut h) = joint_fixture(false);
        for c in &mut e.cells {
            c.material.as_mut().unwrap().proteins[8] = 0.04;
        }
        e.cells[0].material.as_mut().unwrap().fuel = 0.25;
        e.cells[1].material.as_mut().unwrap().fuel = 0.;
        e.cells[0].energy = 8.;
        e.cells[1].energy = 1.;
        (h.initial_organic, h.initial_mineral, h.initial_energy) = h.totals(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        h.tissue_transport(&mut e);
        assert!(e.cells[1].energy > 1. && e.cells[1].material.as_ref().unwrap().fuel > 0.);
        assert!(h.physiology.energy_transported > 0. && h.physiology.fuel_transported > 0.);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
        e.bonds.clear();
        let before = serde_json::to_string(&e.cells).unwrap();
        h.tissue_transport(&mut e);
        assert_eq!(before, serde_json::to_string(&e.cells).unwrap());
    }
    #[test]
    fn excitable_contact_network_is_material_paid_and_drives_real_motion() {
        let (mut core, mut e, mut h) = joint_fixture(false);
        for c in &mut e.cells {
            c.chemicals = [0.; 2];
            c.material.as_mut().unwrap().proteins[12] = 0.04;
            c.material.as_mut().unwrap().proteins[8] = 0.04;
        }
        e.cells[0].state.excitation = 0.9;
        (h.initial_organic, h.initial_mineral, h.initial_energy) = h.totals(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        let mut detached = e.clone();
        detached.bonds.clear();
        for _ in 0..12 {
            h.excitable_signals(&mut e);
        }
        assert!(e.cells[1].state.excitation > 0.);
        let mut detached_h = h.clone();
        detached_h.excitable_signals(&mut detached);
        assert_eq!(detached.cells[1].state.excitation, 0.);
        assert!(h.physiology.signal_work > 0.);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
        let mut passive = e.clone();
        let mut passive_h = h.clone();
        passive_h.passive = true;
        let before: Vec<_> = e.cells.iter().map(|c| c.p).collect();
        h.mechanics(&mut e, &mut core, &[]);
        passive_h.mechanics(&mut passive, &mut core, &[]);
        assert!(e.motor_work > 0. && passive.motor_work == 0.);
        assert!(e
            .cells
            .iter()
            .zip(&passive.cells)
            .any(|(a, b)| a.p.sub(b.p).len() > 1e-6));
        assert!(e
            .cells
            .iter()
            .zip(before)
            .any(|(a, p)| a.p.sub(p).len() > 1e-6));
        for c in &mut detached.cells {
            c.state.excitation = 0.;
            c.material.as_mut().unwrap().proteins[12] = 0.;
            c.state.food_change = 1.;
        }
        detached_h.excitable_signals(&mut detached);
        assert!(detached.cells.iter().all(|c| c.state.excitation == 0.));
        detached.cells[0].state.excitation = 1.;
        assert_eq!(
            detached.cells[0].motor_signals(),
            [0.; 2],
            "Lost channel material cannot leave a free motor-control latch"
        );
    }
    #[test]
    fn digestion_wounds_defense_limits_it_and_excretion_feeds_decomposers() {
        let (mut core, mut e, mut h) = joint_fixture(false);
        e.bonds.clear();
        for (i, c) in e.cells.iter_mut().enumerate() {
            c.p = V3::new(i as f32 * 0.3, 5., 0.);
            c.material.as_mut().unwrap().fuel = 0.;
            c.material.as_mut().unwrap().proteins[9] = 0.;
        }
        e.cells[0].material.as_mut().unwrap().proteins[9] = 0.04;
        let mut defended = e.clone();
        defended.cells[1].material.as_mut().unwrap().proteins[14] = 0.08;
        (h.initial_organic, h.initial_mineral, h.initial_energy) = h.totals(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        let mut shield_h = h.clone();
        h.contacts(&mut e, &mut core);
        shield_h.contacts(&mut defended, &mut core);
        assert!(
            defended.cells[1].material.as_ref().unwrap().biomass
                > e.cells[1].material.as_ref().unwrap().biomass
        );
        assert!(e.cells[1].state.injury > 0. && h.physiology.excreted > 0.);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
        let site = field_index(e.cells[2].p.x, e.cells[2].p.z);
        h.solutes[site][0] = 0.;
        h.solutes[site][1] = 0.;
        h.solutes[site][5] = 0.2;
        let mut control = e.clone();
        let mut control_h = h.clone();
        e.cells[2].material.as_mut().unwrap().proteins[15] = 0.04;
        (h.initial_organic, h.initial_mineral, h.initial_energy) = h.totals(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        h.physiology(&mut e, &core);
        control_h.physiology(&mut control, &core);
        assert!(h.physiology.decomposed > 0. && control_h.physiology.decomposed == 0.);
        assert!(h.solutes[site][5] < control_h.solutes[site][5]);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
    }
    #[test]
    fn regions_are_observations_not_supplied_organs_and_new_state_resumes() {
        let (mut core, mut e, mut h) = joint_fixture(false);
        for c in &mut e.cells {
            let m = c.material.as_mut().unwrap();
            m.lipid = 0.05;
            m.proteins[12] = 0.04;
            m.proteins[8] = 0.04;
        }
        e.cells[0].state.excitation = 0.9;
        let view = h.physiology_view(&e);
        assert_eq!(view["mixedBodies"], 1);
        assert!(view["regions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["regions"] == 1));
        let mut detached = e.clone();
        detached.bonds.clear();
        assert_eq!(h.physiology_view(&detached)["mixedBodies"], 0);
        (h.initial_organic, h.initial_mineral, h.initial_energy) = h.totals(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        h.planet.as_mut().unwrap().storm = 4.;
        e.physical = Some(h);
        core.organisms.clear();
        core.autonomous = Some(e);
        core.step(4);
        let saved = serde_json::to_string(&core).unwrap();
        let mut copy = Core::load(&saved).unwrap();
        core.step(6);
        copy.step(6);
        assert_eq!(
            serde_json::to_string(&core).unwrap(),
            serde_json::to_string(&copy).unwrap()
        );
        let mut bad = serde_json::to_value(&core).unwrap();
        bad["autonomous"]["cells"][0]["state"]["excitation"] = 2.into();
        assert!(Core::load(&bad.to_string()).is_err());
    }
    #[test]
    fn old_vocabularies_migrate_without_inventing_specialized_genes_or_matter() {
        let mut core = Core::new(Config::default()).unwrap();
        core.start_physical();
        let mut value = serde_json::to_value(&core).unwrap();
        let e = &mut value["autonomous"];
        for c in e["cells"].as_array_mut().unwrap() {
            c.as_object_mut().unwrap().remove("state");
            c["material"].as_object_mut().unwrap().remove("lipid");
            c["expression"].as_array_mut().unwrap().truncate(10);
            c["material"]["proteins"]
                .as_array_mut()
                .unwrap()
                .truncate(10);
            for g in c["genome"]["genes"].as_array_mut().unwrap() {
                g["promoter"].as_array_mut().unwrap().truncate(8);
            }
        }
        e["physical"].as_object_mut().unwrap().remove("planet");
        e["physical"].as_object_mut().unwrap().remove("physiology");
        for s in e["physical"]["solutes"].as_array_mut().unwrap() {
            s.as_array_mut().unwrap().truncate(5);
        }
        let mut migrated = Core::load(&value.to_string()).unwrap();
        assert!(migrated
            .autonomous
            .as_ref()
            .unwrap()
            .cells
            .iter()
            .all(|c| c.genome.genes.len() == 4 && c.material.as_ref().unwrap().lipid == 0.));
        migrated.step(4);
        assert!(migrated.autonomous.as_ref().unwrap().valid());
        value["autonomous"]["cells"][0]["expression"]
            .as_array_mut()
            .unwrap()
            .truncate(9);
        assert!(Core::load(&value.to_string()).is_err());
    }
    #[test]
    fn injury_impairs_material_functions_and_terminal_damage_recycles_real_stores() {
        let (_, mut e, mut h) = joint_fixture(false);
        let healthy = e.cells[0].activity(4);
        e.cells[0].state.injury = 0.7;
        assert!(e.cells[0].activity(4) < healthy);
        e.cells[0].material.as_mut().unwrap().lipid = 0.05;
        e.cells[0].state.injury = 1.;
        let before = h.totals(&e.cells);
        let site = field_index(e.cells[0].p.x, e.cells[0].p.z);
        h.recycle(&mut e);
        let after = h.totals(&e.cells);
        assert_eq!(e.cells.len(), 2);
        assert!(h.solutes[site][5] > 0.);
        assert!((before.0 - after.0).abs() < 1e-9 && (before.1 - after.1).abs() < 1e-9);
        assert!((before.2 - after.2 - h.heat).abs() < 1e-8);
    }
}

#[cfg(test)]
mod curved_tests {
    use super::*;
    #[test]
    fn sphere_gravity_pulls_from_all_directions_and_surface_supports_cells() {
        let mut core = Core::new(Config {
            world_size: 128,
            world_shape: crate::domain::Shape::Sphere,
            initial_cells: 3,
            ..Config::default()
        })
        .unwrap();
        core.start_physical();
        let mut e = core.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        let d = h.domain;
        h.planet.as_mut().unwrap().water.fill(0.);
        for (c, n) in e.cells.iter_mut().zip([
            V3::new(1., 0., 0.),
            V3::new(0., 1., 0.),
            V3::new(0., -1., 0.),
        ]) {
            c.p = n.mul(d.radius() + 10.);
            c.v = V3::default();
        }
        let radii: Vec<_> = e.cells.iter().map(|c| c.p.len()).collect();
        h.mechanics(&mut e, &mut core, &[]);
        for (c, before) in e.cells.iter().zip(radii) {
            assert!(c.p.len() < before);
            assert!(c.v.dot(d.up(c.p)) < 0.);
        }
        for c in &mut e.cells {
            let (x, z) = d.coords(c.p);
            let floor = h
                .planet
                .as_ref()
                .unwrap()
                .floor(x, z, c.radius(), core.config.seed);
            c.p = d.position(x, z, floor - 0.2);
            c.v = V3::default();
        }
        h.mechanics(&mut e, &mut core, &[]);
        for c in &e.cells {
            let (x, z) = d.coords(c.p);
            let floor = h
                .planet
                .as_ref()
                .unwrap()
                .floor(x, z, c.radius(), core.config.seed);
            assert!(c.p.len() - d.radius() >= floor - 0.001);
        }
    }
    #[test]
    fn globe_and_large_planar_populations_conserve_and_resume() {
        for shape in [crate::domain::Shape::Planar, crate::domain::Shape::Sphere] {
            for size in [64, 128, 256] {
                let mut core = Core::new(Config {
                    world_size: size,
                    world_shape: shape,
                    founder_spread: "global".into(),
                    initial_cells: 16,
                    ..Config::default()
                })
                .unwrap();
                core.start_physical();
                let radius = core.autonomous.as_ref().unwrap().cells[0].radius();
                assert!((radius - 0.22).abs() < 1e-6);
                core.step(12);
                let e = core.autonomous.as_ref().unwrap();
                assert!(e.valid());
                let h = e.physical.as_ref().unwrap();
                assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-7));
                assert!(h.planet.as_ref().unwrap().water_error().abs() < 1e-7);
                let save = serde_json::to_string(&core).unwrap();
                let mut resumed = Core::load(&save).unwrap();
                core.step(4);
                resumed.step(4);
                assert_eq!(
                    serde_json::to_string(&core).unwrap(),
                    serde_json::to_string(&resumed).unwrap()
                );
            }
        }
    }
    #[test]
    fn mismatched_geometry_in_saves_is_rejected() {
        let mut core = Core::new(Config {
            world_size: 128,
            world_shape: crate::domain::Shape::Sphere,
            ..Config::default()
        })
        .unwrap();
        core.start_physical();
        let mut s = serde_json::to_value(&core).unwrap();
        s["config"]["worldSize"] = 64.into();
        assert!(Core::load(&s.to_string()).is_err());
        let mut s = serde_json::to_value(&core).unwrap();
        s["autonomous"]["physical"]["planet"]["domain"]["grid"] = 40.into();
        assert!(Core::load(&s.to_string()).is_err());
    }
}
