//! The planet's record-holders, and painting life onto the map.
//! A champion is the living body that leads on one measure. The interface
//! keeps a copy of each so the player can release it again; nothing here
//! changes the world except `paint_life`, which is explicit external input.
use crate::{
    autonomous::{Cell, Evolution},
    body::V3,
    physical::{Habitat, CHEMO, ENGULF, FIN, PROPEL, SPINE, STEER, TOXIN, WALL},
    physiology::{BARRIER, CHANNEL, RECEPTOR},
    *,
};

/// Record slots: key and title.
pub const SLOTS: [(&str, &str); 4] = [
    ("advanced", "Most advanced"),
    ("largest", "Largest body"),
    ("hunter", "Top hunter"),
    ("swimmer", "Fastest swimmer"),
];
/// Bodies larger than this are not copied automatically: the snapshot would be huge.
const LARGEST_KEPT: usize = 160;
/// Working parts a cell can build, by the material that makes each.
const PARTS: [usize; 16] = [
    0, 4, 5, 9, RECEPTOR, CHANNEL, BARRIER, 16, 18, 20, PROPEL, STEER, ENGULF, TOXIN, SPINE, CHEMO,
];

#[derive(Serialize)]
pub struct Champion {
    pub slot: &'static str,
    pub title: &'static str,
    /// The body's handle: pass it to `capture_specimen`.
    pub body: u32,
    pub cells: usize,
    pub score: f32,
    /// What earned the record, in plain words.
    pub note: String,
}

fn has(c: &Cell, action: usize) -> bool {
    c.activity(action) > 0.25
}

impl Habitat {
    /// How many different working parts a body has built, counting bone, fat,
    /// walls and fins alongside the materials in `PARTS`.
    fn parts(&self, e: &Evolution, group: &[usize]) -> usize {
        let any = |f: &dyn Fn(&Cell) -> bool| usize::from(group.iter().any(|&i| f(&e.cells[i])));
        PARTS.iter().map(|&k| any(&|c| has(c, k))).sum::<usize>()
            + any(&|c| has(c, WALL))
            + any(&|c| has(c, FIN))
            + any(&|c| {
                let m = c.material.as_ref().unwrap();
                m.mineral / m.mass().max(1e-12) > 0.025
            })
            + any(&|c| {
                let m = c.material.as_ref().unwrap();
                m.lipid / m.biomass.max(1e-12) > 0.08
            })
    }
    /// The leading living body on each measure. A slot with no body that
    /// earns it is left out.
    pub(crate) fn champions(&self, e: &Evolution) -> Vec<Champion> {
        let mut best: [Option<Champion>; 4] = [None, None, None, None];
        let mut offer = |k: usize, body: u32, cells: usize, score: f32, note: String| {
            if score > 0. && best[k].as_ref().is_none_or(|b| score > b.score) {
                best[k] = Some(Champion { slot: SLOTS[k].0, title: SLOTS[k].1, body, cells, score, note });
            }
        };
        for g in e.groups() {
            let n = g.len();
            if n > LARGEST_KEPT {
                continue;
            }
            let body = e.cells[g[0]].id;
            let parts = self.parts(e, &g);
            let fates = g.iter().map(|&i| e.cells[i].state.fate).collect::<BTreeSet<_>>().len();
            let ids: BTreeSet<_> = g.iter().map(|&i| e.cells[i].id).collect();
            let joints = self.joints.iter().filter(|j| ids.contains(&j.b)).count();
            // Many kinds of working part, in a body big enough to need them.
            let kinds = parts + fates - 1 + joints.min(3);
            if kinds >= 3 {
                offer(
                    0,
                    body,
                    n,
                    kinds as f32 * (1. + (n as f32).log2()),
                    format!(
                        "{parts} kinds of working part{}{} in {n} cell{}",
                        if fates > 1 { format!(", {fates} cell fates") } else { String::new() },
                        if joints > 0 { format!(", {joints} joint{}", if joints == 1 { "" } else { "s" }) } else { String::new() },
                        if n == 1 { "" } else { "s" }
                    ),
                );
            }
            if n >= 4 {
                offer(1, body, n, n as f32, format!("{n} cells in one body"));
            }
            let prey: f64 = g.iter().map(|&i| e.cells[i].state.flux.prey).sum();
            if prey > 0.05 {
                let meals: u32 = g.iter().map(|&i| e.cells[i].state.meals).sum();
                offer(2, body, n, prey as f32, format!("{prey:.2} fuel taken from living prey in {meals} meals"));
            }
            // Speed through the water under its own power, not on the current.
            if g.iter().any(|&i| has(&e.cells[i], PROPEL) || (has(&e.cells[i], 4) && has(&e.cells[i], FIN))) {
                let mut v = V3::default();
                for &i in &g {
                    let c = &e.cells[i];
                    let flow = self.fluid.surface.as_ref().map_or(V3::default(), |f| f.velocities[self.domain.locate(c.p)]);
                    v = v.add(c.v.sub(flow));
                }
                let speed = v.mul(1. / n as f32).len();
                if speed > 0.01 {
                    offer(3, body, n, speed, format!("{speed:.2} units a cycle under its own power"));
                }
            }
        }
        best.into_iter().flatten().collect()
    }
}

impl Core {
    /// The current record-holders, as JSON for the interface.
    pub fn champions(&self) -> String {
        let list = self
            .autonomous
            .as_ref()
            .and_then(|e| e.physical.as_ref().map(|h| h.champions(e)))
            .unwrap_or_default();
        serde_json::to_string(&list).unwrap()
    }
    /// Scatter copies of one organism around each painted point: `per_point`
    /// tries within `radius` of every (x, z) pair in `points`. Spots that are
    /// taken, or off the world, are skipped. Returns how many were placed.
    pub fn paint_life(
        &mut self,
        json: &str,
        points: &[f32],
        radius: f32,
        per_point: u32,
    ) -> Result<u32, String> {
        if points.is_empty() || points.len() % 2 != 0 || points.len() > 64 {
            return Err("Paint between 1 and 32 points at a time".into());
        }
        if !radius.is_finite() || radius <= 0. || !(1..=16).contains(&per_point) {
            return Err("Choose a brush size and between 1 and 16 organisms a dab".into());
        }
        let d = self
            .autonomous
            .as_ref()
            .and_then(|e| e.physical.as_ref())
            .ok_or("Painting life needs the current cellular world")?
            .domain;
        // A body of many cells moves loose single cells aside as it is
        // set down; a single cell takes only a free spot.
        let whole = serde_json::from_str::<serde_json::Value>(&crate::specimens::inspect(json)?)
            .ok()
            .and_then(|v| v["cells"].as_u64())
            .is_some_and(|n| n > 1);
        let mut placed = 0;
        let mut full = None;
        for point in points.chunks(2) {
            if !point[0].is_finite() || !point[1].is_finite() {
                return Err("Choose a site on this world".into());
            }
            let centre = d.position(point[0], point[1], 0.);
            let (east, north) = (d.tangent(centre, 0.), d.tangent(centre, std::f32::consts::FRAC_PI_2));
            for _ in 0..per_point {
                // Evenly over the disc under the brush.
                let (r, a) = (radius * self.random().sqrt(), self.random() * std::f32::consts::TAU);
                let (x, z) = d.coords(centre.add(east.mul(r * a.cos())).add(north.mul(r * a.sin())));
                match self.place_body(json, x, z, whole) {
                    Ok(_) => placed += 1,
                    // A crowded spot is simply skipped; a full world ends the stroke.
                    Err(e) if e.contains("limit") => full = Some(e),
                    Err(_) => {}
                }
            }
        }
        match (placed, full) {
            (0, Some(e)) => Err(e),
            _ => Ok(placed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> Core {
        let mut c = Core::new(Config { initial_cells: 24, ..Config::default() }).unwrap();
        c.start_physical();
        c.autonomous.as_mut().unwrap().physical.as_mut().unwrap().oxygenate(1.);
        c
    }
    fn living(c: &Core, name: &str) -> usize {
        let e = c.autonomous.as_ref().unwrap();
        e.cells.iter().filter(|cell| cell.collection_sources.first().is_some_and(|s| s == name)).count()
    }

    #[test]
    fn painting_scatters_copies_under_the_brush_and_nowhere_else() {
        let mut c = world();
        let (_, hunter) = c.species_specimen("hunter").unwrap();
        let before = c.autonomous.as_ref().unwrap().cells.len();
        let placed = c.paint_life(&hunter, &[10., -8., 14., -8.], 3., 5).unwrap();
        assert!(placed >= 8, "most of ten dabs found room: {placed}");
        let e = c.autonomous.as_ref().unwrap();
        assert_eq!(e.cells.len(), before + placed as usize);
        assert_eq!(living(&c, "Hunter"), placed as usize);
        for cell in e.cells.iter().filter(|cell| !cell.collection_sources.is_empty()) {
            let near = [10f32, 14.].iter().any(|x| (cell.p.x - x).hypot(cell.p.z + 8.) <= 3.01);
            assert!(near, "painted at {:?}", (cell.p.x, cell.p.z));
        }
        assert!(e.valid(), "the world's books still balance");
        assert!(c.paint_life(&hunter, &[], 3., 5).is_err());
        assert!(c.paint_life(&hunter, &[0., 0.], 0., 5).is_err());
        assert!(c.paint_life(&hunter, &[0., 0.], 3., 99).is_err());
        assert!(c.paint_life("{}", &[0., 0.], 3., 1).is_err());
        assert_eq!(c.paint_life(&hunter, &[9e9, 0.], 3., 1), Ok(0), "off the world: nothing placed");
        c.steer(r#"{"kind":"keep","species":"hunter"}"#).unwrap();
        c.steer(r#"{"kind":"keep","species":"hunter"}"#).unwrap();
        let h = c.autonomous.as_ref().unwrap().physical.as_ref().unwrap();
        assert_eq!(h.steering.sustained, vec!["hunter".to_string()]);
        assert!(c.steer(r#"{"kind":"keep","species":"dragon"}"#).is_err());
    }

    #[test]
    fn champions_name_the_leading_bodies_and_each_can_be_kept_and_painted_back() {
        let mut c = world();
        assert!(c.champions().starts_with('['));
        for key in ["hunter", "swimmer", "rower", "alga"] {
            let (_, json) = c.species_specimen(key).unwrap();
            c.paint_life(&json, &[10., -8.], 6., 12).unwrap();
        }
        for _ in 0..3 {
            c.step(400);
        }
        let e = c.autonomous.as_ref().unwrap();
        let list = e.physical.as_ref().unwrap().champions(e);
        let slots: Vec<_> = list.iter().map(|x| x.slot).collect();
        assert!(slots.contains(&"advanced") && slots.contains(&"swimmer"), "{slots:?}");
        for champion in &list {
            // The handle names a living body of the stated size.
            let group = e.groups().into_iter().find(|g| e.cells[g[0]].id == champion.body).unwrap();
            assert_eq!(group.len(), champion.cells);
            assert!(champion.score > 0. && !champion.note.is_empty());
        }
        let advanced = list.iter().find(|x| x.slot == "advanced").unwrap();
        let kept = c.capture_specimen(advanced.body, advanced.title).unwrap();
        let before = c.autonomous.as_ref().unwrap().cells.len();
        // A large body needs open water to be set down in; try a few places.
        let placed = [[-12., 9.], [18., 18.], [-20., -20.], [22., -18.], [0., 24.], [-24., 0.]]
            .iter()
            .find_map(|at| c.paint_life(&kept, at, 6., 3).ok().filter(|&n| n >= 1))
            .unwrap_or(0);
        assert!(placed >= 1, "{:?} for {} cells", c.place_specimen(&kept, -12., 9.), advanced.cells);
        assert_eq!(c.autonomous.as_ref().unwrap().cells.len(), before + placed as usize * advanced.cells);
        assert_eq!(living(&c, "Most advanced"), placed as usize * advanced.cells);
    }
}
