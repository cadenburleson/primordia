# Creator powers, life's ascent and speed

This revision turns the observatory into something you steer. The physics is unchanged and nothing is faked: genes still arise only through copying errors at division, and every tissue is still built from real material and energy. What changed is that the player can now decide how fast life varies and which variations survive, and can see what the world has actually built.

## Why this was needed

Measured on the previous build with ordinary founders:

- **Max speed was throttled.** The worker ran up to 6 ms of work and then slept 35 ms. A 64-founder world ran at about 5 cycles per second in the browser while the same engine ran far faster natively.
- **Evolution stalled.** After 3,000 cycles, 97% of cells still carried only the four founder genes. Muscle was in 0% of cells.
- **Every world slowly starved.** About 98% of deaths were starvation. Founders live on the starting fuel; photosynthesis at the default brightness returns roughly a third of a cell's upkeep. Once local fuel was eaten, populations fell to 100–200 cells and kept declining.
- **Nothing could be steered.** The habitat `mutation` setting reached only the legacy engine.

## Speed

- `src/sim/worker.ts`: at Max the worker runs back-to-back 14 ms batches and yields with a message to itself, so commands are still handled between batches. Paced speeds idle only when they are caught up.
- `scripts/build-wasm.mjs`: the shipped WASM is now built with the `release` profile. `npm run build:wasm:preview` keeps the fast-compiling profile for engine iteration.
- `engine/src/surface.rs`: static mesh geometry (edges, areas, normals, seeded ground) is built once per domain instead of every tick. Results are bit-identical.

Measured in the browser on the quick-start planet: 17–44 cycles per second with 200–560 cells (10–17 ms per step on a busy machine), against about 5 cycles per second before.

**Still slow:** step cost grows roughly linearly with living cells, and about 40% of it is body mechanics (20 substeps per tick, each evaluating terrain under every cell). A world at a 4,096-cell cap steps several times slower than one with a few hundred cells.

## Powers (`engine/src/steering.rs`)

Every command goes through `World.steer(json)` and is saved with the world. With default settings the engine is bit-identical to before; this is tested.

| Power | What it does |
| --- | --- |
| Pace of change | Multiplies every copying-error probability (0–10×). |
| Breed for a purpose | Every N cycles, scores each body on a goal and removes the lowest scorers, up to a fraction of the population. Goals: bodies, movement, support, senses, hunting. |
| Chosen lineage | Marks a body. Descendants inherit the mark; every other cell is removed at a small chance per cycle. |
| The star | Multiplies light reaching photosynthesis (0.25–8×). New planets default to Bright. |
| The oceans | Multiplies diffusion of fuel, waste, minerals and remains (0.25–8×). Chemical signals keep their range. |
| Rain of plenty | Adds fuel at inhabited sites. External input, accounted for like a brush stroke. |
| Mutation storm | A temporary multiplier on the pace of change. |
| Ice age | Lowers baseline temperature for a set time, then restores it. |
| Meteor strike | Removes cells near a clicked site and leaves a crater. Can be undone. |
| Great dying | Removes a random fraction of all cells. Can be undone. |

Removed cells are recycled through the normal death path, so matter and energy stay conserved.

**How breeding scores a body.** Each goal has a measured performance term (for movement: contraction work actually paid for, scaled by how far the body really travelled) and a smaller material term (the enabling protein, its expression, or merely carrying the gene). Performance dominates; the material term gives selection something to act on before a new tissue does useful work. A round removes nothing when no body differs, when fewer than 48 cells live, or when removal would push the population below 60% of its high-water mark. Breeding therefore harvests regrowth rather than shrinking the world, but a bred population still runs smaller than an unbred one.

**What breeding achieved.** Muscle and skeleton are scored only inside bodies, because a lone cell has nothing to pull on or hold up. (An earlier scoring credited lone cells: it drove muscle to 89% of cells in about 1,000 cycles, and dissolved every body while doing it.)

- Native, 128-unit patchy planet, Bright star, pace of change 3×, 1,500 cycles, breeding for movement: on one seed colonies rose to 99% of cells and powered movement was established (30% of bodies paying for contraction); on a second seed movement reached "working" but bodies stayed rare. The unbred control ended with muscle built in 2% of cells and no paid movement.
- Browser, quick-start planet: breeding for movement alone did almost nothing while most cells lived alone (2 of 286 bodies scored). Following the ladder instead, "Breed for bigger bodies" established colonies and multicellular bodies within about 1,200 cycles, and by cycle 2,760 specialised tissues and powered movement were established with ten cell types alive.

Under a bright star cells drift toward living alone, so the reliable order is the ladder's: bodies first, then movement, skeleton and senses. Results vary a great deal between seeds, because populations are a few hundred cells.

## Life's ascent

Each step is measured from living cells every two cycles. The first version had eleven: living cells, colonies, multicellular bodies, specialised tissues, senses, muscle, skeleton, nerve nets, powered movement, organs, sexual reproduction. There are now twenty-two; see the next section.

Each step has four stages: absent, gene in the pool, first working cells, established. A step is established only when cells have built the tissue, for example muscle in 10% of living cells. The first time a world reaches each stage is recorded in the save and announced once.

These are thresholds on measurements. "Organs" means bodies of eight or more cells running four or more tissue types; it does not mean an anatomically organised heart or gut. "Powered movement" means bodies whose cells have paid for contraction work; it does not mean a coordinated gait.

The panel also names what is holding life back, from deaths over a recent window: starvation points to the star, then the oceans, then a rain of plenty.

## Seeing it

- Cell types have distinct colours on the planet and in the specimen view, with a live legend: red muscle, ivory skeleton, blue senses and nerves, violet digestive, and so on (`src/sim/roles.ts`).
- The chosen lineage is tinted gold.

## Movement, the food web, vents and complex cells

A second round added six genes (38 in all) and four directional senses (32 promoter inputs). Founders carry none of them. Older saves load with the new slots padded with zeros, then evolve under the new rules.

| Gene | What a cell with it can do |
| --- | --- |
| Propulsion | Push itself along its heading. It swims in water and crawls at half strength on ground it touches. The water gets the opposite push, and the work against drag is paid in energy. |
| Steering | Turn its heading by left-minus-right differences it senses: dissolved fuel, light, the waste living cells give off, and remains. Each steering gene's own weights decide what attracts and what repels. It carries a crude chemical sense; receptors sharpen it. |
| Engulfing | Swallow a touching cell from another body, one cell at a time, taking its fuel store too. Size is living structure, not carried fuel: nothing against prey a quarter larger than the whole hunter, full speed at half again its size. A cell built for this grows to twice normal size before dividing, gives up 85% of its own feeding, and does not bond to cells that lack a mouth. |
| Venom | Wound any other body it touches. Skin blunts it. |
| Spines | Cut what an engulfer can take, and wound it for trying. |
| Chemosynthesis | Make fuel from waste using mineral chemistry, with no light. The energy comes from the planet's interior and is tracked like sunlight. |

Rules that hold for every weapon:

- **A body does not attack itself.** Contact digestion used to eat a cell's own attached neighbours; it, engulfing and venom now act only between bodies.
- **A weapon spares armed kin.** Kin are cells sharing at least four-fifths of their working genes. Without any immunity, weapons hit unattached relatives and a silent copy of the gene outcompeted a working one. With immunity for anyone carrying the weapon, the mouth spread as a shield until nothing was edible.
- **A body is a refuge from small hunters.** A cell held by its neighbours is hard for a hunter its own size to pull off and easy for one eight times its bulk; a body's interior cannot be reached at all.
- **Steering cells feel nearby life.** Within one field spacing they sense which side other bodies are on, and this counts for more than the far-off scent of waste, which mostly points at a cell's own colony.

**Complex cells.** Each time an engulfer swallows a light-feeder there is a 1% chance per feeding tick that it keeps the prey alive inside instead. The host then harvests light through its captive regardless of its own membrane, pays a small upkeep for it, and passes it to its daughters. This is one event from the origin of eukaryotes, not nuclei or organelle biology.

**Diets.** Every body is classified from what it has actually taken in. An eater takes a quarter or more of its fuel from other life: a herbivore if its prey mostly feed themselves, a carnivore if they are mostly eaters, an omnivore in between. Other bodies are light feeders, soup feeders (dissolved fuel), decomposers or vent feeders. Life shows the shares as a bar.

**New powers.** Breed for Swimmers or Defences; "Movement" is now "Muscle". The Deep sets how strongly the planet's interior feeds vents (Ember sea starts at Active). The ladder has 19 steps, adding swimming, guided swimming, hunters, carnivores, complex cells, guts, defences and reflex arcs (a sensing cell wired directly to a moving one).

**Breeding was reworked** after it was found to stunt every world it touched (about 100 cells against 1,200 unbred):

- A round now removes a share of the births since the last round, so the population keeps growing.
- A removed cell returns to the world as dissolved fuel rather than as remains only a decomposer can use.
- A low-scoring body larger than the remaining quota loses that many cells instead of escaping. Before, breeding for anything bred big bodies.
- Carrying a silent gene earns almost nothing; expressing it does.

### What was measured

- **Swimmers spread on their own.** Guided swimmers seeded into 8% of an unbred world reached 42–48% of cells in 1,500 cycles. Bred for, motors were built by every cell within 2,500 cycles on two native seeds, and by 445 of about 500 cells by cycle 1,620 in the browser, where they visibly spread across the planet.
- **Defences respond to breeding** in every run, reaching between a fifth and all of the cells depending on seed.
- **Hunters arise on their own and persist.** This took a second pass; see "Making hunters viable" below.
- **Complex cells** arise unprompted wherever hunters feed; see below.
- **Reflex arcs** reached "gene appeared" and no further.

### Making hunters viable

The first version left hunters under 1% of cells and dying out. A benchmark of three prey worlds seeded with swimming engulfers, instrumented tick by tick, found five separate causes. Each fix exposed the next:

1. **Colonies were immune.** Size was compared against the prey's whole body, so any attached cell was safe from a lone hunter. A hunter now swallows a cell.
2. **Lean hunters were smaller than fat prey**, because size counted carried fuel. It now counts living structure, and the cutoff is graded.
3. **Wounded hunters ate each other.** Injury halved their immunity to their own kind. Immunity now saturates.
4. **The mouth became a shield.** With immunity for any mouth-carrier, the neighbourhood armed itself and nothing was edible. Immunity is now for kin only.
5. **Hunters wiped out their prey**, then stagnated. The body refuge above lets prey persist.

Measured afterwards, native, 128-unit patchy planets, Bright star, pace of change 3×:

- **Unbred:** hunters arose within about 500 cycles in all three worlds and were still feeding 3,500 cycles later. Between 1 and 26 cells at a time lived mainly on prey, about 1% of bodies. Herbivores appeared in every run; an omnivore and a carnivore appeared in separate runs.
- **Mouths outnumber hunters.** Up to 40% of cells carried a mouth while about 1% hunted, because a mouth also protects a cell from its armed kin.
- **Breeding for hunters** originally made things worse by culling the prey they need. It now judges only bodies that carry a weapon, on what they catch, and never removes the unarmed. Bred worlds kept 1–8 mouths with most of them feeding. Hunters still died out in one of three bred runs.
- **Browser, quick-start planet, pace of change Restless, no breeding:** "Hunters established" by about cycle 1,400; herbivores at 2% and the first omnivores by 2,340; complex cells established (14% of cells) by 2,850. At cycle 3,900 twelve cell types were alive, with mouths, venom, digesters and mineral armour all common.

About one hunting body in a hundred is what these worlds sustain, so the ladder counts hunters as established at 1% of bodies. Predation pushed prey toward living as bodies in some runs and toward armour and venom in others.

### Guts

A mouth is plumbed into the body it belongs to. Fuel runs across a mouth cell's bonds toward whichever side is emptier, fast: out to its neighbours after a meal, and back in when there is nothing to catch. Carrying it deeper than a mouth's own neighbours still takes transport tissue. A receiving cell is never filled past its capacity, and the Evidence test that switches transport off switches this off too.

Two things that were destroying hunter bodies were fixed along the way:

- **A mouth could not bond to its own unarmed kin**, because of the rule that stops it gluing itself to prey. That rule now applies to strangers only.
- **A body's motors tore it apart.** One cell's thrust stretched its bonds far past breaking. A body's thrust is now pooled and shared by mass; motors pointing different ways cancel instead of shearing the body.

The ladder gains a Guts step: a body of four or more whose mouths pass at least a fifth of everything the body takes in to cells with no mouth. Inspect shows a body's mouth cells, its largest mouth, and how much its mouths have passed on.

**No gut-bearing body has evolved in any run.** The sharing itself works and is tested. The obstacle is upstream: hunters live as single cells. In one seeded world 108 of 109 mouth-bearing bodies were a single cell after 1,200 cycles, and bodies seeded as hunters had come apart within 300. Their prey are mostly single cells too, so a lone hunter loses nothing by being small and keeps its full exposure to light. Breeding for bigger bodies did not produce hunter bodies either, and shrank the worlds it was tried on. Mouth material also saturates in every cell of a lineage, so a body does not sort itself into mouth and non-mouth cells.

### From cell to body: fates, workers and walls

One rule now carries life from a cell to a body: parts that stay together and reproduce through one of their number become a whole. Three pieces make it work.

- **Bodies are clones.** Cells bond only to kin, so a body is one genome and has no cheats inside it.
- **Cell fates.** A cell on its own is a stem cell and reads its whole genome. Inside a body, a stem cell whose commitment gene is driven past half switches to one of three fates and keeps it. Every gene carries a mask of the fates it is read in, so one genome can build different cells in different places. A committed cell's daughters inherit its fate while they stay attached; a cell cut loose is a stem cell again and can found a new body. Machinery whose gene is silent is taken apart about eight times faster, so a committed cell really loses what it no longer makes.
- **Walls.** A walled cell cannot be swallowed whole and stays cemented to its walled sisters without glue. It cannot swallow or swim. Enzymes still digest it, so grazers and decomposers are what eat walled life.

Three commitment genes and the wall bring the vocabulary to 42. The ladder gains Cell fates, Workers and breeders (a body of four or more with a dividing stem cell and a committed cell that has given up dividing) and Walled life: 22 steps.

Life shows a second bar, Kinds of life: simple cells, plant-like (walled, mostly fed by light or vents), fungus-like (walled, absorbing or digesting) and animal-like (bare, with a mouth or a motor). Inspect names what a body is, lists the organs inside its cells, and counts its cells by fate.

**What was measured** (three seeded worlds, native runs):

- At the natural pace of change, left alone or bred for bodies for 1,500 cycles, commitment genes spread to 1–5% of cells in five of six worlds and walls were built in three. One body committed all seven of its cells. No body with mixed fates evolved.
- At four times the pace of change for 2,000 cycles, unbred: commitment genes in 9–23% of cells, one 17-cell body with mixed fates, and a few plant-like cells in two worlds.
- At four times the pace and bred gently for bodies (strength 0.15): **walled life became established in two of three worlds**, 51% and 20% of all cells, nearly all of it plant-like (180 and 84 cells) with a few fungus-like. Walled sisters stay joined, so a wall is itself a road to a body. The third world produced the first bodies with workers and breeders (two, of seven and three cells).
- Given to every third body by hand, a body whose inner cells commit, stop dividing and run a motor steered toward light held its ground for 800 cycles: 17%, 6% and 13% of cells at the start, 20%, 5% and 10% at the end. Workers and breeders was established in two of the three worlds and reached first working bodies in the third.
- The same body with workers that have no job fell to 9%, 6% and 3%. A walled body with growing tips fell to 7%, 3% and 6%. A wall is a cost that only predators repay, and hunters are about one body in a hundred.

So division of labour pays once a worker has work to do. Unaided evolution reaches walled, plant-like bodies readily when the pace of change is raised and bodies are favoured; bodies with workers and breeders it reaches rarely and has not yet established, because a commitment gene that reads position, a masked gene and a job must arrive in one lineage. The gene editor (World, edit a body's genes) now sets which fates read each gene, so such a body can be built by hand.

### Muscle that gets somewhere

Measured first: a body pulsing its muscles made no headway in water. Drag on a cell was the same in every direction, so each stroke was undone by the return stroke (0.00003 units in 100 cycles, against 0.25 units in 6 cycles for one ciliated cell). Contraction also counted as strain and snapped the bonds doing the pulling. Meanwhile every cell in a body could carry a motor and thrust was pooled, so a big body swam as well as a small one and nothing needed muscle.

Five changes to the physics:

- **Fins** (gene 43, a new cell type). A fin slips easily along its cell's heading and catches the water, or the ground, when pushed back: eight times the resistance when fully built. It makes no thrust. It turns any back-and-forth stroke into travel.
- **Muscle is springy.** A bond between muscle cells is much stiffer, so it pulls hard and snaps back when it lets go.
- **A working muscle is not strained** by reaching the length it pulls to.
- **Motors work only on a body's surface.** A motor's thrust is scaled by how exposed its cell is, and a cell held in a body no longer tumbles. Big compact bodies outgrow cilia; a muscle's stroke grows with the body.
- **Excitable cells fire three times faster, and across a wider range.** A steadily stimulated cell beats about every 9 cycles instead of every 50. It used to beat only in a narrow band of stimulus and lock on above it; it now keeps beating across about a fourfold range, which covers a photoreceptor in ordinary light or a cell bathed in signal A.

With these, a row of four cells carrying muscle, excitable channels and fins, bathed in signal A, beats on its own and rows 0.37 units in 80 cycles; the same row without fins moves 0.002.

Powered movement on the ladder now means a body with a muscle at work, excitable cells to make it beat, and a fin to push with. Fins and feet is a new step (23 in all). Breeding for Muscle now scores ground covered by bodies with muscle and fins. Until a body manages it, being a body earns credit first, then the parts: muscle, channels, fins. A lone cell earns nothing, because muscle has nothing to pull on.

**Breeding now leans variation too.** While a breeding goal is set, half of all new genes are drawn from the kinds that goal needs (for Muscle: glue, muscle, signal A, excitable channels, fins). The gene still arrives by a copying error with random regulation and still has to be built and paid for. With no goal set, nothing changes.

**What was measured** (three seeded worlds, native runs):

- A "rower" genome (muscle, channels, signal A, fins in every cell) given to every third body by hand, unbred, 800 cycles: rowers persisted in all three worlds (muscle in 17%, 6% and 13% of cells at the start; 11%, 3% and 3% at the end) and Powered movement became established in two.
- Bred for Muscle at four times the pace of change for 2,000 cycles, before variation leaned: muscle and fins each reached a few percent of cells but never met in one working body.
- The same breeding with variation leaning toward the goal: muscle built in 5–8% of cells in all three worlds, fins in 9–20% (established in two), and the first working rowers in one world (8% of its bodies). Bred for bodies instead, commitment genes reached 24–73% of cells.
- In the browser, a fresh quick-start planet bred for Muscle at Restless had muscle and fins both established after about 2,200 cycles, but no working rower: the muscle was built and never fired. What sets the beat (signal A, or excitable channels with something to excite them) is now the missing part.
- Ciliated swimmers rose from 2–4% of cells to as much as 49% in some runs, because a body's motors now pull together instead of tumbling apart.

**Gene sharing** is a new dial in Powers (Off, Rare, Common, Rampant; off by default). When it is on, two touching cells now and then pass a copy of one gene across, as microbes do: the taker gains a gene the giver carries and it lacks. Genes that arose in separate lineages can then meet in one.

Bred for Muscle at four times the pace of change for 2,000 cycles, with bodies favoured first:

| | Muscle built | Fins built | Muscle bonds firing | Bodies of 3+ | Working rowers |
|---|---|---|---|---|---|
| Sharing off | 7%, 26%, 4% of cells | 19%, 13%, 9% | 7, 3, 0 | 11, 6, 19 | none |
| Sharing Common | 29%, 38%, 30% | 26%, 57%, 53% | 53, 4, 18 | 27, 3, 11 | first working bodies in one world |

With sharing on, muscle and fins became established in all three worlds and muscles fired inside bodies. Complete rowers (muscle, fins and beat-makers in one body) are still only a handful, and these worlds are small (200–340 cells).

### Bodies that last, grow to a shape, and work as levers

Measured first: in evolved worlds most cells carried no glue at all (2–13% of cells had any). Being attached cost a cell a sixth of its feeding per neighbour, so selection removed it. Bonds between glued cells lasted about 150 cycles, and a body of 20 lost one every dozen cycles.

- **Crowding, not contact, buries a cell.** A cell's open surface now falls with the square of how many neighbours it has: one neighbour costs 3%, four cost 44%, six bury it. Thin bodies feed almost freely; thick ones still need transport.
- **Strong glue holds.** A bond between two well-glued cells now lasts roughly a thousand cycles; a body that stops making glue still falls apart within a few dozen.
- **Aimed growth.** A cell with a polar scaffold already divided along its heading. A new regulatory gene, Sideways division (gene 44, builds nothing), swings that a quarter turn, so a body can widen as well as lengthen; with cell fates the two can be used in different parts of one body.
- **A heading belongs to the body.** When a body turns, the heading of every cell in it turns with it, so fins and growth stay aligned with the body instead of with the map.
- **Bone is a lever.** Joints between bony cells were about a thousand times weaker than muscle. They now hold their angle firmly, and muscle at the joint swings it.

Tested as supplied mechanisms: a beating row with feet crawls 0.35 units along the bed in 80 cycles (0.009 without feet). Two bones meeting at a joint with muscle and a beat-maker swing through 0.34 radians; with feet on the ends the limb strides 0.24 units in 80 cycles, and without feet it goes nowhere.

The ladder gains Shaped bodies (24 steps). Inspect gains a Body plan line: bone cells and joints, muscle, beat-makers, fins, fat, whether the body aims its growth, and what it is missing in order to move.

**What was measured** (three seeded worlds):

| | Cells living in bodies of 3+ | Largest body |
|---|---|---|
| Unbred, natural pace, 1,500 cycles, before | 12%, 2%, 4% | 14, 7, 12 |
| The same, after | 13%, 6%, 15% | 15, 22, 25 |
| Bred for Muscle, 4× pace, sharing on, 2,000 cycles, before | 50%, 5%, 26% | 21, 4, 7 |
| The same, after | 48%, 18%, 79% | 11, 20, 38 |

Bred for Muscle after these changes, **Powered movement became established in one world** (45% of its bodies; five complete rowers), the first time breeding alone has done it. The other two worlds had one rower and none. Bred for bodies, the sideways gene reached 12–17% of cells and the growth compass was built in 5–15%.

**Seasons** are a new dial in Powers (None, Mild, Harsh; off by default): the star dims toward midwinter and returns every 200 cycles. In the first test it did not do its job. Harsh seasons cut populations to about a quarter and no fat evolved, with or without them. Cells store fat only when they have a clear surplus, and in these worlds they almost never do.

### Speed

Terrain under each cell is now sampled on every fourth mechanics substep instead of all twenty. A comparable native run (500–650 cells on a 128-unit sphere) went from 13.2 to 31.6 cycles per second. In the browser the same quick-start world read 13–23 cycles per second with 410–530 cells, on a busy machine; I could not show a clear browser gain from this change.

## Not solved

- Hunters are a real but small guild, about 1% of bodies, and can still die out. A carnivore appeared in one run; a lasting second trophic level has not.
- Hunters stay single-celled, so guts, and bodies built around a mouth, have not evolved.
- Bodies with workers and breeders have appeared unaided only as a handful and have not become established; seeded by hand they persist. Fungus-like life stays rare.
- Limbs, coordinated walking and anything resembling a brain have not been observed to evolve. Rowing is slow: a four-cell rower covers about a body length in 300 cycles. Bred worlds assemble muscle and fins but only a handful of complete rowers, and stay small.
- Left alone, most cells still live singly (6–15% in bodies). Bodies are common only where the player breeds for them.
- Fat has not evolved in any run, with or without seasons.
- Nothing has yet evolved a jointed limb. The lever works when supplied; evolution has not been seen to build one.
- Most cells that carry a mouth do not hunt; it protects them from armed kin.
- Horns and image-forming eyes are not modelled. Propulsion is one push per cell, not a visible flagellum.
- A cell's heading is not carried round when its body rotates, so a body's swimmers can end up pulling against each other.
- How much life a planet holds depends sharply on the star. On the default patchy-resource planet (128 units, 3,000 cycles, native runs) the population ended at 84 cells at 1×, about 220 at 3× and 4×, and passed 2,000 and was still growing at 6×. New planets therefore default to Bright (6×); with evenly spread resources the same star fills the cell limit.
- The legacy labs are untouched.

## Checks

```sh
cargo test --profile preview --manifest-path engine/Cargo.toml --lib   # 133 tests
npm run check:steering   # the shipped WASM, through the same API the UI uses
npm run check:scheduler && npm run check:nursery-ui && npm run check:brush-ui
npm run build
```

Native experiments with powers:

```sh
npm run simulate:fast -- --founders 64 --cells 4096 --size 128 --shape sphere --layout patchy --cycles 3000 \
  --steer '{"kind":"light","scale":6}' --steer '{"kind":"mutation","scale":4}' \
  --steer '{"kind":"goal","goal":"movement","strength":0.25,"period":30}'
```

The run prints population vitals, causes of death and the ladder.
