# Primordia — physical habitat preview

An isolated runnable extension of Primordia. It adds a horizontal lattice-Boltzmann fluid, conservative diffusion/advection, finite chemical inventories, energy-accounted reactions and inherited material synthesis. The main checkout at `../primordia` and its port 5190 are independent.

## Run on your Mac

Open `Primordia.command`, or run:

```sh
npm install
npm run dev
```

Then open http://127.0.0.1:5192/. Dependencies and build caches are local to this preview. Built WASM is included and no Rust toolchain is needed to play.

## What to observe

Choose 1–8,192 identical supplied replicators (default 256) and a total living-cell ceiling up to 64,000 (default 8,192). New worlds begin with those cells and four basic catalyst-production pathways. Cells make materials from finite stores, grow, divide, copy/mutate their genomes and associate on contact. Resource competition selects surviving descendants. Use the layers button for nutrient availability and measured fluid arrows. The inspector shows real material stores and gene programs; the habitat panel reports conserved balances.

This is physically grounded, reduced artificial life in model units. Chemistry and catalyst functions are defined approximations. Atomistic bonding, molecular abiogenesis, tissues/organs and evolved walking have not been demonstrated. See [the exact model and limits](docs/PHYSICAL_HABITAT.md) and [validation](docs/PHYSICS_VALIDATION.md).

## Population, speed and shared experiments

The physical world and the new passive/scarcity population assays use the same engine, including material-based bending and paid angular actuation. The older supplied-body labs are labeled legacy comparisons. Choose starting organisms separately from the total cell ceiling, and raise the ceiling during a run without altering its biology. New-world creation holds the previous run for restoration.

Select speeds through 10,000× or **Max**, and watch the **actual** speed beside the control. Larger populations can run slower, and 64,000-cell real-time performance is not established. For longer experiments, `npm run simulate:fast -- --founders 256 --cells 16384 --cycles 1000 --output examples/native-run.json` runs the same engine natively without rendering (requires Rust). See [shared engine, scale and exact limits](docs/SHARED_ENGINE_AND_SCALE.md).

## Development and checks

```sh
npm run test
npm run build:wasm
npm run check:physical
npm run check:scale
npm run check:scheduler
npm run build
```

If rebuilding WASM, set WASM_BINDGEN to a wasm-bindgen 0.2.114 CLI binary, or install that matching version. Legacy APIs and labs remain as comparisons. Physical states are additive version-four saves and preserve fluid/chemical state for exact continuation within a build.
