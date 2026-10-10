//! Measured material fluxes. These counters do not influence selection or physics.
use crate::{autonomous::Evolution, physical::Habitat, *};
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Flux {
    pub fuel: f64,
    pub detritus: f64,
    pub photosynthesis: f64,
    pub prey: f64,
    pub lost_to_predators: f64,
    pub defense_blocked: f64,
    pub minerals: f64,
    pub waste: f64,
    pub organic_waste: f64,
    pub signal_a: f64,
    pub signal_b: f64,
    pub proteins: f64,
    pub absorbed: f64,
    pub filtered: f64,
    pub respiration: f64,
    pub water_transport: f64,
    pub solute_transport: f64,
    pub matrix: f64,
    pub fibers: f64,
}
impl Flux {
    pub fn add(&mut self, b: &Self) {
        self.fuel += b.fuel;
        self.detritus += b.detritus;
        self.photosynthesis += b.photosynthesis;
        self.prey += b.prey;
        self.lost_to_predators += b.lost_to_predators;
        self.defense_blocked += b.defense_blocked;
        self.minerals += b.minerals;
        self.waste += b.waste;
        self.organic_waste += b.organic_waste;
        self.signal_a += b.signal_a;
        self.signal_b += b.signal_b;
        self.proteins += b.proteins;
        self.absorbed += b.absorbed;
        self.filtered += b.filtered;
        self.respiration += b.respiration;
        self.water_transport += b.water_transport;
        self.solute_transport += b.solute_transport;
        self.matrix += b.matrix;
        self.fibers += b.fibers;
    }
    pub fn valid(&self) -> bool {
        [
            self.fuel,
            self.detritus,
            self.photosynthesis,
            self.prey,
            self.lost_to_predators,
            self.defense_blocked,
            self.minerals,
            self.waste,
            self.organic_waste,
            self.signal_a,
            self.signal_b,
            self.proteins,
            self.absorbed,
            self.filtered,
            self.respiration,
            self.water_transport,
            self.solute_transport,
            self.matrix,
            self.fibers,
        ]
        .iter()
        .all(|x| x.is_finite() && *x >= 0.)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Meal {
    pub time: f32,
    pub attacker: u32,
    pub prey: u32,
    #[serde(default)]
    pub attacker_genome: u32,
    #[serde(default)]
    pub prey_genome: u32,
    pub material: String,
    pub amount: f64,
}
impl Habitat {
    pub(crate) fn record_meal(
        &mut self,
        time: f32,
        attacker: u32,
        prey: u32,
        attacker_genome: u32,
        prey_genome: u32,
        material: &str,
        amount: f64,
    ) {
        if amount <= 1e-12 {
            return;
        }
        if let Some(m) = self.meals.iter_mut().rev().find(|m| {
            m.attacker == attacker
                && m.prey == prey
                && m.attacker_genome == attacker_genome
                && m.prey_genome == prey_genome
                && m.material == material
                && time - m.time < 0.5
        }) {
            m.amount += amount;
            m.time = time;
            return;
        }
        if self.meals.len() >= 24 {
            self.meals.remove(0);
        }
        self.meals.push(Meal {
            time,
            attacker,
            prey,
            attacker_genome,
            prey_genome,
            material: material.into(),
            amount,
        });
    }
    pub(crate) fn observation(&self, e: &Evolution, group: &[usize]) -> serde_json::Value {
        let mut flux = Flux::default();
        let mut center = body::V3::default();
        let mut stocks = [0f64; 8];
        let mut functions = [0usize; autonomous::ACTIONS];
        let mut signature = 0u64;
        for &i in group {
            let c = &e.cells[i];
            flux.add(&c.state.flux);
            center = center.add(c.p);
            signature = signature.wrapping_add((c.id as u64).wrapping_mul(0x9e3779b97f4a7c15));
            if let Some(m) = &c.material {
                stocks[0] += m.fuel;
                stocks[1] += m.biomass;
                stocks[2] += m.mineral;
                stocks[3] += m.body.water;
                stocks[4] += m.body.matrix;
                stocks[5] += m.body.fibers;
                stocks[6] += m.lipid;
                stocks[7] += c.energy as f64;
            }
            for (k, n) in functions.iter_mut().enumerate() {
                if c.activity(k) > 0.1 {
                    *n += 1;
                }
            }
        }
        center = center.mul(1. / group.len().max(1) as f32);
        let site = self.domain.locate(center);
        serde_json::json!({"flux":flux,"stocks":stocks,"functions":functions,"centroid":center,"membership":signature.to_string(),"members":group.len(),"site":site,"ambientFlow":if let Some(flow)=&self.fluid.surface {flow.velocities[site]}else{let v=self.fluid.velocity(site);body::V3::new(v[0] as f32,0.,v[1] as f32)},"localResources":self.solutes[site].map(|v|v*2.56/self.domain.area(site))})
    }
    /// Spatial resource differences are environmental forcing, not a trait reward.
    pub(crate) fn light_fraction(&self, site: usize) -> f64 {
        self.shade.get(site).copied().unwrap_or(1.) as f64
    }
}
