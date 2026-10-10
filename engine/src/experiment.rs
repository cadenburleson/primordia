//! Controlled evolutionary locomotion assay. This is deliberately labelled as
//! artificial selection, separate from survival/reproduction in the ecosystem.
use crate::{
    body::{default_development, Body, DEVELOPMENT, V3},
    Core, Organism, ADHESION, ARMOR, BRANCHING, CHEMO, FERTILITY, GENE_COUNT, MOBILITY, MUSCLE,
    PHOTO, PIGMENT, PREDATION, SENSING, SEX, SHAPE, SKELETON, SPECIALIZATION, THERMAL,
};
use serde::{Deserialize, Serialize};
pub const TRIAL_LENGTH: f32 = 20.;
pub const MEASURE_FROM: f32 = 8.;
pub const CANDIDATES: u32 = 12;
fn enabled() -> bool {
    true
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Program {
    pub genes: [f32; GENE_COUNT],
    pub development: [f32; DEVELOPMENT],
    #[serde(default)]
    pub anatomy: Option<crate::articulation::Anatomy>,
}
impl Default for Program {
    fn default() -> Self {
        let mut g = [0.0; GENE_COUNT];
        g[PHOTO] = 0.7;
        g[CHEMO] = 0.4;
        g[MOBILITY] = 0.;
        g[ADHESION] = 0.95;
        g[ARMOR] = 0.3;
        g[THERMAL] = 0.48;
        g[FERTILITY] = 0.2;
        g[SEX] = 0.1;
        g[SHAPE] = 0.6;
        g[BRANCHING] = 0.7;
        g[SPECIALIZATION] = 0.9;
        g[PIGMENT] = 0.5;
        g[MUSCLE] = 0.85;
        g[SKELETON] = 0.9;
        g[SENSING] = 0.5;
        g[PREDATION] = 0.;
        Self {
            genes: g,
            development: default_development(),
            anatomy: None,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrialResult {
    pub generation: u32,
    pub candidate: u32,
    pub distance: f32,
    pub work: f32,
    pub score: f32,
    pub contacts: u32,
    pub steps: u32,
    pub supported: f32,
    pub viable: bool,
    #[serde(default)]
    pub joints: u32,
    #[serde(default)]
    pub joint_error: f32,
    #[serde(default)]
    pub alternations: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Champion {
    pub program: Program,
    pub result: TrialResult,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Experiment {
    #[serde(default)]
    pub jointed: bool,
    #[serde(default = "enabled")]
    pub motors_enabled: bool,
    #[serde(default)]
    pub alternations: u32,
    #[serde(default)]
    pub stance_side: i8,
    #[serde(default)]
    pub max_joint_error: f32,
    #[serde(default)]
    pub replaying: bool,
    #[serde(default)]
    pub queued: Option<Program>,
    pub generation: u32,
    pub candidate: u32,
    pub elapsed: f32,
    pub program: Program,
    pub elite: Program,
    pub champion: Option<Champion>,
    pub round_best: Option<Champion>,
    pub results: Vec<TrialResult>,
    pub progress: Vec<TrialResult>,
    pub origin: V3,
    pub work_from: f32,
    pub steps_from: u32,
    pub supporting: f32,
    pub trace: Vec<[f32; 3]>,
    pub baseline: Option<TrialResult>,
}
impl Default for Experiment {
    fn default() -> Self {
        Self {
            jointed: false,
            motors_enabled: true,
            alternations: 0,
            stance_side: 0,
            max_joint_error: 0.,
            replaying: false,
            queued: None,
            generation: 0,
            candidate: 0,
            elapsed: 0.,
            program: Program::default(),
            elite: Program::default(),
            champion: None,
            round_best: None,
            results: vec![],
            progress: vec![],
            origin: V3::default(),
            work_from: 0.,
            steps_from: 0,
            supporting: 0.,
            trace: vec![],
            baseline: None,
        }
    }
}
impl Experiment {
    pub fn set_motors(&mut self, core: &mut Core, enabled: bool) {
        if self.motors_enabled == enabled {
            return;
        }
        self.motors_enabled = enabled;
        self.elapsed = 0.;
        self.supporting = 0.;
        self.alternations = 0;
        self.stance_side = 0;
        self.max_joint_error = 0.;
        self.trace.clear();
        self.spawn(core);
    }
    pub fn replay(&mut self, core: &mut Core, enabled: bool) {
        if enabled && !self.replaying {
            if let Some(champion) = &self.champion {
                self.queued = Some(self.program.clone());
                self.program = champion.program.clone();
                self.replaying = true;
            } else {
                return;
            }
        } else if !enabled && self.replaying {
            self.program = self.queued.take().unwrap_or_else(|| self.elite.clone());
            self.replaying = false;
        } else {
            return;
        }
        self.elapsed = 0.;
        self.alternations = 0;
        self.stance_side = 0;
        self.max_joint_error = 0.;
        self.supporting = 0.;
        self.trace.clear();
        self.spawn(core);
    }
    pub fn spawn(&self, core: &mut Core) {
        let program = &self.program;
        if core.lineages.is_empty() {
            core.new_lineage(program.genes, 0, self.generation);
        }
        let o = Organism {
            id: 1,
            parent: 0,
            mate: 0,
            family: 0,
            offspring: 0,
            lineage: 1,
            generation: self.generation,
            x: 0.,
            z: 0.,
            vx: 0.,
            vz: 0.,
            heading: 0.,
            energy: 100.,
            age: 0.,
            cells: 1,
            genes: program.genes,
            development: program.development,
            regulation: None,
            anatomy: program.anatomy.clone(),
            body: Body::default(),
            cooldown: 0.,
            alive: true,
            feeding_cooldown: 0.,
            meals: 0,
        };
        core.organisms = vec![o];
        core.next_id = 2;
    }
    pub fn tick(&mut self, core: &mut Core) {
        self.elapsed += 0.25;
        core.time += 0.25;
        let o = &mut core.organisms[0];
        // The assay supplies food and retains daughters. It never spawns an adult.
        // Development still adds one physical cell at a time, with division cost.
        let target = 12;
        if self.elapsed >= o.cells as f32 * 0.5 && o.cells < target {
            o.cells += 1;
            core.births += 1;
            o.energy -= 3.;
        }
        o.body.anatomy = o.anatomy.clone();
        o.body.sync(
            o.cells as usize,
            &o.genes,
            &o.development,
            o.x,
            o.z,
            0.,
            core.config.seed,
            true,
        );
        let mut effective_genes = o.genes;
        if !self.motors_enabled {
            effective_genes[MUSCLE] = 0.;
        }
        let cost = o.body.advance(
            0.25,
            &effective_genes,
            &o.development,
            core.config.seed,
            true,
            (0., 0.),
            (0., 0.),
        );
        o.energy -= cost;
        o.age += 0.25;
        let center = o.body.center();
        o.x = center.x;
        o.z = center.z;
        if self.elapsed == MEASURE_FROM {
            self.origin = center;
            self.work_from = o.body.work;
            self.steps_from = o.body.steps;
        }
        if self.elapsed > MEASURE_FROM {
            self.max_joint_error = self.max_joint_error.max(
                o.body
                    .joints
                    .iter()
                    .map(|j| j.limit_error)
                    .fold(0., f32::max),
            );
            let left = o
                .body
                .nodes
                .iter()
                .skip(4)
                .filter(|n| n.contact && n.rest.z < 0.)
                .count();
            let right = o
                .body
                .nodes
                .iter()
                .skip(4)
                .filter(|n| n.contact && n.rest.z >= 0.)
                .count();
            let side = if left > right {
                -1
            } else if right > left {
                1
            } else {
                0
            };
            if side != 0 {
                if self.stance_side != 0 && side != self.stance_side {
                    self.alternations += 1;
                }
                self.stance_side = side;
            }
            let core_height = o.body.nodes.iter().take(4).map(|n| n.p.y).sum::<f32>() / 4.;
            if core_height > 0.4 && o.body.contacts > 0 {
                self.supporting += 0.25;
            }
            self.trace.push([center.x, center.y, center.z]);
            if self.trace.len() > 120 {
                self.trace.remove(0);
            }
        }
        if self.elapsed >= TRIAL_LENGTH && (self.replaying || !self.motors_enabled) {
            self.elapsed = 0.;
            self.alternations = 0;
            self.stance_side = 0;
            self.max_joint_error = 0.;
            self.supporting = 0.;
            self.trace.clear();
            self.spawn(core);
        } else if self.elapsed >= TRIAL_LENGTH {
            let distance = (center.x - self.origin.x).hypot(center.z - self.origin.z);
            let work = o.body.work - self.work_from;
            let supported = self.supporting / (TRIAL_LENGTH - MEASURE_FROM);
            let viable = o.body.valid(target as usize)
                && o.body.max_strain < 1.
                && o.energy > 0.
                && self.max_joint_error < 0.15;
            // Net travel rejects oscillation-in-place. Work and collapsed posture
            // have costs, and all trials receive exactly the same growth/settling.
            let score = if viable {
                distance * (0.65 + supported * 0.35) - work * 0.018
            } else {
                -100.
            };
            let result = TrialResult {
                generation: self.generation,
                candidate: self.candidate,
                distance,
                work,
                score,
                contacts: o.body.contacts,
                steps: o.body.steps - self.steps_from,
                supported,
                viable,
                joints: o.body.joints.len() as u32,
                joint_error: self.max_joint_error,
                alternations: self.alternations,
            };
            if self.baseline.is_none() {
                self.baseline = Some(result.clone());
            }
            let better =
                |c: &Option<Champion>| c.as_ref().is_none_or(|v| result.score > v.result.score);
            if better(&self.round_best) {
                self.round_best = Some(Champion {
                    program: self.program.clone(),
                    result: result.clone(),
                });
            }
            if better(&self.champion) {
                self.champion = Some(Champion {
                    program: self.program.clone(),
                    result: result.clone(),
                });
            }
            self.results.push(result);
            if self.results.len() > 120 {
                self.results.remove(0);
            }
            self.candidate += 1;
            if self.candidate == CANDIDATES {
                let winner = self.round_best.take().unwrap();
                self.elite = winner.program;
                self.progress.push(winner.result.clone());
                if self.progress.len() > 100 {
                    self.progress.remove(0);
                }
                core.event("motion",format!("Locomotion generation {}: best net travel {:.2}, work {:.2}. Artificial selection retained the best viable program.",self.generation+1,winner.result.distance,winner.result.work));
                self.generation += 1;
                self.candidate = 0;
            }
            self.program = self.elite.clone();
            // Candidate zero retests the elite; every other descendant varies
            // morphology and controller together. No canned walking animation.
            if self.candidate > 0 {
                if let Some(anatomy) = &mut self.program.anatomy {
                    for value in anatomy.values_mut() {
                        if core.random() < 0.5 {
                            *value = (*value + (core.random() - 0.5) * 0.4).clamp(0., 1.);
                        }
                    }
                }
                for v in &mut self.program.development {
                    if core.random() < 0.65 {
                        *v = (*v + (core.random() - 0.5) * 0.5).clamp(0., 1.);
                    }
                }
                for k in [MUSCLE, SKELETON, SPECIALIZATION, SENSING] {
                    if core.random() < 0.4 {
                        self.program.genes[k] =
                            (self.program.genes[k] + (core.random() - 0.5) * 0.2).clamp(0.2, 1.);
                    }
                }
            }
            self.elapsed = 0.;
            self.alternations = 0;
            self.stance_side = 0;
            self.max_joint_error = 0.;
            self.supporting = 0.;
            self.trace.clear();
            self.spawn(core);
        }
        core.history_clock += 0.25;
        if core.history_clock >= 3. {
            core.history_clock = 0.;
            core.record_history();
        }
    }
    pub fn metadata(&self, core: &Core) -> serde_json::Value {
        let o = &core.organisms[0];
        let center = o.body.center();
        let measuring = self.elapsed >= MEASURE_FROM;
        serde_json::json!({"jointed":self.jointed,"motorsEnabled":self.motors_enabled,"alternations":self.alternations,"jointError":self.max_joint_error,"replaying":self.replaying,"generation":self.generation,"candidate":self.candidate,"candidates":CANDIDATES,
            "elapsed":self.elapsed,"duration":TRIAL_LENGTH,"phase":if self.elapsed<6.{"Developing"}else if !measuring{"Settling"}else{"Measuring"},
            "distance":if measuring{(center.x-self.origin.x).hypot(center.z-self.origin.z)}else{0.},
            "work":if measuring{o.body.work-self.work_from}else{0.},"baseline":self.baseline,
            "champion":self.champion.as_ref().map(|c|&c.result),"progress":self.progress,"results":self.results,
            "trace":self.trace})
    }
    pub fn valid(&self) -> bool {
        let programs = [&self.program, &self.elite];
        let program_valid = |p: &Program| {
            p.anatomy.as_ref().is_none_or(|a| a.valid())
                && p.genes
                    .iter()
                    .chain(p.development.iter())
                    .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        };
        let result_valid = |r: &TrialResult| {
            [r.distance, r.work, r.score, r.supported, r.joint_error]
                .iter()
                .all(|v| v.is_finite())
                && r.distance >= 0.
                && r.work >= 0.
                && (0.0..=1.1).contains(&r.supported)
        };
        self.jointed == self.program.anatomy.is_some()
            && self.jointed == self.elite.anatomy.is_some()
            && (-1..=1).contains(&self.stance_side)
            && self.max_joint_error.is_finite()
            && self.max_joint_error >= 0.
            && self.candidate < CANDIDATES
            && self.elapsed.is_finite()
            && (0.0..TRIAL_LENGTH).contains(&self.elapsed)
            && self.results.len() <= 120
            && self.progress.len() <= 100
            && self.trace.len() <= 120
            && programs.iter().all(|p| program_valid(p))
            && self.queued.as_ref().is_none_or(program_valid)
            && (!self.replaying || (self.champion.is_some() && self.queued.is_some()))
            && [&self.champion, &self.round_best].iter().all(|c| {
                c.as_ref()
                    .is_none_or(|c| program_valid(&c.program) && result_valid(&c.result))
            })
            && self
                .results
                .iter()
                .chain(self.progress.iter())
                .all(result_valid)
            && self.baseline.as_ref().is_none_or(result_valid)
            && self
                .trace
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite() && v.abs() < 100.))
            && [
                self.work_from,
                self.supporting,
                self.origin.x,
                self.origin.y,
                self.origin.z,
            ]
            .iter()
            .all(|v| v.is_finite())
    }
}
