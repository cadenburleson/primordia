//! Local cell regulation and resource transport in the ecological experiment.
//! This is a bounded artificial-life rule set, not molecular developmental biology.
use crate::body::Body;
use crate::{
    ADHESION, FERTILITY, GENE_COUNT, MUSCLE, PREDATION, SENSING, SEX, SKELETON, SPECIALIZATION,
};
use serde::{Deserialize, Serialize};

pub const MAX_BODY_CELLS: usize = 64;
pub const CELL_BUDGET: usize = 64_000;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Regulation {
    // Six tissue rows: metabolic, support, muscle, sensory, digestive, reproductive.
    // Inputs: bias, exposure, food, stress, neighboring differentiation, polarity.
    // Coefficients encode signed weights as numbers in [0,1].
    pub rules: [[f32; 6]; 6],
    // Supported size, daughter retention, transport conductance, repair investment.
    pub growth: [f32; 4],
}
impl Default for Regulation {
    fn default() -> Self {
        Self {
            rules: [
                [0.64, 0.72, 0.85, 0.38, 0.42, 0.50],
                [0.92, 0.05, 0.45, 0.75, 0.62, 0.92],
                [0.75, 0.85, 0.55, 0.65, 0.68, 0.02],
                [0.56, 0.96, 0.45, 0.40, 0.48, 0.30],
                [0.60, 0.95, 0.15, 0.48, 0.65, 0.45],
                [0.55, 0.15, 0.95, 0.20, 0.60, 0.65],
            ],
            growth: [0.35, 0.7, 0.65, 0.6],
        }
    }
}
impl Regulation {
    pub fn valid(&self) -> bool {
        self.rules
            .iter()
            .flatten()
            .chain(&self.growth)
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
    }
    pub fn target_cells(&self, g: &[f32; GENE_COUNT], body: &Body) -> usize {
        let support = body
            .nodes
            .iter()
            .filter(|n| n.role == 1 && n.health > 0.4)
            .count();
        let carrying = if body.nodes.len() >= 4 {
            (support as f32 / body.nodes.len() as f32 * 3.).min(1.)
        } else {
            0.
        };
        (4. + g[ADHESION] * 12. + carrying * g[SKELETON] * (16. + self.growth[0] * 32.)) as usize
    }
}
impl Body {
    /// External growth/fission/ingestion changes the total cell-energy budget.
    /// Reconcile its distribution once, retaining existing concentration differences.
    pub fn balance_reserves(&mut self, total: f32) {
        let sum = self.reserve_energy();
        let count = self.nodes.len().max(1) as f32;
        for n in &mut self.nodes {
            n.reserve = if sum > 1e-6 {
                n.reserve * total.max(0.) / sum
            } else {
                total.max(0.) / count
            };
        }
    }
    pub fn reserve_energy(&self) -> f32 {
        self.nodes.iter().map(|n| n.reserve).sum()
    }
    pub fn uptake_factor(&self, i: usize) -> f32 {
        let n = &self.nodes[i];
        n.health * if n.role == 0 { 1.25 } else { 0.65 }
    }
    /// Food and stress cues diffuse one graph neighborhood per ecological update.
    /// Simultaneous signal updates avoid a directional traversal-order shortcut.
    pub fn regulate(&mut self, dt: f32, g: &[f32; GENE_COUNT], program: &Regulation, food: &[f32]) {
        self.regulated = true;
        let count = self.nodes.len();
        if count < 2 {
            return;
        }
        let mut neighbors = vec![Vec::new(); count];
        let mut stress = vec![0.0_f32; count];
        for l in &self.links {
            if l.purpose != 0 || l.integrity <= 0.08 {
                continue;
            }
            neighbors[l.a].push(l.b);
            neighbors[l.b].push(l.a);
            stress[l.a] = stress[l.a].max(l.strain.min(1.));
            stress[l.b] = stress[l.b].max(l.strain.min(1.));
        }
        let previous: Vec<_> = self.nodes.iter().map(|n| n.signal).collect();
        let rise = self
            .nodes
            .iter()
            .map(|n| n.rest.y.abs())
            .fold(0.5_f32, f32::max);
        let potency = [
            0.24,
            g[SKELETON] * g[SPECIALIZATION],
            g[MUSCLE] * g[SPECIALIZATION],
            g[SENSING] * g[SPECIALIZATION],
            g[PREDATION] * 0.6,
            (g[FERTILITY] + g[SEX]) * 0.22,
        ];
        for i in 0..count {
            let degree = neighbors[i].len().max(1) as f32;
            let mut nearby = [0.; 3];
            for &j in &neighbors[i] {
                for k in 0..3 {
                    nearby[k] += previous[j][k] / degree;
                }
            }
            let source = [
                food.get(i).copied().unwrap_or(0.).clamp(0., 1.),
                stress[i],
                if self.nodes[i].role == 0 { 0. } else { 1. },
            ];
            let n = &mut self.nodes[i];
            for k in 0..3 {
                n.signal[k] = (previous[i][k]
                    + dt * (source[k] - previous[i][k]) * 0.8
                    + dt * (nearby[k] - previous[i][k]) * 0.7)
                    .clamp(0., 1.);
            }
            let exposure = (1. - neighbors[i].len() as f32 / 6.).clamp(0., 1.);
            let input = [
                1.,
                exposure * 2. - 1.,
                n.signal[0] * 2. - 1.,
                n.signal[1] * 2. - 1.,
                n.signal[2] * 2. - 1.,
                (n.rest.y / rise).clamp(-1., 1.),
            ];
            let mut scores = [0.; 6];
            for role in 0..6 {
                if count < 4 && role != 0 && role != 4 {
                    continue;
                }
                if role == 4 && g[PREDATION] <= 0.06 {
                    continue;
                }
                let response = program.rules[role]
                    .iter()
                    .zip(input)
                    .map(|(w, x)| (w * 2. - 1.) * x)
                    .sum::<f32>();
                scores[role] = potency[role] * (1. + response * 0.5).max(0.05);
            }
            let best = (0..6)
                .max_by(|&a, &b| scores[a].total_cmp(&scores[b]))
                .unwrap();
            if best != n.role as usize && scores[best] > scores[n.role as usize] * 1.12 + 0.015 {
                n.role = best as u8;
                self.differentiations += 1;
            }
        }
        self.refresh_tissue_links();
    }
    pub(crate) fn refresh_tissue_links(&mut self) {
        for l in &mut self.links {
            let a = &self.nodes[l.a];
            let b = &self.nodes[l.b];
            l.kind = match l.purpose {
                0 => {
                    if a.role == 1 || b.role == 1 {
                        1
                    } else {
                        0
                    }
                }
                1 => {
                    if a.role == 2 || b.role == 2 {
                        2
                    } else {
                        3
                    }
                }
                _ => 3,
            };
        }
    }
    /// Conservative transport along intact adhesion/support links. Signed flux
    /// is capped by the donor store, so no cell obtains a negative reserve.
    pub fn transport(&mut self, dt: f32, conductance: f32) {
        for l in &self.links {
            if l.purpose != 0 || l.integrity <= 0.08 {
                continue;
            }
            let delta = (self.nodes[l.a].reserve - self.nodes[l.b].reserve)
                * (dt * (0.15 + conductance * 1.8) * l.integrity).min(0.45);
            let flux = if delta >= 0. {
                delta.min(self.nodes[l.a].reserve)
            } else {
                delta.max(-self.nodes[l.b].reserve)
            };
            self.nodes[l.a].reserve -= flux;
            self.nodes[l.b].reserve += flux;
        }
    }
    pub fn maintain_tissues(
        &mut self,
        dt: f32,
        program: &Regulation,
        acquired: &[f32],
        maintenance: f32,
    ) {
        let count = self.nodes.len().max(1) as f32;
        for (i, n) in self.nodes.iter_mut().enumerate() {
            n.reserve += acquired.get(i).copied().unwrap_or(0.);
        }
        self.transport(dt, program.growth[2]);
        for n in &mut self.nodes {
            let needed = maintenance / count;
            let paid = needed.min(n.reserve);
            n.reserve -= paid;
            // Local starvation impairs a cell even if distant cells still hold food.
            n.health = (n.health - (needed - paid) * 0.18).max(0.05);
            let repair = (dt * (0.03 + program.growth[3] * 0.18))
                .min(1. - n.health)
                .min(n.reserve / 1.8);
            n.health += repair;
            // Multiplication after donor-limited division can round above the
            // available store. Charge at most the actual finite reserve.
            let repair_cost = (repair * 1.8).min(n.reserve);
            n.reserve -= repair_cost;
            self.repair_work += repair_cost;
        }
        for l in &mut self.links {
            let health = self.nodes[l.a].health.min(self.nodes[l.b].health);
            let repair = (dt * program.growth[3] * 0.12 * health)
                .min(1. - l.integrity)
                .min(self.nodes[l.a].reserve.min(self.nodes[l.b].reserve) * 0.5);
            l.integrity += repair;
            self.nodes[l.a].reserve -= repair;
            self.nodes[l.b].reserve -= repair;
            self.repair_work += repair * 2.;
        }
    }
    pub fn pay_motor_work(&mut self, work: f32) {
        let motors = self.nodes.iter().filter(|n| n.role == 2).count().max(1) as f32;
        for n in &mut self.nodes {
            if n.role != 2 {
                continue;
            }
            let due = work / motors;
            let paid = due.min(n.reserve);
            n.reserve -= paid;
            n.health = (n.health - (due - paid) * 0.15).max(0.05);
        }
    }
    pub fn injure(&mut self, i: usize, amount: f32) {
        self.injuries += 1;
        self.nodes[i].health = (self.nodes[i].health - amount).max(0.05);
        for l in &mut self.links {
            if l.a == i || l.b == i {
                l.integrity = (l.integrity - amount).max(0.);
            }
        }
    }
    pub fn regenerative_mass(&self) -> f32 {
        self.nodes.iter().map(|n| n.health).sum::<f32>() / self.nodes.len().max(1) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{default_development, Body};
    fn body() -> Body {
        let mut b = Body::default();
        b.regulated = true;
        b.sync(
            12,
            &[0.9; GENE_COUNT],
            &default_development(),
            0.,
            0.,
            0.,
            8042,
            true,
        );
        b.balance_reserves(24.);
        b
    }
    #[test]
    fn tiny_repair_stores_remain_nonnegative_and_valid() {
        for scale in 1..256 {
            let mut b = body();
            for (i, n) in b.nodes.iter_mut().enumerate() {
                n.reserve = scale as f32 * (i + 1) as f32 * 1e-12;
                n.health = 0.5;
            }
            let before = b.reserve_energy();
            b.maintain_tissues(0.25, &Regulation::default(), &[0.; 12], 0.);
            assert!(b.nodes.iter().all(|n| n.reserve >= 0.));
            assert!(b.valid(12));
            assert!((b.reserve_energy() + b.repair_work - before).abs() < 1e-8);
        }
    }
    #[test]
    fn local_regulation_differentiates_and_rules_can_be_removed() {
        let mut a = body();
        let mut program = Regulation::default();
        let g = [0.9; GENE_COUNT];
        for _ in 0..30 {
            a.regulate(0.25, &g, &program, &[0.8; 12]);
        }
        let roles: std::collections::BTreeSet<_> = a.nodes.iter().map(|n| n.role).collect();
        assert!(roles.len() >= 2, "{:?}", roles);
        assert!(a.differentiations > 0);
        // Inhibit every support response and reset fate to metabolic to compare.
        program.rules[1] = [0.5; 6];
        program.rules[1][0] = 0.;
        let mut off = body();
        for n in &mut off.nodes {
            n.role = 0;
        }
        for _ in 0..30 {
            off.regulate(0.25, &g, &program, &[0.8; 12]);
        }
        assert!(
            off.nodes.iter().filter(|n| n.role == 1).count()
                < a.nodes.iter().filter(|n| n.role == 1).count()
        );
    }
    #[test]
    fn transport_conserves_energy_and_local_repair_spends_it() {
        let mut a = body();
        for n in &mut a.nodes {
            n.reserve = 0.;
        }
        a.nodes[0].reserve = 24.;
        for _ in 0..30 {
            a.transport(0.25, 1.);
        }
        assert!((a.reserve_energy() - 24.).abs() < 0.001);
        assert!(a.nodes[8].reserve > 0.);
        a.injure(0, 0.7);
        let health = a.nodes[0].health;
        let energy = a.reserve_energy();
        for _ in 0..20 {
            a.maintain_tissues(0.25, &Regulation::default(), &[], 0.);
        }
        assert!(a.nodes[0].health > health);
        assert!(a.reserve_energy() < energy);
        assert!(a.repair_work > 0.);
    }
    #[test]
    fn empty_stores_do_not_create_free_repairs() {
        let mut a = body();
        for n in &mut a.nodes {
            n.reserve = 0.;
        }
        a.injure(0, 0.8);
        let before = a.nodes[0].health;
        a.maintain_tissues(1., &Regulation::default(), &[], 0.);
        assert_eq!(a.nodes[0].health, before);
        assert_eq!(a.repair_work, 0.);
    }
    #[test]
    fn mechanical_damage_disables_motors_and_survives_growth() {
        let mut healthy = body();
        let g = [0.9; GENE_COUNT];
        let d = default_development();
        for _ in 0..30 {
            healthy.regulate(0.25, &g, &Regulation::default(), &[0.8; 12]);
        }
        assert!(healthy.nodes.iter().any(|n| n.role == 2));
        let mut injured = healthy.clone();
        for i in 0..injured.nodes.len() {
            if injured.nodes[i].role == 2 {
                injured.injure(i, 0.95);
            }
        }
        let before = injured.links.iter().filter(|l| l.integrity <= 0.08).count();
        assert!(before > 0);
        let mut a = 0.;
        let mut b = 0.;
        for _ in 0..20 {
            a += healthy.advance(0.25, &g, &d, 8042, true, (0., 0.), (0., 0.));
            b += injured.advance(0.25, &g, &d, 8042, true, (0., 0.), (0., 0.));
        }
        assert!(a > b + 0.1, "healthy {a} injured {b}");
        injured.sync(13, &g, &d, 0., 0., 0., 8042, true);
        assert!(injured.links.iter().filter(|l| l.integrity <= 0.08).count() >= before);
        assert!(injured.valid(13));
    }
    #[test]
    fn larger_body_bounds_and_real_support_capacity() {
        let mut a = body();
        let mut d = default_development();
        d[0] = 0.;
        d[1] = 1.;
        d[2] = 1.;
        a.sync(64, &[1.; GENE_COUNT], &d, 0., 0., 0., 8042, true);
        assert!(a.valid(64));
        for n in &mut a.nodes {
            n.role = 1;
        }
        let mut p = Regulation::default();
        p.growth[0] = 1.;
        assert_eq!(p.target_cells(&[1.; GENE_COUNT], &a), 64);
        for n in &mut a.nodes {
            n.health = 0.2;
        }
        assert_eq!(p.target_cells(&[1.; GENE_COUNT], &a), 16);
    }
}
