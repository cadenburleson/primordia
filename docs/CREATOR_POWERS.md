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

Each step is measured from living cells every two cycles. The first version had eleven: living cells, colonies, multicellular bodies, specialised tissues, senses, muscle, skeleton, nerve nets, powered movement, organs, sexual reproduction. There are now eighteen; see the next section.

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
| Engulfing | Swallow a touching cell from another body whose bulk is at most 80% of its own, taking the prey's fuel store too. A cell built for this takes up little dissolved fuel and harvests little light. |
| Venom | Wound any other body it touches. Skin blunts it. |
| Spines | Cut what an engulfer can take, and wound it for trying. |
| Chemosynthesis | Make fuel from waste using mineral chemistry, with no light. The energy comes from the planet's interior and is tracked like sunlight. |

Rules that hold for every weapon:

- **A body does not attack itself.** Contact digestion used to eat a cell's own attached neighbours; it, engulfing and venom now act only between bodies.
- **A cell is immune to the weapon it makes.** Without this, weapons also hit unattached relatives nearby, and in every run a silent copy of the gene outcompeted a working one.

**Complex cells.** Each time an engulfer swallows a light-feeder there is a 1% chance per feeding tick that it keeps the prey alive inside instead. The host then harvests light through its captive regardless of its own membrane, pays a small upkeep for it, and passes it to its daughters. This is one event from the origin of eukaryotes, not nuclei or organelle biology.

**Diets.** Every body is classified from what it has actually taken in. An eater takes a quarter or more of its fuel from other life: a herbivore if its prey mostly feed themselves, a carnivore if they are mostly eaters, an omnivore in between. Other bodies are light feeders, soup feeders (dissolved fuel), decomposers or vent feeders. Life shows the shares as a bar.

**New powers.** Breed for Swimmers or Defences; "Movement" is now "Muscle". The Deep sets how strongly the planet's interior feeds vents (Ember sea starts at Active). The ladder has 18 steps, adding swimming, guided swimming, hunters, carnivores, complex cells, defences and reflex arcs (a sensing cell wired directly to a moving one).

**Breeding was reworked** after it was found to stunt every world it touched (about 100 cells against 1,200 unbred):

- A round now removes a share of the births since the last round, so the population keeps growing.
- A removed cell returns to the world as dissolved fuel rather than as remains only a decomposer can use.
- A low-scoring body larger than the remaining quota loses that many cells instead of escaping. Before, breeding for anything bred big bodies.
- Carrying a silent gene earns almost nothing; expressing it does.

### What was measured

- **Swimmers spread on their own.** Guided swimmers seeded into 8% of an unbred world reached 42–48% of cells in 1,500 cycles. Bred for, motors were built by every cell within 2,500 cycles on two native seeds, and by 445 of about 500 cells by cycle 1,620 in the browser, where they visibly spread across the planet.
- **Defences respond to breeding** in every run, reaching between a fifth and all of the cells depending on seed.
- **Hunters stay rare.** In a seeded test an engulfer was touching another body about 8% of the time, and about one in five of those was small enough to swallow, which is roughly 5% of what a lone hunter needs to live on. Eaters were under 1% of cells in the browser run. Bodies pass the size test every time, so the workable order is bodies, then swimmers, then hunters. A dimmer star makes prey worth more.
- **Carnivores** were not observed.
- **A complex cell** arose unprompted in the browser quick-start run by about cycle 1,300.
- **Reflex arcs** reached "gene appeared" and no further.

### Speed

Terrain under each cell is now sampled on every fourth mechanics substep instead of all twenty. A comparable native run (500–650 cells on a 128-unit sphere) went from 13.2 to 31.6 cycles per second. In the browser the same quick-start world read 13–23 cycles per second with 410–530 cells, on a busy machine; I could not show a clear browser gain from this change.

## Not solved

- Hunters that live mainly on prey, carnivores, limbs, coordinated walking and anything resembling a brain have not been observed to evolve.
- Horns and image-forming eyes are not modelled. Propulsion is one push per cell, not a visible flagellum.
- A cell's heading is not carried round when its body rotates, so a body's swimmers can end up pulling against each other.
- How much life a planet holds depends sharply on the star. On the default patchy-resource planet (128 units, 3,000 cycles, native runs) the population ended at 84 cells at 1×, about 220 at 3× and 4×, and passed 2,000 and was still growing at 6×. New planets therefore default to Bright (6×); with evenly spread resources the same star fills the cell limit.
- The legacy labs are untouched.

## Checks

```sh
cargo test --profile preview --manifest-path engine/Cargo.toml --lib   # 110 tests
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
