//! Who feeds on whom, and where the planet's matter and energy are. A cell's
//! place in the food chain follows the carbon it is built from. Nothing here
//! steers the physics: these are measurements.
use crate::{
    autonomous::{Cell, Evolution},
    physical::{Habitat, CO2, IRON, NITROGEN, REMAINS_N, SALT},
    physiology::CellState,
    *,
};

/// Producers, then three levels of eaters. Decomposers are counted apart.
pub const LEVELS: usize = 4;
pub const NAMES: [&str; LEVELS + 1] = [
    "Producers",
    "Plant-eaters",
    "Meat-eaters",
    "Top predators",
    "Decomposers",
];
/// Where a mouthful of carbon came from.
pub enum Source {
    /// Built from carbon dioxide with light or vent chemistry.
    Light,
    /// Remains, or matter dissolved in the water.
    Dead,
    /// Another living cell, that many steps above the producers.
    Prey(f32),
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Web {
    tick: [f64; LEVELS],
    tick_dead: f64,
    /// Carbon entering each level per cycle, smoothed over about fifty cycles.
    pub flow: [f64; LEVELS],
    /// Carbon taken from remains and dissolved matter per cycle.
    pub dead: f64,
}
impl Web {
    /// Records carbon taken in by a cell holding `held`, and moves the cell's
    /// place in the chain toward one step above its source.
    pub fn feed(&mut self, state: &mut CellState, held: f64, amount: f64, source: Source) {
        if amount <= 0. {
            return;
        }
        let (steps, dead) = match source {
            Source::Light => (0., 0.),
            Source::Dead => (1., 1.),
            Source::Prey(steps) => ((steps + 1.).min(LEVELS as f32 - 1.), 0.),
        };
        if dead > 0. {
            self.tick_dead += amount;
        } else {
            self.tick[(steps.round() as usize).min(LEVELS - 1)] += amount;
        }
        let share = (amount / held.max(1e-9)).min(1.) as f32;
        state.trophic += (steps - state.trophic) * share;
        state.scavenged += (dead - state.scavenged) * share;
    }
    pub fn settle(&mut self, dt: f64) {
        let keep = (-dt / 50.).exp();
        for k in 0..LEVELS {
            self.flow[k] = self.flow[k] * keep + self.tick[k] / dt * (1. - keep);
            self.tick[k] = 0.;
        }
        self.dead = self.dead * keep + self.tick_dead / dt * (1. - keep);
        self.tick_dead = 0.;
    }
    pub fn valid(&self) -> bool {
        self.flow
            .iter()
            .chain(&self.tick)
            .chain([&self.dead, &self.tick_dead])
            .all(|v| v.is_finite() && *v >= 0.)
    }
}
impl CellState {
    /// The level this cell feeds at: 0 producers, up to 3; 4 for a cell
    /// living mostly on remains and dissolved matter.
    pub fn level(&self) -> usize {
        if self.scavenged > 0.5 {
            LEVELS
        } else {
            (self.trophic.round() as usize).min(LEVELS - 1)
        }
    }
}
impl Habitat {
    /// Where the planet's carbon, nitrogen, oxygen and energy are, and how
    /// much life feeds at each level. Every amount is counted, none is made up:
    /// the errors are what the books fail to account for.
    pub fn cycles(&self, e: &Evolution) -> serde_json::Value {
        let living = |f: &dyn Fn(&crate::physical::Inventory) -> f64| {
            e.cells
                .iter()
                .filter_map(|c| c.material.as_ref())
                .map(f)
                .sum::<f64>()
        };
        let site = |k: usize| self.solutes.iter().map(|s| s[k]).sum::<f64>();
        let grain = |k: usize| self.systems.particles.iter().map(|p| p.solutes[k]).sum::<f64>();
        let errors = self.errors(&e.cells);
        let (organic, _, stored) = self.totals(&e.cells);
        let mut cells = [0usize; LEVELS + 1];
        let mut carbon = [0.; LEVELS + 1];
        for c in &e.cells {
            if let Some(m) = &c.material {
                cells[c.state.level()] += 1;
                carbon[c.state.level()] += m.organic() - m.body.solutes[CO2];
            }
        }
        let flows: Vec<f64> = self.web.flow.iter().copied().chain([self.web.dead]).collect();
        let levels: Vec<_> = (0..=LEVELS)
            .map(|k| serde_json::json!({"name":NAMES[k],"cells":cells[k],"carbon":carbon[k],"intake":flows[k]}))
            .collect();
        let efficiency: Vec<f64> = (1..LEVELS)
            .map(|k| if self.web.flow[k - 1] > 1e-9 { self.web.flow[k] / self.web.flow[k - 1] } else { 0. })
            .collect();
        let water = self.planet.as_ref().map_or([0.; 4], |p| {
            [p.atmosphere, p.water.iter().sum(), p.snow.iter().sum(), p.biological_water]
        });
        // Salt under standing water is dissolved; on ground that has dried it is a crust.
        let salt = self.planet.as_ref().map_or([site(SALT), 0.], |p| {
            self.solutes.iter().enumerate().fold([0.; 2], |mut sum, (i, s)| {
                sum[usize::from(p.water[i] / self.domain.area(i) < 0.01)] += s[SALT];
                sum
            })
        });
        let gas: [f64; 2] = self.systems.gas.iter().fold([0.; 2], |a, g| [a[0] + g[0], a[1] + g[1]]);
        let body_gas = living(&|m| m.body.gas[0]);
        // Each tracked element and where its stock is: in the air, in the
        // water, in the ground and ice, and in the living.
        let pools = |symbol: &str, note: &str, at: [f64; 4]| {
            serde_json::json!({"symbol":symbol,"note":note,"air":at[0],"water":at[1],"ground":at[2],"life":at[3]})
        };
        let elements = [
            pools("H", "Counted as water: vapour in the air, the seas, snow and ice, and the water held in bodies.", water),
            pools("C", "As carbon dioxide in the air and the water, as food dissolved in the water, in remains lying on the bed or buried under silt, and built into the living.",
                [self.air_carbon, site(CO2) + grain(CO2) + site(0) + site(3) + site(4) + grain(0) + grain(3) + grain(4), site(5) + grain(5) + self.buried_carbon, living(&|m| m.organic())]),
            pools("N", "As gas in the air that no cell can use, dissolved in the water, bound in remains lying or buried, and built into the living.",
                [self.air_nitrogen, site(NITROGEN) + grain(NITROGEN), site(REMAINS_N) + grain(REMAINS_N) + self.buried_nitrogen, living(&|m| m.nitrogen_total())]),
            pools("P", "As mineral dissolved out of rock, built into every living cell, and stored in bone. It has no gas. The rock itself is not counted.",
                [0., site(2) + grain(2), 0., living(&|m| m.mineral_total())]),
            pools("Ca", "Tracked together with phosphorus as the mineral of rock and bone.",
                [0., site(2) + grain(2), 0., living(&|m| m.stone())]),
            pools("Fe", "Dissolved in the sea until free oxygen rusts it out onto the bed. Rusting iron soaks up a young planet's first oxygen.",
                [0., site(IRON), self.rusted_iron, 0.]),
            pools("Na", "As salt, dissolved in water or left as a crust where water has dried away.", [0., salt[0], salt[1], 0.]),
            pools("Cl", "As salt, dissolved in water or left as a crust where water has dried away.", [0., salt[0], salt[1], 0.]),
            pools("O", "Free oxygen is counted as air. The rest is bound in water and carbon dioxide.",
                [gas[0] + body_gas, gas[1] + living(&|m| m.body.gas[1]), 0., 0.]),
        ];
        serde_json::json!({
            "carbon":{"air":self.air_carbon,"buried":self.buried_carbon,"dioxide":site(CO2)+grain(CO2)+living(&|m|m.body.solutes[CO2]),
                "dissolved":site(0)+site(3)+site(4)+grain(0)+grain(3)+grain(4),
                "living":living(&|m|m.organic()-m.body.solutes[CO2]),
                "remains":site(5)+grain(5),"total":organic,"error":errors[0]},
            "iron":{"dissolved":site(IRON),"rusted":self.rusted_iron},
            "salt":{"dissolved":salt[0],"crust":salt[1],"water":water[1]},
            "nitrogen":{"buried":self.buried_nitrogen,"air":self.air_nitrogen,"dissolved":site(NITROGEN)+grain(NITROGEN),
                "living":living(&|m|m.nitrogen_total()),
                "remains":site(REMAINS_N)+grain(REMAINS_N),"total":self.initial_nitrogen,"error":errors[3]},
            "oxygen":{"free":gas[0]+body_gas,"bound":gas[1]+living(&|m|m.body.gas[1])},
            "energy":{"sunlight":self.photon_energy,"vents":self.geothermal_energy,"heat":self.heat,
                "work":self.mechanical_input,"stored":stored,"start":self.initial_energy,"error":errors[2]},
            "levels":levels,"efficiency":efficiency,"elements":elements,
        })
    }
    /// The level a whole body feeds at: where most of its carbon sits.
    pub fn body_level(cells: &[&Cell]) -> usize {
        let mut carbon = [0.; LEVELS + 1];
        for c in cells {
            carbon[c.state.level()] += c.material.as_ref().map_or(0., |m| m.organic());
        }
        (0..=LEVELS).max_by(|a, b| carbon[*a].total_cmp(&carbon[*b])).unwrap_or(0)
    }
}
