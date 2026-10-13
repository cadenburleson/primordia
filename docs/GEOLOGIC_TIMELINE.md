# The geologic timeline, and where the sim stands on it

Written 2026-10-12 from the timeline diagram Caden supplied (Hadean at the bottom, Quaternary at the top) and the sources at the end. For each stretch of Earth's history: what the planet was like, what life could newly do, what made that possible, and whether the engine can do it. The app shows the same list under **Deep time** in the ladder panel and marks how far a planet's life has come (`src/timeline.ts`).

Dates are millions of years ago and differ by a few million between sources.

## What each period added

| Period | When | The planet | What life could newly do | What made it possible |
| --- | --- | --- | --- | --- |
| Hadean | 4,600–4,000 | Forming; struck constantly; a Moon-forming impact about 4,500; thin air, new seas | Nothing lived | |
| Archean | 4,000–2,500 | Cooling; first stable continents; no free oxygen; ultraviolet reaches the surface | Single cells; mats (stromatolites) common by the end; light-harvesting | Chemistry at vents; later, sunlight |
| Proterozoic | 2,500–541 | Oxygen rises (2,400–2,000); iron rusts out of the sea; ozone forms; "snowball" ice ages near the end | Breathing oxygen; complex cells (2,700–2,100 by different estimates); many-celled life by about 750; soft-bodied animals in its last period, the Ediacaran (635–541) | Free oxygen, which gives fifteen times the energy; one cell living inside another |
| Cambrian | 541–485 | Warm shallow seas; no life on land | Most animal body plans within a few tens of millions of years: muscle, nerves, eyes, mouths, guts, armour. Trilobites. | Still argued: a small rise in oxygen passing the point where hunting pays, then an arms race between eater and eaten; eyes are another candidate |
| Ordovician | 485–444 | Seas spread over the continents; ends in an ice age that kills about three species in four | Shells and reefs; the first fish, jawless with bone armour; first plants on wet ground (spores, cuticles) | Mineral skeletons; bone |
| Silurian | 444–419 | Ice melts; stable warm climate | Plants with stems and vessels; the first animals on land; fish with jaws and fin rays; swimmers in open water | A waterproof skin and plumbing; jaws |
| Devonian | 419–359 | Ends in another great dying | "Age of fishes"; the first forests; amphibians, the first four-legged animals; winged insects by some accounts | Backbones; wood; limbs |
| Carboniferous | 359–299 | Swamp forests bury carbon; oxygen at its highest | Giant insects; reptiles; eggs that can be laid on dry land | High oxygen; the shelled egg |
| Permian | 299–252 | One dry continent; ends in the largest extinction on record | Sail-backed ancestors of mammals | Skin and eggs that hold water |
| Triassic | 252–201 | Hot and dry; the continent begins to split | First dinosaurs and first mammals; conifers dominate | Empty niches after the dying |
| Jurassic | 201–145 | Warm, wet | Giant dinosaurs; the first birds; insect pollinators | |
| Cretaceous | 145–66 | Ends with an asteroid strike | Flowering plants | Flowers and their pollinators |
| Paleogene | 66–23 | Warm, then cooling | Mammals grow large and diversify; ancestors of horses, camels and elephants | The dinosaurs' absence |
| Neogene | 23–2.6 | Cooling, drying; mountains rise | Grasslands; hoofed grazers; the first upright apes about 6 | Grasses that keep growing when eaten |
| Quaternary | 2.6–now | Ice ages | Mammoths; people | |

## The same list for the sim

Every period now has a first-pass cast and a test. "Reached" is the test the app applies, in order: a planet is in the last period whose test it passes along with every earlier one. The later tests are stand-ins and say so.

| Period | Mechanisms in the engine | Starters | Test the app applies |
| --- | --- | --- | --- |
| Hadean | Thin young air; a strike every 17 cycles or so, fading; strikes leave carbon, nitrogen, phosphorus and iron | | Always |
| Archean | Vent chemistry and hydrogen; light-feeding; no oxygen; ultraviolet kills at the surface until ozone forms | Archaeon, Cyanobacterium, Anabaena, **Mat** | Living cells that hold their numbers |
| Proterozoic | Oxygen, rusting iron, ozone; breathing at fifteen times fermenting; kept breathers and light-feeders; **matrix and fibre (collagen) that take free oxygen to make** and hold a body together | Plankton, Alga, Paramecium, Hunter, Volvox, **Sponge**, **Coral**, **Dickinsonia** | Oxygen breathing and complex cells established; ozone stopping half the ultraviolet |
| Cambrian | Mouths and pumped feeding currents; venom, spines, skin; **muscle that works**, nerves, bristles; eyes that **see other bodies at three times the reach of touch**; **scent that follows dissolved organic matter**; **a body plan read from position** (middle line, paired limbs, head); **gills and a pumped fluid that carry oxygen to cells inside a body** | Jelly, Hydra, **Worm**, **Trilobite**, **Shrimp**, **Copepod** | Bodies, animals, hunters, defences, senses and powered movement established |
| Ordovician | Mineral skeletons, capped at 15% of a cell; **teeth**: mineral in a mouth cuts through armour and a body's grip on its cells | **Sea scorpion** | Skeletons (in one animal cell in ten) and meat-eaters (one body in two hundred takes a third of its prey or more from animals) established |
| Silurian | Land with weight, drying and a water store; holdfasts that grip only where it is damp; **wind that carries a dry spore**; plants begin at the water's edge | **Moss**, **Fungus**, **Fish** | 20 cells of walled plant ashore |
| Devonian | **Branching by rule** (fractal growth) | **Fern**, **Amphibian** | 20 cells of branching plant and 10 of animal ashore |
| Carboniferous | **Wings**: grip on air, lift from forward speed; **greenhouse**: the air's carbon dioxide sets the planet's warmth, so burying carbon cools it | **Tree**, **Dragonfly** | 200 cells of branching plant ashore and 3 winged cells in the air |
| Permian | Waterproof skin, fat as yolk | **Reptile** | 50 cells of dry-skinned animal ashore |
| Triassic | **Warm blood**: full pace in the cold, at a price fat and a compact body bring down | **Dinosaur** | 10 warm-blooded cells in animal bodies |
| Jurassic | | **Bird** | A land animal of 30 cells and 3 warm winged cells in the air |
| Cretaceous | Gametes and fusion on land | **Flower** | 20 cells of land plant with the machinery of sex |
| Paleogene | Fat under the skin of a warm body | **Mammal** | 100 warm-blooded cells in animal bodies |
| Neogene | Glass in plant cells | **Grass** | 50 cells of glassy plant ashore |
| Quaternary | Nerve junctions that learn | **Ape** | 50 learning junctions between warm-blooded cells. This is as near a mind as the engine goes |

Bold marks what was added on 2026-10-12.

### The whole cast, measured

Six planets (seeds 5, 77, 8042, 11, 23, 404). Light-feeders alone for 200 cycles, then 24 single cells of every other creature released once, no restocking. Run twice: once before the oxygen law and once on the final engine. Cells of each at cycle 1,400 on the final engine, and on how many planets each was alive in the two runs:

| Creature | On the six planets (final engine) | Alive on (first run, final) |
| --- | --- | --- |
| Sponge | 202, 262, 548, 257, 394, 642 | 6, 6 |
| Coral | 22, 35, 55, 125, 154, 129 | 6, 6 |
| Mat | 100, 24, 58, 316, 133, 202 | 6, 6 |
| Fern | 29, 15, 10, 40, 4, 7 | 6, 6 |
| Tree | 9, 42, 3, 28, 5, 32 | 5, 6 |
| Flower | 1, 4, 19, 7, 108, 24 | 6, 6 |
| Grass | 19, 45, 25, 39, 2, 98 | 6, 6 |
| Dickinsonia | 8, 2, 18, 106, 0, 88 | 5, 5 |
| Trilobite | 16, 0, 0, 27, 41, 39 | 6, 4 |
| Moss | 0, 136, 19, 41, 0, 55 | 6, 4 |
| Hydra | 0, 0, 15, 0, 4, 0 | 2, 2 |
| Jelly | 16, 10, 0, 0, 0, 0 | 1, 2 |
| Fish | 0, 0, 0, 0, 33, 0 | 2, 1 |
| Worm | 0, 0, 0, 0, 0, 4 | 1, 1 |
| Dragonfly | 0, 0, 4, 0, 0, 0 | 0, 1 |
| Sea scorpion, Bird | none | 1, 0 |
| Amphibian, Reptile, Dinosaur, Mammal, Ape | none | 0, 0 |

On the final engine every planet had many-celled bodies, hunters, defences and senses established; animals on five; movement by muscle on one, and partly on four more. By the app's test that one planet is in its Cambrian.

What this says:

- What sits still and filters or grazes does well. Sponges were about a thousand cells a planet before the oxygen law and about four hundred after it.
- What swims and hunts does badly when everything is released at once. Jelly, alone with the single-celled hunters, lived on five planets of six.
- **Nothing lives on land.** In the first run the five land plants were counted cell by cell: 25 of their 1,353 cells were out of the water. They live as water plants. Nearly everything found ashore was stranded single cells. The app's tests for the land periods now count only plants that hold on to the ground and animals that have been alive a while and are not dying. No land animal has survived at all. **This changed later the same day: see "Land" below.**

### The whole cast again, with grown bodies

The same six planets and the same release, on the engine as it stood at the end of 2026-10-12: the 21 starters that grow into bodies set down grown, the rest as single cells. All six planets passed their final validation. Cells of each at cycle 1,400 on planets 5, 77, 8042, 11, 23 and 404:

| Creature | Cells on the six planets | Alive on, before and now |
| --- | --- | --- |
| Dickinsonia | 560, 895, 511, 1,066, 164, 776 | 5, 6 |
| Sponge | 185, 389, 587, 366, 207, 323 | 6, 6 |
| Trilobite | 65, 11, 74, 31, 346, 436 | 4, 6 |
| Mat | 41, 395, 251, 211, 40, 252 | 6, 6 |
| Coral | 63, 231, 91, 118, 120, 25 | 6, 6 |
| Moss | 131, 11, 242, 115, 77, 14 | 4, 6 |
| Tree | 49, 71, 68, 89, 77, 114 | 6, 6 |
| Grass | 60, 35, 130, 67, 46, 95 | 6, 6 |
| Flower | 22, 1, 61, 3, 34, 19 | 6, 6 |
| Fern | 47, 0, 78, 25, 17, 71 | 6, 5 |
| Amphibian | 0, 0, 57, 19, 15, 30 | 0, 4 |
| Reptile | 72, 16, 5, 2, 0, 0 | 0, 4 |
| Fish | 0, 1, 180, 183, 0, 0 | 1, 3 |
| Hydra | 5, 0, 29, 0, 0, 7 | 2, 3 |
| Jelly | 4, 0, 76, 0, 0, 28 | 2, 3 |
| Dragonfly | 2, 0, 13, 0, 0, 9 | 1, 3 |
| Dinosaur | 0, 0, 8, 4, 1, 0 | 0, 3 |
| Ape | 0, 2, 5, 0, 10, 0 | 0, 3 |
| Worm | 0, 0, 0, 2, 0, 12 | 1, 2 |
| Sea scorpion | 0, 0, 0, 0, 108, 0 | 0, 1 |
| Bird, Mammal | none | 0, 0 |

Each planet held 106 to 370 bodies of four cells or more and 24 to 85 of ten or more, the largest 27 to 50 cells; 16 to 31% of all cells were in bodies of four or more.

**By the app's own tests, four of the six planets are in their Cambrian** (8042, 11, 23 and 404): animals, hunters, defences, senses and movement by muscle all established. One was before. Movement by muscle is established on five of six; it was established on one before. The other two are held one step back, planet 5 by the animals step and planet 77 by movement.

All four Cambrian planets fail the same next test, the Ordovician's: carnivores established. A planet had 2 to 14 cells living mainly on other animals (0.1 to 0.2% of its bodies), where the test wants one body in two hundred. The eaters here eat light-feeders. The Sea scorpion, built to eat animals, lasted on one planet.

The tests for the later periods, each taken alone, were met on these numbers of planets: Silurian (twenty cells of plant out of the water) six; Triassic six; Devonian four; Cretaceous four; Neogene four; Jurassic two; Paleogene two; Permian one; Carboniferous and Quaternary none. A planet is only said to be in a period when it has met every earlier one, so none is shown past the Cambrian.

One run is one draw. The run before this one, on the same engine but with the ten simpler starters still released as single cells, ended with Fish on five planets (172 cells on one), Amphibian on five (863 on one), Dragonfly on four (327 on one) and Trilobite at up to 1,495; one of its planets failed validation at the end, which is what the top-of-the-air rule fixed.

### Left alone: what a planet invents by itself

Caden, 2026-10-12: "Now this is just a system. We need to have it to where the system is also able to create its own complex life through what we know as biology and genetics." And: "if we get the cell division and genetics right and the world is set up properly everything else should create itself over time, no?"

In principle yes. In practice this is the open problem of the field ("open-ended evolution"): Tierra, Avida, DISHTINY, ALIEN and others have each got part of the way and none has gone from single cells to animals. Three things stand in the way, and the engine's own runs show all three:

1. **Every step has to pay by itself.** Selection cannot hold on to a part that is useless until two others arrive.
2. **Scale.** A planet here has about ten thousand cells and a few thousand cycles.
3. **Nothing evolves that the rules do not allow.** The mechanisms built above are the vocabulary.

What was changed in the genetics for this:

- New genes mostly arise as copies of old ones that keep the old gene's switch (six in ten), so a new part appears in the same cells as an old one. The rest answer to three signals, not to all 34 at random.
- Genes pass between touching cells in runs of up to three.
- A cell can read where it is in its body (how far forward, how far from the middle line), and sideways growth is mirrored, so a two-sided body can be coded by one rule.
- **Parts were given an earlier use of their own**, as they had on Earth: an excitable membrane makes a single swimming cell bolt while it is firing, and vanes among the cilia push more water. Before this, nerve and bristles did nothing until a body already had muscle.

The test: a planet seeded with 16 of the simplest cell (light harvest, uptake, division, glue, breathing) and nothing else. Mutation at twice the natural rate, gene sharing on, no breeding goal.

| | Before the earlier uses (six planets, 1,400–3,000 cycles; four cut short, see below) | With them (five planets, 2,600–3,000 cycles) |
| --- | --- | --- |
| Established on every planet | Living cells, many-celled bodies, senses, swimming, hunters, nitrogen fixing | Living cells, many-celled bodies, swimming, hunters, nitrogen fixing, complex cells, defences |
| On some | Defences 5, complex cells 4, skeleton 3, walls 3, muscle 2, animals 1 | Steering 3, walls 3, animals 2, skeleton 2, fins 2, cell fates 2, senses 2, muscle 1, castes 1 |
| Bodies of four cells or more | 70–206 a planet, the largest 110–176 cells | 81–344 a planet |
| Bodies with muscle and nerve | 1, 1, 14, 3, 9, 1 | 3, 3, 2, 5, 6 |
| With bristles too | 0, 0, 1, 0, 0, 0 | 1, 0, 1, 0, 4 |
| And a mouth | none | 1, 0, 1, 0, 3 |

So unaided, a planet reaches roughly the end of the Proterozoic in about 2,000 cycles, and with the earlier uses in place the parts of a muscle-driven animal now come together in a few bodies on three planets of five. On none has movement by muscle spread far enough to count as established.

A third run, with joints and over-stretched ties bounded, gave the first planet on which **movement by muscle was established with nothing placed by hand** (planet 77, at cycle 2,400; muscle at 2,200, fins at 1,000), and bodies with muscle, nerve and bristles together on four planets of six.

A fourth run on the final engine, with the oxygen law below in force (six planets, 2,400 cycles, none failing):

| | Result |
| --- | --- |
| Established on every planet | Living cells, swimming, hunters, defences |
| On some | Many-celled bodies 5, nitrogen fixing 5, senses 4, skeleton 3, walls 2, shapes 2, muscle 1, complex cells 1 |
| Largest body on each planet | 835, 23, 81, 152, 30, 94 cells |
| Bodies with muscle, nerve and bristles | 2, 0, 0, 6, 0, 0 (three of them with a mouth) |
| Bodies of ten cells or more | Half their surface open on average, five or six cells touching each |
| Gas-exchange and pumping cells | Appeared unaided: on one planet 493 and 988 cells of 11,619, in 6 and 8 of its 32 larger bodies. Whether they are doing any work there was not measured |

A fifth run on the engine as it stood at the end of 2026-10-12 (cheap genes, carried oxygen, one individual to a body; six planets, 2,400 cycles, none failing):

| | Result |
| --- | --- |
| Established on every planet | Living cells, senses, hunters, nitrogen fixing, complex cells, defences |
| On some | Swimming 5, colonies 4, walls 4, many-celled bodies 3, shapes 2, skeleton 1, steering 1, muscle 1 |
| Bodies of four cells or more | 335, 25, 174, 229, 263, 300 |
| Largest body on each planet | 25, 20, 35, 23, 24, 19 cells |
| Bodies with a muscle pulling | 11, 0, 6, 6, 37, 0 |
| Bodies with muscle and nerve | 1, 0, 0, 0, 1, 8; none with bristles as well |
| Movement by muscle | The gene is in every pool; no body on any planet moves by it |

Against the fourth run: complex cells and senses went from one and four planets to all six, and there are more small bodies, but the largest body on a planet, which had been 23 to 835 cells, is now 19 to 35, and the parts of a muscle animal came together in no body. The cause of the smaller bodies is the rule that a cell which leaves a body founds a new individual. Before it, loose kin could glue back on, and the bodies of hundreds of cells were clumps made that way. Tested by running the same six planets again with only that rule switched off:

| Six bare planets, 2,400 cycles | Rule on (a cell that leaves is a new individual) | Rule off (loose kin may glue back on) |
| --- | --- | --- |
| Largest body on each planet | 25, 20, 35, 23, 24, 19 | 108, 43, 53, 321, 102, 278 |
| Bodies of four cells or more | 335, 25, 174, 229, 263, 300 | 95, 34, 34, 19, 97, 124 |
| Bodies of thirty or more | 0, 0, 2, 0, 0, 0 | 10, 2, 4, 3, 7, 12 |
| Many-celled bodies established | 3 planets | 6 planets |
| Bodies with a muscle pulling | 11, 0, 6, 6, 37, 0 | 1, 0, 0, 0, 2, 1 |
| Bodies with muscle and nerve | 1, 0, 0, 0, 1, 8 | none |
| Movement by muscle | none | none |

So it is a trade. With the rule, a planet makes many small bodies that each grew from one cell, and fifteen times as many of them work a muscle. Without it, a planet makes a few bodies several times larger by clumping. The rule stays on: the designed animals need it (their young glued back onto their parents and grew there as lumps), and a body grown from one cell is what an animal is. What it costs is that an unaided planet no longer reaches a body of 24 cells on half the planets, which is one of the steps the Cambrian test asks for. A narrower fix, a coat on a shed young cell in place of a rule about its ancestry, would keep both and is not built.

These runs differ from one another as much as from the changes between them, so no single column proves a change helped. What holds across all five: a planet left alone gets to hunters and defences every time, to bodies of a few dozen cells every time, and now and then to the parts of a muscle animal in one body. It has once got to movement by muscle. The designed animals show the physics will carry a muscle-driven body; evolution here has not yet found one and kept it.

**A fault this found.** Four of the first six planets, and one in each of the next two runs, stopped with the engine's own validity check failing. Cells were leaving the planet. Three causes, in the order found: the new wings gave lift without limit and added energy (lift now acts across the direction of travel and thins with the air); joints with no lever arm and over-stretched ties could push without limit (bounded); and in very crowded water cells were knocked to 6–10 units a cycle, when anything leaving this small planet at more than about 3.5 never returns. Nothing that begins a moment under water can now be thrown out of it, which is what surface tension does to anything that small. This probably also explains the loose grains thrown upward that were noted earlier and never traced.

### Cell types: the building blocks

Caden, 2026-10-12: "we need to figure out all the cell types, bc they are the building blocks of what gets developed after", with a list. The engine has no fixed cell types. A cell is whatever its genes have built out of 53 products; the app names a cell by what it has most of. So the question for each type on the list is whether the products exist and whether the physics gives them a job, because selection only keeps what has one.

| Cell type | Products in the engine | Does the physics give it a job? |
| --- | --- | --- |
| Bacterium, prokaryote | A small cell that breathes through its own membrane, nothing kept inside | Yes |
| Eukaryote | A cell that keeps a swallowed breather (mitochondria) or light-feeder (chloroplast) | Yes |
| Stem cell | An uncommitted cell in a body; three committed fates | Yes |
| Muscle cell | Contraction (4) | Yes, since 2026-10-12 |
| Nerve cell | Excitable channels (12), receptors (11), light (16), touch (24), junctions that learn (17) | Yes; channels now also make a lone cell bolt |
| Skin, epithelial cell | Protective membrane (14); adhesion (2); sideways division into sheets (43) | Yes. A sheet used to be counted as buried; exposure is now counted by what touches a cell |
| Fat cell | Lipid store (10) | Yes: energy, buoyancy, insulation for warm blood |
| Sex cell | Gametic division (18), fusion (19) | Yes |
| Bone cell | Mineral (3) in matrix; remodelling (29) | Yes: stiffness, joints, armour, teeth |
| Fibroblast | Matrix (21), fibre (28), polymer assembly (30) | Yes: holds bodies together, needs oxygen |
| Plant cell | Light harvest (0), wall (41), water store | Yes |
| Gland, pancreatic cell | Secreted enzymes (9, 15, 27, 31) and two signal chemicals (5, 6) | Partly: signals drive nerves and growth direction; nothing yet acts as a hormone across a body |
| Leucocyte | Membrane recognition (25) with a mouth or enzymes | Partly: a body's mouths already eat strangers; nothing infects a body from inside |
| Liver cell | Stores fuel and passes it on (8, 10) | Partly: no poisons to clear |
| Blood cell, endothelial cell | Gas exchange (23), a pumped body fluid that carries oxygen and solutes (22), membrane transport (8) | **Now yes.** Until 2026-10-12 every cell breathed freely however deep in a body it was, so none of this had a use |
| Cancer cell | Any cell in a body whose control of division mutates | It can happen (every division can mutate); nothing counts it yet |

**The law that was missing.** Oxygen now gets into a cell only through the surface it has open to the water, in proportion. A cell packed inside a body gets almost none that way. So a body can stay thin, or branch, or grow gills, a pumped fluid and vessels, and nothing else lets it be thick. The app now names five more kinds of cell: Gill or lung, Heart or vessel, Wing, Gland, Immune.

**Hard limits and laws.** Caden: "We should try less to hard code limitations, but set world physics that govern the life, this way it has to adapt on its own." Replaced that day: a cap on collagen (now a rate of making against a rate of breakdown), a cap on a wing's lift (now area times air thickness times speed squared, acting across the direction of travel so it adds no energy, with drag by the same measure), and a wall on leaving the water that exempted wings (now nothing that starts a moment under water leaves it; things take off from the surface or the ground). Still a cap standing in for missing physics: skeleton at 15% of a cell, because skeleton is drawn from the same dissolved pool as phosphorus. The fix is shell made from calcium and carbonate.

### An animal is many cells: body plans, the nursery, and release as grown bodies

Caden, 2026-10-12: "I'm confused bc everything still looks like a single cell? A sea scorpion is a single cell? Shouldn't it be a bunch of cells working together to make an organism?" It should, and it was not: every starter was set down as one founder cell, and most of the animals never grew past a clump.

**The body plan.** Eleven animals (Worm, Trilobite, Sea scorpion, Fish, Amphibian, Dragonfly, Reptile, Dinosaur, Bird, Mammal, Ape) are now built by one set of developmental genes, `planned` and `organs` in `engine/src/bestiary.rs`. Nothing in it places a cell. It is read by each cell from where it finds itself:

| Part of the plan | How a cell knows |
| --- | --- |
| A middle line of a set length | The stem cell at the front divides forward until its lineage is that many divisions deep, then stops |
| A limb on each side of every segment | A cell on the middle line buds sideways, to whichever side it has fewer neighbours on; a limb cell divides only outward |
| A head | Mouth, eyes and the pacemaker switch on by how far forward of the body's first cell a cell was born |
| Muscle, and the pump | On the middle line, off the limbs |
| Fins or feet, and gills | On the limbs |
| Nerve and matrix | Wherever a cell has a neighbour |
| Young | A cell that comes away from the body is the first cell of a new one |

A gene's switch reads three facts of position for this: how far along the body a cell is, how far out from the middle line, and whether it has set a daughter down ahead of itself. The backboned animals add bone along the middle, fat, senses and nerves that learn; the land animals add a waterproof skin; birds and the dragonfly put wings on the limbs.

**One genome to a body.** Three changes keep a body's cells copies of its egg. Copying inside a body is twenty times more faithful than in a cell alone. A cell takes up genes only from other individuals, never from its own body. And a cell that leaves a body becomes the origin of its own: before, it counted as part of the body it had left, glued itself back on, and grew there as a lump.

**The nursery.** `Core::nursery` raises one egg in an empty, oxygenated pond with nothing miscopied and a keeper's hand feeding every cell. After 300 cycles:

| Animal | Cells | Middle line | Left | Right | Other cells in the nursery (its young) |
| --- | --- | --- | --- | --- | --- |
| Worm | 21 | 8 | 6 | 7 | 8 |
| Trilobite | 24 | 9 | 8 | 7 | 105 |
| Sea scorpion | 21 | 5 | 8 | 8 | 50 |
| Fish | 22 | 13 | 6 | 3 | 46 |
| Amphibian | 33 | 11 | 13 | 9 | 20 |
| Dragonfly | 19 | 8 | 5 | 6 | 0 |
| Reptile | 19 | 9 | 4 | 6 | 0 |
| Dinosaur | 16 | 3 | 7 | 6 | 0 |
| Bird | 16 | 5 | 6 | 5 | 0 |
| Mammal | 36 | 10 | 15 | 11 | 12 |
| Ape | 22 | 10 | 6 | 6 | 0 |

Limbs come away and grow back all the time, so the two sides are near a match at any moment, not an exact one. Five of the eleven shed nothing in the nursery: young leave when a tie dissolves, which is quicker under strain, and a calm, well-fed body keeps its limbs.

**Released grown.** Releasing or painting one of these eleven sets down a body raised in the nursery, laid flat, in the best-lit water deep enough to hold it (land animals at the water's edge). Loose single cells in the way are moved aside. Where there is no room for a body, its eggs are released as before. The app's creature brush paints the same grown body.

Ten more that grow into bodies by simpler rules (budding, branching, spreading as a sheet) are released grown the same way: Jelly, Hydra, Sponge, Coral, Dickinsonia, Moss, Fern, Tree, Flower and Grass. In the nursery they reach 12 to 125 cells (Flower 12, Jelly 14, Sponge 15, Coral 24, Moss 24, Grass 34, Tree 48, Fern 60, Hydra 77, Dickinsonia 125). A release sets down one body for every four founders asked for, and fewer of a large one: about four cells for each founder. The other 27 starters are still released as single cells. Most are single-celled; Volvox, Ulothrix, Sea lettuce and Seaweed grow their colonies from one.

**What it took for a grown body to live.** Set down in a pond of prey, the first grown Worms and Sea scorpions were dead within 100 cycles, their fuel gone in about 50 ticks. The Trilobite lived, on remains. The Fish lived only because its nerves had jammed and it never moved. Five causes, each found by printing what a body spent and caught tick by tick:

| Cause | Measured | Change |
| --- | --- | --- |
| A pond's bed has hills, and bodies were being set down on them | Out of the water one tick after release | Release needs standing water deeper than the body is tall |
| A cell inside a body ferments | These cells have 40 to 50% of their surface open, so at most that share of their fuel was burned fully and the rest gave a fifteenth. Oxygen brought in by gills could only be burned by the gill cells themselves | Any cell burns fuel with oxygen its body carries to it. Gills pass gas about thirteen times faster (a gill is a folded surface). Measured after: 0.90 to 1.00 of the fuel burned on the middle line is burned fully, and all of it in the limbs |
| Every cell made muscle, at full strength | Muscle took half of what the body spent | Muscle on the middle line only; the limbs are paddles |
| Carrying a gene cost 7% of a cell's upkeep, per gene | A body cell with the 23 genes an animal needs paid 45% of everything it spent on upkeep, two thirds of that for carrying genes | Carrying a gene costs about 1% of upkeep. What a gene costs is what it makes, charged where the protein is built |
| A strongly driven nerve jammed on | Excitation pinned at 0.55 to 0.75; muscles held tight and never beat. It did this to every land animal at the shore | The drive on a nerve has a ceiling. Worked out from the membrane's equations, it stops firing above about 0.53. With the ceiling at 0.5 the Bird and the Mammal still held; at 0.35 all eleven beat |

A bite now also holds on: mouth and prey share their momentum, so a swimming body no longer knocks away what it is eating.

**What it took for one to have young that live.** On six planets (a run stopped between cycles 600 and 1,200) every grown animal but the Trilobite was falling however well its adults fed: none of the other ten held more than 48 cells on any planet and most were gone, while the Trilobite stood at 76 to 1,266 cells and rising. Two causes:

| Cause | Measured | Change |
| --- | --- | --- |
| A larva had no mouth | The mouth switches on by a cell's place along a body, and a cell alone has no place. The Trilobite's young lived because every cell of it digests remains through its skin | A cell alone, or at the very end of a limb, has a mouth of its own. It eats its way up to an egg's size and then cleaves into a body. Every one of the eleven now sheds young in the nursery; five shed none before |
| Larvae ate their parents | With mouths, the young swallowed the fins and tails of their own kind: grown Worms were gone from a pond in 200 cycles, Birds and Apes in 400 | A mouth spares every cell of a kin body that has a mouth, not only that body's mouth cells |

After both, in a pond of prey for 400 cycles from two grown bodies: Worm 37 cells in bodies of 23 and 14, catching 0.0056 fuel a cell-cycle and burning 0.0057; Fish 37 cells (24, 9, 2 and two larvae); Amphibian 48 (20, 14, 3 and two larvae); Sea scorpion 28 (23, 5); Dragonfly 24; Bird 23; Mammal 15; Reptile 8; Dinosaur 5; Ape 1. The grown animals hold their own in a fresh pond. Their larvae feed, and few had yet grown into second bodies in that time.

**A fault in that measurement, found later.** The test pond had a limit of 8,192 cells, and the plankton in it reached 8,191 about 200 cycles after each release. In a full world a cell divides only when a place falls free, and thousands of plankton cells are waiting for each one. So "few larvae had grown into second bodies" was the limit, not the animals: a Shrimp larva was watched reaching the size it needed (2.55 of tissue against 2.21, energy full, nothing holding it back) and then sitting for 400 ticks without dividing. Every pond figure above past cycle 200 is under that cloud, and so is the trial of tiring muscle. The pond probes now have a limit of 40,000.

With room, one Shrimp line was followed for 600 cycles in a pond that grew from 6,000 to 24,600 cells. Two grown bodies of 14 and 12 cells were set down. One shed a larva; the larva ate for about 120 cycles, from 0.6 of tissue to 2.2, and cleaved into a body of 8 to 9 cells; that body shed a larva in its turn, which became a body of 13; and a third generation followed. So the whole life cycle runs: body, larva, body. What it does not do is multiply. There were never more than two bodies at once, each leaving about one that lived, and a generation took 150 to 200 cycles.

The ledger for a grown Worm on a crowded planet before the last two changes, per cell per tick: upkeep 0.023, muscle 0.018, making protein 0.007, everything else 0.002. It caught about a sixth of that.

**Measured on the final engine.** Two grown bodies of each, set down in a pond where plankton and cyanobacteria had grown for 800 cycles, watched for 120 ticks (30 cycles). Work is in energy and prey in fuel; one fuel is 22.5 energy.

| Animal | Cells at release | Fuel a cell, start to end | Muscle work a cell-tick | Prey a cell-tick | Speed of its cells | Share under water |
| --- | --- | --- | --- | --- | --- | --- |
| Worm | 42 | 0.282 to 0.259 | 0.023 | 0.0032 | 0.21 | 1.00 |
| Trilobite | 48 | 0.207 to 0.003 | 0.032 | 0.0007 | 0.22 | 1.00 |
| Sea scorpion | 42 | 0.318 to 0.320 | 0.012 | 0.0031 | 0.16 | 1.00 |
| Fish | 44 | 0.264 to 0.136 | 0.056 | 0.0038 | 0.35 | 0.99 |
| Amphibian | 64 | 0.226 to 0.026 | 0.005 | 0.0013 | 0.08 | 1.00 |
| Dragonfly | 38 | 0.329 to 0.163 | 0.042 | 0.0033 | 0.28 | 1.00 |
| Reptile | 36 | 0.286 to 0.226 | 0.046 | 0.0054 | 0.24 | 0.26 |
| Dinosaur | 32 | 0.302 to 0.106 | 0.002 | 0.0036 | 0.05 | 0.10 |
| Bird | 32 | 0.287 to 0.066 | 0.038 | 0.0044 | 0.22 | 0.10 |
| Mammal | 72 | 0.308 to 0.004 | 0.020 | 0.0029 | 0.19 | 0.27 |
| Ape | 44 | 0.322 to 0.104 | 0.051 | 0.0066 | 0.28 | 0.16 |

All eleven were alive at the end, none had lost more than two cells, all were working their muscles and all were eating. The five land animals stood at the water's edge with a tenth to a quarter of each cell under water and moved there by muscle. Only the Worm and the Sea scorpion were holding their fuel: the rest were spending faster than they caught, the warm-blooded ones fastest. The Trilobite does not swallow prey; it lives on remains, which a fresh pond has little of.

### Land

Until this day the five land plants lived as water plants: 25 of their 1,353 cells were out of the water. They were set down on the sea bed and stayed there, since nothing carried a spore ashore. Four changes:

- **They start at the water's edge**, in a hand's depth of water (0.02 to 0.2 units), where a plant's feet are wet and the rest of it is in air. So do the land animals.
- **Wind.** A loose, dry cell lying on the ground is blown along it: half a unit a tick if it is as small as a spore, less in proportion if it is heavier. It is never blown out over open water.
- **A dried-out holdfast grips nothing.** A cell short of water lies dormant and loose until water reaches it, so a spore travels until it is rained on or reaches a damp place.
- **Ashore is counted by body.** A plant is on land when something of it out of the water holds the ground, and every cell of it out of the water counts. The old count took only the one cell touching the ground.

Measured on the six planets of the last run, at cycle 1,400. "Holding ground" is a cell out of the water that is anchored or resting on the ground; the three numbers are cells holding ground, cells out of the water, and all cells:

| Planet | Plant cells ashore, by the app's count | Moss | Fern | Tree | Flower | Grass |
| --- | --- | --- | --- | --- | --- | --- |
| 5 | 222 | 38 / 58 / 131 | 26 / 40 / 47 | 23 / 25 / 49 | 21 / 22 / 22 | 15 / 16 / 60 |
| 77 | 80 | 9 / 9 / 11 | none left | 15 / 19 / 71 | none ashore | 9 / 9 / 35 |
| 8042 | 330 | 73 / 98 / 242 | 17 / 23 / 78 | 18 / 32 / 68 | 7 / 7 / 61 | 31 / 42 / 130 |
| 11 | 275 | 63 / 79 / 115 | 13 / 20 / 25 | 23 / 33 / 89 | 2 / 2 / 3 | 22 / 29 / 67 |
| 23 | 229 | 44 / 58 / 77 | 10 / 13 / 17 | 51 / 75 / 77 | 29 / 33 / 34 | 7 / 10 / 46 |
| 404 | 325 | 6 / 6 / 14 | 13 / 27 / 71 | 42 / 51 / 114 | 9 / 12 / 19 | 44 / 59 / 95 |

So wherever a plant lasted, between a tenth and nearly all of its cells now stand out of the water (the Fern died out on planet 77, and the Flower was down to one cell there). They are plants of the wet margin, as the first land plants were: none was measured far from water, and nothing here yet makes soil or brings water up a stem.

**Land animals.** At the water's edge all five (Reptile, Dinosaur, Bird, Mammal, Ape) stand a tenth to a quarter under water and move there by muscle; the pond table above has their speeds. On the planets the Reptile lasted on four of six (72 cells on one, 24 of them holding ground out of the water), the Dinosaur and the Ape on three, and the Bird and the Mammal on none. The warm-blooded ones spend the most and go first. The Trilobite walks out too: on planet 23, 41 of its 346 cells were holding ground out of the water.

### Carnivores, a crustacean, muscle's early use, and Earth's order

**Carnivores.** The Ordovician test asks for carnivores at one body in two hundred. Three things were found.

- *The count was wrong, and fixing it changed little.* Prey was counted as animal only if the bitten cell had itself eaten, so a Fish swallowing an Amphibian's tail, or anything eating a Trilobite, counted as plant-eating. Prey is now judged by the body it belongs to: an animal is a body that lives by eating, or a many-celled body that does not feed itself. The same six planets, run again and ending with the same count of every creature, went from 0.1 to 0.2% carnivores to 0.1 to 0.3%.
- *The pass mark is fair.* Eaters were 7 to 8% of bodies on the two planets checked, and a tenth of a feeding level is the usual share of the one above it: about 0.7%. A suggestion made earlier in the day to lower the mark to 0.1% was wrong and is withdrawn.
- *Why there are so few.* Every eater on two planets took 5 to 17% of its prey from other eaters, the same as their share of what is there to eat: nothing chooses animal prey. And the easy meal is plankton. Every kind of light-feeder has evolved down to a tiny soft cell (median 0.08 of tissue in each), while animals' cells are 0.2 to 1.5 and carry spines, mineral or a body's grip. Earth's plankton is walled and shelled and its grazers are many-celled; here the plankton is naked and the grazers were single cells.

**A crustacean.** Caden, 2026-10-12: "what about crustaceans, do we have those?" There was none. The Shrimp is the 49th starter: a short jointed body in a thin shell, raised to about 25 cells (6 on the middle line, 8 and 11 on the sides), with mouths along its limbs that sweep in drifting cells. It swims by muscle (cells moving at 0.17 a tick) and eats. It is meant as the missing link in the middle: a small many-celled grazer for the Fish and the Sea scorpion to live on, as krill are.

**Muscle's early use.** A body that beats now stirs the water round its mouths, and they draw up to twice as much (half of that at 0.01 of muscle work a cell-tick). Before, muscle did nothing for a body without fins, so nothing kept it. Six bare planets, 2,400 cycles, against the same planets without it:

| Six bare planets | Without | With stirring |
| --- | --- | --- |
| Bodies carrying muscle | 11, 0, 6, 18, 37, 45 | 37, 0, 1, 2, 69, 52 |
| With nerve too | 1, 0, 0, 0, 1, 8 | 1, 0, 1, 0, 4, 13 |
| With bristles as well | none | 0, 0, 0, 0, 0, 3 |
| Muscle established | 1 planet | 2 planets |
| Movement by muscle | the gene only, on all six | working on 1 planet, not established |

The second planet has no muscle in either run and came out identical, which is the check that nothing else differed. Three planets gained and two lost, so this is a lean, not a proof. It is the first of these runs in which a bare planet had bodies moving by muscle.

**Earth's order.** One run released each period's cast in turn on a bare planet, a period every 100 cycles. It came out worse than releasing everything at once: all six planets ended in their Proterozoic. The cause was the Leviathan, a very large single cell built to swallow hunters, added to the Cambrian cast for this run. It reached 254 to 601 cells on every planet and was still rising; Hydra, Sea scorpion, Dragonfly and Dinosaur were gone from all six and movement by muscle fell back to the gene alone on three. It did not even count as a carnivore: most of what it ate was plankton. It has been taken out of that cast, and the run has to be repeated before anything can be said about the order of release.

A cell limit also turned up here. Three of the bare planets came within a few dozen cells of their limit of 16,000 in mid-run, so later runs use 24,000.

**The whole cast with the Shrimp, stirring and room** (six planets, 1,400 cycles, limit 24,000, everything released at cycle 200):

| | Result |
| --- | --- |
| In their Cambrian by the app's tests | 4 of 6 (planets 5, 77, 8042, 11) |
| Movement by muscle established | 5 of 6; on planet 5 a quarter of all many-celled bodies move by muscle |
| Trilobite | 1,391, 908, 5, 984, 3, 112 cells |
| Shrimp | 26, 182, 1, 8, 0, 5 |
| Amphibian | 6, 11, 140, 16, 2, 102: alive on all six for the first time |
| Fish, Dragonfly | alive on 4 each (Fish 143 on one) |
| Sea scorpion | 173 cells on one planet, 1 on another |
| Worm, Mammal | 2 each; Reptile, Bird, Ape 1; Dinosaur 0 |
| Plants | all five on all six; a Fern stand of 623 cells on one planet |
| Carnivores | 0.0 to 0.3% of bodies: unchanged |
| Later tests met on their own | Silurian 6, Triassic 6, Neogene 6, Devonian 5, Cretaceous 3, Jurassic 2, Permian 1, Paleogene 1, Quaternary 1 |

On planet 404 every test from the Silurian to the Quaternary was met except the Carboniferous and the Permian, and the planet is still shown in its Proterozoic, because its animals fell just short of one many-celled body in twenty (4.4%).

**What an animal is, in the count.** The code counted a body as an animal only if most of its cells had mouths or enzymes. That was written when the only animals were balls of mouths, and it left out every animal with a head and a body behind it. The app has always told the player an animal is "a body of four cells or more that lives by eating other life", and the code now says the same. On the six planets above it moved one planet's animals from just under the mark to just over.

**Why the Shrimp did not make carnivores, and what would.** A planned animal here is about 25 cells of 0.2 each: some sixty times the weight of a plankton cell. On Earth a fish is millions of times the weight of the plankton, cannot hold it, and so has to eat what does. Here every animal is the size of a rotifer and plankton is its natural food. A real third link in the chain needs bodies of hundreds or thousands of cells whose mouths cannot take anything as small as plankton, on planets with the room to hold them. That is a matter of scale, and it is not built.

**Built afterwards from reading, and not yet measured.** Caden asked for research in place of more runs. What it led to is in the last section of `docs/EARLY_EARTH.md`: there is a Copepod to graze in numbers and be eaten, eggs are smaller and land eggs carry yolk, touching walled cells trade across their shared wall (with a Fungus starter), and water creeps up a walled body. Whether any of it moves the numbers in this document is not known.

### One planet four times the size, counted honestly (2026-10-12, evening)

Caden said "run it". One planet of size 256 (four times the surface of the planets above), a limit of 64,000 cells, seed 5, star at 6, vents at 4, mutation 2, sharing 1. A bare planet with 64 founders, then each period's cast in Earth's order, 64 founders or the grown bodies they make, the first after nothing and one more every 100 cycles; 1,800 cycles in all. It took an hour. The Copepod and the Fungus were in the cast for the first time.

**The later tests had been passing on microbes.** At cycle 200, with nothing on the planet but four kinds of bacteria, the count showed 115 warm-blooded cells, 16 cells of land animal, 4 fliers and a land body of 35 cells. The counts took any cell that had built the protein, wherever it was:

- *warm-blooded* was any cell with the warmth protein, in the sea or out of it;
- *land animal* was any cell out of the water that did not feed on light, so bacteria on a wet shore;
- *flier* was any cell off the ground with the wing protein, which included a mat's cells on their stalks;
- *land plant* included mats of bacteria with a holdfast, which have no wall and were on shores long before plants;
- *the largest land body* was whatever was anchored out of the water, usually a mat.

So the line in the table above, "Later tests met on their own: Silurian 6, Triassic 6, Neogene 6, Devonian 5 ...", is withdrawn, and so is every earlier statement in this document that a planet met the Devonian, Permian, Triassic, Jurassic or Paleogene test. The counts are now (`Habitat::ashore_view`): an animal is a body of four cells or more with a mouth, most of whose cells do not feed on light; a land animal is one of those standing on the ground out of the water and not fixed to it (a coral left by the tide has not come ashore); warm-blooded cells, fliers and the largest land animal are counted among animals only; a land plant has a wall. The app's timeline says the same in its own words.

**What the planet did**, with those counts applied to the saved planet at each step:

| Cycle (cast just added) | Cells | Walled plant ashore | of it branching | flowering | grass | Land animal | dry-skinned | largest | Warm-blooded | in the air |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 800 (Tree, Dragonfly) | 43,801 | 432 | 169 | 0 | 3 | 2 | 0 | 5 | 0 | 0 |
| 1,000 (Dinosaur) | 41,151 | 401 | 82 | 14 | 22 | 0 | 0 | 0 | 16 | 0 |
| 1,200 (Flower) | 31,099 | 742 | 260 | 192 | 50 | 21 | 0 | 23 | 11 | 0 |
| 1,400 (Grass) | 28,538 | 1,191 | 295 | 178 | 371 | 1 | 0 | 5 | 17 | 0 |
| 1,600 | 25,492 | 909 | 221 | 178 | 263 | 0 | 0 | 0 | 6 | 0 |
| 1,800 | 24,677 | 940 | 203 | 105 | 281 | 4 | 0 | 9 | 6 | 0 |

- **By the app's tests the planet ends in its Proterozoic.** The Cambrian test fails on two steps. Animals reached one many-celled body in twenty from cycle 1,100 to 1,400 and ended at four fifths of that. Movement by muscle was established at cycle 400, just after the Cambrian animals went in, and at no sample after, as the animals that had it died.
- **Plants came ashore and stayed**: all five kinds alive at the end, about a thousand walled cells out of the water, a fifth of them branching. The plant halves of the Silurian, Devonian, Carboniferous, Cretaceous and Neogene tests are met.
- **No animal lives on land.** At most 21 cells of animal stood on dry ground at any sample, none with a waterproof skin, and the largest such body was 23 cells. Nothing flew. Warm-blooded cells peaked at 24. So the Devonian's animal half and the Carboniferous, Permian, Triassic (met by a few cells for a while), Jurassic, Paleogene and Quaternary tests are not met.
- **Carnivores: 0.1 to 0.3% of bodies** against the 0.5% the Ordovician test asks, the same as on every planet before. Every eater took 3 to 17% of its prey from animals.
- **Every planned animal but one died out.** Cells of each at the cycle it was added and at each 200 after:

| Animal | Added | +100 | +300 | +500 | End |
| --- | --- | --- | --- | --- | --- |
| Hydra | 300 | 305 | 42 | 19 | 0 |
| Jelly | 300 | 169 | 36 | 29 | 17 |
| Worm | 300 | 8 | 0 | | 0 |
| Trilobite | 300 | 150 | 80 | 59 | 215 |
| Shrimp | 300 | 66 | 54 | 55 | 0 |
| Copepod | 300 | 79 | 24 | 9 | 0 |
| Sea scorpion | 400 | 45 | 2 | 5 | 0 |
| Fish | 500 | 83 | 0 | | 0 |
| Amphibian | 600 | 69 | 39 | 58 | 0 |
| Dragonfly | 700 | 16 | 10 | 1 | 0 |
| Reptile | 800 | 72 | 26 | 10 | 0 |
| Dinosaur | 900 | 31 | 17 | 10 | 0 |
| Bird | 1,000 | 32 | 25 | 4 | 0 |
| Mammal | 1,200 | 42 | 11 | 3 | 5 |
| Ape | 1,400 | 9 | 0 | | 0 |

- **What throve**: the single-celled eaters (Paramecium 1,339 at the end, Grazer 843, Hunter 721), the fixed filter-feeders (Sponge up to 1,249, Dickinsonia 1,692, Coral 699), and the Trilobite, which is armoured, skinned and has no soft mouth.
- **The mechanisms that had never been on a planet.** The Copepod and smaller eggs did not help: Copepods fell from 79 cells to none in 700 cycles. The Fungus fell from 314 to 2. Water creeping up walled bodies and yolk in land eggs cannot be told apart from the rest in this run; plants did stand ashore in numbers, and no land animal bred. The changes to feeding (one prey at a time, mouth size) and to warm blood were in the engine throughout and the outcome above is with them.

**Why the animals died: they were eaten alive by single cells.** The engine now records what every cell died of and who took its tissue. Of the cells of grown Fish, Amphibians, Reptiles, Sea scorpions and Worms that died in fifty cycles, four in five had been swallowed or digested, and the eaters were Paramecium, Grazer, Hunter and Sponge. The fault, what was read about it and what was changed are in `docs/EARLY_EARTH.md`, "A fifth round". In short: a body's hold on its cells was divided by how much bigger the eater's cell was than the prey's cell, and these animals' cells are small; a body is now a refuge from any mouth that cannot take all of it, tissue matrix slows enzymes, and a mouth of many cells needs muscle to open. Over the same fifty cycles the new rules halved the tissue taken and left 12 to 136% more of each animal alive.

**The same planet again, with a body as a refuge.** Same seed and settings, the engine with the three changes of "A fifth round" and the honest counts built in; the strainer rule of "A sixth round" was written after this run began and is not in it. Cells of each animal at the same cycle, first run and second:

| Animal | Cycle 800 | 1,200 | 1,800 |
| --- | --- | --- | --- |
| Trilobite | 59 → 287 | 168 → 1,104 | 215 → 1,572 |
| Sea scorpion | 8 → 78 | 0 → 235 | 0 → 246 |
| Worm | 0 → 6 | 0 → 85 | 0 → 74 |
| Dragonfly | 16 → 36 | 1 → 81 | 0 → 121 |
| Shrimp | 55 → 15 | 0 → 3 | 0 → 64 |
| Copepod | 9 → 29 | 0 → 103 | 0 → 2 |
| Hydra | 19 → 20 | 0 → 16 | 0 → 23 |
| Reptile | | 14 → 36 | 0 → 11 |
| Dinosaur | | 17 → 35 | 0 → 3 |
| Ape | | | 0 → 11 |
| Jelly | 29 → 63 | 23 → 13 | 17 → 0 |
| Fish | 0 → 33 | 0 → 5 | 0 → 0 |
| Amphibian | 59 → 26 | 48 → 32 | 0 → 0 |
| Bird | | 18 → 36 | 0 → 0 |
| Mammal | | | 5 → 0 |

- **The planet reaches its Cambrian at cycle 900 and is still in it at 1,800**, ten samples in a row: bodies, animals, hunters, defences, senses and movement by muscle all established. The first run never left its Proterozoic.
- **Movement by muscle is established and growing**: the share of many-celled bodies moving by muscle went from 2.2% at cycle 400 to 9.1% at the end, and it was established at every sample but one. The Worm, which had never lasted on any planet, reached 200 cells.
- **The Ordovician test still fails.** Carnivores are 0.1 to 0.2% of bodies. Skeletons are in 7.8% of what the step counts and it asks for 10%; that share rose all run as the Trilobite spread.
- **The later tests, by honest counts, each taken alone.** Met at the end: Silurian (1,586 cells of walled plant ashore), Cretaceous (93 flowering), Neogene (417 of grass). Met for a while and lost: Devonian (ten or more cells of animal standing on land at seven samples, 49 at most, 8 at the end), Carboniferous (a forest and three or more cells in the air at cycles 1,200, 1,300 and 1,500, the first fliers any honest count has shown), Triassic (ten or more warm-blooded cells from cycle 1,000 to 1,700, 37 at most), Quaternary (fifty or more learning junctions between warm-blooded cells from 1,000 to 1,400, in Dinosaurs). Never met: Permian (6 cells of dry-skinned animal on land at most, 50 asked), Jurassic (the largest land animal was 23 cells, 30 asked), Paleogene (37 warm-blooded cells at most, 100 asked).
- **Fish, Amphibian, Bird, Mammal and Jelly still die out.** The last Fish, Birds and Mammals to die were digested by Grazers or starved.
- **The whole planet shrank late on**, as it had in the first run: from 45,000 cells at cycle 1,100 to 20,000 at the end, with free oxygen falling from 0.82 to 0.20. Grazers trebled after cycle 1,200 (664 to 1,902 cells) and the bacteria that make nitrogen usable fell from 8,386 cells to 481, nearly all of them digested, two fifths by Grazers. There was no shortage of nitrogen in the water. The single cells that eat Grazers were fewer than in the first run (Paramecium 514 against 1,339), which is what losing the animals as food would do to them. Not looked into further; one planet.

This is one seed run twice. It shows which way the change goes and not how far on another planet.

**Many-celled animals eating each other.** Sixty more cycles from the end of that planet, read by the record of who took whose tissue:

| Animal (cells) | Tissue it took | From other animals | What it took most of |
| --- | --- | --- | --- |
| Sea scorpion (246) | 25.7 | 57% | Trilobite 40%, bacteria 30%, its own kind 6%, Hunter 6%, Grazer 5% |
| Trilobite (1,572) | 24.1 | 58% | bacteria 29%, Sea scorpion 20%, Grazer 16%, Paramecium 9%, its own kind 8%, Hydra 5% |
| Worm (74) | 29.9 | 7% | bacteria 56%, Fern 16%, Tree 6% |
| Shrimp (64) | 3.1 | 43% | bacteria 46%, Trilobite 42%, Fern 11% |

- The two commonest many-celled animals live largely on each other. Half of what was taken from Trilobites went to Sea scorpions, which swallow them (141 of the 283 Trilobite cells that died were swallowed); half of what was taken from Sea scorpions went to Trilobites, which have no mouth and digest them from outside (63 of the 98 Sea scorpion cells that died were digested).
- Both move by muscle: bodies with working muscle were 10.3% of many-celled bodies and movement by muscle 10.9%, and the muscle step itself was established in this stretch, which it had not been at any sample of the run.
- Neither counts as a carnivore. The step asks for seven tenths of a body's prey to be animal and they take a little under six, so they are counted as eating both. Carnivores stay at 0.1% of bodies. Whether seven tenths, counted over every body on the planet single cells included, is the right mark for a planet whose hunters are some sixty times the weight of most of what lives there is a question for Caden and has not been touched.
- The Worm was taking a fifth of its food from land plants.

**Two Ordovician tests changed, on the record.** Caden was asked whether the carnivore step should count omnivores and said to go by what is known. What is known:

- *Meat-eaters.* In 58 real food webs, above the plant-eaters "food webs are better characterized as a tangled web of omnivores", most of all in the sea; things that eat animals mostly eat at more than one level. The Ordovician's own mark is that animals ate animals (nautiloids the chief hunters, a race between shell-crushers and armour), not that anything ate animals only. The step asked for bodies taking seven tenths of their prey from animals. It now counts every body that takes more than three tenths, omnivores with carnivores, and is called Meat-eaters. The pass mark is where it was, one body in two hundred.
- *Skeletons.* The step asked for one living cell in ten, of every kind, to carry mineral. Earth has never met that: most of its cells are bacteria and plants. Hard parts are counted among animals, and even there they were a minority when they were new: in the Burgess Shale about 15% of genera and under 5% of individuals had them. The step now counts the cells of animal-like bodies only, and asks for one in ten of those.

Under the old tests the second big planet ended with carnivores at 0.1% and skeletons at 7.8%. Under the new ones it has meat-eaters at 0.7% of bodies and skeleton in 16.3% of its animal cells, and both steps are established. The first big planet passes both as well (1.1% and 18.9%) and still fails the Cambrian test before them, on animals and on movement by muscle. No engine rule changed here, only what is counted.

**So by the app's tests the second big planet's last saved moment reads Devonian** (a third run, below, did not repeat it)**.** Cambrian, Ordovician, Silurian (1,591 cells of walled plant ashore) and Devonian (391 of branching plant, and 10 cells of animal standing on land, which is exactly the number asked and was between 3 and 49 through the run). The Carboniferous test fails: nothing is in the air. This is the final saved state read with the engine as it is now; the planet was not run again.

**A third run, from nothing, with everything above in the engine.** Same planet and seed, with soil, spores on the wind, the strainer rule, the corrected counts and the two changed tests all in from the start. It did not repeat the Devonian reading.

- It reached its Cambrian at cycle 900 and held it to 1,400, six samples, then fell back to its Proterozoic at 1,500 and stayed there: animals dropped under one many-celled body in twenty and ended at two fifths of that.
- Meat-eaters were established from cycle 700 to the end (0.8% of bodies at the end). Skeleton was established at cycles 500 to 800 and again at 1,700 and 1,800 (14.1% of animal cells), and under its mark between.
- Movement by muscle was established at every sample from cycle 400.
- On land: 9 to 47 cells of animal at every sample from cycle 500, up to 24 with a dry skin, the largest 26 cells; warm-blooded cells reached 93, far above either earlier run; one or two cells in the air at most. Walled plant ashore reached 1,645 cells, 582 on ground with no open water.
- The animals at the end were different from the second run's: Trilobite 327 cells where there had been 1,572, Sea scorpion 128 for 246, no Worm, and Fish, Amphibian, Bird and Mammal each still alive in small numbers where the second run had lost them.

So the Devonian reading above is one planet's last moment and not where the sim stands. What two runs out of three now show is a planet that reaches its Cambrian and holds it for hundreds of cycles, with muscle and meat-eaters established, and that does not hold anything later. The same seed run on three versions of the engine gave three different histories, which is the reason no single run should be read as a result.

**Why most of an animal's cells are loose single cells.** Caden, 2026-10-13: "once a thing is a trilobite with all its components ... it should stay together. The only thing left for it is to grow, mate, eat, or die." At the end of the second run the Trilobite was 785 separate bodies and 675 of them were single cells; the Sea scorpion 133 and 112. Those cells were looked at one by one. They were not fragments and had lost no genes: every one still carried the genes for glue, matrix, dividing and size. They were egg-sized (1.7 times an ordinary cell where a cell in a body is 0.3), well fed, a median of 228 cycles old, and had not divided once since coming loose. They are young that never finish. An animal's young here is a small cell that comes off the end of a limb and has to feed itself up to an egg's size, alone and among true single-celled eaters, before it can become a body, and most stall there. So each kind piles up as loose cells that are neither a body nor dead.

*Tried first and taken out: ties that hold for life.* A tie with matrix tended on both sides was made never to let go by chance. Over 150 cycles on that planet nothing changed (Trilobite single cells 570 and 568, bodies of four or more 79 and 75), and it stopped the Trilobite shedding young and starved the Worm in their own tests. An animal here breeds by its ties letting go, so a body cannot be made to hold together before its young have another way to leave.

*The egg, tried three ways and not kept.* Caden: "do the egg stage ... after something hatches out of an egg it is a complete organism and it eats smaller molecules and organisms, as it eats it grows." A loose young cell needs 0.58 to 1.1 of tissue before it can begin dividing into a body and leaves its parent with about 0.25, because the cell at the end of a limb does not grow while it has a neighbour. Making that cell large on the parent filled it (to 1.0 to 2.5) but left it needing more still once loose; making it large only while attached ended the pile of stalled young in a Trilobite pond (none, where there had been 14) with fewer cells in all (22 for 54); and reading the same genes only inside a body killed the Trilobite by cycle 300 and the Worm by cycle 100. All three are out.

*Why, measured.* A grown Trilobite in that pond takes in almost exactly what it burns for its first 200 cycles (0.0043 of fuel a cell-cycle against 0.0039, then 0.0071 against 0.0069, its energy falling to a twentieth of full). It has nothing to fill an egg with, and a large egg on a limb starves it. Once it is on good food (0.052 against 0.029) it breeds under the rules as they are: 54 cells and a second generation of bodies by cycle 400. So feeding comes before eggs.

*And why it only breaks even: it cannot go and find food.* Four grown animals with mouths, each in the same pond (Fish, Shrimp, Worm, Amphibian). Their mouths were busy with something between none and a quarter of the time. Each mouth had one to six cells of other kinds within a unit and a half of it, the reach of its feeding current, and usually none in touch. Where an Amphibian had thirteen to fifteen within reach its mouths were busy half the time and it took in 0.021 against 0.013 burned. The fastest body in each pond moved 0.04 to 0.13 units a cycle: prey drifts to these animals faster than they travel to it. So the limit is not holding or digesting a meal, and a gut would not lift it. It is travel. The Worm, the one built first and purely for muscle, is the one that breeds here: 17 cells to 107 in 400 cycles, five bodies of ten or more.

*One stroke, traced.* A grown Worm, twelve cells and 2.6 units long, swimming alone for forty cycles, went 1.32 units: a little over one part in a hundred of its own length at each beat. A grown Fish, thirty-six cells and 9.2 units long, went 2.35 units, 0.6% of its length a cycle, for twelve times the Worm's muscle work. In both, every muscle pulls on the same tick, and the nerve fires at the head and at the tail on the same tick: the whole body squeezes and lets go at once. Nothing passes down the body and nothing alternates from side to side. In water a squeeze and its undoing carry a body nowhere, and what these gain is only what their fins grip on the back-stroke. A real fish makes about half its length at a beat. This is what "moves by muscle" has meant in every count in this document.

*Agreed with Caden, not built:* for animals the unit becomes a piece of tissue and not a cell, built in this order (movement that travels first, as just found: muscle in opposing pairs against bone in bodies large enough to row, as in the soft robots Caden pointed to; then feeding). A gut that holds what is swallowed and feeds every part, so that a grown animal takes in well over what it burns. An egg the parent fills before it leaves, which becomes a small complete animal without feeding. Parts with real size, so an animal of thirty parts is hundreds of times the bulk of what it eats. A part torn off an animal dies; only an egg starts a body. Then the cast re-sized to fit.

### Not built

- Sulfur and methane metabolisms, and photosynthesis without oxygen.
- A magnetic field. Oxygen as a poison.
- Branching inside a body (gills, lungs, vessels): the branching rule shapes the whole body only.
- Shells of calcium carbonate: all skeleton is still drawn from the same dissolved mineral that supplies phosphorus.
- Eggs as a stage of their own, pollination by animals, grazing that grass regrows from, herds, tools.
- The great dyings as events of their own: strikes and eruptions exist, a flood-basalt age does not.
- Strict carnivores as an established part of a food web (bodies living on animals and little else are still 0.1% of bodies; the Ordovician test, which now counts omnivores with them, is met on the second big planet). See "Carnivores" above for why, and the fifth and sixth rounds in `docs/EARLY_EARTH.md` for what was tried on 2026-10-12.
- Animals that live on land. With honest counts at most 49 cells of animal stood on dry ground on the best planet, and none bred there. What would take them there (food worth leaving the water for, and young that do not need it) has not been worked out.
- Anything that keeps grazers from eating the planet's light-feeders down.
- A way out of the water for an animal. The Dragonfly has wings and the muscle to beat them, and a wing lifts only a body that is out of the water and moving; set down in a pond it never leaves the water. Nothing here has a young stage in the water and a grown one in the air, and no grown animal crawls ashore to feed. The Carboniferous test (something on the wing) and every test after it that asks for animals living on land wait on this.
- A way for a grown animal's effort to follow its hunger. Its pacemaker beats nearly as hard when it is fed, and in a pond of drifting prey swimming finds it no more food than staying put, so muscle is a third of what it spends. Only the Worm and the Sea scorpion hold their fuel in a pond. Tried and taken out: muscle that tires as its cells' fuel runs low (full pull above a quarter of a full store). Over 400 cycles in a pond the Worm and the Sea scorpion then died out and most of the others stopped pulling and sat still, though the land animals kept more cells (Ape 34 against 1, Dinosaur 16 against 5). No gene can read a cell's fuel store, only its energy, which stays full until the fuel is gone; that is the missing sense.
- Eggs with yolk. A larva is a lone cell that has to eat its way up to an egg's size before it can cleave into a body, which takes it about 120 cycles; a body leaves about one young that lives, so a line holds its numbers and does not grow.
- Roots that carry water, and a spore that stays a spore until it has travelled. Soil is built (remains hold rain; see the seventh round in `docs/EARLY_EARTH.md`): with it, plant three or more sites from open water went from 73 cells to 170 on one planet in 300 cycles.
- Whether the planned animals' gills, pump and transport each pull their weight was measured only together (breathing 0.9 to 1.0 on the middle line), not one at a time.

## What the periods say about the build order

1. **Oxygen comes before animals, and animals before armour.** The engine now has that order: breathing pays fifteen times fermenting, and a hunter cannot pay its way without it.
2. **The Cambrian was about muscle.** Cilia move a small thing well and a large thing barely at all; muscle is what let animals be large and fast. Until 2026-10-12 muscle here moved a body a fiftieth as fast as cilia (see `docs/EARLY_EARTH.md`), so nothing could live by it.
3. **Arms races need both sides.** Eyes, armour and speed only pay when there is something to see, something biting, and something to catch. The feeding current here still reaches farther than anything can flee, so speed buys little.
4. **Land waits for a skin and a skeleton**, and then for plants to make it worth going to.
5. **Everything after the Devonian is about land**: holding water, holding heat, standing up, flying. Each now has a first-pass mechanism and none has been measured.

## Sources

- [Macmillan, *Life*, ch. 24: groups that arose in the Cambrian](https://digfir-published.macmillanusa.com/life11e/life11e_ch24_19.html)
- [Swedish Museum of Natural History, Fossils and evolution](https://utstallning.nrm.se/fossilochevolution/en/startpagefossilsandevolution.4.4f4d16d8151338bf6039030f.html)
- [Te Ara, Fossils](https://teara.govt.nz/en/fossils/page-2)
- [UC Museum of Paleontology, the Proterozoic](https://ucmp.berkeley.edu/precambrian/proterozoic.php)
- [Polish Geological Institute, Earth's history](https://www.pgi.gov.pl/muzeum/kopalnia-wiedzy-1/15065-earth-s-history.html)
- [University of Maryland, GEOL 102 notes](https://www.geol.umd.edu/~tholtz/G102/102x2r17.pdf)
- [Valdosta State University, the Mesozoic](https://fossils.valdosta.edu/era_mesozoic.html)
- [Scientific American / Nature, What sparked the Cambrian explosion?](https://www.scientificamerican.com/article/what-sparked-the-cambrian-explosion1)
- [The Great Ordovician Biodiversification Event, overview](https://en.wikipedia.org/wiki/Great_Ordovician_Biodiversification_Event)
- [Servais and others 2010, the palaeoecological dimension of the Ordovician radiation](https://cris.fau.de/publications/109887404/)
- [Thompson and others 2007, the prevalence of omnivory in real food webs](https://researchprofiles.canberra.edu.au/en/publications/trophic-levels-and-trophic-tangles-the-prevalence-of-omnivory-in-/)
- [The Open University, the Burgess Shale: how few of its animals had hard parts](https://www.open.edu/openlearn/nature-environment/natural-history/life-the-palaeozoic/content-section-1.2)

The four above were read as search summaries on 2026-10-12, not checked against the papers.
