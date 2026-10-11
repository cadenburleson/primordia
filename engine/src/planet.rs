//! Seasonal forcing and a closed effective water cycle on the habitat grid.
//! Runoff conserves solutes; volcanism is an explicitly tracked mineral input.
//! These are reduced model processes, not a climate/geology forecast.
use crate::*;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Planet {
    #[serde(default)]
    pub domain: crate::domain::Domain,
    pub water: Vec<f64>,
    pub snow: Vec<f64>,
    pub relief: Vec<f32>,
    pub heat: Vec<f32>,
    pub atmosphere: f64,
    pub initial_water: f64,
    #[serde(default)]
    pub biological_water: f64,
    #[serde(default)]
    pub cloud_cover: f32,
    #[serde(default = "default_season_strength")]
    pub season_strength: f32,
    #[serde(default = "default_season_period")]
    pub season_period: f32,
    pub sunlight: f32,
    #[serde(default = "default_sun_direction")]
    pub sun_direction: [f32; 3],
    pub temperature: f32,
    pub storm: f32,
    #[serde(default = "default_storm_radius")]
    pub storm_radius: f32,
    pub storm_center: [f32; 2],
    pub precipitation: f64,
    pub evaporation: f64,
    pub snowmelt: f64,
    pub runoff: f64,
    pub eruptions: u32,
    pub mineral_input: f64,
    pub terrain_version: u32,
    /// Standing vent fields on the sea bed: how strongly each site pours out
    /// hydrogen from the hot rock under it, from none to one. Empty in
    /// worlds saved before vents were permanent.
    #[serde(default)]
    pub vents: Vec<f32>,
}

fn default_season_strength() -> f32 { 1. }
fn default_season_period() -> f32 { 240. }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rain_snow_melt_and_runoff_preserve_water_and_solutes() {
        let mut core = Core::new(Config {
            temperature: 0.1,
            ..Config::default()
        })
        .unwrap();
        let mut p = Planet::new();
        p.storm = 6.;
        p.storm_center = [0., 0.];
        let mut solutes = vec![[0.; crate::physical::SOLUTES]; GRID * GRID];
        solutes[20 * GRID + 20] = [1., 0.5, 0.2, 0.1, 0.1, 0.3, 0.2, 0.1, 0.05, 0.4, 0.3, 0.2];
        let initial: [f64; crate::physical::SOLUTES] = std::array::from_fn(|k| solutes.iter().map(|s| s[k]).sum());
        // Disable only the volcanic random hazard by keeping this short and
        // tracking its explicit mineral input, should an eruption still occur.
        for _ in 0..12 {
            core.time += DT;
            p.tick(&mut core, &mut solutes);
        }
        assert!(p.snow.iter().sum::<f64>() > 0. && p.precipitation > 0.);
        core.config.temperature = 0.8;
        for _ in 0..40 {
            core.time += DT;
            p.tick(&mut core, &mut solutes);
        }
        assert!(p.snowmelt > 0. && p.evaporation > 0. && p.runoff > 0.);
        assert!(p.valid() && p.water_error().abs() < 1e-8);
        for k in 0..crate::physical::SOLUTES {
            let expected = initial[k] + if k == 2 { p.mineral_input } else { 0. };
            assert!((solutes.iter().map(|s| s[k]).sum::<f64>() - expected).abs() < 1e-9);
        }
    }
    #[test]
    fn volcanism_changes_real_ground_and_tracks_new_minerals() {
        let mut core = Core::new(Config::default()).unwrap();
        core.start_physical();
        let mut e = core.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        let mut p = h.planet.take().unwrap();
        let before = h.totals(&e.cells);
        p.erupt(&mut core, &mut h.solutes);
        let site = p
            .relief
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap()
            .0;
        let (x, z) = (
            (site % GRID) as f32 * 1.6 - 31.2,
            (site / GRID) as f32 * 1.6 - 31.2,
        );
        assert!(p.relief_at(x, z) > 0. && p.heat[site] > 0. && p.mineral_input > 0.);
        // The raised ground is the floor a body rests on.
        assert!(
            p.floor(x, z, 0.2, core.config.seed)
                > body::ground(x, z, core.config.seed, false) + 0.2
        );
        let input = p.mineral_input;
        h.planet = Some(p);
        assert!((h.totals(&e.cells).1 - before.1 - input).abs() < 1e-9);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
        e.physical = Some(h);
        core.autonomous = Some(e);
        core.step(2);
        assert!(core.autonomous.as_ref().unwrap().valid());
    }
    #[test]
    fn day_and_season_change_environment_not_genes() {
        let mut core = Core::new(Config::default()).unwrap();
        core.start_physical();
        let mut e = core.autonomous.take().unwrap();
        let genes = e.cells[0].genome.clone();
        let h = e.physical.as_mut().unwrap();
        let p = h.planet.as_mut().unwrap();
        core.time = 0.;
        p.tick(&mut core, &mut h.solutes);
        let noon = p.sunlight;
        core.time = 24.;
        p.tick(&mut core, &mut h.solutes);
        assert!(p.sunlight < noon);
        core.time = 60.;
        p.tick(&mut core, &mut h.solutes);
        let warm = p.temperature;
        core.time = 180.;
        p.tick(&mut core, &mut h.solutes);
        assert!(p.temperature < warm);
        assert_eq!(genes, e.cells[0].genome);
    }
}
/// Water the air holds over each unit of surface before it starts to rain out.
/// Height a new planet's seas are filled to. About seven tenths of its
/// surface lies below this, as on Earth. Life is made of water and cannot
/// live out of it without skin, support and a way to carry water.
const SEA_LEVEL: f64 = 0.95;
/// How many times deeper a new planet's sea bed lies than its bare ground would put it.
const BASIN: f64 = 3.;
const AIR_CAPACITY: f64 = 0.2;
/// Share of the air's excess water that falls each cycle.
const RAIN_OUT: f64 = 0.07;
fn default_storm_radius() -> f32 {
    14.
}
fn default_sun_direction() -> [f32; 3] {
    [0., 0., 1.]
}
impl Planet {
    pub fn new() -> Self {
        Self::new_in(crate::domain::Domain::default())
    }
    pub fn new_in(domain: crate::domain::Domain) -> Self {
        Self {
            domain,
            water: vec![0.15; domain.sites()],
            snow: vec![0.; domain.sites()],
            relief: vec![0.; domain.sites()],
            heat: vec![0.; domain.sites()],
            atmosphere: 0.2 * domain.sites() as f64,
            initial_water: 0.2 * domain.sites() as f64 + 0.15 * (domain.sites()) as f64,
            biological_water: 0.,
            cloud_cover: 0.,
            season_strength: 1.,
            season_period: 240.,
            sunlight: 1.,
            sun_direction: default_sun_direction(),
            temperature: 0.48,
            storm: 0.,
            storm_radius: 14.,
            storm_center: [0.; 2],
            precipitation: 0.,
            evaporation: 0.,
            snowmelt: 0.,
            runoff: 0.,
            eruptions: 0,
            mineral_input: 0.,
            terrain_version: 0,
            vents: vec![0.; domain.sites()],
        }
    }
    /// Lay out the planet's vent fields: narrow lines across the sea bed,
    /// as the ridges are where a young planet's crust pulls apart. They
    /// stay where they are; eruptions come and go on top of them.
    pub fn seed_vents(&mut self, seed: u32) {
        self.vents = (0..self.domain.sites())
            .map(|i| {
                let (x, z) = self.domain.point(i);
                let wave = if self.domain.spherical() {
                    let n = self.domain.site_normal(i);
                    crate::body::smooth_noise(n.x * 5. + n.y * 3. + 40., n.z * 5. - n.y * 3. + 40., seed.wrapping_add(911))
                } else {
                    crate::body::smooth_noise(x * 0.12 + 40., z * 0.12 + 40., seed.wrapping_add(911))
                };
                // A ridge runs where the noise crosses its middle value.
                let ridge = 1. - (2. * wave - 1.).abs();
                let under_water = !self.domain.spherical() || self.surface_depth(i) > 1.;
                if under_water { ((ridge - 0.9) / 0.1).clamp(0., 1.) } else { 0. }
            })
            .collect();
    }
    pub fn initialize_surface(&mut self, seed: u32) {
        if !self.domain.spherical() {
            return;
        }
        for i in 0..self.domain.sites() {
            let (x, z) = self.domain.point(i);
            // Ocean basins are cut deep: three times as far below the sea as
            // the bare ground lies, so that a body many cells across has
            // water to swim in and is not aground wherever it goes.
            let below = (SEA_LEVEL - self.domain.ground(x, z, seed) as f64).max(0.);
            self.relief[i] = (-(BASIN - 1.) * below).max(-8.) as f32;
            self.water[i] = (below - self.relief[i] as f64).max(0.002) * self.domain.area(i);
        }
        self.atmosphere = (0..self.domain.sites())
            .map(|i| self.domain.area(i) * 0.2)
            .sum();
        self.initial_water = self.water.iter().sum::<f64>() + self.atmosphere;
    }
    pub fn surface_depth(&self, site: usize) -> f64 {
        if self.domain.spherical() {
            self.water[site] / self.domain.area(site)
        } else {
            if self.water[site] > 0. {
                0.12
            } else {
                0.
            }
        }
    }
    pub(crate) fn distance_from(&self, i: usize, center: [f32; 2]) -> f32 {
        let (x, z) = self.domain.point(i);
        if !self.domain.spherical() {
            return (x - center[0]).hypot(z - center[1]);
        }
        self.domain
            .normal(x, z)
            .dot(self.domain.normal(center[0], center[1]))
            .clamp(-1., 1.)
            .acos()
            * self.domain.radius()
    }
    pub fn water_error(&self) -> f64 {
        self.water.iter().sum::<f64>()
            + self.snow.iter().sum::<f64>()
            + self.atmosphere
            + self.biological_water
            - self.initial_water
    }
    /// How strongly the bed at a site vents, from none to one.
    pub fn vent(&self, site: usize) -> f64 {
        self.vents.get(site).copied().unwrap_or(0.) as f64
    }
    pub fn valid(&self) -> bool {
        self.season_strength.is_finite() && (0.0..=2.).contains(&self.season_strength)
            && self.season_period.is_finite() && (48.0..=2400.).contains(&self.season_period)
            && self.storm_radius.is_finite()
            && self.storm_radius > 0.
            && self.storm_radius <= self.domain.size()
            && self.cloud_cover.is_finite()
            && (0.0..=1.).contains(&self.cloud_cover)
            && self.domain.valid()
            && self.water.len() == self.domain.sites()
            && self.snow.len() == self.domain.sites()
            && self.relief.len() == self.domain.sites()
            && self.heat.len() == self.domain.sites()
            && self
                .water
                .iter()
                .chain(&self.snow)
                .chain([
                    &self.atmosphere,
                    &self.biological_water,
                    &self.initial_water,
                    &self.precipitation,
                    &self.evaporation,
                    &self.snowmelt,
                    &self.runoff,
                    &self.mineral_input,
                ])
                .all(|x| x.is_finite() && *x >= 0. && *x < 1e12)
            && self
                .relief
                .iter()
                .all(|x| x.is_finite() && (-8.0..=8.).contains(x))
            && self
                .heat
                .iter()
                .all(|x| x.is_finite() && (0.0..=1.).contains(x))
            && [
                self.sunlight,
                self.temperature,
                self.storm,
                self.storm_center[0],
                self.storm_center[1],
            ]
            .iter()
            .all(|x| x.is_finite())
            && self
                .sun_direction
                .iter()
                .all(|x| x.is_finite() && x.abs() <= 1.)
            && (0.0..=2.).contains(&self.sunlight)
            && (0.0..=1.).contains(&self.temperature)
            && (0.0..=20.).contains(&self.storm)
            && self
                .storm_center
                .iter()
                .all(|x| x.abs() <= self.domain.half())
            && self.water_error().abs() < 1e-7 * (1. + self.initial_water)
    }
    pub fn relief_at(&self, x: f32, z: f32) -> f32 {
        let gx = (x + self.domain.half()) / self.domain.spacing() - 0.5;
        let gz = (z + if self.domain.spherical() {
            self.domain.half() * 0.5
        } else {
            self.domain.half()
        }) / self.domain.spacing()
            - 0.5;
        let gx = if self.domain.spherical() {
            gx
        } else {
            gx.clamp(0., (self.domain.grid - 1) as f32)
        };
        let gz = gz.clamp(0., (self.domain.rows() - 1) as f32);
        let wrap = |i: i32| {
            if self.domain.spherical() {
                i.rem_euclid(self.domain.grid as i32) as usize
            } else {
                i.clamp(0, self.domain.grid as i32 - 1) as usize
            }
        };
        let (ix, iz) = (gx.floor() as i32, gz.floor() as usize);
        let (jx, jz) = (wrap(ix + 1), (iz + 1).min(self.domain.rows() - 1));
        let (u, v) = (gx - ix as f32, gz - iz as f32);
        let ix = wrap(ix);
        (self.relief[iz * self.domain.grid + ix] * (1. - u)
            + self.relief[iz * self.domain.grid + jx] * u)
            * (1. - v)
            + (self.relief[jz * self.domain.grid + ix] * (1. - u)
                + self.relief[jz * self.domain.grid + jx] * u)
                * v
    }
    /// The lowest a body of this radius can rest: the sea bed or the land.
    /// Water holds a body up by buoyancy, never by acting as a floor.
    pub fn floor(&self, x: f32, z: f32, r: f32, seed: u32) -> f32 {
        let ground = self.domain.ground(x, z, seed) + self.relief_at(x, z);
        ground + r
    }
    /// Where a new arrival of this radius is set down: just under the surface
    /// where the water is deep enough to cover it, on the bottom elsewhere.
    /// How high the water stands in a site. A sea lies level, whatever the
    /// bed under it does.
    pub fn sea_level(&self, site: usize, seed: u32) -> f32 {
        crate::surface::site_ground(self.domain, seed, site)
            + self.relief[site]
            + self.surface_depth(site) as f32
    }
    /// The depth of water over a point in a site whose bed lies at `ground`:
    /// more over a hollow, less over a rise, none where the bed stands clear.
    /// (A mere film of water does not fill the hollows.)
    pub fn water_over(&self, site: usize, ground: f32, seed: u32) -> f32 {
        (self.sea_level(site, seed) - ground).max(0.) * (self.surface_depth(site) as f32 / 0.1).min(1.)
    }
    pub fn berth(&self, x: f32, z: f32, r: f32, seed: u32) -> f32 {
        let floor = self.floor(x, z, r, seed);
        let depth = if self.domain.spherical() {
            self.water_over(self.domain.index(x, z), floor - r, seed)
        } else {
            0.12 - (floor - r)
        };
        floor.max(floor - r + depth - r - 0.02)
    }
    pub fn illumination(&self, site: usize) -> f64 {
        if !self.domain.spherical() {
            return (self.sunlight * (1. - 0.8 * self.cloud_cover)) as f64;
        }
        let n = self.domain.site_normal(site);
        let sun = body::V3::new(
            self.sun_direction[0],
            self.sun_direction[1],
            self.sun_direction[2],
        );
        (self.sunlight * (1. - 0.8 * self.cloud_cover) * n.dot(sun).max(0.)) as f64
    }
    pub(crate) fn temperature_at(&self, site: usize) -> f32 {
        if !self.domain.spherical() {
            return self.temperature;
        }
        let n = self.domain.site_normal(site);
        (self.temperature - n.y.abs() * 0.15 + self.illumination(site) as f32 * 0.04).clamp(0., 1.)
    }
    pub fn kinetics(&self, site: usize) -> f64 {
        // Effective temperature / moisture response, deliberately uncalibrated.
        let temperature = (self.temperature_at(site) + self.heat[site] * 0.5).min(1.);
        ((0.35 + temperature as f64 * 1.1)
            * (0.45
                + (self.water[site]
                    / if self.domain.spherical() {
                        self.domain.area(site) * 0.2
                    } else {
                        0.2
                    })
                .min(1.)
                    * 0.55))
            .clamp(0.1, 1.5)
    }
    pub(crate) fn erupt(&mut self, core: &mut Core, solutes: &mut [[f64; crate::physical::SOLUTES]]) {
        let center = [
            (core.random() - 0.5) * (self.domain.size() - 20.),
            (core.random() - 0.5)
                * (if self.domain.spherical() {
                    self.domain.half() - 4.
                } else {
                    self.domain.size() - 20.
                }),
        ];
        for i in 0..self.domain.sites() {
            let weight = (1. - self.distance_from(i, center) / 5.).max(0.);
            self.relief[i] = (self.relief[i] + weight * 0.65).min(8.);
            self.heat[i] = (self.heat[i] + weight).min(1.);
            let mineral = weight as f64
                * 0.12
                * if self.domain.spherical() {
                    self.domain.area(i) / 2.56
                } else {
                    1.
                };
            solutes[i][2] += mineral;
            self.mineral_input += mineral;
        }
        self.eruptions += 1;
        self.terrain_version += 1;
        core.event("planet",format!("Volcanic disturbance #{} raised local terrain, released tracked minerals and heated nearby habitat.",self.eruptions));
    }
    pub fn tick(&mut self, core: &mut Core, solutes: &mut [[f64; crate::physical::SOLUTES]]) {
        let dt = DT as f64;
        let season = (core.time * std::f32::consts::TAU / self.season_period).sin() * self.season_strength;
        let phase = core.time * std::f32::consts::TAU / 48.;
        let sun = body::V3::new(phase.sin(), season * 0.25, phase.cos()).normalized();
        self.sun_direction = [sun.x, sun.y, sun.z];
        let day = if self.domain.spherical() {
            1.
        } else {
            phase.cos().max(0.)
        };
        if self.storm <= 0. && core.random() < DT * 0.04 {
            self.storm = 6. + core.random() * 6.;
            self.storm_radius = 14f32.max(self.domain.spacing() * 3.);
            self.storm_center = [
                (core.random() - 0.5) * (self.domain.size() - 20.),
                (core.random() - 0.5)
                    * (if self.domain.spherical() {
                        self.domain.half() - 4.
                    } else {
                        self.domain.size() - 20.
                    }),
            ];
        }
        let raining = self.storm > 0.;
        self.storm = (self.storm - DT).max(0.);
        self.sunlight = (0.08 + day * 1.5) * (1. + season * 0.15) * if raining { 0.65 } else { 1. };
        self.temperature = (core.config.temperature + season * 0.18 + day * 0.05
            - if raining { 0.06 } else { 0. }
            - self.cloud_cover * 0.06)
            .clamp(0., 1.);
        // Air can hold only so much. Whatever it carries beyond what it began
        // with falls back as rain everywhere, so the seas are not lost to the sky.
        let area = |i: usize| {
            if self.domain.spherical() {
                self.domain.area(i)
            } else {
                1.
            }
        };
        let surface: f64 = (0..self.domain.sites()).map(area).sum();
        let drizzle = (self.atmosphere - AIR_CAPACITY * surface).max(0.) * dt * RAIN_OUT / surface;
        for i in 0..self.domain.sites() {
            let weight =
                (1. - self.distance_from(i, self.storm_center) / self.storm_radius).max(0.) as f64;
            let storm = if raining { dt * 0.018 * weight } else { 0. };
            let rain = ((storm + drizzle) * area(i)).min(self.atmosphere);
            if rain > 0. {
                self.atmosphere -= rain;
                self.precipitation += rain;
                if self.temperature_at(i) < 0.3 {
                    self.snow[i] += rain;
                } else {
                    self.water[i] += rain;
                }
            }
            let melt = (self.snow[i] * dt * 0.2 * (self.temperature_at(i) - 0.3).max(0.) as f64)
                .min(self.snow[i]);
            self.snow[i] -= melt;
            self.water[i] += melt;
            self.snowmelt += melt;
            let evaporated = (dt
                * 0.0015
                * self.sunlight as f64
                * if self.domain.spherical() {
                    self.domain.area(i)
                } else {
                    1.
                })
            .min(self.water[i]);
            self.water[i] -= evaporated;
            self.atmosphere += evaporated;
            self.evaporation += evaporated;
            self.heat[i] *= (-0.035 * DT).exp();
        }
        // Sequential bounded donor-cell runoff; each transfer moves existing
        // water and entrained solutes. Nothing is converted into generic food.
        if !self.domain.spherical() {
            for z in 0..self.domain.rows() {
                for x in 0..self.domain.grid {
                    let i = z * self.domain.grid + x;
                    for j in [
                        if x + 1 < self.domain.grid {
                            Some(i + 1)
                        } else {
                            None
                        },
                        if z + 1 < self.domain.grid {
                            Some(i + self.domain.grid)
                        } else {
                            None
                        },
                    ]
                    .into_iter()
                    .flatten()
                    {
                        let height = |k: usize| {
                            self.domain.ground(
                                self.domain.point(k).0,
                                self.domain.point(k).1,
                                core.config.seed,
                            ) as f64
                                + self.relief[k] as f64
                                + self.water[k] * 0.25
                        };
                        let difference = height(i) - height(j);
                        let (a, b) = if difference >= 0. { (i, j) } else { (j, i) };
                        let moved = (difference.abs() * dt * 0.003).min(self.water[a] * 0.04);
                        let fraction = if self.water[a] > 0. {
                            moved / self.water[a] * 0.2
                        } else {
                            0.
                        };
                        self.water[a] -= moved;
                        self.water[b] += moved;
                        self.runoff += moved;
                        for k in 0..crate::physical::SOLUTES {
                            let transported = solutes[a][k] * fraction;
                            solutes[a][k] -= transported;
                            solutes[b][k] += transported;
                        }
                    }
                }
            }
        }
        let rate = if core.config.kind == "volcanic" {
            0.008
        } else {
            0.002
        };
        if core.random() < DT * rate {
            self.erupt(core, solutes);
        }
    }
    pub fn view(&self, time: f32) -> serde_json::Value {
        serde_json::json!({"day":(time/48.).floor(),"season":(time/self.season_period).fract(),"seasonStrength":self.season_strength,"seasonPeriod":self.season_period,"sunlight":self.sunlight,
            "cloudCover":self.cloud_cover,"sunDirection":self.sun_direction,"temperature":self.temperature,"weather":if self.storm>0. {if self.temperature<0.3 {"Snow"}else{"Rain"}}else{if self.cloud_cover>0.5 {"Cloudy"}else{"Clear"}},
            "stormRadius":self.storm_radius,"storm":self.storm,"stormCenter":self.storm_center,"water":self.water.iter().sum::<f64>(),
            "snow":self.snow.iter().sum::<f64>(),"atmosphere":self.atmosphere,"waterError":self.water_error(),
            "precipitation":self.precipitation,"evaporation":self.evaporation,"snowmelt":self.snowmelt,
            "runoff":self.runoff,"eruptions":self.eruptions,"mineralInput":self.mineral_input,
            "waterDepth":(0..self.domain.sites()).map(|i|self.surface_depth(i)).collect::<Vec<_>>(),"relief":self.relief,"snowCover":self.snow,"heat":self.heat,"terrainVersion":self.terrain_version})
    }
}
