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

**Fat.** The planet already has days (48 cycles) and mild seasons, set in World. A second seasons dial added here was removed as a duplicate. The reason fat never evolves was measured instead: 97–99% of cells are hungry at any moment, and a cell stored fat only when it was glutted and spent it whenever it was merely peckish. Fat is now laid down by any comfortably fed cell and kept until the cell is truly short. Breed for Fat reserves and Breed for Limbs are new goals. Neither has been benchmarked yet.

### Starter species and the feed

Evolution from one founder builds the parts (muscle, fins, mouths, walls) but rarely a whole creature. A survey of other evolution sims found the same thing everywhere: the ones with sustained visible behaviour start from working organisms, and most keep a population floor so predators cannot blink out (ALIEN, The Bibites, Species: ALRE, Polyworld, evolv.io).

Powers now has **Release a species**. Each starter species is one founder cell with a founder's stores and a genome written from the ordinary gene set (`engine/src/bestiary.rs`). It is released through the same path as any introduced organism, has to feed, build and breed like everything else, and mutates from its first division.

| Species | What its genes build |
|---|---|
| Alga | Strong light harvest inside a wall; sisters stay cemented |
| Swimmer | A motor, steered toward light |
| Grazer | Digestive enzymes and a motor steered toward the scent of life; only a weak light harvest of its own |
| Hunter | A mouth and a motor steered toward scent |
| Thornback | Spines and skin |
| Rower | Glue, a growth compass, muscle, a beat and fins |
| Courter | A motor steered toward scent, and gametes that fuse with a gamete from another body |
| Drifter | The plain founder |

**Keep released species from dying out** (on by default) restocks a species with four founders when it falls below eight cells. The count of restocked founders is shown; it is external input, like the first release.

**Happening now** is a feed on the planet view: who is eating whom, by species where known, and each fusion of two gametes. Clicking a line jumps to the eater. Life shows the share of living cells descended from each released species, and Inspect names a body after its species.

**What was measured.** On fresh bright planets seeded with all seven species (three worlds, 1,500 cycles, no restocking): populations reached 2,100–3,000 cells against about 1,000 unseeded. Swimmers, algae, grazers, thornbacks, hunters and courters all survived in every world; rowers died out in one and fell to three cells in another. Hunters held 5–125 cells. Only 13–33 cells at a time were getting a quarter of their fuel by eating: grazers with a founder's full light harvest thrived without grazing. Grazers with no light harvest at all starved to two or three cells and survived only by restocking, so the design now keeps a weak harvest; that middle setting has not been measured. With restocking on, hunters, rowers and courters all stayed present for 900 cycles in every world. In a separate test 80 courters produced 969 gametes and 51 fused zygotes in 600 cycles, but the mating lineage shrank in every mixed world.

### Speed

Terrain under each cell is now sampled on every fourth mechanics substep instead of all twenty. A comparable native run (500–650 cells on a 128-unit sphere) went from 13.2 to 31.6 cycles per second. In the browser the same quick-start world read 13–23 cycles per second with 410–530 cells, on a busy machine; I could not show a clear browser gain from this change.

### Water, plants and animals, and survival as the only selector

**The seas were being lost to the sky.** Evaporation ran about five times faster than the one roaming storm could rain back, so a planet's seas shrank from 40% of its surface to 12% in 300 cycles and every earlier measurement was taken on a drying world. Air now rains out whatever it holds beyond what it began with. Seas hold at 36–40% cover; old saves refill by themselves.

**Buoyancy is by material.** Tissue is as dense as water, fat lighter (0.9) and bound mineral heavier (2.5). A cell's weight is its mass less the water it displaces, by how much of it is under the surface. Water is no longer a floor: the only floor is the sea bed or the land. Measured on one cell in deep water: plain tissue does not move, fat rises, bone sinks seven times faster than fat rises, and over dry land the same cell falls a hundred times faster than bone sinks. The ground grips a body, and a foot digs in, only as hard as the body presses down. New arrivals are set afloat just under the surface.

**What this does not do.** Swimming in this world is limited by the water's drag, not by weight, so buoyancy changes where a body lives and how much the ground grips it, not how fast it swims. Sinking and rising take hundreds of cycles.

**Plants and animals.** A cell given over to harvesting light cannot swallow or digest living prey (`Cell::bite`, the square of what light-harvesting machinery it lacks). To eat, a cell has to give the light up. The Hunter and Grazer starters therefore carry no light harvest at all. Kinds of life are sorted by how a body feeds: one that makes its own food is plant-like, wall or no wall. Walled cells are drawn as blocks.

**Dispersal.** Free cells and loose grains in water are jostled by its eddies, more when the oceans are stirred (about one unit a generation at natural mixing). Bodies and anything on land are not.

**Survival is the only selector by default.** Evolve is one dial, Natural, Fast or Fastest, that sets how often copied genes vary (1×, 2×, 4×) and lets touching cells swap a gene now and then. Selective breeding is an optional, collapsed experiment. Measured on seeded planets over 800–1,200 cycles: rewarding a trait automatically, with 4× variation and common gene swapping, cut populations from about 2,100–3,300 cells to 250–700; selection alone removed nearly all the animals (34–74 animal-like cells against 400–1,200); 4× variation with common gene swapping about halved the world, because species blur into each other. The presets as shipped, on two planets at 800 cycles: Natural held 1,882 and 3,255 cells with 5 and 6 steps of the ladder established; Fast held 1,840 and 2,413 with 7 and 7; Fastest held 1,873 and 1,893 with 6 and 8. Two planets is a small sample.

**Planet at a glance.** The Planet panel opens on the planet's age, season, weather, temperature and where its water is: seas and lakes (standing water sorted by connected body), clouds and vapour, snow and ice. Great flood and Long drought add water everywhere or remove a third of it; both can be undone.

**Creatures.** One palette holds the starter species, the bodies you saved and the planet's record-holders. Pick one and drag across the map to scatter copies (`Core::paint_life`); a stroke is one undo. Every few seconds the engine names the leading living body on four measures (most kinds of working part for its size, largest, most fuel from prey, fastest under its own power) and the interface keeps a copy of each, replacing one only when its record is clearly beaten.

### Earth as the model

Earth's life went: slow microbes living on dissolved food and vent chemistry with no free oxygen; light-harvesters, whose waste oxygen slowly filled the air; complex cells, made when one cell swallowed another and kept it; sex; bodies; then fast animals and an arms race of eaters and eaten; and last, life on land, plants first. Three pieces of that were missing or wrong here.

| On Earth | Here |
| --- | --- |
| Life before oxygen is slow | A cell with no oxygen captures half the energy in its fuel (`FERMENT`; the real figure is nearer a fifteenth) |
| Light-harvesters make the oxygen | Each unit of fuel made from light frees oxygen; burning fuel fully uses it up. The air mixes planet-wide within days |
| Oxygen rises once the reduced sea is used up | A new planet starts with none. Dissolved food is burned first and soaks up what is made |
| Water is thin; a swimmer covers many body lengths cheaply | Drag on a body is 8% of the fluid lattice's own (`SLIP`), and a motor's work is charged at 8% of face value, so the same effort carries a cell twelve times as far. Bonds are damped so bodies do not ring |
| The ground is not | A crawler works against the old, full drag |
| Animals live by eating | Hunter and Grazer starters harvest no light and soak up no dissolved fuel. An eater does not follow the scent of its own kind |
| Complex cells from a swallowed light-feeder | Already here |
| Land, plants first | Not here yet |

**Measured.** In the browser on a small planet with all seven starter species: oxygen stood at 2% free at cycle 250, 21% at 450 and 53% at 600, and life went from breathing at 75% to 96%. Natively, three planets at 1,200 cycles held 2,138, 1,729 and 2,144 cells with 54–77% of their oxygen free, life breathing at 96–97%, and 10–26 cells eating at any moment. A swimming cell at full drive covers 15 units in ten cycles for 3 units of energy; a starter's motor runs at about a fifth of that drive. With thin water, grazers and hunters reach food: before any limit on them, a boom of grazers fed on the starting dissolved fuel made 104,000 meals in 400 cycles and ate the seas down to 250 cells, which is why animals no longer soak up dissolved fuel.

## Matter, food chains and land

The rule for this round: energy and matter are never made or destroyed, only changed. Carbon, mineral and energy were already audited. Nitrogen and the food chain are new, and land now behaves as land.

**Carbon and oxygen.** A light-feeder builds fuel from carbon dioxide and water with light and frees oxygen. Burning fuel uses oxygen and gives carbon dioxide back. The player-facing name for the old "oxidised waste" pool is now carbon dioxide.

**Nitrogen** (`physical.rs`: `NITROGEN`, `REMAINS_N`, `N_RATIO`). Living tissue and protein hold 0.15 units of nitrogen per unit of carbon, as sea life does (16 atoms to 106). Sugar and fat hold none.

| Step | What happens |
| --- | --- |
| Uptake | Only a cell that feeds itself (light, vents, dissolved food) or rots remains can take dissolved nitrogen from the water. It costs energy. |
| Growth | Tissue, protein and repair cannot be built without it. |
| Eating | The nitrogen in what is digested goes with it: 85% to the eater, 15% left in droppings. An animal has no other source. |
| Excretion | A cell gives off what it holds beyond a working store, so animals fertilise the water. |
| Death | Bound nitrogen stays in the remains until a decomposer frees it, or it leaches out at 0.4% a cycle. |
| Air | Most nitrogen is gas no cell can use. Lightning fixes a little into the rain of a storm; where oxygen is scarce but present some goes back. |

`Habitat::errors` has a fourth entry for nitrogen and saved worlds are rejected if it does not balance. Worlds saved before nitrogen receive theirs the first time they run.

**The creator cannot make matter either.** A painted creature, a restocked species or a brush stroke of food takes its carbon from the planet's carbon dioxide, its mineral from dissolved mineral and its nitrogen from the air (`settle_arrivals`). Only the energy is new, as sunlight is.

**Trophic levels** (`trophic.rs`). Each cell carries how many steps above the producers it sits, moved toward one more than its food by every mouthful, weighted by how much of its carbon that mouthful replaces. Cells living mostly on remains and dissolved matter are counted apart as decomposers. Planet → "Food web and the planet's books" shows carbon held and taken in at each level, the share passed up, and where all carbon, nitrogen, oxygen and energy are.

**Land** (`Habitat::land`).

| Rule | Effect |
| --- | --- |
| Weight | A body out of water puts its whole weight on the cells touching the ground. Bare soft tissue carries three times its own weight; past that the footing is injured. Bone carries up to eleven times as much, a wall or laid-down fibre five. |
| Drying | A bare cell in dry air is injured at 1.2% a cycle. Skin stops 95% of it and a wall most of it. Ground holding 0.01 of water counts as wet. |
| Water for chemistry | A cell with no water cannot build with light or take in anything dissolved. Its upkeep falls by nine tenths: it lies dormant until rain. Water held in the body counts. |
| Swimming | A motor that beats against water loses its push as the cell leaves the water, and has none on dry ground. |
| Fat and nerves | Fat sheathing two neighbouring nerve cells passes a signal between them up to four times faster. |

**Seas.** New planets are filled to cover about seven tenths of their surface, as Earth is. Before, a third was sea and two thirds of all life sat on dry ground as if it were water.

**New starter:** the Decomposer, a free cell that lives on remains.

**Measured** (three planets, 1,600 cycles, eight starter species, Fast pace):

- 1,807, 1,610 and 2,544 cells, steady or rising, with 56–65% of the surface under standing water.
- With the old seas and the land rules, the same planets fell to 350–550 cells by 2,400 cycles. With land rules off, one of them held 1,290–1,670.
- Nitrogen: 11–15% of the usable nitrogen locked in remains, 30–43% in the living. Unlimited nitrogen raised one planet's population by about a fifth.
- Books balance to within 4 parts in ten million of the energy total.
- Producers take in 58–93 units of carbon a cycle. Plant-eaters take in 0.5–1.1% of that (Earth: about 10%). Meat-eaters take 5–29% of what plant-eaters do.
- Tests: a six-cell soft body standing on one cell is loaded to twice its strength and injured; on bone, wall or fibre the load is 0.22–0.47 and it is unhurt. A signal reaching the fourth nerve cell is nine times stronger with fat.

## Elements, feeding currents and size

Built after researching what carbon-based life is made of and how food chains and body size work on Earth. Each change was measured on three planets (seeds 8042, 77, 5; 1,200 cycles; eight starter species; Fast pace).

**The periodic table** (`src/elements.ts`, data in `src/elements-data.ts` from PubChem). Planet → "Periodic table" shows all 118 elements. A temperature slider shows each pure element as solid, liquid or gas from its real melting and boiling points; it starts at the planet's own temperature (0.3 on the engine's scale is 0 °C, and each 0.1 is about 11 degrees). Nine elements are tracked on the planet and show where their stock is: hydrogen (as water), carbon, nitrogen, oxygen, phosphorus and calcium (together, as the mineral of rock and bone), iron, and sodium and chlorine (together, as salt). Sulfur and silicon are not simulated.

**Matter is counted in carbon atoms.**

| Compound | Nitrogen | Energy per carbon | Weight per carbon |
| --- | --- | --- | --- |
| Sugar (fuel) | none | 1 | 1 |
| Protein and tissue | 0.15 | 1 | 1 |
| Cell wall (cellulose) | none | 1 | 1 |
| Fat | none | 1.3 | 0.47 |

Laying fat down gives off the carbon it does not keep as carbon dioxide; burning it pays the extra energy at once. Stored fat costs a fifth of tissue's upkeep.

**The planet's own chemistry** (`Habitat::planet_chemistry`). One rule moves each substance between air, water and ground:

| Substance | Air | Water | Ground |
| --- | --- | --- | --- |
| Water | vapour | seas | snow and ice |
| Carbon dioxide | gas | dissolved; cold water holds more | remains, lying or buried under silt |
| Nitrogen | gas no cell can use | dissolved | bound in remains, lying or buried |
| Oxygen | free | (mixed with the air) | bound in rust, water and carbon dioxide |
| Iron | none | dissolved | rust on the sea bed |
| Salt | none | dissolved | crust where water has dried |
| Phosphorus and calcium | none | dissolved mineral | bone; the rock itself is not counted |

- Silt buries 0.3% of lying remains each cycle. Buried carbon holds its energy and leaves oxygen free; the planet's heat burns 0.05% of it back out each cycle with free oxygen.
- Free oxygen rusts dissolved iron out of the sea, which takes about 8% of a planet's oxygen before any can build up.
- **Phosphorus** (`P_RATIO` 0.01) is needed to grow and repair. Self-feeders take it from dissolved mineral, animals get it in food, bone lends it, and dead tissue gives it straight back to the water.

A world saved under older rules has its books reset the first time it runs (`Habitat.chemistry`).

**Feeding currents.** A mouth draws in free cells within 1.5 units that its body could swallow, as strongly as it could swallow them, and lets go of spiny or walled ones. A cell held in a body is not drawn. Related changes:

- The rate a cell can burn fuel now comes from oxygen (`0.035 + 0.12 × max(uptake, air)`), so an active life waits for oxygen.
- Hunters are released full grown (a mouth has to be bigger than its prey).
- Swimmers, Grazers and Hunters carry a light-sensing spot. Before this no starter could sense light at all.
- Starters are released in the best lit of four spots of open water.
- The sea keeps stirring a free cell until it is nine tenths out of the water.

**Size.** Water drags on a body as a whole, by how far it reaches, so a compact body meets far less drag for its mass. A cell enclosed by its body pays half the upkeep of a free one. New planets' ocean basins are cut three times deeper: the sea had been about two cell diameters deep on average, and big cells ran aground across a third of it.

**Cell size** (`SIZE`, gene 44; `Cell::stature`). Size is inherited. A regulatory gene sets how large a cell grows before it divides: a quarter of the usual size when silent, the usual size at half drive, four times at full drive. A genome without the gene makes cells of the usual size, so older organisms are unchanged. What size does follows from geometry:

| Follows the cell's surface (two-thirds power of its bulk) | Follows its metabolism (three-quarter power) | Follows its bulk |
| --- | --- | --- |
| Light harvest, uptake of food, nitrogen and phosphorus | The rate it can burn fuel | The energy it can hold |
| What a mouth or enzymes take in a tick | Upkeep, protein building, growth | The energy and size it needs to divide, and what dividing costs |
| A motor's thrust | | |

In the test, under a bright star, a cell a third of the usual size harvests 1.65 times its own carbon a cycle and first divides after 16 cycles; a usual cell 1.38 and 20; a cell three times the usual size 0.94 and 30.

Starters now come in sizes: Plankton (new: a free light-feeder a third of the usual size) and Decomposers are small, Grazers twice the usual size, Hunters half as large again with a mouth that doubles it.

**Dividing.** A cell divides when it has grown to its size, holds enough energy and is ready; that is unchanged, and it is what single cells do in nature. Two faults were fixed. The cost of dividing now follows the cell's size. And on a planet near its cell limit each round of division starts at a different cell, where before the oldest cells took every place that opened up.

**An egg that becomes a body** (the Volvox starter). Alone, its cell grows three times the usual size. Each neighbour a cell is tied to turns its size gene down, to nothing at three neighbours, and strong glue keeps daughters attached. So each daughter of the egg is already big enough to divide again: the egg cleaves into a ball of small cells without growing first, as an embryo does, and the ball then grows as its small cells feed on light. In the test pond one egg of 0.67 was 4 cells holding 1.31 at 30 cycles and 13 cells holding 1.76 at 40.

It also has a life cycle, built from three more ordinary genes:

| Gene | Wiring | What it does |
| --- | --- | --- |
| Replication | falls with each division since the egg, silent at the sixth | The body stops growing at a few dozen cells |
| Commitment to fate A, with the glue gene switched off in that fate | rises with divisions, falls with neighbours | Late on, a cell left with one neighbour lets go |
| Motor | on when alone, off with two neighbours | The loose cell swims clear, and alone it is the next egg |

For this, a cell's division count (`lifecycle.depth`) now rises in both daughters and starts again at nothing when a cell comes away from a body. One egg in the test pond became, in 300 cycles, a population of dozens of bodies (most of 6 to 40 cells; the largest several hundred, where kin bodies had joined) and hundreds of loose cells.

**A hunter of hunters** (the Leviathan starter): three times the usual size before its mouth doubles it, and skinned, which makes it no kin of ordinary hunters, so it can swallow them.

**Planets.** A 512-unit planet sits between 256 and 1,024. Quick start now allows 8,192 cells.

**Two faults that threw things off the planet** (and made saves unreadable):

- Dragging on a body as a whole had removed the water's damping of movement inside it, so a big body's springs rang until it flew apart. Each cell's movement against its own body is damped again at its own rate.
- A bond or a bump sized for an ordinary cell threw a small one about. Bonds and contact forces between cells lighter than a newborn of the usual size are now weaker in proportion. No cell moves faster than 30 units a cycle.

**Other faults found by releasing hunters into dense plankton.**

- The feeding current's work was priced twelve times dearer than a motor's. It is now priced the same.
- A motor kept pushing a cell up a beach until it was nearly dry. Thrust now fades from 70% submerged and is gone at 30%.
- Loose grains had no weight out of water and were carried at the water's speed even in a film over nearly dry ground. Water now carries a grain only as far as it covers it, and a grain in air falls.
- A mouth kept trying to swallow spiny prey and was pricked to death. It now lets go: the more fully armed the prey, the less it tries.

**Measured.**

- Plant-eaters take 0.6–1.75% of what producers make, up from 0.2–0.6% at the start of the round; the best three-planet run, with every starter and room for plankton, gave 1.2–1.75%. Earth's seas run at about 35%. **The 10% target was not met.**
- Hunters now breed (three to four generations deep) and hold 8–21 cells on a planet. Released into dense plankton, a hunter eats two to three times what it needs. Grazers and hunters still need restocking on most planets.
- Decomposers, now small, took hold unaided on two planets in five (111–209 cells feeding on remains).
- Bodies: 5–39% of cells live in bodies of four or more (6–15% before). The largest body was 14–58 cells; bodies of 30 or more appeared on one planet.
- A nine-cell body at full drive covers 26.8 units in ten cycles and a twenty-five-cell body 29.0, against 14.6 for one cell.
- Populations with the default starters: 1,539–3,264 cells.
- Plankton fills a 4,096-cell planet within 400 cycles. Given room it levels off: with every starter seeded, a planet allowed 8,192 cells held 4,813 at 800 cycles (3,477 plankton) and one allowed 12,000 held 4,880 (3,798 plankton). Quick start therefore allows 8,192, and plankton is part of "Fill the seas with all of them". Those planets run at about 1.5 cycles a second natively on a busy machine, against 4–5 at 1,000–1,500 cells.
- Three planets with every starter and an 8,192 limit ran 1,000 cycles and reloaded: 4,092–5,161 cells, 2,900–4,500 of them plankton, hunters 15–46.
- A sessile colony of mouths starved: prey drift too slowly to reach it, so the Sponge starter was removed.
- Algae painted on land 2.9 units above the sea did not stay: within 120 cycles all but three of 41 were in water.

## Not solved

- **Eaters still take about a hundredth of what plants make.** Feeding currents, a faster metabolism with oxygen, sized cells, light-seeking and deeper seas together moved it from 0.2–0.6% to 0.9–1.6%. Catching food now works; what is left is that most producers are walled algae in clumps that only enzyme grazers can eat, and that a planet's cell limit fills with small cells before eaters can multiply.
- **Something throws a loose grain of remains upward now and then**, far faster than anything in the sea moves. The cause is not found. A grain more than two units clear of the surface is set back down on it, and grains now have weight out of water; before that they drifted off the planet and made a save unreadable.
- **Volvox does little on a planet.** With every starter seeded it holds 11–190 cells and its bodies stay small (largest body on those planets 14–71 cells). Kin bodies that touch still join into one.
- **No animal grows from an egg.** A Jelly starter (a big hunting egg cleaving into a ball of small mouths) was tried and removed: running a dozen genes at three times the usual size cost more than it caught, and all twelve starved within 120 cycles.
- **Nothing anchors to land.** A plant on a slope slides or is washed down to the water. Land plants need a holdfast as well as a water store.
- **No fish.** A long body swimming by a wave of muscle has not been built; it would have to pay its way as an eater first.
- **A thick body's inner cells breathe as freely as its surface.** Gills and circulation have no job yet.
- Sulfur and silicon are not simulated. Iron and salt do not yet affect life.

- **Decomposers barely live.** A cell living on remains alone stays at the restocking floor. A decomposer that also soaked up dissolved food cut the planet's population by a third.
- **No cell fixes nitrogen from the air.** Only lightning does.
- **Fermentation still gives half the energy of breathing**, not a fifteenth.

- **Animals reach food but still do not pay their way.** An obligate grazer now takes 0.012–0.022 fuel a cycle from prey (0.001–0.004 before thin water) and a hunter up to 0.02, against roughly 0.035–0.05 to live and swim. Both stay near the restocking floor of eight. They need about twice the intake: they sense prey only within 1.6 units and search blind beyond it.
- **Worlds are somewhat smaller than before oxygen** (about 1,700–2,150 cells against 1,900–3,300), and the first few hundred cycles are slow.
- An eater no longer follows the scent of its own kind, but grazers still mostly touch grazers, and once their genes drift apart they digest each other.

- Hunters are a real but small guild, about 1% of bodies, and can still die out. A carnivore appeared in one run; a lasting second trophic level has not.
- Hunters stay single-celled, so guts, and bodies built around a mouth, have not evolved.
- Bodies with workers and breeders have appeared unaided only as a handful and have not become established; seeded by hand they persist. Fungus-like life stays rare.
- Limbs, coordinated walking and anything resembling a brain have not been observed to evolve. Rowing is slow: a four-cell rower covers about a body length in 300 cycles. Bred worlds assemble muscle and fins but only a handful of complete rowers, and stay small.
- Left alone, most cells still live singly (6–15% in bodies).
- Fat has not evolved in any run. The storage rule has been changed and not yet re-measured.
- Eating is still rare: well under one cell in a hundred at any moment. Mating happens when seeded and does not yet pay for itself.
- The sea is drawn level, but the engine still treats water as a layer of one depth over each field site, so on a small planet rough ground under a sea is wet to the same depth everywhere.
- A seeded planet holds two to three times the cells and runs correspondingly slower (about 6–10 cycles per second in the browser at 2,500 cells).
- Nothing has yet evolved a jointed limb. The lever works when supplied; evolution has not been seen to build one.
- Most cells that carry a mouth do not hunt; it protects them from armed kin.
- Horns and image-forming eyes are not modelled. Propulsion is one push per cell, not a visible flagellum.
- A cell's heading is not carried round when its body rotates, so a body's swimmers can end up pulling against each other.
- How much life a planet holds depends sharply on the star. On the default patchy-resource planet (128 units, 3,000 cycles, native runs) the population ended at 84 cells at 1×, about 220 at 3× and 4×, and passed 2,000 and was still growing at 6×. New planets therefore default to Bright (6×); with evenly spread resources the same star fills the cell limit.
- The legacy labs are untouched.

## Checks

```sh
cargo test --profile preview --manifest-path engine/Cargo.toml --lib   # 161 tests
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
