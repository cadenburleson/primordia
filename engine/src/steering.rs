//! Creator steering: selection, copying-error rate and disturbances chosen by
//! the player. Nothing here builds a trait. Genes still arise through copying
//! errors at division and every material is still paid for by the cell that
//! makes it. Steering only changes who survives and how often copies vary.
use crate::{
    autonomous::{Cell, Evolution},
    body::V3,
    physical::{Habitat, BROADEN, ENGULF, FIN, PROPEL, SPINE, STEER, TOXIN, WALL},
    physiology::{BARRIER, CHANNEL, RECEPTOR},
    *,
};

/// What marks one line of a starter species out from another: the
/// adaptations a genome carries, one bit for each, in the order the app
/// names them. A line is every cell of a species with the same set.
pub(crate) const TRAITS: [&[usize]; 20] = [
    &[0],                                                         // feeds on light
    &[ENGULF],                                                    // a mouth
    &[9],                                                         // digests living tissue from outside
    &[15],                                                        // rots remains
    &[TOXIN],                                                     // venom
    &[SPINE],                                                     // spines
    &[3, 29],                                                     // a skeleton
    &[WALL],                                                      // a wall
    &[BARRIER],                                                   // a skin
    &[crate::development::PHOTIC],                                // eyes
    &[STEER],                                                     // steering
    &[PROPEL],                                                    // cilia
    &[4],                                                         // muscle
    &[FIN],                                                       // fins
    &[crate::physical::WING],                                     // wings
    &[crate::physical::HOLDFAST],                                 // a holdfast
    &[crate::physical::WARM],                                     // warm blood
    &[crate::development::GAMETIC, crate::development::FUSOGEN],  // sex
    &[crate::physical::FIX],                                      // fixes nitrogen
    &[crate::development::SYNAPSE],                               // junctions that learn
];
/// What each of `TRAITS` is called.
pub const TRAIT_NAMES: [&str; 20] = [
    "feeds on light", "a mouth", "digests living tissue", "rots remains", "venom", "spines", "a skeleton", "a wall",
    "a skin", "eyes", "steering", "cilia", "muscle", "fins", "wings", "a holdfast", "warm blood", "sex",
    "fixes nitrogen", "junctions that learn",
];
/// The adaptations a set of genes carries, as bits of `TRAITS`.
pub(crate) fn traits(genes: &[autonomous::Gene]) -> u32 {
    let mut mask = 0;
    for g in genes {
        if g.strength > 0.05 {
            for (bit, actions) in TRAITS.iter().enumerate() {
                if actions.contains(&g.action) {
                    mask |= 1 << bit;
                }
            }
        }
    }
    mask
}
/// One line of a species: the adaptations it carries, the line it arose
/// from (itself, for the line records began with) and when it first appeared.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub traits: u32,
    pub from: u32,
    pub since: f32,
}
/// The most lines kept for one species. Past this the ones with nothing
/// living in them and no living descendants are forgotten first.
pub(crate) const LINES: usize = 96;

pub const GOALS: [&str; 9] = [
    "bodies", "swimming", "movement", "support", "senses", "hunting", "defence", "limbs",
    "reserves",
];
const CENSUS_PERIOD: f32 = 2.;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Milestone {
    pub key: String,
    pub stage: u8,
    pub time: f32,
    pub generation: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Steering {
    /// Multiplies every copying-error probability. 1 is the natural rate.
    pub mutation_scale: f32,
    pub storm_scale: f32,
    pub storm_until: f32,
    /// Multiplies how fast dissolved fuel, waste, minerals and remains spread.
    pub mixing: f32,
    /// How readily touching cells pass a copy of a gene across. 0 is never.
    pub sharing: f32,
    /// Starter species the player asked to keep from dying out, and how many
    /// founders have been released since to do it.
    pub sustained: Vec<String>,
    pub restocked: u32,
    /// Brightness of the star: multiplies the light reaching photosynthesis.
    pub light: f32,
    /// Activity of the planet's interior: multiplies chemosynthesis at vents.
    pub vents: f32,
    /// How crowded the planet's neighbourhood is: multiplies how often
    /// bodies from space arrive. None at zero.
    pub bombardment: f32,
    /// Empty, or one of `GOALS`: the trait bodies are bred for.
    pub goal: String,
    /// The goal follows life's ladder by itself: whatever step has not been
    /// reached yet is the one rewarded.
    pub auto: bool,
    /// Rounds spent on the current step, and steps set aside for now because
    /// they made no headway.
    pub auto_rounds: u32,
    pub auto_tried: Vec<String>,
    pub goal_strength: f32,
    pub goal_period: f32,
    pub next_round: f32,
    pub rounds: u32,
    pub culled: u32,
    pub round_note: String,
    pub round_scored: u32,
    pub round_bodies: u32,
    /// Divisions counted at the last round; a round removes a share of the
    /// births since then, so a bred world keeps growing.
    pub round_births: u32,
    /// Chance per cycle that a cell outside the favoured lineage is removed.
    pub favour_pressure: f32,
    pub favour_culled: u32,
    pub winter_until: f32,
    pub winter_depth: f32,
    pub next_census: f32,
    /// Cells lost to an empty energy store, to wasting, and to injury
    /// (wounds, heat, disasters and culling).
    pub losses: [u32; 3],
    /// Times a swallowed cell was first kept alive inside its host.
    pub captures: u32,
    pub milestones: Vec<Milestone>,
    /// Cell positions at the last movement round, to measure real travel.
    pub anchors: BTreeMap<u32, V3>,
}
impl Default for Steering {
    fn default() -> Self {
        Self {
            mutation_scale: 1.,
            storm_scale: 1.,
            storm_until: 0.,
            mixing: 1.,
            sharing: 0.,
            sustained: Vec::new(),
            restocked: 0,
            light: 1.,
            vents: 1.,
            bombardment: 1.,
            goal: String::new(),
            auto: false,
            auto_rounds: 0,
            auto_tried: Vec::new(),
            goal_strength: 0.25,
            goal_period: 30.,
            next_round: 0.,
            rounds: 0,
            culled: 0,
            round_note: String::new(),
            round_scored: 0,
            round_bodies: 0,
            round_births: 0,
            favour_pressure: 0.,
            favour_culled: 0,
            winter_until: 0.,
            winter_depth: 0.,
            next_census: 0.,
            losses: [0; 3],
            captures: 0,
            milestones: Vec::new(),
            anchors: BTreeMap::new(),
        }
    }
}
impl Steering {
    /// The genes a breeding goal leans new copying errors toward. Breeding
    /// can only choose among what variation offers; this makes it offer more
    /// of what is being bred for. No goal, no lean.
    pub(crate) fn focus(&self) -> &'static [usize] {
        match self.goal.as_str() {
            "bodies" => &[2, 20, 21, 38, 39, 40, BROADEN],
            "swimming" => &[PROPEL, STEER],
            "movement" => &[2, 4, 5, 20, CHANNEL, FIN],
            "support" => &[3, 28],
            "limbs" => &[2, 3, 4, 5, 20, CHANNEL, FIN],
            "reserves" => &[crate::physiology::LIPID],
            "senses" => &[RECEPTOR, CHANNEL, 16, 24],
            "hunting" => &[9, ENGULF, TOXIN],
            "defence" => &[BARRIER, SPINE, TOXIN, WALL],
            _ => &[],
        }
    }
    pub fn valid(&self) -> bool {
        [
            self.mutation_scale,
            self.storm_scale,
            self.storm_until,
            self.goal_strength,
            self.goal_period,
            self.next_round,
            self.favour_pressure,
            self.winter_until,
            self.winter_depth,
            self.next_census,
        ]
        .iter()
        .all(|v| v.is_finite() && *v >= 0.)
            && self.mutation_scale <= 10.
            && (0.25..=8.).contains(&self.mixing)
            && (0.0..=8.).contains(&self.sharing)
            && self.sustained.len() <= crate::bestiary::SPECIES.len()
            && self
                .sustained
                .iter()
                .all(|s| crate::bestiary::SPECIES.iter().any(|known| known.0 == s))
            && (0.25..=8.).contains(&self.light)
            && (0.25..=8.).contains(&self.vents)
            && (0.0..=8.).contains(&self.bombardment)
            && self.storm_scale <= 16.
            && (self.goal_strength == 0. || (0.05..=0.6).contains(&self.goal_strength))
            && (5.0..=400.).contains(&self.goal_period)
            && self.favour_pressure <= 0.2
            && self.winter_depth <= 1.
            && (self.goal.is_empty() || GOALS.contains(&self.goal.as_str()))
            && self.milestones.len() <= RUNG_NAMES.len() * 3
            && self
                .milestones
                .iter()
                .all(|m| m.time.is_finite() && m.stage <= 3 && m.key.len() <= 24)
            && self.anchors.values().all(|p| p.len().is_finite())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Rung {
    pub key: &'static str,
    /// 0 absent · 1 gene in the pool · 2 working somewhere · 3 established.
    pub stage: u8,
    pub progress: f32,
    /// Fraction of living cells carrying an enabling gene.
    pub genes: f32,
    /// Fraction of living cells (or bodies) that have built it.
    pub built: f32,
    pub count: u32,
    pub first: [Option<f32>; 3],
}

const RUNG_NAMES: [(&str, &str); 27] = [
    ("cells", "Living cells"),
    ("colonies", "Colonies"),
    ("bodies", "Multicellular bodies"),
    ("tissues", "Specialised tissues"),
    ("fates", "Cell fates"),
    ("castes", "Workers and breeders"),
    ("shapes", "Shaped bodies"),
    ("senses", "Senses"),
    ("swimming", "Swimming"),
    ("steering", "Guided swimming"),
    ("muscle", "Muscle"),
    ("fins", "Fins and feet"),
    ("skeleton", "Skeleton"),
    ("hunters", "Hunters"),
    ("carnivores", "Meat-eaters"),
    ("fixing", "Nitrogen fixing"),
    ("breathing", "Oxygen breathing"),
    ("complex", "Complex cells"),
    ("animals", "Animals"),
    ("guts", "Guts"),
    ("defences", "Defences"),
    ("walls", "Walled life"),
    ("nerves", "Nerve nets"),
    ("movement", "Powered movement"),
    ("reflexes", "Reflex arcs"),
    ("organs", "Organs"),
    ("sex", "Sexual reproduction"),
];

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Command {
    Mutation {
        scale: f32,
    },
    Mixing {
        scale: f32,
    },
    Sharing {
        scale: f32,
    },
    Seed {
        species: String,
        count: u32,
        /// Keep this species from dying out by releasing a few more founders
        /// whenever it falls below a handful.
        #[serde(default)]
        sustain: bool,
    },
    Sustain {
        on: bool,
    },
    /// Watch over one species the player has painted in by hand.
    Keep {
        species: String,
    },
    Light {
        scale: f32,
    },
    Vents {
        scale: f32,
    },
    Bombardment {
        scale: f32,
    },
    Goal {
        goal: String,
        strength: f32,
        period: f32,
    },
    Favour {
        body: u32,
        pressure: f32,
    },
    Unfavour {},
    Storm {
        scale: f32,
        duration: f32,
    },
    Winter {
        depth: f32,
        duration: f32,
    },
    Meteor {
        x: f32,
        z: f32,
        radius: f32,
    },
    Dying {
        fraction: f32,
    },
    Nourish {
        amount: f64,
    },
    /// Set how much of the planet's oxygen is free: 0 for a young planet's
    /// air, 1 for air that everything can breathe.
    Oxygen {
        level: f64,
    },
    /// Move nitrogen between the air and the water: a share of the air's
    /// fixed into the seas and soils as a great lightning storm would
    /// (positive), or a share of what is dissolved driven back to the air.
    Nitrogen {
        change: f64,
    },
    /// Rain new water over the whole planet (a depth, positive) or draw a
    /// share of all standing water off into space (a fraction, negative).
    Water {
        change: f64,
    },
}

/// Rounds an automatic goal stays on one step before trying the next.
const AUTO_PATIENCE: u32 = 12;
/// The trait to reward to reach a step of the ladder, where there is one.
fn rung_goal(key: &str) -> Option<&'static str> {
    Some(match key {
        "colonies" | "bodies" | "tissues" | "fates" | "castes" | "shapes" | "organs" => "bodies",
        "senses" | "nerves" | "reflexes" => "senses",
        "swimming" | "steering" => "swimming",
        "muscle" | "fins" | "movement" => "movement",
        "skeleton" => "support",
        "hunters" => "hunting",
        "defences" | "walls" => "defence",
        _ => return None,
    })
}
fn carries(c: &Cell, actions: &[usize]) -> bool {
    let has = |g: &autonomous::Genome| {
        g.genes
            .iter()
            .any(|g| g.strength > 0.05 && actions.contains(&g.action))
    };
    has(&c.genome) || c.lifecycle.homolog.as_ref().is_some_and(has)
}
fn built(c: &Cell, actions: &[usize]) -> bool {
    actions.iter().any(|&k| c.activity(k) > 0.25)
}
/// A gene being expressed before its material has built up. Merely carrying a
/// silent gene earns almost nothing, or breeding would fill the pool with
/// genes that are never switched on.
fn latent(c: &Cell, actions: &[usize]) -> f32 {
    actions
        .iter()
        .map(|&k| c.expression[k])
        .fold(0f32, f32::max)
        + if carries(c, actions) { 0.03 } else { 0. }
}
fn mineral_fraction(c: &Cell) -> f32 {
    c.material
        .as_ref()
        .map_or(0., |m| (m.mineral / m.mass().max(1e-12)) as f32)
}

impl Habitat {
    /// Copying-error multiplier in force at `time`.
    /// Diffusion rates in force. Chemical signals keep their natural range.
    pub(crate) fn diffusivity(&self, natural: [f64; crate::physical::SOLUTES]) -> [f64; crate::physical::SOLUTES] {
        let mut rates = natural;
        for k in [0, 1, 2, 5] {
            rates[k] *= self.steering.mixing as f64;
        }
        rates
    }
    /// Life out of the water, in the air and keeping itself warm: what the
    /// later ages of a planet are counted by.
    pub(crate) fn ashore_view(&self, e: &Evolution) -> serde_json::Value {
        use crate::physical::{WARM, WING};
        // Living there, not washed up there: a plant holds on to the ground
        // it stands on, and an animal has been alive a while and is not
        // dying of it.
        // A body is ashore when something of it out of the water holds or
        // stands on the ground, and every cell of it out of the water is
        // ashore with it: a tree is not only its root.
        let out = |c: &Cell| c.state.under < 0.3;
        let (mut rooted, mut standing, mut bodied, mut walking) =
            (BTreeSet::new(), BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
        let mut largest = 0;
        for g in e.groups() {
            let held = g.iter().any(|&i| e.cells[i].state.anchored && out(&e.cells[i]));
            let afoot = g.iter().any(|&i| e.cells[i].state.ground && out(&e.cells[i]));
            // An animal is a body: several cells, a mouth or digestive
            // enzymes among them, most of them neither feeding on light nor
            // walled as a fungus is. A microbe that has washed up, or one
            // that happens to carry a gene for warmth or for wings, is not
            // a land animal, a warm-blooded one or a flier.
            let animal = g.len() >= 4
                && g.iter().any(|&i| e.cells[i].activity(ENGULF) > 0.05 || e.cells[i].activity(9) > 0.05)
                && g.iter().filter(|&&i| built(&e.cells[i], &[0])).count() * 2 < g.len()
                && g.iter().filter(|&&i| built(&e.cells[i], &[WALL])).count() * 2 < g.len();
            if held {
                rooted.extend(g.iter().map(|&i| e.cells[i].id));
            }
            if held || afoot {
                standing.extend(g.iter().map(|&i| e.cells[i].id));
            }
            if animal {
                bodied.extend(g.iter().map(|&i| e.cells[i].id));
                // An animal of the land stands on it. One fixed to the
                // bed of the shallows, as a coral is, is left by the tide
                // and has not come ashore.
                if afoot && !held {
                    walking.extend(g.iter().map(|&i| e.cells[i].id));
                    largest = largest.max(g.len());
                }
            }
        }
        let ashore = |c: &Cell| out(c) && standing.contains(&c.id);
        // A plant of the land has a wall to stand up in. A mat of bacteria
        // on a wet shore has none, and was there long before plants were.
        let plant = |c: &Cell| built(c, &[0]) && built(c, &[WALL]) && rooted.contains(&c.id);
        let animal = |c: &Cell| walking.contains(&c.id) && !built(c, &[0]) && c.age > 20. && c.state.injury < 0.5;
        let count = |f: &dyn Fn(&Cell) -> bool| e.cells.iter().filter(|c| f(c)).count();
        // Away from the water's edge: on ground with no open water on it,
        // only what its soil holds. And how many sites from the nearest
        // open water each site is, up to three.
        let open: Vec<bool> = self.planet.as_ref().map_or(Vec::new(), |p| {
            (0..self.domain.sites())
                .map(|site| {
                    let area = if self.domain.spherical() { self.domain.area(site) } else { 1. };
                    p.water[site] - crate::planet::soil(self.solutes[site][5], area, p.water[site]) >= 0.001 * area
                })
                .collect()
        });
        let mut steps: Vec<u8> = open.iter().map(|&wet| if wet { 0 } else { 3 }).collect();
        if !steps.is_empty() {
            let edges = self.domain.edges();
            for _ in 0..2 {
                let before = steps.clone();
                for &(i, j) in &edges {
                    steps[i] = steps[i].min(before[j] + 1);
                    steps[j] = steps[j].min(before[i] + 1);
                }
            }
        }
        let away = |c: &Cell| steps.get(self.domain.locate(c.p)).copied().unwrap_or(0);
        let warm: BTreeSet<u32> =
            e.cells.iter().filter(|c| bodied.contains(&c.id) && built(c, &[WARM])).map(|c| c.id).collect();
        serde_json::json!({
            "plants": count(&|c| ashore(c) && plant(c)),
            "inland": count(&|c| ashore(c) && plant(c) && away(c) >= 1),
            "interior": count(&|c| ashore(c) && plant(c) && away(c) >= 3),
            "animals": count(&|c| out(c) && animal(c)),
            "dry": count(&|c| out(c) && animal(c) && built(c, &[BARRIER])),
            "woody": count(&|c| ashore(c) && plant(c) && c.state.order >= 2),
            "flowering": count(&|c| ashore(c) && plant(c) && built(c, &[18, 19])),
            "grass": count(&|c| ashore(c) && plant(c) && c.material.as_ref().is_some_and(|m| m.shell() > 0.1)),
            "aloft": count(&|c| bodied.contains(&c.id) && !c.state.ground && c.state.under < 0.05 && built(c, &[WING])),
            "warm": warm.len(),
            "warmAloft": count(&|c| !c.state.ground && c.state.under < 0.05 && built(c, &[WING, WARM]) && warm.contains(&c.id)),
            "largest": largest,
            "minds": self.development.synapses.iter().filter(|s| warm.contains(&s.from) && warm.contains(&s.to)).count(),
            "greenhouse": self.planet.as_ref().map_or(0., |p| p.greenhouse),
        })
    }
    /// The sky as the player is shown it: what arrives from space, and
    /// what each layer of the air is stopping now.
    pub(crate) fn sky(&self) -> serde_json::Value {
        serde_json::json!({
            "bombardment": self.steering.bombardment,
            "impacts": self.impacts,
            "airbursts": self.airbursts,
            "airColumn": self.air_column(),
            "ozone": self.ozone(),
            "hardRays": self.hard_rays(),
            "burnedUp": self.burned_up(),
            "rockNitrogen": self.rock_nitrogen,
            "dust": self.planet.as_ref().map_or(0., |p| p.dust),
        })
    }
    pub(crate) fn mutation_factor(&self, time: f32) -> f32 {
        let s = &self.steering;
        let storm = if time < s.storm_until {
            s.storm_scale
        } else {
            1.
        };
        (s.mutation_scale * storm).clamp(0., 16.)
    }

    /// Runs after division and before dead cells are recycled, so removed
    /// cells return their material to the world on the same tick.
    pub(crate) fn steer(&mut self, e: &mut Evolution, core: &mut Core) {
        let now = core.time;
        if self.steering.winter_until > 0. && now >= self.steering.winter_until {
            core.config.temperature =
                (core.config.temperature + self.steering.winter_depth).clamp(0., 1.);
            self.steering.winter_until = 0.;
            self.steering.winter_depth = 0.;
            core.event("world", "The ice age has ended. Warmth returns.".into());
        }
        if self.steering.favour_pressure > 0. {
            if e.cells.iter().any(|c| c.state.favoured) {
                let chance = self.steering.favour_pressure * DT;
                for c in &mut e.cells {
                    if !c.state.favoured && core.random() < chance {
                        c.state.injury = 1.;
                        c.state.culled = true;
                        self.steering.favour_culled += 1;
                    }
                }
            } else {
                self.steering.favour_pressure = 0.;
                core.event(
                    "world",
                    "The favoured lineage died out. Its rivals are no longer culled.".into(),
                );
            }
        }
        if !self.steering.goal.is_empty() && now >= self.steering.next_round {
            self.selection_round(e, core);
            self.steering.next_round = now + self.steering.goal_period;
        }
        if now >= self.steering.next_census {
            self.census(e, core);
            self.prune_lines(e);
            self.steering.next_census = now + CENSUS_PERIOD;
        }
    }

    /// A daughter cell has been born carrying `now` where its parent carried
    /// `was`. The first time a set of adaptations is seen in a species it is
    /// a new line, and what it arose from and when is kept.
    pub(crate) fn note_line(&mut self, species: &str, was: u32, now: u32, time: f32) {
        if species.is_empty() {
            return;
        }
        if !self.lines.contains_key(species) {
            self.lines.insert(species.to_string(), Vec::new());
        }
        let lines = self.lines.get_mut(species).unwrap();
        for (traits, from) in [(was, was), (now, was)] {
            if lines.len() < LINES * 2 && !lines.iter().any(|l| l.traits == traits) {
                lines.push(Line { traits, from, since: time });
            }
        }
    }

    /// Living cells of each line of each starter species.
    fn living_lines(e: &Evolution) -> BTreeMap<&str, BTreeMap<u32, usize>> {
        let mut living: BTreeMap<&str, BTreeMap<u32, usize>> = BTreeMap::new();
        for c in &e.cells {
            if let Some(name) = c.collection_sources.first() {
                *living.entry(name.as_str()).or_default().entry(traits(&c.genome.genes)).or_default() += 1;
            }
        }
        living
    }

    /// Forgets lines that have died out and left nothing living behind
    /// them, once a species has more on record than it may keep.
    fn prune_lines(&mut self, e: &Evolution) {
        let living = Self::living_lines(e);
        self.lines.retain(|name, _| living.contains_key(name.as_str()));
        for (name, lines) in self.lines.iter_mut() {
            if lines.len() <= LINES {
                continue;
            }
            let alive = &living[name.as_str()];
            // Keep whatever is alive and every line on the way back from it.
            let mut keep: BTreeSet<u32> = alive.keys().copied().collect();
            loop {
                let more: Vec<u32> =
                    lines.iter().filter(|l| keep.contains(&l.traits) && !keep.contains(&l.from)).map(|l| l.from).collect();
                if more.is_empty() {
                    break;
                }
                keep.extend(more);
            }
            lines.retain(|l| keep.contains(&l.traits));
            lines.truncate(LINES * 2);
        }
    }

    /// The family tree of each starter species, as the app draws it: its
    /// living lines with a hundredth of the species or more in them, what
    /// each arose from, and when. A line that was there before records
    /// began is hung from the species as it was designed, with no date.
    pub(crate) fn lines_view(&self, e: &Evolution) -> serde_json::Value {
        let living = Self::living_lines(e);
        let mut trees = Vec::new();
        for (name, alive) in &living {
            let total: usize = alive.values().sum();
            let designed = crate::bestiary::designed_traits(name);
            let known = self.lines.get(*name);
            let line = |t: u32| known.and_then(|k| k.iter().find(|l| l.traits == t));
            let floor = (total / 100).max(6);
            let mut shown: Vec<u32> = alive.iter().filter(|a| *a.1 >= floor).map(|a| *a.0).collect();
            shown.sort_by_key(|t| std::cmp::Reverse(alive[t]));
            shown.truncate(12);
            // Where each came from: what was recorded, or the design.
            let parent = |t: u32| match line(t) {
                Some(l) if l.from != t => Some(l.from),
                _ => designed.filter(|d| *d != t),
            };
            let mut all = shown.clone();
            let mut k = 0;
            while k < all.len() && all.len() < 40 {
                if let Some(p) = parent(all[k]) {
                    if !all.contains(&p) {
                        all.push(p);
                    }
                }
                k += 1;
            }
            if all.len() < 2 {
                continue;
            }
            let branches: Vec<serde_json::Value> = all
                .iter()
                .map(|&t| {
                    serde_json::json!({
                        "traits": t,
                        "from": parent(t),
                        "since": line(t).filter(|l| l.from != t).map(|l| l.since),
                        "cells": alive.get(&t).copied().unwrap_or(0),
                    })
                })
                .collect();
            trees.push((total, serde_json::json!({"species": name, "cells": total, "lines": alive.len(), "branches": branches})));
        }
        trees.sort_by_key(|t| std::cmp::Reverse(t.0));
        serde_json::Value::Array(trees.into_iter().take(16).map(|t| t.1).collect())
    }

    /// (measured performance, enabling material). Performance dominates; the
    /// material term gives selection something to act on before a new tissue
    /// does useful work.
    fn goal_score(&self, e: &Evolution, group: &[usize], goal: &str) -> (f32, f32) {
        let n = group.len() as f32;
        let mean =
            |f: &dyn Fn(&Cell) -> f32| group.iter().map(|&i| f(&e.cells[i])).sum::<f32>() / n;
        // Muscle and skeleton are properties of a body: a lone cell has nothing
        // to pull on or hold up, so its material earns no credit.
        let embodied = ((n - 1.) / 3.).min(1.);
        match goal {
            "bodies" => {
                let roles = group
                    .iter()
                    .map(|&i| e.cells[i].material_role())
                    .collect::<BTreeSet<_>>()
                    .len() as f32;
                // Cells that have committed to different fates count for more
                // than cells that merely hold different materials.
                let fates = group
                    .iter()
                    .map(|&i| e.cells[i].state.fate)
                    .collect::<BTreeSet<_>>()
                    .len() as f32;
                (
                    n.min(64.) * (1. + 0.5 * (roles - 1.) + (fates - 1.)) - 1.,
                    mean(&|c| c.activity(2)),
                )
            }
            "swimming" | "movement" => {
                let rate = mean(&|c| c.state.motor_work as f32 / (c.age + 1.));
                let mut travel = V3::default();
                let mut tracked = 0.;
                for &i in group {
                    if let Some(&from) = self.steering.anchors.get(&e.cells[i].id) {
                        travel = travel.add(e.cells[i].p.sub(from));
                        tracked += 1.;
                    }
                }
                let travel = if tracked > 0. {
                    travel.mul(1. / tracked).len()
                } else {
                    0.
                };
                if goal == "swimming" {
                    // A driven motor that actually got somewhere. Lone cells count.
                    (
                        mean(&|c| c.activity(PROPEL) * c.expression[PROPEL])
                            * (0.2 + travel.min(12.)),
                        mean(&|c| {
                            c.activity(PROPEL)
                                + 0.5 * c.activity(STEER)
                                + latent(c, &[PROPEL, STEER])
                        }),
                    )
                } else {
                    // A body that rowed somewhere: muscle, fins to push with,
                    // and ground covered. Cilia earn nothing here. Until a
                    // body manages it, the parts are bred for: muscle, the
                    // excitable channels that make it beat, and fins.
                    (
                        embodied
                            * mean(&|c| c.activity(4))
                            * mean(&|c| c.activity(FIN)).sqrt()
                            * (1. - mean(&|c| c.activity(PROPEL)))
                            * (rate.min(1.) + travel.min(12.)),
                        // Muscle needs a body to pull on, so being a body
                        // is itself the first part worth having.
                        embodied
                            * (0.3
                                + mean(&|c| {
                                    c.activity(4)
                                        + 0.5 * c.activity(CHANNEL)
                                        + 0.5 * c.activity(FIN)
                                        + latent(c, &[4])
                                        + 0.5 * latent(c, &[CHANNEL])
                                        + 0.5 * latent(c, &[FIN])
                                })),
                    )
                }
            }
            "support" => (
                embodied * mean(&mineral_fraction),
                embodied * mean(&|c| c.activity(3) + latent(c, &[3])),
            ),
            // A limb: bone meeting at a joint that muscle swings. Scored by
            // the work the body's joints have done, and by ground covered
            // once it has feet; until then, by the parts.
            "limbs" => {
                let ids: BTreeSet<u32> = group.iter().map(|&i| e.cells[i].id).collect();
                let swung = self
                    .joints
                    .iter()
                    .filter(|j| ids.contains(&j.b))
                    .map(|j| j.work)
                    .sum::<f64>() as f32;
                let mut travel = V3::default();
                let mut tracked = 0.;
                for &i in group {
                    if let Some(&from) = self.steering.anchors.get(&e.cells[i].id) {
                        travel = travel.add(e.cells[i].p.sub(from));
                        tracked += 1.;
                    }
                }
                let travel = if tracked > 0. { travel.mul(1. / tracked).len() } else { 0. };
                (
                    embodied * swung.min(4.) * (0.5 + mean(&|c| c.activity(FIN)).sqrt() * travel.min(12.)),
                    embodied
                        * (0.3
                            + mean(&|c| {
                                (8. * mineral_fraction(c)).min(1.)
                                    + c.activity(4)
                                    + 0.5 * c.activity(CHANNEL)
                                    + 0.5 * c.activity(FIN)
                                    + latent(c, &[3])
                                    + latent(c, &[4])
                                    + 0.5 * latent(c, &[CHANNEL])
                                    + 0.5 * latent(c, &[FIN])
                            })),
                )
            }
            // Fat laid down against lean times. Lone cells count.
            "reserves" => (
                mean(&|c| {
                    c.material
                        .as_ref()
                        .map_or(0., |m| (m.lipid / m.mass().max(1e-12)) as f32)
                }),
                mean(&|c| c.activity(crate::physiology::LIPID) + latent(c, &[crate::physiology::LIPID])),
            ),
            "senses" => (
                mean(&|c| {
                    c.activity(RECEPTOR) + c.activity(CHANNEL) + c.activity(16) + c.activity(17)
                }),
                mean(&|c| {
                    c.expression[RECEPTOR]
                        + c.expression[CHANNEL]
                        + c.expression[16]
                        + c.expression[17]
                }),
            ),
            "hunting" => (
                mean(&|c| c.state.flux.prey as f32 / (c.age + 1.)),
                mean(&|c| {
                    c.activity(9)
                        + c.activity(ENGULF)
                        + 0.5 * c.activity(TOXIN)
                        + latent(c, &[9, ENGULF, TOXIN])
                }),
            ),
            "defence" => (
                mean(&|c| {
                    (c.state.flux.defense_blocked + c.state.flux.wounded) as f32 / (c.age + 1.)
                }),
                mean(&|c| {
                    c.activity(BARRIER)
                        + c.activity(SPINE)
                        + c.activity(TOXIN)
                        + c.activity(WALL)
                        + 4. * mineral_fraction(c)
                        + latent(c, &[BARRIER, SPINE, TOXIN, WALL])
                }),
            ),
            _ => (0., 0.),
        }
    }

    /// Point the goal at the first step of life's ladder that is not yet
    /// established. A step that makes no headway in `AUTO_PATIENCE` rounds is
    /// set aside and the next one tried, so one hard step cannot hold up the rest.
    fn follow_ladder(&mut self, e: &Evolution) {
        let open: Vec<(&str, &str)> = self
            .ladder(e)
            .iter()
            .filter(|r| r.stage < 3)
            .filter_map(|r| rung_goal(r.key).map(|g| (r.key, g)))
            .collect();
        let s = &mut self.steering;
        s.auto_tried.retain(|k| open.iter().any(|o| o.0 == k));
        if open.iter().all(|o| s.auto_tried.iter().any(|k| k == o.0)) {
            s.auto_tried.clear();
        }
        // Everything on the ladder is established: keep rewarding richer bodies.
        let Some(&(key, goal)) = open.iter().find(|o| !s.auto_tried.iter().any(|k| k == o.0))
        else {
            s.goal = "bodies".into();
            return;
        };
        if s.goal == goal {
            s.auto_rounds += 1;
            if s.auto_rounds >= AUTO_PATIENCE && open.len() > 1 {
                s.auto_tried.push(key.into());
                s.auto_rounds = 0;
            }
        } else {
            s.goal = goal.into();
            s.auto_rounds = 0;
            s.anchors.clear();
            s.round_note.clear();
        }
    }

    /// One round of artificial selection: the lowest-scoring bodies are
    /// removed and recycled. Survivors inherit the freed room and material.
    fn selection_round(&mut self, e: &mut Evolution, core: &mut Core) {
        if self.steering.auto {
            self.follow_ladder(e);
        }
        let goal = self.steering.goal.clone();
        let groups = e.groups();
        let total = e.cells.len();
        let raw: Vec<_> = groups
            .iter()
            .map(|g| self.goal_score(e, g, &goal))
            .collect();
        self.steering.anchors = if goal == "movement" || goal == "swimming" || goal == "limbs" {
            e.cells.iter().map(|c| (c.id, c.p)).collect()
        } else {
            BTreeMap::new()
        };
        let top = |f: fn(&(f32, f32)) -> f32| raw.iter().map(f).fold(0f32, f32::max);
        let (best_measure, best_material) = (top(|r| r.0), top(|r| r.1));
        // A hunter needs prey. Once any body is really eating, hunters are judged
        // on what they catch alone; rewarding the weapon as well would arm the
        // whole world and leave nothing to hunt.
        let material_weight = if goal == "hunting" && best_measure > 0. {
            0.
        } else {
            0.25
        };
        let scores: Vec<f32> = raw
            .iter()
            .map(|r| {
                (if best_measure > 0. {
                    r.0 / best_measure
                } else {
                    0.
                }) + material_weight
                    * (if best_material > 0. {
                        r.1 / best_material
                    } else {
                        0.
                    })
            })
            .collect();
        // Breeding hunters never touches their prey: only bodies that carry a
        // weapon compete, and the ones that catch least are removed. Culling
        // the unarmed would starve the hunters it was meant to favour.
        let exempt: Vec<bool> = groups
            .iter()
            .map(|g| {
                goal == "hunting" && !g.iter().any(|&i| built(&e.cells[i], &[9, ENGULF, TOXIN]))
            })
            .collect();
        let judged = || {
            scores
                .iter()
                .zip(&exempt)
                .filter(|(_, x)| !**x)
                .map(|(v, _)| *v)
        };
        let best = judged().fold(0f32, f32::max);
        let worst = judged().fold(f32::INFINITY, f32::min);
        let contenders: usize = groups
            .iter()
            .zip(&exempt)
            .filter(|(_, x)| !**x)
            .map(|(g, _)| g.len())
            .sum();
        let s = &mut self.steering;
        s.rounds += 1;
        s.round_bodies = groups.len() as u32;
        s.round_scored = scores.iter().filter(|v| **v > 0.).count() as u32;
        // Remove a share of the births since the last round, never more than
        // `goal_strength` of everything alive. Selection then rides on the
        // world's own turnover instead of shrinking it.
        let births = e.divisions.saturating_sub(s.round_births);
        s.round_births = e.divisions;
        let quota =
            (births as f32 * 1.2 * s.goal_strength).min(contenders as f32 * s.goal_strength);
        if s.goal_strength == 0. {
            s.round_note = "leaning".into();
            return;
        }
        if total < 48 {
            s.round_note = "too-few".into();
            return;
        }
        if best <= 0. || best - worst < 1e-6 {
            s.round_note = "no-variation".into();
            return;
        }
        if quota < 1. {
            s.round_note = "recovering".into();
            return;
        }
        let mut order: Vec<(f32, usize)> = scores
            .iter()
            .enumerate()
            .filter(|(i, _)| !exempt[*i])
            .map(|(i, v)| (v + core.random() * 1e-5, i))
            .collect();
        order.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut removed = 0usize;
        for (_, i) in order {
            // Never remove the leaders, and never overshoot the quota.
            if scores[i] >= best * 0.999 || removed as f32 >= quota {
                break;
            }
            // A body larger than what is left of the quota loses that many
            // cells rather than escaping: size alone is no protection.
            let take = groups[i]
                .len()
                .min((quota - removed as f32).ceil() as usize);
            for &c in groups[i].iter().take(take) {
                e.cells[c].state.injury = 1.;
                e.cells[c].state.culled = true;
            }
            removed += take;
        }
        s.culled += removed as u32;
        s.round_note = if removed > 0 {
            "selecting"
        } else {
            "recovering"
        }
        .into();
    }

    /// How living cells feed, counted by body: light, dissolved fuel, remains
    /// and vents for bodies that do not live on prey; herbivore, omnivore and
    /// carnivore for those that do. An eater takes a quarter or more of its
    /// fuel from other life; it is a herbivore if that prey mostly feeds
    /// itself, a carnivore if the prey are mostly animals (cells of bodies
    /// that eat, or of many-celled bodies that do not feed themselves), an
    /// omnivore between.
    /// Also returns the number of eater bodies, and of those that eat
    /// animals: omnivores and carnivores together. Above the plant-eaters a
    /// real food web is a tangle of things that eat at more than one level,
    /// and its predators are not marked off from the rest by eating nothing
    /// else.
    fn diets(&self, e: &Evolution, groups: &[Vec<usize>]) -> ([usize; 7], usize, usize) {
        let mut cells = [0usize; 7];
        let (mut eaters, mut carnivores) = (0, 0);
        for g in groups {
            let mut flux = crate::observations::Flux::default();
            for &i in g {
                flux.add(&e.cells[i].state.flux);
            }
            if flux.intake() <= 0. {
                continue;
            }
            let kind = if flux.eater() {
                eaters += 1;
                let animal = flux.prey_animal / flux.prey;
                if animal >= 0.7 {
                    carnivores += 1;
                    6
                } else if animal > 0.3 {
                    carnivores += 1;
                    5
                } else {
                    4
                }
            } else {
                let own = [
                    flux.photosynthesis,
                    flux.fuel + flux.absorbed,
                    flux.detritus,
                    flux.chemosynthesis,
                ];
                (0..4).fold(0, |best, k| if own[k] > own[best] { k } else { best })
            };
            cells[kind] += g.len();
        }
        (cells, eaters, carnivores)
    }

    /// Cells living as each broad kind of life: free microbes, plant-like
    /// (walled, feeding itself from light or vents), fungus-like (walled,
    /// absorbing or digesting) and animal-like (bare, with a mouth, motor or fin).
    pub(crate) fn kinds(&self, e: &Evolution, groups: &[Vec<usize>]) -> [usize; 4] {
        let mut cells = [0usize; 4];
        for g in groups {
            if let Some(kind) = Self::kind_of(e, g) {
                cells[kind] += g.len();
            }
        }
        cells
    }

    /// Which of those four one body is, or none for a walled body that has
    /// not fed yet.
    fn kind_of(e: &Evolution, g: &[usize]) -> Option<usize> {
        let mut flux = crate::observations::Flux::default();
        for &i in g {
            flux.add(&e.cells[i].state.flux);
        }
        let walled = g.iter().filter(|&&i| built(&e.cells[i], &[WALL])).count() * 2 >= g.len();
        let armed = g.iter().any(|&i| built(&e.cells[i], &[9, ENGULF, PROPEL, FIN]));
        // A body that makes its own food lives as a plant does, wall or no
        // wall. One that does not is a fungus if it is walled, an animal
        // if it has a mouth, enzymes, a motor or fins, and a microbe otherwise.
        Some(if flux.intake() <= 0. {
            if walled {
                return None;
            }
            if armed { 3 } else { 0 }
        } else if (flux.photosynthesis + flux.chemosynthesis) * 2. >= flux.intake() {
            1
        } else if walled {
            2
        } else if armed {
            3
        } else {
            0
        })
    }

    /// Living cells descended from each starter species the player released,
    /// in the bestiary's order. A lineage that has mated outside its own kind
    /// is counted under the first of its names.
    pub(crate) fn species(&self, e: &Evolution) -> Vec<(String, usize)> {
        crate::bestiary::SPECIES
            .iter()
            .map(|&(_, name)| {
                let living = e
                    .cells
                    .iter()
                    .filter(|c| c.collection_sources.first().is_some_and(|s| s == name))
                    .count();
                (name.to_string(), living)
            })
            .filter(|s| s.1 > 0)
            .collect()
    }

    /// Signalling contacts that run from a sensing cell to a moving one.
    fn reflex_arcs(&self, e: &Evolution) -> usize {
        if self.development.synapses.is_empty() {
            return 0;
        }
        let ids = e.index();
        self.development
            .synapses
            .iter()
            .filter(|s| {
                let (Some(&from), Some(&to)) = (ids.get(&s.from), ids.get(&s.to)) else {
                    return false;
                };
                built(&e.cells[from], &[RECEPTOR, 16, 24])
                    && built(&e.cells[to], &[4, PROPEL, STEER])
            })
            .count()
    }

    /// Measured state of each step from single cells to complex bodies.
    pub(crate) fn ladder(&self, e: &Evolution) -> Vec<Rung> {
        let n = e.cells.len();
        let fraction = |count: usize| if n > 0 { count as f32 / n as f32 } else { 0. };
        let genes =
            |actions: &[usize]| fraction(e.cells.iter().filter(|c| carries(c, actions)).count());
        let makers =
            |actions: &[usize]| fraction(e.cells.iter().filter(|c| built(c, actions)).count());
        let groups = e.groups();
        let multi: Vec<&Vec<usize>> = groups.iter().filter(|g| g.len() >= 3).collect();
        let largest = groups.iter().map(Vec::len).max().unwrap_or(0);
        let attached = groups
            .iter()
            .filter(|g| g.len() >= 2)
            .map(|g| g.len())
            .sum::<usize>();
        let roles = |g: &[usize]| {
            g.iter()
                .map(|&i| e.cells[i].material_role())
                .collect::<BTreeSet<_>>()
                .len()
        };
        // A tissue is a group of like cells: two types, each at least two cells.
        let tissues = |g: &[usize]| {
            let mut counts = BTreeMap::<u8, usize>::new();
            for &i in g {
                *counts.entry(e.cells[i].material_role()).or_default() += 1;
            }
            counts.values().filter(|n| **n >= 2).count()
        };
        let differentiated = multi
            .iter()
            .filter(|g| g.len() >= 6 && tissues(g) >= 2)
            .count();
        let organ_bodies = multi
            .iter()
            .filter(|g| g.len() >= 8 && roles(g) >= 4)
            .count();
        // Powered movement: a body with a muscle at work, excitable cells to
        // make it beat, and a fin to push with.
        let ids = e.index();
        let mut pulling = vec![false; n];
        for b in e.bonds.iter().filter(|b| b.activation > 0.05) {
            pulling[ids[&b.a]] = true;
            pulling[ids[&b.b]] = true;
        }
        let moving = multi
            .iter()
            .filter(|g| {
                g.iter().any(|&i| pulling[i])
                    && g.iter().any(|&i| built(&e.cells[i], &[FIN]))
                    && g.iter().any(|&i| built(&e.cells[i], &[CHANNEL]))
            })
            .count();
        let per_body = |count: usize| {
            if multi.is_empty() {
                0.
            } else {
                count as f32 / multi.len() as f32
            }
        };
        // Skeleton is counted among animals, as it is in a fossil bed: of
        // the cells of animal-like bodies, the share that carry mineral. Of
        // every living cell it would be a mark Earth itself has never met,
        // most of whose cells are bacteria and plants.
        let mineral = {
            let (mut animal, mut stony) = (0usize, 0usize);
            for g in &groups {
                if Self::kind_of(e, g) == Some(3) {
                    animal += g.len();
                    stony += g.iter().filter(|&&i| mineral_fraction(&e.cells[i]) > 0.025).count();
                }
            }
            if animal > 0 { stony as f32 / animal as f32 } else { 0. }
        };
        let synapses = self.development.synapses.len();
        let diploid = fraction(
            e.cells
                .iter()
                .filter(|c| c.lifecycle.homolog.is_some())
                .count(),
        );
        let guided = e
            .cells
            .iter()
            .filter(|c| built(c, &[PROPEL]) && built(c, &[STEER]))
            .count();
        let defended = e
            .cells
            .iter()
            .filter(|c| built(c, &[BARRIER, SPINE, TOXIN, WALL]) || mineral_fraction(c) > 0.025)
            .count();
        let walled = e.cells.iter().filter(|c| built(c, &[WALL])).count();
        let (_, hunters, carnivores) = self.diets(e, &groups);
        let complex = e
            .cells
            .iter()
            .filter(|c| c.state.symbiont > 0. || c.state.mitochondria > 0.)
            .count();
        // Cells burning most of their fuel right down with oxygen.
        let breathing = e.cells.iter().filter(|c| c.state.breath > 0.5).count();
        // An animal: a body of four cells or more, with a mouth or digestive
        // enzymes somewhere in it, that takes a quarter or more of its fuel
        // from other life. (It used to need most of its cells to be mouths,
        // which left out every animal with a head and a body behind it.)
        let animals = groups
            .iter()
            .filter(|g| {
                let mut flux = crate::observations::Flux::default();
                for &i in g.iter() {
                    flux.add(&e.cells[i].state.flux);
                }
                g.len() >= 4 && flux.eater() && g.iter().any(|&i| built(&e.cells[i], &[9, ENGULF]))
            })
            .count();
        // Workers and breeders: a body of four or more with at least one stem
        // cell that can still divide and one committed cell that cannot.
        let castes = groups
            .iter()
            .filter(|g| {
                g.len() >= 4
                    && g.iter()
                        .any(|&i| e.cells[i].state.fate == 0 && e.cells[i].activity(7) > 0.25)
                    && g.iter()
                        .any(|&i| e.cells[i].state.fate > 0 && e.cells[i].activity(7) <= 0.25)
            })
            .count();
        let committed = e.cells.iter().filter(|c| c.state.fate > 0).count();
        // A shaped body: four or more cells, most of them carrying the polar
        // scaffold that aims each division along or across the heading.
        let shaped = groups
            .iter()
            .filter(|g| {
                g.len() >= 4
                    && g.iter().filter(|&&i| built(&e.cells[i], &[20])).count() * 2 >= g.len()
            })
            .count();
        // A gut: a body of four or more whose mouths pass on at least a fifth
        // of everything the body takes in to cells that have no mouth.
        let guts = groups
            .iter()
            .filter(|g| {
                let (mut shared, mut intake, mut unarmed) = (0., 0., 0);
                for &i in g.iter() {
                    shared += e.cells[i].state.flux.shared;
                    intake += e.cells[i].state.flux.intake();
                    unarmed += usize::from(!built(&e.cells[i], &[ENGULF]));
                }
                g.len() >= 4 && unarmed >= 2 && shared > 0.01 && shared >= 0.2 * intake
            })
            .count();
        let arcs = self.reflex_arcs(e);
        let per_any = |count: usize| {
            if groups.is_empty() {
                0.
            } else {
                count as f32 / groups.len() as f32
            }
        };
        // key, gene carriers, built measure, measure needed, count, extra gate
        let rows: [(&'static str, f32, f32, f32, usize, bool); 27] = [
            (
                "cells",
                1.,
                if e.divisions > 0 { 1. } else { 0.5 },
                1.,
                n,
                n > 0,
            ),
            (
                "colonies",
                genes(&[2]),
                fraction(attached),
                0.3,
                attached,
                true,
            ),
            (
                "bodies",
                genes(&[2, 20, 21]),
                largest as f32 / 24.,
                1.,
                largest,
                largest >= 3,
            ),
            (
                "tissues",
                0.,
                per_body(differentiated),
                0.25,
                differentiated,
                differentiated >= 5,
            ),
            (
                "fates",
                genes(&crate::physical::BECOME),
                fraction(committed),
                0.1,
                committed,
                true,
            ),
            (
                "castes",
                genes(&crate::physical::BECOME),
                per_body(castes),
                0.1,
                castes,
                castes >= 3,
            ),
            (
                "shapes",
                genes(&[20, BROADEN]),
                per_body(shaped),
                0.1,
                shaped,
                shaped >= 3,
            ),
            (
                "senses",
                genes(&[RECEPTOR, CHANNEL, 16, 24]),
                makers(&[RECEPTOR, CHANNEL, 16, 24]),
                0.1,
                0,
                true,
            ),
            (
                "swimming",
                genes(&[PROPEL]),
                makers(&[PROPEL]),
                0.1,
                0,
                true,
            ),
            (
                "steering",
                genes(&[STEER]),
                fraction(guided),
                0.05,
                guided,
                true,
            ),
            ("muscle", genes(&[4]), makers(&[4]), 0.1, 0, true),
            ("fins", genes(&[FIN]), makers(&[FIN]), 0.1, 0, true),
            ("skeleton", genes(&[3, 29]), mineral, 0.1, 0, true),
            // Eaters are about one body in a hundred of a food web that lasts.
            (
                "hunters",
                genes(&[9, ENGULF, TOXIN]),
                per_any(hunters),
                0.01,
                hunters,
                hunters >= 3,
            ),
            (
                "carnivores",
                genes(&[9, ENGULF, TOXIN]),
                per_any(carnivores),
                0.005,
                carnivores,
                carnivores >= 3,
            ),
            (
                "fixing",
                genes(&[crate::physical::FIX]),
                makers(&[crate::physical::FIX]),
                0.02,
                0,
                true,
            ),
            (
                "breathing",
                genes(&[crate::physical::RESPIRE]),
                fraction(breathing),
                0.1,
                breathing,
                true,
            ),
            (
                "complex",
                genes(&[ENGULF]),
                fraction(complex),
                0.1,
                complex,
                true,
            ),
            // Animals, like all eaters, are few beside what they eat.
            (
                "animals",
                genes(&[9, ENGULF]),
                per_body(animals) * 20.,
                1.,
                animals,
                animals >= 3,
            ),
            (
                "guts",
                genes(&[ENGULF]),
                per_body(guts),
                0.05,
                guts,
                guts >= 3,
            ),
            (
                "defences",
                genes(&[BARRIER, SPINE, TOXIN, WALL]),
                fraction(defended),
                0.15,
                defended,
                true,
            ),
            (
                "walls",
                genes(&[WALL]),
                fraction(walled),
                0.1,
                walled,
                true,
            ),
            (
                "nerves",
                genes(&[17]),
                fraction(synapses),
                0.05,
                synapses,
                synapses >= 8,
            ),
            // Bodies that move by muscle are animals, and animals are few
            // beside the threads and mats they live among.
            (
                "movement",
                genes(&[4, FIN]),
                per_body(moving),
                0.02,
                moving,
                moving >= 3,
            ),
            (
                "reflexes",
                genes(&[17, CHANNEL]),
                fraction(arcs),
                0.03,
                arcs,
                arcs >= 8,
            ),
            (
                "organs",
                genes(&[8, 9, 21, 22, 23]),
                per_body(organ_bodies),
                0.1,
                organ_bodies,
                organ_bodies >= 3,
            ),
            (
                "sex",
                genes(&[18, 19]),
                diploid,
                0.05,
                self.development.zygotes as usize,
                self.development.zygotes >= 10,
            ),
        ];
        rows.into_iter()
            .map(|(key, genes, built, needed, count, gate)| {
                let progress = (built / needed).clamp(0., 1.);
                let stage = if n == 0 {
                    0
                } else if progress >= 1. && gate {
                    3
                } else if built > 0. {
                    2
                } else if genes > 0. {
                    1
                } else {
                    0
                };
                let first = [1u8, 2, 3].map(|stage| {
                    self.steering
                        .milestones
                        .iter()
                        .find(|m| m.key == key && m.stage == stage)
                        .map(|m| m.time)
                });
                Rung {
                    key,
                    stage,
                    progress,
                    genes,
                    built,
                    count: count as u32,
                    first,
                }
            })
            .collect()
    }

    /// Record the first time each step is reached. Observation only.
    fn census(&mut self, e: &Evolution, core: &mut Core) {
        for rung in self.ladder(e) {
            for stage in 1..=rung.stage {
                if self
                    .steering
                    .milestones
                    .iter()
                    .any(|m| m.key == rung.key && m.stage == stage)
                {
                    continue;
                }
                self.steering.milestones.push(Milestone {
                    key: rung.key.into(),
                    stage,
                    time: core.time,
                    generation: e.peak_generation,
                });
                // The founders' own abilities are not news.
                if core.time < CENSUS_PERIOD || (rung.key == "cells" && stage < 3) {
                    continue;
                }
                let name = RUNG_NAMES
                    .iter()
                    .find(|r| r.0 == rung.key)
                    .map_or(rung.key, |r| r.1);
                core.event(
                    "milestone",
                    match stage {
                        1 => format!("{name}: an enabling gene entered the gene pool."),
                        2 => format!("{name}: the first working cells appeared."),
                        _ => format!("{name} established across the population."),
                    },
                );
            }
        }
    }

    pub fn steering_view(&self, e: &Evolution, core: &Core) -> serde_json::Value {
        let s = &self.steering;
        let now = core.time;
        serde_json::json!({
            "mutation": s.mutation_scale,
            "mutationFactor": self.mutation_factor(now),
            "mixing": s.mixing,
            "sharing": s.sharing,
            "light": s.light,
            "vents": s.vents,
            "sky": self.sky(),
            "storm": (s.storm_until - now).max(0.),
            "goal": s.goal,
            "auto": s.auto,
            "oxygen": self.oxygen(),
            "cycles": self.cycles(e),
            "breath": if e.cells.is_empty() { 0. } else { e.cells.iter().map(|c| c.state.breath).sum::<f32>() / e.cells.len() as f32 },
            "strength": s.goal_strength,
            "period": s.goal_period,
            "nextRound": if s.goal.is_empty() { 0. } else { (s.next_round - now).max(0.) },
            "rounds": s.rounds,
            "culled": s.culled,
            "note": s.round_note,
            "scored": s.round_scored,
            "bodies": s.round_bodies,
            "favoured": e.cells.iter().filter(|c| c.state.favoured).count(),
            "favourPressure": s.favour_pressure,
            "favourCulled": s.favour_culled,
            "winter": (s.winter_until - now).max(0.),
            "losses": s.losses,
            "ladder": self.ladder(e),
            "diet": self.diets(e, &e.groups()).0,
            "kinds": self.kinds(e, &e.groups()),
            "species": self.species(e),
            "sustained": s.sustained,
            "restocked": s.restocked,
            "captures": s.captures,
            "ashore": e.cells.iter().filter(|c| (c.state.ground || c.state.anchored) && c.state.under < 0.3).count(),
            "land": self.ashore_view(e),
            "trees": self.lines_view(e),
            "milestones": s.milestones,
        })
    }
}

impl Core {
    /// Apply one steering command. Invalid commands leave the world intact.
    pub fn steer(&mut self, json: &str) -> Result<String, String> {
        if json.len() > 4096 {
            return Err("Steering command is too large".into());
        }
        let command: Command =
            serde_json::from_str(json).map_err(|_| "Invalid steering command")?;
        if self.config.physical_assay != "full" {
            return Err("Return to the main world before steering evolution".into());
        }
        if let Command::Seed { species, count, sustain } = &command {
            let note = self.seed_species(species, *count)?;
            if *sustain {
                let kept = &mut self
                    .autonomous
                    .as_mut()
                    .and_then(|e| e.physical.as_mut())
                    .unwrap()
                    .steering
                    .sustained;
                if !kept.contains(species) {
                    kept.push(species.clone());
                }
            }
            return Ok(note);
        }
        let mut e = self
            .autonomous
            .take()
            .ok_or("Steering requires the current cellular world")?;
        let result = match e.physical.take() {
            Some(mut h) => {
                let result = self.steer_world(&mut e, &mut h, command);
                e.physical = Some(h);
                result
            }
            None => Err("Steering requires a physical habitat".into()),
        };
        self.autonomous = Some(e);
        let note = result?;
        self.event("world", note.clone());
        Ok(note)
    }

    fn steer_world(
        &mut self,
        e: &mut Evolution,
        h: &mut Habitat,
        command: Command,
    ) -> Result<String, String> {
        let now = self.time;
        if !h.ablation.is_empty() {
            return Err("Return to the main world before steering evolution".into());
        }
        let unit = |v: f32, low: f32, high: f32, what: &str| {
            if v.is_finite() && (low..=high).contains(&v) {
                Ok(v)
            } else {
                Err(format!("{what} must be between {low} and {high}"))
            }
        };
        Ok(match command {
            Command::Mutation { scale } => {
                h.steering.mutation_scale = unit(scale, 0., 10., "Mutation rate")?;
                format!("Copying errors now arise at {scale:.2}× the natural rate.")
            }
            Command::Mixing { scale } => {
                h.steering.mixing = unit(scale, 0.25, 8., "Ocean mixing")?;
                format!("The waters now mix at {scale:.2}× their natural rate.")
            }
            Command::Seed { .. } => unreachable!("released before the world is taken"),
            Command::Keep { species } => {
                let name = crate::bestiary::SPECIES
                    .iter()
                    .find(|s| s.0 == species)
                    .ok_or("Unknown starter species")?
                    .1;
                if !h.steering.sustained.contains(&species) {
                    h.steering.sustained.push(species);
                }
                format!("{name}s will be restocked if they fall to a handful.")
            }
            Command::Sustain { on } => {
                h.steering.sustained = if on {
                    let living = h.species(&e);
                    crate::bestiary::SPECIES
                        .iter()
                        .filter(|s| living.iter().any(|l| l.0 == s.1))
                        .map(|s| s.0.to_string())
                        .collect()
                } else {
                    Vec::new()
                };
                if on {
                    "Released species will be restocked if they fall to a handful.".into()
                } else {
                    "Released species are on their own.".into()
                }
            }
            Command::Sharing { scale } => {
                h.steering.sharing = unit(scale, 0., 8., "Gene sharing")?;
                if scale > 0. {
                    format!("Touching cells now trade genes at {scale:.2}× the rare rate.")
                } else {
                    "Cells keep their genes to themselves.".into()
                }
            }
            Command::Light { scale } => {
                h.steering.light = unit(scale, 0.25, 8., "Star brightness")?;
                format!("The star now shines at {scale:.2}× its natural brightness.")
            }
            Command::Vents { scale } => {
                h.steering.vents = unit(scale, 0.25, 8., "Vent activity")?;
                format!("The planet's interior now feeds vents at {scale:.2}× its natural rate.")
            }
            Command::Bombardment { scale } => {
                h.steering.bombardment = unit(scale, 0., 8., "Bombardment")?;
                if scale == 0. {
                    "Nothing more falls from space.".to_string()
                } else {
                    format!("Bodies from space now arrive at {scale:.2}× the natural rate.")
                }
            }
            Command::Goal {
                goal,
                strength,
                period,
            } => {
                let auto = goal == "auto";
                if !goal.is_empty() && !auto && !GOALS.contains(&goal.as_str()) {
                    return Err("Unknown breeding goal".into());
                }
                // Strength 0 only leans new genes toward the trait: nothing is removed.
                h.steering.goal_strength = if strength == 0. {
                    0.
                } else {
                    unit(strength, 0.05, 0.6, "Selection strength")?
                };
                h.steering.goal_period = unit(period, 5., 400., "Selection period")?;
                h.steering.round_note.clear();
                h.steering.round_births = e.divisions;
                h.steering.anchors.clear();
                h.steering.next_round = now + h.steering.goal_period;
                h.steering.goal = if auto { String::new() } else { goal.clone() };
                h.steering.auto = auto;
                h.steering.auto_rounds = 0;
                h.steering.auto_tried.clear();
                if auto {
                    h.follow_ladder(e);
                    format!(
                        "New genes now lean toward whatever life has not reached yet, starting with {}.{}",
                        h.steering.goal,
                        if strength > 0. {
                            format!(" Every {period:.0} cycles the bodies showing least of it make way.")
                        } else {
                            " Nothing is removed.".into()
                        }
                    )
                } else if goal.is_empty() {
                    "Selection stopped. Survival is decided by the world alone.".into()
                } else {
                    format!("Rewarding {goal}: every {period:.0} cycles the bodies showing least of it make way.")
                }
            }
            Command::Favour { body, pressure } => {
                let pressure = unit(pressure, 0., 0.2, "Favour pressure")?;
                let group = e
                    .groups()
                    .into_iter()
                    .find(|g| g.iter().any(|&i| e.cells[i].id == body))
                    .ok_or("Select a living body to favour")?;
                for &i in &group {
                    e.cells[i].state.favoured = true;
                }
                h.steering.favour_pressure = pressure;
                format!(
                    "Body #{body} and its descendants are favoured ({} cells marked).",
                    group.len()
                )
            }
            Command::Unfavour {} => {
                for c in &mut e.cells {
                    c.state.favoured = false;
                }
                h.steering.favour_pressure = 0.;
                "No lineage is favoured.".into()
            }
            Command::Storm { scale, duration } => {
                h.steering.storm_scale = unit(scale, 1., 16., "Storm strength")?;
                h.steering.storm_until = now + unit(duration, 1., 2000., "Storm length")?;
                format!("A mutation storm: copying errors ×{scale:.0} for {duration:.0} cycles.")
            }
            Command::Winter { depth, duration } => {
                let depth = unit(depth, 0.02, 0.6, "Cooling")?;
                let duration = unit(duration, 5., 5000., "Ice age length")?;
                if h.steering.winter_until > 0. {
                    return Err("An ice age is already under way".into());
                }
                let applied = depth.min(self.config.temperature);
                self.config.temperature -= applied;
                h.steering.winter_depth = applied;
                h.steering.winter_until = now + duration;
                format!("An ice age begins: {duration:.0} cycles of cold.")
            }
            Command::Meteor { x, z, radius } => {
                let d = h.domain;
                interventions::site(d, x, z)?;
                let radius = unit(radius, 1., d.size() * 0.25, "Impact radius")?;
                let relief = h.planet.as_ref().map_or(0., |p| p.relief_at(x, z));
                let impact = d.position(x, z, d.ground(x, z, self.config.seed) + relief);
                let mut lost = 0;
                for c in &mut e.cells {
                    if c.p.sub(impact).len() < radius {
                        c.state.injury = 1.;
                        lost += 1;
                    }
                }
                if let Some(p) = &mut h.planet {
                    let centre = d.normal(x, z);
                    for i in 0..d.sites() {
                        let (sx, sz) = d.point(i);
                        let distance = if d.spherical() {
                            centre.dot(d.normal(sx, sz)).clamp(-1., 1.).acos() * d.radius()
                        } else {
                            (sx - x).hypot(sz - z)
                        };
                        if distance < radius {
                            p.relief[i] =
                                (p.relief[i] - 2.2 * (1. - distance / radius)).clamp(-8., 8.);
                        }
                    }
                    p.terrain_version = p.terrain_version.wrapping_add(1);
                }
                format!("A meteor struck. {lost} cells were lost and their material returns to the world.")
            }
            Command::Nourish { amount } => {
                if !amount.is_finite() || !(0.01..=4.).contains(&amount) {
                    return Err("Nourishment must be between 0.01 and 4".into());
                }
                // External fuel where life already is, accounted like a brush stroke.
                let before = h.totals(&e.cells);
                let nitrogen = h.nitrogen_total(&e.cells);
                let sites: BTreeSet<usize> = e.cells.iter().map(|c| h.domain.locate(c.p)).collect();
                for &i in &sites {
                    h.solutes[i][0] += amount * h.domain.area(i);
                }
                h.settle_arrivals(before, nitrogen, &e.cells);
                format!(
                    "Fuel rained on {} inhabited sites. Genes are unchanged.",
                    sites.len()
                )
            }
            Command::Nitrogen { change } => {
                if !change.is_finite() || !(-0.9..=0.5).contains(&change) || change == 0. {
                    return Err("Nitrogen changes by between -0.9 and 0.5".into());
                }
                let k = crate::physical::NITROGEN;
                if change > 0. {
                    let moved = h.air_nitrogen * change;
                    let surface: f64 = (0..h.domain.sites()).map(|i| h.domain.area(i)).sum();
                    for i in 0..h.domain.sites() {
                        h.solutes[i][k] += moved * h.domain.area(i) / surface;
                    }
                    h.air_nitrogen -= moved;
                    format!("Lightning fixed {:.0}% of the air's nitrogen into the seas and soils. Whatever feeds on light can now grow further.", change * 100.)
                } else {
                    let mut moved = 0.;
                    for s in &mut h.solutes {
                        moved -= s[k] * change;
                        s[k] *= 1. + change;
                    }
                    h.air_nitrogen += moved;
                    format!("{:.0}% of the dissolved nitrogen went back to the air. Growth is tighter now.", -change * 100.)
                }
            }
            Command::Oxygen { level } => {
                if !level.is_finite() || !(0.0..=1.).contains(&level) {
                    return Err("Oxygen runs from 0 to 1".into());
                }
                h.oxygenate(level);
                if level == 0. {
                    "The air holds no free oxygen. Life is slow until light-feeders make some.".into()
                } else {
                    format!("{:.0}% of the planet's oxygen is now free to breathe.", level * 100.)
                }
            }
            Command::Water { change } => {
                if !change.is_finite() || !(-0.9..=2.).contains(&change) || change == 0. {
                    return Err("Add up to 2 deep, or remove up to nine-tenths".into());
                }
                let d = h.domain;
                let p = h.planet.as_mut().ok_or("This world has no planetary surface")?;
                let before: f64 = p.water.iter().sum();
                for i in 0..d.sites() {
                    if change > 0. {
                        p.water[i] += change * if d.spherical() { d.area(i) } else { 1. };
                    } else {
                        p.water[i] *= 1. + change;
                    }
                }
                // Water from outside the world, or lost to it: the books move with it.
                let delta = p.water.iter().sum::<f64>() - before;
                p.initial_water += delta;
                if change > 0. {
                    format!("A great flood: water {change:.2} deep fell everywhere and is running to the low ground.")
                } else {
                    format!("A long drought: {:.0}% of the standing water is gone.", -change * 100.)
                }
            }
            Command::Dying { fraction } => {
                let fraction = unit(fraction, 0.05, 0.95, "Extinction fraction")?;
                let mut lost = 0;
                for c in &mut e.cells {
                    if self.random() < fraction {
                        c.state.injury = 1.;
                        lost += 1;
                    }
                }
                format!("A great dying: {lost} cells lost. The survivors inherit the world.")
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world(cells: usize) -> Core {
        let mut c = Core::new(Config {
            initial_cells: cells,
            capacity: 4096,
            ..Config::default()
        })
        .unwrap();
        c.start_physical();
        // These tests are about steering, on a planet whose air is already breathable.
        habitat(&mut c).oxygenate(1.);
        c
    }
    fn save(c: &Core) -> String {
        serde_json::to_string(c).unwrap()
    }
    fn habitat(c: &mut Core) -> &mut Habitat {
        c.autonomous.as_mut().unwrap().physical.as_mut().unwrap()
    }
    fn make_gene(action: usize) -> autonomous::Gene {
        autonomous::Gene {
            action,
            promoter: [0.; autonomous::INPUTS],
            bias: 2.,
            strength: 1.,
            fates: 15,
        }
    }

    #[test]
    fn default_steering_leaves_the_world_bit_identical() {
        let mut a = world(64);
        let mut b = world(64);
        a.steer(r#"{"kind":"mutation","scale":1}"#).unwrap();
        a.events.clear();
        a.step(120);
        b.step(120);
        b.events.clear();
        a.events.clear();
        assert_eq!(save(&a), save(&b));
    }

    #[test]
    fn mutation_scale_changes_how_often_copies_vary_and_zero_freezes_genes() {
        let edits = |scale: f32| {
            let mut c = world(64);
            c.steer(&format!(r#"{{"kind":"mutation","scale":{scale}}}"#))
                .unwrap();
            c.step(400);
            let e = c.autonomous.as_ref().unwrap();
            assert!(e.divisions > 0);
            e.point_edits + e.duplications + e.deletions + e.innovations
        };
        let (frozen, natural, wild) = (edits(0.), edits(1.), edits(8.));
        assert_eq!(frozen, 0);
        assert!(natural > 0 && wild > natural * 2, "{natural} {wild}");
    }

    #[test]
    fn breeding_removes_low_scorers_recycles_them_and_spares_the_leaders() {
        let mut c = world(96);
        c.step(8);
        // Give a third of the founders a real gene for skin. They must still
        // synthesise the material themselves before it counts.
        {
            let e = c.autonomous.as_mut().unwrap();
            for (i, cell) in e.cells.iter_mut().enumerate() {
                if i % 3 == 0 {
                    cell.genome.genes.push(make_gene(BARRIER));
                    // An edited chromosome is a new genome.
                    cell.genome.id = e.next_genome;
                    e.next_genome += 1;
                }
            }
        }
        c.step(60);
        let carriers_before = {
            let e = c.autonomous.as_ref().unwrap();
            e.cells.iter().filter(|c| carries(c, &[BARRIER])).count() as f32 / e.cells.len() as f32
        };
        c.steer(r#"{"kind":"goal","goal":"defence","strength":0.4,"period":5}"#)
            .unwrap();
        c.step(4 * 60);
        let e = c.autonomous.as_ref().unwrap();
        let h = e.physical.as_ref().unwrap();
        assert!(h.steering.rounds >= 5 && h.steering.culled > 0);
        let carriers_after =
            e.cells.iter().filter(|c| carries(c, &[BARRIER])).count() as f32 / e.cells.len() as f32;
        assert!(
            carriers_after > carriers_before,
            "{carriers_before} -> {carriers_after}"
        );
        assert!(
            h.errors(&e.cells).iter().all(|v| v.abs() < 1e-4),
            "{:?}",
            h.errors(&e.cells)
        );
        assert!(e.valid() && !e.cells.is_empty());
    }

    #[test]
    fn breeding_without_variation_removes_nothing() {
        let mut c = world(64);
        c.steer(r#"{"kind":"mutation","scale":0}"#).unwrap();
        c.steer(r#"{"kind":"goal","goal":"support","strength":0.5,"period":5}"#)
            .unwrap();
        c.step(200);
        let s = &habitat(&mut c).steering;
        assert!(s.rounds > 0);
        assert_eq!(s.culled, 0);
    }

    #[test]
    fn favoured_lineage_is_inherited_and_displaces_the_rest() {
        let mut c = world(64);
        c.step(40);
        let body = c.autonomous.as_ref().unwrap().cells[0].id;
        c.steer(&format!(
            r#"{{"kind":"favour","body":{body},"pressure":0.1}}"#
        ))
        .unwrap();
        c.step(4 * 120);
        let e = c.autonomous.as_ref().unwrap();
        let favoured = e.cells.iter().filter(|c| c.state.favoured).count();
        assert!(favoured > 1, "descendants inherit the mark");
        assert!(
            favoured * 2 > e.cells.len(),
            "{favoured} of {}",
            e.cells.len()
        );
        c.steer(r#"{"kind":"unfavour"}"#).unwrap();
        assert!(c
            .autonomous
            .as_ref()
            .unwrap()
            .cells
            .iter()
            .all(|c| !c.state.favoured));
    }

    #[test]
    fn disasters_are_bounded_reversible_and_conserve_material() {
        let mut c = world(128);
        c.step(20);
        let warm = c.config.temperature;
        c.steer(r#"{"kind":"winter","depth":0.2,"duration":10}"#)
            .unwrap();
        assert!((c.config.temperature - (warm - 0.2)).abs() < 1e-6);
        assert!(c
            .steer(r#"{"kind":"winter","depth":0.2,"duration":10}"#)
            .is_err());
        c.step(48);
        assert!((c.config.temperature - warm).abs() < 1e-6);
        let before = c.autonomous.as_ref().unwrap().cells.len();
        c.steer(r#"{"kind":"dying","fraction":0.5}"#).unwrap();
        c.step(1);
        let e = c.autonomous.as_ref().unwrap();
        assert!(e.cells.len() < before * 3 / 4 && !e.cells.is_empty());
        let h = e.physical.as_ref().unwrap();
        assert!(h.errors(&e.cells).iter().all(|v| v.abs() < 1e-4));
        c.steer(r#"{"kind":"storm","scale":8,"duration":20}"#)
            .unwrap();
        let now = c.time;
        assert_eq!(habitat(&mut c).mutation_factor(now), 8.);
        assert!(c.steer(r#"{"kind":"mutation","scale":99}"#).is_err());
        assert!(c.steer(r#"{"kind":"mixing","scale":4}"#).is_ok());
        assert!(c.steer(r#"{"kind":"mixing","scale":40}"#).is_err());
        assert!(c.steer(r#"{"kind":"sharing","scale":4}"#).is_ok());
        assert!(c.steer(r#"{"kind":"sharing","scale":40}"#).is_err());
        assert!(c.steer(r#"{"kind":"sharing","scale":0}"#).is_ok());

        assert!(c.steer(r#"{"kind":"light","scale":3}"#).is_ok());
        assert!(c.steer(r#"{"kind":"light","scale":0}"#).is_err());
        assert!(c
            .steer(r#"{"kind":"goal","goal":"wings","strength":0.2,"period":30}"#)
            .is_err());
        assert!(c
            .steer(r#"{"kind":"meteor","x":0,"z":0,"radius":6}"#)
            .is_ok());
        assert!(c.steer(r#"{"kind":"nourish","amount":0.5}"#).is_ok());
        assert!(c.steer(r#"{"kind":"nourish","amount":50}"#).is_err());
        c.step(4);
        let e = c.autonomous.as_ref().unwrap();
        assert!(e.valid());
        let h = e.physical.as_ref().unwrap();
        assert!(
            h.errors(&e.cells).iter().all(|v| v.abs() < 1e-4),
            "{:?}",
            h.errors(&e.cells)
        );
    }

    #[test]
    fn breeding_hunters_prunes_armed_bodies_that_catch_nothing_and_spares_prey() {
        let mut c = world(96);
        let mut e = c.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        // Twenty-one founders carry a built mouth; only the first has eaten.
        for cell in e.cells.iter_mut().take(21) {
            cell.material.as_mut().unwrap().proteins[ENGULF] = 0.02;
        }
        e.cells[0].state.flux.prey = 5.;
        h.steering.goal = "hunting".into();
        e.divisions = 1000; // plenty of recent births to draw the quota from
        h.selection_round(&mut e, &mut c);
        let culled: Vec<usize> = (0..e.cells.len())
            .filter(|&i| e.cells[i].state.culled)
            .collect();
        assert!((1..=6).contains(&culled.len()), "{culled:?}");
        assert!(
            culled.iter().all(|&i| (1..21).contains(&i)),
            "only idle armed cells: {culled:?}"
        );
        assert_eq!(h.steering.round_note, "selecting");
        // With no armed body at all there is nobody to judge.
        for cell in &mut e.cells {
            cell.material.as_mut().unwrap().proteins[ENGULF] = 0.;
            cell.state.culled = false;
        }
        e.divisions = 2000;
        h.selection_round(&mut e, &mut c);
        assert!(e.cells.iter().all(|cell| !cell.state.culled));
        assert_eq!(h.steering.round_note, "no-variation");
    }

    #[test]
    fn diets_tell_feeders_herbivores_omnivores_and_carnivores_apart() {
        let mut c = world(64);
        let e = c.autonomous.as_mut().unwrap();
        // Founders start apart: each is its own body.
        let diet = |cell: &mut Cell, f: fn(&mut crate::observations::Flux)| f(&mut cell.state.flux);
        diet(&mut e.cells[0], |f| f.photosynthesis = 10.);
        diet(&mut e.cells[1], |f| f.detritus = 3.);
        diet(&mut e.cells[2], |f| f.chemosynthesis = 3.);
        diet(&mut e.cells[3], |f| (f.photosynthesis, f.prey) = (5., 5.));
        diet(&mut e.cells[4], |f| (f.prey, f.prey_animal) = (5., 2.5));
        diet(&mut e.cells[5], |f| (f.prey, f.prey_animal) = (5., 4.5));
        diet(&mut e.cells[6], |f| (f.photosynthesis, f.prey) = (9.5, 0.5)); // a nibble is not a diet
        let h = e.physical.as_ref().unwrap();
        let (cells, eaters, carnivores) = h.diets(e, &e.groups());
        // light, dissolved fuel, remains, vents, herbivore, omnivore, carnivore
        assert_eq!(cells, [2, 0, 1, 1, 1, 1, 1]);
        // Two of the three eaters eat animals: the omnivore and the carnivore.
        assert_eq!((eaters, carnivores), (3, 2));
    }

    #[test]
    fn a_breeding_goal_leans_new_genes_toward_itself_and_no_goal_changes_nothing() {
        let family = |goal: &str| {
            let mut c = world(64);
            let mut s = Steering::default();
            s.goal = goal.into();
            let focus = s.focus();
            let genome = c.autonomous.as_ref().unwrap().cells[0].genome.clone();
            let founders = genome.genes.len();
            let (mut new, mut hits) = (0, 0);
            for _ in 0..4000 {
                let mut copy = genome.clone();
                let edits = copy.mutate_toward(&mut c, 0., 4., focus);
                if edits[3] > 0 && copy.genes.len() > founders {
                    new += 1;
                    hits += usize::from([2, 4, 5, 20, CHANNEL, FIN].contains(&copy.genes.last().unwrap().action));
                }
            }
            hits as f32 / new as f32
        };
        let (plain, leaning) = (family(""), family("movement"));
        assert!(plain < 0.25, "six genes in forty-four by chance: {plain}");
        assert!(leaning > 0.45, "half of new genes come from the goal's family: {leaning}");
        // With no goal a copy draws exactly the random numbers it always did.
        let (mut a, mut b) = (world(64), world(64));
        let genome = a.autonomous.as_ref().unwrap().cells[0].genome.clone();
        for _ in 0..500 {
            let (mut x, mut y) = (genome.clone(), genome.clone());
            x.mutate_scaled(&mut a, 0., 4.);
            y.mutate_toward(&mut b, 0., 4., Steering::default().focus());
            assert_eq!(x.genes, y.genes);
        }
    }

    #[test]
    fn kinds_tell_microbes_plant_like_fungus_like_and_animal_like_life_apart() {
        let mut c = world(64);
        let e = c.autonomous.as_mut().unwrap();
        let make = |cell: &mut Cell, action: usize| cell.material.as_mut().unwrap().proteins[action] = 0.02;
        e.cells[0].state.flux.photosynthesis = 5.; // a bare light feeder: it lives as a plant
        make(&mut e.cells[6], PROPEL);
        e.cells[6].state.flux.photosynthesis = 5.; // so does a light feeder that swims
        make(&mut e.cells[7], ENGULF);
        e.cells[7].state.flux.prey = 5.; // an animal fed on prey
        make(&mut e.cells[1], WALL);
        e.cells[1].state.flux.photosynthesis = 5.;
        make(&mut e.cells[2], WALL);
        e.cells[2].state.flux.detritus = 5.;
        make(&mut e.cells[3], ENGULF);
        make(&mut e.cells[4], PROPEL);
        make(&mut e.cells[5], WALL); // walled but not yet fed: unsorted
        let h = e.physical.as_ref().unwrap();
        let kinds = h.kinds(e, &e.groups());
        assert_eq!(kinds, [e.cells.len() - 8, 3, 1, 3]);
    }

    #[test]
    fn a_species_splits_into_lines_and_each_is_hung_from_the_line_it_arose_from() {
        let mut c = world(64);
        let e = c.autonomous.as_mut().unwrap();
        let gene = |action: usize| autonomous::Gene { action, promoter: [0.; autonomous::INPUTS], bias: 1., strength: 1., fates: 15 };
        // Forty cells of one kind. Ten have gained a mouth, and six of those eyes as well.
        let base = traits(&e.cells[0].genome.genes);
        for (k, cell) in e.cells.iter_mut().take(40).enumerate() {
            cell.collection_sources = vec!["Plankton".into()];
            cell.genome.genes.retain(|g| g.action != ENGULF && g.action != crate::development::PHOTIC);
            if k < 10 {
                cell.genome.genes.push(gene(ENGULF));
            }
            if k < 6 {
                cell.genome.genes.push(gene(crate::development::PHOTIC));
            }
        }
        let plain = traits(&e.cells[39].genome.genes);
        let (mouthed, sighted) = (plain | 1 << 1, plain | 1 << 1 | 1 << 9);
        assert_eq!(traits(&e.cells[9].genome.genes), mouthed);
        assert_eq!(traits(&e.cells[0].genome.genes), sighted);
        assert!(base & (1 << 1) == 0, "a silent or absent gene is no adaptation");
        // The births that made them, as the engine notes them when a cell divides.
        let mut h = e.physical.take().unwrap();
        h.note_line("Plankton", plain, plain, 10.);
        h.note_line("Plankton", plain, mouthed, 20.);
        h.note_line("Plankton", mouthed, sighted, 30.);
        h.note_line("Plankton", plain, mouthed, 40.); // seen again: nothing new
        assert_eq!(h.lines["Plankton"].len(), 3);
        let view = h.lines_view(e);
        let tree = view.as_array().unwrap().iter().find(|t| t["species"] == "Plankton").unwrap();
        let branch = |t: u32| tree["branches"].as_array().unwrap().iter().find(|b| b["traits"] == t).unwrap().clone();
        assert_eq!(tree["cells"], 40);
        assert_eq!((branch(mouthed)["from"].as_u64(), branch(mouthed)["since"].as_f64(), branch(mouthed)["cells"].as_u64()), (Some(plain as u64), Some(20.), Some(4)));
        assert_eq!((branch(sighted)["from"].as_u64(), branch(sighted)["since"].as_f64(), branch(sighted)["cells"].as_u64()), (Some(mouthed as u64), Some(30.), Some(6)));
        assert_eq!(branch(plain)["cells"], 30);
        // A line with nothing living in it and nothing living come of it is forgotten once there are too many.
        for k in 0..LINES as u32 + 5 {
            h.note_line("Plankton", plain, plain | 1 << 19 | (k + 1) << 20, 50.);
        }
        h.prune_lines(e);
        assert_eq!(h.lines["Plankton"].len(), 3);
        assert!(h.valid(&e.cells));
        e.physical = Some(h);
    }

    #[test]
    fn flood_and_drought_change_how_much_water_the_planet_holds_and_the_books_follow() {
        let mut c = world(64);
        let sea = |c: &mut Core| {
            let p = habitat(c).planet.as_ref().unwrap();
            (p.water.iter().sum::<f64>(), p.water_error())
        };
        let (before, _) = sea(&mut c);
        c.steer(r#"{"kind":"water","change":0.5}"#).unwrap();
        let (flooded, error) = sea(&mut c);
        assert!(flooded > before * 1.5 && error.abs() < 1e-9, "{before} -> {flooded}, error {error}");
        c.steer(r#"{"kind":"water","change":-0.5}"#).unwrap();
        let (dried, error) = sea(&mut c);
        assert!((dried - flooded * 0.5).abs() < 1e-6 && error.abs() < 1e-9);
        for bad in ["0", "3", "-0.95"] {
            assert!(c.steer(&format!(r#"{{"kind":"water","change":{bad}}}"#)).is_err());
        }
        c.step(40);
        assert!(c.autonomous.as_ref().unwrap().valid());
    }

    #[test]
    fn an_automatic_goal_follows_the_ladder_and_moves_on_from_a_step_that_stalls() {
        let mut c = world(64);
        c.step(4 * 30);
        let note = c
            .steer(r#"{"kind":"goal","goal":"auto","strength":0.25,"period":5}"#)
            .unwrap();
        assert!(note.contains("starting with bodies"), "{note}");
        let state = |c: &mut Core| {
            let s = &habitat(c).steering;
            (s.auto, s.goal.clone(), s.auto_tried.clone())
        };
        // Colonies come first on the ladder, and bigger bodies are how to reach them.
        assert_eq!(state(&mut c), (true, "bodies".to_string(), vec![]));
        // It never stays on one step past its patience: the step is reached,
        // or it is set aside and the next one tried.
        for _ in 0..16 {
            c.step(4 * 5);
            assert!(habitat(&mut c).steering.auto_rounds < AUTO_PATIENCE);
        }
        let (auto, goal, _) = state(&mut c);
        assert!(auto && GOALS.contains(&goal.as_str()), "{goal}");
        // A save carries it, and choosing a trait by hand ends it.
        let mut resumed = Core::load(&save(&c)).unwrap();
        resumed.step(40);
        c.step(40);
        assert_eq!(save(&resumed), save(&c));
        c.steer(r#"{"kind":"goal","goal":"senses","strength":0.25,"period":30}"#).unwrap();
        assert_eq!(state(&mut c), (false, "senses".to_string(), vec![]));
        c.steer(r#"{"kind":"goal","goal":"auto","strength":0.25,"period":30}"#).unwrap();
        c.steer(r#"{"kind":"goal","goal":"","strength":0.25,"period":30}"#).unwrap();
        assert_eq!(state(&mut c), (false, String::new(), vec![]));
    }

    #[test]
    fn ladder_reports_measured_steps_and_steering_resumes_from_a_save() {
        let mut c = world(64);
        c.step(4 * 60);
        c.steer(r#"{"kind":"goal","goal":"bodies","strength":0.2,"period":20}"#)
            .unwrap();
        c.step(40);
        let e = c.autonomous.as_ref().unwrap();
        let h = e.physical.as_ref().unwrap();
        let ladder = h.ladder(e);
        assert_eq!(ladder.len(), 27);
        assert_eq!(ladder[0].stage, 3, "founders divide");
        assert!(ladder
            .iter()
            .all(|r| (0.0..=1.).contains(&r.progress) && r.stage <= 3));
        // No founder carries muscle, skeleton or nerve genes.
        assert!(h.steering.milestones.iter().any(|m| m.key == "cells"));
        let saved = save(&c);
        let mut resumed = Core::load(&saved).unwrap();
        resumed.step(80);
        c.step(80);
        assert_eq!(save(&resumed), save(&c));
    }
}
