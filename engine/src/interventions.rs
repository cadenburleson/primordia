//! Explicit editor actions. These never count as natural mutations or births.
use crate::body::V3;
use crate::*;

fn one() -> f32 { 1. }

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Intervention {
    Climate {
        sunlight: f32,
        temperature: f32,
        clouds: f32,
    },
    Brush {
        tool: String,
        x: f32,
        z: f32,
        radius: f32,
        amount: f64,
        #[serde(default = "one")]
        aspect: f32,
        #[serde(default)]
        angle: f32,
        #[serde(default)]
        level: Option<f32>,
        #[serde(default)]
        points: Vec<[f32; 2]>,
    },
    Seasons { strength: f32, period: f32 },
    Rain {
        x: f32,
        z: f32,
    },
    Genes {
        body: u32,
        cell: u32,
        genes: Vec<autonomous::Gene>,
    },
}
pub(crate) fn site(d: domain::Domain, x: f32, z: f32) -> Result<(), String> {
    if !x.is_finite()
        || !z.is_finite()
        || x.abs() > d.half()
        || z.abs()
            > if d.spherical() {
                d.half() * 0.5
            } else {
                d.wall()
            }
    {
        return Err("Choose a surface site inside this world".into());
    }
    Ok(())
}
impl Core {
    pub fn intervene(&mut self, json: &str) -> Result<String, String> {
        if json.len() > 64 * 1024 {
            return Err("Editor action is too large".into());
        }
        let action: Intervention =
            serde_json::from_str(json).map_err(|_| "Invalid editor action")?;
        let original = self
            .autonomous
            .as_ref()
            .ok_or("Editing requires the current cellular world")?;
        let h = original
            .physical
            .as_ref()
            .ok_or("Editing requires a physical habitat")?;
        if !h.ablation.is_empty() || self.config.physical_assay != "full" {
            return Err("Return to the main world before editing".into());
        }
        // Commit only after validation; invalid edits leave the world intact.
        let mut next = original.clone();
        let mut config = self.config.clone();
        let note;
        match action {
            Intervention::Climate {
                sunlight,
                temperature,
                clouds,
            } => {
                if ![sunlight, temperature, clouds]
                    .iter()
                    .all(|v| v.is_finite() && (0.0..=1.).contains(v))
                {
                    return Err("Climate inputs must be between 0 and 1".into());
                }
                let p = next
                    .physical
                    .as_mut()
                    .unwrap()
                    .planet
                    .as_mut()
                    .ok_or("No planetary climate")?;
                p.temperature = (p.temperature + temperature
                    - config.temperature
                    - (clouds - p.cloud_cover) * 0.06)
                    .clamp(0., 1.);
                p.cloud_cover = clouds;
                config.sunlight = sunlight;
                config.temperature = temperature;
                note = format!("Climate edited: sunlight {:.0}%, baseline temperature {:.0}%, cloud cover {:.0}%.",sunlight*100.,temperature*100.,clouds*100.);
            }
            Intervention::Seasons { strength, period } => {
                if !strength.is_finite() || !(0.0..=2.).contains(&strength) || !period.is_finite() || !(48.0..=2400.).contains(&period) {
                    return Err("Season strength must be 0–2 and period 48–2400 cycles".into());
                }
                let p = next.physical.as_mut().unwrap().planet.as_mut().ok_or("No planetary climate")?;
                p.season_strength = strength;
                p.season_period = period;
                note = format!("Seasonal forcing set to {:.0}% over {:.0} cycles. Genes unchanged.", strength*100., period);
            }
            Intervention::Rain { x, z } => {
                site(h.domain, x, z)?;
                let p = next
                    .physical
                    .as_mut()
                    .unwrap()
                    .planet
                    .as_mut()
                    .ok_or("No water cycle")?;
                p.storm = 12.;
                p.storm_radius = 14f32.max(p.domain.spacing() * 3.);
                p.storm_center = [x, z];
                note = "Rain system introduced. Precipitation draws from existing atmospheric water as evolution resumes.".into();
            }
            Intervention::Brush {
                tool,
                x,
                z,
                radius,
                amount, aspect, angle, level, mut points,
            } => {
                let d = h.domain;
                site(d, x, z)?;
                if !radius.is_finite()
                    || radius < d.spacing() * 0.75
                    || radius > d.size() * 0.5
                    || !amount.is_finite()
                    || !(0.0..=8.).contains(&amount)
                    || amount == 0.
                    || !aspect.is_finite() || !(0.25..=1.).contains(&aspect)
                    || radius * aspect + d.spacing() * 1e-5 < d.spacing() * 0.75
                    || !angle.is_finite() || angle.abs() > 360.
                    || level.is_some_and(|v| !v.is_finite() || v.abs() > 64.)
                    || points.len() > 12
                {
                    return Err("Choose a brush radius of at least 0.75 field spacings and a positive amount up to 8".into());
                }
                if points.is_empty() { points.push([x, z]); }
                for &[px, pz] in &points { site(d, px, pz)?; }
                let terrain = matches!(tool.as_str(), "raise" | "lower" | "flatten");
                let component = match tool.as_str() {
                    "fuel" => Some(0),
                    "mineral" => Some(2),
                    "detritus" => Some(5),
                    "water" => None,
                    _ if terrain => None,
                    _ => return Err("Unknown surface tool".into()),
                };
                let nh = next.physical.as_mut().unwrap();
                let before = nh.totals(&next.cells);
                let p = nh.planet.as_mut().ok_or("No planetary surface")?;
                let level = level.unwrap_or_else(|| d.ground(x, z, self.config.seed) + p.relief_at(x, z));
                let mut touched = 0;
                let mut supplied = 0.;
                let samples: Vec<_> = (0..d.sites()).map(|i| { let (sx,sz)=d.point(i); (sx,sz,d.normal(sx,sz)) }).collect();
                let (sin, cos) = angle.to_radians().sin_cos();
                for [x,z] in points {
                  let center = d.normal(x,z);
                  let center_site = d.index(x,z);
                  let east = V3::new((x / d.radius()).cos(), 0., -(x / d.radius()).sin());
                  let north = V3::new(center.y * east.z, center.z * east.x - center.x * east.z, -center.y * east.x);
                  for (i, &(sx,sz,n)) in samples.iter().enumerate() {
                    let (u,v) = if d.spherical() {
                        let dot = center.dot(n).clamp(-1.,1.);
                        let distance = dot.acos() * d.radius();
                        if distance > radius && i != center_site { continue; }
                        let tangent = n.sub(center.mul(dot));
                        let length = tangent.len();
                        if length > 1e-6 { (tangent.dot(east)*distance/length, tangent.dot(north)*distance/length) } else { (distance, 0.) }
                    } else { (sx-x, sz-z) };
                    let major = u*cos+v*sin;
                    let minor = -u*sin+v*cos;
                    let distance = (major/radius).hypot(minor/(radius*aspect));
                    let weight = if i == center_site { (1.-distance).max(0.1) } else { (1.-distance).max(0.) };
                    if weight <= 0. {
                        continue;
                    }
                    touched += 1;
                    if terrain {
                        p.relief[i] = match tool.as_str() {
                            "raise" => p.relief[i] + amount as f32 * weight,
                            "lower" => p.relief[i] - amount as f32 * weight,
                            _ => {
                                p.relief[i]
                                    + (level - d.ground(sx, sz, self.config.seed) - p.relief[i])
                                        * (amount as f32).min(1.)
                                        * weight
                            }
                        }
                        .clamp(-8., 8.);
                    } else {
                        let mass = amount * weight as f64 * d.area(i);
                        if let Some(k) = component {
                            nh.solutes[i][k] += mass;
                        } else {
                            p.water[i] += mass;
                            p.initial_water += mass;
                        }
                        supplied += mass;
                    }
                }
                }
                p.terrain_version = p.terrain_version.wrapping_add(1);
                let after = nh.totals(&next.cells);
                nh.initial_organic += after.0 - before.0;
                nh.initial_mineral += after.1 - before.1;
                nh.initial_energy += after.2 - before.2;
                if terrain {
                    // Lift connected bodies together when the edited ground rises.
                    // No new tissue, chemical energy, or material is created.
                    let groups = next.groups();
                    let nh = next.physical.as_ref().unwrap();
                    let p = nh.planet.as_ref().unwrap();
                    for group in groups {
                        let lift = group
                            .iter()
                            .map(|&i| {
                                let c = &next.cells[i];
                                let (cx, cz) = d.coords(c.p);
                                let height = if d.spherical() {
                                    c.p.len() - d.radius()
                                } else {
                                    c.p.y
                                };
                                let ground =
                                    d.ground(cx, cz, self.config.seed) + p.relief_at(cx, cz);
                                let old_ground = d.ground(cx, cz, self.config.seed)
                                    + h.planet.as_ref().map_or(0., |old| old.relief_at(cx, cz));
                                if ground > old_ground + 1e-5 {
                                    ground + c.radius() + 0.02 - height
                                } else {
                                    0.
                                }
                            })
                            .fold(0f32, f32::max);
                        if lift > 0. {
                            let center = group.iter().fold(V3::default(), |s, &i| {
                                s.add(next.cells[i].p.mul(1. / group.len() as f32))
                            });
                            let up = d.up(center);
                            for i in group {
                                next.cells[i].p = next.cells[i].p.add(up.mul(lift));
                                next.cells[i].state.ground = false;
                            }
                        }
                    }
                }
                note = if terrain {
                    format!("Terrain '{}' edited at ({:.1}, {:.1}); {} field sites. Bodies lifted clear of raised ground.",tool,x,z,touched)
                } else {
                    format!("External {} input: {:.4} model mass over {} field sites at ({:.1}, {:.1}).",tool,supplied,touched,x,z)
                };
            }
            Intervention::Genes { body, cell, genes } => {
                let group = next
                    .groups()
                    .into_iter()
                    .find(|g| next.cells[g[0]].id == body)
                    .ok_or("Select a living body")?;
                let index = group
                    .into_iter()
                    .find(|&i| next.cells[i].id == cell)
                    .ok_or("That cell no longer belongs to the selected body")?;
                let id = next.next_genome;
                next.next_genome = id.checked_add(1).ok_or("Genome identities exhausted")?;
                next.cells[index].genome = autonomous::Genome { id, genes };
                // Material and homologous chromosome stay intact. Subsequent
                // expression/synthesis and descent follow the existing rules.
                note=format!("Manual chromosome edit in cell #{} of body #{}; new genome #{}. Not a natural mutation; future descendants can inherit it.",cell,body,id);
            }
        }
        config.validate()?;
        if !next.valid() {
            return Err("Editor action would violate cellular or material constraints".into());
        }
        self.autonomous = Some(next);
        self.config = config;
        self.event("world", note.clone());
        Ok(note)
    }
    pub fn founder_specimen(&self) -> Result<String, String> {
        let cell = autonomous::Cell {
            collection_sources: vec![],
            material: Some(physical::Inventory::founder()),
            state: physiology::CellState::default(),
            lifecycle: development::Lifecycle::default(),
            id: 1,
            parent: 0,
            founder: 1,
            generation: 0,
            genome: autonomous::Genome::founder(),
            p: V3::default(),
            v: V3::default(),
            energy: 5.,
            age: 0.,
            cooldown: 0.,
            chemicals: [0.; 2],
            expression: [0.; autonomous::ACTIONS],
            heading: 0.,
        };
        let s = specimens::Specimen {
            format: "primordia-organism".into(),
            version: 1,
            name: "Introduced founder".into(),
            source_body: 0,
            source_seed: self.config.seed,
            captured_cycle: self.time,
            cells: vec![cell],
            bonds: vec![],
            joints: vec![],
            synapses: vec![],
        };
        let json = serde_json::to_string(&s).map_err(|_| "Could not encode founder")?;
        specimens::inspect(&json)?;
        Ok(json)
    }
}
