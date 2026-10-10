# Primordia

An interactive 3D artificial-life laboratory. Create a world, change its conditions, and watch inherited variation and resource competition shape life over time.

**Rust runs the simulation, compiled to WebAssembly in a Web Worker. TypeScript and Three.js draw and operate the laboratory.** The browser version runs locally on a Mac; no account, backend, or cloud service is required.

## Run on your Mac

Double-click **Primordia.command**, or run:

```sh
cd /Users/cadenburleson/repos/primordia
npm install
npm run dev
```

Open **http://127.0.0.1:5190**. Node 22.12+ is recommended. The generated WebAssembly is included, so Rust is only needed when changing the engine.

## What you can do

- Start an autonomous world with 64 identical four-gene cells. Choose a habitat preset before the run; conditions and replication errors then stay fixed.
- Watch actual cell divisions copy genomes with occasional edits, duplication, deletion and new genes. Local resources, energy, neighbors and signals regulate expression.
- Inspect each cell's genes, parent, generation, energy and expressed actions. Retained daughters form physical connected bodies; survival and further division determine which programs persist.
- Observe finite food uptake, starvation, gene transfer, bond transport and contact digestion when their genes are expressed. Extinction is possible; there is no automatic rescue.
- Orbit, zoom, select and follow cells. Pause or accelerate model time and inspect genome diversity, division ancestry and population history.
- Save locally and export JSON. **Restore** recovers the saved state; **Open** imports an exported checkpoint. Save before closing, and retain exports as portable backups.
- Open the older **Competing ecosystem**, **Tissue study**, **Locomotion lab** and **Jointed locomotion lab** as explicitly supplied-trait comparisons. **Return to ecosystem** recovers the held autonomous world. Imported older saves retain their original rules.

## Biological scope

The default engine uses variable cell-local gene programs, with no assigned tissue roles, body blueprint or timed anatomy upgrades. All founders receive basic uptake, division and weak adhesion genes. Other action genes must arise through sequence changes. This is a supplied replicator and a designed finite physical/chemical language; it does not model molecular abiogenesis, literal DNA/protein synthesis or arbitrary biochemical invention.

The first three 300-cycle autonomous runs produced inherited genome variation and connected bodies of up to three cells. **Organs, walking and an evolved advantage for multicellular coordination have not been demonstrated.** Those remain measured goals in [the plan](docs/EVOLUTION_PLAN.md). The system does not guarantee increasingly complex or Earth-like life. Temperature/radiation-specific adaptation and conserved molecular chemistry are not implemented in the new engine.

The older labs supply support, contractile and controller capabilities and can select improved movement. Their results demonstrate selectable physical locomotion within that designed grammar; they are not autonomous invention of muscles or bones.

Read [the cellular engine](docs/CELLULAR_ENGINE.md), [validation](docs/VALIDATION.md) and [architecture](docs/ARCHITECTURE.md) for exact rules, evidence and limits. [MODEL.md](docs/MODEL.md) documents the retained legacy model.

Simulation work now runs in short CPU slices with idle time, stops while the tab is hidden, and avoids repeated snapshots while paused. High acceleration may advance more slowly under load. [Performance notes](docs/PERFORMANCE.md) describe these changes and the future GPU compute path. Runtime testing is left to the user; this scheduling revision has only been checked by a production build.

An example evolved for 100 cycles with the default rules is included at [examples/tidal-world-100.json](examples/tidal-world-100.json). Use **Open** to load it. No genomes or body counts were edited to produce this example. It is a version-one save and migrates to the physical-body model on import. A current, naturally evolved ecosystem snapshot is included at [examples/evolved-ecosystem.json](examples/evolved-ecosystem.json). A measured champion replay from the controlled assay is included at [examples/locomotion-lab.json](examples/locomotion-lab.json); use **Open** to inspect it. The lab precursor is supplied intentionally. [examples/tissue-study.json](examples/tissue-study.json) holds an actual grown tissue-study checkpoint. Its initial precursor is also explicitly supplied; growth and body counts were produced by the ecosystem rules.

A jointed champion that grew from a single supplied precursor is included at [examples/jointed-lab.json](examples/jointed-lab.json). Use **Open** to inspect it, then **1×** and resume to watch its measured movement. Legacy tissue studies use that joint model; new autonomous worlds use cell-local bonds, and imported saves preserve their original engine.

A grown competing population is included at [examples/population-ecosystem.json](examples/population-ecosystem.json). **Open** imports it. These founders are explicitly supplied; their body growth and offspring are generated by the ecological rules. Founder family labels describe initial biases, not permanent species or prescribed winners.

## Autonomous cellular evolution

New app worlds now use the cell-local engine. Start from 64 identical basic replicators and let variable genomes change through copying errors, duplication, deletion and new genes. Cells express actions from local signals; daughter retention creates physical bond graphs. Inspect real per-cell genes, promoters, expression, ancestry and genome lengths. There are no tissue unlocks, prescribed body plans or trait sliders in this run. Habitat presets start new autonomous worlds; older experiments remain separate comparisons.

See [CELLULAR_ENGINE.md](docs/CELLULAR_ENGINE.md) for the exact rules and limits. The implemented language and basic replicator are supplied; arbitrary biochemistry, organs and walking are not claimed. [autonomous-world.json](examples/autonomous-world.json) is an actual grown checkpoint.

## Develop

```sh
npm test                 # Native Rust tests
npm run benchmark        # Native Rust simulation throughput
npm run check:wasm       # Test the actual bundled WebAssembly
npm run check:autonomous # Cell-local heredity, variable programs and retained-division physics
npm run check:navigation # Paired equal-start ecological sensing comparisons
npm run check:population # Shared jointed ecology, ancestry, breeding, feeding and saves
npm run check:joints     # Angular mechanics, controller ablation, three-seed selection
npm run check:locomotion  # Physical development, motor ablation, selection and replay
npm run check:ecology     # Three 3,000-cycle ecological runs and save validation
npm run check:tissue     # Replicated supplied-precursor ecology and cell energy checks
npm run build            # TypeScript check + browser production build
npm run preview          # Production build at http://127.0.0.1:5191
```

To modify the Rust engine:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.114 --locked
npm run build:wasm
npm run build
```

The engine crate and wasm-bindgen CLI must use the same version. A local `.tools/wasm-bindgen` is supported, or set `WASM_BINDGEN` to the executable path. `Cargo.lock` and `package-lock.json` pin the dependency graph. Generated bindings and the `.wasm` module live in `src/wasm/`.

All landscape, rocks, deposits, water, cells, and body plans are procedural; no game assets are imported. Interface fonts fall back to local serif and system fonts if offline.
