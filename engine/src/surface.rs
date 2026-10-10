//! A reduced viscous shallow surface layer on the spherical mesh.
//! Gravity acts on free-surface height; bounded finite-volume transfers carry
//! existing water and all solutes across longitude and both polar caps.
//! This is not a turbulent 3D ocean or a calibrated terrestrial climate.
use crate::{body::V3, domain::Domain, planet::Planet, *};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SurfaceFlow {
    pub velocities: Vec<V3>,
    pub water_transported: f64,
}
impl SurfaceFlow {
    pub fn new(d: Domain) -> Self {
        Self {
            velocities: vec![V3::default(); d.sites()],
            water_transported: 0.,
        }
    }
    pub fn valid(&self, d: Domain) -> bool {
        self.velocities.len() == d.sites()
            && self
                .velocities
                .iter()
                .all(|v| [v.x, v.y, v.z].iter().all(|x| x.is_finite()) && v.len() < 20.)
            && self.water_transported.is_finite()
            && self.water_transported >= 0.
    }
    pub fn step(&mut self, d: Domain, p: &mut Planet, solutes: &mut [[f64; 6]], seed: u32) {
        let edges = d.edges();
        let count = d.sites();
        let areas: Vec<_> = (0..count).map(|i| d.area(i)).collect();
        let normals: Vec<_> = (0..count)
            .map(|i| {
                let (x, z) = d.point(i);
                d.normal(x, z)
            })
            .collect();
        let ground: Vec<_> = (0..count)
            .map(|i| {
                let (x, z) = d.point(i);
                d.ground(x, z, seed) as f64 + p.relief[i] as f64
            })
            .collect();
        let geometry: Vec<_> = edges
            .iter()
            .map(|&(i, j)| {
                let delta = normals[j].sub(normals[i]);
                let a = delta.sub(normals[i].mul(delta.dot(normals[i])));
                let b = delta.sub(normals[j].mul(delta.dot(normals[j])));
                (
                    i,
                    j,
                    a.mul(1. / a.len().max(1e-8)),
                    b.mul(1. / b.len().max(1e-8)),
                    d.distance(i, j),
                )
            })
            .collect();
        // Small physical substeps, with conservative donor limiting at tiny polar
        // cells. The limiter reduces accuracy at extreme Courant numbers, not mass.
        let dt = DT as f64 / 16.;
        for _ in 0..16 {
            let old = self.velocities.clone();
            let mut change = vec![V3::default(); count];
            for &(i, j, a, b, length) in &geometry {
                let hi = ground[i] + p.water[i] / areas[i];
                let hj = ground[j] + p.water[j] / areas[j];
                let acceleration = (0.3 * (hi - hj) / length * dt / 4.) as f32;
                change[i] = change[i].add(a.mul(acceleration));
                change[j] = change[j].add(b.mul(acceleration));
                let viscosity = (0.06 * dt / (length * length)).min(0.04) as f32;
                let dv = old[j].sub(old[i]).mul(viscosity);
                change[i] = change[i].add(dv);
                change[j] = change[j].sub(dv);
            }
            for i in 0..count {
                let v = old[i].add(change[i]);
                let tangent = v.sub(normals[i].mul(v.dot(normals[i])));
                self.velocities[i] = if p.water[i] < 1e-12 {
                    V3::default()
                } else {
                    tangent.mul((-0.01 * dt).exp() as f32)
                };
            }
            // Each edge transfer also transports its donor's material and momentum.
            for &(i, j, a, b, _) in &geometry {
                let speed = (self.velocities[i].dot(a) + self.velocities[j].dot(b)) as f64 * 0.5;
                let (donor, receiver) = if speed >= 0. { (i, j) } else { (j, i) };
                let width = areas[i].min(areas[j]).sqrt();
                let amount = (speed.abs() * dt * width * p.water[donor] / areas[donor])
                    .min(p.water[donor] * 0.25);
                if amount <= 0. {
                    continue;
                }
                let fraction = amount / p.water[donor];
                let existing = p.water[receiver];
                self.velocities[receiver] = self.velocities[receiver]
                    .mul((existing / (existing + amount)) as f32)
                    .add(self.velocities[donor].mul((amount / (existing + amount)) as f32));
                p.water[donor] -= amount;
                p.water[receiver] += amount;
                self.water_transported += amount;
                p.runoff += amount;
                for k in 0..6 {
                    let carried = solutes[donor][k] * fraction;
                    solutes[donor][k] -= carried;
                    solutes[receiver][k] += carried;
                }
            }
            for i in 0..count {
                let v = self.velocities[i];
                self.velocities[i] = v.sub(normals[i].mul(v.dot(normals[i])));
            }
        }
    }
    pub fn diffuse(d: Domain, solutes: &mut [[f64; 6]], dt: f64, diffusivity: [f64; 6]) {
        let edges = d.edges();
        let areas: Vec<_> = (0..d.sites()).map(|i| d.area(i)).collect();
        let geometry: Vec<_> = edges
            .into_iter()
            .map(|(i, j)| (i, j, d.distance(i, j)))
            .collect();
        for _ in 0..4 {
            for &(i, j, length) in &geometry {
                for k in 0..6 {
                    let flux = dt / 4. * diffusivity[k] * areas[i].min(areas[j])
                        / (length * length)
                        * (solutes[i][k] / areas[i] - solutes[j][k] / areas[j]);
                    let (a, b) = if flux >= 0. { (i, j) } else { (j, i) };
                    let amount = flux.abs().min(solutes[a][k] * 0.25);
                    solutes[a][k] -= amount;
                    solutes[b][k] += amount;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Domain, Planet, Vec<[f64; 6]>, SurfaceFlow) {
        let mut d = Domain::from_size(128);
        d.shape = crate::domain::Shape::Sphere;
        let mut p = Planet::new_in(d);
        p.initialize_surface(8042);
        for i in 0..d.sites() {
            let (x, z) = d.point(i);
            p.water[i] = (4. - d.ground(x, z, 8042) as f64) * d.area(i);
        }
        p.initial_water = p.water.iter().sum::<f64>() + p.atmosphere;
        (d, p, vec![[0.; 6]; d.sites()], SurfaceFlow::new(d))
    }
    #[test]
    fn water_advection_carries_all_species_across_longitude() {
        let (d, mut p, mut s, mut f) = fixture();
        let a = d.rows() / 2 * d.grid + d.grid - 1;
        let b = d.rows() / 2 * d.grid;
        s[a] = [1., 2., 3., 4., 5., 6.];
        let initial = s[a];
        for i in 0..d.sites() {
            let (x, z) = d.point(i);
            f.velocities[i] = d.tangent(d.position(x, z, 0.), 0.).mul(0.2);
        }
        f.step(d, &mut p, &mut s, 8042);
        assert!(s[b].iter().all(|v| *v > 0.));
        assert!(p.water_error().abs() < 1e-7 && f.valid(d));
        for k in 0..6 {
            assert!((s.iter().map(|s| s[k]).sum::<f64>() - initial[k]).abs() < 1e-10);
        }
    }
    #[test]
    fn polar_diffusion_is_connected_positive_and_conservative() {
        let (d, _, mut s, _) = fixture();
        s[0] = [1.; 6];
        let opposite = d.grid / 2;
        SurfaceFlow::diffuse(d, &mut s, 0.25, [0.12; 6]);
        assert!(s[opposite][0] > 0.);
        assert!(s.iter().flatten().all(|x| *x >= 0. && x.is_finite()));
        for k in 0..6 {
            assert!((s.iter().map(|s| s[k]).sum::<f64>() - 1.).abs() < 1e-10);
        }
    }
    #[test]
    fn hydrostatic_level_has_no_spontaneous_current() {
        let (d, mut p, mut s, mut f) = fixture();
        f.step(d, &mut p, &mut s, 8042);
        assert!(f.velocities.iter().all(|v| v.len() < 1e-6));
        assert!(p.water_error().abs() < 1e-7);
    }
}
