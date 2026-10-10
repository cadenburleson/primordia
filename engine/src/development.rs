//! Reduced inherited development, contact signaling and a gamete/zygote cycle.
//! These are physical affordances, never adult anatomy or a complexity reward.
use crate::{
    autonomous::{Cell, Evolution, Genome, ACTIONS, MAX_GENES},
    body::V3,
    physical::{Habitat, MaterialJoint},
    *,
};

pub const PHOTIC: usize = 16;
pub const SYNAPSE: usize = 17;
pub const GAMETIC: usize = 18;
pub const FUSOGEN: usize = 19;
pub const POLARITY: usize = 20;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    #[default]
    Somatic,
    Gamete,
    Zygote,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Lifecycle {
    /// Identity of the reproductive bottleneck, not the current contact group.
    pub root: u32,
    pub depth: u32,
    pub phase: Phase,
    pub mate: u32,
    /// A second variable chromosome makes this cell diploid in the model.
    pub homolog: Option<Genome>,
}
impl Lifecycle {
    pub fn valid(&self, c: &Cell) -> bool {
        self.root <= c.id
            && self.mate < c.id
            && self.depth <= 1_000_000
            && self.homolog.as_ref().is_none_or(Genome::valid)
            && (self.phase != Phase::Gamete || (self.homolog.is_none() && c.parent > 0))
            && (self.phase != Phase::Zygote
                || (self.homolog.is_some() && self.mate > 0 && self.root == c.id))
    }
}
impl Cell {
    pub fn body_origin(&self) -> u32 {
        if self.lifecycle.root == 0 {
            self.founder
        } else {
            self.lifecycle.root
        }
    }
    pub(crate) fn cellular_expression(&self, inputs: [f32; autonomous::INPUTS]) -> [f32; ACTIONS] {
        let mut expression = self.genome.express(inputs);
        if let Some(other) = &self.lifecycle.homolog {
            for (a, b) in expression.iter_mut().zip(other.express(inputs)) {
                *a = (*a + b) * 0.5;
            }
        }
        expression
    }
    /// One crossover in an abstract variable-length chromosome. Gene order is
    /// the alignment; this does not model DNA sequence, chromosomes or meiosis machinery.
    pub(crate) fn gamete_chromosome(&self, core: &mut Core) -> Genome {
        let Some(other) = &self.lifecycle.homolog else {
            return self.genome.clone();
        };
        let (a, b) = if core.random() < 0.5 {
            (&self.genome, other)
        } else {
            (other, &self.genome)
        };
        let cut = (core.random() * (a.genes.len().min(b.genes.len()) + 1) as f32) as usize;
        let genes = a.genes[..cut]
            .iter()
            .chain(&b.genes[cut..])
            .take(MAX_GENES)
            .cloned()
            .collect();
        Genome { id: a.id, genes }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Synapse {
    pub from: u32,
    pub to: u32,
    pub weight: f32,
    pub pre: f32,
    pub post: f32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Development {
    pub synapses: Vec<Synapse>,
    pub sensory_work: f64,
    pub neural_work: f64,
    pub adaptations: u64,
    pub transmissions: u64,
    pub gametes: u32,
    pub zygotes: u32,
    pub meioses: u32,
    pub fusion_work: f64,
    pub ancestry: Vec<SexualBirth>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SexualBirth {
    pub time: f32,
    pub child: u32,
    pub parent: u32,
    pub mate: u32,
    pub generation: u32,
}
impl Development {
    pub fn valid(&self, cells: &[Cell], _joints: &[MaterialJoint]) -> bool {
        let ids: BTreeMap<_, _> = cells.iter().map(|c| (c.id, c)).collect();
        let mut edges = BTreeSet::new();
        self.synapses.len() <= cells.len() * 6
            && self.ancestry.len() <= 256
            && [self.sensory_work, self.neural_work, self.fusion_work]
                .iter()
                .all(|x| x.is_finite() && *x >= 0.)
            && self.synapses.iter().all(|s| {
                s.from != s.to
                    && ids.contains_key(&s.from)
                    && ids.contains_key(&s.to)
                    && edges.insert((s.from, s.to))
                    && s.weight.is_finite()
                    && s.weight.abs() <= 1.
                    && [s.pre, s.post]
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=1.).contains(v))
            })
            && self.ancestry.iter().all(|b| {
                b.time.is_finite()
                    && b.time >= 0.
                    && b.parent > 0
                    && b.mate > 0
                    && b.parent != b.mate
                    && b.parent < b.child
                    && b.mate < b.child
            })
    }
}

fn compatibility(a: &Cell, b: &Cell) -> f32 {
    let mask = |c: &Cell| c.genome.genes.iter().fold(0u32, |m, g| m | (1 << g.action));
    let (a, b) = (mask(a), mask(b));
    (a & b).count_ones() as f32 / (a | b).count_ones().max(1) as f32
}
impl Habitat {
    pub(crate) fn photoreception(&mut self, e: &mut Evolution) {
        let Some(planet) = &self.planet else { return };
        let sun = V3::new(
            planet.sun_direction[0],
            planet.sun_direction[1],
            planet.sun_direction[2],
        );
        let ids = e.index();
        let mut shade = vec![0f32; e.cells.len()];
        // Effective shading by attached neighbors; no global resource/prey map.
        for b in &e.bonds {
            for (a, n) in [(ids[&b.a], ids[&b.b]), (ids[&b.b], ids[&b.a])] {
                let delta = e.cells[n].p.sub(e.cells[a].p);
                let facing = delta.dot(sun) / delta.len().max(0.01);
                shade[a] += facing.max(0.) * 0.2 * b.integrity;
            }
        }
        let light: Vec<_> = e
            .cells
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let axis = self
                    .domain
                    .tangent(c.p, c.heading)
                    .add(self.domain.up(c.p))
                    .mul(std::f32::consts::FRAC_1_SQRT_2);
                (planet.illumination(self.domain.locate(c.p)) as f32
                    * self.light_fraction(self.domain.locate(c.p)) as f32
                    * axis.dot(sun).max(0.)
                    * (1. - shade[i]).max(0.))
                .clamp(0., 1.)
            })
            .collect();
        for (c, light) in e.cells.iter_mut().zip(light) {
            let required = DT as f64 * 0.001 * c.activity(PHOTIC) as f64;
            let paid = self.spend(c, required, false);
            self.development.sensory_work += paid;
            c.state.light = if required > 0. {
                light * (paid / required).min(1.) as f32
            } else {
                0.
            };
            let up = self.domain.up(c.p);
            let side = self.domain.tangent(c.p, c.heading);
            let a = up.add(side).normalized().dot(sun).max(0.);
            let b = up.sub(side).normalized().dot(sun).max(0.);
            c.state.light_contrast = ((a - b) * c.state.light).clamp(-1., 1.);
        }
    }

    pub(crate) fn neural_contacts(&mut self, e: &mut Evolution) {
        let ids = e.index();
        let mut old: BTreeMap<_, _> = std::mem::take(&mut self.development.synapses)
            .into_iter()
            .map(|s| ((s.from, s.to), s))
            .collect();
        let mut edges = Vec::new();
        let mut degrees = vec![0; e.cells.len()];
        for bond in &e.bonds {
            let (a, b) = (ids[&bond.a], ids[&bond.b]);
            if bond.integrity <= 0.
                || [a, b].iter().any(|&i| {
                    e.cells[i].lifecycle.phase == Phase::Gamete
                        || e.cells[i]
                            .activity(physiology::CHANNEL)
                            .min(e.cells[i].activity(SYNAPSE))
                            < 0.08
                })
            {
                continue;
            }
            for (a, b) in [(a, b), (b, a)] {
                let from = &e.cells[a];
                let to = &e.cells[b];
                degrees[a] += 1;
                let seed = from
                    .genome
                    .genes
                    .iter()
                    .chain(from.lifecycle.homolog.iter().flat_map(|g| &g.genes))
                    .filter(|g| g.action == SYNAPSE)
                    .map(|g| (g.promoter[8] - g.promoter[9] + g.bias * 0.2) * g.strength)
                    .sum::<f32>()
                    .clamp(-1., 1.);
                edges.push(old.remove(&(from.id, to.id)).unwrap_or(Synapse {
                    from: from.id,
                    to: to.id,
                    weight: seed,
                    pre: 0.,
                    post: 0.,
                }));
            }
        }
        let mut gates = vec![0.; e.cells.len()];
        for (i, c) in e.cells.iter_mut().enumerate() {
            c.state.neural_input = 0.;
            if degrees[i] == 0 {
                continue;
            }
            let gate = c.activity(SYNAPSE).min(c.activity(physiology::CHANNEL));
            let required = DT as f64 * 0.002 * degrees[i] as f64 * gate as f64;
            let paid = self.spend(c, required, false);
            self.development.neural_work += paid;
            gates[i] = gate * (paid / required).min(1.) as f32;
        }
        let mut drive = vec![0f32; e.cells.len()];
        for edge in &mut edges {
            let (a, b) = (ids[&edge.from], ids[&edge.to]);
            let gate = gates[a].min(gates[b]);
            let (pre, post) = (e.cells[a].state.excitation, e.cells[b].state.excitation);
            drive[b] += pre * edge.weight * gate / (degrees[b].max(1) as f32).sqrt();
            if gate > 0. {
                edge.pre += DT * gate * (pre - edge.pre);
                edge.post += DT * gate * (post - edge.post);
                // Local activity-dependent plasticity with bounded weight decay.
                // No reward, fitness score, target gait or inherited learned memory.
                if pre + post > 0.001 {
                    let change = DT
                        * 0.04
                        * gate
                        * ((pre - edge.pre) * (post - edge.post)
                            - 0.01 * edge.weight * post * post);
                    let next = (edge.weight + change).clamp(-1., 1.);
                    if next != edge.weight {
                        self.development.adaptations += 1;
                    }
                    edge.weight = next;
                }
                if pre > 0.01 {
                    self.development.transmissions += 1;
                }
            }
        }
        for (c, value) in e.cells.iter_mut().zip(drive) {
            c.state.neural_input = value.clamp(-1., 1.);
        }
        self.development.synapses = edges;
    }
    pub(crate) fn prune_synapses(&mut self, e: &Evolution) {
        let alive: BTreeSet<_> = e.cells.iter().map(|c| c.id).collect();
        let contacts: BTreeSet<_> = e
            .bonds
            .iter()
            .filter(|b| b.integrity > 0.)
            .map(|b| (b.a, b.b))
            .collect();
        self.development.synapses.retain(|s| {
            alive.contains(&s.from)
                && alive.contains(&s.to)
                && contacts.contains(&(s.from.min(s.to), s.from.max(s.to)))
        });
    }

    pub(crate) fn fertilize(
        &mut self,
        e: &mut Evolution,
        core: &mut Core,
        pairs: &[(usize, usize)],
    ) {
        let mut removed = BTreeSet::new();
        let mut embryos = Vec::new();
        for &(a, b) in pairs {
            let (ca, cb) = (&e.cells[a], &e.cells[b]);
            if removed.contains(&ca.id)
                || removed.contains(&cb.id)
                || ca.lifecycle.phase != Phase::Gamete
                || cb.lifecycle.phase != Phase::Gamete
                || ca.body_origin() == cb.body_origin()
                || ca.parent == cb.parent
                || ca.activity(FUSOGEN).min(cb.activity(FUSOGEN)) < 0.2
                || compatibility(ca, cb) < 0.25
                || ca.energy < 0.15
                || cb.energy < 0.15
                || ca.state.injury >= 0.98
                || cb.state.injury >= 0.98
                || ca.p.sub(cb.p).len() > ca.radius() + cb.radius() + 0.12
                || core.random()
                    >= ca.activity(FUSOGEN).min(cb.activity(FUSOGEN))
                        * ca.recognition(cb)
                        * DT
                        * 0.8
            {
                continue;
            }
            self.development.fusion_work += self.spend(&mut e.cells[a], 0.1, false);
            self.development.fusion_work += self.spend(&mut e.cells[b], 0.1, false);
            let (ca, cb) = (&e.cells[a], &e.cells[b]);
            let mut embryo = ca.clone();
            embryo.collection_sources=ca.collection_sources.iter().chain(&cb.collection_sources).cloned().collect::<BTreeSet<_>>().into_iter().take(16).collect();
            embryo.id = e.next_cell;
            e.next_cell += 1;
            embryo.parent = ca.parent;
            embryo.generation = ca.generation.max(cb.generation).saturating_add(1);
            embryo.lifecycle = Lifecycle {
                root: embryo.id,
                depth: 0,
                phase: Phase::Zygote,
                mate: cb.parent,
                homolog: Some(cb.genome.clone()),
            };
            // Recombination during gamete production created a new sequence. All
            // recombinant sequences and the new diploid combination receive new IDs.
            embryo.genome.id = e.next_genome;
            e.next_genome += 1;
            let (ma, mb) = (ca.material.as_ref().unwrap(), cb.material.as_ref().unwrap());
            let fraction = (mb.mass() / (ma.mass() + mb.mass())) as f32;
            embryo.p = ca.p.mul(1. - fraction).add(cb.p.mul(fraction));
            embryo.v = ca.v.mul(1. - fraction).add(cb.v.mul(fraction));
            let inventory = embryo.material.as_mut().unwrap();
            inventory.biomass += mb.biomass;
            inventory.fuel += mb.fuel;
            inventory.mineral += mb.mineral;
            inventory.lipid += mb.lipid;
            inventory.fuel += inventory.body.merge(&mb.body);
            for (a, b) in inventory.proteins.iter_mut().zip(mb.proteins) {
                *a += b;
            }
            let total = ca.energy as f64 + cb.energy as f64;
            let mut rounded = total as f32;
            if rounded as f64 > total {
                rounded = f32::from_bits(rounded.to_bits() - 1);
            }
            embryo.energy = rounded;
            self.heat += total - rounded as f64;
            embryo.age = 0.;
            embryo.cooldown = 3.;
            embryo.state = physiology::CellState {
                favoured: ca.state.favoured || cb.state.favoured,
                ..Default::default()
            };
            embryo.chemicals = [0.; 2];
            embryo.expression = [0.; ACTIONS];
            removed.insert(ca.id);
            removed.insert(cb.id);
            self.development.zygotes += 1;
            e.peak_generation = e.peak_generation.max(embryo.generation);
            self.development.ancestry.push(SexualBirth {
                time: core.time,
                child: embryo.id,
                parent: embryo.parent,
                mate: embryo.lifecycle.mate,
                generation: embryo.generation,
            });
            if self.development.ancestry.len() > 256 {
                self.development.ancestry.remove(0);
            }
            embryos.push(embryo);
        }
        e.cells.retain(|c| !removed.contains(&c.id));
        e.cells.extend(embryos);
        e.bonds
            .retain(|b| !removed.contains(&b.a) && !removed.contains(&b.b));
        self.prune_synapses(e);
    }

    pub(crate) fn development_view(&self, e: &Evolution) -> serde_json::Value {
        serde_json::json!({"gametesProduced":self.development.gametes,"zygotes":self.development.zygotes,"meioses":self.development.meioses,
            "livingGametes":e.cells.iter().filter(|c| c.lifecycle.phase == Phase::Gamete).count(),
            "diploidCells":e.cells.iter().filter(|c| c.lifecycle.homolog.is_some()).count(),
            "synapses":self.development.synapses.len(),"transmissions":self.development.transmissions,"adaptations":self.development.adaptations,
            "sensoryWork":self.development.sensory_work,"neuralWork":self.development.neural_work,"fusionWork":self.development.fusion_work,
            "recent":self.development.ancestry.iter().rev().take(8).collect::<Vec<_>>()})
    }
}
