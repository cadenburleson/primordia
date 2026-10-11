//! Conserved tissue materials and finite-volume internal transport.
//! These local processes have no named-organ templates or fitness objectives.
use crate::{
    autonomous::{Cell, Evolution},
    physical::Habitat,
    *,
};

pub const MATRIX: usize = 21;
pub const PUMP: usize = 22;
pub const GAS: usize = 23;
pub const TOUCH: usize = 24;
pub const RECOGNITION: usize = 25;
pub const FIDELITY: usize = 26;
pub const MATRIX_DIGEST: usize = 27;
pub const FIBER: usize = 28;
pub const REMODEL: usize = 29;
pub const ASSEMBLY: usize = 30;
pub const LYSIS: usize = 31;
const Q: f64 = 3.;

/// Abstract equal-mass monomer classes, not atomistic molecules. Sequences are
/// assembled from food and their contact properties follow composition/motifs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Polymer {
    pub sequence: Vec<u8>,
    pub amount: f64,
}
impl Polymer {
    fn properties(&self) -> [f64; 3] {
        let n = self.sequence.len() as f64;
        let sticky = self.sequence.iter().filter(|&&x| x == 1).count() as f64 / n;
        let charged = self.sequence.iter().filter(|&&x| x == 2).count() as f64 / n;
        let rigid = self
            .sequence
            .windows(2)
            .filter(|x| x[0] == 3 && x[1] == 3)
            .count() as f64
            / n;
        [sticky, charged, rigid]
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BodyStores {
    pub water: f64,
    #[serde(deserialize_with = "crate::physical::padded")]
    pub solutes: [f64; crate::physical::SOLUTES],
    pub gas: [f64; 2],
    pub matrix: f64,
    pub fibers: f64,
    pub polymers: Vec<Polymer>,
}
impl BodyStores {
    pub fn dissolved_organic(&self) -> f64 {
        self.solutes[0] + self.solutes[1] + self.solutes[3] + self.solutes[4] + self.solutes[5]
    }
    pub fn organic(&self) -> f64 {
        self.dissolved_organic()
            + self.matrix
            + self.fibers
            + self.polymers.iter().map(|p| p.amount).sum::<f64>()
    }
    pub fn properties(&self, biomass: f64) -> [f32; 3] {
        let mut values = [0.; 3];
        for p in &self.polymers {
            for (v, x) in values.iter_mut().zip(p.properties()) {
                *v += x * p.amount;
            }
        }
        values.map(|v| (v / (0.02 * biomass + v).max(1e-12)) as f32)
    }
    pub fn scale(&mut self, fraction: f64) {
        self.water *= fraction;
        self.matrix *= fraction;
        self.fibers *= fraction;
        for x in self.solutes.iter_mut().chain(&mut self.gas) {
            *x *= fraction;
        }
        for p in &mut self.polymers {
            p.amount *= fraction;
        }
    }
    /// A bounded species cache hydrolyzes excess variants back into fuel.
    pub fn merge(&mut self, other: &Self) -> f64 {
        self.water += other.water;
        self.matrix += other.matrix;
        self.fibers += other.fibers;
        for (a, b) in self.solutes.iter_mut().zip(other.solutes) {
            *a += b;
        }
        for (a, b) in self.gas.iter_mut().zip(other.gas) {
            *a += b;
        }
        let mut hydrolyzed = 0.;
        for p in &other.polymers {
            if let Some(old) = self.polymers.iter_mut().find(|x| x.sequence == p.sequence) {
                old.amount += p.amount;
            } else if self.polymers.len() < 12 {
                self.polymers.push(p.clone());
            } else {
                hydrolyzed += p.amount;
            }
        }
        hydrolyzed
    }
    pub fn valid(&self) -> bool {
        [self.water, self.matrix, self.fibers]
            .iter()
            .chain(&self.solutes)
            .chain(&self.gas)
            .all(|x| x.is_finite() && *x >= 0. && *x < 1e6)
            && self.polymers.len() <= 12
            && self.polymers.iter().all(|p| {
                p.amount.is_finite()
                    && p.amount >= 0.
                    && p.amount < 1e6
                    && (4..=24).contains(&p.sequence.len())
                    && p.sequence.iter().all(|&x| x < 4)
            })
            && self
                .polymers
                .iter()
                .map(|p| &p.sequence)
                .collect::<BTreeSet<_>>()
                .len()
                == self.polymers.len()
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemLedger {
    pub particles: Vec<MatterParticle>,
    /// Available/spent redox carrier, a closed effective gas budget.
    pub gas: Vec<[f64; 2]>,
    pub initial_gas: f64,
    pub water_flow: f64,
    pub solute_flow: f64,
    pub membrane_exchange: f64,
    pub absorbed: f64,
    pub filtered: f64,
    pub gas_exchange: f64,
    pub respired: f64,
    pub pump_work: f64,
    pub matrix_built: f64,
    pub fiber_built: f64,
    pub mineral_remodeled: f64,
    pub polymers_built: f64,
    pub fidelity_work: f64,
    pub matrix_meals: f64,
}
impl SystemLedger {
    pub fn gas_error(&self, cells: &[Cell]) -> f64 {
        self.gas.iter().flatten().sum::<f64>()
            + cells
                .iter()
                .filter_map(|c| c.material.as_ref())
                .map(|m| m.body.gas.iter().sum::<f64>())
                .sum::<f64>()
            - self.initial_gas
    }
    pub fn valid(&self, domain: domain::Domain, cells: &[Cell]) -> bool {
        (self.gas.is_empty() || self.gas.len() == domain.sites())
            && self.particles.len() <= 4096
            && self.particles.iter().all(|p| {
                p.solutes
                    .iter()
                    .all(|v| v.is_finite() && *v >= 0. && *v < 1e6)
                    && [p.p.x, p.p.y, p.p.z, p.v.x, p.v.y, p.v.z]
                        .iter()
                        .all(|v| v.is_finite())
                    && p.v.len() < 100.
                    && p.p.len() < domain.size() * 2. + 100.
            })
            && self
                .gas
                .iter()
                .flatten()
                .all(|x| x.is_finite() && *x >= 0. && *x < 1e9)
            && [
                self.initial_gas,
                self.water_flow,
                self.solute_flow,
                self.membrane_exchange,
                self.absorbed,
                self.filtered,
                self.gas_exchange,
                self.respired,
                self.pump_work,
                self.matrix_built,
                self.fiber_built,
                self.mineral_remodeled,
                self.polymers_built,
                self.fidelity_work,
                self.matrix_meals,
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.)
            && self.gas_error(cells).abs() < 1e-7 * (1. + self.initial_gas)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatterParticle {
    pub p: body::V3,
    pub v: body::V3,
    #[serde(deserialize_with = "crate::physical::padded")]
    pub solutes: [f64; crate::physical::SOLUTES],
}
impl MatterParticle {
    fn mass(&self) -> f64 {
        self.solutes[..6].iter().sum()
    }
    fn radius(&self) -> f32 {
        (0.04 + self.mass().cbrt() * 0.12) as f32
    }
}
impl Cell {
    pub(crate) fn membrane_identity(&self) -> f32 {
        let mut total = 0.;
        let mut weight = 0.;
        for g in self
            .genome
            .genes
            .iter()
            .chain(self.lifecycle.homolog.iter().flat_map(|g| &g.genes))
            .filter(|g| g.action == RECOGNITION)
        {
            total += (g.bias + g.promoter[4] - g.promoter[5]) * g.strength;
            weight += g.strength;
        }
        if weight > 0. {
            (total / weight).clamp(-2., 2.)
        } else {
            0.
        }
    }
    pub(crate) fn recognition(&self, other: &Cell) -> f32 {
        let gate = self.activity(RECOGNITION).min(other.activity(RECOGNITION));
        1. / (1. + gate * (self.membrane_identity() - other.membrane_identity()).abs() * 3.)
    }
    pub(crate) fn scaffold(&self) -> f32 {
        self.material.as_ref().map_or(0., |m| {
            ((m.body.matrix + m.body.fibers) / (0.04 * m.biomass + m.body.matrix + m.body.fibers))
                as f32
        })
    }
}

fn transfer(x: &mut f64, y: &mut f64, requested: f64) -> f64 {
    let moved = requested.clamp(-*y * 0.2, *x * 0.2);
    *x -= moved;
    *y += moved;
    moved
}
/// Oxygen a planet holds over each field site, free or bound. It has to be
/// large beside what life makes and burns in a cycle, or the air could not
/// carry a crowded sea through a single night.
const AIR: f64 = 0.6;
/// Share of the gap between a site's air and the planet's that closes each tick.
const AIR_MIXING: f64 = 0.2;
impl Habitat {
    /// Set how much of the planet's oxygen is free, from none (0) to all of
    /// it (1), everywhere at once. A young planet has none: it is all bound up.
    pub(crate) fn oxygenate(&mut self, level: f64) {
        if self.systems.gas.is_empty() {
            self.systems.gas = vec![[0.; 2]; self.domain.sites()];
            for (i, g) in self.systems.gas.iter_mut().enumerate() {
                g[1] = AIR * self.domain.area(i) / 2.56;
            }
            self.systems.initial_gas = self.systems.gas.iter().flatten().sum::<f64>();
        }
        for g in &mut self.systems.gas {
            let total = g[0] + g[1];
            *g = [total * level, total * (1. - level)];
        }
    }
    /// Share of the planet's oxygen that is free.
    pub(crate) fn oxygen(&self) -> f64 {
        let free: f64 = self.systems.gas.iter().map(|g| g[0]).sum();
        let all: f64 = self.systems.gas.iter().flatten().sum();
        if all > 0. { free / all } else { 0. }
    }
    fn gas_transport(&mut self, e: &Evolution) {
        if self.systems.gas.is_empty() {
            self.oxygenate(0.);
        }
        // Worlds saved when the air was a tenth this size keep their mix.
        let full: f64 = (0..self.domain.sites()).map(|i| AIR * self.domain.area(i) / 2.56).sum();
        if self.systems.initial_gas < full * 0.5 {
            for g in self.systems.gas.iter_mut().flatten() {
                *g *= 10.;
            }
            self.systems.initial_gas += self.systems.gas.iter().flatten().sum::<f64>() * 0.9;
        }
        for (a, b) in self.domain.edges() {
            let (lo, hi) = (a.min(b), a.max(b));
            let (left, right) = self.systems.gas.split_at_mut(hi);
            let (x, y) = if a == lo {
                (&mut left[lo], &mut right[0])
            } else {
                (&mut right[0], &mut left[lo])
            };
            for k in 0..2 {
                let difference = x[k] / self.domain.area(a) - y[k] / self.domain.area(b);
                transfer(&mut x[k], &mut y[k], DT as f64 * 0.05 * difference);
            }
        }
        // The air above mixes within days: each site's gases relax toward the
        // planet's average, so oxygen made in one sea reaches every other.
        let area: Vec<f64> = (0..self.domain.sites()).map(|i| self.domain.area(i)).collect();
        let surface: f64 = area.iter().sum();
        for k in 0..2 {
            let mean = self.systems.gas.iter().map(|g| g[k]).sum::<f64>() / surface;
            for (g, a) in self.systems.gas.iter_mut().zip(&area) {
                g[k] += (mean * a - g[k]) * AIR_MIXING;
            }
        }
        // Atmospheric transport is reduced diffusion; body transport is separate.
        let _ = e;
    }
    pub(crate) fn body_chemistry(&mut self, e: &mut Evolution, core: &mut Core) {
        self.gas_transport(e);
        for c in &mut e.cells {
            let site = self.domain.locate(c.p);
            let exposure = c.state.exposure.max(0.05) as f64;
            let gates: Vec<_> = (0..autonomous::ACTIONS)
                .map(|k| c.activity(k) as f64)
                .collect();
            let dt = DT as f64;
            let seal = gates[physiology::BARRIER].max(gates[MATRIX] * 0.5);
            let capacity = c.material.as_ref().unwrap().biomass * (0.15 + 0.65 * seal);
            let current = c.material.as_ref().unwrap().body.water;
            let requested = if seal > 0.05 || gates[PUMP] > 0.05 {
                (capacity - current).max(0.) * dt * (0.015 + gates[PUMP] * 0.2)
            } else {
                0.
            };
            let available = self.planet.as_ref().map_or(0., |p| p.water[site]);
            let taken = requested.min(available).min(c.energy as f64 / 0.35);
            let paid = self.spend(c, taken * 0.35, false);
            self.systems.pump_work += paid;
            let stature = c.stature();
            let m = c.material.as_mut().unwrap();
            m.body.water += taken;
            if let Some(p) = &mut self.planet {
                p.water[site] -= taken;
                p.biological_water += taken;
            }
            let leak = m.body.water
                * (1. - (-dt * (0.001 + c.state.injury as f64 * 0.15 + (1. - seal) * 0.006)).exp());
            let fraction = leak / m.body.water.max(1e-12);
            m.body.water -= leak;
            for k in 0..crate::physical::SOLUTES {
                let lost = m.body.solutes[k] * fraction;
                m.body.solutes[k] -= lost;
                self.solutes[site][k] += lost;
            }
            if let Some(p) = &mut self.planet {
                p.water[site] += leak;
                p.biological_water -= leak;
            }
            let volume = m.body.water;
            if volume > 1e-9 {
                let exterior_volume = self
                    .planet
                    .as_ref()
                    .map_or(1., |p| p.water[site])
                    .max(self.domain.area(site) * 0.02);
                for k in 0..crate::physical::SOLUTES {
                    let permeability = (if k == 5 {
                        gates[9].max(gates[MATRIX_DIGEST]) * 0.001
                    } else {
                        0.003 * gates[8] + 0.001 * gates[1]
                    }) * (1. + m.body.properties(m.biomass)[1] as f64 * 0.3);
                    let gradient =
                        self.solutes[site][k] / exterior_volume - m.body.solutes[k] / volume;
                    let moved = transfer(
                        &mut self.solutes[site][k],
                        &mut m.body.solutes[k],
                        dt * permeability * exposure * volume * gradient,
                    );
                    self.systems.membrane_exchange += moved.abs();
                }
                let uptake = (dt * 0.12 * gates[1])
                    .min(m.body.solutes[0])
                    .min((m.biomass * 1.5 - m.fuel).max(0.));
                m.body.solutes[0] -= uptake;
                m.fuel += uptake;
                self.systems.absorbed += uptake;
                c.state.flux.absorbed += uptake;
                let digestion = (dt * 0.03 * gates[9])
                    .min(m.body.solutes[5])
                    .min((m.biomass * 1.5 - m.fuel).max(0.) / 0.85)
                    .min(c.energy as f64 / 0.2);
                let unbound = if m.body.solutes[5] > 0. {
                    (m.body.solutes[crate::physical::REMAINS_N] * digestion * 0.85 / m.body.solutes[5])
                        .min(m.body.solutes[crate::physical::REMAINS_N])
                } else {
                    0.
                };
                m.body.solutes[crate::physical::REMAINS_N] -= unbound;
                m.nitrogen += unbound;
                m.body.solutes[5] -= digestion;
                m.fuel += digestion * 0.85;
                c.state.flux.detritus += digestion * 0.85;
                m.body.solutes[5] += digestion * 0.15;
                self.physiology.digested += digestion;
                self.physiology.excreted += digestion * 0.15;
                let before = c.energy as f64;
                c.energy = (before - digestion * 0.2) as f32;
                self.heat += before - c.energy as f64;
                let filtered = (dt * 0.04 * gates[8] * exposure)
                    .min(m.body.solutes[1])
                    .min(c.energy as f64 / 0.15);
                m.body.solutes[1] -= filtered;
                self.solutes[site][1] += filtered;
                self.systems.filtered += filtered;
                c.state.flux.filtered += filtered;
                let before = c.energy as f64;
                c.energy = (before - filtered * 0.15) as f32;
                self.heat += before - c.energy as f64;
            }
            for k in 0..2 {
                let concentration = self.systems.gas[site][k] / (self.domain.area(site) / 2.56);
                let interior = m.body.gas[k] / m.biomass.max(0.02);
                let moved = transfer(
                    &mut self.systems.gas[site][k],
                    &mut m.body.gas[k],
                    dt * 0.015 * gates[GAS] * exposure * (concentration - interior),
                );
                self.systems.gas_exchange += moved.abs();
            }
            let respiration = (dt * 0.08 * gates[GAS])
                .min(m.fuel)
                .min(m.body.gas[0] * 2.)
                .min(((9. * stature - c.energy as f64) / Q).max(0.));
            m.fuel -= respiration;
            m.body.gas[0] -= respiration * 0.5;
            m.body.gas[1] += respiration * 0.5;
            if volume > 1e-9 {
                m.body.solutes[1] += respiration;
            } else {
                self.solutes[site][1] += respiration;
            }
            let before = c.energy as f64;
            let next = before + respiration * Q;
            c.energy = next as f32;
            if c.energy as f64 > next {
                c.energy = f32::from_bits(c.energy.to_bits().saturating_sub(1));
            }
            self.heat += respiration * Q - (c.energy as f64 - before);
            self.systems.respired += respiration;
            c.state.flux.respiration += respiration;
            c.state.flux.waste += respiration;
            e.food_energy += respiration * Q;
            for (gate, rate, cost, fiber) in [
                (gates[MATRIX], 0.006, 0.5, false),
                (gates[FIBER], 0.004, 0.7, true),
            ] {
                let amount = (dt * rate * gate).min(m.fuel).min(c.energy as f64 / cost);
                m.fuel -= amount;
                if fiber {
                    m.body.fibers += amount;
                    self.systems.fiber_built += amount;
                    c.state.flux.fibers += amount;
                } else {
                    m.body.matrix += amount;
                    self.systems.matrix_built += amount;
                    c.state.flux.matrix += amount;
                }
                let before = c.energy as f64;
                c.energy = (before - amount * cost) as f32;
                self.heat += before - c.energy as f64;
            }
            let turnover = m.body.matrix * (1. - (-dt * 0.006).exp());
            m.body.matrix -= turnover;
            m.fuel += turnover;
            let damaged =
                m.body.fibers * (1. - (-dt * (0.006 + c.state.injury as f64 * 0.03)).exp());
            m.body.fibers -= damaged;
            m.fuel += damaged;
            let released = (dt * 0.008 * gates[REMODEL] * c.state.injury as f64).min(m.mineral);
            m.mineral -= released;
            self.solutes[site][2] += released;
            let bound = (dt * 0.006 * gates[REMODEL] * c.state.strain as f64)
                .min(m.body.solutes[2])
                .min(c.energy as f64 / 0.3);
            m.body.solutes[2] -= bound;
            m.mineral += bound;
            self.systems.mineral_remodeled += bound + released;
            let before = c.energy as f64;
            c.energy = (before - bound * 0.3) as f32;
            self.heat += before - c.energy as f64;
            let assembled = (dt * 0.004 * gates[ASSEMBLY])
                .min(m.fuel)
                .min(c.energy as f64 / 0.8);
            if assembled > 0. {
                let gene = c
                    .genome
                    .genes
                    .iter()
                    .chain(c.lifecycle.homolog.iter().flat_map(|g| &g.genes))
                    .find(|g| g.action == ASSEMBLY);
                if let Some(gene) = gene {
                    let n = (4. + (gene.bias + 2.) * 4.).clamp(4., 20.) as usize;
                    let mut sequence: Vec<u8> = (0..n)
                        .map(|i| ((gene.promoter[i] + 2.) * 0.99).clamp(0., 3.) as u8)
                        .collect();
                    if !m.body.polymers.is_empty() && core.random() < 0.5 {
                        sequence = m.body.polymers
                            [(core.random() * m.body.polymers.len() as f32) as usize]
                            .sequence
                            .clone();
                    }
                    if core.random() < 0.08 {
                        let i = (core.random() * sequence.len() as f32) as usize;
                        sequence[i] = (core.random() * 4.) as u8;
                    }
                    if let Some(p) = m.body.polymers.iter_mut().find(|p| p.sequence == sequence) {
                        p.amount += assembled;
                    } else {
                        if m.body.polymers.len() >= 12 {
                            let index = m
                                .body
                                .polymers
                                .iter()
                                .enumerate()
                                .min_by(|a, b| a.1.amount.total_cmp(&b.1.amount))
                                .unwrap()
                                .0;
                            m.fuel += m.body.polymers.remove(index).amount;
                        }
                        m.body.polymers.push(Polymer {
                            sequence,
                            amount: assembled,
                        });
                    }
                    m.fuel -= assembled;
                    self.systems.polymers_built += assembled;
                    let before = c.energy as f64;
                    c.energy = (before - assembled * 0.8) as f32;
                    self.heat += before - c.energy as f64;
                }
            }
            for p in &mut m.body.polymers {
                let decay = p.amount * (1. - (-dt * 0.01).exp());
                p.amount -= decay;
                m.fuel += decay;
            }
            let lysed = (dt * 0.025 * gates[LYSIS]).min((m.biomass - 0.005).max(0.));
            m.biomass -= lysed;
            self.solutes[site][5] += lysed;
            self.solutes[site][crate::physical::REMAINS_N] += lysed * crate::physical::N_RATIO;
            self.solutes[site][2] += lysed * crate::physical::P_RATIO;
            let osmotic = m.body.solutes.iter().sum::<f64>() / m.body.water.max(0.002);
            c.state.pressure = ((m.body.water / capacity.max(0.002) - 0.7).max(0.) * 0.8
                + osmotic / (1. + osmotic) * 0.2)
                .min(4.) as f32;
            c.state.pressure_gradient = 0.;
            c.state.germline = (gates[FIDELITY]
                * (0.3 + 0.7 * (1. - exposure))
                * (c.energy as f64 / 0.2).min(1.)) as f32;
        }
    }

    pub(crate) fn internal_circulation(&mut self, e: &mut Evolution) {
        let ids = e.index();
        for c in &mut e.cells {
            let motor = if self.passive {
                0.
            } else {
                c.activity(4) * c.motor_signals()[0] * c.activity(PUMP)
            };
            let required = DT as f64 * 0.004 * motor as f64;
            let paid = self.spend(c, required, false);
            self.systems.pump_work += paid;
            if required > 0. {
                c.state.pressure =
                    (c.state.pressure + motor * (paid / required).min(1.) as f32 * 0.8).min(4.);
            }
        }
        for b in &e.bonds {
            if b.integrity <= 0. {
                continue;
            }
            let (a, z) = (ids[&b.a], ids[&b.b]);
            let (left, right) = e.cells.split_at_mut(a.max(z));
            let (x, y) = if a < z {
                (&mut left[a], &mut right[0])
            } else {
                (&mut right[0], &mut left[z])
            };
            let gate = x.activity(8).min(y.activity(8)) as f64 * b.integrity as f64;
            let difference = (x.state.pressure - y.state.pressure) as f64;
            let radius = 0.025 + gate * 0.075;
            let direction = y.p.sub(x.p).normalized();
            let orientation = direction.dot(self.domain.tangent(x.p, x.heading));
            let reverse = if difference >= 0. {
                (-orientation).max(0.)
            } else {
                orientation.max(0.)
            };
            let valve = 1.
                - reverse as f64
                    * x.activity(physiology::BARRIER) as f64
                    * x.activity(development::POLARITY) as f64
                    * 0.95;
            let flow = DT as f64 * std::f64::consts::PI * radius.powi(4)
                / (8. * 0.005 * x.p.sub(y.p).len().max(0.1) as f64)
                * difference
                * valve;
            let (xm, ym) = (x.material.as_mut().unwrap(), y.material.as_mut().unwrap());
            let (donor, receiver) = if flow >= 0. {
                (&mut xm.body, &mut ym.body)
            } else {
                (&mut ym.body, &mut xm.body)
            };
            let moved = (flow.abs() * gate).min(donor.water * 0.1);
            let fraction = moved / donor.water.max(1e-12);
            donor.water -= moved;
            receiver.water += moved;
            self.systems.water_flow += moved;
            x.state.flux.water_transport += moved * 0.5;
            y.state.flux.water_transport += moved * 0.5;
            for k in 0..crate::physical::SOLUTES {
                let transported = donor.solutes[k] * fraction;
                donor.solutes[k] -= transported;
                receiver.solutes[k] += transported;
                self.systems.solute_flow += transported;
                x.state.flux.solute_transport += transported * 0.5;
                y.state.flux.solute_transport += transported * 0.5;
            }
            for k in 0..2 {
                let transported = donor.gas[k] * fraction;
                donor.gas[k] -= transported;
                receiver.gas[k] += transported;
            }
            if xm.body.water > 1e-9 && ym.body.water > 1e-9 {
                for k in 0..crate::physical::SOLUTES {
                    let gradient =
                        xm.body.solutes[k] / xm.body.water - ym.body.solutes[k] / ym.body.water;
                    let transported = transfer(
                        &mut xm.body.solutes[k],
                        &mut ym.body.solutes[k],
                        DT as f64 * 0.002 * gate * gradient,
                    );
                    self.systems.solute_flow += transported.abs();
                    x.state.flux.solute_transport += transported.abs() * 0.5;
                    y.state.flux.solute_transport += transported.abs() * 0.5;
                }
                for k in 0..2 {
                    let gradient = xm.body.gas[k] / xm.body.water - ym.body.gas[k] / ym.body.water;
                    transfer(
                        &mut xm.body.gas[k],
                        &mut ym.body.gas[k],
                        DT as f64 * 0.002 * gate * gradient,
                    );
                }
            }
            x.state.pressure_gradient =
                (x.state.pressure_gradient - difference as f32 * gate as f32).clamp(-1., 1.);
            y.state.pressure_gradient =
                (y.state.pressure_gradient + difference as f32 * gate as f32).clamp(-1., 1.);
        }
    }
    pub(crate) fn particle_ecology(&mut self, e: &mut Evolution, core: &Core) {
        // Each cell's size is worked out once here, and again only for a cell
        // that has just fed and so grown.
        let mut radii: Vec<f32> = e.cells.iter().map(Cell::radius).collect();
        let dissolving = 1. - (-DT as f64 * 0.006).exp();
        let width = (radii.iter().copied().fold(0., f32::max)
            + self
                .systems
                .particles
                .iter()
                .map(MatterParticle::radius)
                .fold(0., f32::max))
        .max(1.6);
        let key = |p: body::V3| {
            (
                (p.x / width).floor() as i32,
                (p.y / width).floor() as i32,
                (p.z / width).floor() as i32,
            )
        };
        // Cells are not moved here, so their places are read from one list.
        let places: Vec<body::V3> = e.cells.iter().map(|c| c.p).collect();
        let mut bins = crate::body::Bins::default();
        for (i, &at) in places.iter().enumerate() {
            bins.entry(key(at)).or_default().push(i);
        }
        for p in &mut self.systems.particles {
            let (x, z) = self.domain.coords(p.p);
            let site = self.domain.index(x, z);
            let up = self.domain.up(p.p);
            let fluid = if let Some(flow) = &self.fluid.surface {
                flow.velocities[site]
            } else {
                let u = self.fluid.velocity(site);
                body::V3::new(u[0] as f32, 0., u[1] as f32)
            };
            // Water carries a grain only as far as it covers it: a film over
            // nearly dry ground moves almost nothing, however fast it runs.
            // Out of the water a grain has its full weight and falls. In it,
            // remains hang as tissue does and only mineral sinks.
            let (carried, falling) = if self.domain.spherical() {
                let planet = self.planet.as_ref().unwrap();
                let bed = self.domain.ground(x, z, core.config.seed) + planet.relief_at(x, z);
                let depth = planet.water_over(site, bed, core.config.seed);
                let surface = bed + depth;
                let above = p.p.len() - self.domain.radius() - surface;
                // Something now and then throws a grain upward far faster
                // than anything in the sea moves. Until that is found, a
                // grain well clear of the surface is set back down on it.
                if above > 2. {
                    p.p = up.mul(self.domain.radius() + surface + p.radius());
                    p.v = p.v.sub(up.mul(p.v.dot(up)));
                }
                if above > 0. {
                    (0., crate::physical::GRAVITY as f32)
                } else {
                    ((depth / (2. * p.radius())).min(1.), 0.)
                }
            } else {
                (1., 0.)
            };
            p.v =
                p.v.add(fluid.sub(p.v).mul(DT * 0.4 * carried))
                    // Remains sink slowly, as marine snow does; mineral and
                    // glass sink faster.
                    .sub(up.mul(
                        DT * (falling
                            + crate::physical::SINKING
                            + 0.03
                                * ((p.solutes[2] + p.solutes[crate::physical::SILICA])
                                    / (p.mass() + p.solutes[crate::physical::SILICA]).max(1e-12))
                                    as f32),
                    ));
            // Nothing in a sea moves faster than this.
            if p.v.len() > 10. {
                p.v = p.v.mul(10. / p.v.len());
            }
            p.p = p.p.add(p.v.mul(DT));
            // Loose grains ride the same eddies as free cells, so what is shed
            // in one spot scatters as it sinks instead of hanging in a column.
            if self.planet.as_ref().unwrap().surface_depth(site) > 0.05 {
                let turn = ((p.p.x * 12.9898 + p.p.y * 4.1414 + p.p.z * 78.233 + core.time * 37.719).sin()
                    * 43758.547)
                    .fract()
                    * std::f32::consts::TAU;
                p.p = p.p.add(
                    self.domain
                        .tangent(p.p, turn)
                        .mul(crate::physical::JOSTLE * self.steering.mixing.sqrt()),
                );
            }
            if !self.domain.spherical() {
                p.p.x = p.p.x.clamp(-self.domain.wall(), self.domain.wall());
                p.p.z = p.p.z.clamp(-self.domain.wall(), self.domain.wall());
            }
            let (x, z) = self.domain.coords(p.p);
            let floor = self
                .planet
                .as_ref()
                .unwrap()
                .floor(x, z, p.radius(), core.config.seed);
            let altitude = if self.domain.spherical() {
                p.p.len() - self.domain.radius()
            } else {
                p.p.y
            };
            let site = if altitude < floor {
                p.p = p.p.add(up.mul(floor - altitude));
                let speed = p.v.dot(up);
                if speed < 0. {
                    p.v = p.v.sub(up.mul(speed));
                }
                self.domain.locate(p.p)
            } else {
                self.domain.index(x, z)
            };
            for k in 0..crate::physical::SOLUTES {
                let dissolved = p.solutes[k] * dissolving;
                p.solutes[k] -= dissolved;
                self.solutes[site][k] += dissolved;
            }
            let mut grain = p.radius();
            let (x, y, z) = key(p.p);
            for yy in y - 1..=y + 1 {
                for zz in z - 1..=z + 1 {
                    for xx in x - 1..=x + 1 {
                        let Some(neighbors) = bins.get(&(xx, yy, zz)) else {
                            continue;
                        };
                        for &i in neighbors {
                            if places[i].sub(p.p).len() > radii[i] + grain {
                                continue;
                            }
                            let c = &mut e.cells[i];
                            let gate = c.activity(15).max(c.activity(9)) as f64;
                            let mineral_gate = c.activity(3) as f64;
                            let m = c.material.as_mut().unwrap();
                            let extracted = (DT as f64 * 0.02 * gate)
                                .min(p.solutes[5])
                                .min((m.biomass * 1.5 - m.fuel).max(0.) / 0.85)
                                .min(c.energy as f64 / 0.2);
                            let unbound = if p.solutes[5] > 0. {
                                (p.solutes[crate::physical::REMAINS_N] * extracted * 0.85 / p.solutes[5])
                                    .min(p.solutes[crate::physical::REMAINS_N])
                            } else {
                                0.
                            };
                            p.solutes[crate::physical::REMAINS_N] -= unbound;
                            m.nitrogen += unbound;
                            p.solutes[5] -= extracted * 0.85;
                            m.fuel += extracted * 0.85;
                            self.web.feed(
                                &mut c.state,
                                m.organic(),
                                extracted * 0.85,
                                crate::trophic::Source::Dead,
                            );
                            c.state.flux.detritus += extracted * 0.85;
                            self.physiology.digested += extracted;
                            self.physiology.excreted += extracted * 0.15;
                            let before = c.energy as f64;
                            c.energy = (before - extracted * 0.2) as f32;
                            self.heat += before - c.energy as f64;
                            let mineral = (DT as f64 * 0.01 * mineral_gate)
                                .min(p.solutes[2])
                                .min(c.energy as f64 / 0.2);
                            p.solutes[2] -= mineral;
                            m.mineral += mineral;
                            c.state.flux.minerals += mineral;
                            let before = c.energy as f64;
                            c.energy = (before - mineral * 0.2) as f32;
                            self.heat += before - c.energy as f64;
                            if extracted > 0. || mineral > 0. {
                                radii[i] = c.radius();
                                grain = p.radius();
                            }
                            if extracted > 0. {
                                c.state.meals += 1;
                                e.meals += 1;
                            }
                        }
                    }
                }
            }
        }
        for p in &mut self.systems.particles {
            if p.mass() <= 1e-12 {
                let site = self.domain.locate(p.p);
                for k in 0..crate::physical::SOLUTES {
                    self.solutes[site][k] += p.solutes[k];
                    p.solutes[k] = 0.;
                }
            }
        }
        self.systems.particles.retain(|p| p.mass() > 0.);
    }
    pub(crate) fn systems_view(&self, e: &Evolution) -> serde_json::Value {
        let stores: Vec<_> = e.cells.iter().filter_map(|c| c.material.as_ref()).collect();
        let species: BTreeSet<_> = stores
            .iter()
            .flat_map(|m| {
                m.body
                    .polymers
                    .iter()
                    .filter(|p| p.amount > 1e-7)
                    .map(|p| &p.sequence)
            })
            .collect();
        let fluid_cells = stores.iter().filter(|m| m.body.water > 1e-6).count();
        let matrix = stores.iter().map(|m| m.body.matrix).sum::<f64>();
        let fibers = stores.iter().map(|m| m.body.fibers).sum::<f64>();
        let ids = e.index();
        let fluid_edges = e
            .bonds
            .iter()
            .filter(|b| {
                b.integrity > 0.
                    && [b.a, b.b]
                        .iter()
                        .all(|id| e.cells[ids[id]].material.as_ref().unwrap().body.water > 1e-6)
            })
            .count();
        let s = &self.systems;
        let ledger = serde_json::json!({"waterFlow":s.water_flow,"soluteFlow":s.solute_flow,"membraneExchange":s.membrane_exchange,"absorbed":s.absorbed,"filtered":s.filtered,"gasExchange":s.gas_exchange,"respired":s.respired,"pumpWork":s.pump_work,"matrixBuilt":s.matrix_built,"fiberBuilt":s.fiber_built,"mineralRemodeled":s.mineral_remodeled,"polymersBuilt":s.polymers_built,"fidelityWork":s.fidelity_work,"matrixMeals":s.matrix_meals});
        serde_json::json!({"ledger":ledger,"particles":self.systems.particles.iter().map(|p|serde_json::json!({"p":p.p,"radius":p.radius(),"mineral":p.solutes[2]/p.mass().max(1e-12)})).collect::<Vec<_>>(),"fluidCells":fluid_cells,"fluidConnections":fluid_edges,"internalWater":stores.iter().map(|m|m.body.water).sum::<f64>(),"internalSolutes":stores.iter().map(|m|m.body.dissolved_organic()).sum::<f64>(),"matrix":matrix,"fibers":fibers,"polymerSpecies":species.len(),"polymerMass":stores.iter().map(|m|m.body.polymers.iter().map(|p|p.amount).sum::<f64>()).sum::<f64>(),"gasError":self.systems.gas_error(&e.cells),"germlineCells":e.cells.iter().filter(|c|c.state.germline>0.25).count(),"pressureMax":e.cells.iter().map(|c|c.state.pressure).fold(0.,f32::max)})
    }
}
