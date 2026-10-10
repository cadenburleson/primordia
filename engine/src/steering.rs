//! Creator steering: selection, copying-error rate and disturbances chosen by
//! the player. Nothing here builds a trait. Genes still arise through copying
//! errors at division and every material is still paid for by the cell that
//! makes it. Steering only changes who survives and how often copies vary.
use crate::{
    autonomous::{Cell, Evolution},
    body::V3,
    physical::Habitat,
    physiology::{CHANNEL, RECEPTOR},
    *,
};

pub const GOALS: [&str; 5] = ["bodies", "movement", "support", "senses", "hunting"];
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
    /// Brightness of the star: multiplies the light reaching photosynthesis.
    pub light: f32,
    /// Empty, or one of `GOALS`: the trait bodies are bred for.
    pub goal: String,
    pub goal_strength: f32,
    pub goal_period: f32,
    pub next_round: f32,
    pub rounds: u32,
    pub culled: u32,
    pub round_note: String,
    pub round_scored: u32,
    pub round_bodies: u32,
    /// High-water mark of the population; a round never culls far below it.
    pub round_peak: f32,
    /// Chance per cycle that a cell outside the favoured lineage is removed.
    pub favour_pressure: f32,
    pub favour_culled: u32,
    pub winter_until: f32,
    pub winter_depth: f32,
    pub next_census: f32,
    /// Cells lost to an empty energy store, to wasting, and to injury
    /// (wounds, heat, disasters and culling).
    pub losses: [u32; 3],
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
            light: 1.,
            goal: String::new(),
            goal_strength: 0.25,
            goal_period: 30.,
            next_round: 0.,
            rounds: 0,
            culled: 0,
            round_note: String::new(),
            round_scored: 0,
            round_bodies: 0,
            round_peak: 0.,
            favour_pressure: 0.,
            favour_culled: 0,
            winter_until: 0.,
            winter_depth: 0.,
            next_census: 0.,
            losses: [0; 3],
            milestones: Vec::new(),
            anchors: BTreeMap::new(),
        }
    }
}
impl Steering {
    pub fn valid(&self) -> bool {
        [
            self.mutation_scale,
            self.storm_scale,
            self.storm_until,
            self.goal_strength,
            self.goal_period,
            self.next_round,
            self.round_peak,
            self.favour_pressure,
            self.winter_until,
            self.winter_depth,
            self.next_census,
        ]
        .iter()
        .all(|v| v.is_finite() && *v >= 0.)
            && self.mutation_scale <= 10.
            && (0.25..=8.).contains(&self.mixing)
            && (0.25..=8.).contains(&self.light)
            && self.storm_scale <= 16.
            && (0.05..=0.6).contains(&self.goal_strength)
            && (5.0..=400.).contains(&self.goal_period)
            && self.favour_pressure <= 0.2
            && self.winter_depth <= 1.
            && (self.goal.is_empty() || GOALS.contains(&self.goal.as_str()))
            && self.milestones.len() <= 64
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

const RUNG_NAMES: [(&str, &str); 11] = [
    ("cells", "Living cells"),
    ("colonies", "Colonies"),
    ("bodies", "Multicellular bodies"),
    ("tissues", "Specialised tissues"),
    ("senses", "Senses"),
    ("muscle", "Muscle"),
    ("skeleton", "Skeleton"),
    ("nerves", "Nerve nets"),
    ("movement", "Powered movement"),
    ("organs", "Organs"),
    ("sex", "Sexual reproduction"),
];

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Command {
    Mutation { scale: f32 },
    Mixing { scale: f32 },
    Light { scale: f32 },
    Goal { goal: String, strength: f32, period: f32 },
    Favour { body: u32, pressure: f32 },
    Unfavour {},
    Storm { scale: f32, duration: f32 },
    Winter { depth: f32, duration: f32 },
    Meteor { x: f32, z: f32, radius: f32 },
    Dying { fraction: f32 },
    Nourish { amount: f64 },
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
/// A gene that is expressed, or merely carried, before its material exists.
fn latent(c: &Cell, actions: &[usize]) -> f32 {
    actions.iter().map(|&k| c.expression[k]).fold(0f32, f32::max)
        + if carries(c, actions) { 0.25 } else { 0. }
}
fn mineral_fraction(c: &Cell) -> f32 {
    c.material
        .as_ref()
        .map_or(0., |m| (m.mineral / m.mass().max(1e-12)) as f32)
}

impl Habitat {
    /// Copying-error multiplier in force at `time`.
    /// Diffusion rates in force. Chemical signals keep their natural range.
    pub(crate) fn diffusivity(&self, natural: [f64; 6]) -> [f64; 6] {
        let mut rates = natural;
        for k in [0, 1, 2, 5] {
            rates[k] *= self.steering.mixing as f64;
        }
        rates
    }
    pub(crate) fn mutation_factor(&self, time: f32) -> f32 {
        let s = &self.steering;
        let storm = if time < s.storm_until { s.storm_scale } else { 1. };
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
            self.steering.next_census = now + CENSUS_PERIOD;
        }
    }

    /// (measured performance, enabling material). Performance dominates; the
    /// material term gives selection something to act on before a new tissue
    /// does useful work.
    fn goal_score(&self, e: &Evolution, group: &[usize], goal: &str) -> (f32, f32) {
        let n = group.len() as f32;
        let mean = |f: &dyn Fn(&Cell) -> f32| group.iter().map(|&i| f(&e.cells[i])).sum::<f32>() / n;
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
                (
                    n.min(64.) * (1. + 0.5 * (roles - 1.)) - 1.,
                    mean(&|c| c.activity(2)),
                )
            }
            "movement" => {
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
                (
                    rate * (1. + travel.min(8.)),
                    embodied * mean(&|c| c.activity(4) + 0.5 * c.activity(CHANNEL) + latent(c, &[4])),
                )
            }
            "support" => (
                embodied * mean(&mineral_fraction),
                embodied * mean(&|c| c.activity(3) + latent(c, &[3])),
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
                mean(&|c| c.activity(9) + latent(c, &[9])),
            ),
            _ => (0., 0.),
        }
    }

    /// One round of artificial selection: the lowest-scoring bodies are
    /// removed and recycled. Survivors inherit the freed room and material.
    fn selection_round(&mut self, e: &mut Evolution, core: &mut Core) {
        let goal = self.steering.goal.clone();
        let groups = e.groups();
        let total = e.cells.len();
        let raw: Vec<_> = groups
            .iter()
            .map(|g| self.goal_score(e, g, &goal))
            .collect();
        self.steering.anchors = if goal == "movement" {
            e.cells.iter().map(|c| (c.id, c.p)).collect()
        } else {
            BTreeMap::new()
        };
        let top = |f: fn(&(f32, f32)) -> f32| raw.iter().map(f).fold(0f32, f32::max);
        let (best_measure, best_material) = (top(|r| r.0), top(|r| r.1));
        let scores: Vec<f32> = raw
            .iter()
            .map(|r| {
                (if best_measure > 0. { r.0 / best_measure } else { 0. })
                    + 0.25 * (if best_material > 0. { r.1 / best_material } else { 0. })
            })
            .collect();
        let best = scores.iter().copied().fold(0f32, f32::max);
        let worst = scores.iter().copied().fold(f32::INFINITY, f32::min);
        let s = &mut self.steering;
        s.rounds += 1;
        s.round_bodies = groups.len() as u32;
        s.round_scored = scores.iter().filter(|v| **v > 0.).count() as u32;
        s.round_peak = s.round_peak.max(total as f32);
        // Harvest regrowth: remove up to `goal_strength` of the population but
        // never push it below 60% of its recent high-water mark.
        let quota = (total as f32 * s.goal_strength).min(total as f32 - 0.6 * s.round_peak);
        if total < 48 {
            s.round_note = "too-few".into();
            return;
        }
        if best <= 0. || best - worst < 1e-6 {
            s.round_note = "no-variation".into();
            return;
        }
        if quota < 1. {
            // A world that can no longer reach its old peak slowly resets the bar.
            s.round_peak *= 0.99;
            s.round_note = "recovering".into();
            return;
        }
        let mut order: Vec<(f32, usize)> = scores
            .iter()
            .enumerate()
            .map(|(i, v)| (v + core.random() * 1e-5, i))
            .collect();
        order.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut removed = 0usize;
        for (_, i) in order {
            // Never remove the leaders, and never overshoot the quota.
            if scores[i] >= best * 0.999 || removed as f32 >= quota {
                break;
            }
            if (removed + groups[i].len()) as f32 > quota + 1. {
                continue;
            }
            for &c in &groups[i] {
                e.cells[c].state.injury = 1.;
            }
            removed += groups[i].len();
        }
        s.culled += removed as u32;
        s.round_note = if removed > 0 { "selecting" } else { "recovering" }.into();
    }

    /// Measured state of each step from single cells to complex bodies.
    pub(crate) fn ladder(&self, e: &Evolution) -> Vec<Rung> {
        let n = e.cells.len();
        let fraction = |count: usize| if n > 0 { count as f32 / n as f32 } else { 0. };
        let genes = |actions: &[usize]| fraction(e.cells.iter().filter(|c| carries(c, actions)).count());
        let makers = |actions: &[usize]| fraction(e.cells.iter().filter(|c| built(c, actions)).count());
        let groups = e.groups();
        let multi: Vec<&Vec<usize>> = groups.iter().filter(|g| g.len() >= 3).collect();
        let largest = groups.iter().map(Vec::len).max().unwrap_or(0);
        let attached = groups.iter().filter(|g| g.len() >= 2).map(|g| g.len()).sum::<usize>();
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
        let moving = multi
            .iter()
            .filter(|g| g.iter().map(|&i| e.cells[i].state.motor_work).sum::<f64>() > 1e-6)
            .count();
        let per_body = |count: usize| {
            if multi.is_empty() {
                0.
            } else {
                count as f32 / multi.len() as f32
            }
        };
        let mineral = fraction(e.cells.iter().filter(|c| mineral_fraction(c) > 0.025).count());
        let synapses = self.development.synapses.len();
        let diploid = fraction(
            e.cells
                .iter()
                .filter(|c| c.lifecycle.homolog.is_some())
                .count(),
        );
        // key, gene carriers, built measure, measure needed, count, extra gate
        let rows: [(&'static str, f32, f32, f32, usize, bool); 11] = [
            ("cells", 1., if e.divisions > 0 { 1. } else { 0.5 }, 1., n, n > 0),
            ("colonies", genes(&[2]), fraction(attached), 0.3, attached, true),
            ("bodies", genes(&[2, 20, 21]), largest as f32 / 24., 1., largest, largest >= 3),
            ("tissues", 0., per_body(differentiated), 0.25, differentiated, differentiated >= 5),
            ("senses", genes(&[RECEPTOR, CHANNEL, 16, 24]), makers(&[RECEPTOR, CHANNEL, 16, 24]), 0.1, 0, true),
            ("muscle", genes(&[4]), makers(&[4]), 0.1, 0, true),
            ("skeleton", genes(&[3, 29]), mineral, 0.1, 0, true),
            ("nerves", genes(&[17]), fraction(synapses), 0.05, synapses, synapses >= 8),
            ("movement", genes(&[4]), per_body(moving), 0.1, moving, moving >= 3),
            ("organs", genes(&[8, 9, 21, 22, 23]), per_body(organ_bodies), 0.1, organ_bodies, organ_bodies >= 3),
            ("sex", genes(&[18, 19]), diploid, 0.05, self.development.zygotes as usize, self.development.zygotes >= 10),
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
                Rung { key, stage, progress, genes, built, count: count as u32, first }
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
                let name = RUNG_NAMES.iter().find(|r| r.0 == rung.key).map_or(rung.key, |r| r.1);
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
            "light": s.light,
            "storm": (s.storm_until - now).max(0.),
            "goal": s.goal,
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
            Command::Light { scale } => {
                h.steering.light = unit(scale, 0.25, 8., "Star brightness")?;
                format!("The star now shines at {scale:.2}× its natural brightness.")
            }
            Command::Goal { goal, strength, period } => {
                if !goal.is_empty() && !GOALS.contains(&goal.as_str()) {
                    return Err("Unknown breeding goal".into());
                }
                h.steering.goal_strength = unit(strength, 0.05, 0.6, "Selection strength")?;
                h.steering.goal_period = unit(period, 5., 400., "Selection period")?;
                h.steering.round_note.clear();
                h.steering.round_peak = 0.;
                h.steering.anchors.clear();
                h.steering.next_round = now + h.steering.goal_period;
                h.steering.goal = goal.clone();
                if goal.is_empty() {
                    "Breeding stopped. Survival is decided by the world alone.".into()
                } else {
                    format!("Breeding for {goal}: every {period:.0} cycles the weakest {:.0}% are removed.", strength * 100.)
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
                format!("Body #{body} and its descendants are favoured ({} cells marked).", group.len())
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
                            p.relief[i] = (p.relief[i] - 2.2 * (1. - distance / radius)).clamp(-8., 8.);
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
                let sites: BTreeSet<usize> = e.cells.iter().map(|c| h.domain.locate(c.p)).collect();
                for &i in &sites {
                    h.solutes[i][0] += amount * h.domain.area(i);
                }
                let after = h.totals(&e.cells);
                h.initial_organic += after.0 - before.0;
                h.initial_mineral += after.1 - before.1;
                h.initial_energy += after.2 - before.2;
                format!("Fuel rained on {} inhabited sites. Genes are unchanged.", sites.len())
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
            c.steer(&format!(r#"{{"kind":"mutation","scale":{scale}}}"#)).unwrap();
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
        // Give a third of the founders a real contractile gene. They must still
        // synthesise the material themselves before it counts.
        {
            let e = c.autonomous.as_mut().unwrap();
            for (i, cell) in e.cells.iter_mut().enumerate() {
                if i % 3 == 0 {
                    cell.genome.genes.push(make_gene(4));
                    // An edited chromosome is a new genome.
                    cell.genome.id = e.next_genome;
                    e.next_genome += 1;
                }
            }
        }
        c.step(60);
        let carriers_before = {
            let e = c.autonomous.as_ref().unwrap();
            e.cells.iter().filter(|c| carries(c, &[4])).count() as f32 / e.cells.len() as f32
        };
        c.steer(r#"{"kind":"goal","goal":"movement","strength":0.4,"period":5}"#).unwrap();
        c.step(4 * 60);
        let e = c.autonomous.as_ref().unwrap();
        let h = e.physical.as_ref().unwrap();
        assert!(h.steering.rounds >= 5 && h.steering.culled > 0);
        let carriers_after =
            e.cells.iter().filter(|c| carries(c, &[4])).count() as f32 / e.cells.len() as f32;
        assert!(carriers_after > carriers_before, "{carriers_before} -> {carriers_after}");
        assert!(h.errors(&e.cells).iter().all(|v| v.abs() < 1e-4), "{:?}", h.errors(&e.cells));
        assert!(e.valid() && !e.cells.is_empty());
    }

    #[test]
    fn breeding_without_variation_removes_nothing() {
        let mut c = world(64);
        c.steer(r#"{"kind":"mutation","scale":0}"#).unwrap();
        c.steer(r#"{"kind":"goal","goal":"support","strength":0.5,"period":5}"#).unwrap();
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
        c.steer(&format!(r#"{{"kind":"favour","body":{body},"pressure":0.1}}"#)).unwrap();
        c.step(4 * 120);
        let e = c.autonomous.as_ref().unwrap();
        let favoured = e.cells.iter().filter(|c| c.state.favoured).count();
        assert!(favoured > 1, "descendants inherit the mark");
        assert!(favoured * 2 > e.cells.len(), "{favoured} of {}", e.cells.len());
        c.steer(r#"{"kind":"unfavour"}"#).unwrap();
        assert!(c.autonomous.as_ref().unwrap().cells.iter().all(|c| !c.state.favoured));
    }

    #[test]
    fn disasters_are_bounded_reversible_and_conserve_material() {
        let mut c = world(128);
        c.step(20);
        let warm = c.config.temperature;
        c.steer(r#"{"kind":"winter","depth":0.2,"duration":10}"#).unwrap();
        assert!((c.config.temperature - (warm - 0.2)).abs() < 1e-6);
        assert!(c.steer(r#"{"kind":"winter","depth":0.2,"duration":10}"#).is_err());
        c.step(48);
        assert!((c.config.temperature - warm).abs() < 1e-6);
        let before = c.autonomous.as_ref().unwrap().cells.len();
        c.steer(r#"{"kind":"dying","fraction":0.5}"#).unwrap();
        c.step(1);
        let e = c.autonomous.as_ref().unwrap();
        assert!(e.cells.len() < before * 3 / 4 && !e.cells.is_empty());
        let h = e.physical.as_ref().unwrap();
        assert!(h.errors(&e.cells).iter().all(|v| v.abs() < 1e-4));
        c.steer(r#"{"kind":"storm","scale":8,"duration":20}"#).unwrap();
        let now = c.time;
        assert_eq!(habitat(&mut c).mutation_factor(now), 8.);
        assert!(c.steer(r#"{"kind":"mutation","scale":99}"#).is_err());
        assert!(c.steer(r#"{"kind":"mixing","scale":4}"#).is_ok());
        assert!(c.steer(r#"{"kind":"mixing","scale":40}"#).is_err());
        assert!(c.steer(r#"{"kind":"light","scale":3}"#).is_ok());
        assert!(c.steer(r#"{"kind":"light","scale":0}"#).is_err());
        assert!(c.steer(r#"{"kind":"goal","goal":"wings","strength":0.2,"period":30}"#).is_err());
        assert!(c.steer(r#"{"kind":"meteor","x":0,"z":0,"radius":6}"#).is_ok());
        assert!(c.steer(r#"{"kind":"nourish","amount":0.5}"#).is_ok());
        assert!(c.steer(r#"{"kind":"nourish","amount":50}"#).is_err());
        c.step(4);
        let e = c.autonomous.as_ref().unwrap();
        assert!(e.valid());
        let h = e.physical.as_ref().unwrap();
        assert!(h.errors(&e.cells).iter().all(|v| v.abs() < 1e-4), "{:?}", h.errors(&e.cells));
    }

    #[test]
    fn ladder_reports_measured_steps_and_steering_resumes_from_a_save() {
        let mut c = world(64);
        c.step(4 * 60);
        c.steer(r#"{"kind":"goal","goal":"bodies","strength":0.2,"period":20}"#).unwrap();
        c.step(40);
        let e = c.autonomous.as_ref().unwrap();
        let h = e.physical.as_ref().unwrap();
        let ladder = h.ladder(e);
        assert_eq!(ladder.len(), 11);
        assert_eq!(ladder[0].stage, 3, "founders divide");
        assert!(ladder.iter().all(|r| (0.0..=1.).contains(&r.progress) && r.stage <= 3));
        // No founder carries muscle, skeleton or nerve genes.
        assert!(h.steering.milestones.iter().any(|m| m.key == "cells"));
        let saved = save(&c);
        let mut resumed = Core::load(&saved).unwrap();
        resumed.step(80);
        c.step(80);
        assert_eq!(save(&resumed), save(&c));
    }
}
