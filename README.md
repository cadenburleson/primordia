# Primordia — grow life on your own planet

Plant single cells on a planet and steer what they become. **Life** shows how far your world has climbed, from colonies to tissues, muscle, skeleton and nerves, measured from living cells. **Powers** let you set the pace of change, brighten the star, breed for a purpose, favour a lineage, or drop a meteor. You never write a gene; you decide which inventions get a future. Cells can evolve motors and steering, swallow or sting each other, grow spines, feed at vents, and occasionally keep a swallowed cell alive inside them. Life shows who is a light feeder, herbivore, omnivore or carnivore. Cell types have their own colours on the planet, and each first is announced as it happens.

Max speed now uses the whole simulation thread and the shipped engine is a release build. See [creator powers, life's ascent and speed](docs/CREATOR_POWERS.md) for what changed, what was measured, and what is still unsolved. The sections below describe the engine those powers act on; where they describe the old throttled scheduler or a "Guide / Shape" dock, this section supersedes them.


## One project, one active app

Develop and launch from `/Users/cadenburleson/repos/primordia`. The current v0.12 app, Rust engine, dependencies and launcher all live here. The browser address remains **http://127.0.0.1:5194/**.

```text
primordia/
  src/                     Browser UI, renderer, worker and built WASM
  engine/                  Current Rust simulation engine
  docs/                    Current documentation and historical design notes
  tests/                   Checks and user-run simulation experiments
  Primordia.command        Launch the current app on port 5194
  archive/
    prototype-v0.6/        Preserved original prototype (old port 5190)
    physics-v0.8/          Preserved physics preview (old port 5192)
```

The archive contains previous versions, not required pieces of the active engine. Work on `src/` and `engine/` at the root. The active app owns its `node_modules`; it no longer depends on a sibling repository. See [archive notes](archive/README.md).

The planet fills the screen. Floating **Guide, Shape, Inspect, and Collection** buttons open one contextual panel at a time. The guide responds to population limits, reproduction, decline, connected bodies, and measured diet. A compact **evolution nursery** holds your planet while you explore descendants; save an organism and choose **Return & place on planet**. See [the planet and nursery workflow](docs/OBSERVATORY.md).

**Paint the habitat:** Shape → Surface now supports drag strokes, a visible footprint, size/strength controls, ellipse stretch/rotation and whole-stroke undo. Right-drag / Alt-drag orbits while a brush is selected. Climate → Seasons controls variation and year length. Guide offers one actionable experiment at a time. See [painting and seasonal controls](docs/WORLD_TOOLS.md) and [research, supported functions and remaining anatomical gaps](docs/EVOLUTION_PRESSURES.md).

Configure a planet before anything runs: startup opens setup with no seeded world or simulation steps. **Progress** is the default view, and **Fast-forward evolution** prioritizes simulation over display updates. See [the workflow, signals to watch and display-rate explanation](docs/EVOLUTION_WORKFLOW.md). Save before refreshing to this revision. Runtime testing remains with the user.

Explore **Ecology** for resource maps, site inventories, measured body uptake/output and feeding contacts. **Evidence** lets you start matched-checkpoint baseline, actuation, transport and excitable-signal comparisons. New worlds offer patchy food and shade; no evolved trait is awarded. See [how to observe and test functional claims](docs/OBSERVATION_AND_ASSAYS.md).

> Current connected implementation: [v0.12 systems](docs/CONNECTED_SYSTEMS.md). Development, transport, materials, signaling, reproduction, particles and larger globe spans are assembled in the same physical engine. The material below records an earlier stage; it is not validation of v0.12.

New worlds default to a **1,024-unit spherical circumference**, with **8,192** and **64,000** available under World size. The default is sixteen times the surface area of a 256-unit globe. Founders begin at one local origin; cell size and the 8,192-cell ceiling stay unchanged. Large environmental fields are coarse, capped at 160×80 sites, rather than resolved at microscopic scale over the whole planet. Large worlds open near living cells at their actual scale. Use the microscope button for the local population view, the expand button for a world overview, and Follow for one body. Overview dots locate cells; their screen size does not represent body dimensions. Save before creating a new world. This revision is build-checked only; simulation and performance testing are left to the user.

The view uses simple materials with no bloom, fog, shadows, decorative particles, artificial nuclei or automatic rotation. Camera interaction draws are capped at 30 per second in all modes, and the inspector at 15. Stationary views redraw only when needed; Evolution first prepares periodic snapshots every two seconds. Paused, unchanged views redraw only for interaction; hidden tabs do not draw. Body matrices update on simulation snapshots, rather than every display frame. Physical weather and biology continue in the engine. The resolution button changes pixel density only.

Create worlds with **New world** in the header. The setup dialog closes after creation; the left sidebar controls the active environment, terrain, resources and organisms. Seed and placement are in a collapsed setup section. See [active world tools](docs/WORLD_TOOLS.md) for copying, gene editing and undo. The header **Labs** button contains experiments, with older models labeled **Legacy comparisons**. All current cellular, material, sensing, transport and reproduction mechanisms already run in the main world; labs do not enable them. Choosing a habitat prepares the next world without interrupting the current run.

Current extension: [Inherited development, adaptive contact signals and reproductive cycles](docs/LIFE_ENGINE_ROADMAP.md). This remains an artificial-life research model; organs and intelligence are not guaranteed outcomes.

# Primordia — organism and ecosystem preview

The current application lives at `/Users/cadenburleson/repos/primordia` and runs at **http://127.0.0.1:5194/**. Earlier previews are retained under `archive/`; they are historical reference copies and do not run alongside the main engine.

Open `Primordia.command` on your Mac, or run `npm install` and `npm run dev`. Built WASM is included; Rust is only needed to rebuild the engine or run native experiments.

The default physical ecosystem is now a spherical planet: radial gravity, terrain contact, shallow surface water and material transport across longitude and poles. Choose world spans of 64, 128 or 256, retaining cell size; span is globe circumference or planar width. Choose one local origin or founders across the world. See [curved physics, scale and limitations](docs/SPHERICAL_SCALE.md).

Start with 256 replicating cells and a ceiling of 8,192–16,384; select **Max** and watch achieved speed. Up to 8,192 founders and 64,000 living cells are allowed, but larger populations can be slower. Every founder still starts with only four genes: photosynthetic catalyst, nutrient transporter, weak adhesion, and replication. Mutation and gene exchange can introduce the other production pathways. No adult animal, skeleton, gait, organ stage or behavioral fitness score is inserted into the ecosystem.

The same physical world now supports mineralized support and paid contraction alongside lipid-like reserves, local receptors, excitable signaling, attached-cell fuel/ATP exchange, digestion, protection and repair. These mechanisms interact through actual material and energy stores. Cells divide in three dimensions; differentiated material-bearing regions are observations of the living contact graph, not anatomical unlocks.

Unassimilated food and prey material become organic detritus. Metabolism produces oxidized waste, corpses recycle their existing inventories, detritus-breakdown catalysts can feed on organic remains, and photosynthesis converts oxidized material using tracked light. The resource overlay can show fuel, detritus or minerals. Day and seasonal forcing, conserved rain/snow/evaporation/melt, downhill runoff and volcanic terrain/mineral disturbances affect the same population autonomously.

This is a reduced artificial-life model with a designed chemical vocabulary. Complex anatomical organs, learning, evolved walking, directional foraging and an enduring predator/decomposer food web are **not guaranteed or demonstrated merely by adding these mechanisms**. See [the model, evidence and research limits](docs/ORGANISMS_AND_ECOSYSTEM.md). Earlier physical-model documentation describes the inherited baseline; this document supersedes its static-environment and ten-pathway descriptions.

## Checks and experiments

```sh
cargo test --profile preview --manifest-path engine/Cargo.toml --lib
npm run build:wasm
npm run check:organisms
npm run check:geometry
npm run check:scale
npm run check:scheduler
npm run build
```

`npm run build:wasm` uses the optimized local `preview` profile without LTO. Set `WASM_PROFILE=release` for a production optimization build. The local wasm-bindgen CLI must match version 0.2.114.

For long experiments without rendering:

```sh
npm run simulate:fast -- --founders 256 --cells 16384 --cycles 1000 --output examples/native-run.json
```

Import the resulting file with **Open**. Old physical saves are migrated by padding the vocabulary with zero stores/weights, preserving their existing cells and molecules. New physics changes their future; this is not a claim of identical behavior across engine versions. Exact continuation is tested within the current build.
