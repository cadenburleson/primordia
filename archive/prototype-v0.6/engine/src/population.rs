//! A supplied founder population, selected only by the shared ecological rules.
//! Family accounting follows the reproducing parent; mates still contribute genes.
use crate::*;

const FAMILY_COUNT: usize = 6;
const ANCESTRY_LIMIT: usize = 256;
#[derive(Clone, Serialize, Deserialize)]
pub struct Family {
    pub id: u32,
    pub name: String,
    pub genes: [f32; GENE_COUNT],
    pub development: [f32; DEVELOPMENT],
    pub regulation: Regulation,
    pub anatomy: Anatomy,
    pub introduced: u32,
    pub births: u32,
    pub exchanges: u32,
    pub meals: u32,
    pub deaths: u32,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Ancestry {
    pub time: f32,
    pub child: u32,
    pub parent: u32,
    pub mate: u32,
    pub family: u32,
    pub generation: u32,
    pub genes: [f32; GENE_COUNT],
    pub anatomy: Option<Anatomy>,
}
fn default_sensing() -> bool {
    true
}
fn default_renewal() -> f32 {
    0.35
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Competition {
    #[serde(default = "default_sensing")]
    pub directional_sensing: bool,
    #[serde(default)]
    pub food_energy: f64,
    #[serde(default = "default_renewal")]
    pub renewal: f32,
    #[serde(default)]
    pub peak_generation: u32,
    pub families: Vec<Family>,
    pub ancestry: Vec<Ancestry>,
    pub peak_jointed: usize,
    pub largest_body: u32,
}
impl Competition {
    pub fn birth(&mut self, child: &Organism, time: f32) {
        self.peak_generation = self.peak_generation.max(child.generation);
        if let Some(f) = self.families.iter_mut().find(|f| f.id == child.family) {
            f.births += 1;
            f.exchanges += u32::from(child.mate != 0);
        }
        self.ancestry.push(Ancestry {
            time,
            child: child.id,
            parent: child.parent,
            mate: child.mate,
            family: child.family,
            generation: child.generation,
            genes: child.genes,
            anatomy: child.anatomy.clone(),
        });
        if self.ancestry.len() > ANCESTRY_LIMIT {
            self.ancestry.remove(0);
        }
    }
    pub fn valid(&self, next_id: u32) -> bool {
        let ids: BTreeSet<_> = self.families.iter().map(|f| f.id).collect();
        self.food_energy.is_finite()
            && self.food_energy >= 0.
            && self.renewal.is_finite()
            && (0.0..=1.).contains(&self.renewal)
            && self.families.len() == FAMILY_COUNT
            && ids.len() == FAMILY_COUNT
            && !ids.contains(&0)
            && self.ancestry.len() <= ANCESTRY_LIMIT
            && self.peak_jointed <= MAX_POPULATION
            && self.largest_body <= MAX_BODY_CELLS as u32
            && self.families.iter().all(|f| {
                f.regulation.valid()
                    && f.anatomy.valid()
                    && f.genes
                        .iter()
                        .chain(&f.development)
                        .all(|v| v.is_finite() && (0.0..=1.).contains(v))
            })
            && self.ancestry.iter().all(|a| {
                a.time.is_finite()
                    && a.time >= 0.
                    && a.child > 0
                    && a.child < next_id
                    && a.parent > 0
                    && a.parent < a.child
                    && a.mate < a.child
                    && ids.contains(&a.family)
                    && a.genes
                        .iter()
                        .all(|g| g.is_finite() && (0.0..=1.).contains(g))
                    && a.anatomy.as_ref().is_some_and(Anatomy::valid)
            })
    }
    pub fn metadata(&self, core: &Core) -> serde_json::Value {
        let families:Vec<_> = self.families.iter().map(|f| {
            let live:Vec<_> = core.organisms.iter().filter(|o|o.family==f.id).collect();
            let n=live.len().max(1) as f32;
            let mean = |gene:usize| live.iter().map(|o|o.genes[gene]).sum::<f32>()/n;
            let control_change=live.iter().filter_map(|o|o.anatomy.as_ref()).map(|a|
                a.network.iter().flatten().zip(f.anatomy.network.iter().flatten())
                .map(|(x,y)|(x-y).abs()).sum::<f32>()/16.).sum::<f32>()/n;
            serde_json::json!({"id":f.id,"name":f.name,"introduced":f.introduced,
                "living":live.len(),"cells":live.iter().map(|o|o.cells).sum::<u32>(),
                "births":f.births,"exchanges":f.exchanges,"meals":f.meals,"deaths":f.deaths,
                "generation":live.iter().map(|o|o.generation).max().unwrap_or(0),
                "jointed":live.iter().filter(|o|o.body.joints.iter().any(|j|j.enabled)).count(),
                "meanPredation":mean(PREDATION),"meanSensing":mean(SENSING),"meanMuscle":mean(MUSCLE),
                "controllerChange":control_change,"representative":live.iter().max_by_key(|o|(o.cells,o.generation)).map(|o|o.id)})
        }).collect();
        let poor = core
            .organisms
            .iter()
            .filter(|o| core.field[field_index(o.x, o.z)] < 0.5)
            .count();
        serde_json::json!({"directionalSensing":self.directional_sensing,"foodEnergy":self.food_energy,"families":families,"ancestry":self.ancestry.iter().rev().take(12).collect::<Vec<_>>(),
            "peakGeneration":self.peak_generation,"renewal":self.renewal,"peakJointed":self.peak_jointed,"largestBody":self.largest_body,
            "foodPoorFraction":poor as f32/core.organisms.len().max(1) as f32})
    }
}
impl Core {
    pub fn start_population(&mut self, count: u32) {
        let config = Config {
            seed: self.config.seed,
            name: "The competing ecosystem".into(),
            origin: "chemistry".into(),
            sunlight: 0.72,
            nutrients: 0.45,
            minerals: 0.65,
            solvent: 1.,
            temperature: 0.48,
            radiation: 0.08,
            mutation: 0.09,
            capacity: 1000,
            ..Config::default()
        };
        *self = Self::new(config).unwrap();
        self.config.origin = "seeded".into();
        self.events.clear();
        self.history.clear();
        let mut families = Vec::new();
        for index in 0..FAMILY_COUNT {
            let mut genes = [0.3; GENE_COUNT];
            for (k, v) in [
                (PHOTO, 0.75),
                (CHEMO, 0.75),
                (ADHESION, 0.98),
                (MUSCLE, 0.9),
                (SKELETON, 0.95),
                (SPECIALIZATION, 0.95),
                (SENSING, 0.85),
                (FERTILITY, 0.82),
                (SEX, 0.65),
                (THERMAL, 0.48),
                (MOBILITY, 0.6),
                (ARMOR, 0.25),
                (PREDATION, 0.35),
            ] {
                genes[k] = v;
            }
            let label = match index {
                0 => {
                    genes[PHOTO] = 0.95;
                    genes[CHEMO] = 0.45;
                    genes[PREDATION] = 0.08;
                    "Light-leaning"
                }
                1 => {
                    genes[PHOTO] = 0.45;
                    genes[CHEMO] = 0.95;
                    genes[PREDATION] = 0.08;
                    "Mineral-leaning"
                }
                2 => {
                    genes[PREDATION] = 0.9;
                    "Digestion-leaning"
                }
                3 => {
                    genes[ARMOR] = 0.8;
                    genes[PREDATION] = 0.55;
                    "Defense-leaning"
                }
                4 => {
                    genes[MUSCLE] = 0.65;
                    genes[ARMOR] = 0.1;
                    "Low-investment"
                }
                _ => {
                    genes[PREDATION] = 0.65;
                    genes[SENSING] = 0.95;
                    "Mixed foragers"
                }
            };
            genes[PIGMENT] = 0.1 + index as f32 * 0.15;
            let id = self.new_lineage(genes, 0, 0);
            let name = format!("{} · {}", (b'A' + index as u8) as char, label);
            self.lineages.get_mut(&id).unwrap().name = name.clone();
            let mut development = default_development();
            development[5] = 0.35 + index as f32 * 0.1;
            let mut anatomy = Anatomy::default();
            anatomy.traits[2] = 0.35 + index as f32 * 0.09;
            families.push(Family {
                id,
                name,
                genes,
                development,
                regulation: Regulation::default(),
                anatomy,
                introduced: 0,
                births: 0,
                exchanges: 0,
                meals: 0,
                deaths: 0,
            });
        }
        self.competition = Some(Competition {
            directional_sensing: true,
            food_energy: 0.,
            renewal: default_renewal(),
            peak_generation: 0,
            families,
            ancestry: Vec::new(),
            peak_jointed: 0,
            largest_body: 1,
        });
        self.seed_competitors(count.clamp(24, 96));
        self.event("world","Competing ecosystem: supplied single-cell families with varied metabolic, defensive, digestive and joint-control potentials. Food, survival and actual reproduction select descendants; no distance score, automatic rescue or timed upgrades.".into());
    }
    pub fn seed_competitors(&mut self, count: u32) {
        let Some(mut competition) = self.competition.take() else {
            return;
        };
        let count = (count as usize)
            .min(96)
            .min(self.config.capacity.saturating_sub(self.organisms.len()))
            .min(CELL_BUDGET.saturating_sub(self.organisms.iter().map(|o| o.cells as usize).sum()));
        for index in 0..count {
            let family = &mut competition.families[index % FAMILY_COUNT];
            let mut genes = family.genes;
            for g in &mut genes {
                *g = bounded(*g + (self.random() - 0.5) * 0.04);
            }
            let mut anatomy = family.anatomy.clone();
            for v in anatomy.values_mut() {
                *v = bounded(*v + (self.random() - 0.5) * 0.12);
            }
            let angle = self.random() * std::f32::consts::TAU;
            let radius = 3. + self.random().sqrt() * 8.;
            let cooldown = self.random() * 2.;
            self.organisms.push(Organism {
                id: self.next_id,
                parent: 0,
                mate: 0,
                family: family.id,
                offspring: 0,
                lineage: family.id,
                generation: 0,
                x: angle.cos() * radius,
                z: angle.sin() * radius,
                vx: 0.,
                vz: 0.,
                heading: angle,
                energy: 16.,
                age: 0.,
                cells: 1,
                genes,
                development: family.development,
                regulation: Some(family.regulation.clone()),
                anatomy: Some(anatomy),
                body: Body::default(),
                cooldown,
                alive: true,
                feeding_cooldown: 0.,
                meals: 0,
            });
            self.next_id += 1;
            family.introduced += 1;
        }
        self.competition = Some(competition);
        self.record_history();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn supplied_population_is_bounded_single_cells_and_extinction_has_no_rescue() {
        let mut core = Core::new(Config::default()).unwrap();
        core.start_population(48);
        assert!(core.lab.is_none() && !core.study);
        assert_eq!(core.organisms.len(), 48);
        assert!(core.organisms.iter().all(|o| o.cells == 1
            && o.body.nodes.is_empty()
            && o.anatomy.is_some()
            && o.parent == 0));
        assert!(core
            .competition
            .as_ref()
            .unwrap()
            .families
            .iter()
            .all(|f| f.introduced == 8));
        core.config.solvent = 0.;
        core.step(120);
        assert!(core.organisms.is_empty());
        assert_eq!(
            core.competition
                .as_ref()
                .unwrap()
                .families
                .iter()
                .map(|f| f.deaths)
                .sum::<u32>(),
            48
        );
        let json = serde_json::to_string(&core).unwrap();
        assert!(Core::load(&json).is_ok());
        core.config.solvent = 1.;
        core.seed(12);
        assert_eq!(core.organisms.len(), 12);
        assert!(core.organisms.iter().all(|o| o.family != 0 && o.cells == 1));
        core.start_population(10000);
        assert_eq!(core.organisms.len(), 96);
    }
    #[test]
    fn real_mating_inherits_controllers_and_records_parent_contributions() {
        let mut core = Core::new(Config::default()).unwrap();
        core.start_population(24);
        core.config.mutation = 0.;
        for o in &mut core.organisms {
            o.cooldown = 1000.;
        }
        let mut genes = core.organisms[0].genes;
        genes[ADHESION] = 0.;
        genes[SEX] = 0.65;
        for o in &mut core.organisms[..2] {
            o.genes = genes;
            o.energy = 100.;
            o.cooldown = 0.;
            o.x = 0.;
            o.z = 0.;
        }
        let first = core.organisms[0].clone();
        let second = core.organisms[1].clone();
        core.step(1);
        let child = core
            .organisms
            .iter()
            .find(|o| o.parent == first.id)
            .unwrap();
        assert_eq!(child.mate, second.id);
        assert_eq!(child.family, first.family);
        assert_eq!(child.offspring, 0);
        for ((value, a), b) in child
            .anatomy
            .as_ref()
            .unwrap()
            .values()
            .zip(first.anatomy.as_ref().unwrap().values())
            .zip(second.anatomy.as_ref().unwrap().values())
        {
            assert!(value == a || value == b);
        }
        assert_eq!(core.organisms[0].offspring, 1);
        assert_eq!(core.organisms[1].offspring, 1);
        let c = core.competition.as_ref().unwrap();
        assert_eq!(c.ancestry[0].child, child.id);
        assert_eq!(
            c.families.iter().map(|f| f.births).sum::<u32>(),
            core.births
        );
        assert_eq!(
            c.families.iter().map(|f| f.exchanges).sum::<u32>(),
            core.mating
        );
        let mut resumed = Core::load(&serde_json::to_string(&core).unwrap()).unwrap();
        core.step(13);
        resumed.step(13);
        assert_eq!(
            serde_json::to_string(&core).unwrap(),
            serde_json::to_string(&resumed).unwrap()
        );
    }
}
