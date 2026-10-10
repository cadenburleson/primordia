# Shared physical engine and experiment scale (v0.8)

These notes describe the earlier v0.8 physics preview, now preserved at `archive/physics-v0.8/` (old port 5192). Its mechanisms were extended in the current app at the repository root on port 5194; see [connected systems](CONNECTED_SYSTEMS.md).

## One physical engine, controlled assays

The normal physical world, passive-mechanics assay and scarce-resource assay all call `start_physical` and run the same `Habitat::tick`: fluid, chemical transport, gene-regulated synthesis, metabolism, adhesion, material mechanics, contact digestion, gene transfer, fission, mutation and material recycling. They use the selected starting-cell count, seed and cell ceiling. Assays hold the prior physical world; Return restores the held state.

Passive mechanics disables linear and angular actuation; passive elasticity, chemistry, growth and inheritance continue. Scarcity uses the full mechanics engine with initial nutrients set to 0.08. There is no movement score or injected adult body in either assay. These are population experiments, not single-candidate trials.

Legacy locomotion/joint/tissue/competition comparisons remain explicitly labeled. Their supplied developmental chromosomes, tissue identities, recurrent controllers and sexual-mating rules have **not** been copied into the physical engine. Its cells reproduce by fission and can exchange genes on contact. A joint mechanism is available in the physical world; this does not establish spontaneous organs or evolved walking.

## Material bending and antagonistic actuation

Adjacent adhesive contacts between mineral-bearing cells create a three-particle bending interaction. The rest angle is the observed deposition geometry and persists while those contacts survive. The system supplies no limb plan. There are at most three adjacent bending pairs per cell, a mechanical complexity bound.

Each participating cell must have mineral mass fraction above 0.025; this is a numerical material cutoff, not a tissue stage. Stiffness is continuous above that cutoff: `k = 0.08 * min(mineral_fraction)` across the three cells. The elastic energy is `0.5 * k * (angle - target)^2`. Angle gradients are shared with the legacy angular-mechanics code and generate internal forces with zero net force and torque. Soft bending uses no rigid hinge axes or hard anatomical joint limits.

At the central cell, actual contractile polymer availability gates flexion and extension driven by local signals A and B. Their difference shifts the target angle by up to 0.45 radians. Positive elastic-energy increases plus a basal motor cost consume that cell's energy store. Empty stores block activation; the passive assay has no actuation. Broken contacts, loss of deposited mineral or cell death remove the interaction. These are coarse material mechanics, not histological bones or muscles. Mechanical energy is still separate from the chemical energy ledger's accounted input sink.

## Population and computational budget

- Starting organisms: 1–8,192, each supplied as one cell with the same four basic genes. Default UI: 256.
- Total living-cell ceiling: 100–64,000; default UI: 8,192. It counts cells across every body.
- A run's ceiling can be changed without modifying genomes, material stores, resources, RNG, age or time. Lowering below the current living or configured starting population is rejected; cells are never culled to fit.
- Cell division respects the selected ceiling. Initial organisms and grown bodies are distinct quantities. No organism/tissue is unlocked at a cell count.
- Larger starting populations receive their own initial cellular material/energy, tracked in the chemical budget, and share the same environmental pools and spatial habitat. Starting count alone therefore changes scarcity and contact density; it is not a controlled measurement of evolutionary speed.
- New-world creation keeps the previous run for Restore previous world. The preview also stores that checkpoint locally. Save/Open supports files up to 256 MB; very large populations produce large snapshots.

The 64,000 ceiling is a validated configuration guardrail, **not a demonstrated 64,000-cell real-time workload**. Initial populations of 256, 1,536, 4,096 and 8,192 have short WASM conservation/render/save-continuation checks. Their cold, concurrent-build timings are in `scale-results.json`; these are not steady-state hardware benchmarks. Large starting populations are densely placed and can simulate much more slowly than small ones. Start at 256–512 and raise the ceiling to 8,192–16,384 before trying denser starts.

## Requested speed and actual throughput

The browser offers 1×, 10×, 50×, 100×, 500×, 1,000×, 10,000× and Max. One nominal 1× is one simulated cycle per real second. Biology remains at four ticks per simulated cycle, and mechanical substeps remain unchanged. Increasing speed never increases the physical timestep or mutation rate.

The worker yields after a target 25 ms batch (or 256 ticks), publishes less frequently in Max mode and resets outstanding work when paused, restarted or when speed changes. A single slow Rust tick cannot be interrupted. Outstanding requested work is bounded; it does not accumulate unbounded catch-up debt. The UI reports actual cycles per real second, labeled × actual. Max continually uses available worker time; no 1,000× or 10,000× throughput is promised.

Rendering projections are generated once per browser publication instead of separately for each buffer. Contact radii are cached and refreshed whenever digestion transfers material; connected-component lookups compress paths. All cells remain simulated. The world can draw all physical cells; link/joint visuals retain display budgets of 22,000/8,000. The specimen preview shows up to 256 cells and the gene inspector up to 64; these display limits do not change evolution.

## Native batch mode

For experiments without rendering overhead (requires Rust):

```sh
npm run simulate:fast -- --founders 256 --cells 16384 --cycles 1000 --seed 8042 --output examples/native-run.json
```

Resume an exported physical save, optionally raising its cell ceiling:

```sh
npm run simulate:fast -- --input examples/native-run.json --cells 32000 --cycles 1000 --output examples/native-run-next.json
```

The native executable uses an optimized batch profile without whole-program LTO for faster first builds. It runs the same fixed-timestep Rust `Core` and chemical/mechanical rules as the browser. It prints observed throughput, generation, divisions, population and conservation errors. Import its JSON with Open in the browser. Numerical continuation is deterministic within a build/target; native and WASM floating-point trajectories are not claimed byte-identical.

A verified native example is included at `examples/native-108-cycles.json`: 256 starting cells, 16,384-cell ceiling, 108 elapsed cycles, 1,440 living cells, 1,190 divisions and generation 6. Its native save was tested through WASM import. The short 8-cycle native sample ran at 20.77 cycles/second; the following 100-cycle sample ran at 2.34 cycles/second under concurrent compilation as the population grew. These are observations, not general throughput guarantees.
