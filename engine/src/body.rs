//! Cell-scale articulated bodies. The renderer consumes these positions directly.
//! XPBD distance constraints, contact friction and internal actuation provide motion;
//! there is no forward force or prescribed gait for multicellular bodies.
use crate::{ADHESION, GENE_COUNT, MUSCLE, SENSING, SKELETON, SPECIALIZATION};
use serde::{Deserialize, Serialize};

pub const DEVELOPMENT: usize = 12;
fn intact() -> f32 {
    1.
}
pub fn default_development() -> [f32; DEVELOPMENT] {
    // Branch count, reach, elevation, curl, width, frequency, phase, bias,
    // friction, contact feedback, left/right phase, motor distribution.
    [0.4, 0.5, 0.65, 0.15, 0.5, 0.4, 0.5, 0.4, 0.7, 0.3, 0.5, 0.6]
}
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl V3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub(crate) fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
    pub(crate) fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
    pub(crate) fn mul(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
    pub(crate) fn dot(self, b: Self) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
    pub(crate) fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub(crate) fn normalized(self) -> Self {
        self.mul(1.0 / self.len().max(1e-6))
    }
    fn finite(self) -> bool {
        [self.x, self.y, self.z]
            .iter()
            .all(|v| v.is_finite() && v.abs() < 1000.)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub p: V3,
    pub v: V3,
    pub rest: V3,
    pub radius: f32,
    pub role: u8,
    pub contact: bool,
    #[serde(default = "intact")]
    pub health: f32,
    #[serde(default)]
    pub reserve: f32,
    #[serde(default)]
    pub signal: [f32; 3],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Link {
    pub a: usize,
    pub b: usize,
    pub rest: f32,
    pub target: f32,
    /// 0 adhesion, 1 load-bearing support, 2 active contractile link, 3 joint brace.
    pub kind: u8,
    pub activation: f32,
    pub strain: f32,
    #[serde(default = "intact")]
    pub integrity: f32,
    #[serde(default)]
    pub purpose: u8,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Body {
    #[serde(default)]
    pub anatomy: Option<crate::articulation::Anatomy>,
    #[serde(default)]
    pub joints: Vec<crate::articulation::Joint>,
    #[serde(default)]
    pub orientation: f32,
    pub nodes: Vec<Cell>,
    pub links: Vec<Link>,
    pub clock: f32,
    pub work: f32,
    pub path: f32,
    pub contacts: u32,
    pub steps: u32,
    pub max_strain: f32,
    #[serde(default)]
    pub regulated: bool,
    #[serde(default)]
    pub differentiations: u32,
    #[serde(default)]
    pub injuries: u32,
    #[serde(default)]
    pub repair_work: f32,
}

// Shared integer noise and height function are mirrored in src/sim/random.ts.
fn hash(x: i32, z: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (z as u32).wrapping_mul(668265263) ^ seed;
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    (h ^ (h >> 16)) as f32 / u32::MAX as f32
}
pub(crate) fn smooth_noise(x: f32, z: f32, seed: u32) -> f32 {
    let ix = x.floor() as i32;
    let iz = z.floor() as i32;
    let mut fx = x - x.floor();
    let mut fz = z - z.floor();
    fx = fx * fx * (3. - 2. * fx);
    fz = fz * fz * (3. - 2. * fz);
    (hash(ix, iz, seed) * (1. - fx) + hash(ix + 1, iz, seed) * fx) * (1. - fz)
        + (hash(ix, iz + 1, seed) * (1. - fx) + hash(ix + 1, iz + 1, seed) * fx) * fz
}
pub fn ground(x: f32, z: f32, seed: u32, flat: bool) -> f32 {
    if flat {
        return 0.;
    }
    ground_scaled(x, z, seed, 31.)
}
pub fn ground_scaled(x: f32, z: f32, seed: u32, edge: f32) -> f32 {
    smooth_noise(x * 0.065 + 20., z * 0.065 + 20., seed) * 5.2
        + smooth_noise(x * 0.16, z * 0.16, seed.wrapping_add(72)) * 1.2
        - 2.15
        - (x.hypot(z) / edge).powi(4) * 4.
}
/// Analytic vertical equilibrium for single-cell swimmers. Multicellular
/// particles use dynamic buoyancy; both render and ingestion share this surface.
pub fn habitat_y(x: f32, z: f32, radius: f32, seed: u32) -> f32 {
    let floor = ground(x, z, seed, false);
    if floor > 0.12 {
        floor + radius
    } else {
        0.12 + radius * (1. - 2. * 0.78)
    }
}
/// A bounded developmental grammar, not a pre-authored animal. Every point is
/// introduced by retained division. Four cells can brace a core; later buds can
/// branch into chains. Genome coefficients change reach, branch count and posture.
pub(crate) fn bud(i: usize, d: &[f32; DEVELOPMENT]) -> (V3, usize) {
    let width = 0.35 + d[4] * 0.55;
    let reach = 0.35 + d[1] * 0.6;
    let rise = 0.3 + d[2] * 0.65;
    let trunk = [
        V3::new(-width / 2., rise, -width / 2.),
        V3::new(width / 2., rise, -width / 2.),
        V3::new(-width / 2., rise, width / 2.),
        V3::new(width / 2., rise, width / 2.),
    ];
    if i < 4 {
        return (trunk[i], i.saturating_sub(1));
    }
    let branches = 2 + (d[0] * 4.99) as usize;
    let branch = (i - 4) % branches;
    let level = (i - 4) / branches + 1;
    let root = branch % 4;
    let angle =
        std::f32::consts::TAU * (branch as f32 / branches as f32) + d[3] * level as f32 * 1.8;
    // Distal branching compresses gradually to keep a 64-cell body within the
    // supported mechanical length bounds. Early developmental shapes are unchanged.
    let extent = if level > 4 {
        4. + ((level - 4) as f32).sqrt()
    } else {
        level as f32
    };
    let r = reach * extent;
    let p = trunk[root].add(V3::new(
        angle.cos() * r,
        -rise * 0.62 * extent,
        angle.sin() * r,
    ));
    (p, if level == 1 { root } else { i - branches })
}
impl Body {
    pub fn center(&self) -> V3 {
        if self.nodes.is_empty() {
            return V3::default();
        }
        self.nodes
            .iter()
            .fold(V3::default(), |p, n| p.add(n.p))
            .mul(1. / self.nodes.len() as f32)
    }
    pub fn sync(
        &mut self,
        count: usize,
        g: &[f32; GENE_COUNT],
        d: &[f32; DEVELOPMENT],
        x: f32,
        z: f32,
        heading: f32,
        seed: u32,
        flat: bool,
    ) {
        if self.nodes.len() == count {
            return;
        }
        let base = if self.nodes.is_empty() {
            self.orientation = heading;
            V3::new(
                x,
                if flat {
                    0.22
                } else {
                    ground(x, z, seed, false).max(0.12) + 0.22
                },
                z,
            )
        } else {
            let n = &self.nodes[0];
            n.p.sub(n.rest)
        };
        let heading = self.orientation;
        self.nodes.truncate(count);
        for i in self.nodes.len()..count {
            let (rest, _) = bud(i, d);
            let rest = V3::new(
                rest.x * heading.cos() - rest.z * heading.sin(),
                rest.y,
                rest.x * heading.sin() + rest.z * heading.cos(),
            );
            self.nodes.push(Cell {
                p: base.add(rest),
                v: V3::default(),
                rest,
                radius: 0.19,
                role: 0,
                contact: false,
                health: 1.,
                reserve: 0.,
                signal: [0.; 3],
            });
        }
        let previous = std::mem::take(&mut self.links);
        let specialized = if count >= 4 { g[SPECIALIZATION] } else { 0. };
        let support = g[SKELETON] * specialized > 0.16;
        let contract = g[MUSCLE] * specialized > 0.12;
        for (i, n) in self.nodes.iter_mut().enumerate() {
            if self.regulated {
                continue;
            }
            n.role = if support && (i < 4 || i % 3 == 0) {
                1
            } else if contract && i % 3 == 1 {
                2
            } else if g[SENSING] * specialized > 0.2 && i % 3 == 2 {
                3
            } else {
                0
            };
        }
        // Core braces are load-bearing only after structural differentiation.
        for i in 1..count.min(4) {
            for j in 0..i {
                self.link(j, i, if support { 1 } else { 0 });
            }
        }
        for i in 4..count {
            let (_, parent) = bud(i, d);
            self.link(parent, i, if support { 1 } else { 0 });
            // Angular joints replace the earlier remote-anchor distance braces.
            if self.anatomy.is_some() {
                continue;
            }
            // A second distance controls joint flexion without pinning orientation.
            let anchor = (parent + 1) % 4;
            self.link(anchor, i, if contract { 2 } else { 3 });
            self.links.last_mut().unwrap().purpose = 1;
            // Passive elasticity restores joint posture when the pulling muscle
            // relaxes; a muscle cannot extend a limb by pushing its endpoints.
            if contract || self.regulated {
                self.link(anchor, i, 3);
            }
            if i >= 6 && d[11] > 0.5 {
                self.link(i - 2, i, if contract { 2 } else { 3 });
                self.links.last_mut().unwrap().purpose = 1;
            }
        }
        // Contracting diagonals permit a soft, unbranched colony to crawl too.
        if self.anatomy.is_none() && (contract || self.regulated) && count >= 4 {
            self.link(0, 3, 2);
            self.link(1, 2, 2);
        }
        if self.regulated {
            for l in &mut self.links {
                if let Some(old) = previous
                    .iter()
                    .find(|old| old.a == l.a && old.b == l.b && old.purpose == l.purpose)
                {
                    l.integrity = old.integrity;
                }
            }
            self.refresh_tissue_links();
        }
        self.sync_joints(d);
    }
    fn link(&mut self, a: usize, b: usize, kind: u8) {
        if a == b {
            return;
        }
        let rest = self.nodes[a].rest.sub(self.nodes[b].rest).len().max(0.12);
        self.links.push(Link {
            a,
            b,
            rest,
            target: rest,
            kind,
            activation: 0.,
            strain: 0.,
            integrity: 1.,
            purpose: if kind == 2 {
                1
            } else if kind == 3 {
                2
            } else {
                0
            },
        });
    }
    pub fn advance(
        &mut self,
        dt: f32,
        g: &[f32; GENE_COUNT],
        d: &[f32; DEVELOPMENT],
        seed: u32,
        flat: bool,
        gradient: (f32, f32),
        external: (f32, f32),
    ) -> f32 {
        let substeps = (dt * 60.).round().max(1.) as usize;
        let h = dt / substeps as f32;
        let start = self.center();
        let mut spent = 0.;
        self.max_strain = 0.;
        let strength = if self.nodes.len() >= 4 {
            g[MUSCLE] * g[SPECIALIZATION]
        } else {
            0.
        };
        let support = if self.nodes.len() >= 4 {
            g[SKELETON]
                * g[SPECIALIZATION]
                * if self.regulated {
                    (self
                        .nodes
                        .iter()
                        .filter(|n| n.role == 1 && n.health > 0.4)
                        .count() as f32
                        / self.nodes.len() as f32
                        * 3.)
                        .min(1.)
                } else {
                    1.
                }
        } else {
            0.
        };
        let mut old = vec![V3::default(); self.nodes.len()];
        let mut lambdas = vec![0.; self.links.len()];
        let mass = self.nodes.len().max(1) as f32;
        let mut target_change = vec![0.; self.links.len()];
        let mut joint_lambdas = vec![[0.; 4]; self.joints.len()];
        for _ in 0..substeps {
            self.clock += h;
            for (i, n) in self.nodes.iter_mut().enumerate() {
                old[i] = n.p;
                let submerged = if flat {
                    0.
                } else {
                    ((0.12 - (n.p.y - n.radius)) / (2. * n.radius)).clamp(0., 1.)
                };
                let density = 0.78 + support * 0.42;
                n.v.y -= 9.81 * (1. - submerged / density) * h;
                n.v.x += external.0 * h / mass;
                n.v.z += external.1 * h / mass;
                n.v = n.v.mul((-h * (0.35 + submerged * 4.)).exp());
                n.p = n.p.add(n.v.mul(h));
            }
            lambdas.fill(0.);
            joint_lambdas.fill([0.; 4]);
            self.control_joints(h, g, d, gradient);
            for (k, l) in self.links.iter_mut().enumerate() {
                target_change[k] = 0.;
                if l.kind == 2 {
                    let a = &self.nodes[l.a];
                    let b = &self.nodes[l.b];
                    let side = if b.rest.z > a.rest.z { 1. } else { -1. };
                    let phase = self.clock * (0.5 + d[5] * 2.5) * std::f32::consts::TAU
                        + l.b as f32 * d[6] * 4.
                        + side * d[10] * std::f32::consts::PI;
                    let feedback = (if b.contact { 1. } else { 0. }
                        - if a.contact { 1. } else { 0. })
                        * (d[9] - 0.5)
                        * 1.4;
                    let toward = b.p.sub(a.p).normalized();
                    let sensing = (gradient.0 * toward.x + gradient.1 * toward.z)
                        * g[SENSING]
                        * (d[11] - 0.5)
                        * 1.6;
                    l.activation =
                        (phase.sin() * 0.5 + 0.5 + (d[7] - 0.5) * 0.7 + feedback + sensing)
                            .clamp(0., 1.);
                    let target = l.rest * (1. - strength * 0.34 * l.activation);
                    target_change[k] = (target - l.target).abs();
                    l.target = target;
                } else {
                    l.target = l.rest;
                    l.activation = 0.;
                }
            }
            // XPBD: accumulated lambda is reset for each substep, retained per iteration.
            for _ in 0..if !self.joints.is_empty() { 8 } else { 4 } {
                for (k, l) in self.links.iter().enumerate() {
                    if l.integrity <= 0.08 {
                        continue;
                    }
                    let delta = self.nodes[l.b].p.sub(self.nodes[l.a].p);
                    let distance = delta.len().max(1e-6);
                    let compliance = match l.kind {
                        1 if self.anatomy.is_some() => {
                            0.00000002 + (1. - self.anatomy.as_ref().unwrap().traits[3]) * 0.000001
                        }
                        1 => 0.000003 + (1. - support) * 0.00004,
                        2 => 0.00004 + (1. - strength) * 0.0004,
                        3 => 0.002,
                        _ => 0.0003 + (1. - g[ADHESION]) * 0.003,
                    };
                    let health = if self.regulated {
                        self.nodes[l.a].health.min(self.nodes[l.b].health) * l.integrity
                    } else {
                        1.
                    };
                    let alpha = compliance / (h * h * health.max(0.02));
                    let proposed = (-(distance - l.target) - alpha * lambdas[k]) / (2. + alpha);
                    let next = if l.kind == 2 {
                        // Muscle pulls, never actively pushes, and has finite force.
                        let supply = if self.regulated {
                            (self.nodes[l.a].reserve.min(self.nodes[l.b].reserve) / 0.25)
                                .clamp(0., 1.)
                                * health
                        } else {
                            1.
                        };
                        let capacity = (8. + strength * 90.) * h * h * supply;
                        (lambdas[k] + proposed).clamp(-capacity, 0.)
                    } else {
                        lambdas[k] + proposed
                    };
                    let dl = next - lambdas[k];
                    lambdas[k] = next;
                    let correction = delta.mul(dl / distance);
                    self.nodes[l.a].p = self.nodes[l.a].p.sub(correction);
                    self.nodes[l.b].p = self.nodes[l.b].p.add(correction);
                }
                self.solve_joints(h, g, &mut joint_lambdas);
                // Non-neighbor cells cannot occupy the same volume.
                for i in 0..self.nodes.len() {
                    for j in 0..i {
                        let delta = self.nodes[i].p.sub(self.nodes[j].p);
                        let distance = delta.len();
                        let limit = self.nodes[i].radius + self.nodes[j].radius;
                        if distance < limit && distance > 1e-6 {
                            let correction = delta.mul((limit - distance) * 0.5 / distance);
                            self.nodes[i].p = self.nodes[i].p.add(correction);
                            self.nodes[j].p = self.nodes[j].p.sub(correction);
                        }
                    }
                }
                for (i, n) in self.nodes.iter_mut().enumerate() {
                    let floor = ground(n.p.x, n.p.z, seed, flat);
                    let penetration = floor + n.radius - n.p.y;
                    if penetration > 0. {
                        let dx = (ground(n.p.x + 0.03, n.p.z, seed, flat)
                            - ground(n.p.x - 0.03, n.p.z, seed, flat))
                            / 0.06;
                        let dz = (ground(n.p.x, n.p.z + 0.03, seed, flat)
                            - ground(n.p.x, n.p.z - 0.03, seed, flat))
                            / 0.06;
                        let normal = V3::new(-dx, 1., -dz).normalized();
                        let correction = penetration / normal.y.max(0.1);
                        n.p = n.p.add(normal.mul(correction));
                        let movement = n.p.sub(old[i]);
                        let tangent = movement.sub(normal.mul(movement.dot(normal)));
                        let friction = 0.15 + d[8] * 1.1;
                        let fraction = (friction * correction / tangent.len().max(1e-6)).min(1.);
                        n.p = n.p.sub(tangent.mul(fraction));
                    }
                    let radius = n.p.x.hypot(n.p.z);
                    if radius > 29.5 {
                        n.p.x *= 29.5 / radius;
                        n.p.z *= 29.5 / radius;
                    }
                }
            }
            // Absolute actuator work estimated from XPBD constraint force and
            // commanded length change. The scale maps mechanics to model energy.
            for (k, l) in self.links.iter().enumerate() {
                if l.kind == 2 {
                    spent += lambdas[k].abs() / (h * h) * target_change[k] * 0.001;
                }
            }
            spent += self.joint_work(h, g, &joint_lambdas);
            self.contacts = 0;
            for (i, n) in self.nodes.iter_mut().enumerate() {
                let touching = n.p.y - ground(n.p.x, n.p.z, seed, flat) - n.radius < 0.025;
                if touching {
                    self.contacts += 1;
                    if !n.contact && i >= 4 {
                        self.steps += 1;
                    }
                }
                n.contact = touching;
                n.v = n.p.sub(old[i]).mul(1. / h);
            }
            // Dampen excessive relative velocity uniformly, preserving the body's
            // center-of-mass momentum (per-cell caps would create artificial thrust).
            let mean = self
                .nodes
                .iter()
                .fold(V3::default(), |sum, n| sum.add(n.v))
                .mul(1. / mass);
            let maximum = self
                .nodes
                .iter()
                .map(|n| n.v.sub(mean).len())
                .fold(0.0_f32, f32::max);
            if maximum > 12. {
                for n in &mut self.nodes {
                    n.v = mean.add(n.v.sub(mean).mul(12. / maximum));
                }
            }

            for l in &mut self.links {
                l.strain = (self.nodes[l.a].p.sub(self.nodes[l.b].p).len() / l.target - 1.).abs();
                self.max_strain = self.max_strain.max(l.strain);
                if self.regulated && l.strain > 0.45 {
                    let damage = (l.strain - 0.45).min(2.) * h * 0.08;
                    l.integrity = (l.integrity - damage).max(0.);
                    self.nodes[l.a].health = (self.nodes[l.a].health - damage * 0.15).max(0.05);
                    self.nodes[l.b].health = (self.nodes[l.b].health - damage * 0.15).max(0.05);
                }
            }
        }
        let end = self.center();
        self.path += end.sub(start).x.hypot(end.sub(start).z);
        self.work += spent;
        spent
    }
    pub fn valid(&self, count: usize) -> bool {
        (self.nodes.is_empty() || self.nodes.len() == count)
            && self.joints_valid(count)
            && self.nodes.len() <= crate::tissue::MAX_BODY_CELLS
            && self.links.len() <= 256
            && self.orientation.is_finite()
            && [
                self.clock,
                self.work,
                self.path,
                self.max_strain,
                self.repair_work,
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.)
            && self.nodes.iter().all(|n| {
                n.p.finite()
                    && n.v.finite()
                    && n.rest.finite()
                    && n.v.len() < 100.
                    && n.radius.is_finite()
                    && (0.1..=0.5).contains(&n.radius)
                    && n.role <= 5
                    && n.health.is_finite()
                    && (0.0..=1.).contains(&n.health)
                    && n.reserve.is_finite()
                    && (0.0..=100000.).contains(&n.reserve)
                    && n.signal
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=1.).contains(v))
            })
            && self.links.iter().all(|l| {
                l.a < count
                    && l.b < count
                    && l.a != l.b
                    && l.kind <= 3
                    && l.purpose <= 2
                    && l.integrity.is_finite()
                    && (0.0..=1.).contains(&l.integrity)
                    && [l.rest, l.target, l.activation, l.strain]
                        .iter()
                        .all(|v| v.is_finite())
                    && (0.05..=15.).contains(&l.rest)
                    && (0.03..=15.).contains(&l.target)
                    && (0.0..=1.).contains(&l.activation)
                    && l.strain >= 0.
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn genes() -> [f32; GENE_COUNT] {
        let mut g = [0.5; GENE_COUNT];
        g[MUSCLE] = 0.9;
        g[SKELETON] = 0.9;
        g[SPECIALIZATION] = 0.9;
        g[ADHESION] = 0.9;
        g
    }
    #[test]
    fn gravity_and_support_hold_a_developing_body() {
        let g = genes();
        let d = default_development();
        let mut b = Body::default();
        for n in 1..=12 {
            b.sync(n, &g, &d, 0., 0., 0., 8042, true);
            b.advance(0.5, &g, &d, 8042, true, (0., 0.), (0., 0.));
        }
        for _ in 0..40 {
            b.advance(0.25, &g, &d, 8042, true, (0., 0.), (0., 0.));
        }
        assert!(b.valid(12));
        assert!(b.contacts > 0);
        assert!(b.nodes.iter().all(|n| n.p.y >= n.radius - 0.03));
        assert!(b.links.iter().any(|l| l.kind == 1));
        assert!(b.links.iter().any(|l| l.kind == 2));
        assert!(b.work > 0.);
        assert!(b.steps > 0);
    }
    #[test]
    fn internal_forces_do_not_translate_a_body_in_free_space() {
        let g = genes();
        let d = default_development();
        let mut b = Body::default();
        b.sync(12, &g, &d, 0., 0., 0., 0, true);
        for n in &mut b.nodes {
            n.p.y += 100.;
        }
        let start = b.center();
        b.advance(0.5, &g, &d, 0, true, (0., 0.), (0., 0.));
        let end = b.center();
        assert!((end.x - start.x).abs() < 0.0001);
        assert!((end.z - start.z).abs() < 0.0001);
    }
    #[test]
    fn sensed_direction_changes_motors_without_external_thrust() {
        let g = genes();
        let mut d = default_development();
        d[11] = 1.;
        let mut a = Body::default();
        a.sync(12, &g, &d, 0., 0., 0., 0, true);
        for n in &mut a.nodes {
            n.p.y += 100.;
        }
        let mut b = a.clone();
        let start = a.center();
        a.advance(0.25, &g, &d, 0, true, (1., 0.), (0., 0.));
        b.advance(0.25, &g, &d, 0, true, (-1., 0.), (0., 0.));
        assert!(a
            .links
            .iter()
            .zip(&b.links)
            .any(|(x, y)| (x.activation - y.activation).abs() > 0.01));
        for body in [a, b] {
            assert!((body.center().x - start.x).abs() < 0.0001);
            assert!((body.center().z - start.z).abs() < 0.0001);
        }
    }
    #[test]
    fn motor_activity_consumes_work_and_moves_contacts() {
        let mut g = genes();
        let d = default_development();
        let mut active = Body::default();
        active.sync(12, &g, &d, 0., 0., 0., 0, true);
        let mut passive = active.clone();
        let mut off = g;
        off[MUSCLE] = 0.;
        for _ in 0..80 {
            active.advance(0.25, &g, &d, 0, true, (0., 0.), (0., 0.));
            passive.advance(0.25, &off, &d, 0, true, (0., 0.), (0., 0.));
        }
        assert!(active.work > 1.);
        assert_eq!(passive.work, 0.);
        assert!(
            active.path > passive.path + 0.05,
            "active {} passive {}",
            active.path,
            passive.path
        );
        g[SKELETON] = 0.;
        let mut soft = Body::default();
        soft.sync(12, &g, &d, 0., 0., 0., 0, true);
        assert!(!soft.links.iter().any(|l| l.kind == 1));
    }
}
