//! Cell-local physiology, not adult organs or supplied behavioral controllers.
//! A designed reduced vocabulary: evolution chooses where/when to produce it.
use crate::{
    autonomous::{Cell, Evolution, INPUTS},
    physical::Habitat,
    *,
};

pub const LIPID: usize = 10;
/// How fast excitable cells fire and recover. At 3 a steadily stimulated
/// cell beats about once every seventeen cycles: quick enough to row with.
const NERVE_PACE: f32 = 3.;
pub const RECEPTOR: usize = 11;
pub const CHANNEL: usize = 12;
pub const REPAIR: usize = 13;
pub const BARRIER: usize = 14;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CellState {
    pub excitation: f32,
    pub recovery: f32,
    pub nutrient_memory: f32,
    pub food_change: f32,
    pub injury: f32,
    pub strain: f32,
    pub exposure: f32,
    pub ground: bool,
    pub motor_work: f64,
    pub meals: u32,
    pub light: f32,
    pub neural_input: f32,
    pub pressure: f32,
    pub pressure_gradient: f32,
    pub germline: f32,
    pub light_contrast: f32,
    pub flux: crate::observations::Flux,
    /// Member of a lineage the player chose to favour; inherited at division.
    pub favoured: bool,
    /// Signed steering command from the lateral senses; positive turns left.
    pub turn: f32,
    /// Whether the last tick's contacts and fluid let the cell push itself.
    pub wet: f32,
    /// Which side other life lies on, close by: positive is left.
    pub nearby: f32,
    /// Light-harvesting capacity of a cell swallowed and kept alive inside
    /// this one. Inherited at division: the step that made complex cells.
    pub symbiont: f32,
    /// 0 for a stem cell that can still become anything; 1 to 3 once it has
    /// committed. Daughters inherit it. A cell on its own is always a stem cell.
    pub fate: u8,
    /// Removed by the player's selection: its body returns to the world as
    /// fuel the survivors can use, rather than as remains.
    pub culled: bool,
}
impl CellState {
    pub fn valid(&self) -> bool {
        self.flux.valid()
            && [
                self.excitation,
                self.recovery,
                self.nutrient_memory,
                self.injury,
                self.exposure,
            ]
            .iter()
            .all(|x| x.is_finite() && (0.0..=1.).contains(x))
            && self.food_change.is_finite()
            && self.food_change.abs() <= 1.
            && self.strain.is_finite()
            && (0.0..=2.).contains(&self.strain)
            && self.motor_work.is_finite()
            && self.motor_work >= 0.
            && self.light.is_finite()
            && (0.0..=1.).contains(&self.light)
            && self.neural_input.is_finite()
            && self.neural_input.abs() <= 1.
            && self.pressure.is_finite()
            && (0.0..=4.).contains(&self.pressure)
            && self.pressure_gradient.is_finite()
            && self.pressure_gradient.abs() <= 1.
            && self.germline.is_finite()
            && (0.0..=1.).contains(&self.germline)
            && self.light_contrast.is_finite()
            && self.light_contrast.abs() <= 1.
            && self.turn.is_finite()
            && self.turn.abs() <= 1.
            && self.wet.is_finite()
            && (0.0..=1.).contains(&self.wet)
            && self.nearby.is_finite()
            && self.nearby.abs() <= 1.
            && self.fate < crate::autonomous::FATES
            && self.symbiont.is_finite()
            && (0.0..=1.).contains(&self.symbiont)
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Ledger {
    pub lipid_stored: f64,
    pub lipid_used: f64,
    pub fuel_transported: f64,
    pub energy_transported: f64,
    pub signal_work: f64,
    pub repairs: f64,
    pub digested: f64,
    pub excreted: f64,
    pub decomposed: f64,
    pub spikes: u32,
    pub contact_changes: u32,
}
impl Ledger {
    pub fn valid(&self) -> bool {
        [
            self.lipid_stored,
            self.lipid_used,
            self.fuel_transported,
            self.energy_transported,
            self.signal_work,
            self.repairs,
            self.digested,
            self.excreted,
            self.decomposed,
        ]
        .iter()
        .all(|x| x.is_finite() && *x >= 0.)
    }
}

impl Cell {
    /// Local membrane responses and short-lived cellular state. No resource or
    /// prey search across the world, target direction, fitness score or clock.
    pub(crate) fn physical_inputs(
        &mut self,
        solutes: &[[f64; 6]],
        degree: usize,
        domain: crate::domain::Domain,
    ) -> [f32; INPUTS] {
        let site = domain.locate(self.p);
        let density = domain.area(site) / 2.56;
        let s = solutes[site].map(|v| v / density);
        let sensed = s[0] + s[5] * self.activity(15) as f64;
        let food = (sensed / (0.5 + sensed)) as f32;
        let receptor = self.activity(RECEPTOR);
        self.state.food_change = (food - self.state.nutrient_memory).clamp(-1., 1.);
        self.state.nutrient_memory += (food - self.state.nutrient_memory) * 0.08;
        // Two round cells touch over a small patch, so one neighbour costs a
        // cell little of its surface; it is being packed in on every side
        // that buries it. Thin bodies feed almost freely, thick ones do not.
        self.state.exposure = (1. - (degree as f32 / 6.).powi(2)).clamp(0.05, 1.);
        // Samples one membrane radius along the cell's inherited spatial
        // polarity. This is a scalar promoter input; it never sets a force.
        let r = self.radius();
        let direction = domain.tangent(self.p, self.heading);
        let fi = domain.locate(self.p.add(direction.mul(r)));
        let bi = domain.locate(self.p.sub(direction.mul(r)));
        let front = solutes[fi][0] * 2.56 / domain.area(fi);
        let back = solutes[bi][0] * 2.56 / domain.area(bi);
        let gradient = ((front - back) / (0.5 + front + back)) as f32 * receptor;
        let m = self.material.as_ref().unwrap();
        [
            food,
            (s[2] / (0.1 + s[2])) as f32,
            (self.energy / 10.).min(1.),
            (degree as f32 / 6.).min(1.),
            self.chemicals[0],
            self.chemicals[1],
            (self.age / 80.).min(1.),
            (self.v.len() / 2.).min(1.),
            self.state.excitation,
            self.state.recovery,
            self.state.food_change * receptor,
            self.state.injury * receptor,
            self.state.exposure,
            (self.state.strain / 2.).min(1.) * receptor.max(self.activity(crate::organics::TOUCH)),
            (m.lipid / m.mass()) as f32,
            gradient,
            self.state.light * self.activity(crate::development::PHOTIC),
            self.state.neural_input,
            (self.lifecycle.depth as f32 / 32.).min(1.),
            (self.chemicals[0] - self.chemicals[1]).clamp(-1., 1.),
            self.state.pressure * 0.25,
            (m.body.solutes[0] / (0.02 + m.body.solutes[0])) as f32,
            (m.body.gas[0] / (0.005 + m.body.gas[0])) as f32,
            (m.body.solutes[1] / (0.02 + m.body.solutes[1])) as f32,
            self.scaffold() * (self.state.strain * 0.5).min(1.),
            self.state.light_contrast * self.activity(crate::development::PHOTIC),
            self.state.germline,
            self.state.pressure_gradient,
            // Left-minus-right senses, filled by the habitat where fields are known.
            0.,
            0.,
            0.,
            0.,
        ]
    }
    pub(crate) fn motor_signals(&self) -> [f32; 2] {
        let channels = self.activity(CHANNEL) * (self.energy / 0.02).clamp(0., 1.);
        [
            (self.chemicals[0] + channels * self.state.excitation * (1. - self.state.recovery))
                .min(1.),
            (self.chemicals[1] + channels * self.state.recovery * (1. - self.state.excitation))
                .min(1.),
        ]
    }
    pub(crate) fn material_role(&self) -> u8 {
        let Some(m) = &self.material else { return 0 };
        use crate::physical::{CHEMO, ENGULF, FIN, PROPEL, SPINE, TOXIN, WALL};
        if self.activity(WALL) > 0.25 {
            17
        } else if self.activity(FIN) > 0.25 {
            18
        } else if self.activity(ENGULF) > 0.25 {
            15
        } else if self.activity(TOXIN) > 0.25 {
            13
        } else if self.activity(SPINE) > 0.25 {
            14
        } else if self.activity(PROPEL) > 0.25 {
            12
        } else if m.mineral / m.mass() > 0.025 {
            1
        } else if self.activity(4) > 0.25 {
            2
        } else if m.lipid / m.mass() > 0.08 {
            6
        } else if self.activity(RECEPTOR).max(self.activity(CHANNEL)) > 0.25 {
            3
        } else if self.activity(9) > 0.25 {
            5
        } else if self.activity(BARRIER) > 0.25 {
            7
        } else if self.activity(8).max(self.activity(REPAIR)) > 0.25 {
            4
        } else if m.body.fibers / m.organic() > 0.02 {
            8
        } else if m.body.water > m.biomass * 0.1 {
            9
        } else if self
            .activity(crate::development::GAMETIC)
            .max(self.activity(crate::organics::FIDELITY))
            > 0.25
        {
            10
        } else if m.body.matrix / m.organic() > 0.02 {
            11
        } else if self.activity(CHEMO) > 0.25 {
            16
        } else {
            0
        }
    }
}
impl Habitat {
    /// Adipose-like carbon storage with the same chemical-energy convention as
    /// reduced fuel. Storage isn't free ATP: it must be mobilized and respired.
    pub(crate) fn reserves_and_repair(&mut self, c: &mut Cell) {
        let enzyme = c.activity(LIPID) as f64;
        let repair = c.activity(REPAIR) as f64;
        let m = c.material.as_mut().unwrap();
        let amount = if c.energy > 6. {
            (DT as f64 * 0.025 * enzyme)
                .min((m.fuel - m.biomass * 0.6).max(0.))
                .min((m.biomass * 1.5 - m.lipid).max(0.))
                .min(c.energy as f64 / 0.2)
        } else {
            0.
        };
        m.fuel -= amount;
        m.lipid += amount;
        self.physiology.lipid_stored += amount;
        self.spend(c, amount * 0.2, false);
        let m = c.material.as_mut().unwrap();
        if c.energy < 4.5 || m.fuel < m.biomass * 0.15 {
            let used = (DT as f64 * 0.035 * enzyme).min(m.lipid);
            m.lipid -= used;
            m.fuel += used;
            self.physiology.lipid_used += used;
        }
        let repaired = (DT as f64 * 0.008 * repair * c.state.injury as f64)
            .min(m.fuel)
            .min(c.energy as f64 / 0.6);
        m.fuel -= repaired;
        m.biomass += repaired;
        c.state.injury = (c.state.injury - repaired as f32 / (m.biomass as f32).max(0.02)).max(0.);
        self.physiology.repairs += repaired;
        self.spend(c, repaired * 0.6, false);
    }
    pub(crate) fn tissue_transport(&mut self, e: &mut Evolution) {
        let ids = e.index();
        for b in &e.bonds {
            if b.integrity <= 0. {
                continue;
            }
            let (a, c) = (ids[&b.a], ids[&b.b]);
            let (left, right) = e.cells.split_at_mut(a.max(c));
            let (x, y) = if a < c {
                (&mut left[a], &mut right[0])
            } else {
                (&mut right[0], &mut left[c])
            };
            let mobility = (x.activity(8) + y.activity(8)) as f64 * 0.5 * b.integrity as f64;
            // A mouth is plumbed into the body it belongs to. Fuel runs across
            // its bonds toward whichever side is emptier: out after a meal, and
            // back in when there is nothing to catch. Carrying it deeper than a
            // mouth's own neighbours takes transport tissue.
            let mouths = (
                x.activity(crate::physical::ENGULF),
                y.activity(crate::physical::ENGULF),
            );
            let gut = mouths.0.max(mouths.1) as f64 * b.integrity as f64;
            let (xm, ym) = (x.material.as_mut().unwrap(), y.material.as_mut().unwrap());
            let slope = xm.fuel / xm.biomass - ym.fuel / ym.biomass;
            // Transport tissue carries fuel between any two cells that build it.
            let carried = slope * DT as f64 * 0.3 * mobility;
            // Never past the receiver's capacity: a gut fills, it does not burst.
            let room = |m: &crate::physical::Inventory| (m.biomass * 1.5 - m.fuel).max(0.);
            let poured = (slope * DT as f64 * 0.5 * gut).clamp(-room(xm), room(ym));
            let flow = (carried + poured).clamp(-ym.fuel, xm.fuel);
            xm.fuel -= flow;
            ym.fuel += flow;
            self.physiology.fuel_transported += flow.abs();
            // What a mouth hands to an unarmed neighbour is how a body is fed.
            if flow > 0. && mouths.0 > 0.25 && mouths.1 <= 0.25 {
                x.state.flux.shared += flow;
                y.state.flux.fed += flow;
            } else if flow < 0. && mouths.1 > 0.25 && mouths.0 <= 0.25 {
                y.state.flux.shared -= flow;
                x.state.flux.fed -= flow;
            }
            // Transfer existing ATP-like stores, never a colony-wide energy pool.
            let difference = x.energy / xm.biomass as f32 - y.energy / ym.biomass as f32;
            let flow = (difference as f64 * DT as f64 * 0.025 * mobility)
                .clamp(-(y.energy as f64) * 0.2, x.energy as f64 * 0.2);
            let (donor, receiver) = if flow >= 0. {
                (&mut *x, &mut *y)
            } else {
                (&mut *y, &mut *x)
            };
            let before_d = donor.energy as f64;
            let before_r = receiver.energy as f64;
            let amount = flow.abs().min((9. - before_r).max(0.));
            donor.energy = (before_d - amount) as f32;
            let moved = before_d - donor.energy as f64;
            receiver.energy = (before_r + moved) as f32;
            // Float conversion may round up; choose the adjacent lower float so
            // every conversion loss goes to heat and ATP cannot be created.
            if donor.energy as f64 + receiver.energy as f64 > before_d + before_r {
                receiver.energy = f32::from_bits(receiver.energy.to_bits().saturating_sub(1));
            }
            self.heat += before_d + before_r - donor.energy as f64 - receiver.energy as f64;
            self.physiology.energy_transported += (receiver.energy as f64 - before_r).max(0.);
        }
    }
    /// Bounded excitable/refractory dynamics with synchronous gap coupling.
    /// An effective reaction system, not Hodgkin-Huxley biophysics or learning.
    /// Channels, receptor input and signal maintenance require produced stores.
    pub(crate) fn excitable_signals(&mut self, e: &mut Evolution) {
        let ids = e.index();
        let gates: Vec<_> = e
            .cells
            .iter()
            .map(|c| c.activity(CHANNEL) * (c.energy / 0.02).clamp(0., 1.))
            .collect();
        let mut state: Vec<_> = e
            .cells
            .iter()
            .map(|c| [c.state.excitation, c.state.recovery])
            .collect();
        let start = state.clone();
        for _ in 0..8 {
            let mut coupling = vec![0.; e.cells.len()];
            for b in &e.bonds {
                let (a, c) = (ids[&b.a], ids[&b.b]);
                let conductivity = gates[a].min(gates[c])
                    * b.integrity
                    * (e.cells[a].activity(8) + e.cells[c].activity(8))
                    * 0.15;
                let flow = (state[c][0] - state[a][0]) * conductivity;
                coupling[a] += flow;
                coupling[c] -= flow;
            }
            let old = state.clone();
            for (i, c) in e.cells.iter().enumerate() {
                let [u, v] = old[i];
                let stimulus = c.activity(RECEPTOR)
                    * (c.state.food_change * 1.5
                        + c.state.injury * 0.5
                        + c.state.strain * 0.1
                        + if c.state.ground { 0.03 } else { 0. })
                    + c.activity(crate::development::PHOTIC) * c.state.light * 0.5
                    + c.activity(crate::organics::TOUCH)
                        * (c.state.strain * 0.2 + c.state.pressure_gradient * 0.1)
                    + c.state.neural_input;
                // Recovery pulls hard enough that a cell held under steady
                // stimulation, by light or by signal A, fires again and again
                // across a wide range of strengths instead of locking on.
                let du = 3. * u * (1. - u) * (u - 0.18) - v
                    + stimulus
                    + 0.4 * (c.chemicals[0] - c.chemicals[1])
                    + coupling[i];
                let dv = 0.12 * (u - v * 0.8);
                state[i] = [
                    (u + DT / 8. * NERVE_PACE * gates[i] * du).clamp(0., 1.),
                    (v + DT / 8. * NERVE_PACE * gates[i] * dv).clamp(0., 1.),
                ];
            }
        }
        for (i, c) in e.cells.iter_mut().enumerate() {
            let required = DT as f64 * 0.002 * gates[i] as f64 * (0.1 + state[i][0] as f64);
            let paid = self.spend(c, required, false);
            self.physiology.signal_work += paid;
            c.state.excitation = state[i][0];
            c.state.recovery = state[i][1];
            if start[i][0] < 0.65 && state[i][0] >= 0.65 {
                self.physiology.spikes += 1;
            }
        }
    }
    pub(crate) fn physiology_view(&self, e: &Evolution) -> serde_json::Value {
        // Observation only: connected material-bearing regions. These thresholds
        // never change physics, survival, reproduction or an anatomical stage.
        let names = [
            "Mineral support",
            "Contractile",
            "Storage",
            "Receptive",
            "Excitable",
            "Transport",
            "Digestive",
            "Repair / barrier",
            "Photoreceptive",
            "Adaptive signaling",
            "Gametic / fusion",
            "Polar scaffold",
            "Secreted matrix",
            "Hydraulic pumps",
            "Gas exchange",
            "Mechanical sensing",
            "Membrane recognition",
            "Replication repair",
            "Matrix digestion",
            "Fiber crosslinking",
            "Support remodeling",
            "Sequence assembly",
            "Membrane lysis",
        ];
        let present: Vec<_> = e
            .cells
            .iter()
            .map(|c| {
                let m = c.material.as_ref().unwrap();
                [
                    m.mineral / m.mass() > 0.025,
                    c.activity(4) > 0.25,
                    m.lipid / m.mass() > 0.08,
                    c.activity(RECEPTOR) > 0.25,
                    c.activity(CHANNEL) > 0.25,
                    c.activity(8) > 0.25,
                    c.activity(9) > 0.25,
                    c.activity(REPAIR).max(c.activity(BARRIER)) > 0.25,
                    c.activity(crate::development::PHOTIC) > 0.25,
                    c.activity(crate::development::SYNAPSE) > 0.25,
                    c.activity(crate::development::GAMETIC)
                        .max(c.activity(crate::development::FUSOGEN))
                        > 0.25,
                    c.activity(crate::development::POLARITY) > 0.25,
                    m.body.matrix > m.biomass * 0.015,
                    c.activity(crate::organics::PUMP) > 0.25,
                    c.activity(crate::organics::GAS) > 0.25,
                    c.activity(crate::organics::TOUCH) > 0.25,
                    c.activity(crate::organics::RECOGNITION) > 0.25,
                    c.state.germline > 0.25,
                    c.activity(crate::organics::MATRIX_DIGEST) > 0.25,
                    m.body.fibers > m.biomass * 0.015,
                    c.activity(crate::organics::REMODEL) > 0.25,
                    c.activity(crate::organics::ASSEMBLY) > 0.25,
                    c.activity(crate::organics::LYSIS) > 0.25,
                ]
            })
            .collect();
        let ids = e.index();
        let mut adjacency = vec![Vec::new(); e.cells.len()];
        for b in &e.bonds {
            if b.integrity > 0. {
                let (a, c) = (ids[&b.a], ids[&b.b]);
                adjacency[a].push(c);
                adjacency[c].push(a);
            }
        }
        let mut regions = Vec::new();
        for (k, name) in names.iter().enumerate() {
            let mut visited = vec![false; e.cells.len()];
            let mut sizes = Vec::new();
            for i in 0..e.cells.len() {
                if visited[i] || !present[i][k] {
                    continue;
                }
                let mut todo = vec![i];
                visited[i] = true;
                let mut n = 0;
                while let Some(a) = todo.pop() {
                    n += 1;
                    for &b in &adjacency[a] {
                        if !visited[b] && present[b][k] {
                            visited[b] = true;
                            todo.push(b);
                        }
                    }
                }
                if n >= 3 {
                    sizes.push(n);
                }
            }
            regions.push(
                serde_json::json!({"function":name,"cells":present.iter().filter(|x|x[k]).count(),
                "regions":sizes.len(),"largest":sizes.into_iter().max().unwrap_or(0)}),
            );
        }
        let mixed = e
            .groups()
            .iter()
            .filter(|g| {
                if g.len() < 3 {
                    return false;
                }
                let mut kinds = [false; 23];
                for &i in *g {
                    for k in 0..23 {
                        kinds[k] |= present[i][k];
                    }
                }
                kinds.iter().filter(|&&x| x).count() >= 3
            })
            .count();
        serde_json::json!({"ledger":self.physiology,"regions":regions,"mixedBodies":mixed,
            "lipid":e.cells.iter().map(|c|c.material.as_ref().unwrap().lipid).sum::<f64>(),
            "excited":e.cells.iter().filter(|c|c.state.excitation>0.65).count(),
            "injured":e.cells.iter().filter(|c|c.state.injury>0.05).count(),
            "grounded":e.cells.iter().filter(|c|c.state.ground).count()})
    }
}
