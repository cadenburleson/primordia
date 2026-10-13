//! Reduced, nondimensional physics and chemistry. This is not atomistic chemistry.
//! Conserved material pools, reaction kinetics, D2Q9 fluid, and material mechanics.
use crate::autonomous::{Bond, Cell, Division, Evolution, ACTIONS, INPUTS, LATERAL, MAX_CELLS};
use crate::body::V3;
use crate::*;

const DX: f64 = 64. / GRID as f64;
/// Chemical energy in a unit of reduced organic matter: what a cell that
/// breathes oxygen captures from it.
pub(crate) const Q: f64 = 22.5;
const FLUID_MASS: f64 = 20.; // solvent mass in each lattice site
const WATER_DENSITY: f64 = 3.;
/// Oxygen freed for each unit of fuel made from light, and used for each
/// unit burned right down.
const OXYGEN_PER_FUEL: f64 = 0.5;
/// Oxygen level in the water at which a cell burns half its fuel fully.
const OXYGEN_HALF: f64 = 0.01;
/// Share of its fuel's energy a cell captures with no oxygen at all: a
/// fifteenth, as fermenting sugar gives 2 units of a cell's energy currency
/// where breathing it gives 30. Life before oxygen is slow life.
pub(crate) const FERMENT: f64 = 1. / 15.;
/// Chance, each moment a mouth is swallowing a light-feeder, that it keeps
/// it alive instead. On Earth it took a billion years to happen once; here
/// it happens a handful of times in a planet's first thousand cycles.
const KEPT_LIGHT: f32 = 0.00002;
/// What a cell spends each moment keeping the breathers inside it alive.
const MITOCHONDRIA_UPKEEP: f64 = 0.02;
/// How easily a body slips through water, beside a rigid sphere in the
/// fluid lattice's own thick medium. Real water is thin: a swimming cell
/// covers many body lengths on little effort.
const SLIP: f64 = 0.08;
/// Drag on a body of unit radius.
const BODY_DRAG: f64 = 6. * std::f64::consts::PI * WATER_DENSITY * NU * SLIP;
/// Chemical energy spent per unit of mechanical work by a motor. A unit of
/// fuel holds far more energy than it takes to push a cell a body length.
const WORK_PRICE: f64 = 0.08;
/// Tissue is viscous as well as springy: a bond resists being stretched or
/// squeezed quickly, so a body in thin water does not ring like a bell.
const BOND_DAMPING: f32 = 3.;
/// Biomass a working set of digestive enzymes takes from a touching cell each
/// cycle. Only a cell that has given up the light digests at this rate, and
/// it has to eat enough this way to pay for swimming after its food.
const GRAZE: f64 = 0.12;
/// A mouth draws in free cells smaller than the body it belongs to, as
/// hunters from ciliates to whales do, so it clears a volume of water
/// instead of having to bump into its food. This is how close to a single
/// full mouth its current runs at full speed. Farther out the same water
/// crosses a wider shell and so moves slower, by the square of the distance.
const FEED_MOUTH: f32 = 0.5;
/// The current is lost in the sea's own stirring once it is this slow, in
/// units a cycle. Where that happens is as far as a body's mouths reach:
/// one and a half units for one mouth, farther for many pumping together.
const FEED_FLOOR: f32 = 0.08;
/// No body draws from farther than this, however many mouths it has.
const FEED_LIMIT: f32 = 8.;
/// The current draws only what the body's mouth can swallow, and as strongly
/// as it can swallow it: nothing a quarter larger than the mouth, at full
/// strength what is two thirds its bulk or less. A bigger mouth draws in
/// bigger prey.
const FILTER_RATIO: f64 = 0.8;
/// Speed of the feeding current at a fully built mouth, in units a cycle.
const FEED_CURRENT: f32 = 1.2;
/// Mass of a newborn cell of the usual size. Bonds and bumps between cells
/// lighter than this are weaker in proportion.
const SLIGHT: f64 = 0.25;
/// What it costs a cell each cycle to carry one gene, beside the 0.055 it
/// costs to be alive at all. Keeping and copying a gene is cheap: about a
/// hundredth of a cell's upkeep. What a gene costs is what it makes, and
/// that is charged where the protein is built. (At eight times this, which
/// it was, a body cell with the two dozen genes an animal needs paid two
/// thirds of its upkeep for carrying them.)
const GENE_UPKEEP: f64 = 0.0005;
/// Share of a free cell's upkeep that a cell fully enclosed by its body pays.
const INTERIOR: f64 = 0.5;
/// How far the sea's eddies carry a free cell in one tick, at natural mixing.
pub(crate) const JOSTLE: f32 = 0.07;
/// How long a mouth is taken up with each thing it swallows, after the
/// last bite of it: four ticks. Long enough that a mouth filling itself
/// on the smallest cells gets a few times what it needs and no more.
const HANDLE: f32 = 1.;
/// How fast what is dissolved crosses a wall two cells share, for each
/// unit of difference between them.
const SEEP: f64 = 0.05;
/// The most a beating body adds to the water its mouths draw: as much again.
const STIR: f32 = 1.;
/// The muscle work a cell-moment, averaged over a body, at which its
/// beating adds half of that.
const STIR_HALF: f32 = 0.01;
/// Seas are stirred far less up and down than along.
const UPWELL: f32 = 0.25;
/// How far the wind carries a spore lying on dry ground in one tick.
const GUST: f32 = 0.5;
/// The mass of a cell small enough for the wind to carry it freely.
const SPORE: f64 = 0.15;
/// How many times faster the wind is clear of the ground than along it.
const LOFT: f32 = 4.;
/// How fast water takes light: each unit of depth leaves about three
/// quarters of it, so the shallows are bright and the deep sea bed is dim.
const CLARITY: f64 = 0.3;
/// How far above the bed the chemistry of hot rock still feeds a cell.
const VENT_REACH: f64 = 1.;
/// How fast loose remains settle through still water.
pub(crate) const SINKING: f32 = 0.004;
/// Pull of the planet on a unit of mass.
pub(crate) const GRAVITY: f64 = 0.3;
/// The weight bare soft tissue can carry before it is crushed, in multiples
/// of its own. Water carries a body's weight for it; land does not.
const SOFT_STRENGTH: f64 = 3.;
/// Injury a cycle to whatever a body stands on, for each multiple of its
/// strength it is overloaded by.
const CRUSH: f32 = 0.05;
/// Injury a cycle to a bare cell in dry air. A dried cell lies dormant, and
/// the drying itself wears it down slowly.
const DRYING: f32 = 0.012;
/// Share of its upkeep a cell with no water stops paying: without water its
/// chemistry all but stops, and it waits for rain as a spore does.
const DORMANCY: f64 = 0.9;
/// Depth of water on the ground that keeps a cell on it wet.
const DAMP: f64 = 0.01;
/// Matter is counted in carbon atoms. A carbon held as fat carries about a
/// third more energy than one held as sugar or protein, because fat carries
/// almost no oxygen of its own: that also makes it under half the weight.
pub(crate) const FAT_ENERGY: f64 = 1.3;
pub(crate) const FAT_WEIGHT: f64 = 0.47;
/// Share of ordinary tissue's upkeep that stored fat costs. Fat is nearly inert.
const FAT_UPKEEP: f64 = 0.2;
/// The rules of chemistry a saved world was last run under. A world saved
/// under older rules has its books reset to what it holds the first time it
/// runs under these.
pub(crate) const CHEMISTRY: u8 = 6;
/// How dense fat and bound mineral are beside water.
const FAT_DENSITY: f64 = 0.9;
const MINERAL_DENSITY: f64 = 2.5;
const TAU: f64 = 0.9;
const NU: f64 = (TAU - 0.5) / 3. * DX * DX / DT as f64;
const C: [(i32, i32); 9] = [
    (0, 0),
    (1, 0),
    (0, 1),
    (-1, 0),
    (0, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
    (1, -1),
];
const W: [f64; 9] = [
    4. / 9.,
    1. / 9.,
    1. / 9.,
    1. / 9.,
    1. / 9.,
    1. / 36.,
    1. / 36.,
    1. / 36.,
    1. / 36.,
];
const OPP: [usize; 9] = [0, 3, 4, 1, 2, 7, 8, 5, 6];
/// Kinds of dissolved matter a site of water or wet ground holds: fuel,
/// carbon dioxide, mineral, two signals, organic remains, nitrogen a plant
/// can take up, nitrogen still bound in remains, dissolved iron, salt,
/// silica, and hydrogen.
pub const SOLUTES: usize = 12;
/// Carbon dioxide: what burning fuel gives off and light-feeders build with.
pub const CO2: usize = 1;
pub const NITROGEN: usize = 6;
pub const REMAINS_N: usize = 7;
/// Iron dissolved in the sea. Free oxygen rusts it out onto the sea bed.
pub const IRON: usize = 8;
/// Salt. Water that evaporates leaves it behind.
pub const SALT: usize = 9;
/// Silica dissolved out of rock. Glass shells are built from it, and it goes
/// back into the water as the shells of the dead dissolve.
pub const SILICA: usize = 10;
/// Hydrogen out of the hot rock at vents: what the first life lived on.
/// It leaks away to the air and to space, so it is found only near a vent.
pub const HYDROGEN: usize = 11;
/// Hydrogen a full-strength vent pours into the water over it each cycle.
/// All of a planet's vents together supply far less than its star does:
/// before any cell split water, the rock's hydrogen was the limit on life.
const VENT_FLOW: f64 = 0.1;
/// Share of dissolved hydrogen lost each cycle.
const H_LOSS: f64 = 0.1;
/// Hydrogen at which a cell's uptake runs at half speed.
const H_HALF: f64 = 0.2;
/// Silica in a whole glass shell, beside the living matter inside it.
pub const SHELL_RATIO: f64 = 0.2;
/// Energy to lay down a unit of shell. Glass is cheap: a wall of carbon
/// costs several times as much, which is much of why diatoms do so well.
const SHELL_PRICE: f64 = 0.05;
/// Glass is about twice as dense as water.
const SILICA_DENSITY: f64 = 2.1;
/// A holdfast pulls its cell back toward the spot it gripped this hard for
/// each unit it is drawn away, and tears free beyond this distance.
const HOLD: f32 = 600.;
const TEAR: f32 = 0.8;
/// Nitrogen bound in each unit of living tissue and protein. Sea life holds
/// about 16 atoms of nitrogen to 106 of carbon. Sugar and fat hold none.
pub const N_RATIO: f64 = 0.15;
/// Phosphorus built into each unit of living tissue, in DNA, membranes and
/// the carriers of a cell's energy: about one atom to a hundred of carbon.
/// It has no gas; it comes from rock, as the dissolved mineral, and a body
/// with bone keeps its reserve there.
pub const P_RATIO: f64 = 0.01;
/// Dissolved nitrogen in a unit of new sea. It is the stock that runs out
/// first, as it is in the open ocean, so it sets how much life a planet
/// holds: about one ordinary cell and a half for each unit of sea.
const SEA_NITROGEN: f64 = 0.12;
/// Nitrogen gas in the air over a unit of surface. No cell can use it as it is.
const AIR_NITROGEN: f64 = 2.;
/// Share of its nitrogen gas a new planet has already let out of its rock.
/// The rest comes out through its vents over the first few hundred cycles
/// and is knocked out by whatever strikes it, so a young planet's air is
/// thin and thickens as it ages.
const AIR_YOUNG: f64 = 0.25;
/// Cycles for vents at their natural strength to let out all but a third
/// of the nitrogen still in the rock.
const AIR_RISE: f64 = 400.;
/// Bodies from space that reach a newborn planet of ordinary size each
/// cycle, the cycles it takes that to fall to a third as the planet sweeps
/// its neighbourhood clean, and what is left when it has.
const BOMBARD_YOUNG: f64 = 0.06;
const BOMBARD_FADE: f64 = 500.;
const BOMBARD_OLD: f64 = 0.0008;
/// The smallest body counted and the largest that ever comes, by size.
/// There are four times as many of half the size.
const BODY_SMALLEST: f64 = 0.5;
const BODY_LARGEST: f64 = 6.;
/// Size a body loses crossing a unit of air. A body burns as it falls, and
/// one that meets its own weight in air on the way down never lands, so
/// thicker air stops bigger ones: a grown planet's air stops three in
/// four, a newborn's about half.
const ABLATION: f64 = 0.22;
/// What a body of size one brings: carbon that burns to carbon dioxide,
/// carbon already in food-like molecules, nitrogen and phosphorus in forms
/// life can use, and iron. Bigger bodies bring as much more as they weigh.
const BODY_CARBON: f64 = 6.;
const BODY_ORGANICS: f64 = 0.6;
const BODY_NITROGEN: f64 = 0.5;
const BODY_PHOSPHORUS: f64 = 0.03;
const BODY_IRON: f64 = 2.;
/// Air nitrogen the heat of a body's fall fixes, as lightning does, and
/// rock nitrogen a strike knocks loose, for a body of size one.
const SHOCK_FIXED: f64 = 0.2;
const SHOCK_GAS: f64 = 4.;
/// Share of the light that the dust of a size-one strike blocks on a planet
/// of ordinary size, and the cycles it takes to settle to a third.
const IMPACT_DUST: f32 = 0.02;
const DUST_SETTLES: f32 = 15.;
/// The air works in layers, and each stops something different.
///
/// High up, where it is thinnest, it takes the star's hardest rays, the
/// ones that break the molecules they hit. Any gas does this, so what gets
/// through depends only on how much air there is. This is how much air
/// cuts those rays to a third.
const HARD_RAY_AIR: f64 = 0.5;
/// How many times over hard rays reaching the ground raise the rate of
/// copying errors, with no air at all.
const HARD_RAY_ERRORS: f32 = 4.;
/// Lower down, ozone takes the ultraviolet. Ozone is made from free
/// oxygen, so a planet has none until something living makes oxygen. This
/// is the share of the oxygen a planet can hold that must be free for
/// ozone to stop half the ultraviolet: about a hundredth, as on Earth.
const OZONE_HALF: f64 = 0.01;
/// Water takes ultraviolet too, and faster than it takes the light that
/// plants use: this many units of water cut it to a third.
const UV_REACH: f32 = 1.;
/// Every cell mends what ultraviolet breaks in it, up to a point: this much
/// it shrugs off, which is about what a good ozone layer lets through from
/// a star bright enough to feed a sea.
const UV_TOLERATED: f32 = 1.;
/// Share of itself a bare cell loses each cycle to each unit of
/// ultraviolet beyond that, and how many times over each unit raises its
/// rate of copying errors. With no ozone, a cell at the surface is dead in
/// a day, and one under two units of water is safe.
const UV_HARM: f32 = 0.017;
const UV_ERRORS: f32 = 1.;
/// Share of a site's air nitrogen that lightning fixes into rain each cycle
/// of storm.
const LIGHTNING: f64 = 0.00004;
/// Share of dissolved nitrogen lost back to the air each cycle where oxygen
/// is scarce but present, by the chemistry of rotting matter.
const DENITRIFY: f64 = 0.0006;
/// Share of the nitrogen bound in remains that leaches out each cycle with
/// nothing eating them. A decomposer frees it far faster.
const LEACH: f64 = 0.004;
/// Energy a cell spends to take in and build with a unit of nitrogen.
const N_PRICE: f64 = 0.6;
/// Dissolved iron and salt in a unit of new sea, and carbon dioxide in the
/// air over a unit of surface.
const SEA_IRON: f64 = 0.1;
const SEA_SALT: f64 = 1.;
/// Silica in the sea: on Earth somewhat more plentiful than usable nitrogen.
const SEA_SILICA: f64 = 0.6;
const AIR_CARBON: f64 = 0.3;
/// Share of the gap between the carbon dioxide a site's water holds and
/// what the air would keep in it that closes each cycle.
const GAS_EXCHANGE: f64 = 0.05;
/// Share of the remains on the bed that silt covers each cycle, out of the
/// reach of anything alive. Burying carbon is what leaves oxygen free.
const BURIAL: f64 = 0.003;
/// Share of buried carbon the planet's heat drives back out each cycle, at
/// natural vent activity, burning it with whatever oxygen is free.
const OUTGAS: f64 = 0.0005;
/// Share of dissolved iron that rusts out each cycle in water full of
/// oxygen, and the oxygen each unit of iron takes with it.
const RUSTING: f64 = 0.05;
const RUST_OXYGEN: f64 = 0.5;
const DIFFUSIVITY: [f64; SOLUTES] = [0.12, 0.18, 0.06, 0.3, 0.3, 0.01, 0.18, 0.01, 0.1, 0.15, 0.1, 0.3];
/// Reads a list of dissolved amounts written before newer kinds were tracked.
pub(crate) fn padded<'de, D: serde::Deserializer<'de>>(d: D) -> Result<[f64; SOLUTES], D::Error> {
    let values = Vec::<f64>::deserialize(d)?;
    if values.len() > SOLUTES {
        return Err(serde::de::Error::custom("Invalid solute vocabulary"));
    }
    let mut s = [0.; SOLUTES];
    s[..values.len()].copy_from_slice(&values);
    Ok(s)
}
fn solute_array<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<[f64; SOLUTES]>, D::Error> {
    Vec::<Vec<f64>>::deserialize(d)?
        .into_iter()
        .map(|values| {
            if !(5..=SOLUTES).contains(&values.len()) {
                return Err(serde::de::Error::custom("Invalid solute vocabulary"));
            }
            let mut s = [0.; SOLUTES];
            s[..values.len()].copy_from_slice(&values);
            Ok(s)
        })
        .collect()
}
pub const SYNTHESIS_NAMES: [&str; ACTIONS] = [
    "Photosynthetic catalyst synthesis",
    "Nutrient transporter synthesis",
    "Adhesive polymer synthesis",
    "Mineral-binding catalyst synthesis",
    "Contractile polymer synthesis",
    "Signal A catalyst synthesis",
    "Signal B catalyst synthesis",
    "Replication catalyst synthesis",
    "Contact transporter synthesis",
    "Digestive catalyst synthesis",
    "Lipid storage catalyst synthesis",
    "Local receptor synthesis",
    "Excitable channel synthesis",
    "Matrix repair catalyst synthesis",
    "Protective membrane synthesis",
    "Detritus-breakdown catalyst synthesis",
    "Photoreceptor synthesis",
    "Adaptive contact polymer synthesis",
    "Gametic division catalyst synthesis",
    "Membrane fusogen synthesis",
    "Polar scaffold synthesis",
    "Extracellular matrix secretion",
    "Hydraulic pump synthesis",
    "Gas exchanger synthesis",
    "Mechanosensitive channel synthesis",
    "Membrane recognition synthesis",
    "Replication repair synthesis",
    "Matrix digesting enzyme synthesis",
    "Fiber crosslinker synthesis",
    "Mineral remodeling enzyme synthesis",
    "Sequence polymer assembler synthesis",
    "Membrane lysis catalyst synthesis",
    "Propulsion motor synthesis",
    "Steering apparatus synthesis",
    "Engulfing membrane synthesis",
    "Venom synthesis",
    "Spine synthesis",
    "Chemosynthetic catalyst synthesis",
    "Fate A commitment",
    "Fate B commitment",
    "Fate C commitment",
    "Cell wall synthesis",
    "Fin synthesis",
    "Sideways division",
    "Cell size regulation",
    "Silica deposition",
    "Holdfast synthesis",
    "Cell elongation",
    "Respiratory chain synthesis",
    "Nitrogenase synthesis",
    "Branch timing",
    "Wing membrane synthesis",
    "Heat-making synthesis",
];
/// Regulatory genes: a bonded stem cell driving one of these past half commits
/// to that fate. They build no material.
pub const BECOME: [usize; 3] = [38, 39, 40];
pub const PROPEL: usize = 32;
pub const STEER: usize = 33;
pub const ENGULF: usize = 34;
pub const TOXIN: usize = 35;
pub const SPINE: usize = 36;
pub const CHEMO: usize = 37;
/// A rigid casing. A walled cell cannot be swallowed whole and is cemented to
/// its walled sisters, but it can neither swallow nor swim: the trade that
/// separates plant-like and fungus-like life from animal-like life. Enzymes
/// still digest it, so grazers and decomposers are what eat walled life.
pub const WALL: usize = 41;
/// Bristles, flaps and feet that feather: they slip easily along the cell's
/// heading and catch the water, or the ground, when pushed back against it.
/// A fin makes no thrust of its own. It turns any back-and-forth stroke, such
/// as a beating muscle, into travel along the heading.
pub const FIN: usize = 42;
/// Bending stiffness of a joint per unit of mineral in its three cells. Bone
/// holds the angle it was laid down at, firmly enough for the muscle at the
/// joint to swing a limb against water or ground.
const JOINT_STRENGTH: f32 = 30.;
/// A full wall holds an angle as firmly as a cell a tenth mineral.
const WALL_STIFFNESS: f32 = 0.1;
/// The most a joint can twist, so a sudden change of target cannot fling a cell.
const JOINT_TORQUE: f32 = 1.;
/// The hardest a joint pushes on any one of the cells it bends about.
const JOINT_PUSH: f32 = 20.;
/// How nearly straight three cells must lie, as the angle between the two
/// arms, for the joint between them to be a backbone's.
const STRAIGHT: f32 = 2.9;
/// Bending stiffness of a backbone's joint per unit of mineral in its three
/// cells: enough for it to follow its muscle within a beat.
const SPINE_STRENGTH: f32 = 120.;
/// The hardest muscle twists a backbone's joint: what the same muscle pulls
/// a tie with, at one cell's width from the joint.
const MUSCLE_TWIST: f32 = 13.;
/// The hardest a backbone's joint pushes on any one of its cells.
const SPINE_PUSH: f32 = 60.;
/// A backbone's own spring toward straight, which muscle works against and
/// no muscle takes away. Without it a line of joints has nothing to stop it
/// folding one way and the other in turn.
const BACKBONE: f32 = 2.;
/// How far its muscle turns a backbone's joint, each way.
const SWING: f32 = 0.45;
/// Where bone meets bone, and how hard.
const STOP: f32 = 0.9;
const STOP_STIFF: f32 = 60.;
/// The share of the way a joint's aim goes, each moment, toward where its
/// muscle is taking it: a muscle takes time to pull a joint round.
const EASE: f32 = 0.5;
/// The cover each tie gives a cell from the water along the tie. A cell in
/// the middle of a line has two, and meets half the water end-on that it
/// meets broadside, as a thin rod does.
const SHIELD: f32 = 0.25;
/// How much wider a full fin makes a cell across its own line.
const FIN_SPAN: f32 = 3.;
/// How firmly a newborn and its mother must hold (the product of the glue each
/// has, of 1) for it to stay in a body with a plan without fail.
const HELD: f32 = 0.25;
/// Stiffness a bond gains from fully built muscle in both its cells.
const MUSCLE_SPRING: f32 = 0.6;
/// Share of the way a muscle goes, each moment, toward as tight as its
/// nerves are telling it to be, and toward slack once they stop.
const MUSCLE_QUICK: f32 = 0.9;
const MUSCLE_SLOW: f32 = 0.4;
/// How hard a working muscle pulls toward its shortened length, beside the
/// springiness of an ordinary bond. Muscle is by far the strongest thing a
/// cell can make: for its cross-section it pulls thousands of times harder
/// than a coat of cilia pushes.
const MUSCLE_PULL: f64 = 300.;
/// A regulatory gene that builds nothing. Driven fully, it swings a polarised
/// cell's division a quarter turn off its heading, so the body widens instead
/// of lengthening. With cell fates it lets one genome grow a body with a
/// length, a width and branches.
pub const BROADEN: usize = 43;
/// A regulatory gene that builds nothing. How hard it is driven sets how
/// large a cell grows before it divides: a quarter of the usual size when
/// silent, the usual size at half drive, four times it at full drive. A
/// genome without the gene makes cells of the usual size.
///
/// Size changes what a cell can do, by geometry. Everything that crosses
/// its surface (light, food, nitrogen, a mouthful, the push of a motor)
/// grows with its area, the two-thirds power of its bulk. What it burns to
/// stay alive and how fast it builds grow with the three-quarter power.
/// So a small cell feeds and breeds fast for its size, and a big one is
/// cheap to run for its size, swims faster and can swallow the small.
pub const SIZE: usize = 44;
/// Lays down a shell of glass from dissolved silica.
pub const SHELL: usize = 45;
/// Grips the ground a cell rests on and holds it there.
pub const HOLDFAST: usize = 46;
/// Regulatory: how long a rod the cell grows as. It builds nothing.
pub const ELONGATE: usize = 47;
/// The machinery of breathing: with it and with free oxygen a cell burns its
/// fuel right down and captures all the energy in it. It sits in the cell's
/// outer membrane, so it shares that membrane with a mouth, and a cell
/// several times the usual size has too little membrane for its bulk. A
/// breather kept alive inside a cell does the same work with neither limit.
pub const RESPIRE: usize = 48;
/// The enzyme that makes the air's nitrogen usable, which nothing else in a
/// cell can do. It costs a great deal of energy and oxygen wrecks it, so it
/// works in water without free oxygen, behind a wall, or deep in a body, and
/// never while the cell is itself splitting water for light.
pub const FIX: usize = 49;
/// A rule of growth, not a material. A growing tip with it forks every so
/// many divisions: one daughter is set off to one side, the cell itself
/// turns to the other, and each goes on to do the same. Every fork's
/// branches fork sooner and are made of smaller cells than the one before,
/// so the body repeats itself at smaller and smaller scales, as a fern, a
/// coral, a gill or a lung does.
pub const BRANCH: usize = 50;
/// A thin stiff membrane that grips air as a fin grips water. With muscle
/// to beat it, it rows a body through the air; moving forward, it carries
/// the body's weight.
pub const WING: usize = 51;
/// Machinery that burns fuel for no purpose but heat. A cell with it works
/// at full pace however cold its surroundings, at a price that fat and
/// being wrapped in other cells bring down.
pub const WARM: usize = 52;
/// Copying errors a cell makes dividing inside a body, beside those of a
/// cell dividing alone.
const SOMATIC: f32 = 0.05;
/// How much harder cilia push with a full set of vanes among them, and
/// while the cell's membrane is firing.
const VANE: f32 = 0.5;
const BOLT: f32 = 1.;
/// Warmth each doubling of the air's carbon dioxide adds, and the amount
/// over a unit of surface at which it adds none.
const GREENHOUSE: f64 = 0.06;
const GREENHOUSE_EVEN: f64 = 2.;
/// The angle each branch of a fork is set off by, in turns of a circle.
const FORK: f32 = 0.1;
/// How much smaller the cells of each fork's branches are than the last.
const TAPER: f32 = 0.85;
/// How well a wing grips air beside how a fin grips water, and the lift a
/// full wing of unit area gives in thick air for each unit of forward
/// speed squared.
const AIR_GRIP: f64 = 0.15;
/// The height over which air thins to a third.
const AIR_HEIGHT: f32 = 2.;
/// How far above the sea or the ground the air ends, for anything alive:
/// seven of those heights, where it is a thousandth as thick.
const SPACE: f32 = 7. * AIR_HEIGHT;
const WING_LIFT: f32 = 0.3;
/// Drag on a full wing for each unit of area, at the same measure. A wing
/// lifts six times what it drags.
const WING_DRAG: f32 = 0.05;
/// Energy a fully warm-blooded bare cell burns each cycle for each unit its
/// surroundings fall short of the warmth it works best at.
const WARM_PRICE: f64 = 0.25;
/// How much less armour and a body's grip on its cells count against a
/// mouth for each part in ten of the mouth that is mineral: teeth.
const JAW: f64 = 20.;
/// How hard each tie to its body holds a cell against a mouth too small to
/// take the body whole. With three ties a cell is taken at a tenth the
/// rate of a loose one: a colony is a refuge from whatever cannot swallow
/// it, which is what first made living together pay.
const TIE: f64 = 3.;
/// How much the matrix of a tissue slows enzymes working in from outside.
const MESH: f64 = 8.;
/// The smallest thing a mouth of many cells keeps hold of, as a share of the
/// mouth's own bulk. Chosen, not measured: it puts the smallest plankton
/// out of reach of the largest mouths here and nothing else.
const LEAK: f64 = 1. / 16.;
/// The most skeleton a cell lays down, as a share of its own bulk.
const SKELETON_SHARE: f64 = 0.15;
/// Energy to fix a unit of nitrogen from the air: about what burning three
/// and a half units of fuel gives, as it costs a real cell.
const FIX_PRICE: f64 = 80.;
/// How many times harder a fully built fin is to push backward than forward.
const FIN_GRIP: f64 = 8.;
/// Share of a cell's own feeding (light, vents, dissolved fuel) given up by a
/// fully built engulfing membrane. Low enough that a hunter between meals can
/// limp along on light, as many single-celled hunters do.
const MOUTH_COST: f64 = 0.85;
/// Thrust of a fully built, fully driven propulsion motor.
const THRUST: f64 = 2.5;
/// Fastest turn of a fully built steering apparatus, radians per cycle.
const TURN_RATE: f32 = 1.2;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub biomass: f64,
    pub fuel: f64,
    pub mineral: f64,
    #[serde(default)]
    pub lipid: f64,
    /// Nitrogen held loose in the cell, ready to build with.
    #[serde(default)]
    pub nitrogen: f64,
    /// Phosphorus held loose in the cell.
    #[serde(default)]
    pub phosphorus: f64,
    /// Glass laid down as a shell.
    #[serde(default)]
    pub silica: f64,
    #[serde(default)]
    pub body: crate::organics::BodyStores,
    #[serde(
        deserialize_with = "crate::autonomous::protein_array",
        serialize_with = "crate::autonomous::as_list"
    )]
    pub proteins: [f64; ACTIONS],
}
impl Inventory {
    pub fn founder() -> Self {
        let mut proteins = [0.; ACTIONS];
        for k in [0, 1, 2, 7, RESPIRE] {
            proteins[k] = 0.002;
        }
        Self {
            biomass: 0.22,
            fuel: 0.038,
            mineral: 0.,
            lipid: 0.,
            silica: 0.,
            nitrogen: 0.015,
            phosphorus: 0.001,
            body: crate::organics::BodyStores::default(),
            proteins,
        }
    }
    pub fn organic(&self) -> f64 {
        self.biomass
            + self.fuel
            + self.lipid
            + self.proteins.iter().sum::<f64>()
            + self.body.organic()
    }
    pub fn reduced(&self) -> f64 {
        self.organic() - self.body.solutes[1]
    }
    /// Nitrogen built into tissue and protein.
    pub fn bound_nitrogen(&self) -> f64 {
        N_RATIO * (self.biomass + self.proteins.iter().sum::<f64>() - self.proteins[WALL])
    }
    /// Chemical energy held beyond what the same carbon would hold as sugar.
    pub fn richness(&self) -> f64 {
        self.lipid * (FAT_ENERGY - 1.)
    }
    pub fn nitrogen_total(&self) -> f64 {
        self.nitrogen
            + self.bound_nitrogen()
            + self.body.solutes[NITROGEN]
            + self.body.solutes[REMAINS_N]
    }
    /// Mineral laid down as bone or carried in the body's fluid.
    pub fn stone(&self) -> f64 {
        self.mineral + self.body.solutes[2]
    }
    /// Phosphorus built into living tissue.
    pub fn bound_phosphorus(&self) -> f64 {
        P_RATIO * self.biomass
    }
    /// Everything this cell holds that came out of rock.
    pub fn mineral_total(&self) -> f64 {
        self.stone() + self.phosphorus + self.bound_phosphorus()
    }
    pub fn mass(&self) -> f64 {
        self.organic() - self.lipid * (1. - FAT_WEIGHT)
            + self.stone()
            + self.silica
            + self.body.water * 3.
    }
    /// How whole this cell's glass shell is, from none to complete.
    pub fn shell(&self) -> f64 {
        (self.silica / (SHELL_RATIO * self.biomass).max(1e-12)).min(1.)
    }
    /// The water a submerged cell pushes aside, as mass. Living tissue is as
    /// dense as water; fat is lighter and mineral heavier, so fat lifts a body
    /// and bone weighs it down.
    pub fn displaced(&self) -> f64 {
        self.mass() + self.lipid * FAT_WEIGHT * (1. / FAT_DENSITY - 1.)
            - self.stone() * (1. - 1. / MINERAL_DENSITY)
            - self.silica * (1. - 1. / SILICA_DENSITY)
    }
    pub fn radius(&self) -> f32 {
        // Organic and mineral densities differ; precipitation increases inertia.
        (0.22
            * ((self.organic() - self.lipid * (1. - FAT_WEIGHT / FAT_DENSITY)
                + self.mineral * 0.3
                + self.silica / SILICA_DENSITY
                + self.body.water * 3.)
                / 0.268)
                .cbrt()) as f32
    }
    pub fn activity(&self, k: usize) -> f32 {
        // Most cells make only a few of the proteins there are.
        if self.proteins[k].to_bits() == 0 {
            return 0.;
        }
        (self.proteins[k] / (0.015 * self.biomass + self.proteins[k]).max(1e-12)) as f32
    }
    /// How much of this cell is bone, and how much is laid-down fibre.
    pub fn bone(&self) -> f64 {
        self.stone() / (0.1 * self.biomass + self.stone()).max(1e-12)
    }
    pub fn fibre(&self) -> f64 {
        let laid = self.body.matrix + self.body.fibers;
        laid / (0.1 * self.biomass + laid).max(1e-12)
    }
    /// How thickly fat sheathes this cell. Sheathed nerves pass a signal on
    /// several times faster, which is what lets a large body think quickly.
    pub fn sheath(&self) -> f64 {
        self.lipid / (0.25 * self.biomass + self.lipid).max(1e-12)
    }
    fn split(&mut self) -> Self {
        self.biomass *= 0.5;
        self.fuel *= 0.5;
        self.mineral *= 0.5;
        self.lipid *= 0.5;
        self.nitrogen *= 0.5;
        self.phosphorus *= 0.5;
        self.silica *= 0.5;
        for p in &mut self.proteins {
            *p *= 0.5;
        }
        self.body.scale(0.5);
        self.clone()
    }
    pub(crate) fn valid(&self) -> bool {
        [self.biomass, self.fuel, self.mineral, self.lipid, self.silica]
            .iter()
            .chain(self.proteins.iter())
            .all(|x| x.is_finite() && *x >= 0. && *x < 1e6)
            && self.biomass > 0.
            && self.mass() > 0.
            && self.body.valid()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fluid {
    #[serde(default)]
    pub domain: crate::domain::Domain,
    pub populations: Vec<[f64; 9]>,
    #[serde(default)]
    pub surface: Option<crate::surface::SurfaceFlow>,
}
impl Fluid {
    fn equilibrium(rho: f64, u: [f64; 2]) -> [f64; 9] {
        let mut f = [0.; 9];
        for q in 0..9 {
            let cu = C[q].0 as f64 * u[0] + C[q].1 as f64 * u[1];
            f[q] = W[q] * rho * (1. + 3. * cu + 4.5 * cu * cu - 1.5 * (u[0] * u[0] + u[1] * u[1]));
        }
        f
    }
    pub fn still() -> Self {
        Self::still_in(crate::domain::Domain::default())
    }
    pub fn still_in(domain: crate::domain::Domain) -> Self {
        Self {
            domain,
            populations: if domain.spherical() {
                vec![]
            } else {
                vec![W; domain.sites()]
            },
            surface: domain
                .spherical()
                .then(|| crate::surface::SurfaceFlow::new(domain)),
        }
    }
    pub fn new() -> Self {
        Self::new_in(crate::domain::Domain::default())
    }
    pub fn new_in(domain: crate::domain::Domain) -> Self {
        let mut f = Self::still_in(domain);
        if domain.spherical() {
            return f;
        }
        let grid = domain.grid;
        // One low-Mach, approximately divergence-free initial vortex; no forcing
        // or nutrient steering sustains it. No-slip walls dissipate its momentum.
        for z in 0..grid {
            for x in 0..grid {
                let a = std::f64::consts::PI * (x as f64 + 0.5) / grid as f64;
                let b = std::f64::consts::PI * (z as f64 + 0.5) / grid as f64;
                let u = [
                    0.018 * a.sin().powi(2) * (2. * b).sin(),
                    -0.018 * b.sin().powi(2) * (2. * a).sin(),
                ];
                f.populations[z * grid + x] = Self::equilibrium(1., u);
            }
        }
        f
    }
    pub fn lattice_velocity(&self, site: usize) -> [f64; 2] {
        if self.domain.spherical() {
            let v = self.surface.as_ref().unwrap().velocities[site];
            let (x, z) = self.domain.point(site);
            let p = self.domain.position(x, z, 0.);
            return [
                v.dot(self.domain.tangent(p, 0.)) as f64 * DT as f64 / DX,
                v.dot(self.domain.tangent(p, std::f32::consts::FRAC_PI_2)) as f64 * DT as f64 / DX,
            ];
        }
        let f = &self.populations[site];
        let rho = f.iter().sum::<f64>();
        let mut u = [0.; 2];
        for q in 0..9 {
            u[0] += f[q] * C[q].0 as f64 / rho;
            u[1] += f[q] * C[q].1 as f64 / rho;
        }
        u
    }
    pub fn velocity(&self, site: usize) -> [f64; 2] {
        let u = self.lattice_velocity(site);
        [u[0] * DX / DT as f64, u[1] * DX / DT as f64]
    }
    pub fn momentum(&self) -> [f64; 2] {
        let mut p = [0.; 2];
        for f in &self.populations {
            for q in 0..9 {
                p[0] += f[q] * C[q].0 as f64 * FLUID_MASS * DX / DT as f64;
                p[1] += f[q] * C[q].1 as f64 * FLUID_MASS * DX / DT as f64;
            }
        }
        p
    }
    fn impulse(&mut self, site: usize, impulse: [f64; 2]) {
        // Linear moment update has zero zeroth moment and the exact requested
        // first moment. It is the reaction to drag, not a prescribed current.
        let j = [
            impulse[0] / FLUID_MASS * DT as f64 / DX,
            impulse[1] / FLUID_MASS * DT as f64 / DX,
        ];
        for q in 0..9 {
            self.populations[site][q] += 3. * W[q] * (C[q].0 as f64 * j[0] + C[q].1 as f64 * j[1]);
        }
    }
    pub fn step(&mut self) {
        if self.domain.spherical() {
            return;
        }
        let mut next = vec![[0.; 9]; self.domain.sites()];
        for z in 0..self.domain.rows() {
            for x in 0..self.domain.grid {
                let site = z * self.domain.grid + x;
                let f = self.populations[site];
                let rho = f.iter().sum();
                let eq = Self::equilibrium(rho, self.lattice_velocity(site));
                for q in 0..9 {
                    let post = f[q] + (eq[q] - f[q]) / TAU;
                    let xx = x as i32 + C[q].0;
                    let zz = z as i32 + C[q].1;
                    if xx < 0
                        || zz < 0
                        || xx >= self.domain.grid as i32
                        || zz >= self.domain.grid as i32
                    {
                        next[site][OPP[q]] += post; // halfway no-slip bounce-back
                    } else {
                        next[zz as usize * self.domain.grid + xx as usize][q] += post;
                    }
                }
            }
        }
        self.populations = next;
    }
    fn valid(&self) -> bool {
        if self.domain.spherical() {
            return self.domain.valid()
                && self.populations.is_empty()
                && self.surface.as_ref().is_some_and(|f| f.valid(self.domain));
        }
        if self.surface.is_some() {
            return false;
        }
        self.domain.valid()
            && self.populations.len() == self.domain.sites()
            && self.populations.iter().enumerate().all(|(i, f)| {
                f.iter().all(|v| v.is_finite() && *v > 0. && *v < 2.)
                    && (0.5..1.5).contains(&f.iter().sum::<f64>())
                    && self.lattice_velocity(i).iter().all(|v| v.abs() < 0.18)
            })
    }
}

/// A bending interaction between two mineral-bearing adhesive contacts.
/// Created from actual contact geometry, not a supplied skeleton/body plan.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialJoint {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub rest: f32,
    pub target: f32,
    pub angle: f32,
    pub flexion: f32,
    pub extension: f32,
    pub work: f64,
    /// Laid down in a straight and level line, a joint is a backbone's: it
    /// can turn to either side, and which side matters. `bend` is how far
    /// it is turned to the left (less than nothing, to the right) and `aim`
    /// where its muscle is taking it. `angle` and `target` still hold the
    /// plain angle between its two arms, for whoever reads those.
    #[serde(default)]
    pub straight: bool,
    #[serde(default)]
    pub bend: f32,
    #[serde(default)]
    pub aim: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Habitat {
    /// Deaths since this world was loaded, by starter species: starved,
    /// wasted away, and then each kind of injury in `physiology::HARMS`.
    #[serde(skip)]
    pub deaths: std::collections::BTreeMap<String, [u32; 13]>,
    /// Living tissue taken since this world was loaded, by the starter
    /// species it was taken from and the one that took it.
    #[serde(skip)]
    pub taken: std::collections::BTreeMap<(String, String), f64>,
    /// Cells born since this world was loaded, by starter species.
    #[serde(skip)]
    pub births: std::collections::BTreeMap<String, u32>,
    /// The lines each starter species has split into: every set of
    /// adaptations seen in it, what it arose from and when.
    #[serde(default)]
    pub lines: std::collections::BTreeMap<String, Vec<crate::steering::Line>>,
    #[serde(default)]
    pub ablation: String,
    #[serde(default)]
    pub meals: Vec<crate::observations::Meal>,
    #[serde(default)]
    pub shade: Vec<f32>,
    #[serde(default)]
    pub domain: crate::domain::Domain,
    #[serde(default)]
    pub planet: Option<crate::planet::Planet>,
    #[serde(default)]
    pub physiology: crate::physiology::Ledger,
    #[serde(default)]
    pub development: crate::development::Development,
    #[serde(default)]
    pub systems: crate::organics::SystemLedger,
    #[serde(default)]
    pub steering: crate::steering::Steering,
    #[serde(default)]
    pub joints: Vec<MaterialJoint>,
    #[serde(default)]
    pub passive: bool,
    /// Fuel, oxidized waste, mineral, signal A/B, and organic detritus.
    #[serde(deserialize_with = "solute_array")]
    pub solutes: Vec<[f64; SOLUTES]>,
    pub fluid: Fluid,
    pub initial_organic: f64,
    pub initial_mineral: f64,
    pub initial_energy: f64,
    /// All the nitrogen there is: in the air, the water, remains and the living.
    #[serde(default)]
    pub initial_nitrogen: f64,
    /// Nitrogen gas in the air. No cell can use it until lightning fixes it.
    #[serde(default)]
    pub air_nitrogen: f64,
    /// Carbon dioxide in the air.
    #[serde(default)]
    pub air_carbon: f64,
    /// Nitrogen gas still in the planet's rock. Vents let it out and
    /// strikes knock it loose; nothing alive can reach it.
    #[serde(default)]
    pub rock_nitrogen: f64,
    /// Bodies from space that reached the ground, and those that burned up
    /// in the air on the way.
    #[serde(default)]
    pub impacts: u32,
    #[serde(default)]
    pub airbursts: u32,
    /// Carbon and nitrogen of remains that silt has covered. Nothing alive
    /// can reach them; the planet's heat slowly drives them back out.
    #[serde(default)]
    pub buried_carbon: f64,
    #[serde(default)]
    pub buried_nitrogen: f64,
    /// Iron that oxygen has rusted out of the sea onto its bed.
    #[serde(default)]
    pub rusted_iron: f64,
    /// Who feeds on whom, measured.
    #[serde(default)]
    pub web: crate::trophic::Web,
    /// The chemistry rules this world's books were last balanced under.
    #[serde(default)]
    pub chemistry: u8,
    pub photon_energy: f64,
    /// Chemical energy supplied by the planet's interior to chemosynthesis.
    #[serde(default)]
    pub geothermal_energy: f64,
    pub heat: f64,
    pub mechanical_input: f64,
    pub bound_events: u32,
    pub dissociations: u32,
}
impl Habitat {
    pub fn new(core: &Core, cells: &mut [Cell]) -> Self {
        let mut domain = crate::domain::Domain::from_size(core.config.world_size);
        domain.shape = core.config.world_shape;
        let mut solutes = vec![[0.; SOLUTES]; domain.sites()];
        for (i, s) in solutes.iter_mut().enumerate() {
            s[0] = if domain == crate::domain::Domain::default() {
                core.field[i] as f64
            } else {
                (0.25
                    + noise(
                        (i % domain.grid) as f32 / 4.,
                        (i / domain.grid) as f32 / 4.,
                        core.config.seed,
                    ) * 0.75) as f64
                    * core.config.nutrients as f64
                    * 18.
            };
            s[1] = 0.3;
            let (x, z) = domain.point(i);
            let density = domain.area(i) / (DX * DX);
            s[0] *= density;
            s[1] *= density;
            s[2] = core.minerals_at(x, z) as f64 * 0.4 * density;
        }
        let mut shade = Vec::new();
        if core.config.resource_layout == "patchy" {
            let before: f64 = solutes.iter().map(|s| s[0]).sum();
            shade = (0..domain.sites())
                .map(|i| {
                    let (x, z) = domain.point(i);
                    let n = if domain.spherical() {
                        let n = domain.normal(x, z);
                        noise(
                            n.x * 25. + 20.,
                            n.y * 25. + 20.,
                            core.config.seed.wrapping_add(314),
                        )
                    } else {
                        noise(
                            x * 0.25 + 20.,
                            z * 0.25 + 20.,
                            core.config.seed.wrapping_add(314),
                        )
                    };
                    let patch = ((n - 0.5) * 4.).clamp(0., 1.);
                    solutes[i][0] *= 0.001 + patch as f64;
                    0.02 + 0.98 * patch
                })
                .collect();
            let after: f64 = solutes.iter().map(|s| s[0]).sum();
            if after > 0. {
                for s in &mut solutes {
                    s[0] *= before / after;
                }
            }
        }
        for cell in cells.iter_mut() {
            cell.material = Some(Inventory::founder());
            let ground = domain.ground(cell.p.x, cell.p.z, core.config.seed);
            let height = if ground > 0.12 {
                ground + cell.radius()
            } else {
                0.12 + cell.radius() * (1. - 2. * 0.78)
            };
            cell.p = domain.position(cell.p.x, cell.p.z, height);
        }
        let mut h = Self {
            ablation: String::new(),
            deaths: Default::default(),
            taken: Default::default(),
            births: Default::default(),
            lines: Default::default(),
            meals: Vec::new(),
            shade,
            domain,
            planet: Some({
                let mut p = crate::planet::Planet::new_in(domain);
                p.initialize_surface(core.config.seed);
                p
            }),
            physiology: crate::physiology::Ledger::default(),
            development: crate::development::Development::default(),
            systems: crate::organics::SystemLedger::default(),
            steering: crate::steering::Steering::default(),
            joints: Vec::new(),
            passive: core.config.physical_assay == "passive",
            solutes,
            fluid: Fluid::new_in(domain),
            initial_organic: 0.,
            initial_mineral: 0.,
            initial_energy: 0.,
            initial_nitrogen: 0.,
            air_nitrogen: 0.,
            rock_nitrogen: 0.,
            impacts: 0,
            airbursts: 0,
            air_carbon: 0.,
            buried_carbon: 0.,
            buried_nitrogen: 0.,
            rusted_iron: 0.,
            web: crate::trophic::Web::default(),
            chemistry: CHEMISTRY,
            photon_energy: 0.,
            geothermal_energy: 0.,
            heat: 0.,
            mechanical_input: 0.,
            bound_events: 0,
            dissociations: 0,
        };
        h.seed_elements(true);
        h.seed_silica();
        if let Some(p) = &mut h.planet {
            p.seed_vents(core.config.seed);
        }
        let (organic, mineral, energy) = h.totals(cells);
        h.initial_organic = organic;
        h.initial_mineral = mineral;
        h.initial_energy = energy;
        h.seed_nitrogen(cells);
        h
    }
    /// Gives a world its nitrogen: gas in the air, a little dissolved in
    /// every site, and a working store in each living cell. Worlds saved
    /// before nitrogen was tracked receive theirs the first time they run.
    pub(crate) fn seed_nitrogen(&mut self, cells: &mut [Cell]) {
        for i in 0..self.domain.sites() {
            let density = self.domain.area(i) / (DX * DX);
            self.solutes[i][NITROGEN] += SEA_NITROGEN * density;
            // A planet with a sky starts with part of its air; a dish in
            // a laboratory has all of it.
            let out = if self.domain.spherical() { AIR_YOUNG } else { 1. };
            self.air_nitrogen += AIR_NITROGEN * density * out;
            self.rock_nitrogen += AIR_NITROGEN * density * (1. - out);
        }
        for m in cells.iter_mut().filter_map(|c| c.material.as_mut()) {
            m.nitrogen = m.nitrogen.max(m.bound_nitrogen() * 0.5);
        }
        self.initial_nitrogen = self.nitrogen_total(cells);
    }
    /// Gives a world its iron and salt, and a new world the carbon dioxide
    /// of its air. Worlds saved before these were tracked receive the first
    /// two when they next run; their air fills from their seas.
    pub(crate) fn seed_elements(&mut self, new: bool) {
        for i in 0..self.domain.sites() {
            let density = self.domain.area(i) / (DX * DX);
            self.solutes[i][IRON] += SEA_IRON * density;
            self.solutes[i][SALT] += SEA_SALT * density;
            if new {
                self.air_carbon += AIR_CARBON * density;
            }
        }
    }
    /// Gives a world's seas their dissolved silica. Worlds saved before
    /// silicon was tracked receive it when they next run.
    pub(crate) fn seed_silica(&mut self) {
        for i in 0..self.domain.sites() {
            self.solutes[i][SILICA] += SEA_SILICA * self.domain.area(i) / (DX * DX);
        }
    }
    /// All the silicon on the planet: dissolved, in grains of remains, and
    /// in the shells of the living. Nothing makes or destroys it.
    pub fn silicon_total(&self, cells: &[Cell]) -> f64 {
        self.solutes.iter().map(|s| s[SILICA]).sum::<f64>()
            + self.systems.particles.iter().map(|p| p.solutes[SILICA]).sum::<f64>()
            + cells
                .iter()
                .filter_map(|c| c.material.as_ref())
                .map(|m| m.silica + m.body.solutes[SILICA])
                .sum::<f64>()
    }
    pub fn nitrogen_total(&self, cells: &[Cell]) -> f64 {
        self.air_nitrogen
            + self.rock_nitrogen
            + self.buried_nitrogen
            + self
                .solutes
                .iter()
                .map(|s| s[NITROGEN] + s[REMAINS_N])
                .sum::<f64>()
            + self
                .systems
                .particles
                .iter()
                .map(|p| p.solutes[NITROGEN] + p.solutes[REMAINS_N])
                .sum::<f64>()
            + cells
                .iter()
                .filter_map(|c| c.material.as_ref())
                .map(Inventory::nitrogen_total)
                .sum::<f64>()
    }
    /// Resets every conservation ledger to what the world holds now.
    pub fn rebase(&mut self, cells: &[Cell]) {
        (self.initial_organic, self.initial_mineral, self.initial_energy) = self.totals(cells);
        self.initial_nitrogen = self.nitrogen_total(cells);
    }
    pub fn totals(&self, cells: &[Cell]) -> (f64, f64, f64) {
        // Buried remains still hold their chemical energy; carbon dioxide holds none.
        let mut organic = self.air_carbon + self.buried_carbon;
        let mut mineral = 0.;
        let mut reduced = self.buried_carbon;
        let mut atp = 0.;
        for s in &self.solutes {
            organic += s[0] + s[1] + s[3] + s[4] + s[5];
            reduced += s[0] + s[3] + s[4] + s[5];
            mineral += s[2];
        }
        for c in cells {
            if let Some(m) = &c.material {
                organic += m.organic();
                reduced += m.reduced() + m.richness();
                mineral += m.mineral_total();
                atp += c.energy as f64;
            }
        }
        for particle in &self.systems.particles {
            let s = particle.solutes;
            organic += s[0] + s[1] + s[3] + s[4] + s[5];
            reduced += s[0] + s[3] + s[4] + s[5];
            mineral += s[2];
        }
        (organic, mineral, reduced * Q + atp)
    }
    /// Matter cannot be made. Whatever the creator has just placed or taken
    /// away is balanced against the planet's own stores: carbon against the
    /// carbon dioxide in its water, mineral against dissolved mineral,
    /// nitrogen against the air. Only the energy is new, as sunlight is.
    pub(crate) fn settle_arrivals(&mut self, before: (f64, f64, f64), nitrogen: f64, cells: &[Cell]) {
        let after = self.totals(cells);
        let surface: f64 = (0..self.domain.sites()).map(|i| self.domain.area(i)).sum();
        for (k, change) in [(CO2, after.0 - before.0), (2, after.1 - before.1)] {
            let pool: f64 = self.solutes.iter().map(|s| s[k]).sum();
            let taken = if change > 0. { change.min(pool) } else { change };
            for i in 0..self.domain.sites() {
                let share = if taken > 0. {
                    self.solutes[i][k] / pool
                } else {
                    self.domain.area(i) / surface
                };
                // Never below nothing, down to the last rounding.
                self.solutes[i][k] = (self.solutes[i][k] - taken * share).max(0.);
            }
            // A planet with too little left to give takes the rest from outside.
            if k == CO2 {
                self.initial_organic += change - taken;
            } else {
                self.initial_mineral += change - taken;
            }
        }
        self.initial_energy += self.totals(cells).2 - before.2;
        let change = self.nitrogen_total(cells) - nitrogen;
        let taken = change.min(self.air_nitrogen);
        self.air_nitrogen -= taken;
        self.initial_nitrogen += change - taken;
    }
    /// The planet's own chemistry, with no life in it. Each substance moves
    /// between the air, the water and the ground by where it is and how warm:
    /// carbon dioxide between air and water, more of it held by cold water;
    /// remains silted over and later driven back out by the planet's heat;
    /// iron rusted out of the sea by free oxygen; nitrogen fixed by
    /// lightning, leached from remains, and lost where oxygen is scarce.
    pub(crate) fn planet_chemistry(&mut self, dt: f64) {
        let sites = self.domain.sites();
        let surface: f64 = (0..sites).map(|i| self.domain.area(i)).sum();
        let mut lost = 0.;
        let air = self.air_carbon / surface;
        let (mut breathed, mut free) = (0., 0.);
        for i in 0..sites {
            let area = self.domain.area(i);
            let warmth = self.planet.as_ref().map_or(0.5, |p| p.temperature_at(i)) as f64;
            let s = &mut self.solutes[i];
            // Cold water holds more gas than warm.
            let held = air * area * (1.5 - warmth).clamp(0.5, 1.5);
            let moved = ((held - s[CO2]) * (1. - (-GAS_EXCHANGE * dt).exp())).max(-s[CO2]);
            s[CO2] += moved;
            breathed += moved;
            // Vents pour hydrogen into the water over them, and wherever it
            // is it leaks away, so it is plentiful only near a vent.
            let vent = self.planet.as_ref().map_or(0., |p| p.vent(i));
            s[HYDROGEN] = (s[HYDROGEN] + VENT_FLOW * vent * self.steering.vents as f64 * area / (DX * DX) * dt)
                * (-H_LOSS * dt).exp();
            let fraction = 1. - (-BURIAL * dt).exp();
            self.buried_carbon += s[5] * fraction;
            self.buried_nitrogen += s[REMAINS_N] * fraction;
            s[5] -= s[5] * fraction;
            s[REMAINS_N] -= s[REMAINS_N] * fraction;
            if let Some(gas) = self.systems.gas.get_mut(i) {
                let level = gas[0] / (area / 2.56);
                let rusted = (s[IRON] * (1. - (-RUSTING * level / (OXYGEN_HALF + level) * dt).exp()))
                    .min(gas[0] / RUST_OXYGEN);
                s[IRON] -= rusted;
                self.rusted_iron += rusted;
                let used = (rusted * RUST_OXYGEN).min(gas[0]);
                gas[0] -= used;
                gas[1] += used;
                free += gas[0];
            }
        }
        // The air cannot give more than it holds, down to the last rounding.
        if breathed > self.air_carbon {
            let excess = breathed - self.air_carbon;
            let dissolved: f64 = self.solutes.iter().map(|s| s[CO2]).sum();
            for s in &mut self.solutes {
                s[CO2] -= excess * s[CO2] / dissolved;
            }
            breathed = self.air_carbon;
        }
        self.air_carbon -= breathed;
        // Buried carbon comes back as carbon dioxide only as fast as there
        // is free oxygen to burn it with. Its nitrogen returns to the air.
        let fraction = 1. - (-OUTGAS * self.steering.vents as f64 * dt).exp();
        let burned = (self.buried_carbon * fraction).min(free / OXYGEN_PER_FUEL);
        if burned > 0. && free > 0. {
            let share = burned / self.buried_carbon.max(1e-12);
            let used = burned * OXYGEN_PER_FUEL;
            for gas in &mut self.systems.gas {
                let taken = (used * gas[0] / free).min(gas[0]);
                gas[0] -= taken;
                gas[1] += taken;
            }
            let nitrogen = self.buried_nitrogen * share;
            self.buried_carbon -= burned;
            self.buried_nitrogen -= nitrogen;
            self.air_carbon += burned;
            self.air_nitrogen += nitrogen;
            self.heat += burned * Q;
        }
        for i in 0..sites {
            let s = &mut self.solutes[i];
            let leached = s[REMAINS_N] * (1. - (-LEACH * dt).exp());
            s[REMAINS_N] -= leached;
            s[NITROGEN] += leached;
            if let Some(gas) = self.systems.gas.get(i) {
                let level = gas[0] / (self.domain.area(i) / 2.56);
                let o = level / (OXYGEN_HALF + level);
                let gone = s[NITROGEN] * (1. - (-DENITRIFY * 4. * o * (1. - o) * dt).exp());
                s[NITROGEN] -= gone;
                lost += gone;
            }
        }
        self.air_nitrogen += lost;
        let Some(p) = self.planet.as_ref().filter(|p| p.storm > 0.) else {
            return;
        };
        let surface: f64 = (0..sites).map(|i| self.domain.area(i)).sum();
        let air = self.air_nitrogen;
        for i in 0..sites {
            let weight =
                (1. - p.distance_from(i, p.storm_center) / p.storm_radius).max(0.) as f64;
            let strike = (air * self.domain.area(i) / surface * LIGHTNING * weight * dt)
                .min(self.air_nitrogen);
            self.solutes[i][NITROGEN] += strike;
            self.air_nitrogen -= strike;
        }
    }
    /// Share of the star's ultraviolet that ozone stops. Ozone forms from
    /// free oxygen high in the air, so there is none until life makes oxygen.
    pub fn ozone(&self) -> f64 {
        let free: f64 = self.systems.gas.iter().map(|g| g[0]).sum();
        let full: f64 = (0..self.domain.sites())
            .map(|i| crate::organics::AIR * self.domain.area(i) / 2.56)
            .sum();
        if full > 0. { free / (free + OZONE_HALF * full) } else { 0. }
    }
    /// Share of the star's hardest rays that reach the ground through
    /// the air there is. A dish in a laboratory is under a whole sky.
    pub fn hard_rays(&self) -> f64 {
        if self.domain.spherical() { (-self.air_column() / HARD_RAY_AIR).exp() } else { 0. }
    }
    /// Share of the bodies that arrive from space which burn up before
    /// they reach the ground, through the air there is now.
    pub fn burned_up(&self) -> f64 {
        1. - (BODY_SMALLEST / (ABLATION * self.air_column()).max(BODY_SMALLEST)).powi(2)
    }
    /// How deep the water is at a site, where there is a planet to say.
    fn planet_depth(&self, site: usize) -> Option<f32> {
        self.planet.as_ref().map(|p| p.surface_depth(site) as f32)
    }
    /// Air over a unit of the planet's surface: what a falling body has to
    /// get through.
    pub fn air_column(&self) -> f64 {
        let surface: f64 = (0..self.domain.sites()).map(|i| self.domain.area(i)).sum();
        (self.air_nitrogen + self.air_carbon) / (surface / (DX * DX))
    }
    /// A young planet is struck often, by whatever is still loose in its
    /// neighbourhood, and less and less as it sweeps that up. Each body
    /// burns as it falls. In thin air most reach the ground: they kill what
    /// they land on, dig a crater, throw up dust that dims the star, and
    /// leave what they were made of where they fell. In thick air all but
    /// the largest burn up, and what they were made of settles over the
    /// whole planet as dust. Either way the planet gains it. Meanwhile the
    /// rock gives up its own gas and the air thickens.
    fn bombard(&mut self, e: &mut Evolution, core: &mut Core, planet: &mut crate::planet::Planet) {
        planet.dust *= (-DT / DUST_SETTLES).exp();
        if !self.domain.spherical() {
            return;
        }
        let dt = DT as f64;
        let out = self.rock_nitrogen * (1. - (-dt * self.steering.vents as f64 / AIR_RISE).exp());
        self.rock_nitrogen -= out;
        self.air_nitrogen += out;
        let d = self.domain;
        let surface: f64 = (0..d.sites()).map(|i| d.area(i)).sum();
        // How many ordinary sites the planet's surface would make.
        let extent = surface / (DX * DX);
        let rate = self.steering.bombardment as f64
            * extent / 2000.
            * (BOMBARD_YOUNG * (-(core.time as f64) / BOMBARD_FADE).exp() + BOMBARD_OLD);
        // Its own dice, so that a quiet sky and a busy one leave everything
        // else on the planet to fall out the same way.
        let mut state = (core.config.seed as u64) << 32 | (core.time / DT).round() as u64;
        let mut dice = || {
            state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = state;
            z = (z ^ z >> 30).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ z >> 27).wrapping_mul(0x94D0_49BB_1331_11EB);
            ((z ^ z >> 31) >> 11) as f64 / (1u64 << 53) as f64
        };
        if dice() >= rate * dt {
            return;
        }
        let size = (BODY_SMALLEST / dice().max(1e-9).sqrt()).min(BODY_LARGEST);
        let landed = (size - ABLATION * self.air_column()).max(0.);
        // Where it comes down: anywhere, as likely as there is surface.
        let mut pick = dice() * surface;
        let site = (0..d.sites())
            .find(|&i| {
                pick -= d.area(i);
                pick <= 0.
            })
            .unwrap_or(d.sites() - 1);
        let (x, z) = d.point(site);
        let (bulk, ground) = (size.powi(3), landed.powi(3));
        let radius = if landed > 0. { (1.5 + 1.5 * landed as f32).min(d.size() * 0.2) } else { 0. };
        let centre = d.normal(x, z);
        let near: Vec<(usize, f64)> = (0..d.sites())
            .filter_map(|i| {
                let (sx, sz) = d.point(i);
                let apart = centre.dot(d.normal(sx, sz)).clamp(-1., 1.).acos() * d.radius();
                (apart < radius).then(|| (i, 1. - (apart / radius) as f64))
            })
            .collect();
        // What burned up settles everywhere; what landed stays where it fell.
        let crater: f64 = near.iter().map(|&(i, _)| d.area(i)).sum();
        let fixed = (SHOCK_FIXED * bulk).min(self.air_nitrogen);
        self.air_nitrogen -= fixed;
        for i in 0..d.sites() {
            let here = near.iter().any(|&(k, _)| k == i);
            let share = (bulk - ground) * d.area(i) / surface
                + if here { ground * d.area(i) / crater } else { 0. };
            let s = &mut self.solutes[i];
            s[0] += BODY_ORGANICS * share;
            s[NITROGEN] += BODY_NITROGEN * share + fixed * d.area(i) / surface;
            s[2] += BODY_PHOSPHORUS * share;
            s[IRON] += BODY_IRON * share;
        }
        self.air_carbon += BODY_CARBON * bulk;
        // The planet is that much the richer, and its books say so.
        self.initial_organic += (BODY_CARBON + BODY_ORGANICS) * bulk;
        self.initial_energy += BODY_ORGANICS * bulk * Q;
        self.initial_nitrogen += BODY_NITROGEN * bulk;
        planet.mineral_input += BODY_PHOSPHORUS * bulk;
        if landed <= 0. {
            self.airbursts += 1;
            return;
        }
        self.impacts += 1;
        let loosed = (SHOCK_GAS * ground).min(self.rock_nitrogen);
        self.rock_nitrogen -= loosed;
        self.air_nitrogen += loosed;
        planet.dust = (planet.dust + IMPACT_DUST * ground as f32 * (2000. / extent as f32)).min(0.9);
        for &(i, depth) in &near {
            planet.relief[i] = (planet.relief[i] - 0.3 * landed as f32 * depth as f32).clamp(-8., 8.);
        }
        planet.terrain_version = planet.terrain_version.wrapping_add(1);
        let struck = d.position(x, z, d.ground(x, z, core.config.seed) + planet.relief_at(x, z));
        let mut lost = 0;
        for c in &mut e.cells {
            if c.p.sub(struck).len() < radius {
                c.state.injury = 1.;
                c.state.harm[0] += 1.;
                lost += 1;
            }
        }
        core.event(
            "planet",
            format!(
                "A {} body from space struck the planet. It left a crater {:.0} wide and killed {lost} cells; what it was made of is now the planet's.",
                if landed < 1. { "small" } else if landed < 3. { "large" } else { "huge" },
                radius * 2.
            ),
        );
    }
    pub fn errors(&self, cells: &[Cell]) -> [f64; 4] {
        // Books kept under older rules are reset when the world next runs.
        if self.chemistry != CHEMISTRY {
            return [0.; 4];
        }
        let (o, m, e) = self.totals(cells);
        [
            o - self.initial_organic,
            m - self.initial_mineral - self.planet.as_ref().map_or(0., |p| p.mineral_input),
            e + self.heat + self.mechanical_input
                - self.initial_energy
                - self.photon_energy
                - self.geothermal_energy,
            if self.initial_nitrogen > 0. {
                self.nitrogen_total(cells) - self.initial_nitrogen
            } else {
                0.
            },
        ]
    }
    pub fn valid(&self, cells: &[Cell]) -> bool {
        let ids: BTreeSet<_> = cells.iter().map(|c| c.id).collect();
        let mut keys = BTreeSet::new();
        ["", "motors", "transport", "signals"].contains(&self.ablation.as_str())
            && self.steering.valid()
            && self.meals.len() <= 24
            && self.meals.iter().all(|m| {
                m.time.is_finite() && m.time >= 0. && m.amount.is_finite() && m.amount >= 0.
            })
            && (self.shade.is_empty()
                || self.shade.len() == self.domain.sites()
                    && self
                        .shade
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=1.).contains(v)))
            && self.domain.valid()
            && self.fluid.domain == self.domain
            && self.planet.as_ref().is_none_or(|p| p.domain == self.domain)
            && self.physiology.valid()
            && self.web.valid()
            && [self.air_nitrogen, self.air_carbon, self.buried_carbon, self.buried_nitrogen, self.rusted_iron, self.rock_nitrogen]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.)
            && self.development.valid(cells, &self.joints)
            && self.systems.valid(self.domain, cells)
            && self.planet.as_ref().is_none_or(|p| {
                (p.biological_water
                    - cells
                        .iter()
                        .map(|c| c.material.as_ref().map_or(0., |m| m.body.water))
                        .sum::<f64>())
                .abs()
                    < 1e-7 * (1. + p.initial_water)
            })
            && self
                .planet
                .as_ref()
                .is_none_or(crate::planet::Planet::valid)
            && self.lines.len() <= 256
            && self.lines.iter().all(|(name, lines)| {
                name.len() <= 80
                    && lines.len() <= crate::steering::LINES * 2
                    && lines.iter().all(|l| l.since.is_finite() && l.since >= 0.)
            })
            && self.joints.len() <= MAX_CELLS * 3
            && self.joints.iter().all(|j| {
                ids.contains(&j.a)
                    && ids.contains(&j.b)
                    && ids.contains(&j.c)
                    && j.a < j.c
                    && j.a != j.b
                    && j.b != j.c
                    && keys.insert((j.a, j.b, j.c))
                    && [j.rest, j.target, j.angle]
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=std::f32::consts::PI).contains(v))
                    && j.bend.is_finite()
                    && j.aim.is_finite()
                    && [j.flexion, j.extension]
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=1.).contains(v))
                    && j.work.is_finite()
                    && j.work >= 0.
            })
            && self.solutes.len() == self.domain.sites()
            && self
                .solutes
                .iter()
                .flatten()
                .all(|v| v.is_finite() && *v >= 0. && *v < 1e12)
            && self.fluid.valid()
            && cells
                .iter()
                .all(|c| c.material.as_ref().is_some_and(Inventory::valid))
            && [
                self.initial_organic,
                self.initial_mineral,
                self.initial_energy,
                self.photon_energy,
                self.heat,
                self.mechanical_input,
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.)
            && self
                .errors(cells)
                .iter()
                .all(|v| v.abs() < 1e-6 * (1. + self.initial_energy))
    }
    pub fn transport(&mut self, dt: f64) {
        let diffusivity = self.diffusivity(DIFFUSIVITY);
        if self.domain.spherical() {
            crate::surface::SurfaceFlow::diffuse(self.domain, &mut self.solutes, dt, diffusivity);
            self.decay_signals(dt);
            return;
        }
        // Conservative donor-cell advection + central Fickian diffusion.
        // Four substeps keep outgoing coefficients below one for the validated
        // low-Mach domain. No concentration clamp can erase/create material.
        let velocity: Vec<_> = (0..self.domain.sites())
            .map(|i| self.fluid.velocity(i))
            .collect();
        for _ in 0..4 {
            let old = &self.solutes;
            let mut delta = vec![[0.; SOLUTES]; self.domain.sites()];
            for z in 0..self.domain.rows() {
                for x in 0..self.domain.grid {
                    let i = z * self.domain.grid + x;
                    for (j, axis) in [
                        (
                            if x + 1 < self.domain.grid {
                                Some(i + 1)
                            } else {
                                None
                            },
                            0,
                        ),
                        (
                            if z + 1 < self.domain.grid {
                                Some(i + self.domain.grid)
                            } else {
                                None
                            },
                            1,
                        ),
                    ] {
                        if let Some(j) = j {
                            let u = (velocity[i][axis] + velocity[j][axis]) * 0.5;
                            for k in 0..SOLUTES {
                                let donor = if u >= 0. { old[i][k] } else { old[j][k] };
                                let flux = dt / 4.
                                    * (u / DX * donor
                                        + diffusivity[k] / (DX * DX) * (old[i][k] - old[j][k]));
                                delta[i][k] -= flux;
                                delta[j][k] += flux;
                            }
                        }
                    }
                }
            }
            for i in 0..self.domain.sites() {
                for k in 0..SOLUTES {
                    self.solutes[i][k] += delta[i][k];
                }
            }
        }
        self.decay_signals(dt);
    }
    fn decay_signals(&mut self, dt: f64) {
        for s in &mut self.solutes {
            for k in [3, 4] {
                let decayed = s[k] * (1. - (-0.015 * dt).exp());
                s[k] -= decayed;
                s[1] += decayed;
                self.heat += decayed * Q;
            }
        }
    }
    pub(crate) fn spend(&mut self, c: &mut Cell, amount: f64, mechanical: bool) -> f64 {
        let before = c.energy as f64;
        c.energy = (before - amount.min(before)).max(0.) as f32;
        let paid = before - c.energy as f64;

        if mechanical {
            self.mechanical_input += paid;
            c.state.motor_work += paid;
        } else {
            self.heat += paid;
        }
        paid
    }
    /// Left-minus-right differences sampled one field spacing to either side
    /// of the heading: dissolved fuel, light, the waste living cells give off,
    /// and organic remains. Each needs the matching receptor material.
    fn lateral_senses(&self, c: &Cell, core: &Core, at: (f32, f32)) -> [f32; LATERAL] {
        // A steering apparatus carries a crude chemical sense of its own;
        // dedicated receptors sharpen it.
        let receptor = c
            .activity(crate::physiology::RECEPTOR)
            .max(0.35 * c.activity(STEER));
        let photic = c.activity(crate::development::PHOTIC);
        if receptor <= 0.01 && photic <= 0.01 {
            return [0.; LATERAL];
        }
        let reach = self.domain.spacing().max(c.radius());
        let side = self
            .domain
            .tangent_at(at.0, at.1, c.heading + std::f32::consts::FRAC_PI_2)
            .mul(reach);
        let (l, r) = (
            self.domain.locate(c.p.add(side)),
            self.domain.locate(c.p.sub(side)),
        );
        let level = |i: usize, k: usize| self.solutes[i][k] * 2.56 / self.domain.area(i);
        let contrast = |a: f64, b: f64, half: f64| ((a - b) / (half + a + b)) as f32;
        let light = |i: usize| {
            self.planet.as_ref().map_or(1., |p| p.illumination(i))
                * self.light_fraction(i)
                * core.config.sunlight as f64
                * Self::fade(c.state.depth)
        };
        [
            contrast(level(l, 0), level(r, 0), 0.5) * receptor,
            contrast(light(l), light(r), 0.05) * photic,
            // What the living leak and the dead leave behind, dissolved in
            // the water, is the scent that carries.
            // It is a hint beside what is felt or seen close by, or a cell
            // would turn on every eddy.
            ((0.3 * contrast(level(l, 0) + level(l, 5), level(r, 0) + level(r, 5), 0.2)
                + c.state.nearby)
                * receptor)
                .clamp(-1., 1.),
            contrast(level(l, 5), level(r, 5), 0.5) * receptor,
        ]
    }
    /// The same four senses as `lateral_senses`, up against down: positive
    /// is up. Dissolved food is the same at every depth; light is always
    /// brighter above; other life is wherever it is; and remains lie on the bed.
    fn vertical_senses(&self, c: &Cell, core: &Core, site: usize) -> [f32; LATERAL] {
        let receptor = c
            .activity(crate::physiology::RECEPTOR)
            .max(0.35 * c.activity(STEER));
        let photic = c.activity(crate::development::PHOTIC);
        if (receptor <= 0.01 && photic <= 0.01) || c.state.under < 0.5 {
            return [0.; LATERAL];
        }
        let lit = self.planet.as_ref().map_or(1., |p| p.illumination(site))
            * self.light_fraction(site)
            * core.config.sunlight as f64;
        let (up, down) = (lit * Self::fade(c.state.depth - 0.5), lit * Self::fade(c.state.depth + 0.5));
        let remains = self.solutes[site][5] * 2.56 / self.domain.area(site);
        [
            0.,
            ((up - down) / (0.05 + up + down)) as f32 * photic,
            (c.state.above * receptor).clamp(-1., 1.),
            if c.state.height > 0.3 { -(remains / (0.5 + remains)) as f32 * receptor } else { 0. },
        ]
    }
    pub(crate) fn physiology(&mut self, e: &mut Evolution, core: &Core) {
        let mut degree = crate::body::Ids::default();
        for b in &e.bonds {
            *degree.entry(b.a).or_default() += 1;
            *degree.entry(b.b).or_default() += 1;
        }
        let unscreened = 1. - self.ozone();
        for c in &mut e.cells {
            let at = self.domain.coords(c.p);
            let site = self.domain.index(at.0, at.1);
            // Ultraviolet: what the star sends here, less what ozone and
            // the water over the cell take out. Skin, a wall, a shell of
            // glass and the cells around it screen a cell from the rest.
            c.state.ultraviolet = (self.planet.as_ref().map_or(0., |p| p.illumination(site))
                * self.light_fraction(site)
                * self.steering.light as f64
                * core.config.sunlight as f64
                * unscreened) as f32
                * (-c.state.depth.max(0.) / UV_REACH).exp();
            c.chemicals = [
                (self.solutes[site][3]
                    / (0.03 * self.domain.area(site) / 2.56 + self.solutes[site][3]))
                    as f32,
                (self.solutes[site][4]
                    / (0.03 * self.domain.area(site) / 2.56 + self.solutes[site][4]))
                    as f32,
            ];
            let body = &c.material.as_ref().unwrap().body;
            if body.water > 1e-9 {
                let fraction = (body.water / (0.02 + body.water)) as f32;
                for k in 0..2 {
                    let concentration = body.solutes[3 + k] / body.water;
                    let internal = (concentration / (0.03 + concentration)) as f32;
                    c.chemicals[k] = c.chemicals[k] * (1. - fraction) + internal * fraction;
                }
            }
            let inputs = c.physical_inputs_at(
                &self.solutes,
                degree.get(&c.id).copied().unwrap_or(0),
                self.domain,
                at,
            );
            let mut inputs = inputs;
            let lateral = self.lateral_senses(c, core, at);
            inputs[INPUTS - LATERAL..].copy_from_slice(&lateral);
            // A cell on its own is its own organism and can become anything.
            // Inside a body a stem cell commits when one of its commitment
            // genes is driven past half by where it finds itself; from then
            // on only the genes switched on for that fate are read.
            let bonded = degree.get(&c.id).copied().unwrap_or(0) > 0;
            if !bonded {
                c.state.fate = 0;
                c.state.order = 0;
                c.state.since_fork = 0;
                c.state.axial = 0;
                c.state.lateral = 0;
                c.state.passed = false;
                // A cell that has just come away from a body is the beginning
                // of whatever body it goes on to build.
                if c.state.tied {
                    c.lifecycle.depth = 0;
                    // And it is a new individual, not a piece of the old:
                    // the body it builds begins with itself.
                    c.lifecycle.root = c.id;
                }
            }
            c.state.tied = bonded;
            c.expression = c.cellular_expression(inputs);
            if bonded && c.state.fate == 0 {
                let (fate, drive) = BECOME.iter().enumerate().fold((0, 0.5), |best, (k, &a)| {
                    if c.expression[a] > best.1 {
                        (k as u8 + 1, c.expression[a])
                    } else {
                        best
                    }
                });
                if drive > 0.5 {
                    c.state.fate = fate;
                    c.expression = c.cellular_expression(inputs);
                }
            }
            // Size is read from the genes each tick; a genome without the
            // gene makes cells of the usual size.
            c.state.stature = if c
                .genome
                .genes
                .iter()
                .chain(c.lifecycle.homolog.iter().flat_map(|g| &g.genes))
                .any(|g| g.action == SIZE)
            {
                4f32.powf(2. * c.expression[SIZE].clamp(0., 1.) - 1.)
            } else {
                1.
            } * TAPER.powi(c.state.order.min(6) as i32);
            // Its outline likewise: a cell with the gene grows as a rod, up to
            // four times as long as it is wide.
            c.state.aspect = 1. + 3. * c.expression[ELONGATE].clamp(0., 1.);
            let (stature, area, pace) = (c.stature(), c.area(), c.pace());
            let fate = c.state.fate;
            c.state.turn = {
                let turn = c.genome.steer(lateral, fate);
                c.lifecycle
                    .homolog
                    .as_ref()
                    .map_or(turn, |other| (turn + other.steer(lateral, fate)) * 0.5)
            };
            // The same steering genes weigh up against down.
            let vertical = self.vertical_senses(c, core, site);
            c.state.climb = {
                let climb = c.genome.steer(vertical, fate);
                c.lifecycle
                    .homolog
                    .as_ref()
                    .map_or(climb, |other| (climb + other.steer(vertical, fate)) * 0.5)
            };
            self.reserves_and_repair(c);
            // Everything a cell does goes at the pace its warmth allows.
            // A warm-blooded cell makes up what its surroundings lack in
            // warmth and nothing else: dry ground slows it like any other.
            let (outside, warmth) =
                self.planet.as_ref().map_or((1., 1.), |p| (p.kinetics(site), p.warmth(site)));
            let heated = c.activity(WARM) as f64 * (1. - warmth).max(0.);
            let dt = DT as f64 * outside * (1. + heated / warmth);
            // Water takes light: a cell deep down gets a fraction of what
            // falls on the surface above it.
            let illumination = self.planet.as_ref().map_or(1., |p| p.illumination(site))
                * self.light_fraction(site)
                * self.steering.light as f64
                * Self::fade(c.state.depth);
            let s = &mut self.solutes[site];
            let m = c.material.as_mut().unwrap();
            let activity: [f64; ACTIONS] = std::array::from_fn(|k| {
                m.activity(k) as f64 * (1. - c.state.injury * 0.75) as f64
            });
            // Transport moves actual reduced carbon into the cell.
            // A membrane given over to swallowing prey has little room left for
            // soaking up dissolved fuel, harvesting light or vent chemistry: a
            // cell is mostly a feeder or mostly a hunter.
            let autotrophy = 1. - MOUTH_COST * activity[ENGULF];
            // Nothing dissolved reaches a cell on dry ground: it takes water to
            // carry food, nitrogen and mineral across a membrane.
            let watered = 1. - c.state.thirst as f64;
            let uptake = (dt
                * 0.35
                * activity[1]
                * area
                * watered
                * autotrophy
                * core.config.solvent as f64
                * c.state.exposure as f64)
                .min(s[0])
                .min((m.biomass * 1.5 - m.fuel).max(0.));
            s[0] -= uptake;
            m.fuel += uptake * 0.9;
            s[5] += uptake * 0.1; // unassimilated organic excretion, not oxidized waste
            self.physiology.excreted += uptake * 0.1;
            c.state.flux.fuel += uptake * 0.9;
            c.state.flux.organic_waste += uptake * 0.1;
            self.web.feed(&mut c.state, m.organic(), uptake * 0.9, crate::trophic::Source::Dead);
            // Dissolved nitrogen crosses a membrane only through the machinery
            // of a cell that feeds itself or rots what is dead, or of one that
            // keeps a light-feeder inside it. An animal gets its nitrogen in
            // its food. A cell keeps a working store of half what it is built of.
            let level = s[NITROGEN] * 2.56 / self.domain.area(site);
            let taken = (dt
                * 0.05
                * activity[0].max(activity[1]).max(activity[CHEMO]).max(activity[15]).max(c.state.symbiont as f64)
                * area
                * watered
                * autotrophy
                * c.state.exposure as f64
                * level
                / (0.01 + level))
                .min(s[NITROGEN])
                .min((m.bound_nitrogen() * 0.5 - m.nitrogen).max(0.))
                .min(c.energy as f64 / N_PRICE);
            s[NITROGEN] -= taken;
            m.nitrogen += taken;
            c.state.flux.nitrogen += taken;
            // Phosphorus comes in the same way, from mineral dissolved out of rock.
            let rock = s[2] * 2.56 / self.domain.area(site);
            let mined = (dt
                * 0.01
                * activity[0].max(activity[1]).max(activity[CHEMO]).max(activity[15]).max(c.state.symbiont as f64)
                * area
                * watered
                * autotrophy
                * c.state.exposure as f64
                * rock
                / (0.01 + rock))
                .min(s[2])
                .min((m.bound_phosphorus() * 0.5 - m.phosphorus).max(0.));
            s[2] -= mined;
            m.phosphorus += mined;
            let before = c.energy as f64;
            c.energy = (before - taken * N_PRICE) as f32;
            self.heat += before - c.energy as f64;
            let decomposed = (dt * 0.05 * activity[15] * area * watered * s[5] / (0.05 + s[5]))
                .min(s[5])
                .min((m.biomass * 1.5 - m.fuel).max(0.))
                .min(c.energy as f64 / 0.15);
            // A decomposer takes the nitrogen bound in what it breaks down,
            // and gives off what it has no use for.
            let unbound = if s[5] > 0. {
                (s[REMAINS_N] * decomposed / s[5]).min(s[REMAINS_N])
            } else {
                0.
            };
            s[REMAINS_N] -= unbound;
            m.nitrogen += unbound;
            s[5] -= decomposed;
            m.fuel += decomposed;
            self.web.feed(&mut c.state, m.organic(), decomposed, crate::trophic::Source::Dead);
            self.physiology.decomposed += decomposed;
            c.state.flux.detritus += decomposed;
            let before = c.energy as f64;
            c.energy = (before - decomposed * 0.15) as f32;
            self.heat += before - c.energy as f64;
            // W + light -> reduced C. This is a deliberately abstract redox
            // reaction. Its chemical energy is supplied by tracked photons.
            // A captive symbiont harvests light inside the host, whatever the
            // host's own membrane is given over to.
            let harvest = (activity[0] * autotrophy).max(c.state.symbiont as f64);
            let photo = (dt
                * 0.06
                * harvest
                * area
                * core.config.sunlight as f64
                * illumination
                * c.state.exposure as f64
                // Building with light takes water as well as carbon dioxide.
                * (1. - c.state.thirst as f64))
                .min(s[1])
                .min((m.biomass * 1.5 - m.fuel).max(0.));
            s[1] -= photo;
            m.fuel += photo;
            self.web.feed(&mut c.state, m.organic(), photo, crate::trophic::Source::Light);
            self.photon_energy += photo * Q;
            c.state.flux.photosynthesis += photo;
            // Splitting water for its hydrogen leaves oxygen behind: the only
            // source of free oxygen a planet has.
            if let Some(gas) = self.systems.gas.get_mut(site) {
                let freed = (photo * OXYGEN_PER_FUEL).min(gas[1]);
                gas[1] -= freed;
                gas[0] += freed;
            }
            // W + mineral chemistry -> reduced C. Works in the dark; the energy is
            // supplied by the planet's interior and tracked like sunlight.
            // Hydrogen from the vents and carbon dioxide from the water,
            // joined into living matter: how the first cells fed.
            let hydrogen = s[HYDROGEN] * 2.56 / self.domain.area(site);
            // At a vent a cell can feed as fast as one in bright light; it is
            // the vent's flow that sets how many can.
            let chemo = (dt
                * 0.3
                * activity[CHEMO]
                * area
                * (hydrogen / (H_HALF + hydrogen))
                // The rock's chemistry is in the rock: it feeds what lies on
                // the bed, and less with every step above it.
                * (-(c.state.height as f64) / VENT_REACH).exp()
                * watered
                * autotrophy
                * c.state.exposure as f64)
                .min(s[HYDROGEN])
                .min(s[1])
                .min((m.biomass * 1.5 - m.fuel).max(0.));
            s[HYDROGEN] -= chemo;
            s[1] -= chemo;
            m.fuel += chemo;
            self.web.feed(&mut c.state, m.organic(), chemo, crate::trophic::Source::Light);
            self.geothermal_energy += chemo * Q;
            c.state.flux.chemosynthesis += chemo;
            // How fast a cell can burn fuel. Oxygen gives any cell the high
            // rate an active life needs, which is why animals wait for it.
            // Breathing is done in a cell's outer membrane. A mouth crowds
            // it out, and a big cell has too little membrane for its bulk.
            // A breather kept alive inside does the work whatever the
            // membrane is given over to.
            let breathing = (activity[RESPIRE] * autotrophy * (area / stature).min(1.))
                .max(c.state.mitochondria as f64);
            c.state.breathing = breathing as f32;
            // Oxygen gets into a cell through the surface it has open to
            // the water and no other way. A cell packed inside a body gets
            // almost none this way, and must be brought it.
            let open = c.state.exposure as f64;
            let oxygen = self.systems.gas.get(site).map_or(1., |gas| {
                let level = gas[0] / (self.domain.area(site) / 2.56);
                level / (OXYGEN_HALF + level)
            }) * open;
            let air = breathing * oxygen;
            let wanted = (dt * (0.035 + 0.12 * activity[1].max(air)) * pace)
                .min(m.fuel)
                .min(((9. * stature - c.energy as f64) / Q).max(0.));
            // A cell that breathes captures all the energy in the fuel it
            // burns. One that cannot, for want of oxygen or of the machinery,
            // gets a fifteenth and loses the rest as heat.
            let aerobic = match self.systems.gas.get_mut(site) {
                Some(gas) => {
                    let level = gas[0] / (self.domain.area(site) / 2.56);
                    let aerobic = (wanted * breathing * open * level / (OXYGEN_HALF + level))
                        .min(gas[0] / OXYGEN_PER_FUEL);
                    // Never spend more than is there, down to the last rounding.
                    let used = (aerobic * OXYGEN_PER_FUEL).min(gas[0]);
                    gas[0] -= used;
                    gas[1] += used;
                    aerobic
                }
                // Worlds from before oxygen was tracked breathe freely.
                None => wanted * breathing,
            };
            // Oxygen brought to a cell by the body it is part of, through
            // gills and whatever carries it round, burns fuel as well as
            // oxygen drawn through the cell's own surface does. It is how
            // a cell deep inside a body breathes at all.
            let carried = ((wanted - aerobic) * breathing).min(m.body.gas[0] / OXYGEN_PER_FUEL).max(0.);
            let used = (carried * OXYGEN_PER_FUEL).min(m.body.gas[0]);
            m.body.gas[0] -= used;
            m.body.gas[1] += used;
            let aerobic = aerobic + carried;
            let burn = wanted;
            let captured = (aerobic + (wanted - aerobic) * FERMENT) * Q;
            c.state.breath = if wanted > 0. { (aerobic / wanted) as f32 } else { c.state.breath };
            m.fuel -= burn;
            s[1] += burn;
            c.state.flux.waste += burn;
            let before = c.energy as f64;
            c.energy = (before + captured) as f32;
            // What a cell fails to capture is lost as heat.
            self.heat += burn * Q - (c.energy as f64 - before);
            e.food_energy += captured;
            // Nitrogen from the air. Oxygen wrecks the enzyme, so it works
            // only where oxygen cannot reach it: in water with none, behind
            // a wall, or deep in a body. A cell splitting water for light is
            // making oxygen beside it, so a light-feeder fixes in the dark.
            if activity[FIX] > 0. {
                let shelter = activity[WALL].max(1. - c.state.exposure as f64);
                // A tenth of what full sun would have it make is enough.
                let lit = (photo / (dt * 0.006 * area)).min(1.);
                let fixed = (dt
                    * 0.05
                    * activity[FIX]
                    * area
                    * watered
                    * (1. - oxygen * (1. - shelter))
                    * (1. - lit))
                    .min(self.air_nitrogen)
                    .min((m.bound_nitrogen() * 0.5 - m.nitrogen).max(0.))
                    .min(c.energy as f64 / FIX_PRICE);
                self.air_nitrogen -= fixed;
                m.nitrogen += fixed;
                self.physiology.nitrogen_fixed += fixed;
                c.state.flux.nitrogen += fixed;
                let before = c.energy as f64;
                c.energy = (before - fixed * FIX_PRICE) as f32;
                self.heat += before - c.energy as f64;
            }
            // Gene output is a production rate. Mechanical effects use actual
            // protein/mineral inventories, never promoter output directly.
            // The share of a protein lost each tick: little while its gene is
            // working, a great deal once it falls silent.
            let (kept, dropped) = (1. - (-0.025 * dt).exp(), 1. - (-0.2 * dt).exp());
            for k in 0..ACTIONS {
                // Commitment genes regulate; they build nothing.
                let building = if BECOME.contains(&k) || k == BROADEN || k == SIZE || k == ELONGATE || k == BRANCH {
                    0.
                } else {
                    c.expression[k] as f64
                };
                // Nothing made and nothing held: every sum below would add nothing.
                if building == 0.
                    && m.proteins[k].to_bits() == 0
                    && m.fuel > 0.
                    && c.energy > 0.
                    && m.nitrogen > 0.
                {
                    continue;
                }
                // Protein and tissue are built with nitrogen; fuel and fat hold none.
                // A wall is cellulose: carbon, hydrogen and oxygen only.
                let needs = if k == WALL { 0. } else { N_RATIO };
                let amount = (dt * 0.012 * building * pace)
                    .min(m.fuel)
                    .min(c.energy as f64)
                    .min(if needs > 0. { m.nitrogen / needs } else { f64::INFINITY });
                m.fuel -= amount;
                m.nitrogen = (m.nitrogen - amount * needs).max(0.);
                m.proteins[k] += amount;
                c.state.flux.proteins += amount;
                let before = c.energy as f64;
                c.energy = (before - amount) as f32;
                self.heat += before - c.energy as f64;
                // Machinery whose gene is silent in this cell is dismantled fast,
                // so what a cell is follows what its genes are doing now.
                let turnover = m.proteins[k] * if c.expression[k] > 0. { kept } else { dropped };
                m.proteins[k] -= turnover;
                m.fuel += turnover;
                m.nitrogen += turnover * needs;
            }
            // An engulfing cell grows up to twice the size of an ordinary one,
            // and has to before it divides: bigger than its prey, slower to breed.
            let fed = (dt
                * 0.04
                * activity[7]
                * pace
                * (1. - m.biomass / (0.5 * stature * (1. + activity[ENGULF]))).max(0.))
            .min(m.fuel)
            .min(c.energy as f64 / 0.4);
            let built = fed.min(m.nitrogen / N_RATIO);
            // Bone lends its phosphorus to growth when the cell has none loose.
            // Rounding can leave bone a hair below nothing; it never lends that.
            let bone = m.mineral.max(0.);
            let growth = built.min((m.phosphorus + bone) / P_RATIO);
            let lent = (growth * P_RATIO - m.phosphorus).clamp(0., bone);
            m.mineral = (m.mineral - lent).max(0.);
            m.phosphorus = (m.phosphorus + lent - growth * P_RATIO).max(0.);
            // How far short of nitrogen, and then of phosphorus, this cell's
            // growth is running.
            let short = if fed > 1e-9 { (1. - built / fed) as f32 } else { 0. };
            c.state.nitrogen_short += (short - c.state.nitrogen_short) * 0.05;
            let short = if built > 1e-9 { (1. - growth / built) as f32 } else { 0. };
            c.state.phosphorus_short += (short - c.state.phosphorus_short) * 0.05;
            m.fuel -= growth;
            m.nitrogen = (m.nitrogen - growth * N_RATIO).max(0.);
            m.biomass += growth;
            let before = c.energy as f64;
            c.energy = (before - growth * 0.4) as f32;
            self.heat += before - c.energy as f64;
            // Reversible mineral association: k_on * catalyst * M - k_off * B.
            let bound = (dt * 0.025 * activity[3] * s[2])
                .min(s[2])
                .min(c.energy as f64 / 0.2)
                .min((SKELETON_SHARE * m.biomass - m.mineral).max(0.));
            s[2] -= bound;
            m.mineral += bound;
            c.state.flux.minerals += bound;
            let before = c.energy as f64;
            c.energy = (before - bound * 0.2) as f32;
            self.heat += before - c.energy as f64;
            let released = (m.mineral * (1. - (-0.006 * dt).exp())).max(0.);
            m.mineral = (m.mineral - released).max(0.);
            s[2] += released;
            // A shell of glass is laid down from silica dissolved in the
            // water, as far as the cell's shell machinery calls for, and it
            // stays until the cell dies.
            if activity[SHELL] > 0. {
                let glass = s[SILICA] * 2.56 / self.domain.area(site);
                let laid = (dt * 0.08 * activity[SHELL] * area * watered * glass / (0.05 + glass))
                    .min((m.biomass * SHELL_RATIO * activity[SHELL] - m.silica).max(0.))
                    .min(s[SILICA])
                    .min(c.energy as f64 / SHELL_PRICE);
                s[SILICA] -= laid;
                m.silica += laid;
                let before = c.energy as f64;
                c.energy = (before - laid * SHELL_PRICE) as f32;
                self.heat += before - c.energy as f64;
            }
            for k in 0..2 {
                let secreted = (dt * 0.015 * activity[5 + k])
                    .min(m.fuel)
                    .min(c.energy as f64 / 0.3);
                m.fuel -= secreted;
                if k == 0 {
                    c.state.flux.signal_a += secreted;
                } else {
                    c.state.flux.signal_b += secreted;
                }
                if m.body.water > 1e-9 {
                    m.body.solutes[3 + k] += secreted;
                } else {
                    s[3 + k] += secreted;
                }
                let before = c.energy as f64;
                c.energy = (before - secreted * 0.3) as f32;
                self.heat += before - c.energy as f64;
            }
            // What a cell cannot use it gives off, and the water carries it
            // back to whatever feeds on light.
            let spare = (m.nitrogen - m.bound_nitrogen() * 0.5).max(0.) * dropped;
            m.nitrogen -= spare;
            s[NITROGEN] += spare;
            c.state.flux.nitrogen_given += spare;
            let spare = (m.phosphorus - m.bound_phosphorus() * 0.5).max(0.) * dropped;
            m.phosphorus -= spare;
            s[2] += spare;
            let upkeep = dt
                * ((0.055
                    + (c.genome.genes.len()
                        + c.lifecycle.homolog.as_ref().map_or(0, |g| g.genes.len()))
                        as f64
                        * GENE_UPKEEP)
                    * pace
                    + (m.organic() - m.lipid * (1. - FAT_UPKEEP)) * 0.015
                    + c.state.symbiont as f64 * 0.01
                    // Breathers kept inside have to be kept alive.
                    + c.state.mitochondria as f64 * MITOCHONDRIA_UPKEEP
                    // Heat made to stay warm leaks away through the cell's
                    // open surface; fat under it keeps more of it in.
                    + heated
                        * WARM_PRICE
                        * c.state.exposure as f64
                        * (1. - 0.8 * (m.lipid / m.mass()).min(1.)))
                * (1. - DORMANCY * c.state.thirst as f64)
                // A cell wrapped in its body's other cells loses less to the
                // water than one bathed in it, so a large body costs less to
                // run for its size than a small one.
                * (INTERIOR + (1. - INTERIOR) * c.state.exposure as f64);
            self.spend(c, upkeep, false);
            if let Some(p) = &self.planet {
                let burn = DT * (p.heat[site] - 0.55).max(0.) * 0.05;
                c.state.injury = (c.state.injury + burn).min(1.);
                c.state.harm[1] += burn;
            }
            let screen = c
                .activity(crate::physiology::BARRIER)
                .max(0.7 * c.activity(WALL))
                .max(c.material.as_ref().unwrap().shell() as f32);
            let rays = DT * UV_HARM * (c.state.ultraviolet - UV_TOLERATED).max(0.) * c.state.exposure * (1. - screen);
            c.state.injury = (c.state.injury + rays).min(1.);
            c.state.harm[2] += rays;
            c.age += DT;
            c.cooldown = (c.cooldown - DT).max(0.);
        }
    }
    pub(crate) fn contacts(&mut self, e: &mut Evolution, core: &mut Core) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        for c in &mut e.cells {
            c.state.handling = (c.state.handling - DT).max(0.);
        }
        let sharing = self.steering.sharing as f64;
        // Keep exact inventory-dependent radii, refreshing only after digestion
        // moves matter. Broad-phase candidates need not repeatedly compute cbrt.
        let mut radii: Vec<_> = e.cells.iter().map(Cell::radius).collect();
        // Attacks act between bodies: a body does not eat or sting itself.
        // Bulk is living structure (summed biomass), not the fuel a cell happens
        // to be carrying: a lean hunter is still bigger than a fat prey cell.
        let armed = e
            .cells
            .iter()
            .any(|c| c.activity(9) > 0. || c.activity(ENGULF) > 0. || c.activity(TOXIN) > 0.);
        // Kin are cells whose sets of working genes are at least four-fifths
        // the same. Weapons spare armed kin; they do not spare a stranger just
        // because it carries the same weapon.
        let genes: Vec<u64> = e
            .cells
            .iter()
            .map(|c| {
                std::iter::once(&c.genome)
                    .chain(c.lifecycle.homolog.as_ref())
                    .flat_map(|g| &g.genes)
                    .filter(|g| g.strength > 0.05)
                    .fold(0u64, |set, g| set | 1 << g.action)
            })
            .collect();
        let kin = |a: usize, b: usize| {
            (genes[a] & genes[b]).count_ones() * 5 >= (genes[a] | genes[b]).count_ones() * 4
        };
        let mut body = Vec::new();
        let mut bulk = Vec::new();
        // Whether any cell of each body is a mouth.
        let mut mouthed = Vec::new();
        // Whether each body is an animal to whatever eats it: one that
        // lives by eating other life, or a many-celled body that does not
        // feed itself from light or the chemistry of rock. It is judged by
        // the body, since most of an animal's cells never eat anything:
        // its fins and its tail are fed by its mouths.
        let mut animal = Vec::new();
        // How strongly each cell's mouth draws water: none for a walled cell,
        // a light-feeder, or a mouth buried inside its body.
        let currents: Vec<f32> = if armed {
            e.cells
                .iter()
                .map(|c| {
                    let mouth = c.activity(ENGULF);
                    if mouth > 0. {
                        mouth * (1. - c.activity(WALL)).powi(2) * c.bite() * c.state.exposure
                    } else {
                        0.
                    }
                })
                .collect()
        } else {
            Vec::new()
        };
        let groups = if armed { e.groups() } else { Vec::new() };
        if armed {
            body = vec![0usize; e.cells.len()];
            for (g, members) in groups.iter().enumerate() {
                bulk.push(
                    members
                        .iter()
                        .map(|&i| e.cells[i].material.as_ref().unwrap().biomass)
                        .sum::<f64>(),
                );
                mouthed.push(members.iter().any(|&i| e.cells[i].activity(ENGULF) > 0.05));
                let mut flux = crate::observations::Flux::default();
                for &i in members {
                    flux.add(&e.cells[i].state.flux);
                }
                let own = flux.photosynthesis + flux.chemosynthesis;
                animal.push(
                    flux.eater() || (members.len() >= 4 && flux.intake() > 0.01 && own < 0.5 * flux.intake()),
                );
                for &i in members {
                    body[i] = g;
                }
            }
        }
        let mut degree = vec![0usize; e.cells.len()];
        let mut touching = vec![0u8; e.cells.len()];
        let ids = e.index();
        let mut bonded = crate::body::Pairs::default();
        for b in &e.bonds {
            degree[ids[&b.a]] += 1;
            degree[ids[&b.b]] += 1;
            bonded.insert((b.a, b.b));
        }
        // How big a mouth each cell is part of. A mouth is as many mouth
        // cells as hold on to one another, and its bulk is theirs together:
        // that, and not the size of the body behind it, is what decides how
        // big a thing it can take. One mouth cell on a great body swallows
        // no more than it would alone.
        let mut gape = Vec::new();
        // The biggest mouth each body has.
        let mut widest = vec![0f64; groups.len()];
        // The smallest thing each cell's mouth can keep hold of, and the
        // smallest any mouth of each body can.
        let mut mesh = Vec::new();
        let mut finest = vec![f64::INFINITY; groups.len()];
        if armed {
            let jaws: Vec<bool> = e.cells.iter().map(|c| c.activity(ENGULF) > 0.05).collect();
            let mut root: Vec<usize> = (0..e.cells.len()).collect();
            fn find(r: &mut [usize], mut x: usize) -> usize {
                while r[x] != x {
                    r[x] = r[r[x]];
                    x = r[x];
                }
                x
            }
            for b in &e.bonds {
                let (i, j) = (ids[&b.a], ids[&b.b]);
                if b.integrity > 0. && jaws[i] && jaws[j] {
                    let (i, j) = (find(&mut root, i), find(&mut root, j));
                    root[i.max(j)] = i.min(j);
                }
            }
            let mut patch = vec![0f64; e.cells.len()];
            // A mouth of many cells is opened by muscle and by nothing else.
            // A sheet of mouth cells with none among them, as a sponge is,
            // is so many small mouths side by side, each taking what one
            // cell can.
            let mut opened = vec![false; e.cells.len()];
            for i in 0..e.cells.len() {
                let r = find(&mut root, i);
                patch[r] += e.cells[i].material.as_ref().unwrap().biomass;
                opened[r] |= jaws[i] && e.cells[i].activity(4) > 0.05;
            }
            gape = (0..e.cells.len())
                .map(|i| {
                    let r = find(&mut root, i);
                    if opened[r] { patch[r] } else { e.cells[i].material.as_ref().unwrap().biomass }
                })
                .collect();
            // A mouth that opens wide takes in water with its food, and the
            // water has to go out again. What is small enough goes out with
            // it. One cell that wraps itself round its food lets nothing
            // out, so the small things are for small mouths: a body that
            // lives on them has many, as a sponge has, or as a shrimp has
            // along its limbs.
            mesh = (0..e.cells.len())
                .map(|i| {
                    if gape[i] > e.cells[i].material.as_ref().unwrap().biomass * 1.001 {
                        LEAK * gape[i]
                    } else {
                        0.
                    }
                })
                .collect();
            for (i, &current) in currents.iter().enumerate() {
                if current > 0. {
                    widest[body[i]] = widest[body[i]].max(gape[i]);
                    finest[body[i]] = finest[body[i]].min(mesh[i]);
                }
            }
        }
        // Broad phase by spatial bins. Only geometric contact enables binding,
        // digestion and exchange; no prey direction or supplied kin rule.
        let width = (radii.iter().copied().fold(0., f32::max) * 2. + 0.12).max(1.6);
        let bin = |p: V3| {
            (
                (p.x / width).floor() as i32,
                (p.y / width).floor() as i32,
                (p.z / width).floor() as i32,
            )
        };
        // Nothing here moves a cell, so positions are read from one compact list.
        let places: Vec<V3> = e.cells.iter().map(|c| c.p).collect();
        let mut bins = crate::body::Bins::default();
        for (i, &p) in places.iter().enumerate() {
            bins.entry(bin(p)).or_default().push(i);
        }
        // The feeding current. Every mouth open to the water pumps it, and
        // what a body's mouths pump together has to come from somewhere, so
        // water moves in from all around, as fast as the pumping divided by
        // the area it crosses. A body of many mouths therefore draws from
        // farther than one mouth does and clears water in proportion to
        // their number, as a sponge does. A free cell much smaller than the
        // whole body is carried in with the water. Cells held in a body are
        // not, which is the first reason to live in one.
        for (g, members) in groups.iter().enumerate() {
            let pump: f32 = members.iter().map(|&i| currents[i]).sum();
            if pump < 0.05 {
                continue;
            }
            // A body that beats stirs the water round it. Still water by a
            // mouth is soon emptied of what was in it; stirred, it is
            // replaced, so the same mouths draw up to twice as much. This
            // is what muscle is good for in a body too small or too plain
            // to swim by it: a pulse that feeds.
            let beat = members.iter().map(|&i| e.cells[i].state.beat).sum::<f32>() / members.len() as f32;
            let pump = pump * (1. + STIR * beat / (beat + STIR_HALF));
            // Where the pumping is centred, and how that point is moving.
            let (mut centre, mut drift) = (V3::default(), V3::default());
            for &i in members {
                let share = currents[i] / pump;
                centre = centre.add(places[i].mul(share));
                drift = drift.add(e.cells[i].v.mul(share));
            }
            let reach = (FEED_MOUTH * (FEED_CURRENT * pump / FEED_FLOOR).sqrt()).min(FEED_LIMIT);
            let span = (reach / width).ceil() as i32;
            let (x, y, z) = bin(centre);
            for yy in y - span..=y + span {
                for zz in z - span..=z + span {
                    for xx in x - span..=x + span {
                        let Some(found) = bins.get(&(xx, yy, zz)) else { continue };
                        for &b in found {
                            if degree[b] > 0 || body[b] == g {
                                continue;
                            }
                            let apart = centre.sub(places[b]);
                            let d = apart.len();
                            if d >= reach
                                || d < 1e-4
                                || kin(members[0], b) && e.cells[b].activity(ENGULF) > 0.25
                            {
                                continue;
                            }
                            let prey = e.cells[b].material.as_ref().unwrap();
                            // Nothing is drawn in that every mouth of the
                            // body would let slip out again.
                            if prey.biomass < finest[g] {
                                continue;
                            }
                            let outsize = widest[g] / prey.biomass;
                            // A mouth lets go of what it cannot get down: spines and
                            // walls weaken the current as much as they slow swallowing.
                            let advantage = ((outsize - FILTER_RATIO) / 0.7).clamp(0., 1.) as f32
                                / (1.
                                    + e.cells[b].activity(SPINE) * 6.
                                    + e.cells[b].activity(WALL) * 10.
                                    + prey.shell() as f32 * 6.);
                            if advantage <= 0.02 {
                                continue;
                            }
                            // Never faster than water enters a single mouth.
                            let speed = FEED_CURRENT
                                * advantage
                                * (pump * (FEED_MOUTH / d).powi(2)).min(1.);
                            let toward = apart.mul(1. / d);
                            let kick = (speed - e.cells[b].v.sub(drift).dot(toward)) * 0.5;
                            if kick <= 0. {
                                continue;
                            }
                            // The mouths pay for the motion they give their prey,
                            // priced as a swimming motor's work is.
                            let work = 0.5 * prey.mass() * (kick * kick) as f64 * WORK_PRICE;
                            let Some(&a) = members
                                .iter()
                                .find(|&&a| currents[a] >= 0.05 && e.cells[a].energy as f64 >= 0.5 + work)
                            else {
                                continue;
                            };
                            self.spend(&mut e.cells[a], work, true);
                            e.cells[b].v = e.cells[b].v.add(toward.mul(kick));
                            e.cells[a].state.flux.drawn += 1.;
                        }
                    }
                }
            }
        }
        // Cells in one bin share the same neighbourhood, so it is gathered
        // once for the bin and not again for each of them.
        let mut around = crate::body::Bins::default();
        let names: Vec<u32> = e.cells.iter().map(|c| c.id).collect();
        for i in 0..e.cells.len() {
            let (x, y, z) = bin(places[i]);
            // Flatten the 27 adjacent Cartesian bins into a deterministic list.
            let neighbors = around.entry((x, y, z)).or_insert_with(|| {
                let mut near = Vec::new();
                for yy in y - 1..=y + 1 {
                    for zz in z - 1..=z + 1 {
                        for xx in x - 1..=x + 1 {
                            if let Some(ids) = bins.get(&(xx, yy, zz)) {
                                near.extend(ids.iter().copied());
                            }
                        }
                    }
                }
                near
            });
            e.cells[i].state.nearby = 0.;
            e.cells[i].state.above = 0.;
            // A cell that can steer feels which side other life is on, within
            // one field spacing: close-range scent. An eater is following its
            // food, so its own kind does not register; anything else is drawn
            // to all life alike, its own kind included.
            if e.cells[i].activity(STEER) > 0.01 {
                let me = &e.cells[i];
                // By touch and close scent a cell tells where others are
                // within a unit and a half. An eye, in the light, sees
                // them three times as far.
                let sight = 1.6
                    * (1. + 2. * me.activity(crate::development::PHOTIC) * me.state.light.min(1.));
                let seen: Vec<usize>;
                let neighbors: &[usize] = if sight > width {
                    let span = (sight / width).ceil() as i32;
                    let mut far = Vec::new();
                    for yy in y - span..=y + span {
                        for zz in z - span..=z + span {
                            for xx in x - span..=x + span {
                                if let Some(ids) = bins.get(&(xx, yy, zz)) {
                                    far.extend(ids.iter().copied());
                                }
                            }
                        }
                    }
                    seen = far;
                    &seen
                } else {
                    neighbors
                };
                let left = self
                    .domain
                    .tangent(me.p, me.heading + std::f32::consts::FRAC_PI_2);
                let eater = me.activity(ENGULF).max(me.activity(9)) * me.bite() > 0.25;
                let overhead = self.domain.up(me.p);
                let mut rise = 0.;
                let mut pull = 0.;
                for &j in neighbors.iter() {
                    let delta = places[j].sub(me.p);
                    let d = delta.len();
                    if d < sight && d > 1e-4 {
                        let key = (me.id.min(names[j]), me.id.max(names[j]));
                        if j == i || bonded.contains(&key) {
                            continue;
                        }
                        if !(eater && kin(i, j)) {
                            pull += delta.dot(left) / d * (1. - d / sight);
                            rise += delta.dot(overhead) / d * (1. - d / sight);
                        }
                    }
                }
                e.cells[i].state.nearby = (pull / (1. + pull.abs())).clamp(-1., 1.);
                e.cells[i].state.above = (rise / (1. + rise.abs())).clamp(-1., 1.);
            }
            for &j in neighbors.iter() {
                if j <= i {
                    continue;
                }
                // Most of what is near is not near enough to matter, which a
                // comparison of squares shows without a root. The margin
                // leaves every close call to the exact test below.
                let apart = places[j].sub(places[i]);
                let limit = radii[i] + radii[j] + 0.12;
                if apart.dot(apart) > limit * limit * 1.01 {
                    continue;
                }
                let d = apart.len();
                if d > radii[i] + radii[j] + 0.12 {
                    continue;
                }
                pairs.push((i, j));
                touching[i] = touching[i].saturating_add(1);
                touching[j] = touching[j].saturating_add(1);
                let key = (
                    e.cells[i].id.min(e.cells[j].id),
                    e.cells[i].id.max(e.cells[j].id),
                );
                let material_affinity = |c: &Cell| {
                    c.material
                        .as_ref()
                        .unwrap()
                        .body
                        .properties(c.material.as_ref().unwrap().biomass)[0]
                };
                // Gene sharing: now and then one of two touching cells takes up
                // a copy of a gene the other carries and it lacks. Off unless
                // the player turns it on.
                if sharing > 0. && (core.random() as f64) < 0.002 * sharing {
                    let (from, to) = if core.random() < 0.5 { (i, j) } else { (j, i) };
                    // Genes that lie together travel together: up to three in
                    // a row, as a stretch of a chromosome does, so a machine
                    // that takes several parts can arrive whole.
                    let donor = &e.cells[from].genome.genes;
                    let start = ((core.random() * donor.len() as f32) as usize).min(donor.len() - 1);
                    let run: Vec<_> = donor[start..donor.len().min(start + 3)].to_vec();
                    let taker = &mut e.cells[to].genome;
                    let mut taken = false;
                    for gene in run {
                        if taker.genes.len() < crate::autonomous::MAX_GENES
                            && !taker.genes.iter().any(|g| g.action == gene.action)
                        {
                            taker.genes.push(gene);
                            taken = true;
                        }
                    }
                    if taken {
                        taker.id = e.next_genome;
                        e.next_genome += 1;
                        e.exchanges += 1;
                    }
                }
                // Glue sticks two cells together, and so does the matrix
                // they lay down between them.
                // Matrix a cell is no longer tending does not stick it to
                // anything new.
                let tended = |c: &Cell| {
                    if c.expression[crate::organics::MATRIX].max(c.expression[crate::organics::FIBER]) > 0.02 {
                        c.scaffold()
                    } else {
                        0.
                    }
                };
                let adhesion = ((e.cells[i].activity(2)
                    + material_affinity(&e.cells[i]) * 0.2
                    + tended(&e.cells[i]))
                .min(1.)
                    * (e.cells[j].activity(2)
                        + material_affinity(&e.cells[j]) * 0.2
                        + tended(&e.cells[j]))
                    .min(1.)
                    * e.cells[i].recognition(&e.cells[j])
                    // Only kin stick together on contact, so a body is a family
                    // and not a crowd of strangers. It also means a mouth
                    // never glues itself to a stranger it could swallow. And
                    // only cells of one individual: a body's young, once they
                    // have left it, are bodies of their own and do not grow
                    // back onto their parent.
                    * if kin(i, j) && e.cells[i].body_origin() == e.cells[j].body_origin() {
                        1.
                    } else {
                        0.
                    }) as f64;
                // A body does not glue to itself: its cells tie on touching
                // only where they were born next to each other, not wherever
                // a bend or a fold later brings two of them together.
                // (Cells that have taken up a place in a body's plan, that is:
                // a clump with no plan sticks as it always did.)
                // Next to each other in the plan, that is: one place apart
                // along the middle line, or one place apart out along the
                // same limb. Not a limb to the limb beside it, nor a limb to
                // the segment behind its own, or the body webs itself shut.
                let apart = if e.cells[i].state.fate != 0 || e.cells[j].state.fate != 0 {
                    let (x, y) = (&e.cells[i].state, &e.cells[j].state);
                    let along = (x.axial as i32 - y.axial as i32).abs();
                    let out = (x.lateral as i32 - y.lateral as i32).abs();
                    // And of one body now, not only of one egg: twins that
                    // parted at the first division, or a piece that has come
                    // away, do not grow back on where they happen to touch.
                    // (Which body each cell is in is only worked out in a
                    // world with mouths in it, which any with planned
                    // animals is.)
                    if body.get(i) == body.get(j)
                        && ((out == 0 && x.lateral == 0 && along == 1) || (along == 0 && out == 1))
                    {
                        1
                    } else {
                        2
                    }
                } else {
                    0
                };
                if !bonded.contains(&key)
                    && apart <= 1
                    && e.cells[i].lifecycle.phase != crate::development::Phase::Gamete
                    && e.cells[j].lifecycle.phase != crate::development::Phase::Gamete
                    && degree[i] < 6
                    && degree[j] < 6
                    && (core.random() as f64) < 1. - (-0.8 * adhesion * DT as f64).exp()
                {
                    e.bonds.push(Bond {
                        a: key.0,
                        b: key.1,
                        rest: d.max(0.1),
                        integrity: 1.,
                        activation: 0.,
                        stiffness: 0.,
                    });
                    degree[i] += 1;
                    degree[j] += 1;
                    bonded.insert(key);
                    self.bound_events += 1;
                }
                // Two walled cells pressed together share a wall, and what
                // is dissolved in one seeps into the other: fuel toward the
                // hungrier, nitrogen and phosphorus toward the poorer. A
                // plant with light to spare against a rotting thread with
                // mineral to spare is a trade; a thread on a plant that
                // gets nothing back is a theft. Cells tied into one body
                // are plumbed together already.
                if !bonded.contains(&key) {
                    let shared = (e.cells[i].activity(WALL) * e.cells[j].activity(WALL)) as f64;
                    if shared > 0.05 {
                        let (left, right) = e.cells.split_at_mut(i.max(j));
                        let (x, y) = if i < j { (&mut left[i], &mut right[0]) } else { (&mut right[0], &mut left[j]) };
                        let (xm, ym) = (x.material.as_mut().unwrap(), y.material.as_mut().unwrap());
                        let rate = DT as f64 * SEEP * shared * xm.biomass.min(ym.biomass);
                        let room = |m: &Inventory| (m.biomass * 1.5 - m.fuel).max(0.);
                        let fuel = ((xm.fuel / xm.biomass - ym.fuel / ym.biomass) * rate)
                            .clamp(-ym.fuel.max(0.).min(room(xm)), xm.fuel.max(0.).min(room(ym)));
                        xm.fuel -= fuel;
                        ym.fuel += fuel;
                        let nitrogen = ((xm.nitrogen / xm.biomass - ym.nitrogen / ym.biomass) * rate)
                            .clamp(-ym.nitrogen.max(0.), xm.nitrogen.max(0.));
                        xm.nitrogen -= nitrogen;
                        ym.nitrogen += nitrogen;
                        let phosphorus = ((xm.phosphorus / xm.biomass - ym.phosphorus / ym.biomass) * rate)
                            .clamp(-ym.phosphorus.max(0.), xm.phosphorus.max(0.));
                        xm.phosphorus -= phosphorus;
                        ym.phosphorus += phosphorus;
                    }
                }
                for (a, b) in [(i, j), (j, i)] {
                    let gate = e.cells[a].activity(crate::organics::MATRIX_DIGEST) as f64;
                    if gate > 0. && e.cells[a].energy > 0.1 {
                        let (left, right) = e.cells.split_at_mut(a.max(b));
                        let (attacker, prey) = if a < b {
                            (&mut left[a], &mut right[0])
                        } else {
                            (&mut right[0], &mut left[b])
                        };
                        let protection = 1. + prey.activity(crate::physiology::BARRIER) as f64 * 5.;
                        let who = [crate::observations::kind(attacker), crate::observations::kind(prey)];
                        let (am, pm) = (
                            attacker.material.as_mut().unwrap(),
                            prey.material.as_mut().unwrap(),
                        );
                        let total = pm.body.matrix + pm.body.fibers;
                        let extracted = (DT as f64 * 0.008 * gate / protection)
                            .min(total)
                            .min((am.biomass * 1.5 - am.fuel).max(0.) / 0.85)
                            .min(attacker.energy as f64 / 0.25);
                        let potential = (DT as f64 * 0.008 * gate)
                            .min(total)
                            .min((am.biomass * 1.5 - am.fuel).max(0.) / 0.85)
                            .min(attacker.energy as f64 / 0.25);
                        prey.state.flux.defense_blocked += (potential - extracted).max(0.);
                        if body.get(b).map_or(prey.state.flux.eater(), |&g| animal[g]) {
                            attacker.state.flux.prey_animal += extracted * 0.85;
                        }
                        attacker.state.flux.prey += extracted * 0.85;
                        prey.state.flux.lost_to_predators += extracted;
                        attacker.state.flux.organic_waste += extracted * 0.15;
                        self.record_meal(
                            core.time,
                            attacker.id,
                            prey.id,
                            attacker.genome.id,
                            prey.genome.id,
                            "matrix / fibers",
                            extracted,
                            who,
                        );
                        let fraction = extracted / total.max(1e-12);
                        pm.body.matrix *= 1. - fraction;
                        pm.body.fibers *= 1. - fraction;
                        am.fuel += extracted * 0.85;
                        self.web.feed(
                            &mut attacker.state,
                            am.organic(),
                            extracted * 0.85,
                            crate::trophic::Source::Prey(prey.state.trophic),
                        );
                        self.solutes[self.domain.locate(attacker.p)][5] += extracted * 0.15;
                        self.physiology.excreted += extracted * 0.15;
                        self.systems.matrix_meals += extracted;
                        self.spend(attacker, extracted * 0.25, false);
                        if extracted > 0. {
                            radii[a] = attacker.radius();
                            radii[b] = prey.radius();
                            e.meals += 1;
                            attacker.state.meals += 1;
                        }
                    }
                    let enzyme = (e.cells[a].activity(9) * e.cells[a].bite()) as f64;
                    if enzyme > 0. && e.cells[a].energy > 0.1 && body[a] != body[b] {
                        let barrier = e.cells[b].activity(crate::physiology::BARRIER) as f64;
                        // A cell is protected from the enzyme it makes itself, so
                        // digesters can live beside their own kind and eat others.
                        let immune = if kin(a, b) {
                            (e.cells[b].activity(9) as f64 * 4.).min(1.)
                        } else {
                            0.
                        };
                        let capacity = (e.cells[a].material.as_ref().unwrap().biomass * 1.5
                            - e.cells[a].material.as_ref().unwrap().fuel)
                            .max(0.);
                        let available = e.cells[a].energy as f64 / 0.2;
                        let reach = e.cells[a].area();
                        // Enzymes reach a cell through what lies around it.
                        // The matrix of a tissue has to be got through first.
                        let tissue = e.cells[b].scaffold() as f64;
                        let prey = e.cells[b].material.as_mut().unwrap();
                        let extracted = (DT as f64 * GRAZE * enzyme * reach * (1. - immune)
                            / (1. + prey.mineral / prey.biomass * 4. + barrier * 5. + prey.shell() * 4. + tissue * MESH))
                            .min((prey.biomass - 0.005).max(0.))
                            .min(capacity / 0.85)
                            .min(available);
                        let potential = (DT as f64 * 0.015 * enzyme)
                            .min((prey.biomass - 0.005).max(0.))
                            .min(capacity / 0.85)
                            .min(available);
                        prey.biomass -= extracted;
                        let damage = (extracted / prey.biomass.max(0.02)) as f32;
                        e.cells[b].state.flux.defense_blocked += (potential - extracted).max(0.);
                        e.cells[b].state.flux.lost_to_predators += extracted;
                        if body.get(b).map_or(e.cells[b].state.flux.eater(), |&g| animal[g]) {
                            e.cells[a].state.flux.prey_animal += extracted * 0.85;
                        }
                        e.cells[a].state.flux.prey += extracted * 0.85;
                        e.cells[a].state.flux.organic_waste += extracted * 0.15;
                        self.record_meal(
                            core.time,
                            e.cells[a].id,
                            e.cells[b].id,
                            e.cells[a].genome.id,
                            e.cells[b].genome.id,
                            "living biomass",
                            extracted,
                            [crate::observations::kind(&e.cells[a]), crate::observations::kind(&e.cells[b])],
                        );
                        e.cells[b].state.injury = (e.cells[b].state.injury + damage).min(1.);
                        e.cells[b].state.harm[3] += damage;
                        self.took(&e.cells[a], &e.cells[b], extracted);
                        // The nitrogen in what is digested goes with it: most to
                        // the eater, the rest left in its droppings.
                        let prey_steps = e.cells[b].state.trophic;
                        let am = e.cells[a].material.as_mut().unwrap();
                        am.fuel += extracted * 0.85;
                        am.nitrogen += extracted * 0.85 * N_RATIO;
                        am.phosphorus += extracted * 0.85 * P_RATIO;
                        let held = am.organic();
                        self.web.feed(
                            &mut e.cells[a].state,
                            held,
                            extracted * 0.85,
                            crate::trophic::Source::Prey(prey_steps),
                        );
                        let site = self.domain.locate(e.cells[a].p);
                        self.solutes[site][5] += extracted * 0.15;
                        self.solutes[site][REMAINS_N] += extracted * 0.15 * N_RATIO;
                        self.solutes[site][2] += extracted * 0.15 * P_RATIO;
                        self.physiology.digested += extracted;
                        self.physiology.excreted += extracted * 0.15;
                        self.spend(&mut e.cells[a], extracted * 0.2, false);
                        if extracted > 0. {
                            radii[a] = e.cells[a].radius();
                            radii[b] = e.cells[b].radius();
                            e.meals += 1;
                            e.cells[a].state.meals += 1;
                        }
                    }
                    // Engulfing is fast and takes one cell at a time. A cell held
                    // by its neighbours is hard for a mouth its own size to
                    // pull off and easy for a much bigger one, so living as a
                    // body is a refuge from small mouths and a body's interior
                    // is out of reach altogether. Skin does not help once
                    // swallowed; spines and mineral armour do, and spines wound
                    // whatever tries.
                    let mouth = (e.cells[a].activity(ENGULF)
                        * (1. - e.cells[a].activity(WALL)).powi(2)
                        * e.cells[a].bite()) as f64;
                    // One thing at a time. A mouth that has hold of something,
                    // or is still getting the last thing down, takes nothing
                    // else: it cannot look for food while it is dealing with
                    // food. So what a mouth can eat in a day is set by how
                    // many things it can handle, and a large meal is worth
                    // more of its time than a small one.
                    let free = e.cells[a].state.handling <= 0. || e.cells[a].state.prey == e.cells[b].id;
                    if mouth > 0.
                        && free
                        && e.cells[a].energy > 0.1
                        && body[a] != body[b]
                        && gape[a] > 0.8 * e.cells[b].material.as_ref().unwrap().biomass
                        // What is swallowed, a loose cell or a body small
                        // enough to go in whole, has to be big enough for
                        // this mouth to keep. A piece torn off a larger body
                        // is held by the bite.
                        && (!(degree[b] == 0 || gape[a] > 0.8 * bulk[body[b]]) || bulk[body[b]] >= mesh[a])
                    {
                        // The bigger the mouth is than its prey, the faster it
                        // gets it down: nothing against prey a quarter larger
                        // than itself, full speed at half again its size.
                        let outsize = gape[a] / e.cells[b].material.as_ref().unwrap().biomass;
                        let advantage = ((outsize - 0.8) / 0.7).clamp(0., 1.);
                        // A body small enough to go in whole is not pulled
                        // apart, so its hold on its own cells is no help to it.
                        let whole = gape[a] > 0.8 * bulk[body[b]];
                        // Anything larger has to be torn apart, and a mouth
                        // is no better at that for being bigger than the
                        // one cell it has hold of: what it pulls against is
                        // the body, every tie the cell has to it and the
                        // matrix laid down between them.
                        let grip =
                            if whole { 0. } else { degree[b] as f64 * TIE * (1. + e.cells[b].scaffold() as f64) };
                        let spines = e.cells[b].activity(SPINE) as f64;
                        let wall = e.cells[b].activity(WALL) as f64;
                        // Engulfers recognise and spare other engulfers.
                        // Engulfers spare armed kin, even wounded or half-built,
                        // and every cell of a kin body that has a mouth: an
                        // animal's young do not eat their own kind's fins.
                        let spared = if kin(a, b) {
                            if mouthed[body[b]] {
                                1.
                            } else {
                                (e.cells[b].activity(ENGULF) as f64 * 4.).min(1.)
                            }
                        } else {
                            0.
                        };
                        let capacity = (e.cells[a].material.as_ref().unwrap().biomass * 1.5
                            - e.cells[a].material.as_ref().unwrap().fuel)
                            .max(0.);
                        let available = e.cells[a].energy as f64 / 0.2;
                        let reach = e.cells[a].area();
                        // Teeth: mineral in the mouth itself. Armour and a
                        // body's hold on its own cells count for less
                        // against a mouth that is hard.
                        let jaw = {
                            let am = e.cells[a].material.as_ref().unwrap();
                            1. + JAW * (am.mineral / am.biomass).min(0.3)
                        };
                        let prey = e.cells[b].material.as_mut().unwrap();
                        let potential = (DT as f64 * 0.3 * mouth * reach * advantage * (1. - spared))
                            .min((prey.biomass - 0.005).max(0.))
                            .min(capacity / 0.85)
                            .min(available);
                        let extracted = potential
                            / (1.
                                + (spines * 6.
                                    + wall * 10.
                                    + prey.shell() * 6.
                                    + prey.mineral / prey.biomass * 4.
                                    + grip)
                                    / jaw);
                        // Swallowing takes the prey's fuel store along with its body.
                        let swallowed = (prey.fuel * extracted / prey.biomass.max(1e-9))
                            .min(prey.fuel)
                            .min((capacity - extracted * 0.85).max(0.));
                        prey.fuel -= swallowed;
                        prey.biomass -= extracted;
                        let damage = (extracted / prey.biomass.max(0.02)) as f32;
                        e.cells[b].state.flux.defense_blocked += potential - extracted;
                        e.cells[b].state.flux.lost_to_predators += extracted;
                        e.cells[b].state.injury = (e.cells[b].state.injury + damage).min(1.);
                        e.cells[b].state.harm[4] += damage;
                        self.took(&e.cells[a], &e.cells[b], extracted);
                        // A bite holds on. What a mouth has hold of goes where
                        // the mouth goes: the two share their momentum, as
                        // anything does that grabs something else.
                        if extracted > 0. {
                            e.cells[a].state.handling = HANDLE;
                            e.cells[a].state.prey = e.cells[b].id;
                            let (ma, mb) = (
                                e.cells[a].material.as_ref().unwrap().mass() as f32,
                                e.cells[b].material.as_ref().unwrap().mass() as f32,
                            );
                            let together = e.cells[a].v.mul(ma).add(e.cells[b].v.mul(mb)).mul(1. / (ma + mb));
                            e.cells[a].v = together;
                            e.cells[b].v = together;
                        }
                        if body.get(b).map_or(e.cells[b].state.flux.eater(), |&g| animal[g]) {
                            e.cells[a].state.flux.prey_animal += extracted * 0.85 + swallowed;
                        }
                        // Once in a great while a swallowed light-feeder is not
                        // digested but kept alive inside: the host gains its
                        // captive's light harvesting, and its descendants
                        // inherit it.
                        let captive = e.cells[b].activity(0);
                        if extracted > 0.
                            && captive > 0.25
                            && captive > e.cells[a].state.symbiont
                            && core.random() < KEPT_LIGHT
                        {
                            if e.cells[a].state.symbiont == 0. {
                                self.steering.captures += 1;
                            }
                            e.cells[a].state.symbiont = captive.min(1.);
                        }
                        // And now and then a swallowed breather: kept, it
                        // breathes for its host from then on, mouth or no
                        // mouth. Every animal's cells descend from such a pair.
                        let breather = e.cells[b].activity(RESPIRE);
                        if extracted > 0.
                            && breather > 0.25
                            && breather > e.cells[a].state.mitochondria
                            && core.random() < 0.01
                        {
                            if e.cells[a].state.mitochondria == 0. {
                                self.steering.captures += 1;
                            }
                            e.cells[a].state.mitochondria = breather.min(1.);
                        }
                        e.cells[a].state.flux.prey += extracted * 0.85 + swallowed;
                        e.cells[a].state.flux.organic_waste += extracted * 0.15;
                        self.record_meal(
                            core.time,
                            e.cells[a].id,
                            e.cells[b].id,
                            e.cells[a].genome.id,
                            e.cells[b].genome.id,
                            "engulfed",
                            extracted + swallowed,
                            [crate::observations::kind(&e.cells[a]), crate::observations::kind(&e.cells[b])],
                        );
                        let prey_steps = e.cells[b].state.trophic;
                        let am = e.cells[a].material.as_mut().unwrap();
                        am.fuel += extracted * 0.85 + swallowed;
                        am.nitrogen += extracted * 0.85 * N_RATIO;
                        am.phosphorus += extracted * 0.85 * P_RATIO;
                        let held = am.organic();
                        self.web.feed(
                            &mut e.cells[a].state,
                            held,
                            extracted * 0.85 + swallowed,
                            crate::trophic::Source::Prey(prey_steps),
                        );
                        let site = self.domain.locate(e.cells[a].p);
                        self.solutes[site][5] += extracted * 0.15;
                        self.solutes[site][REMAINS_N] += extracted * 0.15 * N_RATIO;
                        self.solutes[site][2] += extracted * 0.15 * P_RATIO;
                        self.physiology.digested += extracted;
                        self.physiology.excreted += extracted * 0.15;
                        self.spend(&mut e.cells[a], extracted * 0.2, false);
                        if extracted > 0. {
                            // Caught prey is held: the pair move as one from here
                            // on, sharing their momentum.
                            let (ma, mb) = (
                                e.cells[a].material.as_ref().unwrap().mass() as f32,
                                e.cells[b].material.as_ref().unwrap().mass() as f32,
                            );
                            let shared = e.cells[a]
                                .v
                                .mul(ma / (ma + mb))
                                .add(e.cells[b].v.mul(mb / (ma + mb)));
                            e.cells[a].v = shared;
                            e.cells[b].v = shared;
                        }
                        // A mouth lets go of what pricks it: the more fully armed the
                        // prey, the sooner, so a fully spined cell is simply left alone.
                        // What it is not trying to swallow does not prick it.
                        let pricked =
                            (DT as f64 * 0.05 * spines * (1. - spines) * mouth * (1. - spared)) as f32;
                        e.cells[a].state.injury = (e.cells[a].state.injury + pricked).min(1.);
                        e.cells[a].state.harm[5] += pricked;
                        e.cells[b].state.flux.wounded += pricked as f64;
                        if extracted > 0. {
                            radii[a] = e.cells[a].radius();
                            radii[b] = e.cells[b].radius();
                            e.meals += 1;
                            e.cells[a].state.meals += 1;
                        }
                    }
                    // Venom wounds any other body it touches: a weapon, and a
                    // reason not to bite. Skin blunts it.
                    let venom = e.cells[a].activity(TOXIN) as f64;
                    if venom > 0. && e.cells[a].energy > 0.05 && body[a] != body[b] {
                        let wanted = DT as f64 * 0.01 * venom;
                        let paid = self.spend(&mut e.cells[a], wanted, false);
                        let skin = e.cells[b].activity(crate::physiology::BARRIER) as f64;
                        // Venomous cells are immune to venom.
                        let immune = if kin(a, b) {
                            (e.cells[b].activity(TOXIN) as f64 * 4.).min(1.)
                        } else {
                            0.
                        };
                        let dose =
                            (DT as f64 * 0.03 * venom * (1. - immune) * (paid / wanted).min(1.)
                                / (1. + skin * 4.)) as f32;
                        e.cells[b].state.injury = (e.cells[b].state.injury + dose).min(1.);
                        e.cells[b].state.harm[6] += dose;
                        e.cells[a].state.flux.wounded += dose as f64;
                    }
                    if e.cells[a].activity(8) > 0.1
                        && core.random() >= e.cells[a].state.germline * 0.95
                        && e.cells[a].lifecycle.phase != crate::development::Phase::Gamete
                        && e.cells[b].lifecycle.phase != crate::development::Phase::Gamete
                        && e.cells[a].energy > 0.2
                        && e.cells[a].genome.id != e.cells[b].genome.id
                        // A cell takes up genes from other individuals, not
                        // from the other cells of its own body: those are
                        // copies of itself, and trading copies only piles
                        // the same genes up.
                        && e.cells[a].body_origin() != e.cells[b].body_origin()
                        && e.cells[a].genome.genes.len() < autonomous::MAX_GENES
                        && core.random() < 0.002
                    {
                        let donor = e.cells[b].genome.genes
                            [(core.random() * e.cells[b].genome.genes.len() as f32) as usize]
                            .clone();
                        e.cells[a].genome.genes.push(donor);
                        e.cells[a].genome.id = e.next_genome;
                        e.next_genome += 1;
                        self.spend(&mut e.cells[a], 0.1, false);
                        e.exchanges += 1;
                    }
                }
            }
        }
        for (c, n) in e.cells.iter_mut().zip(touching) {
            c.state.touching = n;
        }
        pairs
    }
    #[cfg(test)]
    fn drag(&mut self, c: &mut Cell, dt: f64) {
        let m = c.material.as_ref().unwrap();
        let mass = m.mass();
        let gamma = BODY_DRAG * m.radius() as f64;
        let site = self.domain.locate(c.p);
        self.drag_cached(c, mass, gamma * dt / mass, site);
    }
    fn drag_cached(&mut self, c: &mut Cell, mass: f64, ratio: f64, site: usize) {
        if let Some(flow) = &mut self.fluid.surface {
            let water = self.planet.as_ref().unwrap().water[site];
            if water < 1e-9 {
                return;
            }
            let fluid_mass = water * WATER_DENSITY;
            let up = self.domain.up(c.p);
            let tangent = c.v.sub(up.mul(c.v.dot(up)));
            let uv = flow.velocities[site];
            let uv = uv.sub(up.mul(uv.dot(up)));
            let delta = uv
                .sub(tangent)
                .mul((ratio / (1. + ratio * (1. + mass / fluid_mass))) as f32);
            c.v = c.v.add(delta);
            flow.velocities[site] =
                flow.velocities[site].sub(delta.mul((mass / fluid_mass) as f32));
            let normal = up.mul(c.v.dot(up));
            c.v = c.v.sub(normal).add(normal.mul((1. / (1. + ratio)) as f32));
            return;
        }
        let u = self.fluid.velocity(site);
        let old = c.v;
        c.v.x = ((old.x as f64 + ratio * u[0]) / (1. + ratio)) as f32;
        c.v.z = ((old.z as f64 + ratio * u[1]) / (1. + ratio)) as f32;
        c.v.y = (old.y as f64 / (1. + ratio)) as f32;
        self.fluid.impulse(
            site,
            [
                -mass * (c.v.x as f64 - old.x as f64),
                -mass * (c.v.z as f64 - old.z as f64),
            ],
        );
    }
    /// How much of a cell is under water, and how deep the water is where it
    /// floats or stands. The sea stands at a fixed level in a flat habitat and
    /// wherever it has pooled on a planet.
    /// The share of the surface's light that reaches this far under it.
    pub(crate) fn fade(depth: f32) -> f64 {
        (-CLARITY * depth.max(0.) as f64).exp()
    }
    pub(crate) fn submerged(&self, c: &Cell, radius: f32, seed: u32) -> (f64, f32) {
        self.submerged_at(c, radius, seed, self.domain.coords(c.p))
    }
    /// As `submerged`, for a cell whose map coordinates are already known.
    fn submerged_at(&self, c: &Cell, radius: f32, seed: u32, at: (f32, f32)) -> (f64, f32) {
        let (wet, deep, ..) = self.column_at(c, radius, seed, at);
        (wet, deep)
    }
    /// Where a cell is in the water: the share of it under the surface, how
    /// deep the water is there, how far the cell is under the surface and
    /// how far above the bed.
    fn column_at(&self, c: &Cell, radius: f32, seed: u32, (x, z): (f32, f32)) -> (f64, f32, f32, f32) {
        let p = self.planet.as_ref().unwrap();
        let ground = self.domain.ground(x, z, seed) + p.relief_at(x, z);
        let (surface, altitude) = if self.domain.spherical() {
            let depth = p.water_over(self.domain.index(x, z), ground, seed);
            (ground + depth, c.p.len() - self.domain.radius())
        } else {
            (ground.max(0.12), c.p.y)
        };
        let wet = if surface > ground {
            ((surface - altitude + radius) / (2. * radius)).clamp(0., 1.) as f64
        } else {
            0.
        };
        (wet, surface - ground, surface - altitude, altitude - radius - ground)
    }
    /// How the water carries a body: the share of it under water, what it
    /// weighs there beside its weight on land, and the depth beneath it.
    pub fn bearing(&self, e: &Evolution, group: &[usize], seed: u32) -> serde_json::Value {
        let (mut wet, mut weight, mut land, mut depth) = (0., 0., 0., 0f32);
        for &i in group {
            let c = &e.cells[i];
            let m = c.material.as_ref().unwrap();
            let (under, deep) = self.submerged(c, m.radius(), seed);
            wet += under;
            weight += m.mass() - m.displaced() * under;
            land += m.mass();
            depth += deep;
        }
        let n = group.len().max(1) as f64;
        let feet = group.iter().filter(|&&i| e.cells[i].state.ground).count();
        let mean = |f: &dyn Fn(&Cell) -> f32| group.iter().map(|&i| f(&e.cells[i])).sum::<f32>() / n as f32;
        serde_json::json!({"submerged":wet/n,"weight":weight/land.max(1e-12),"depth":depth/n as f32,
            "load":mean(&|c|c.state.load),"feet":feet,"dry":mean(&|c|c.state.dry),"thirst":mean(&|c|c.state.thirst),
            "sinks":group.iter().map(|&i|{let m=e.cells[i].material.as_ref().unwrap();m.mass()-m.displaced()}).sum::<f64>()/land.max(1e-12)})
    }
    /// Out of the water a body bears its whole weight on whatever touches the
    /// ground, and dry air draws the water out of bare cells. A body thicker
    /// than its footing can carry is crushed at the base unless bone, walls
    /// or fibre hold it up. Skin or a wall keeps the water in.
    pub(crate) fn land(&mut self, e: &mut Evolution) {
        let Some(p) = self.planet.as_ref() else {
            return;
        };
        for members in e.groups() {
            let (mut weight, mut capacity) = (0., 0.);
            for &i in &members {
                let c = &e.cells[i];
                let m = c.material.as_ref().unwrap();
                weight += (GRAVITY * (m.mass() - m.displaced() * c.state.under as f64)).max(0.);
                if c.state.ground {
                    capacity += c.strength();
                }
            }
            let load = if capacity > 0. { (weight / capacity).min(4.) as f32 } else { 0. };
            for &i in &members {
                let c = &mut e.cells[i];
                c.state.load = load;
                if c.state.ground && load > 1. {
                    c.state.injury = (c.state.injury + DT * CRUSH * (load - 1.)).min(1.);
                    c.state.harm[7] += DT * CRUSH * (load - 1.);
                }
            }
        }
        for c in &mut e.cells {
            let site = self.domain.locate(c.p);
            let film = (p.water[site] / self.domain.area(site) / DAMP).min(1.) as f32;
            let air = (1. - 2. * c.state.under).max(0.) * (1. - film);
            let seal = 1.
                - (1. - c.activity(crate::physiology::BARRIER)) * (1. - 0.7 * c.activity(WALL));
            let stored = c.material.as_ref().map_or(0., |m| m.body.water / (0.02 + m.body.water));
            c.state.thirst = air * (1. - stored as f32);
            c.state.dry = air * (1. - 0.95 * seal);
            c.state.injury = (c.state.injury + DT * DRYING * c.state.dry).min(1.);
            c.state.harm[8] += DT * DRYING * c.state.dry;
            // Above a planet's air there is nothing to breathe, fly in or
            // be shielded by. What is thrown that high dies there, and its
            // remains come down on the ground beneath it.
            if self.domain.spherical() && c.state.height - p.surface_depth(site) as f32 > SPACE {
                c.state.injury = 1.;
                c.state.harm[9] += 1.;
            }
        }
    }
    /// The water's hold on a cell that is part of a body: less along each
    /// line to a cell it is tied to, more across a fin. The water takes the
    /// opposite push.
    fn drag_shaped(
        &mut self,
        c: &mut Cell,
        mass: f64,
        ratio: f64,
        cover: &[(V3, f32)],
        blade: Option<(V3, f32)>,
        site: usize,
    ) {
        // Cover shares the water's hold out by direction and takes nothing
        // from it in all: what a cell is spared along its ties it meets
        // across them. So a body is held as hard as before, taken over
        // every way it might move, and its shape decides which ways.
        let even = 1. / (1. - cover.iter().map(|&(_, share)| share).sum::<f32>() / 3.).max(0.4);
        let widest = (even + blade.map_or(0., |(_, span)| span)) as f64;
        // What the water's speed past the cell comes to, by the shape the
        // cell shows it.
        let met = |slip: V3| {
            let mut held = slip;
            for &(along, share) in cover {
                held = held.sub(along.mul(share * slip.dot(along)));
            }
            // Cover never reaches the whole of a cell.
            if held.dot(slip) < 0.2 * slip.dot(slip) {
                held = slip.mul(0.2);
            }
            held = held.mul(even);
            if let Some((across, span)) = blade {
                held = held.add(across.mul(span * slip.dot(across)));
            }
            held
        };
        if let Some(flow) = &mut self.fluid.surface {
            let water = self.planet.as_ref().unwrap().water[site];
            if water < 1e-9 {
                return;
            }
            let fluid_mass = water * WATER_DENSITY;
            let up = self.domain.up(c.p);
            let uv = flow.velocities[site];
            let uv = uv.sub(up.mul(uv.dot(up)));
            let delta = met(uv.sub(c.v))
                .mul((ratio / (1. + ratio * widest * (1. + mass / fluid_mass))) as f32);
            c.v = c.v.add(delta);
            let level = delta.sub(up.mul(delta.dot(up)));
            flow.velocities[site] =
                flow.velocities[site].sub(level.mul((mass / fluid_mass) as f32));
            return;
        }
        let u = self.fluid.velocity(site);
        let old = c.v;
        let slip = V3::new(u[0] as f32 - old.x, -old.y, u[1] as f32 - old.z);
        c.v = old.add(met(slip).mul((ratio / (1. + ratio * widest)) as f32));
        self.fluid.impulse(
            site,
            [
                -mass * (c.v.x as f64 - old.x as f64),
                -mass * (c.v.z as f64 - old.z as f64),
            ],
        );
    }
    fn sync_joints(&mut self, e: &Evolution) {
        let ids = e.index();
        let mut neighbors = vec![Vec::new(); e.cells.len()];
        for b in &e.bonds {
            let (a, c) = (ids[&b.a], ids[&b.b]);
            if b.integrity > 0. && e.cells[a].rigidity().min(e.cells[c].rigidity()) > 0.025 {
                neighbors[a].push(b.b);
                neighbors[c].push(b.a);
            }
        }
        let previous: BTreeMap<_, _> = std::mem::take(&mut self.joints)
            .into_iter()
            .map(|j| ((j.a, j.b, j.c), j))
            .collect();
        for (b, ns) in neighbors.iter_mut().enumerate() {
            ns.sort_unstable();
            // Bound bending constraints to three adjacent pairs per cell.
            for ends in ns.windows(2).take(3) {
                let (a, c) = (ends[0], ends[1]);
                let key = (a, e.cells[b].id, c);
                let (pa, pb, pc) = (e.cells[ids[&a]].p, e.cells[b].p, e.cells[ids[&c]].p);
                let angle = crate::articulation::angle_and_gradients(pa, pb, pc).0;
                // Three cells laid in a straight and level line make a
                // backbone's joint, which can turn either way.
                let up = self.domain.up(pb);
                let level = |p: V3| {
                    let arm = p.sub(pb);
                    (arm.dot(up) / arm.len().max(1e-4)).abs() < 0.5
                };
                let joint = previous.get(&key).cloned().unwrap_or(MaterialJoint {
                    a,
                    b: e.cells[b].id,
                    c,
                    rest: angle,
                    target: angle,
                    angle,
                    flexion: 0.,
                    extension: 0.,
                    work: 0.,
                    // Or three cells that the body's plan set one after
                    // another in a line, along its length or out along a
                    // limb, however they happen to lie when their bone sets.
                    straight: (angle > STRAIGHT && level(pa) && level(pc)) || {
                        let place = |i: usize| (e.cells[i].state.axial as i32, e.cells[i].state.lateral as i32);
                        let ((xa, ya), (xb, yb), (xc, yc)) = (place(ids[&a]), place(b), place(ids[&c]));
                        (ya == yb && yb == yc && (xa - xb).abs() == 1 && xa + xc == 2 * xb)
                            || (xa == xb && xb == xc && (ya - yb).abs() == 1 && ya + yc == 2 * yb)
                    },
                    bend: 0.,
                    aim: 0.,
                });
                self.joints.push(joint);
            }
        }
    }
    fn joint_k(e: &Evolution, a: usize, b: usize, c: usize) -> f32 {
        [a, b, c]
            .into_iter()
            .map(|i| e.cells[i].rigidity())
            .fold(1., f32::min)
            * JOINT_STRENGTH
    }
    fn spine_k(e: &Evolution, a: usize, b: usize, c: usize) -> f32 {
        [a, b, c]
            .into_iter()
            .map(|i| e.cells[i].rigidity())
            .fold(1., f32::min)
            * SPINE_STRENGTH
            * Self::bulk(e, a, b, c)
    }
    /// How much more a joint's muscle and bone can do in a cell this size
    /// than in a cell of the usual size. Muscle pulls by its cross-section
    /// and works on a lever as long as the cell is wide, so its twist goes
    /// as the cell's volume; so does the water a cell that size has to move.
    /// A joint is as strong as the least of the three cells it joins.
    fn bulk(e: &Evolution, a: usize, b: usize, c: usize) -> f32 {
        ([a, b, c].into_iter().map(|i| e.cells[i].radius()).fold(f32::MAX, f32::min) / 0.22).powi(3)
    }
    fn prepare_joint_motors(&mut self, e: &mut Evolution) {
        let ids = e.index();
        // Who is tied to whom, for the nerve cells that ride on a backbone.
        let mut tied = vec![Vec::new(); e.cells.len()];
        if self.joints.iter().any(|j| j.straight) {
            for b in &e.bonds {
                if b.integrity > 0. {
                    let (x, y) = (ids[&b.a], ids[&b.b]);
                    tied[x].push(y);
                    tied[y].push(x);
                }
            }
        }
        // Temporarily take the vector so ATP accounting can borrow the habitat.
        let mut joints = std::mem::take(&mut self.joints);
        for j in &mut joints {
            let (a, b, c) = (ids[&j.a], ids[&j.b], ids[&j.c]);
            let angle =
                crate::articulation::angle_and_gradients(e.cells[a].p, e.cells[b].p, e.cells[c].p)
                    .0;
            let cell = &e.cells[b];
            let motor = if self.passive { 0. } else { cell.activity(4) };
            let signals = cell.motor_signals();
            if j.straight {
                let (pa, pb, pc) = (e.cells[a].p, cell.p, e.cells[c].p);
                let up = self.domain.up(pb);
                let bend = crate::articulation::bend_and_gradients(pa, pb, pc, up).0;
                // A backbone's joint is pulled from its two sides, each by
                // the nerve cells that ride on that side of it: one way
                // when those on its left fire, the other when those on its
                // right do. With none riding on it, it is turned by its own
                // cell's two signals, as any joint is.
                let left = crate::articulation::cross(up, pc.sub(pa));
                let (mut port, mut starboard, mut ridden) = (0f32, 0f32, false);
                for &n in &tied[b] {
                    let rider = &e.cells[n];
                    if n == a || n == c || rider.activity(crate::physiology::CHANNEL) < 0.02 {
                        continue;
                    }
                    ridden = true;
                    let firing = ((rider.state.excitation - 0.3) * 5.).clamp(0., 1.);
                    if rider.p.sub(pb).dot(left) > 0. {
                        port = port.max(firing);
                    } else {
                        starboard = starboard.max(firing);
                    }
                }
                if !ridden {
                    // A nerve cell of its own stands for both sides of its
                    // segment: the joint is pulled one way while the cell
                    // fires and the other way while it recovers.
                    (port, starboard) = if cell.activity(crate::physiology::CHANNEL) >= 0.02 {
                        let firing = ((cell.state.excitation - 0.3) * 5.).clamp(0., 1.);
                        (firing, ((cell.state.recovery - 0.1) * 5.).clamp(0., 1.) * (1. - firing))
                    } else {
                        (signals[0], signals[1])
                    };
                }
                let (port, starboard) = (motor * port, motor * starboard);
                let wanted = ((port - starboard) * SWING).clamp(-STOP, STOP);
                let next = j.aim + (wanted - j.aim) * EASE;
                let k = Self::spine_k(e, a, b, c) as f64;
                // Never more work than its muscle could do in turning it that far.
                let increase = (0.5 * k * ((bend - next).powi(2) - (bend - j.aim).powi(2)) as f64)
                    .clamp(0., (MUSCLE_TWIST * Self::bulk(e, a, b, c) * (next - j.aim).abs()) as f64);
                let required =
                    increase * WORK_PRICE + DT as f64 * 0.001 * (port + starboard) as f64;
                let fraction = if required > 0. {
                    (Self::spare(&e.cells[b]) / required).min(1.)
                } else {
                    1.
                };
                j.aim += (next - j.aim) * fraction as f32;
                j.flexion = (port * fraction as f32).clamp(0., 1.);
                j.extension = (starboard * fraction as f32).clamp(0., 1.);
                let paid = self.spend(&mut e.cells[b], required * fraction, true);
                j.work += paid;
                e.motor_work += paid;
                j.bend = bend;
                j.angle = angle;
                j.target = std::f32::consts::PI - j.aim.abs();
                continue;
            }
            let flexion = motor * signals[0];
            let extension = motor * signals[1];
            let next = (j.rest + (extension - flexion) * 0.45).clamp(0., std::f32::consts::PI);
            let k = Self::joint_k(e, a, b, c) as f64;
            let increase =
                (0.5 * k * ((angle - next).powi(2) - (angle - j.target).powi(2)) as f64).max(0.);
            let required = increase + DT as f64 * 0.001 * (flexion + extension) as f64;
            let fraction = if required > 0. {
                (Self::spare(&e.cells[b]) / required).min(1.)
            } else {
                1.
            };
            j.target += (next - j.target) * fraction as f32;
            j.flexion = flexion * fraction as f32;
            j.extension = extension * fraction as f32;
            let paid = self.spend(&mut e.cells[b], required * fraction, true);
            j.work += paid;
            e.motor_work += paid;
            j.angle = angle;
        }
        self.joints = joints;
    }
    pub(crate) fn mechanics(&mut self, e: &mut Evolution, core: &mut Core, pairs: &[(usize, usize)]) {
        self.sync_joints(e);
        self.prepare_joint_motors(e);
        let ids = e.index();
        for c in &mut e.cells {
            c.state.beat = 0.;
        }
        for b in &mut e.bonds {
            let (a, c) = (ids[&b.a], ids[&b.b]);
            let (left, right) = e.cells.split_at_mut(c.max(a));
            let (x, y) = if a < c {
                (&mut left[a], &mut right[0])
            } else {
                (&mut right[0], &mut left[c])
            };
            let (xm, ym) = (x.material.as_ref().unwrap(), y.material.as_ref().unwrap());
            let polymer =
                (xm.body.properties(xm.biomass)[2] + ym.body.properties(ym.biomass)[2]) * 0.5;
            let scaffold = (x.scaffold() + y.scaffold()) * 0.5;
            b.stiffness = (((xm.mineral / xm.mass() + ym.mineral / ym.mass()) * 0.5) as f32
                + scaffold * 0.08
                + polymer * 0.05)
                .min(1.);
            let fiber_axis = x.p.sub(y.p).normalized();
            let alignment = (fiber_axis.dot(self.domain.tangent(x.p, x.heading)).abs()
                + fiber_axis.dot(self.domain.tangent(y.p, y.heading)).abs())
                * 0.5;
            let fiber_strength = ((xm.body.fibers / (0.02 * xm.biomass + xm.body.fibers)) as f32
                + (ym.body.fibers / (0.02 * ym.biomass + ym.body.fibers)) as f32)
                * 0.5;
            b.stiffness = (b.stiffness + fiber_strength * alignment * 0.1).min(1.);
            let catalyst = (x.activity(4) + y.activity(4)) * 0.5;
            // Muscle is springy tissue: it pulls hard and snaps back when it lets go.
            b.stiffness = (b.stiffness + catalyst * MUSCLE_SPRING).min(1.);
            let xs = x.motor_signals();
            let ys = y.motor_signals();
            let drive = (xs[0] + ys[0]) * 0.5 * (1. - (xs[1] + ys[1]) * 0.5);
            // Muscle between two bones has nothing to shorten: bone keeps its
            // length, and what muscle does there it does by turning the joint.
            let next = if self.passive || x.rigidity().min(y.rigidity()) > 0.025 {
                0.
            } else {
                // A muscle tightens faster than it lets go.
                b.activation
                    + (catalyst * drive - b.activation)
                        * if catalyst * drive > b.activation { MUSCLE_QUICK } else { MUSCLE_SLOW }
            };
            let d = x.p.sub(y.p).len() as f64;
            let k = Self::spring_k(b);
            let old_energy =
                0.5 * k * (d - b.rest as f64 * (1. - 0.22 * b.activation as f64)).powi(2);
            let new_energy = 0.5 * k * (d - b.rest as f64 * (1. - 0.22 * next as f64)).powi(2);
            // The work of tightening, priced as a swimming motor's work is,
            // and a little for holding tight.
            let required =
                (new_energy - old_energy).max(0.) * WORK_PRICE + DT as f64 * 0.001 * next as f64;
            let fraction = if required > 0. {
                ((x.energy.min(y.energy) as f64 * 2.) / required).min(1.)
            } else {
                1.
            };
            b.activation += (next - b.activation) * fraction as f32;
            let paid = self.spend(x, required * fraction * 0.5, true)
                + self.spend(y, required * fraction * 0.5, true);
            e.motor_work += paid;
            // The work of tightening is what stirs; holding tight does not.
            let stirred = ((new_energy - old_energy).max(0.) * WORK_PRICE * fraction * 0.5) as f32;
            x.state.beat += stirred;
            y.state.beat += stirred;
        }
        let step = DT / 20.;
        // Material inventories do not change within the mechanical substeps.
        // Avoid repeated cbrt, mass sums and tree lookups; preserve force order.
        let mut submersion = Vec::with_capacity(e.cells.len());
        // Where each cell is on the map, kept up as cells are moved, so the
        // conversion is done once for each place a cell is at.
        let mut places = Vec::with_capacity(e.cells.len());
        let mut column = Vec::with_capacity(e.cells.len());
        let properties: Vec<_> = e
            .cells
            .iter()
            .map(|c| {
                let m = c.material.as_ref().unwrap();
                let mass = m.mass();
                let radius = m.radius();
                let at = self.domain.coords(c.p);
                places.push(at);
                let (wet, _, under, above) = self.column_at(c, radius, core.config.seed, at);
                column.push((under.max(0.), above.max(0.)));
                submersion.push(wet as f32);
                // Water carries whatever part of a cell is under it.
                let weight = (GRAVITY * (mass - m.displaced() * wet)) as f32;
                let gamma = BODY_DRAG * radius as f64;
                (mass, radius, weight, gamma * step as f64 / mass)
            })
            .collect();
        // Water drags on a body as a whole, by how far it reaches, not on
        // each of its cells as if the others were not there. A compact body
        // of many cells therefore meets far less drag for its mass than one
        // cell does, and with motors on its surface it swims faster: about
        // the same number of body lengths a cycle at any size. A chain one
        // cell thick gains nothing, as a rod pulled through water does not.
        let mut properties = properties;
        for (c, &(under, above)) in e.cells.iter_mut().zip(&column) {
            c.state.depth = under;
            c.state.height = above;
        }
        // Within a body the water still resists each cell's own movement
        // against its neighbours, as before. Without that a large body's
        // springs ring until it flies apart.
        let mut bodies: Vec<(Vec<usize>, Vec<f32>, f64)> = Vec::new();
        // Bodies neither form nor break while they are being moved.
        let groups = e.groups();
        for members in groups.iter().filter(|g| g.len() >= 3).cloned() {
            let mass: f64 = members.iter().map(|&i| properties[i].0).sum();
            let centre = members
                .iter()
                .fold(V3::default(), |sum, &i| sum.add(e.cells[i].p.mul((properties[i].0 / mass) as f32)));
            let reach = members
                .iter()
                .map(|&i| e.cells[i].p.sub(centre).len() + properties[i].1)
                .fold(0f32, f32::max);
            let ratio = BODY_DRAG * reach as f64 * step as f64 / mass;
            let own: Vec<f32> = members.iter().map(|&i| (1. / (1. + properties[i].3)) as f32).collect();
            for &i in &members {
                properties[i].3 = properties[i].3.min(ratio);
            }
            bodies.push((members, own, mass));
        }
        let springs: Vec<_> = e
            .bonds
            .iter()
            .map(|b| {
                // A tie is as strong as the area it joins: between two parts
                // twice as wide, four times as stiff. A joint's push already
                // grows that way, and a tie that did not was torn by its own
                // body's strokes once the parts were large. (In a body with
                // a plan, whose parts are sized by their place. Applied to
                // every tie it also flattened a clump of algae, which is a
                // question about small walled cells still to be looked at.)
                let (i, j) = (ids[&b.a], ids[&b.b]);
                let area = if e.cells[i].state.fate != 0 || e.cells[j].state.fate != 0 {
                    (e.cells[i].radius().min(e.cells[j].radius()) / 0.22).powi(2)
                } else {
                    1.
                };
                (i, j, b.rest * (1. - b.activation * 0.22), Self::spring_k(b) as f32 * area)
            })
            .collect();
        let angular: Vec<_> = self
            .joints
            .iter()
            .map(|j| {
                let (a, b, c) = (ids[&j.a], ids[&j.b], ids[&j.c]);
                if j.straight {
                    (a, b, c, j.aim, Self::spine_k(e, a, b, c), Self::bulk(e, a, b, c))
                } else {
                    (a, b, c, j.target, Self::joint_k(e, a, b, c), 0.)
                }
            })
            .collect();
        for c in &mut e.cells {
            c.state.strain = 0.;
        }
        // Self-propulsion. A motor pushes the cell along its heading and the
        // medium the other way; the work against drag is paid in ATP. Steering
        // turns the heading by the cell's own sensed left-right differences.
        let mut thrust = vec![V3::default(); e.cells.len()];
        // Crawlers meet the ground's resistance where swimmers meet the water's.
        let mut crawling = vec![false; e.cells.len()];
        let fins: Vec<f64> = e
            .cells
            .iter()
            .map(|c| if self.passive { 0. } else { c.activity(FIN) as f64 })
            .collect();
        let wings: Vec<f32> = e
            .cells
            .iter()
            .map(|c| if self.passive { 0. } else { c.activity(WING) })
            .collect();
        let holds: Vec<f32> = e.cells.iter().map(|c| c.activity(HOLDFAST)).collect();
        let mut bonded = vec![false; e.cells.len()];
        for b in &e.bonds {
            bonded[ids[&b.a]] = true;
            bonded[ids[&b.b]] = true;
        }
        // The water holds a cell by the side of it that meets the water in
        // the way the cell is moving. Along the line to a cell it is tied
        // to it is covered by that cell, and less by a narrower one. A fin
        // is a blade standing along the cell's own line: it widens the cell
        // across that line and not along it. So a wave passed down a body
        // pushes it along, and a stroke that only opens and shuts does not.
        let mut cover: Vec<Vec<(V3, f32)>> = vec![Vec::new(); e.cells.len()];
        for &(a, c, ..) in &springs {
            let along = e.cells[c].p.sub(e.cells[a].p).normalized();
            let (ra, rc) = (properties[a].1, properties[c].1);
            cover[a].push((along, SHIELD * (rc / ra).min(1.).powi(2)));
            cover[c].push((along, SHIELD * (ra / rc).min(1.).powi(2)));
        }
        let blades: Vec<Option<(V3, f32)>> = (0..e.cells.len())
            .map(|i| {
                if fins[i] <= 0. {
                    return None;
                }
                // A cell's line is the line most of its cover lies along.
                let mut line = cover[i].first()?.0;
                for _ in 0..4 {
                    let next = cover[i]
                        .iter()
                        .fold(V3::default(), |sum, &(d, w)| sum.add(d.mul(w * d.dot(line))));
                    if next.len() < 1e-6 {
                        return None;
                    }
                    line = next.normalized();
                }
                let across = crate::articulation::cross(self.domain.up(e.cells[i].p), line);
                (across.len() > 1e-4).then(|| (across.normalized(), FIN_SPAN * fins[i] as f32))
            })
            .collect();
        if !self.passive {
            for (i, c) in e.cells.iter_mut().enumerate() {
                let steer = c.activity(STEER);
                if steer > 0.01 && c.state.turn != 0. {
                    let wanted = DT as f64 * 0.01 * (steer * c.state.turn.abs()) as f64;
                    let paid = self.spend(c, wanted, false);
                    c.heading = (c.heading
                        + TURN_RATE * DT * steer * c.state.turn * (paid / wanted).min(1.) as f32)
                        .rem_euclid(std::f32::consts::TAU);
                }
                // The same rudder points a swimmer up or down.
                if steer > 0.01 && c.state.climb != 0. {
                    let wanted = DT as f64 * 0.01 * (steer * c.state.climb.abs()) as f64;
                    let paid = self.spend(c, wanted, false);
                    c.state.pitch = (c.state.pitch
                        + TURN_RATE * DT * steer * c.state.climb * (paid / wanted).min(1.) as f32)
                        .clamp(-1., 1.);
                }
                // Cilia and flagella work only where a cell faces open water:
                // a cell buried in a body has nothing to beat against. A big
                // body therefore outgrows its motors.
                // Two things a swimming cell can add to its cilia, each long
                // before any body has a use for it. Stiff vanes among them
                // catch more water at each stroke. And an excitable
                // membrane makes them beat harder for as long as it is
                // firing, so a cell that is touched, hurt or startled by
                // light bolts.
                let vanes = 1. + VANE * c.activity(FIN);
                let bolt = 1.
                    + BOLT * c.activity(crate::physiology::CHANNEL) * c.state.excitation;
                let motor = (c.activity(PROPEL)
                    * c.expression[PROPEL]
                    * (1. - c.activity(WALL)).powi(2)
                    * vanes
                    * bolt
                    * if bonded[i] { c.state.exposure } else { 1. }) as f64;
                c.state.wet = 0.;
                c.state.under = submersion[i];
                // The sea is never still. Its eddies jostle a free cell about,
                // more when the oceans are stirred, so drifters spread out
                // instead of piling up where they were born. A body holds its course.
                if !bonded[i] && submersion[i] > 0.1 && !c.state.anchored {
                    let turn = core.random() * std::f32::consts::TAU;
                    // Eddies turn over as well as along, though never far
                    // enough to throw a cell out of the water or into the bed.
                    let lift = ((core.random() * 2. - 1.) * UPWELL * JOSTLE * self.steering.mixing.sqrt())
                        .clamp(-c.state.height, c.state.depth);
                    c.p = c
                        .p
                        .add(
                            self.domain
                                .tangent_at(places[i].0, places[i].1, turn)
                                .mul(JOSTLE * self.steering.mixing.sqrt()),
                        )
                        .add(self.domain.up(c.p).mul(lift));
                    places[i] = self.domain.coords(c.p);
                }
                // Nor is the air. Wind carries a loose cell that lies out of
                // the water along the ground: far if it is as small as a
                // spore, hardly at all if it is big. It comes to rest where
                // it grips, and a holdfast grips only where it is damp.
                // One let go in the air, from the top of a plant, is carried
                // several times as far for as long as it is falling: the
                // wind is slowed along the ground and not above it. So a
                // plant that sheds its spores from a height sends them
                // past its own wet foot, and a taller one farther.
                let aloft = !c.state.ground && self.domain.spherical();
                if !bonded[i]
                    && submersion[i] <= 0.
                    && (c.state.ground || aloft)
                    && !c.state.anchored
                    && self.planet.is_some()
                {
                    let turn = core.random() * std::f32::consts::TAU;
                    let light = (SPORE / properties[i].0).min(1.) as f32 * if aloft { LOFT } else { 1. };
                    let moved = c.p.add(self.domain.tangent_at(places[i].0, places[i].1, turn).mul(GUST * light));
                    let (x, z) = self.domain.coords(moved);
                    let bed = self.domain.ground(x, z, core.config.seed)
                        + self.planet.as_ref().unwrap().relief_at(x, z)
                        + c.radius();
                    // The wind carries a falling cell along and not up: it
                    // comes down as it would have in still air.
                    let height = if aloft {
                        c.p.len() - self.domain.radius()
                    } else if self.domain.spherical() {
                        moved.len() - self.domain.radius()
                    } else {
                        moved.y
                    };
                    // Blown along the ground it follows the ground, and it
                    // is never blown out over the water from the shore.
                    let (wet, ..) = self.column_at(c, c.radius(), core.config.seed, (x, z));
                    if wet <= 0. || height > bed {
                        c.p = self.domain.position(x, z, height.max(bed));
                        places[i] = (x, z);
                    }
                }
                if motor <= 1e-4 {
                    continue;
                }
                // A free cell tumbles; this alone gives an undirected search.
                // A cell held in a body cannot spin on its own.
                if !c.state.ground && !bonded[i] {
                    c.heading = (c.heading + (core.random() - 0.5) * 0.5 * DT.sqrt())
                        .rem_euclid(std::f32::consts::TAU);
                    c.state.pitch =
                        (c.state.pitch + (core.random() - 0.5) * 0.5 * DT.sqrt()).clamp(-1., 1.);
                }
                let site = self.domain.index(places[i].0, places[i].1);
                let water = self
                    .planet
                    .as_ref()
                    .filter(|_| self.domain.spherical())
                    .map_or(f64::INFINITY, |p| p.water[site]);
                // A swimmer can push off only as much water as there is: full
                // thrust in open water, almost none in a film. A cell resting on
                // the ground crawls at half strength instead. Off the ground and
                // out of water there is nothing to push on.
                let (mass, _, _, drag_ratio) = properties[i];
                let fluid_mass = water * WATER_DENSITY;
                let swim = if !self.domain.spherical() {
                    1.
                } else {
                    fluid_mass / (fluid_mass + mass)
                };
                // A motor that beats against water has less and less to push
                // on as the cell leaves it, and nothing once it is two thirds
                // out, so a swimmer cannot drive itself up a beach. Travel
                // over land takes muscle and feet.
                let afloat = if self.domain.spherical() {
                    ((submersion[i] as f64 - 0.3) / 0.4).clamp(0., 1.)
                } else {
                    1.
                };
                let medium = if c.state.ground && swim < 0.5 {
                    crawling[i] = true;
                    0.5 * afloat
                } else {
                    swim * afloat
                };
                c.state.wet = medium as f32;
                // Water lets a body slip; the ground does not. A crawler works
                // against the full grip of what it crawls on.
                let (gamma, price) = if crawling[i] {
                    (drag_ratio * mass / step as f64 / SLIP, 1.)
                } else {
                    (drag_ratio * mass / step as f64, WORK_PRICE)
                };
                let wanted = THRUST * motor * medium * c.area();
                let cost = (wanted * wanted / gamma * price + 0.01 * motor) * DT as f64;
                let paid = self.spend(c, cost, true);
                e.motor_work += paid;
                let force = wanted * (paid / cost).min(1.);
                if force <= 0. {
                    continue;
                }
                // A swimmer pushes the way it points, up and down included;
                // a crawler pushes along the ground it is on.
                let level = self.domain.tangent_at(places[i].0, places[i].1, c.heading);
                let direction = if crawling[i] {
                    level
                } else {
                    // With no water over it there is nothing to climb
                    // through: at the surface a swimmer levels off.
                    let pitch = if c.state.depth <= 0. { c.state.pitch.min(0.) } else { c.state.pitch };
                    level
                        .mul((1. - pitch * pitch).max(0.).sqrt())
                        .add(self.domain.up(c.p).mul(pitch))
                };
                thrust[i] = direction.mul(force as f32);
                let impulse = force * DT as f64;
                if let Some(flow) = &mut self.fluid.surface {
                    if !crawling[i] && water > 1e-9 {
                        let kick = (impulse / fluid_mass).min(0.2) as f32;
                        flow.velocities[site] = flow.velocities[site].sub(direction.mul(kick));
                    }
                } else {
                    self.fluid.impulse(
                        site,
                        [
                            -(direction.x as f64) * impulse,
                            -(direction.z as f64) * impulse,
                        ],
                    );
                }
            }
        }
        // A body's motors push the whole body. Each cell's thrust is pooled
        // and shared by mass, so swimmers in one body do not tear it apart;
        // motors pointing different ways simply cancel, which is wasted work.
        if thrust.iter().any(|t| t.x != 0. || t.y != 0. || t.z != 0.) {
            for group in groups.iter().filter(|g| g.len() >= 2) {
                let net = group
                    .iter()
                    .fold(V3::default(), |sum, &i| sum.add(thrust[i]));
                let mass: f64 = group.iter().map(|&i| properties[i].0).sum();
                for &i in group {
                    thrust[i] = net.mul((properties[i].0 / mass) as f32);
                }
            }
        }
        // Which way each bond points, to see afterwards how the body turned.
        let domain = self.domain;
        let bearing = |e: &Evolution, a: usize, c: usize| {
            let d = e.cells[c].p.sub(e.cells[a].p);
            d.dot(domain.tangent(e.cells[a].p, std::f32::consts::FRAC_PI_2))
                .atan2(d.dot(domain.tangent(e.cells[a].p, 0.)))
        };
        let bearings: Vec<f32> = springs.iter().map(|&(a, c, ..)| bearing(e, a, c)).collect();
        // Terrain and the field site under a cell change little within one
        // tick, so they are sampled on every fourth substep and reused between.
        let mut sites: Vec<usize> = places.iter().map(|&(x, z)| self.domain.index(x, z)).collect();
        let mut terrain = vec![(0f32, false); e.cells.len()];
        // How high the water stood over each cell when this moment began.
        let sea_level: Vec<f32> = e
            .cells
            .iter()
            .zip(&column)
            .map(|(c, &(under, _))| {
                under
                    + if self.domain.spherical() {
                        c.p.len() - self.domain.radius()
                    } else {
                        c.p.y
                    }
            })
            .collect();
        // A working muscle shortens the cells it joins along with itself:
        // they flatten against each other as it pulls, where two cells
        // merely touching are stopped by their own bulk.
        let pulled: std::collections::HashMap<(usize, usize), f32> = e
            .bonds
            .iter()
            .filter(|b| b.activation > 0.01)
            .map(|b| {
                let (a, c) = (ids[&b.a], ids[&b.b]);
                ((a.min(c), a.max(c)), b.activation)
            })
            .collect();
        let room: Vec<f32> = pairs
            .iter()
            .map(|&(i, j)| {
                let tight = if pulled.is_empty() {
                    0.
                } else {
                    pulled.get(&(i.min(j), i.max(j))).copied().unwrap_or(0.)
                };
                (properties[i].1 + properties[j].1) * (1. - 0.22 * tight)
            })
            .collect();
        for substep in 0..20 {
            let mut force = thrust.clone();
            for &(a, c, target, k) in &springs {
                let delta = e.cells[c].p.sub(e.cells[a].p);
                let d = delta.len().max(1e-6);
                let parting = e.cells[c].v.sub(e.cells[a].v).dot(delta) / d;
                // A bond between small cells is a small bond: it is only as
                // strong as the lesser of the two it joins. (Without this a
                // tie that suits a full-sized cell throws a small one about.)
                let slight = (properties[a].0.min(properties[c].0) / SLIGHT).min(1.) as f32;
                // A stiffer tie is damped to match, or it would ring.
                let damping = BOND_DAMPING * (k / 40.).max(1.).sqrt();
                // A tie pulls no harder than it does stretched by half:
                // past that it is giving way, not pulling.
                let stretch = (d - target).min(target * 0.5);
                let f = delta.mul((stretch * k + parting * damping) * slight / d);
                force[a] = force[a].add(f);
                force[c] = force[c].sub(f);
            }
            for &(a, b, c, target, k, bulk) in &angular {
                let straight = bulk > 0.;
                let (pa, pb, pc) = (e.cells[a].p, e.cells[b].p, e.cells[c].p);
                let (gradients, torque, halt, limit) = if straight {
                    let (bend, gradients) =
                        crate::articulation::bend_and_gradients(pa, pb, pc, domain.up(pb));
                    // Muscle toward where it is aiming, no harder than
                    // muscle pulls, and the backbone's own spring.
                    let torque = (-(bend - target) * k).clamp(-MUSCLE_TWIST * bulk, MUSCLE_TWIST * bulk)
                        - BACKBONE * bulk * bend;
                    // Past its stop a joint is bone on bone.
                    let halt = if bend.abs() > STOP {
                        -STOP_STIFF * bulk * (bend - STOP.copysign(bend))
                    } else {
                        0.
                    };
                    (gradients, torque, halt, SPINE_PUSH * bulk.powf(2. / 3.))
                } else {
                    let (angle, gradients) = crate::articulation::angle_and_gradients(pa, pb, pc);
                    (gradients, (-(angle - target) * k).clamp(-JOINT_TORQUE, JOINT_TORQUE), 0., JOINT_PUSH)
                };
                // Two cells pressed almost into one make a lever of no
                // length, and the push from it would be without limit. The
                // three pushes of a joint are held back together, so that
                // they still sum to nothing: held back one by one they
                // push the whole body along with nothing pushed the other way.
                let most = gradients.iter().map(|g| g.len()).fold(0f32, f32::max) * torque.abs();
                let torque = (if most > limit { torque * limit / most } else { torque }) + halt;
                for (i, grad) in [a, b, c].into_iter().zip(gradients) {
                    force[i] = force[i].add(grad.mul(torque));
                }
                // A backbone bends from side to side. Up and down it has
                // only its spring and its stop: no muscle of its own works
                // that way, and without them it would fold where it stood.
                if straight {
                    let side = crate::articulation::cross(domain.up(pb), pc.sub(pa));
                    if side.len() > 1e-3 {
                        let (rise, gradients) =
                            crate::articulation::bend_and_gradients(pa, pb, pc, side.normalized());
                        let torque = -BACKBONE * bulk * rise
                            - if rise.abs() > STOP { STOP_STIFF * bulk * (rise - STOP.copysign(rise)) } else { 0. };
                        for (i, grad) in [a, b, c].into_iter().zip(gradients) {
                            force[i] = force[i].add(grad.mul(torque));
                        }
                    }
                }
            }
            for (&(i, j), &room) in pairs.iter().zip(&room) {
                let delta = e.cells[j].p.sub(e.cells[i].p);
                let d = delta.len().max(1e-6);
                let overlap = room - d;
                if overlap > 0. {
                    let slight = (properties[i].0.min(properties[j].0) / SLIGHT).min(1.) as f32;
                    let f = delta.mul(overlap * 4. * slight / d);
                    force[i] = force[i].sub(f);
                    force[j] = force[j].add(f);
                }
            }
            for (members, own, mass) in &bodies {
                let drift = members.iter().fold(V3::default(), |sum, &i| {
                    sum.add(e.cells[i].v.mul((properties[i].0 / mass) as f32))
                });
                for (&i, &keep) in members.iter().zip(own) {
                    e.cells[i].v = drift.add(e.cells[i].v.sub(drift).mul(keep));
                }
            }
            for (i, c) in e.cells.iter_mut().enumerate() {
                let (mass, radius, weight, drag_ratio) = properties[i];
                let gravity = if self.domain.spherical() {
                    (self.domain.radius() / c.p.len().max(1e-4)).powi(2)
                } else {
                    1.
                };
                force[i] = force[i].sub(self.domain.up(c.p).mul(weight * gravity));
                // A wing moving forward through air carries weight: its
                // own and, through the body's ties, the body's.
                // Air thins with height, so there is a height above the
                // sea or the ground past which no wing holds anything up.
                // A wing's lift is its area times the thickness of the air
                // times the square of its speed through it, whatever the
                // wing weighs. It pushes across the way the wing is moving,
                // never along it, so it turns speed into height and height
                // into speed and adds nothing of its own.
                if wings[i] > 0. && submersion[i] < 0.1 {
                    let speed = c.v.len();
                    if speed > 1e-4 {
                        let going = c.v.mul(1. / speed);
                        let up = self.domain.up(c.p);
                        let across = up.sub(going.mul(up.dot(going)));
                        let over = self
                            .planet_depth(sites[i])
                            .map_or(c.state.height, |sea| c.state.height - sea)
                            .max(0.);
                        let lift = WING_LIFT * wings[i] * c.area() as f32 * (-over / AIR_HEIGHT).exp() * speed * speed;
                        force[i] = force[i].add(across.mul(lift));
                    }
                }
                // A holdfast is a short springy stalk to the spot it gripped.
                // Pulled too far, or left unmade, it lets go.
                if c.state.anchored {
                    let stalk = V3::new(c.state.anchor[0], c.state.anchor[1], c.state.anchor[2]).sub(c.p);
                    if stalk.len() > TEAR + radius || holds[i] < 0.1 {
                        c.state.anchored = false;
                    } else {
                        force[i] = force[i].add(stalk.mul(HOLD * holds[i] * mass as f32));
                    }
                }
                c.v = c.v.add(force[i].mul(step / mass as f32));
                // Nothing alive moves faster than this.
                if c.v.len() > 30. {
                    c.v = c.v.mul(30. / c.v.len());
                }
                // A winged cell out of the water is in air, which drags on
                // it a fiftieth as hard.
                let thin = if wings[i] > 0. { (submersion[i] as f64).max(0.02) } else { 1. };
                if cover[i].is_empty() {
                    self.drag_cached(c, mass, drag_ratio * thin, sites[i]);
                } else {
                    self.drag_shaped(c, mass, drag_ratio * thin, &cover[i], blades[i], sites[i]);
                }
                // And the same air drags on the wing, by the same measure:
                // its area, the air's thickness, the square of its speed.
                if wings[i] > 0. && submersion[i] < 0.1 && !c.state.ground {
                    let over = self
                        .planet_depth(sites[i])
                        .map_or(c.state.height, |sea| c.state.height - sea)
                        .max(0.);
                    let slowed = WING_DRAG
                        * wings[i]
                        * c.area() as f32
                        * (-over / AIR_HEIGHT).exp()
                        * c.v.len()
                        * step
                        / mass as f32;
                    c.v = c.v.mul(1. / (1. + slowed));
                }
                if wings[i] > 0. && submersion[i] < 0.1 {
                    // And its wing grips that air on the back-stroke.
                    let heading = self.domain.tangent(c.p, c.heading);
                    let slip = c.v.dot(heading);
                    if slip < 0. {
                        let grip = drag_ratio * FIN_GRIP * AIR_GRIP * wings[i] as f64;
                        c.v = c.v.add(heading.mul((-slip as f64 * grip / (1. + grip)) as f32));
                    }
                }
                if crawling[i] {
                    let up = self.domain.up(c.p);
                    let normal = up.mul(c.v.dot(up));
                    c.v = normal.add(c.v.sub(normal).mul((1. / (1. + drag_ratio / SLIP)) as f32));
                }
                c.p = c.p.add(c.v.mul(step));
                if !self.domain.spherical() {
                    for axis in [0, 2] {
                        let (p, v) = if axis == 0 {
                            (&mut c.p.x, &mut c.v.x)
                        } else {
                            (&mut c.p.z, &mut c.v.z)
                        };
                        if p.abs() > self.domain.wall() {
                            *p = p.clamp(-self.domain.wall(), self.domain.wall());
                            *v = 0.;
                        }
                    }
                }
                let p = self.planet.as_ref().unwrap();
                if substep % 4 == 0 {
                    let (x, z) = self.domain.coords(c.p);
                    sites[i] = self.domain.index(x, z);
                    terrain[i] = (
                        p.floor(x, z, radius, core.config.seed),
                        if self.domain.spherical() {
                            p.surface_depth(sites[i]) < 0.1
                        } else {
                            self.domain.ground(x, z, core.config.seed) + p.relief_at(x, z) > 0.12
                        },
                    );
                }
                let (floor, firm) = terrain[i];
                let altitude = if self.domain.spherical() {
                    c.p.len() - self.domain.radius()
                } else {
                    c.p.y
                };
                // The surface of water is a skin that nothing as small as a
                // cell breaks through from below, however hard it is thrown:
                // what began this moment under water is still in it at the
                // end. Only what walks out over the ground, or takes wing,
                // leaves the sea, and a wing only from the surface.
                if submersion[i] > 0.5 {
                    let surface = sea_level[i];
                    if altitude > surface && altitude > floor + 0.01 {
                        let up = self.domain.up(c.p);
                        c.p = c.p.sub(up.mul(altitude - surface));
                        let rising = c.v.dot(up);
                        if rising > 0. {
                            c.v = c.v.sub(up.mul(rising));
                        }
                    }
                }
                let altitude = if self.domain.spherical() {
                    c.p.len() - self.domain.radius()
                } else {
                    c.p.y
                };
                let ground = altitude <= floor + 0.001 && firm;
                // A cell with a holdfast grips whatever bed it comes to rest on.
                // Dried out, it is dormant and grips nothing: it lies loose
                // until water reaches it.
                if !c.state.anchored && holds[i] > 0.25 && altitude <= floor + 0.001 && c.state.thirst < 0.5 {
                    c.state.anchored = true;
                    c.state.anchor = [c.p.x, c.p.y, c.p.z];
                }
                if ground != c.state.ground {
                    self.physiology.contact_changes += 1;
                }
                c.state.ground = ground;
                if altitude < floor {
                    let up = self.domain.up(c.p);
                    c.p = c.p.add(up.mul(floor - altitude));
                    let contact_normal = if ground && self.domain.spherical() {
                        let east = self.domain.tangent(c.p, 0.);
                        let north = self.domain.tangent(c.p, std::f32::consts::FRAC_PI_2);
                        let height = |q: V3| {
                            let (a, b) = self.domain.coords(q);
                            self.domain.ground(a, b, core.config.seed) + p.relief_at(a, b)
                        };
                        let slope_e = (height(c.p.add(east.mul(0.03)))
                            - height(c.p.sub(east.mul(0.03))))
                            / 0.06;
                        let slope_n = (height(c.p.add(north.mul(0.03)))
                            - height(c.p.sub(north.mul(0.03))))
                            / 0.06;
                        up.sub(east.mul(slope_e))
                            .sub(north.mul(slope_n))
                            .normalized()
                    } else {
                        up
                    };
                    let normal_speed = c.v.dot(contact_normal);
                    if normal_speed < 0. {
                        c.v = c.v.sub(contact_normal.mul(normal_speed));
                    }
                    // The ground grips a body only as hard as the body presses
                    // on it: fully on land, hardly at all under water.
                    let press = (weight / (GRAVITY * mass) as f32).clamp(0., 1.);
                    let friction = (1.
                        - step * press * (3. + c.activity(crate::physiology::REPAIR) * 3.))
                        .max(0.);
                    let normal = contact_normal.mul(c.v.dot(contact_normal));
                    c.v = normal.add(c.v.sub(normal).mul(friction));
                    // A foot digs in when pushed back along the ground.
                    if fins[i] > 0. {
                        let heading = self.domain.tangent(c.p, c.heading);
                        let back = c.v.dot(heading);
                        if back < 0. {
                            let hold =
                                (step as f64 * 3. * FIN_GRIP * fins[i]).min(1.) as f32 * press;
                            c.v = c.v.sub(heading.mul(back * hold));
                        }
                    }
                }
            }
        }
        // A cell's heading is a direction in its tissue, not on the map: when
        // the body turns, the heading of every cell in it turns too.
        let mut turned = vec![(0f32, 0u8); e.cells.len()];
        for (&(a, c, ..), &before) in springs.iter().zip(&bearings) {
            let swing = (bearing(e, a, c) - before + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            for i in [a, c] {
                turned[i].0 += swing;
                turned[i].1 += 1;
            }
        }
        for (c, &(swing, bonds)) in e.cells.iter_mut().zip(&turned) {
            if bonds > 0 && swing != 0. {
                c.heading = (c.heading + swing / bonds as f32).rem_euclid(std::f32::consts::TAU);
            }
        }
        e.bonds.retain_mut(|b| {
            let (a, c) = (ids[&b.a], ids[&b.b]);
            let (left, right) = e.cells.split_at_mut(a.max(c));
            let (x, y) = if a < c {
                (&mut left[a], &mut right[0])
            } else {
                (&mut right[0], &mut left[c])
            };
            // A working muscle is not strained by reaching the length it pulls to.
            let strain = (x.p.sub(y.p).len() / (b.rest * (1. - b.activation * 0.22)) - 1.).abs();
            // Pulled to more than twice its length, a tie tears at once.
            if x.p.sub(y.p).len() > 2. * b.rest {
                b.integrity = 0.;
            }
            x.state.strain = x.state.strain.max(strain.min(2.));
            y.state.strain = y.state.strain.max(strain.min(2.));
            // Walled sisters share a wall: the join is tough and does not let go.
            // And bone joined to bone along one line of the body is one
            // piece of bone: it does not come apart by chance either. Out
            // along a limb it still can, since that is how these animals'
            // young come away.
            let fused = x.rigidity().min(y.rigidity()) > 0.025 && x.state.lateral == y.state.lateral;
            let cemented = (x.activity(WALL) * y.activity(WALL)).max(if fused { 1. } else { 0. });
            let toughness = 0.4 + (x.scaffold() + y.scaffold()) * 0.5 + cemented * 0.6;
            let brittleness = 1. + b.stiffness * 0.4;
            let damage = DT * 0.03 * (strain - toughness / brittleness).max(0.);
            for cell in [&mut *x, &mut *y] {
                let torn = damage / (1. + cell.activity(crate::physiology::BARRIER) * 4.);
                cell.state.injury = (cell.state.injury + torn).min(1.);
                cell.state.harm[10] += torn;
            }
            let repair = (x.activity(crate::physiology::REPAIR)
                + y.activity(crate::physiology::REPAIR))
                * 0.5;
            let requested = (DT * 0.02 * repair * (1. - b.integrity))
                .min((x.energy.min(y.energy) * 100.).max(0.));
            let paid = self.spend(x, requested as f64 * 0.005, false)
                + self.spend(y, requested as f64 * 0.005, false);
            b.integrity = (b.integrity - damage + paid as f32 / 0.01).clamp(0., 1.);
            // Soft tissue gives as its cells grow: an unstiffened bond eases
            // toward the distance at which the two cells just touch. Without
            // this, growth alone strains every bond until it breaks.
            b.rest = (b.rest + (x.radius() + y.radius() - b.rest) * DT * 0.05 * (1. - b.stiffness))
                .clamp(0.1, 2.);
            let remodel =
                (x.activity(crate::organics::REMODEL) + y.activity(crate::organics::REMODEL)) * 0.5;
            let distance = x.p.sub(y.p).len();
            let change = (distance - b.rest) * DT * 0.004 * remodel;
            let required = change.abs() as f64 * 0.1;
            let available = (x.energy.min(y.energy) as f64 * 2.).min(required);
            let fraction = if required > 0. {
                available / required
            } else {
                0.
            };
            self.spend(x, available * 0.5, false);
            self.spend(y, available * 0.5, false);
            b.rest = (b.rest + change * fraction as f32).clamp(0.1, 2.);
            // Weak glue lets go within a few dozen cycles. Strong glue on both
            // sides holds for hundreds, long enough for a body to be built on.
            // Matrix holds as glue does, for as long as both cells go on
            // tending it. A cell that has stopped is letting go.
            let tended = |c: &Cell| {
                if c.expression[crate::organics::MATRIX].max(c.expression[crate::organics::FIBER]) > 0.02 {
                    c.scaffold()
                } else {
                    0.
                }
            };
            let glue = (x.activity(2) * y.activity(2)).max(tended(x) * tended(y)) as f64;
            let rate = 0.004 * (1. + strain as f64 * 8.)
                / (0.1 + glue * (1. + 8. * glue) + cemented as f64 * 6.);
            // Living tissue does not come apart by chance. In a body with a
            // plan whose young are gametes, a tie between two cells that
            // both hold on in earnest lasts until it is torn. (As at birth,
            // reading the genes for gametes stands in for a missing rule:
            // a planned animal with none still has its young by letting
            // cells go.)
            let held = glue > HELD as f64
                && (x.state.fate != 0 || y.state.fate != 0)
                && x.genome.genes.iter().any(|g| g.action == crate::development::GAMETIC);
            let survive = b.integrity > 0.02
                && strain < 2.
                && (fused || held || (core.random() as f64) >= 1. - (-rate * DT as f64).exp());
            if !survive {
                self.dissociations += 1;
                // `TORN` names each tie of a planned body as it goes, and why.
                #[cfg(test)]
                if (x.state.fate != 0 || y.state.fate != 0) && std::env::var("TORN").is_ok() {
                    eprintln!(
                        "  torn: {}/{} to {}/{} (fates {} {}), stretched to {:.2} of its length, whole {:.2}, glue {:.2}, bone {}, fins {:.2} {:.2}, on the ground {} {}, speeds {:.2} {:.2}",
                        x.state.axial, x.state.lateral, y.state.axial, y.state.lateral, x.state.fate, y.state.fate,
                        x.p.sub(y.p).len() / b.rest, b.integrity, glue, fused,
                        x.activity(FIN), y.activity(FIN), x.state.ground, y.state.ground, x.v.len(), y.v.len(),
                    );
                }
            }
            survive
        });
    }
    /// What a cell's muscle may spend: its energy above a reserve of a
    /// quarter of all it can hold. A muscle tires before it starves its own
    /// cell; without this a single hard stroke could spend a cell to death.
    fn spare(c: &Cell) -> f64 {
        (c.energy as f64 - 2.25 * c.stature()).max(0.)
    }
    fn spring_k(b: &Bond) -> f64 {
        (1. + b.stiffness as f64 * 60. + MUSCLE_PULL * b.activation as f64) * b.integrity as f64
    }
    pub(crate) fn divide(&mut self, e: &mut Evolution, core: &mut Core) {
        let n = e.cells.len();
        // Hard rays that get through thin air break what they hit, genes
        // included; so does ultraviolet, cell by cell.
        let variation = self.mutation_factor(core.time) * (1. + HARD_RAY_ERRORS * self.hard_rays() as f32);
        let focus = self.steering.focus();
        let mut daughters = Vec::new();
        let ids = e.index();
        let mut degree = crate::body::Ids::default();
        for b in &e.bonds {
            *degree.entry(b.a).or_default() += 1;
            *degree.entry(b.b).or_default() += 1;
        }
        // A planet holds only so many cells. Once it is nearly full, each
        // round starts at a different cell, so every ready cell has the same
        // chance at the places that open up and the oldest do not take them all.
        let limit = MAX_CELLS.min(core.config.capacity);
        let first = if n * 10 >= limit * 9 {
            (core.random() * n as f32) as usize % n.max(1)
        } else {
            0
        };
        for i in (0..n).map(|k| (k + first) % n) {
            if n + daughters.len() >= MAX_CELLS.min(core.config.capacity) {
                break;
            }
            let c = &e.cells[i];
            if c.lifecycle.phase == crate::development::Phase::Gamete
                || c.material.as_ref().unwrap().biomass
                    < 0.38 * c.stature() * (1. + c.activity(ENGULF)) as f64
                || (c.energy as f64) < 5.5 * c.stature()
                || c.cooldown > 0.
                // A cell in a glass shell cannot divide until the shell is
                // whole: each daughter takes half and must build the rest.
                || (c.activity(SHELL) > 0.1
                    && (c.material.as_ref().unwrap().shell() as f32) < 0.8 * c.activity(SHELL))
                // The machinery to divide outlasts the signal to. A cell
                // whose division genes are switched off now does not
                // divide, whatever it still holds: crowding stops a cell at
                // once, not a few divisions later.
                || c.expression[7] <= 0.02
                || core.random() > c.activity(7) * DT * 0.5
            {
                continue;
            }
            // A cell whose gene for gametes is driven fully makes nothing
            // else when it divides, so it waits until it can make one. (One
            // that divided before it could left an ordinary daughter in a
            // place the body has no use for: a second tail on a worm.)
            if c.expression[crate::development::GAMETIC] > 0.5
                && (c.activity(crate::development::GAMETIC) <= 0.25 || c.activity(crate::development::FUSOGEN) <= 0.1)
            {
                continue;
            }
            // Dividing costs in proportion to what is divided.
            let cost = e.cells[i].stature();
            self.spend(&mut e.cells[i], cost, false);
            let mut child = e.cells[i].clone();
            child.id = e.next_cell;
            e.next_cell += 1;
            child.parent = e.cells[i].id;
            child.generation += 1;
            child.age = 0.;
            child.cooldown = 3.;
            child.state.motor_work = 0.;
            child.state.anchored = false;
            child.state.flux = crate::observations::Flux::default();
            child.state.meals = 0;
            child.state.handling = 0.;
            child.state.prey = 0;
            child.state.neural_input = 0.;
            child.state.light = 0.;
            child.state.light_contrast = 0.;
            child.lifecycle.root = e.cells[i].body_origin();
            // Both daughters are one division further from the cell their body
            // began as. A gene that reads this can stop a body growing at a size.
            child.lifecycle.depth = child.lifecycle.depth.saturating_add(1).min(1_000_000);
            e.cells[i].lifecycle.depth = child.lifecycle.depth;
            // (Only a cell whose gene for it is at work now makes a gamete,
            // whatever it still carries: a fertilised egg holds what both its
            // gametes held, and must not shed its first daughter as another.)
            let gamete = child.expression[crate::development::GAMETIC] > 0.02
                && child.activity(crate::development::GAMETIC) > 0.25
                && child.activity(crate::development::FUSOGEN) > 0.1
                // (How often is set by how hard the gene is driven: fully
                // driven, every daughter is a gamete, and none is left in
                // the body as a part it has no place for.)
                && core.random()
                    < child.activity(crate::development::GAMETIC).max(child.expression[crate::development::GAMETIC]);
            if gamete {
                child.genome = child.gamete_chromosome(core);
                if child.lifecycle.homolog.is_some() {
                    self.development.meioses += 1;
                }
                child.lifecycle.homolog = None;
                child.lifecycle.phase = crate::development::Phase::Gamete;
                self.development.gametes += 1;
            } else {
                child.lifecycle.phase = crate::development::Phase::Somatic;
            }
            e.cells[i].lifecycle.phase = crate::development::Phase::Somatic;
            let fidelity = e.cells[i].state.germline;
            let requested = fidelity as f64 * 0.15;
            let paid = self.spend(&mut e.cells[i], requested, false);
            self.systems.fidelity_work += paid;
            let fidelity = if requested > 0. {
                fidelity * (paid / requested).min(1.) as f32
            } else {
                0.
            };
            // A body is built from one egg by copying it faithfully: its
            // cells are all but identical, and what differs between one
            // body and the next is the egg each began as. A cell dividing
            // inside a body makes a twentieth the copying errors of a cell
            // dividing alone in the sea.
            let care = if degree.get(&child.parent).copied().unwrap_or(0) > 0 { SOMATIC } else { 1. };
            let variation = (variation
                * care
                * (1. + UV_ERRORS * (child.state.ultraviolet - UV_TOLERATED).clamp(0., 4.)))
            .min(16.);
            let mut edits = child.genome.mutate_toward(core, fidelity, variation, focus);
            if edits.iter().sum::<u32>() > 0 || gamete {
                child.genome.id = e.next_genome;
                e.next_genome += 1;
            }
            if let Some(other) = &mut child.lifecycle.homolog {
                let extra = other.mutate_toward(core, fidelity, variation, focus);
                if extra.iter().sum::<u32>() > 0 {
                    if edits.iter().sum::<u32>() == 0 {
                        child.genome.id = e.next_genome;
                        e.next_genome += 1;
                    }
                    other.id = e.next_genome;
                    e.next_genome += 1;
                }
                for (a, b) in edits.iter_mut().zip(extra) {
                    *a += b;
                }
            }
            e.point_edits += edits[0];
            e.duplications += edits[1];
            e.deletions += edits[2];
            e.innovations += edits[3];
            child.material = Some(e.cells[i].material.as_mut().unwrap().split());
            child.energy = e.cells[i].energy * 0.5;
            e.cells[i].energy = child.energy;
            e.cells[i].cooldown = 3.;
            let angle = core.random() * std::f32::consts::TAU;
            let dist = child.radius() + e.cells[i].radius();
            // A three-dimensional division plane with no limb or adult template.
            let vertical = core.random() * 2. - 1.;
            let horizontal = (1. - vertical * vertical).sqrt();
            let direction = self.domain.tangent(child.p, angle);
            let up = self.domain.up(child.p);
            let random_direction = direction
                .mul(horizontal * dist)
                .add(up.mul(vertical * dist));
            let polarity = child.activity(crate::development::POLARITY);
            let turn = child
                .genome
                .genes
                .iter()
                .chain(child.lifecycle.homolog.iter().flat_map(|g| &g.genes))
                .filter(|g| g.action == crate::development::POLARITY)
                .map(|g| {
                    g.strength
                        * (g.bias * 0.15
                            + g.promoter[19] * (child.chemicals[0] - child.chemicals[1]) * 0.25)
                })
                .sum::<f32>();
            child.heading = (child.heading + turn * polarity).rem_euclid(std::f32::consts::TAU);
            // Branching by rule. A cell that has divided often enough since
            // the last fork sets this daughter off to one side and turns
            // itself to the other; both begin counting again, a fork
            // further from the root. Later forks come sooner.
            let fork = e.cells[i].expression[BRANCH];
            if fork > 0.05 {
                let order = e.cells[i].state.order;
                let wait = ((2. + 6. * (1. - fork)).round() as u8)
                    .saturating_sub(order.min(3))
                    .max(1);
                let since = if e.cells[i].state.since_fork >= wait && order < 6 {
                    let turn = FORK * std::f32::consts::TAU;
                    child.heading = (child.heading + turn).rem_euclid(std::f32::consts::TAU);
                    e.cells[i].heading =
                        (e.cells[i].heading - turn).rem_euclid(std::f32::consts::TAU);
                    child.state.order = order + 1;
                    e.cells[i].state.order = order + 1;
                    0
                } else {
                    e.cells[i].state.since_fork.saturating_add(1)
                };
                child.state.since_fork = since;
                e.cells[i].state.since_fork = since;
            }
            // A body has a middle line and two sides. A cell sets its
            // sideways daughters down in pairs, one to one side and the next
            // to the other, and what a cell on the left does a cell on the
            // right does in mirror image. So what is built on the left is
            // built on the right. Every daughter knows where it was born.
            let lateral = e.cells[i].state.lateral;
            let sideways = e.cells[i].expression[BROADEN] > 0.3;
            if sideways {
                e.cells[i].state.flip = !e.cells[i].state.flip;
            }
            // Off the middle line, always farther out on the side the cell
            // is on, so a limb grows away from the body. On it, to whichever
            // side the cell has fewer neighbours on, taking turns when they
            // are even: the two sides stay a match, and a limb that is
            // lost grows back where it was.
            let turn = if e.cells[i].state.flip { 1. } else { -1. };
            let side = if lateral != 0 {
                f32::from(lateral.signum())
            } else if sideways {
                let id = e.cells[i].id;
                let (mut left, mut right) = (0, 0);
                for b in &e.bonds {
                    let other = if b.a == id {
                        b.b
                    } else if b.b == id {
                        b.a
                    } else {
                        continue;
                    };
                    // A neighbour born this very moment is not counted yet.
                    match ids.get(&other).map(|&k| e.cells[k].state.lateral.signum()) {
                        Some(1) => left += 1,
                        Some(-1) => right += 1,
                        _ => {}
                    }
                }
                match left.cmp(&right) {
                    std::cmp::Ordering::Less => 1.,
                    std::cmp::Ordering::Greater => -1.,
                    std::cmp::Ordering::Equal => turn,
                }
            } else {
                turn
            };
            child.state.flip = false;
            if sideways {
                child.state.lateral = lateral.saturating_add(side as i8);
            } else {
                child.state.axial = e.cells[i].state.axial.saturating_add(1);
                child.state.passed = false;
                e.cells[i].state.passed = true;
            }
            let axis = child.heading
                + side * e.cells[i].expression[BROADEN].min(1.) * std::f32::consts::FRAC_PI_2;
            let mut delta = random_direction
                .mul(1. - polarity)
                .add(self.domain.tangent(child.p, axis).mul(dist * polarity))
                .normalized()
                .mul(dist);
            // The growing end of a body grows outward. A cell with a single
            // neighbour that is lengthening its body sets its daughter down
            // on the side away from that neighbour, whichever way the body
            // has turned or tumbled since it began. Without this a tip that
            // has come to face inward sets its daughter down inside the
            // line, and the line's order is lost.
            if !sideways && polarity > 0.5 {
                let id = e.cells[i].id;
                let mut tied = e.bonds.iter().filter_map(|b| {
                    if b.a == id {
                        Some(b.b)
                    } else if b.b == id {
                        Some(b.a)
                    } else {
                        None
                    }
                });
                if let (Some(only), None) = (tied.next(), tied.next()) {
                    // (In a body with a plan, that is: one whose cells behind
                    // the tip have taken up their places. A plain thread or
                    // sheet grows as it always did.)
                    if let Some(&k) = ids.get(&only).filter(|&&k| e.cells[k].state.fate != 0) {
                        let away = e.cells[i].p.sub(e.cells[k].p);
                        if away.len() > 1e-4 {
                            let away = away.normalized();
                            delta = away.mul(dist);
                            let (east, north) = (
                                self.domain.tangent(child.p, 0.),
                                self.domain.tangent(child.p, std::f32::consts::FRAC_PI_2),
                            );
                            let facing = away.dot(north).atan2(away.dot(east));
                            child.heading = facing;
                            e.cells[i].heading = facing;
                        }
                    }
                }
            }
            child.p = child.p.add(delta.mul(0.5));
            e.cells[i].p = e.cells[i].p.sub(delta.mul(0.5));
            if !self.domain.spherical() {
                for c in [&mut child, &mut e.cells[i]] {
                    c.p.x = c.p.x.clamp(-self.domain.wall(), self.domain.wall());
                    c.p.z = c.p.z.clamp(-self.domain.wall(), self.domain.wall());
                }
            }
            // A cell that is not making glue now does not hold on to the
            // daughter it has just made, whatever glue it still carries.
            let sticking = child.expression[2] > 0.02 || child.activity(WALL) > 0.05 || child.scaffold() > 0.2;
            // In a body with a plan (one whose cells have taken up their
            // places) whose young are gametes, a newborn is born inside the
            // body and stays in it, so long as it and its mother both make
            // glue in earnest. What leaves such a body is what has stopped
            // making glue, or a gamete. Elsewhere a daughter stays by chance,
            // more often the more glue the two hold: that is what sets a
            // colony's size, and it is also the only way a planned animal
            // with no gametes has young at all. (Reading the genes for
            // gametes here stands in for that: once every planned animal
            // breeds by gametes or buds, the plan alone should decide.)
            let germ = child
                .genome
                .genes
                .iter()
                .chain(child.lifecycle.homolog.iter().flat_map(|g| &g.genes))
                .any(|g| g.action == crate::development::GAMETIC);
            let planned = germ && (e.cells[i].state.fate != 0 || {
                let id = e.cells[i].id;
                e.bonds.iter().any(|b| {
                    let other = if b.a == id {
                        b.b
                    } else if b.b == id {
                        b.a
                    } else {
                        return false;
                    };
                    ids.get(&other).is_some_and(|&k| e.cells[k].state.fate != 0)
                })
            });
            let attached = !gamete
                && sticking
                && degree.get(&child.parent).copied().unwrap_or(0) < 6
                && {
                    let hold = (child.activity(2) * e.cells[i].activity(2))
                        .max(child.activity(WALL) * e.cells[i].activity(WALL));
                    core.random() < hold || (planned && hold > HELD)
                };
            if attached {
                e.bonds.push(Bond {
                    a: child.parent,
                    b: child.id,
                    rest: dist,
                    integrity: 1.,
                    activation: 0.,
                    stiffness: 0.,
                });
                e.attached_divisions += 1;
            }
            // A cell that divides along an axis splits what it is tied to
            // with its daughter: a neighbour on the daughter's side is the
            // daughter's neighbour from then on. A body built this way grows
            // longer where it would otherwise sprout a branch at every division.
            // (A cell with a place in a plan that sets a daughter down to
            // one side is budding a limb: the bud is tied to the cell it
            // grew from and to nothing else. Sharing ties by nearness there
            // handed the body's own next segment to the bud, and left the
            // cell that made it hanging off its limb.)
            if attached && polarity > 0.5 && !(sideways && e.cells[i].state.fate != 0) {
                let (front, back, newborn) = (child.p, e.cells[i].p, child.id);
                let reach = child.radius() + 0.05;
                let mut held = 1;
                let mut shared = Vec::new();
                for b in &mut e.bonds {
                    let other = if b.a == child.parent && b.b != newborn {
                        b.b
                    } else if b.b == child.parent {
                        b.a
                    } else {
                        continue;
                    };
                    let Some(&k) = ids.get(&other) else { continue };
                    let at = e.cells[k].p;
                    if held >= 6 {
                        continue;
                    }
                    if at.sub(front).len() < at.sub(back).len() {
                        (b.a, b.b) = (other, newborn);
                        held += 1;
                    } else if at.sub(front).len() < reach + e.cells[k].radius()
                        && degree.get(&other).copied().unwrap_or(0) < 6
                        // In a body with a plan, only a cell born beside the
                        // newborn's place: the one behind its mother is not.
                        && (e.cells[k].state.fate == 0
                            || (e.cells[k].state.axial as i32 - child.state.axial as i32)
                                .abs()
                                .max((e.cells[k].state.lateral as i32 - child.state.lateral as i32).abs())
                                <= 1)
                    {
                        // A neighbour to one side touches both daughters and
                        // stays cemented to both.
                        shared.push((other, at.sub(front).len()));
                        *degree.entry(other).or_default() += 1;
                        held += 1;
                    }
                }
                e.bonds.extend(shared.into_iter().map(|(other, rest)| Bond {
                    a: other,
                    b: newborn,
                    rest,
                    integrity: 1.,
                    activation: 0.,
                    stiffness: 0.,
                }));
            }
            e.divisions += 1;
            let name = child.collection_sources.first().map_or("", |s| s.as_str());
            if let Some(n) = self.births.get_mut(name) {
                *n += 1;
            } else {
                self.births.insert(name.to_string(), 1);
            }
            let (was, now) =
                (crate::steering::traits(&e.cells[i].genome.genes), crate::steering::traits(&child.genome.genes));
            let name = name.to_string();
            self.note_line(&name, was, now, core.time);
            e.peak_generation = e.peak_generation.max(child.generation);
            e.history.push(Division {
                time: core.time,
                parent: child.parent,
                child: child.id,
                genome: child.genome.id,
                generation: child.generation,
                attached,
                edits: edits.iter().sum(),
            });
            if e.history.len() > 256 {
                e.history.remove(0);
            }
            daughters.push(child);
        }
        e.cells.extend(daughters);
    }
    fn took(&mut self, eater: &Cell, eaten: &Cell, amount: f64) {
        if amount > 0. {
            let name = |c: &Cell| c.collection_sources.first().cloned().unwrap_or_default();
            *self.taken.entry((name(eaten), name(eater))).or_default() += amount;
        }
    }
    fn recycle(&mut self, e: &mut Evolution) {
        e.cells.retain(|c| {
            let m = c.material.as_ref().unwrap();
            let alive = c.energy > 0.0001 && m.biomass >= 0.02 && c.state.injury < 0.98;
            if !alive {
                let cause = if c.state.injury >= 0.98 {
                    2
                } else if c.energy <= 0.0001 {
                    0
                } else {
                    1
                };
                self.steering.losses[cause] += 1;
                // What it died of, for whoever asks: of its injuries, the
                // kind that had hurt it most.
                let by = if cause == 2 {
                    2 + (0..c.state.harm.len()).fold(0, |m, k| if c.state.harm[k] > c.state.harm[m] { k } else { m })
                } else {
                    cause
                };
                let name = c.collection_sources.first().map_or("", |s| s.as_str());
                if let Some(row) = self.deaths.get_mut(name) {
                    row[by] += 1;
                } else {
                    let mut row = [0; 13];
                    row[by] = 1;
                    self.deaths.insert(name.to_string(), row);
                }
                let site = self.domain.locate(c.p);
                self.solutes[site][0] += m.fuel;
                let detritus = m.organic() - m.fuel - m.body.dissolved_organic();
                let bound = m.bound_nitrogen();
                // Remains hold what sugar would: the rest of fat's energy goes as heat.
                self.heat += m.richness() * Q;
                let fraction = if c.state.culled || self.systems.particles.len() >= 4096 {
                    0.
                } else {
                    0.2
                };
                // Remains need a decomposer; a culled body is returned as fuel.
                let pool = if c.state.culled { 0 } else { 5 };
                self.solutes[site][pool] += detritus * (1. - fraction);
                self.solutes[site][if c.state.culled { NITROGEN } else { REMAINS_N }] +=
                    bound * (1. - fraction);
                self.solutes[site][NITROGEN] += m.nitrogen;
                // Phosphorus leaves dead tissue at once and dissolves.
                self.solutes[site][2] +=
                    m.mineral * (1. - fraction) + m.phosphorus + m.bound_phosphorus();
                if fraction > 0. {
                    self.systems
                        .particles
                        .push(crate::organics::MatterParticle {
                            p: c.p,
                            v: c.v,
                            solutes: {
                                let mut s = [0.; SOLUTES];
                                s[2] = m.mineral * fraction;
                                // A dead cell's glass stays with its remains
                                // and dissolves as they do.
                                s[SILICA] = m.silica;
                                s[5] = detritus * fraction;
                                s[REMAINS_N] = bound * fraction;
                                s
                            },
                        });
                }
                if fraction == 0. {
                    self.solutes[site][SILICA] += m.silica;
                }
                for k in 0..SOLUTES {
                    self.solutes[site][k] += m.body.solutes[k];
                }
                if !self.systems.gas.is_empty() {
                    for k in 0..2 {
                        self.systems.gas[site][k] += m.body.gas[k];
                    }
                }
                if let Some(p) = &mut self.planet {
                    p.water[site] += m.body.water;
                    p.biological_water -= m.body.water;
                }
                self.heat += c.energy as f64;
                e.deaths += 1;
            }
            alive
        });
        let alive: BTreeSet<_> = e.cells.iter().map(|c| c.id).collect();
        e.bonds
            .retain(|b| alive.contains(&b.a) && alive.contains(&b.b));
    }
    pub fn tick(&mut self, e: &mut Evolution, core: &mut Core) {
        core.time += DT;
        let mut planet = self
            .planet
            .take()
            .unwrap_or_else(|| crate::planet::Planet::new_in(self.domain));
        // Carbon dioxide in the air holds warmth in. Each doubling adds the
        // same again, so a planet whose life buries its carbon cools, and
        // one whose rock or fires give it back warms. A dish in a
        // laboratory is kept at the warmth it was set to.
        if self.domain.spherical() {
            let surface: f64 = (0..self.domain.sites()).map(|i| self.domain.area(i)).sum();
            let column = self.air_carbon / (surface / (DX * DX));
            planet.greenhouse =
                (GREENHOUSE * (column / GREENHOUSE_EVEN).max(1e-3).ln()).clamp(-0.2, 0.2) as f32;
        }
        planet.tick(core, &mut self.solutes);
        self.bombard(e, core, &mut planet);
        if let Some(flow) = &mut self.fluid.surface {
            flow.step(
                self.domain,
                &mut planet,
                &mut self.solutes,
                core.config.seed,
            );
        }
        self.planet = Some(planet);
        if self.initial_nitrogen <= 0. {
            self.seed_nitrogen(&mut e.cells);
        }
        if self.chemistry != CHEMISTRY {
            if self.chemistry < 2 {
                self.seed_elements(false);
            }
            if self.chemistry < 4 {
                self.seed_silica();
            }
            if self.chemistry < 5 {
                if let Some(p) = &mut self.planet {
                    p.seed_vents(core.config.seed);
                }
            }
            if self.chemistry < 3 {
                for m in e.cells.iter_mut().filter_map(|c| c.material.as_mut()) {
                    m.phosphorus = m.phosphorus.max(m.bound_phosphorus() * 0.5);
                }
            }
            if self.chemistry < 6 {
                // Until now every cell breathed without being told how. So
                // that an old world lives on as it did, each of its cells is
                // given the gene, and each mouth and each outsized cell a
                // breather inside it. The machinery they build themselves.
                let gene = crate::autonomous::Gene {
                    action: RESPIRE,
                    promoter: [0.; INPUTS],
                    bias: 0.7,
                    strength: 0.55,
                    fates: 15,
                };
                for c in &mut e.cells {
                    let complex = c.activity(ENGULF) > 0.25 || c.state.stature > 1.5;
                    let genomes = std::iter::once(&mut c.genome).chain(c.lifecycle.homolog.as_mut());
                    let mut given = true;
                    for g in genomes {
                        if g.genes.iter().any(|g| g.action == RESPIRE) {
                            continue;
                        }
                        if g.genes.len() < crate::autonomous::MAX_GENES {
                            g.genes.push(gene.clone());
                        } else {
                            given = false;
                        }
                    }
                    if complex || !given {
                        c.state.mitochondria = 1.;
                    }
                }
            }
            self.chemistry = CHEMISTRY;
            self.rebase(&e.cells);
            self.initial_mineral -= self.planet.as_ref().map_or(0., |p| p.mineral_input);
            self.initial_energy +=
                self.heat + self.mechanical_input - self.photon_energy - self.geothermal_energy;
        }
        self.fluid.step();
        self.transport(DT as f64);
        self.planet_chemistry(DT as f64);
        if self.ablation != "transport" {
            self.tissue_transport(e);
        }
        self.body_chemistry(e, core);
        if self.ablation != "transport" {
            self.internal_circulation(e);
        }
        self.particle_ecology(e, core);
        self.photoreception(e);
        self.physiology(e, core);
        if self.ablation != "signals" {
            self.neural_contacts(e);
            self.excitable_signals(e);
        } else {
            for c in &mut e.cells {
                c.state.neural_input = 0.;
                c.state.excitation = 0.;
                c.state.recovery = 0.;
            }
        }
        let pairs = self.contacts(e, core);
        self.mechanics(e, core, &pairs);
        self.land(e);
        self.fertilize(e, core, &pairs);
        self.divide(e, core);
        self.steer(e, core);
        self.recycle(e);
        self.web.settle(DT as f64);
        if let Some(p) = &mut self.planet {
            p.biological_water = e
                .cells
                .iter()
                .map(|c| c.material.as_ref().unwrap().body.water)
                .sum();
        }
        self.prune_synapses(e);
        self.sync_joints(e); // remove broken contacts/dead endpoints before save/display
        let ids = e.index();
        for j in &mut self.joints {
            let (pa, pb, pc) = (e.cells[ids[&j.a]].p, e.cells[ids[&j.b]].p, e.cells[ids[&j.c]].p);
            j.angle = crate::articulation::angle_and_gradients(pa, pb, pc).0;
            if j.straight {
                j.bend = crate::articulation::bend_and_gradients(pa, pb, pc, self.domain.up(pb)).0;
            }
        }
        // The physical export reads this habitat directly.
        for i in 0..GRID * GRID {
            let x = (i % GRID) as f32 * 1.6 - 31.2;
            let z = (i / GRID) as f32 * 1.6 - 31.2;
            core.field[i] = self.solutes[self.domain.index(x, z)][0] as f32;
        }
        e.peak_body = e
            .peak_body
            .max(e.groups().iter().map(Vec::len).max().unwrap_or(0));
        core.births = e.divisions;
        core.deaths = e.deaths;
        core.mating = self.development.zygotes;
        core.feedings = e.meals;
        if core.time - core.history_clock >= 3. {
            let groups = e.groups();
            core.history.push(History {
                time: core.time,
                population: groups.len(),
                cells: e.cells.len() as u32,
                colonies: groups.iter().filter(|g| g.len() > 1).count(),
                diversity: e
                    .cells
                    .iter()
                    .map(|c| c.genome.id)
                    .collect::<BTreeSet<_>>()
                    .len(),
            });
            if core.history.len() > 240 {
                core.history.remove(0);
            }
            core.history_clock = core.time;
        }
    }
    pub fn view(&self, cells: &[Cell]) -> serde_json::Value {
        let (organic, mineral, energy) = self.totals(cells);
        let errors = self.errors(cells);
        let speed = (0..self.domain.sites())
            .map(|i| {
                let u = self.fluid.velocity(i);
                (u[0] * u[0] + u[1] * u[1]).sqrt()
            })
            .fold(0., f64::max);
        serde_json::json!({"domain":{"width":self.domain.size(),"grid":self.domain.grid,"rows":self.domain.rows(),"sites":self.domain.sites(),"spacing":self.domain.spacing(),"radius":if self.domain.spherical(){self.domain.radius()}else{0.},"shape":self.domain.shape,"area":(0..self.domain.sites()).map(|i|self.domain.area(i)).sum::<f64>(),"boundary":if self.domain.spherical(){"closed sphere"}else{"closed planar"}},"worldFlow":self.fluid.surface.as_ref().map(|f|(0..self.domain.sites()).step_by((self.domain.sites()/400).max(1)).map(|i|{let(x,z)=self.domain.point(i);let p=self.domain.position(x,z,0.12);let v=f.velocities[i];[p.x,p.y,p.z,v.x,v.y,v.z]}).collect::<Vec<_>>()),"assay":if self.passive {"passive"}else{"full"},"joints":self.joints.len(),"angularWork":self.joints.iter().map(|j|j.work).sum::<f64>(),"organic":organic,"mineral":mineral,"chemicalEnergy":energy,"photonEnergy":self.photon_energy,"heat":self.heat,
            "mechanicalInput":self.mechanical_input,"massError":errors[0],"mineralError":errors[1],"energyError":errors[2],
            "fluidSpeed":speed,"flow":(0..self.domain.sites()).step_by(4).map(|i|{let u=self.fluid.velocity(i);[i as f64,u[0],u[1]]}).collect::<Vec<_>>(),"boundEvents":self.bound_events,"dissociations":self.dissociations,
            "proteinMass":cells.iter().map(|c|c.material.as_ref().unwrap().proteins.iter().sum::<f64>()).sum::<f64>(),
            "boundMineral":cells.iter().map(|c|c.material.as_ref().unwrap().mineral).sum::<f64>()})
    }
}

impl Cell {
    /// How large this cell grows before it divides, beside the usual size.
    pub fn stature(&self) -> f64 {
        if self.state.stature > 0. {
            self.state.stature as f64
        } else {
            1.
        }
    }
    /// Its surface beside that of a cell of the usual size.
    pub fn area(&self) -> f64 {
        self.stature().powf(2. / 3.) * self.outline()
    }
    /// The surface of a rod beside that of a ball holding as much: a cell
    /// four times as long as wide has a quarter more membrane to feed through.
    pub fn outline(&self) -> f64 {
        let a = self.state.aspect as f64;
        if a > 1. {
            a * (1. + 1.5 * (a - 1.)).powf(-2. / 3.)
        } else {
            1.
        }
    }
    /// Its metabolism beside that of a cell of the usual size.
    pub fn pace(&self) -> f64 {
        self.stature().powf(0.75)
    }
    /// The weight this cell can carry before it is crushed. Bare soft tissue
    /// carries a few times its own weight; bone, a wall, laid-down fibre and
    /// water held under pressure each carry far more.
    pub fn strength(&self) -> f64 {
        let m = self.material.as_ref().unwrap();
        GRAVITY
            * m.mass()
            * SOFT_STRENGTH
            * (1.
                + 10. * m.bone()
                + 4. * self.activity(WALL) as f64
                + 4. * m.fibre()
                + 0.5 * (self.state.pressure as f64).min(2.))
    }
    pub fn radius(&self) -> f32 {
        self.material.as_ref().map_or(0.22, Inventory::radius)
    }
    /// How firmly this cell holds the angle between its neighbours: by the
    /// mineral in it, as bone does, or by a wall, as a plant's cells do.
    pub fn rigidity(&self) -> f32 {
        let m = self.material.as_ref().unwrap();
        // Against the living tissue the bone is laid in, not the water, fuel
        // or fat the cell is carrying: none of those softens bone.
        (m.mineral / (m.biomass + m.mineral).max(1e-9)) as f32 + WALL_STIFFNESS * self.activity(WALL)
    }
    pub fn activity(&self, k: usize) -> f32 {
        self.material.as_ref().map_or(self.expression[k], |m| {
            m.activity(k) * (1. - self.state.injury * 0.75)
        })
    }
    /// How able a cell is to take in living prey. A cell given over to
    /// harvesting light is a plant: it feeds itself and cannot swallow or
    /// digest its neighbours. To eat, a cell must give the light up, and
    /// that holds whether the machinery is its own or a captive's: a hunter
    /// that keeps a light-feeder inside it has become an alga.
    pub fn bite(&self) -> f32 {
        (1. - self.activity(0).max(self.state.symbiont)).powi(2)
    }
}
impl Core {
    pub fn start_physical(&mut self) {
        self.start_autonomous();
        self.config.name = "Physical cellular habitat".into();
        let mut e = self.autonomous.take().unwrap();
        e.physical = Some(Habitat::new(self, &mut e.cells));
        self.autonomous = Some(e);
        self.event("world","Physical cellular habitat: finite conserved chemical pools, diffusion and fluid transport. Genes regulate material synthesis; bonds and contraction follow material stores. Chemistry is a reduced model, not molecular abiogenesis.".into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn joint_fixture(passive: bool) -> (Core, Evolution, Habitat) {
        let mut core = Core::new(Config {
            initial_cells: 3,
            physical_assay: if passive {
                "passive".into()
            } else {
                "full".into()
            },
            ..Config::default()
        })
        .unwrap();
        let mut e = Evolution::new(&mut core);
        let mut h = Habitat::new(&core, &mut e.cells);
        for (c, p) in e.cells.iter_mut().zip([
            V3::new(0., 5., 0.),
            V3::new(0.5, 5., 0.),
            V3::new(0.5, 5., 0.5),
        ]) {
            c.p = p;
            c.material.as_mut().unwrap().mineral = 0.08;
            c.material.as_mut().unwrap().proteins[4] = 0.03;
            c.chemicals = [1., 0.];
        }
        for (a, b) in [(1, 2), (2, 3)] {
            e.bonds.push(Bond {
                a,
                b,
                rest: 0.5,
                integrity: 1.,
                activation: 0.,
                stiffness: 0.,
            });
        }
        h.rebase(&e.cells);
        (core, e, h)
    }
    #[test]
    fn material_joints_require_deposition_and_intact_contacts() {
        let (_, mut e, mut h) = joint_fixture(false);
        h.sync_joints(&e);
        assert_eq!(h.joints.len(), 1);
        let rest = h.joints[0].rest;
        e.cells[2].p.z += 0.1;
        h.sync_joints(&e);
        assert_eq!(h.joints[0].rest, rest);
        e.bonds.pop();
        h.sync_joints(&e);
        assert!(h.joints.is_empty());
        let (_, mut e, mut h) = joint_fixture(false);
        e.cells[1].material.as_mut().unwrap().mineral = 0.;
        h.sync_joints(&e);
        assert!(h.joints.is_empty());
    }
    #[test]
    fn angular_actuation_is_paid_passive_and_atp_gated() {
        for (passive, energy, active) in [(false, 5., true), (true, 5., false), (false, 0., false)]
        {
            let (_, mut e, mut h) = joint_fixture(passive);
            for c in &mut e.cells {
                c.energy = energy;
            }
            h.rebase(&e.cells);
            h.sync_joints(&e);
            h.prepare_joint_motors(&mut e);
            assert_eq!(h.joints[0].work > 0., active);
            assert_eq!(h.joints[0].flexion > 0., active);
            assert!(h.errors(&e.cells).iter().all(|v| v.abs() < 1e-9));
            if active {
                assert!(h.joints[0].target < h.joints[0].rest);
            }
        }
    }
    #[test]
    fn absent_contractile_material_never_actuates_a_sharp_rest_angle() {
        let (_, mut e, mut h) = joint_fixture(false);
        e.cells[2].p = V3::new(0., 5., 0.0001);
        for c in &mut e.cells {
            c.material.as_mut().unwrap().proteins[4] = 0.;
        }
        h.sync_joints(&e);
        let rest = h.joints[0].rest;
        h.prepare_joint_motors(&mut e);
        assert_eq!(h.joints[0].target, rest);
        assert_eq!(h.joints[0].work, 0.);
    }
    #[test]
    fn angular_internal_forces_have_no_net_force_or_torque() {
        let (_, e, mut h) = joint_fixture(false);
        h.sync_joints(&e);
        let ps: [V3; 3] = [e.cells[0].p, e.cells[1].p, e.cells[2].p];
        let (_, grad) = crate::articulation::angle_and_gradients(ps[0], ps[1], ps[2]);
        let sum = grad.into_iter().fold(V3::default(), |sum, g| sum.add(g));
        assert!(sum.len() < 1e-5);
        let cross = |a: V3, b: V3| {
            V3::new(
                a.y * b.z - a.z * b.y,
                a.z * b.x - a.x * b.z,
                a.x * b.y - a.y * b.x,
            )
        };
        let torque = ps
            .into_iter()
            .zip(grad)
            .fold(V3::default(), |sum, (p, g)| sum.add(cross(p, g)));
        assert!(torque.len() < 1e-5);
    }
    #[test]
    fn deposited_joint_world_resumes_and_rejects_invalid_joint_contacts() {
        let (mut core, mut e, h) = joint_fixture(false);
        e.physical = Some(h);
        core.organisms.clear();
        core.autonomous = Some(e);
        core.step(1);
        assert!(core.autonomous.as_ref().unwrap().valid());
        let mut copy = Core::load(&serde_json::to_string(&core).unwrap()).unwrap();
        core.step(3);
        copy.step(3);
        assert_eq!(
            serde_json::to_string(&core).unwrap(),
            serde_json::to_string(&copy).unwrap()
        );
        let mut bad: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&core).unwrap()).unwrap();
        if !bad["autonomous"]["physical"]["joints"]
            .as_array()
            .unwrap()
            .is_empty()
        {
            bad["autonomous"]["physical"]["joints"][0]["c"] = 99999.into();
            assert!(Core::load(&bad.to_string()).is_err());
        }
    }
    #[test]
    fn configured_population_and_fission_exceed_the_old_1024_ceiling() {
        let mut core = Core::new(Config {
            initial_cells: 1100,
            capacity: 1200,
            ..Config::default()
        })
        .unwrap();
        let mut e = Evolution::new(&mut core);
        let mut h = Habitat::new(&core, &mut e.cells);
        assert_eq!(e.cells.len(), 1100);
        assert_eq!(e.next_cell, 1101);
        for c in &mut e.cells {
            let m = c.material.as_mut().unwrap();
            m.biomass = 0.4;
            m.proteins[7] = 0.1;
            c.energy = 6.;
            c.cooldown = 0.;
        }
        h.rebase(&e.cells);
        h.divide(&mut e, &mut core);
        assert_eq!(e.cells.len(), 1200);
        assert!(e.cells.iter().any(|c| c.generation == 1));
        let count = e.cells.len();
        h.divide(&mut e, &mut core);
        assert_eq!(count, e.cells.len());
        assert!(h.errors(&e.cells).iter().all(|v| v.abs() < 1e-7));
        e.physical = Some(h);
        assert!(e.valid());
        core.organisms.clear();
        core.autonomous = Some(e);
        let restored = Core::load(&serde_json::to_string(&core).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_string(&core).unwrap(),
            serde_json::to_string(&restored).unwrap()
        );
    }
    #[test]
    fn setup_bounds_allow_64000_cells_and_keep_old_save_defaults() {
        assert!(Config {
            capacity: 64000,
            initial_cells: 8192,
            ..Config::default()
        }
        .validate()
        .is_ok());
        assert!(Config {
            capacity: 64001,
            ..Config::default()
        }
        .validate()
        .is_err());
        assert!(Config {
            initial_cells: 8193,
            ..Config::default()
        }
        .validate()
        .is_err());
        let mut old = serde_json::to_value(Config::default()).unwrap();
        old.as_object_mut().unwrap().remove("initialCells");
        old.as_object_mut().unwrap().remove("physicalAssay");
        let c: Config = serde_json::from_value(old).unwrap();
        assert_eq!(c.initial_cells, 64);
        assert_eq!(c.physical_assay, "full");
    }
    #[test]
    fn larger_world_ticks_conserve_material_and_stay_finite() {
        let mut core = Core::new(Config {
            initial_cells: 1536,
            capacity: 8192,
            ..Config::default()
        })
        .unwrap();
        core.start_physical();
        core.step(4);
        let e = core.autonomous.as_ref().unwrap();
        assert!(e.valid());
        assert!(e
            .physical
            .as_ref()
            .unwrap()
            .errors(&e.cells)
            .iter()
            .all(|v| v.abs() < 1e-7));
    }
    #[test]
    fn unforced_vortex_dissipates_and_stays_in_low_mach_domain() {
        let mut f = Fluid::new();
        let energy = |fluid: &Fluid| -> f64 {
            (0..GRID * GRID)
                .map(|i| {
                    let u = fluid.lattice_velocity(i);
                    u[0] * u[0] + u[1] * u[1]
                })
                .sum()
        };
        let before = energy(&f);
        let mass = f.populations.iter().flatten().sum::<f64>();
        for _ in 0..160 {
            f.step();
            assert!(f.valid());
        }
        assert!(energy(&f) < before * 0.8);
        assert!((f.populations.iter().flatten().sum::<f64>() - mass).abs() < 1e-9);
    }
    #[test]
    fn mineral_binding_requires_both_catalyst_material_and_local_mineral() {
        let mut core = Core::new(Config::default()).unwrap();
        core.start_physical();
        let mut e = core.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        e.cells.truncate(1);
        e.cells[0].genome.genes.push(autonomous::Gene {
            action: 3,
            promoter: [0.; crate::autonomous::INPUTS],
            bias: 1.,
            strength: 1.,
            fates: 15,
        });
        let site = field_index(e.cells[0].p.x, e.cells[0].p.z);
        h.solutes[site][2] = 0.;
        for _ in 0..20 {
            h.physiology(&mut e, &core);
        }
        assert!(e.cells[0].material.as_ref().unwrap().proteins[3] > 0.);
        assert_eq!(e.cells[0].material.as_ref().unwrap().mineral, 0.);
        h.solutes[site][2] = 0.5;
        // Everything the cell holds that came out of rock: bone, and the
        // phosphorus loose in it and built into its tissue.
        let before = h.solutes[site][2] + e.cells[0].material.as_ref().unwrap().mineral_total();
        h.physiology(&mut e, &core);
        let m = e.cells[0].material.as_ref().unwrap();
        assert!(m.mineral > 0.);
        assert!((h.solutes[site][2] + m.mineral_total() - before).abs() < 1e-12);
    }
    #[test]
    fn material_contraction_is_paid_and_empty_atp_prevents_activation() {
        let mut core = Core::new(Config::default()).unwrap();
        core.start_physical();
        let mut e = core.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        e.cells.truncate(2);
        h.fluid = Fluid::still();
        for (i, c) in e.cells.iter_mut().enumerate() {
            c.p = V3::new(i as f32 * 0.6, 10., 0.);
            c.v = V3::default();
            c.chemicals = [1., 0.];
            c.material.as_mut().unwrap().proteins[4] = 0.05;
        }
        e.bonds.push(Bond {
            a: 1,
            b: 2,
            rest: 0.6,
            integrity: 1.,
            activation: 0.,
            stiffness: 0.,
        });
        let mut empty = e.clone();
        for c in &mut empty.cells {
            c.energy = 0.;
        }
        let before = e.cells.iter().map(|c| c.energy as f64).sum::<f64>();
        h.mechanics(&mut e, &mut core, &[]);
        assert!(e.bonds[0].activation > 0. && e.motor_work > 0.);
        let after = e.cells.iter().map(|c| c.energy as f64).sum::<f64>();
        assert!((before - after - e.motor_work).abs() < 1e-12);
        h.mechanics(&mut empty, &mut core, &[]);
        assert_eq!(empty.bonds[0].activation, 0.);
        assert_eq!(empty.motor_work, 0.);
    }
    #[test]
    fn closed_fluid_preserves_mass_and_rest_equilibrium() {
        let mut f = Fluid::still();
        let before = f.populations.iter().flatten().sum::<f64>();
        for _ in 0..80 {
            f.step();
        }
        assert!((f.populations.iter().flatten().sum::<f64>() - before).abs() < 1e-9);
        assert!(f.momentum().iter().all(|v| v.abs() < 1e-9));
        assert!(f.valid());
    }
    #[test]
    fn diffusion_spreads_a_pulse_without_loss_or_negative_concentrations() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_physical();
        let e = c.autonomous.as_mut().unwrap();
        let h = e.physical.as_mut().unwrap();
        h.fluid = Fluid::still();
        h.solutes = vec![[0.; SOLUTES]; GRID * GRID];
        let site = 20 * GRID + 20;
        h.solutes[site][0] = 1.;
        for _ in 0..100 {
            h.transport(DT as f64);
        }
        assert!((h.solutes.iter().map(|s| s[0]).sum::<f64>() - 1.).abs() < 1e-12);
        assert!(h.solutes[site][0] < 0.5 && h.solutes[site + 1][0] > 0.);
        assert!(h.solutes.iter().flatten().all(|v| *v >= 0.));
    }
    #[test]
    fn drag_transfers_equal_opposite_momentum_to_solvent() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_physical();
        let mut e = c.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        h.fluid = Fluid::still();
        let cell = &mut e.cells[0];
        cell.v = V3::new(0.2, 0., -0.1);
        let mass = cell.material.as_ref().unwrap().mass();
        let before = [mass * cell.v.x as f64, mass * cell.v.z as f64];
        h.drag(cell, 0.01);
        let solvent = h.fluid.momentum();
        assert!((mass * cell.v.x as f64 + solvent[0] - before[0]).abs() < 1e-12);
        assert!((mass * cell.v.z as f64 + solvent[1] - before[1]).abs() < 1e-12);
        assert!(cell.v.x < 0.2 && cell.v.z.abs() < 0.1);
    }
    #[test]
    fn expression_requires_material_production_before_mechanical_activity() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_physical();
        let mut e = c.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        let cell = &mut e.cells[0];
        cell.genome.genes.push(autonomous::Gene {
            action: 3,
            promoter: [0.; crate::autonomous::INPUTS],
            bias: 1.,
            strength: 1.,
            fates: 15,
        });
        assert_eq!(cell.activity(3), 0.);
        h.physiology(&mut e, &c);
        assert!(e.cells[0].material.as_ref().unwrap().proteins[3] > 0.);
        assert!(h.errors(&e.cells).iter().all(|v| v.abs() < 1e-8));
    }
    #[test]
    fn fission_splits_existing_material_and_copies_with_mutation() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_physical();
        let mut e = c.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        for cell in &mut e.cells {
            cell.material.as_mut().unwrap().biomass = 0.4;
            cell.energy = 8.;
            cell.cooldown = 0.;
        }
        let before = h.totals(&e.cells);
        for _ in 0..100 {
            h.divide(&mut e, &mut c);
        }
        let after = h.totals(&e.cells);
        assert!(e.divisions > 0);
        assert!((after.0 - before.0).abs() < 1e-10);
        assert!((after.1 - before.1).abs() < 1e-10);
        assert!((after.2 + h.heat - before.2).abs() < 1e-9);
    }
    #[test]
    fn coupled_world_conserves_matter_energy_and_resumes_exactly() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_physical();
        c.step(120);
        let e = c.autonomous.as_ref().unwrap();
        let h = e.physical.as_ref().unwrap();
        assert!(h.valid(&e.cells), "{:?}", h.errors(&e.cells));
        let save = serde_json::to_string(&c).unwrap();
        let mut other = Core::load(&save).unwrap();
        c.step(8);
        other.step(8);
        assert_eq!(
            serde_json::to_string(&c).unwrap(),
            serde_json::to_string(&other).unwrap()
        );
    }
    #[test]
    fn corrupted_chemical_store_is_rejected() {
        let mut c = Core::new(Config::default()).unwrap();
        c.start_physical();
        c.autonomous
            .as_mut()
            .unwrap()
            .physical
            .as_mut()
            .unwrap()
            .solutes[0][0] += 1.;
        assert!(Core::load(&serde_json::to_string(&c).unwrap()).is_err());
    }

    #[test]
    fn storage_repair_and_fission_use_existing_material() {
        let (mut core, mut e, mut h) = joint_fixture(false);
        e.bonds.clear();
        h.joints.clear();
        for c in &mut e.cells {
            c.energy = 8.;
            let m = c.material.as_mut().unwrap();
            m.fuel = 0.4;
            m.proteins[crate::physiology::LIPID] = 0.04;
            m.proteins[crate::physiology::REPAIR] = 0.04;
        }
        h.rebase(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        h.reserves_and_repair(&mut e.cells[0]);
        assert!(e.cells[0].material.as_ref().unwrap().lipid > 0.);
        e.cells[0].state.injury = 0.4;
        let biomass = e.cells[0].material.as_ref().unwrap().biomass;
        h.reserves_and_repair(&mut e.cells[0]);
        assert!(
            e.cells[0].state.injury < 0.4
                && e.cells[0].material.as_ref().unwrap().biomass > biomass
        );
        // Moving ATP out is an explicit sink in this controlled starvation test.
        h.spend(&mut e.cells[0], 6., false);
        let lipid = e.cells[0].material.as_ref().unwrap().lipid;
        h.reserves_and_repair(&mut e.cells[0]);
        assert!(e.cells[0].material.as_ref().unwrap().lipid < lipid);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
        e.cells[0].material.as_mut().unwrap().biomass = 0.4;
        e.cells[0].material.as_mut().unwrap().lipid = 0.08;
        e.cells[0].material.as_mut().unwrap().proteins[7] = 0.1;
        e.cells[0].energy = 8.;
        e.cells[0].cooldown = 0.;
        for c in &mut e.cells[1..] {
            c.cooldown = 100.;
        }
        h.rebase(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        for _ in 0..100 {
            h.divide(&mut e, &mut core);
            if e.cells.len() > 3 {
                break;
            }
        }
        assert_eq!(e.cells.len(), 4);
        assert_eq!(e.cells[0].material.as_ref().unwrap().lipid, 0.04);
        assert_eq!(e.cells[3].material.as_ref().unwrap().lipid, 0.04);
        assert_eq!(e.cells[0].state.excitation, e.cells[3].state.excitation);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
    }
    #[test]
    fn attached_carriers_share_fuel_and_atp_without_creating_energy() {
        let (_, mut e, mut h) = joint_fixture(false);
        for c in &mut e.cells {
            c.material.as_mut().unwrap().proteins[8] = 0.04;
        }
        e.cells[0].material.as_mut().unwrap().fuel = 0.25;
        e.cells[1].material.as_mut().unwrap().fuel = 0.;
        e.cells[0].energy = 8.;
        e.cells[1].energy = 1.;
        h.rebase(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        h.tissue_transport(&mut e);
        assert!(e.cells[1].energy > 1. && e.cells[1].material.as_ref().unwrap().fuel > 0.);
        assert!(h.physiology.energy_transported > 0. && h.physiology.fuel_transported > 0.);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
        e.bonds.clear();
        let before = serde_json::to_string(&e.cells).unwrap();
        h.tissue_transport(&mut e);
        assert_eq!(before, serde_json::to_string(&e.cells).unwrap());
    }
    #[test]
    fn excitable_contact_network_is_material_paid_and_drives_real_motion() {
        let (mut core, mut e, mut h) = joint_fixture(false);
        for c in &mut e.cells {
            c.chemicals = [0.; 2];
            c.material.as_mut().unwrap().proteins[12] = 0.04;
            c.material.as_mut().unwrap().proteins[8] = 0.04;
        }
        e.cells[0].state.excitation = 0.9;
        h.rebase(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        let mut detached = e.clone();
        detached.bonds.clear();
        // The neighbour fires and falls quiet again within these moments.
        let mut fired = 0f32;
        for _ in 0..12 {
            h.excitable_signals(&mut e);
            fired = fired.max(e.cells[1].state.excitation);
        }
        assert!(fired > 0.);
        let mut detached_h = h.clone();
        detached_h.excitable_signals(&mut detached);
        assert_eq!(detached.cells[1].state.excitation, 0.);
        assert!(h.physiology.signal_work > 0.);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
        // Caught in the moment of firing, the muscle between them pulls.
        for c in &mut e.cells {
            c.state.excitation = 0.9;
        }
        let mut passive = e.clone();
        let mut passive_h = h.clone();
        passive_h.passive = true;
        let before: Vec<_> = e.cells.iter().map(|c| c.p).collect();
        h.mechanics(&mut e, &mut core, &[]);
        passive_h.mechanics(&mut passive, &mut core, &[]);
        assert!(e.motor_work > 0. && passive.motor_work == 0.);
        assert!(e
            .cells
            .iter()
            .zip(&passive.cells)
            .any(|(a, b)| a.p.sub(b.p).len() > 1e-6));
        assert!(e
            .cells
            .iter()
            .zip(before)
            .any(|(a, p)| a.p.sub(p).len() > 1e-6));
        for c in &mut detached.cells {
            c.state.excitation = 0.;
            c.material.as_mut().unwrap().proteins[12] = 0.;
            c.state.food_change = 1.;
        }
        detached_h.excitable_signals(&mut detached);
        assert!(detached.cells.iter().all(|c| c.state.excitation == 0.));
        detached.cells[0].state.excitation = 1.;
        assert_eq!(
            detached.cells[0].motor_signals(),
            [0.; 2],
            "Lost channel material cannot leave a free motor-control latch"
        );
    }
    #[test]
    fn digestion_wounds_defense_limits_it_and_excretion_feeds_decomposers() {
        let (mut core, mut e, mut h) = joint_fixture(false);
        e.bonds.clear();
        for (i, c) in e.cells.iter_mut().enumerate() {
            c.p = V3::new(i as f32 * 0.3, 5., 0.);
            c.material.as_mut().unwrap().fuel = 0.;
            c.material.as_mut().unwrap().proteins[9] = 0.;
        }
        e.cells[0].material.as_mut().unwrap().proteins[9] = 0.04;
        let mut defended = e.clone();
        defended.cells[1].material.as_mut().unwrap().proteins[14] = 0.08;
        h.rebase(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        let mut shield_h = h.clone();
        h.contacts(&mut e, &mut core);
        shield_h.contacts(&mut defended, &mut core);
        assert!(
            defended.cells[1].material.as_ref().unwrap().biomass
                > e.cells[1].material.as_ref().unwrap().biomass
        );
        assert!(e.cells[1].state.injury > 0. && h.physiology.excreted > 0.);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
        let site = field_index(e.cells[2].p.x, e.cells[2].p.z);
        h.solutes[site][0] = 0.;
        h.solutes[site][1] = 0.;
        h.solutes[site][5] = 0.2;
        let mut control = e.clone();
        let mut control_h = h.clone();
        e.cells[2].material.as_mut().unwrap().proteins[15] = 0.04;
        h.rebase(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        h.physiology(&mut e, &core);
        control_h.physiology(&mut control, &core);
        assert!(h.physiology.decomposed > 0. && control_h.physiology.decomposed == 0.);
        assert!(h.solutes[site][5] < control_h.solutes[site][5]);
        assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-8));
    }
    #[test]
    fn regions_are_observations_not_supplied_organs_and_new_state_resumes() {
        let (mut core, mut e, mut h) = joint_fixture(false);
        for c in &mut e.cells {
            let m = c.material.as_mut().unwrap();
            m.lipid = 0.05;
            m.proteins[12] = 0.04;
            m.proteins[8] = 0.04;
        }
        e.cells[0].state.excitation = 0.9;
        let view = h.physiology_view(&e);
        assert_eq!(view["mixedBodies"], 1);
        assert!(view["regions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["regions"] == 1));
        let mut detached = e.clone();
        detached.bonds.clear();
        assert_eq!(h.physiology_view(&detached)["mixedBodies"], 0);
        h.rebase(&e.cells);
        h.initial_energy += h.heat + h.mechanical_input - h.photon_energy; // rebase this controlled fixture including prior sinks
        h.planet.as_mut().unwrap().storm = 4.;
        e.physical = Some(h);
        core.organisms.clear();
        core.autonomous = Some(e);
        core.step(4);
        let saved = serde_json::to_string(&core).unwrap();
        let mut copy = Core::load(&saved).unwrap();
        core.step(6);
        copy.step(6);
        assert_eq!(
            serde_json::to_string(&core).unwrap(),
            serde_json::to_string(&copy).unwrap()
        );
        let mut bad = serde_json::to_value(&core).unwrap();
        bad["autonomous"]["cells"][0]["state"]["excitation"] = 2.into();
        assert!(Core::load(&bad.to_string()).is_err());
    }
    #[test]
    fn old_vocabularies_migrate_without_inventing_specialized_genes_or_matter() {
        let mut core = Core::new(Config::default()).unwrap();
        core.start_physical();
        let mut value = serde_json::to_value(&core).unwrap();
        let e = &mut value["autonomous"];
        for c in e["cells"].as_array_mut().unwrap() {
            c.as_object_mut().unwrap().remove("state");
            c["material"].as_object_mut().unwrap().remove("lipid");
            c["material"].as_object_mut().unwrap().remove("nitrogen");
            c["expression"].as_array_mut().unwrap().truncate(10);
            c["material"]["proteins"]
                .as_array_mut()
                .unwrap()
                .truncate(10);
            for g in c["genome"]["genes"].as_array_mut().unwrap() {
                g["promoter"].as_array_mut().unwrap().truncate(8);
            }
        }
        e["physical"].as_object_mut().unwrap().remove("planet");
        e["physical"].as_object_mut().unwrap().remove("physiology");
        for key in ["initial_nitrogen", "air_nitrogen", "web", "chemistry"] {
            assert!(e["physical"].as_object_mut().unwrap().remove(key).is_some());
        }
        for s in e["physical"]["solutes"].as_array_mut().unwrap() {
            s.as_array_mut().unwrap().truncate(5);
        }
        let mut migrated = Core::load(&value.to_string()).unwrap();
        assert!(migrated
            .autonomous
            .as_ref()
            .unwrap()
            .cells
            .iter()
            .all(|c| c.genome.genes.len() == 5
                && c.genome.genes.iter().all(|g| [0, 1, 2, 7, RESPIRE].contains(&g.action))
                && c.material.as_ref().unwrap().lipid == 0.));
        migrated.step(4);
        assert!(migrated.autonomous.as_ref().unwrap().valid());
        value["autonomous"]["cells"][0]["expression"]
            .as_array_mut()
            .unwrap()
            .truncate(9);
        assert!(Core::load(&value.to_string()).is_err());
    }
    #[test]
    fn injury_impairs_material_functions_and_terminal_damage_recycles_real_stores() {
        let (_, mut e, mut h) = joint_fixture(false);
        let healthy = e.cells[0].activity(4);
        e.cells[0].state.injury = 0.7;
        assert!(e.cells[0].activity(4) < healthy);
        e.cells[0].material.as_mut().unwrap().lipid = 0.05;
        e.cells[0].state.injury = 1.;
        let before = h.totals(&e.cells);
        let site = field_index(e.cells[0].p.x, e.cells[0].p.z);
        h.recycle(&mut e);
        let after = h.totals(&e.cells);
        assert_eq!(e.cells.len(), 2);
        assert!(h.solutes[site][5] > 0.);
        assert!((before.0 - after.0).abs() < 1e-9 && (before.1 - after.1).abs() < 1e-9);
        assert!((before.2 - after.2 - h.heat).abs() < 1e-8);
    }
}

#[cfg(test)]
mod curved_tests {
    use super::*;
    fn planet() -> Core {
        let mut core = Core::new(Config {
            world_size: 128,
            world_shape: crate::domain::Shape::Sphere,
            initial_cells: 3,
            ..Config::default()
        })
        .unwrap();
        core.start_physical();
        // These are about water and what moves in it: nothing falls on them.
        core.steer(r#"{"kind":"bombardment","scale":0}"#).unwrap();
        core
    }
    #[test]
    fn the_seas_are_not_lost_to_the_sky() {
        let mut core = planet();
        let sea = |c: &Core| {
            let p = c.autonomous.as_ref().unwrap().physical.as_ref().unwrap().planet.as_ref();
            (p.unwrap().water.iter().sum::<f64>(), p.unwrap().water_error())
        };
        let (before, _) = sea(&core);
        for _ in 0..4 {
            core.step(400);
        }
        let (after, error) = sea(&core);
        assert!(after > before * 0.7, "the sea held {after} of {before} over 400 cycles");
        assert!(error.abs() < 1e-6 * before, "and no water was made or lost: {error}");
    }
    /// How fast one cell rises (+) or sinks (-) after a few moments, in deep
    /// water or over dry land, carrying the given fat and mineral.
    fn drift(lipid: f64, mineral: f64, sea: bool) -> f32 {
        let mut core = planet();
        let mut e = core.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        let d = h.domain;
        let p = h.planet.as_mut().unwrap();
        let site = (0..d.sites())
            .max_by(|&a, &b| p.surface_depth(a).total_cmp(&p.surface_depth(b)))
            .unwrap();
        let depth = p.surface_depth(site) as f32;
        assert!(depth > 1.5, "the fixture needs a deep sea: {depth}");
        if !sea {
            p.water[site] = 0.;
        }
        let (x, z) = d.point(site);
        let ground = d.ground(x, z, core.config.seed) + p.relief_at(x, z);
        e.cells.truncate(1);
        e.bonds.clear();
        let c = &mut e.cells[0];
        let m = c.material.as_mut().unwrap();
        (m.lipid, m.mineral) = (lipid, mineral);
        c.p = d.position(x, z, ground + depth * 0.5);
        c.v = V3::default();
        for _ in 0..3 {
            h.mechanics(&mut e, &mut core, &[]);
        }
        e.cells[0].v.dot(d.up(e.cells[0].p))
    }
    /// What one cell makes in a moment in the deepest sea, at noon, from
    /// light and from the rock's chemistry: set the given share of the way
    /// from the surface down to the bed.
    fn harvest(down: f32) -> (f64, f64) {
        let mut core = planet();
        let mut e = core.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        let d = h.domain;
        let p = h.planet.as_mut().unwrap();
        let site = (0..d.sites())
            .max_by(|&a, &b| p.surface_depth(a).total_cmp(&p.surface_depth(b)))
            .unwrap();
        let depth = p.surface_depth(site) as f32;
        assert!(depth > 3., "the fixture needs a deep sea: {depth}");
        let (x, z) = d.point(site);
        let overhead = d.site_normal(site);
        p.sun_direction = [overhead.x, overhead.y, overhead.z];
        let ground = d.ground(x, z, core.config.seed) + p.relief_at(x, z);
        e.cells.truncate(1);
        e.bonds.clear();
        let c = &mut e.cells[0];
        let m = c.material.as_mut().unwrap();
        m.proteins[0] = 0.01;
        m.proteins[CHEMO] = 0.01;
        let r = c.radius();
        c.p = d.position(x, z, ground + r + (depth - 2. * r) * (1. - down));
        c.v = V3::default();
        c.cooldown = 1e6;
        h.solutes[site][HYDROGEN] = 1.;
        h.mechanics(&mut e, &mut core, &[]);
        h.physiology(&mut e, &core);
        let made = &e.cells[0].state.flux;
        (made.photosynthesis, made.chemosynthesis)
    }
    #[test]
    fn hydrogen_is_found_over_the_vents_and_leaks_away_elsewhere() {
        let mut core = planet();
        core.step(160);
        let e = core.autonomous.as_ref().unwrap();
        let h = e.physical.as_ref().unwrap();
        let p = h.planet.as_ref().unwrap();
        let level = |i: usize| h.solutes[i][HYDROGEN] / h.domain.area(i);
        let mean = |sites: &[usize]| sites.iter().map(|&i| level(i)).sum::<f64>() / sites.len() as f64;
        let vents: Vec<usize> = (0..h.domain.sites()).filter(|&i| p.vent(i) > 0.5).collect();
        // Open sea more than a few sites from any vent.
        let far: Vec<usize> = (0..h.domain.sites())
            .filter(|&i| {
                p.surface_depth(i) > 1.
                    && (0..h.domain.sites()).all(|j| p.vent(j) == 0. || h.domain.distance(i, j) > 5.)
            })
            .collect();
        assert!(vents.len() > 10 && vents.len() * 5 < h.domain.sites(), "vents are narrow fields: {}", vents.len());
        assert!(!far.is_empty());
        let (near, away) = (mean(&vents), mean(&far));
        eprintln!("hydrogen over {} vents {near:.4}, over {} far sites {away:.4}", vents.len(), far.len());
        assert!(near > 0.05 && near > 5. * away, "hydrogen is a vent's: {near} against {away}");
    }
    #[test]
    fn light_fades_with_depth_and_the_rocks_chemistry_feeds_only_the_bed() {
        let (top, bed) = (harvest(0.), harvest(1.));
        eprintln!("light harvest at the surface {} and on the bed {}; vent harvest {} and {}", top.0, bed.0, top.1, bed.1);
        assert!(top.0 > 0. && bed.0 < top.0 * 0.3, "the deep bed is dim: {} against {}", bed.0, top.0);
        assert!(bed.1 > 0. && top.1 < bed.1 * 0.3, "vents feed the bed: {} against {}", top.1, bed.1);
    }
    #[test]
    fn a_swimmer_that_seeks_light_climbs_out_of_the_deep_and_a_drifter_does_not() {
        // One of each, set down near the bed of the deepest sea.
        let rise = |species: &str| {
            let mut core = planet();
            core.steer(&format!(r#"{{"kind":"seed","species":"{species}","count":1}}"#)).unwrap();
            let e = core.autonomous.as_mut().unwrap();
            let h = e.physical.as_mut().unwrap();
            let d = h.domain;
            let p = h.planet.as_ref().unwrap();
            let site = (0..d.sites())
                .max_by(|&a, &b| p.surface_depth(a).total_cmp(&p.surface_depth(b)))
                .unwrap();
            let depth = p.surface_depth(site) as f32;
            let (x, z) = d.point(site);
            let ground = d.ground(x, z, core.config.seed) + p.relief_at(x, z);
            e.cells.retain(|c| !c.collection_sources.is_empty());
            e.bonds.clear();
            let c = &mut e.cells[0];
            c.p = d.position(x, z, ground + 0.6);
            c.cooldown = 1e6;
            c.energy = 9.;
            let id = c.id;
            assert!(depth > 3.);
            // The nearest it came to the surface in 100 cycles, and how far
            // above the bed it was then.
            let mut best = (f32::INFINITY, 0.);
            for _ in 0..20 {
                core.step(20);
                let e = core.autonomous.as_ref().unwrap();
                let c = e.cells.iter().find(|c| c.id == id).expect("it lived");
                if c.state.height > 1.5 && c.state.depth < best.0 {
                    best = (c.state.depth, c.state.height);
                }
            }
            best
        };
        let (swimmer, drifter) = (rise("swimmer"), rise("plankton"));
        eprintln!("nearest the surface over deep water in 100 cycles, and how far above the bed: swimmer {swimmer:?}, drifter {drifter:?}");
        assert!(swimmer.0 < 1. && swimmer.1 > 1.5, "a light-seeker swims up to the light: {swimmer:?}");
        // (Eddies over an uneven bed do carry a drifter up off it, but not
        // to the light.)
        assert!(drifter.0 > swimmer.0 + 1., "a drifter stays down in the dark: {drifter:?} against {swimmer:?}");
    }
    #[test]
    fn water_carries_tissue_while_fat_floats_bone_sinks_and_land_bears_the_full_weight() {
        let (plain, fat, bone) = (drift(0., 0., true), drift(0.2, 0., true), drift(0., 0.2, true));
        let land = drift(0., 0., false);
        eprintln!("rise per unit time: plain {plain}, fat {fat}, bone {bone}, on land {land}");
        assert!(plain.abs() < 1e-5, "tissue is weightless in water: {plain}");
        assert!(fat > 1e-4, "fat lifts: {fat}");
        assert!(bone < -1e-4, "bone sinks: {bone}");
        assert!(land < bone * 10., "nothing carries a body in the air: {land} against {bone}");
    }
    #[test]
    fn sphere_gravity_pulls_from_all_directions_and_surface_supports_cells() {
        let mut core = Core::new(Config {
            world_size: 128,
            world_shape: crate::domain::Shape::Sphere,
            initial_cells: 3,
            ..Config::default()
        })
        .unwrap();
        core.start_physical();
        let mut e = core.autonomous.take().unwrap();
        let mut h = e.physical.take().unwrap();
        let d = h.domain;
        h.planet.as_mut().unwrap().water.fill(0.);
        for (c, n) in e.cells.iter_mut().zip([
            V3::new(1., 0., 0.),
            V3::new(0., 1., 0.),
            V3::new(0., -1., 0.),
        ]) {
            c.p = n.mul(d.radius() + 10.);
            c.v = V3::default();
        }
        let radii: Vec<_> = e.cells.iter().map(|c| c.p.len()).collect();
        h.mechanics(&mut e, &mut core, &[]);
        for (c, before) in e.cells.iter().zip(radii) {
            assert!(c.p.len() < before);
            assert!(c.v.dot(d.up(c.p)) < 0.);
        }
        for c in &mut e.cells {
            let (x, z) = d.coords(c.p);
            let floor = h
                .planet
                .as_ref()
                .unwrap()
                .floor(x, z, c.radius(), core.config.seed);
            c.p = d.position(x, z, floor - 0.2);
            c.v = V3::default();
        }
        h.mechanics(&mut e, &mut core, &[]);
        for c in &e.cells {
            let (x, z) = d.coords(c.p);
            let floor = h
                .planet
                .as_ref()
                .unwrap()
                .floor(x, z, c.radius(), core.config.seed);
            assert!(c.p.len() - d.radius() >= floor - 0.001);
        }
    }
    #[test]
    fn globe_and_large_planar_populations_conserve_and_resume() {
        for shape in [crate::domain::Shape::Planar, crate::domain::Shape::Sphere] {
            for size in [64, 128, 256] {
                let mut core = Core::new(Config {
                    world_size: size,
                    world_shape: shape,
                    founder_spread: "global".into(),
                    initial_cells: 16,
                    ..Config::default()
                })
                .unwrap();
                core.start_physical();
                let radius = core.autonomous.as_ref().unwrap().cells[0].radius();
                assert!((radius - 0.22).abs() < 1e-6);
                core.step(12);
                let e = core.autonomous.as_ref().unwrap();
                assert!(e.valid());
                let h = e.physical.as_ref().unwrap();
                assert!(h.errors(&e.cells).iter().all(|x| x.abs() < 1e-7));
                assert!(h.planet.as_ref().unwrap().water_error().abs() < 1e-7);
                let save = serde_json::to_string(&core).unwrap();
                let mut resumed = Core::load(&save).unwrap();
                core.step(4);
                resumed.step(4);
                assert_eq!(
                    serde_json::to_string(&core).unwrap(),
                    serde_json::to_string(&resumed).unwrap()
                );
            }
        }
    }
    #[test]
    fn mismatched_geometry_in_saves_is_rejected() {
        let mut core = Core::new(Config {
            world_size: 128,
            world_shape: crate::domain::Shape::Sphere,
            ..Config::default()
        })
        .unwrap();
        core.start_physical();
        let mut s = serde_json::to_value(&core).unwrap();
        s["config"]["worldSize"] = 64.into();
        assert!(Core::load(&s.to_string()).is_err());
        let mut s = serde_json::to_value(&core).unwrap();
        s["autonomous"]["physical"]["planet"]["domain"]["grid"] = 40.into();
        assert!(Core::load(&s.to_string()).is_err());
    }
}
