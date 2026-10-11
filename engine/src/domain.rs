//! Geometry in model units. Cells and physical time are never scaled with view.
use crate::body::V3;
use crate::*;
pub const SPACING: f32 = 1.6;
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    #[default]
    Planar,
    Sphere,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Domain {
    pub grid: usize,
    #[serde(default)]
    pub span: f32,
    #[serde(default)]
    pub shape: Shape,
}
struct Fixed {
    domain: Domain,
    bands: Vec<f64>,
    normals: Vec<V3>,
}
thread_local! {
    static FIXED: std::cell::RefCell<Vec<Fixed>> = const { std::cell::RefCell::new(Vec::new()) };
}
impl Default for Domain {
    fn default() -> Self {
        Self {
            grid: 40,
            span: 0.,
            shape: Shape::Planar,
        }
    }
}
impl Domain {
    pub fn from_size(size: u16) -> Self {
        Self {
            grid: (size as usize * 5 / 8).min(160),
            span: if size > 256 { size as f32 } else { 0. },
            ..Self::default()
        }
    }
    pub fn valid(self) -> bool {
        [40, 80, 160].contains(&self.grid)
            && (self.span == 0.
                || (self.grid == 160 && [512., 1024., 8192., 64000.].contains(&self.span)))
    }
    pub fn spherical(self) -> bool {
        self.shape == Shape::Sphere
    }
    pub fn rows(self) -> usize {
        if self.spherical() {
            self.grid / 2
        } else {
            self.grid
        }
    }
    pub fn sites(self) -> usize {
        self.grid * self.rows()
    }
    pub fn size(self) -> f32 {
        if self.span > 0. {
            self.span
        } else {
            self.grid as f32 * SPACING
        }
    }
    pub fn spacing(self) -> f32 {
        self.size() / self.grid as f32
    }
    pub fn half(self) -> f32 {
        self.size() * 0.5
    }
    pub fn radius(self) -> f32 {
        self.size() / std::f32::consts::TAU
    }
    pub fn wall(self) -> f32 {
        self.half() - 2.
    }
    pub fn index(self, x: f32, z: f32) -> usize {
        let longitude = if self.spherical() {
            (x + self.half()).rem_euclid(self.size())
        } else {
            x + self.half()
        };
        let ix = (longitude / self.spacing())
            .floor()
            .clamp(0., (self.grid - 1) as f32) as usize;
        let iz = ((z + if self.spherical() {
            self.half() * 0.5
        } else {
            self.half()
        }) / self.spacing())
        .floor()
        .clamp(0., (self.rows() - 1) as f32) as usize;
        iz * self.grid + ix
    }
    pub fn point(self, i: usize) -> (f32, f32) {
        (
            (i % self.grid) as f32 * self.spacing() + self.spacing() * 0.5 - self.half(),
            (i / self.grid) as f32 * self.spacing() + self.spacing() * 0.5
                - if self.spherical() {
                    self.half() * 0.5
                } else {
                    self.half()
                },
        )
    }
    pub fn normal(self, x: f32, z: f32) -> V3 {
        let lon = x / self.radius();
        let lat = z / self.radius();
        V3::new(lon.sin() * lat.cos(), lat.sin(), lon.cos() * lat.cos())
    }
    pub fn coords(self, p: V3) -> (f32, f32) {
        if !self.spherical() {
            return (p.x, p.z);
        }
        let r = p.len().max(1e-9);
        (
            p.x.atan2(p.z) * self.radius(),
            (p.y / r).clamp(-1., 1.).asin() * self.radius(),
        )
    }
    pub fn locate(self, p: V3) -> usize {
        let (x, z) = self.coords(p);
        self.index(x, z)
    }
    pub fn up(self, p: V3) -> V3 {
        if self.spherical() {
            p.mul(1. / p.len().max(1e-9))
        } else {
            V3::new(0., 1., 0.)
        }
    }
    pub fn tangent(self, p: V3, heading: f32) -> V3 {
        if !self.spherical() {
            return V3::new(heading.cos(), 0., heading.sin());
        }
        let (x, z) = self.coords(p);
        self.tangent_at(x, z, heading)
    }
    /// As `tangent`, for a point whose map coordinates are already known.
    pub fn tangent_at(self, x: f32, z: f32, heading: f32) -> V3 {
        if !self.spherical() {
            return V3::new(heading.cos(), 0., heading.sin());
        }
        let lon = x / self.radius();
        let lat = z / self.radius();
        let east = V3::new(lon.cos(), 0., -lon.sin());
        let north = V3::new(-lon.sin() * lat.sin(), lat.cos(), -lon.cos() * lat.sin());
        east.mul(heading.cos()).add(north.mul(heading.sin()))
    }
    pub fn position(self, x: f32, z: f32, height: f32) -> V3 {
        if self.spherical() {
            self.normal(x, z).mul(self.radius() + height)
        } else {
            V3::new(x, height, z)
        }
    }
    pub fn ground(self, x: f32, z: f32, seed: u32) -> f32 {
        if !self.spherical() {
            return crate::body::ground_scaled(x, z, seed, self.half() - 1.);
        }
        let n = self.normal(x, z);
        // Noise on the embedded direction is continuous across longitude and poles.
        crate::body::smooth_noise(n.x * 12. + 20., n.y * 12. + 20., seed) * 4.2
            + crate::body::smooth_noise(n.z * 18. + 20., n.x * 18. + 20., seed.wrapping_add(72))
                * 1.2
            - 2.3
    }
    pub fn area(self, i: usize) -> f64 {
        if !self.spherical() {
            return self.spacing().powi(2) as f64;
        }
        // Every site in a band of latitude has the same area, and this is
        // asked for constantly, so each band's is worked out once.
        let row = i / self.grid;
        self.fixed(|fixed| fixed.bands.get(row).copied())
            .unwrap_or_else(|| self.band(row))
    }
    /// The outward direction at the middle of a field site.
    pub fn site_normal(self, i: usize) -> V3 {
        self.fixed(|fixed| fixed.normals.get(i).copied()).unwrap_or_else(|| {
            let (x, z) = self.point(i);
            self.normal(x, z)
        })
    }
    /// Geometry that depends only on the world's size and shape, worked out
    /// once and read from then on.
    fn fixed<T>(self, read: impl FnOnce(&Fixed) -> T) -> T {
        FIXED.with(|fixed| {
            let mut fixed = fixed.borrow_mut();
            if let Some(hit) = fixed.iter().find(|f| f.domain == self) {
                return read(hit);
            }
            // A session only ever alternates between a planet, its nursery and a lab.
            if fixed.len() >= 4 {
                fixed.remove(0);
            }
            fixed.push(Fixed {
                domain: self,
                bands: (0..self.rows()).map(|row| self.band(row)).collect(),
                normals: (0..self.sites())
                    .map(|i| {
                        let (x, z) = self.point(i);
                        self.normal(x, z)
                    })
                    .collect(),
            });
            read(fixed.last().unwrap())
        })
    }
    fn band(self, row: usize) -> f64 {
        let d = std::f64::consts::TAU / self.grid as f64;
        let low = -std::f64::consts::FRAC_PI_2 + row as f64 * d;
        (self.radius() as f64).powi(2) * d * ((low + d).sin() - low.sin())
    }
    pub fn distance(self, a: usize, b: usize) -> f64 {
        if !self.spherical() {
            let (x, z) = self.point(a);
            let (u, v) = self.point(b);
            return (x - u).hypot(z - v) as f64;
        }
        let (x, z) = self.point(a);
        let (u, v) = self.point(b);
        let r = self.radius() as f64;
        let la = z as f64 / r;
        let lb = v as f64 / r;
        let dl = (u - x) as f64 / r;
        let h = ((lb - la) * 0.5).sin().powi(2) + la.cos() * lb.cos() * (dl * 0.5).sin().powi(2);
        (2. * r * h.clamp(0., 1.).sqrt().asin()).max(0.001)
    }
    pub fn edges(self) -> Vec<(usize, usize)> {
        let mut edges = Vec::new();
        for row in 0..self.rows() {
            for col in 0..self.grid {
                let i = row * self.grid + col;
                if col + 1 < self.grid {
                    edges.push((i, i + 1));
                } else if self.spherical() {
                    edges.push((i, row * self.grid));
                }
                if row + 1 < self.rows() {
                    edges.push((i, i + self.grid));
                }
                // Connect across each polar cap to the opposite longitude. No pole wall.
                if self.spherical() && (row == 0 || row + 1 == self.rows()) && col < self.grid / 2 {
                    edges.push((i, i + self.grid / 2));
                }
            }
        }
        edges
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spherical_geometry_has_no_longitude_wall_and_correct_area() {
        for size in [64, 128, 256] {
            let mut d = Domain::from_size(size);
            d.shape = Shape::Sphere;
            let area: f64 = (0..d.sites()).map(|i| d.area(i)).sum();
            assert!((area - 4. * std::f64::consts::PI * (d.radius() as f64).powi(2)).abs() < 1e-7);
            for i in 0..d.sites() {
                let (x, z) = d.point(i);
                assert_eq!(d.locate(d.position(x, z, 0.)), i);
            }
            assert_eq!(d.index(-d.half() + 0.1, 0.), d.index(d.half() + 0.1, 0.));
            assert!((d.ground(-d.half(), 0., 8042) - d.ground(d.half(), 0., 8042)).abs() < 1e-4);
            let row = d.rows() / 2;
            assert!(d
                .edges()
                .contains(&(row * d.grid + d.grid - 1, row * d.grid)));
            assert!(d.edges().contains(&(0, d.grid / 2)));
        }
    }
    #[test]
    fn larger_planar_worlds_keep_cell_and_lattice_scale() {
        let mut counts = Vec::new();
        for size in [64, 128, 256] {
            let d = Domain::from_size(size);
            assert_eq!(d.size(), size as f32);
            assert!((d.point(1).0 - d.point(0).0 - SPACING).abs() < 0.00002);
            counts.push(d.sites());
        }
        assert_eq!(counts, vec![1600, 6400, 25600]);
    }
}
