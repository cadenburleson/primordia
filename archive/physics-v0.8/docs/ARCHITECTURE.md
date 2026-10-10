# Architecture and validation

## Runtime

```text
TypeScript controls → Web Worker → Rust / WebAssembly
                         ↓
          fixed-step metabolism, physics, genetics
                         ↓
          packed organism, particle, link and joint snapshots
                         ↓
         Three.js instancing + UI and history
```

`engine/src/lib.rs` owns ecology and heredity, `engine/src/body.rs` owns developmental geometry and XPBD physics, `engine/src/articulation.rs` owns angular constraints, paired motors and recurrent control, `engine/src/tissue.rs` owns regulation, stores/transport and repair, and `engine/src/experiment.rs` owns the controlled locomotion assay, and `engine/src/population.rs` owns supplied competing founders and ecological family/ancestry accounting. Rust is the source of truth. The same core is exercised by native tests and compiled into the browser module. Determinism and exact save continuation hold within the same build/runtime; floating-point differences can produce different native and WASM trajectories. The main thread never runs the life solver. The worker sends render arrays with transferable buffers, not a separate JavaScript object for every rendered cell. Metadata includes the selected organism, active lineages, events, and recent history.

The world renderer uses GPU instancing for membranes, nuclei, support links, joint cuffs and paired motor bands. Bloom, animated water, directional lighting, and shadows form the high-quality mode. Performance mode removes bloom/shadows and lowers pixel resolution. Far-away colonies show fewer representative cells; inspection renders their complete body. Render detail does not change simulated biology.

## Practical bounds

- Population budgets: 1,000 / 4,000 / 12,000 **organisms**, including multi-cell bodies.
- Maximum body size: 64 cells, requiring healthy support and sufficient inherited growth capacity; a shared 64,000-cell world budget constrains growth and introductions.
- Renderer: 65,000 visible cell instances plus nuclei, 22,000 physical link segments, 8,000 joint cuffs and 16,000 motor bands; far detail is reduced.
- Local interactions: spatial hash over 40×40 cells, with bounded neighborhood sampling. Dense regions therefore use an approximation, not all-pairs contact resolution.
- Fixed ecology update: 0.25 cycles; body physics: 15 substeps per ecology update, with eight constraint iterations for jointed bodies and four for legacy bodies. At most 64 particles and 256 links and 60 angular joints per body. Packed particles have 9 floats including health/reserve; links have 6 including integrity. Joint buffers have 9 floats: three global cell indices, min/max/current angle, flexor/extensor activation and limit error. Snapshot rate: up to 10 Hz. UI metadata: approximately 4 Hz. Graphics refresh independently.
- History: 240 recent samples, spaced three cycles apart. Journal: 60 recent events. Full living genomes and lineage references persist in saves; the plot is a rolling window.
- Save import: 30 MB frontend limit plus Rust schema, graph indices and numeric bounds validation. Version-four saves additionally retain anatomy coefficients, joint ranges, recurrent state and motor comparisons. Imports from versions one through three preserve their prior mechanics; existing organisms do not silently acquire joint programs. Version-one colonies acquire legacy physical bodies.
- Competing ecosystem: 24/48/96 supplied single cells, six starting families, existing ecological selection and a configurable population budget. Current founder records, cumulative family counters, peak generation/jointed-body counts and the latest 256 parent/mate birth records persist in additive version-four save fields. No automatic rescue or movement fitness score is used. Separate held-world storage supports Return to ecosystem.
- Lab: 12 candidates per generation, 20 cycles per candidate, identical development/settling/measurement windows. Replay and disabled-motor trials hold selection. Motor toggles restart the same genotype from one cell. The tissue study instead uses the main ecological rules with a supplied precursor. A separate IndexedDB key preserves the ecosystem without overwriting the manual save.

The maximum budget is a software limit, not a guarantee of 12,000 survivors or 60 FPS on every Mac. Requested acceleration may run below the selected factor when compute is saturated. Browser background tabs may throttle rendering or timers; simulated time reports completed steps.

## Validation commands

`npm test` covers seeded determinism, save/resume continuation, non-mutating inheritance, nutrient consumption and recycling, population limits, extinction, daughter-retention colonies, compatible recombination, input validation, contrasting metabolic selection under controlled conditions, physical feeding and prey damage, and sensory actuation without free-space thrust.

`npm run check:wasm` executes the actual distributed WebAssembly module and verifies initialization, packed snapshot sizes, evolution, selected-organism lookup, exact save/resume continuation, and malformed save rejection.

`npm run check:locomotion` compares active and inactive motors in the distributed WASM, checks measured evolutionary improvement, validates graph buffers, exact resumed search/replay, old saves and malformed links. It writes a JSON results report and a portable champion replay.

`npm run check:joints` validates the actual distributed WASM angular buffers, limits, three-seed motor/controller comparisons, ten-generation selection, exact search/replay continuation, runtime motor toggles and version-three migration. It exports a jointed champion and report.

`npm run check:population` runs three shared-resource populations, checks multiple active jointed bodies, real offspring/mating/feeding, family totals, cellular stores, bounded ancestry, exact saved continuation and manual single-cell introductions. It exports a grown population checkpoint.

`npm run check:ecology` runs three ordinary presets for 3,000 cycles each, validates sampled saves, checks real feeding events, and writes an ecological report plus an unedited evolved-world example.

`npm run check:tissue` checks three supplied-precursor studies, actual cellular growth/differentiation, stores summing to total energy, support/motor expression, valid saves, exact resumed state and legacy migration. It exports a study checkpoint and report.

`npm run benchmark` initializes 1,000, 4,000, and 12,000 organisms, runs 200 fixed steps in a release build, and reports elapsed time per step. It includes crowding and resulting deaths; it is not a steady-state or GPU benchmark. The machine's measured results are in `VALIDATION.md`.

`npm run build` type-checks TypeScript and produces `dist/`. After editing Rust, regenerate WebAssembly first with `npm run build:wasm`; a browser build alone intentionally uses the checked-in module.

The browser checks include visible rendering, pause/resume, acceleration, organism focus, presets, nutrient overlay, pulse, selection, save/load, desktop and mobile layouts, and browser console diagnostics. Refer to `VALIDATION.md` for the recorded outcome and unresolved limits.

## Technology references

- [Rust and WebAssembly](https://rust-lang.org/what/wasm/) — using Rust alongside JavaScript for compute-heavy work.
- [wasm-bindgen CLI](https://wasm-bindgen.github.io/wasm-bindgen/reference/cli.html) — matching compiler/binding versions and typed array interop.
- [Web Workers](https://developer.mozilla.org/docs/Web/API/Web_Workers_API/Using_web_workers) — keeping compute off the interface thread.
- [Three.js InstancedMesh](https://threejs.org/docs/pages/InstancedMesh.html) — reducing GPU draw calls for repeated geometry.

Directional-sensing intervention is restricted to competing populations and persists in the save. The engine records cumulative environmental uptake energy in f64; metadata and the family panel expose it. The worker command changes only the cue switch, resets timing accumulation and publishes state. `check:navigation` performs paired equal-start on/off runs across three seeds, validates first-update cost equivalence, samples valid saves and checks exact continued state and additive migration.

`engine/src/autonomous.rs` is the default app-world cell engine. `Core.tick` delegates wholly to it when its additive state is present. Fixed legacy genes, tissue types, organism targets and lab control do not govern these cells. The engine generates transient `Organism`/`Body` display projections to reuse Three.js buffer contracts; these are never stored as biological organisms. The worker starts autonomous evolution on new-world initialization, and save/import restores the original engine of each checkpoint.
