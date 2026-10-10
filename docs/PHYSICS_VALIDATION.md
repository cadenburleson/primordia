# Physical habitat validation — 2026-10-09

## Automated verification

The initial full native suite passed all 51 tests (44 existing checks plus seven physics checks). The final expanded physics suite passed all ten focused checks after the material-cache optimization. Together these cover 54 distinct native tests. Final production TypeScript/Vite build passed. The existing basic WASM initialization, packed buffers, legacy evolution, save/resume and corruption suite also passed.

The physics checks establish resting-fluid mass/equilibrium, unforced vortex dissipation within the low-Mach domain, diffusion positivity and mass preservation, equal/opposite cell-solvent drag momentum, catalyst production before activity, mineral availability before deposition, paid contraction and its inhibition by empty ATP stores, conserved material splitting during division, coupled chemical accounting, exact continuation and rejection of inconsistent chemical stores.

The optimization caches unchanged mechanical material properties and bond indices within each tick. A 20-update continuation from the same approximately 1,000-cell checkpoint produced a byte-identical serialized future to the earlier implementation. Cold timing samples are in `physics-cache-baseline.json` and `physics-cache-results.json`; other running jobs strongly affected timing, so they are not a general speedup benchmark.

## Three population trials

Each trial begins with 64 identical supplied replicators, the same four basic production pathways and the same habitat parameters except RNG seed. There is finite initial matter, tracked sunlight energy, no uniform resource renewal, no timed unlock, and no movement score. Each trial runs 1,200 updates (300 cycles), sampled every 40 updates. The test cap is 1,000 cells. These are seed replications, not a comparison between planetary climates.

| Seed | Final cells | Divisions | Peak generation | Peak connected body | Living genome IDs | Paid contraction input | Mean WASM update ms |
|---|---:|---:|---:|---:|---:|---:|---:|
| 8042 | 998 | 1282 | 7 | 192 | 210 | 0.010159 | 50.4 |
| 1197 | 999 | 1294 | 6 | 196 | 207 | 0.050841 | 21.6 |
| 6219 | 1000 | 1293 | 7 | 222 | 204 | 0.002538 | 25.4 |

All 90 sampled saves roundtrip exactly. Stored materials stay finite/nonnegative. Render buffers remain finite, and link endpoints refer to valid cell records. A second loaded world produces exactly the same saved state after another twelve updates. Tampering with a chemical pool is rejected. Maximum organic residual across trials was 3.55e-10 model mass units; mineral residual was 1.60e-12; chemical-energy residual was 6.89e-9. These are bookkeeping checks of the designed reaction network, not measured accuracy against real chemistry or full thermodynamic conservation of numerical mechanics.

All trials reached their engineering population cap, showed random inherited sequence changes, contact association/dissociation, digestive extraction, bound mineral, and some paid contraction. Larger connected components can also include cells joined on contact rather than descended entirely from one founder. The experiments do not demonstrate tissue differentiation, organs, walking, adaptive locomotion, or a multicellular fitness advantage. Tiny paid contraction totals are not evidence of a muscle system. Generation values reflect individual-cell descent; recent division records do not archive every genome.

Raw results: `physical-results.json`. Example checkpoint: `../examples/physical-world.json` (seed 8042, after the trial and twelve continuation updates).

## Browser verification

Verified in the Codex in-app browser at the isolated development origin `http://127.0.0.1:5192/`:

- Real 3D cell bodies and graph links, live population statistics, material/catalyst gene labels and conserved-balance panel.
- Measured fluid arrows and the nutrient overlay respond to the layers control.
- Expanded per-cell records reveal the actual synthesized stores and retain their open state across UI refreshes.
- A jointed comparison lab held the physical habitat. Returning restored the same 93.8-cycle display and 866 living cells.
- Saving enabled local Restore. Restart produced 64 cells at cycle zero; Restore returned the physical state to the same 93.8-cycle display and 866 cells.
- No browser warnings/errors were captured during these checks.

The UI reported successful export; automatic capture of the download event timed out, so the exported file itself was not confirmed through browser automation. Native/WASM JSON serialization and loading were independently verified. Mobile viewports and a packaged native Mac executable were not tested. The Mac launcher runs the browser app locally.

Screenshot: `screenshots/physical-desktop.jpg`.

## Performance and limits

Large-world update averages were roughly 22–50 ms in the Node/WASM trial harness on this machine, with other jobs running. The browser showed about 13 ms per update at 866 cells after optimization and about 60 rendering FPS while paused. This is not a sustained high-speed GPU/device benchmark. Worker work is bounded to preserve controls; selected speed can exceed what the physics kernel can actually sustain. The current model is bounded to 1,024 cells and a 40×40 horizontal solvent lattice. It is not yet a large-scale volumetric planetary simulation.

See `PHYSICAL_HABITAT.md` for equations, defined reactions, inventory/energy accounting, physically grounded assumptions, designed rate constants, and remaining limits.


## v0.8 integration and scale verification

- Full native library suite: 61 tests passed after adding shared material-joint mechanics and population controls. Following the contact-radius/component optimization and sharp-angle correction, all 18 focused physical tests passed (62 distinct native tests across these runs).
- Final distributed WASM: starting populations 256, 1,536, 4,096 and 8,192, each with a 64,000-cell engineering ceiling; short conservation checks, valid finite render buffers, exact save continuation, rejected invalid/lowered ceilings, shared full/passive/scarcity configurations, older physical save migration, and no hidden actuation without contractile material. See `scale-results.json`. A 64,000-cell living workload was not benchmarked.
- Captured four-tick future remained byte-identical after contact-radius caching and component path compression.
- Scheduler checks cover requests above 100x, Max, measured throughput, fixed ticks, bounded outstanding work, pause/reset and rejected nonfinite speeds.
- Native batch: 256 founders, eight cycles, exact native save export; resume with ceiling 16,384 for another 100 cycles reached 1,440 cells, 1,190 divisions and generation 6. Both the initial save and the 108-cycle save imported and advanced in WASM with conserved material. The latter is in `examples/native-108-cycles.json`.
- Browser: changed ceiling from 8,192 to 16,384 while retaining 1,254 living cells; created a reversible 1,536-founder world; restored the held run; opened passive and scarce-resource assays using the same engine and returned to the exact 78.8-cycle held world; accepted 1,000x and Max controls and responded to Pause. A Max sample displayed approximately 12.1x actual throughput at roughly 1,000 living cells under concurrent build load. Saved locally, then restored that 479.5-cycle run in the rebuilt browser engine.
- Production TypeScript/Vite build and WASM generation passed. The bundler's large-chunk advisory remains. Native first-time compilation was slow on this run; the batch profile avoids whole-program LTO, and a local compiled cache is retained for subsequent runs.

These checks establish available mechanics and configurable experiments, not evolved organs, sustained walking, superiority of a genotype, or universal high-speed performance. Legacy supplied-body labs remain comparisons; their developmental chromosomes, recurrent motor networks and sexual mating are not silently introduced into the physical world.
