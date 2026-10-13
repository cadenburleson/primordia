//! Portable connected-body snapshots. Introductions are explicit external input,
//! never mutations, births, or an ecological fitness reward.
use crate::{
    autonomous::{Bond, Cell, Evolution, MAX_CELLS},
    body::V3,
    development::{Development, Synapse},
    physical::MaterialJoint,
    *,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Specimen {
    pub format: String,
    pub version: u32,
    pub name: String,
    pub source_body: u32,
    pub source_seed: u32,
    pub captured_cycle: f32,
    pub cells: Vec<Cell>,
    pub bonds: Vec<Bond>,
    pub joints: Vec<MaterialJoint>,
    pub synapses: Vec<Synapse>,
}
fn cross(a: V3, b: V3) -> V3 {
    V3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}
fn frame(d: domain::Domain, p: V3) -> [V3; 3] {
    let east = d.tangent(p, 0.).normalized();
    let up = d.up(p);
    [east, up, cross(east, up).normalized()]
}
fn local(v: V3, f: [V3; 3]) -> V3 {
    V3::new(v.dot(f[0]), v.dot(f[1]), v.dot(f[2]))
}
fn global(v: V3, f: [V3; 3]) -> V3 {
    f[0].mul(v.x).add(f[1].mul(v.y)).add(f[2].mul(v.z))
}
fn empty(cells: Vec<Cell>, bonds: Vec<Bond>) -> Evolution {
    Evolution {
        physical: None,
        cells,
        bonds,
        next_cell: u32::MAX,
        next_genome: u32::MAX,
        divisions: 0,
        attached_divisions: 0,
        deaths: 0,
        exchanges: 0,
        meals: 0,
        point_edits: 0,
        duplications: 0,
        deletions: 0,
        innovations: 0,
        food_energy: 0.,
        motor_work: 0.,
        history: vec![],
        peak_body: 0,
        peak_generation: 0,
    }
}
impl Specimen {
    fn parse(json: &str) -> Result<Self, String> {
        if json.len() > 64 * 1024 * 1024 {
            return Err("Organism file exceeds 64 MB".into());
        }
        let s: Self = serde_json::from_str(json).map_err(|_| "Invalid organism collection file")?;
        if s.format != "primordia-organism"
            || s.version != 1
            || s.name.trim().is_empty()
            || s.name.len() > 80
            || s.cells.is_empty()
            || s.cells.len() > MAX_CELLS
            || !s.captured_cycle.is_finite()
            || s.captured_cycle < 0.
            || s.bonds.len() > s.cells.len() * 3
            || s.joints.len() > s.cells.len() * 3
            || s.synapses.len() > s.cells.len() * 6
        {
            return Err("Unsupported or oversized organism snapshot".into());
        }
        let mut cells = s.cells.clone();
        for c in &mut cells {
            if ![c.p.x, c.p.y, c.p.z]
                .iter()
                .all(|v| v.is_finite() && v.abs() < 64000.)
                || !c
                    .material
                    .as_ref()
                    .is_some_and(crate::physical::Inventory::valid)
            {
                return Err("Invalid cellular geometry or material stores".into());
            }
            c.p = V3::default();
            c.material = None;
        }
        let e = empty(cells, s.bonds.clone());
        let ids: BTreeSet<_> = e.cells.iter().map(|c| c.id).collect();
        let pairs: BTreeSet<_> = s.bonds.iter().map(|b| (b.a, b.b)).collect();
        let mut joint_keys = BTreeSet::new();
        let joints_valid = s.joints.iter().all(|j| {
            ids.contains(&j.a)
                && ids.contains(&j.b)
                && ids.contains(&j.c)
                && j.a < j.c
                && j.a != j.b
                && j.b != j.c
                && joint_keys.insert((j.a, j.b, j.c))
                && pairs.contains(&(j.a.min(j.b), j.a.max(j.b)))
                && pairs.contains(&(j.b.min(j.c), j.b.max(j.c)))
                && [j.rest, j.target, j.angle]
                    .iter()
                    .all(|v| v.is_finite() && (0.0..=std::f32::consts::PI).contains(v))
                && [j.flexion, j.extension]
                    .iter()
                    .all(|v| v.is_finite() && (0.0..=1.).contains(v))
                && j.work.is_finite()
                && j.work >= 0.
        });
        let development = Development {
            synapses: s.synapses.clone(),
            ..Development::default()
        };
        if !e.valid()
            || e.groups().len() != 1
            || !joints_valid
            || !development.valid(&e.cells, &s.joints)
            || s.synapses
                .iter()
                .any(|n| !pairs.contains(&(n.from.min(n.to), n.from.max(n.to))))
        {
            #[cfg(test)]
            eprintln!(
                "SPECIMEN refused: cells valid {}, pieces {}, joints valid {}, development valid {}; cells with a bad genome {}, state {}, life cycle {}, number {}; states {:?}",
                e.valid(),
                e.groups().len(),
                joints_valid,
                development.valid(&e.cells, &s.joints),
                e.cells.iter().filter(|x| !x.genome.valid()).count(),
                e.cells.iter().filter(|x| !x.state.valid()).count(),
                e.cells.iter().filter(|x| !x.lifecycle.valid(x)).count(),
                e.cells.iter().filter(|x| ![x.v.x, x.v.y, x.v.z, x.energy, x.age, x.cooldown, x.heading].iter().all(|v| v.is_finite())).count(),
                e.cells.iter().filter(|x| !x.state.valid()).map(|x| format!("{:?}", x.state)).next(),
            );
            return Err("Invalid connected organism structure, chromosomes, or signaling".into());
        }
        Ok(s)
    }
    fn summary(&self) -> serde_json::Value {
        let genomes = self
            .cells
            .iter()
            .flat_map(|c| {
                std::iter::once(c.genome.id).chain(c.lifecycle.homolog.as_ref().map(|g| g.id))
            })
            .collect::<BTreeSet<_>>()
            .len();
        let mut genes = [0usize; autonomous::ACTIONS];
        let mut functions = [0usize; autonomous::ACTIONS];
        for c in &self.cells {
            for g in c
                .genome
                .genes
                .iter()
                .chain(c.lifecycle.homolog.iter().flat_map(|h| h.genes.iter()))
            {
                genes[g.action] += 1;
            }
            for (k, v) in functions.iter_mut().enumerate() {
                if c.activity(k) > 0.1 {
                    *v += 1
                }
            }
        }
        serde_json::json!({"name":self.name,"cells":self.cells.len(),"generation":self.cells.iter().map(|c|c.generation).max().unwrap_or(0),"genomes":genomes,"joints":self.joints.len(),"bonds":self.bonds.len(),"synapses":self.synapses.len(),"sourceBody":self.source_body,"sourceSeed":self.source_seed,"cycle":self.captured_cycle,"genes":genes.to_vec(),"functions":functions.as_slice().to_vec(),"shape":self.cells.iter().take(128).map(|c|[c.p.x,c.p.y,c.p.z,c.radius()]).collect::<Vec<_>>()})
    }
}
pub fn inspect(json: &str) -> Result<String, String> {
    Ok(Specimen::parse(json)?.summary().to_string())
}
impl Core {
    pub fn capture_specimen(&self, selected: u32, name: &str) -> Result<String, String> {
        if name.trim().is_empty() || name.len() > 80 {
            return Err("Give the organism a name of at most 80 bytes".into());
        }
        let e = self
            .autonomous
            .as_ref()
            .ok_or("Collections require the current cellular engine")?;
        let h = e
            .physical
            .as_ref()
            .ok_or("Collections require a physical cellular world")?;
        let group = e
            .groups()
            .into_iter()
            .find(|g| e.cells[g[0]].id == selected)
            .ok_or("Select a living organism first")?;
        let ids: BTreeSet<_> = group.iter().map(|&i| e.cells[i].id).collect();
        let center = group.iter().fold(V3::default(), |p, &i| {
            p.add(e.cells[i].p.mul(1. / group.len() as f32))
        });
        let f = frame(h.domain, center);
        let mut cells: Vec<_> = group
            .iter()
            .map(|&i| {
                let mut c = e.cells[i].clone();
                let direction = local(h.domain.tangent(c.p, c.heading), f);
                c.heading = direction.z.atan2(direction.x);
                c.p = local(c.p.sub(center), f);
                c.v = local(c.v, f);
                c
            })
            .collect();
        cells.sort_by_key(|c| c.id);
        let s = Specimen {
            format: "primordia-organism".into(),
            version: 1,
            name: name.trim().into(),
            source_body: selected,
            source_seed: self.config.seed,
            captured_cycle: self.time,
            cells,
            bonds: e
                .bonds
                .iter()
                .filter(|b| ids.contains(&b.a) && ids.contains(&b.b))
                .cloned()
                .collect(),
            joints: h
                .joints
                .iter()
                .filter(|j| [j.a, j.b, j.c].iter().all(|id| ids.contains(id)))
                .cloned()
                .collect(),
            synapses: h
                .development
                .synapses
                .iter()
                .filter(|n| ids.contains(&n.from) && ids.contains(&n.to))
                .cloned()
                .collect(),
        };
        let json = serde_json::to_string(&s).map_err(|_| "Could not encode organism")?;
        Specimen::parse(&json)?;
        Ok(json)
    }
    pub fn place_specimen(&mut self, json: &str, x: f32, z: f32) -> Result<u32, String> {
        self.place_body(json, x, z, false)
    }
    /// Sets a body down at a site. With `clear`, single cells drifting where
    /// it is to go are moved aside to make room, as water moves aside for
    /// anything lowered into it. Other bodies are never moved.
    pub(crate) fn place_body(&mut self, json: &str, x: f32, z: f32, clear: bool) -> Result<u32, String> {
        let mut s = Specimen::parse(json)?;
        let original = self
            .autonomous
            .as_ref()
            .ok_or("Place organisms in the current cellular engine")?;
        let h = original
            .physical
            .as_ref()
            .ok_or("Place organisms in a physical world")?;
        if !h.ablation.is_empty() {
            return Err("Return to the main world before introducing organisms".into());
        }
        let d = h.domain;
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
            return Err("Choose a site on this world".into());
        }
        if original.cells.len() + s.cells.len() > MAX_CELLS.min(self.config.capacity) {
            return Err("This organism does not fit within the current living-cell limit".into());
        }
        let ground = |p: V3| {
            let (x, z) = d.coords(p);
            d.ground(x, z, self.config.seed) + h.planet.as_ref().map_or(0., |p| p.relief_at(x, z))
        };
        let anchor = d.position(x, z, ground(d.position(x, z, 0.)) + 0.5);
        let f = frame(d, anchor);
        for c in &mut s.cells {
            let direction = global(V3::new(c.heading.cos(), 0., c.heading.sin()), f);
            c.p = anchor.add(global(c.p, f));
            c.heading = direction
                .dot(d.tangent(c.p, std::f32::consts::FRAC_PI_2))
                .atan2(direction.dot(d.tangent(c.p, 0.)));
            c.v = V3::default();
            c.state.ground = false;
            c.state.anchored = false;
            c.state.flux = observations::Flux::default();
            c.state.motor_work = 0.;
            c.state.meals = 0;
            c.collection_sources = vec![s.name.clone()];
        }
        // What lives fixed to the bottom is set down on it.
        let settles = s
            .cells
            .iter()
            .any(|c| c.genome.genes.iter().any(|g| g.action == crate::physical::HOLDFAST));
        // A body afloat hangs wholly under the surface where the water is
        // deep enough to hold it, and rests on the bottom where it is not.
        let (mut sink, mut rise) = (f32::INFINITY, f32::NEG_INFINITY);
        for c in &s.cells {
            let (x, z) = d.coords(c.p);
            let height = if d.spherical() {
                c.p.len() - d.radius()
            } else {
                c.p.y
            };
            match h.planet.as_ref() {
                None => rise = rise.max(ground(c.p) + c.radius() + 0.02 - height),
                Some(p) => {
                    let floor = p.floor(x, z, c.radius(), self.config.seed);
                    if settles {
                        rise = rise.max(floor - height);
                    } else {
                        rise = rise.max(floor + 0.02 - height);
                        sink = sink
                            .min(p.berth(x, z, c.radius(), self.config.seed) + 0.02 - height);
                    }
                }
            }
        }
        let lift = if sink.is_finite() { sink.max(rise) } else { rise };
        let bin_size = original
            .cells
            .iter()
            .chain(&s.cells)
            .map(|c| c.radius() * 2. + 0.04)
            .fold(1f32, f32::max);
        let bin = |p: V3| {
            (
                (p.x / bin_size).floor() as i32,
                (p.y / bin_size).floor() as i32,
                (p.z / bin_size).floor() as i32,
            )
        };
        let mut occupied: BTreeMap<_, Vec<&Cell>> = BTreeMap::new();
        for c in &original.cells {
            occupied.entry(bin(c.p)).or_default().push(c);
        }
        let held: BTreeSet<u32> = if clear {
            original.bonds.iter().flat_map(|b| [b.a, b.b]).collect()
        } else {
            BTreeSet::new()
        };
        let mut crowd = BTreeSet::new();
        for c in &mut s.cells {
            c.p = c.p.add(f[1].mul(lift));
            let (a, b, k) = bin(c.p);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        for other in occupied.get(&(a + dx, b + dy, k + dz)).into_iter().flatten() {
                            if other.p.sub(c.p).len() >= other.radius() + c.radius() + 0.04 {
                                continue;
                            }
                            if !clear || held.contains(&other.id) || other.state.anchored {
                                return Err("That spot is occupied. Choose a little more space beside the other organism".into());
                            }
                            crowd.insert(other.id);
                        }
                    }
                }
            }
        }
        // Stage the insertion; a rejected file/placement leaves the world intact.
        let mut next = original.clone();
        if !crowd.is_empty() {
            let n = s.cells.len() as f32;
            let middle = s.cells.iter().fold(V3::default(), |p, c| p.add(c.p.mul(1. / n)));
            let up = d.up(middle);
            let level = |v: V3| v.sub(up.mul(v.dot(up)));
            let reach = s
                .cells
                .iter()
                .map(|c| level(c.p.sub(middle)).len() + c.radius())
                .fold(0f32, f32::max);
            for c in next.cells.iter_mut().filter(|c| crowd.contains(&c.id)) {
                let out = level(c.p.sub(middle));
                let way = if out.len() > 1e-3 { out.normalized() } else { f[0] };
                c.p = c.p.add(way.mul(reach + c.radius() + 0.1 - out.len()));
            }
        }
        let mut references = BTreeSet::new();
        let mut genomes = BTreeSet::new();
        for c in &s.cells {
            references.extend([c.id, c.parent, c.lifecycle.root, c.lifecycle.mate]);
            if c.lifecycle.root == 0 {
                references.insert(c.founder);
            }
            genomes.insert(c.genome.id);
            if let Some(g) = &c.lifecycle.homolog {
                genomes.insert(g.id);
            }
        }
        references.remove(&0);
        let mut remap = BTreeMap::from([(0, 0)]);
        for id in references {
            let fresh = next.next_cell;
            next.next_cell = next
                .next_cell
                .checked_add(1)
                .ok_or("Cell identity space exhausted")?;
            remap.insert(id, fresh);
        }
        let mut genome_map = BTreeMap::new();
        for id in genomes {
            let fresh = next.next_genome;
            next.next_genome = next
                .next_genome
                .checked_add(1)
                .ok_or("Genome identity space exhausted")?;
            genome_map.insert(id, fresh);
        }
        for c in &mut s.cells {
            c.id = remap[&c.id];
            c.parent = remap[&c.parent];
            c.lifecycle.root = remap[&if c.lifecycle.root == 0 {
                c.founder
            } else {
                c.lifecycle.root
            }];
            c.lifecycle.mate = remap[&c.lifecycle.mate];
            c.genome.id = genome_map[&c.genome.id];
            if let Some(g) = &mut c.lifecycle.homolog {
                g.id = genome_map[&g.id];
            }
        }
        for b in &mut s.bonds {
            b.a = remap[&b.a];
            b.b = remap[&b.b];
        }
        for j in &mut s.joints {
            j.a = remap[&j.a];
            j.b = remap[&j.b];
            j.c = remap[&j.c];
            j.work = 0.;
        }
        for n in &mut s.synapses {
            n.from = remap[&n.from];
            n.to = remap[&n.to];
        }
        let selected = s.cells[0].id;
        let count = s.cells.len();
        let imported_water = s
            .cells
            .iter()
            .map(|c| c.material.as_ref().unwrap().body.water)
            .sum::<f64>();
        let imported_gas = s
            .cells
            .iter()
            .map(|c| c.material.as_ref().unwrap().body.gas.iter().sum::<f64>())
            .sum::<f64>();
        let before = next.physical.as_ref().unwrap().totals(&next.cells);
        let nitrogen = next.physical.as_ref().unwrap().nitrogen_total(&next.cells);
        // A body arrives with a working store of nitrogen, as a living one has.
        let mut arrivals = s.cells;
        for m in arrivals.iter_mut().filter_map(|c| c.material.as_mut()) {
            m.nitrogen = m.nitrogen.max(m.bound_nitrogen() * 0.5);
            m.phosphorus = m.phosphorus.max(m.bound_phosphorus() * 0.5);
        }
        next.cells.extend(arrivals);
        next.bonds.extend(s.bonds);
        let nh = next.physical.as_mut().unwrap();
        nh.joints.extend(s.joints);
        nh.development.synapses.extend(s.synapses);
        nh.settle_arrivals(before, nitrogen, &next.cells);
        nh.systems.initial_gas += imported_gas;
        if let Some(p) = &mut nh.planet {
            p.initial_water += imported_water;
            p.biological_water += imported_water;
        }
        next.peak_body = next.peak_body.max(count);
        next.peak_generation = next
            .peak_generation
            .max(next.cells.iter().map(|c| c.generation).max().unwrap_or(0));
        if !next.valid() {
            return Err("This organism cannot be placed intact at that site in this world".into());
        }
        self.autonomous = Some(next);
        self.event("world",format!("Introduced collection specimen '{}' as body #{} ({} cells). External material input; not an evolutionary birth.",s.name,selected,count));
        Ok(selected)
    }
}
