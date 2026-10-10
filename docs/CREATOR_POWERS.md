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

Eleven steps, each measured from living cells every two cycles: living cells, colonies, multicellular bodies, specialised tissues, senses, muscle, skeleton, nerve nets, powered movement, organs, sexual reproduction.

Each step has four stages: absent, gene in the pool, first working cells, established. A step is established only when cells have built the tissue, for example muscle in 10% of living cells. The first time a world reaches each stage is recorded in the save and announced once.

These are thresholds on measurements. "Organs" means bodies of eight or more cells running four or more tissue types; it does not mean an anatomically organised heart or gut. "Powered movement" means bodies whose cells have paid for contraction work; it does not mean a coordinated gait.

The panel also names what is holding life back, from deaths over a recent window: starvation points to the star, then the oceans, then a rain of plenty.

## Seeing it

- Cell types have distinct colours on the planet and in the specimen view, with a live legend: red muscle, ivory skeleton, blue senses and nerves, violet digestive, and so on (`src/sim/roles.ts`).
- The chosen lineage is tinted gold.

## Not solved

- Coordinated walking, limbs and intelligence have still not been observed to evolve.
- Flagella, horns and image-forming eyes are not modelled.
- How much life a planet holds depends sharply on the star. On the default patchy-resource planet (128 units, 3,000 cycles, native runs) the population ended at 84 cells at 1×, about 220 at 3× and 4×, and passed 2,000 and was still growing at 6×. New planets therefore default to Bright (6×); with evenly spread resources the same star fills the cell limit.
- The legacy labs are untouched.

## Checks

```sh
cargo test --profile preview --manifest-path engine/Cargo.toml --lib   # 100 tests, 7 for steering
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
