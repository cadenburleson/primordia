//! Mineralized segments and angular joints, with internal antagonistic torque.
//! Two bounded recurrent units per joint control flexion and extension. This is
//! a designed artificial-life mechanism, not anatomical bone or muscle biology.
use crate::body::{Body, DEVELOPMENT, V3};
use crate::{GENE_COUNT, MUSCLE, SENSING, SKELETON, SPECIALIZATION};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Anatomy {
    /// Flexion range, extension range, torque, mineral stiffness, response time.
    pub traits: [f32; 5],
    /// Bias, oscillator, distal contact, angle, direction, proximal contact,
    /// own recurrent state, opponent recurrent state. Values encode -2..2.
    pub network: [[f32; 8]; 2],
}
impl Default for Anatomy {
    fn default() -> Self {
        Self {
            traits: [0.65, 0.55, 0.65, 0.9, 0.65],
            network: [
                [0.45, 0.96, 0.65, 0.42, 0.7, 0.45, 0.58, 0.38],
                [0.45, 0.04, 0.35, 0.58, 0.3, 0.55, 0.58, 0.38],
            ],
        }
    }
}
impl Anatomy {
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.traits
            .iter_mut()
            .chain(self.network.iter_mut().flatten())
    }
    pub fn values(&self) -> impl Iterator<Item = &f32> {
        self.traits.iter().chain(self.network.iter().flatten())
    }
    pub fn valid(&self) -> bool {
        self.values()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Joint {
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub segments: [usize; 2],
    pub enabled: bool,
    pub min: f32,
    pub max: f32,
    pub angle: f32,
    pub flexion: f32,
    pub extension: f32,
    pub state: [f32; 2],
    pub work: f32,
    pub limit_error: f32,
}
fn angle_and_gradients(a: V3, b: V3, c: V3) -> (f32, [V3; 3]) {
    let u = a.sub(b);
    let v = c.sub(b);
    let lu = u.len().max(0.001);
    let lv = v.len().max(0.001);
    let q = u.mul(1. / lu);
    let r = v.mul(1. / lv);
    let cos = q.dot(r).clamp(-0.99999, 0.99999);
    let sin = (1. - cos * cos).sqrt();
    let ga = q.mul(cos).sub(r).mul(1. / (lu * sin));
    let gc = r.mul(cos).sub(q).mul(1. / (lv * sin));
    (cos.acos(), [ga, ga.add(gc).mul(-1.), gc])
}
impl Body {
    pub(crate) fn sync_joints(&mut self, d: &[f32; DEVELOPMENT]) {
        let Some(anatomy) = self.anatomy.as_ref() else {
            self.joints.clear();
            return;
        };
        let previous = std::mem::take(&mut self.joints);
        for c in 4..self.nodes.len() {
            let b = super::body::bud(c, d).1;
            let a = if b >= 4 {
                super::body::bud(b, d).1
            } else {
                (b + 1) % 4
            };
            if let Some(old) = previous.iter().find(|j| j.a == a && j.b == b && j.c == c) {
                self.joints.push(old.clone());
                continue;
            }
            let rest =
                angle_and_gradients(self.nodes[a].rest, self.nodes[b].rest, self.nodes[c].rest).0;
            self.joints.push(Joint {
                a,
                b,
                c,
                segments: [(a, b), (b, c)].map(|(a, b)| {
                    self.links
                        .iter()
                        .position(|l| {
                            l.purpose == 0 && ((l.a == a && l.b == b) || (l.a == b && l.b == a))
                        })
                        .unwrap()
                }),
                enabled: false,
                min: (rest - 0.2 - anatomy.traits[0] * 0.9).max(0.15),
                max: (rest + 0.15 + anatomy.traits[1] * 0.6).min(3.05),
                angle: rest,
                flexion: 0.,
                extension: 0.,
                state: [0.; 2],
                work: 0.,
                limit_error: 0.,
            });
        }
    }
    /// Health/integrity and actual expressed tissue gate joint support and motors.
    fn joint_supply(&self, j: &Joint, g: &[f32; GENE_COUNT]) -> (f32, f32) {
        let nodes = [&self.nodes[j.a], &self.nodes[j.b], &self.nodes[j.c]];
        let integrity = j
            .segments
            .iter()
            .map(|&i| {
                let l = &self.links[i];
                if l.integrity <= 0.08 {
                    0.
                } else {
                    l.integrity
                }
            })
            .fold(1., f32::min);
        let health = nodes.iter().map(|n| n.health).fold(1., f32::min) * integrity;
        let support = g[SKELETON]
            * g[SPECIALIZATION]
            * if !self.regulated || nodes.iter().any(|n| n.role == 1) {
                health
            } else {
                0.
            };
        let supply = if self.regulated {
            nodes
                .iter()
                .filter(|n| n.role == 2)
                .map(|n| (n.reserve / 0.25).clamp(0., 1.) * n.health)
                .fold(0., f32::max)
                * integrity
        } else {
            health
        };
        (support, supply * g[MUSCLE] * g[SPECIALIZATION])
    }
    pub(crate) fn control_joints(
        &mut self,
        h: f32,
        g: &[f32; GENE_COUNT],
        d: &[f32; DEVELOPMENT],
        gradient: (f32, f32),
    ) {
        let Some(anatomy) = self.anatomy.as_ref() else {
            return;
        };
        let supplies: Vec<_> = self
            .joints
            .iter()
            .map(|j| self.joint_supply(j, g))
            .collect();
        let sensing =
            if !self.regulated || self.nodes.iter().any(|n| n.role == 3 && n.health > 0.25) {
                g[SENSING]
            } else {
                0.
            };
        for (index, j) in self.joints.iter_mut().enumerate() {
            j.enabled = supplies[index].0 > 0.16;
            let active = j.enabled && supplies[index].1 > 0.;
            let a = &self.nodes[j.a];
            let b = &self.nodes[j.b];
            let c = &self.nodes[j.c];
            let angle = angle_and_gradients(a.p, b.p, c.p).0;
            let side = if c.rest.z > b.rest.z { 1. } else { -1. };
            let phase = self.clock * (0.5 + d[5] * 2.5) * std::f32::consts::TAU
                + j.c as f32 * d[6] * 4.
                + side * d[10] * std::f32::consts::PI;
            let toward = c.p.sub(b.p).normalized();
            let direction = (toward.x * gradient.0 + toward.z * gradient.1) * sensing;
            let old = j.state;
            let mut next = [0.; 2];
            for unit in 0..2 {
                let input = [
                    1.,
                    phase.sin(),
                    if c.contact { 1. } else { -1. },
                    ((angle - j.min) / (j.max - j.min) * 2. - 1.).clamp(-1., 1.),
                    direction,
                    if b.contact { 1. } else { -1. },
                    old[unit],
                    old[1 - unit],
                ];
                let drive = anatomy.network[unit]
                    .iter()
                    .zip(input)
                    .map(|(&w, x)| (w * 4. - 2.) * x)
                    .sum::<f32>()
                    .tanh();
                next[unit] = old[unit]
                    + (drive - old[unit]) * (1. - (-h * (2. + anatomy.traits[4] * 12.)).exp());
            }
            j.state = next;
            j.flexion = if active { (next[0] + 1.) * 0.5 } else { 0. };
            j.extension = if active { (next[1] + 1.) * 0.5 } else { 0. };
            j.angle = angle;
        }
    }
    pub(crate) fn solve_joints(&mut self, h: f32, g: &[f32; GENE_COUNT], lambdas: &mut [[f32; 4]]) {
        let Some(anatomy) = self.anatomy.as_ref() else {
            return;
        };
        let torque = 10. + anatomy.traits[2] * 70.;
        for k in 0..self.joints.len() {
            let joint = self.joints[k].clone();
            let (support, motor) = self.joint_supply(&joint, g);
            if support <= 0.16 {
                continue;
            }
            for mode in 0..4 {
                let j = &self.joints[k];
                let (angle, gradient) =
                    angle_and_gradients(self.nodes[j.a].p, self.nodes[j.b].p, self.nodes[j.c].p);
                let (target, low, high, compliance) = match mode {
                    0 => (j.min, 0., f32::INFINITY, 0.0000001 / support),
                    1 => (j.max, f32::NEG_INFINITY, 0., 0.0000001 / support),
                    2 => (j.min, -torque * motor * j.flexion * h * h, 0., 0.0002),
                    _ => (j.max, 0., torque * motor * j.extension * h * h, 0.0002),
                };
                let alpha = compliance / (h * h);
                let denominator = gradient.iter().map(|v| v.dot(*v)).sum::<f32>() + alpha;
                let old = lambdas[k][mode];
                let proposed = (-(angle - target) - alpha * old) / denominator.max(0.00001);
                let next = (old + proposed).clamp(low, high);
                let correction = next - old;
                lambdas[k][mode] = next;
                for (i, grad) in [joint.a, joint.b, joint.c].into_iter().zip(gradient) {
                    self.nodes[i].p = self.nodes[i].p.add(grad.mul(correction));
                }
            }
        }
    }
    pub(crate) fn joint_work(
        &mut self,
        h: f32,
        g: &[f32; GENE_COUNT],
        lambdas: &[[f32; 4]],
    ) -> f32 {
        let mut spent = 0.;
        for k in 0..self.joints.len() {
            let j = &self.joints[k];
            let (support, _) = self.joint_supply(j, g);
            let angle =
                angle_and_gradients(self.nodes[j.a].p, self.nodes[j.b].p, self.nodes[j.c].p).0;
            let cost = (lambdas[k][2].abs() + lambdas[k][3].abs()) / (h * h)
                * ((angle - j.angle).abs() + h * 0.02)
                * 0.001;
            let j = &mut self.joints[k];
            j.work += cost;
            spent += cost;
            j.angle = angle;
            j.limit_error = if support > 0.16 {
                (j.min - angle).max(angle - j.max).max(0.)
            } else {
                0.
            };
        }
        spent
    }
    pub fn joints_valid(&self, count: usize) -> bool {
        self.anatomy.as_ref().is_none_or(Anatomy::valid)
            && (self.anatomy.is_some() || self.joints.is_empty())
            && self.joints.len() <= 60
            && self.joints.iter().all(|j| {
                j.segments.iter().all(|&i| i < self.links.len())
                    && j.segments
                        .iter()
                        .zip([(j.a, j.b), (j.b, j.c)])
                        .all(|(&i, (a, b))| {
                            let l = &self.links[i];
                            l.purpose == 0 && ((l.a == a && l.b == b) || (l.a == b && l.b == a))
                        })
                    && j.a < count
                    && j.b < count
                    && j.c < count
                    && j.a != j.b
                    && j.b != j.c
                    && j.a != j.c
                    && [
                        j.min,
                        j.max,
                        j.angle,
                        j.flexion,
                        j.extension,
                        j.work,
                        j.limit_error,
                    ]
                    .iter()
                    .all(|v| v.is_finite())
                    && j.min >= 0.1
                    && j.max <= 3.1
                    && j.min < j.max
                    && (0.0..=std::f32::consts::PI).contains(&j.angle)
                    && (0.0..=1.).contains(&j.flexion)
                    && (0.0..=1.).contains(&j.extension)
                    && j.state
                        .iter()
                        .all(|s| s.is_finite() && (-1.0..=1.0).contains(s))
                    && j.work >= 0.
                    && j.limit_error >= 0.
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::default_development;
    fn specimen() -> (Body, [f32; GENE_COUNT], [f32; DEVELOPMENT]) {
        let g = [0.9; GENE_COUNT];
        let d = default_development();
        let mut body = Body {
            anatomy: Some(Anatomy::default()),
            ..Body::default()
        };
        for cells in 1..=12 {
            body.sync(cells, &g, &d, 0., 0., 0., 8042, true);
            body.advance(0.5, &g, &d, 8042, true, (0., 0.), (0., 0.));
        }
        (body, g, d)
    }
    #[test]
    fn articulated_bodies_hold_segments_limits_and_pay_for_motion() {
        let (mut body, g, d) = specimen();
        let mut passive = body.clone();
        let mut off = g;
        off[MUSCLE] = 0.;
        let work = body.work;
        let quiet = passive.work;
        for _ in 0..48 {
            body.advance(0.25, &g, &d, 8042, true, (0., 0.), (0., 0.));
            passive.advance(0.25, &off, &d, 8042, true, (0., 0.), (0., 0.));
        }
        println!(
            "joint body: active path {} passive {}, work {}, strain {}, angle error {}",
            body.path,
            passive.path,
            body.work - work,
            body.max_strain,
            body.joints.iter().map(|j| j.limit_error).fold(0., f32::max)
        );
        assert!(body.valid(12));
        assert_eq!(body.joints.len(), 8);
        assert!(body.work > work + 0.1);
        assert_eq!(passive.work, quiet);
        assert!(body
            .links
            .iter()
            .filter(|l| l.kind == 1)
            .all(|l| l.strain < 0.15));
        assert!(body.joints.iter().all(|j| j.limit_error < 0.15));
        assert!(body.contacts > 0);
    }
    #[test]
    fn angular_limits_and_paired_forces_preserve_horizontal_momentum() {
        let (mut body, g, d) = specimen();
        for n in &mut body.nodes {
            n.p.y += 100.;
            n.v = V3::default();
        }
        // Start one joint beyond its inherited maximum and require recovery.
        body.joints[0].min = 0.9;
        body.joints[0].max = 1.4;
        let start = body.center();
        for _ in 0..8 {
            body.advance(0.025, &g, &d, 8042, true, (1., 0.), (0., 0.));
        }
        assert!((body.center().x - start.x).abs() < 0.0001);
        assert!((body.center().z - start.z).abs() < 0.0001);
        assert!(body.joints[0].limit_error < 0.08);
    }
    #[test]
    fn injury_and_empty_stores_disable_joint_actuation() {
        let (mut healthy, g, d) = specimen();
        healthy.regulated = true;
        for n in &mut healthy.nodes {
            n.reserve = 20.;
        }
        let mut empty = healthy.clone();
        for n in &mut empty.nodes {
            n.reserve = 0.;
        }
        let mut injured = healthy.clone();
        for i in 0..injured.nodes.len() {
            injured.injure(i, 0.95);
        }
        let a = healthy.advance(0.25, &g, &d, 8042, true, (0., 0.), (0., 0.));
        let b = empty.advance(0.25, &g, &d, 8042, true, (0., 0.), (0., 0.));
        let c = injured.advance(0.25, &g, &d, 8042, true, (0., 0.), (0., 0.));
        assert!(a > 0.001);
        assert_eq!(b, 0.);
        assert!(c < a * 0.1);
    }
    #[test]
    fn controller_state_growth_and_save_resume_are_preserved() {
        let (mut body, g, d) = specimen();
        let state = body.joints[0].state;
        let work = body.joints[0].work;
        body.sync(13, &g, &d, 0., 0., 0., 8042, true);
        assert_eq!(body.joints[0].state, state);
        assert_eq!(body.joints[0].work, work);
        let mut resumed: Body =
            serde_json::from_str(&serde_json::to_string(&body).unwrap()).unwrap();
        body.advance(0.25, &g, &d, 8042, true, (-1., 0.), (0., 0.));
        resumed.advance(0.25, &g, &d, 8042, true, (-1., 0.), (0., 0.));
        assert_eq!(
            serde_json::to_string(&body).unwrap(),
            serde_json::to_string(&resumed).unwrap()
        );
        assert!(body.valid(13));
    }
    #[test]
    fn sensed_direction_changes_recurrent_control_without_thrust() {
        let (mut a, g, d) = specimen();
        for n in &mut a.nodes {
            n.p.y += 100.;
            n.v = V3::default();
        }
        let mut b = a.clone();
        let start = a.center();
        a.advance(0.25, &g, &d, 8042, true, (1., 0.), (0., 0.));
        b.advance(0.25, &g, &d, 8042, true, (-1., 0.), (0., 0.));
        assert!(a
            .joints
            .iter()
            .zip(&b.joints)
            .any(|(a, b)| (a.flexion - b.flexion).abs() > 0.01));
        assert!((a.center().x - start.x).abs() < 0.0001 && (b.center().x - start.x).abs() < 0.0001);
    }
    #[test]
    fn broken_support_segments_collapse_posture() {
        let (_, mut g, d) = specimen();
        g[MUSCLE] = 0.;
        let mut supported = Body {
            anatomy: Some(Anatomy::default()),
            ..Body::default()
        };
        supported.sync(12, &g, &d, 0., 0., 0., 8042, true);
        let mut soft = supported.clone();
        for link in &mut soft.links {
            link.integrity = 0.;
        }
        for _ in 0..40 {
            supported.advance(0.25, &g, &d, 8042, true, (0., 0.), (0., 0.));
            soft.advance(0.25, &g, &d, 8042, true, (0., 0.), (0., 0.));
        }
        let height = |b: &Body| b.nodes.iter().take(4).map(|n| n.p.y).sum::<f32>() / 4.;
        println!(
            "supported {} unsupported {}",
            height(&supported),
            height(&soft)
        );
        assert!(height(&supported) > height(&soft) + 0.05);
        assert!(soft.joints.iter().all(|j| !j.enabled));
    }
    #[test]
    fn full_size_development_has_bounded_joint_indices_and_finite_mechanics() {
        let g = [0.9; GENE_COUNT];
        let d = default_development();
        let mut body = Body {
            anatomy: Some(Anatomy::default()),
            ..Body::default()
        };
        for count in 1..=64 {
            body.sync(count, &g, &d, 0., 0., 0., 8042, true);
            body.advance(0.25, &g, &d, 8042, true, (0., 0.), (0., 0.));
            assert!(body.valid(count), "invalid at {count}");
        }
        assert_eq!(body.joints.len(), 60);
    }
}
