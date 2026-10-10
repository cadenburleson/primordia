# Validation — 2026-10-09 / autonomous cellular engine (v0.6)

## Protocol and observed evolution

The default world now delegates biology to the cell-local engine. All 64 founders have the same four-gene sequence: light uptake, mineral uptake, division investment and weak adhesion. No muscle/support chromosome, assigned tissue role, target body size, organism steering, movement objective, adult introduction or automatic rescue runs. Environmental conditions and replication-error probabilities are fixed during a run. See [CELLULAR_ENGINE.md](CELLULAR_ENGINE.md) for the implemented rules and bounds.

The distributed browser WASM ran three seeds for 300 cycles, sampling every ten cycles. Each sample passed strict save validation and exact roundtrip. Twelve further updates after reload reproduced the same physical/cellular saved state. Packed render buffers had valid indices and finite values. Invalid action targets were rejected. Actual records are in [autonomous-results.json](autonomous-results.json).

| Seed | Living cells / bodies | Divisions | Attached daughters | Living genomes | Genes/cell | Peak descent | Largest body |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8042 | 443 / 433 | 3,189 | 78 | 120 | 3–7 | 10 | 3 |
| 1197 | 469 / 460 | 3,124 | 84 | 132 | 3–7 | 7 | 3 |
| 6219 | 419 / 413 | 2,821 | 64 | 125 | 3–7 | 8 | 3 |

These runs produced 333/377/274 point edits, 129/146/127 duplications, 82/55/82 deletions and 247/241/229 new-gene insertions respectively. Contact gene transfers were 0/1/12. Cells sharing a genome showed different local expression; this is regulation, not proof of distinct tissues. New action expression was observed, but ecological paid contraction work remained zero in all three runs. No organs, walking or multicellular fitness advantage were demonstrated. The engineering bounds are 1,024 living cells and 48 genes per cell; this is a supplied replicator with a finite action language, not abiogenesis or unlimited biochemical invention.

[autonomous-world.json](../examples/autonomous-world.json) is an unedited seed-8042 checkpoint at 90 cycles, the first sampled three-cell component. It contains 572 living cells in 563 bodies, 125 living genome variants, 1,290 divisions and 26 attached daughters. No traits or bodies were modified to create it.

## Engine and regression checks

- 44 native Rust tests passed, including seven autonomous tests for minimal founders, local expression, sequence mutation operations/bounds, division/exact continuation, paid internal contraction, starvation extinction without rescue, and corrupt saves. The seven focused tests passed again after adding genome-ID/sequence consistency validation.
- Final Rust/WASM compilation, TypeScript checking, production build and Rust formatting passed. The final autonomous WASM suite passed after that save-validation change.
- Core, population and directional-sensing WASM regressions passed with unchanged legacy results. Earlier locomotion, joint, tissue and ecology protocols remain documented in [v0.5 validation](VALIDATION_V05.md) and its prior revisions; they are supplied-precursor comparisons, not autonomous organ evolution.
- Production WASM is 740.79 kB / 257.62 kB gzip. Graphics/UI JavaScript is about 610 kB / 158 kB gzip. Vite retains its large-chunk advisory.

## Browser checks

Production preview was tested at a separate localhost origin, preserving the development-origin manual save.

- New worlds opened directly with 64 identical cells, one genome and four genes per cell. Autonomous conditions, introductions and nutrient pulses were disabled. The capacity label correctly shows the 1,024-cell bound. At 10×, a live run reached 187.3 cycles with 336 living cells, 334 bodies, 81 genome variants, 2,134 divisions, 50 attached daughters and generation nine. Genome lengths varied from three to six genes.
- The inspector showed actual gene sequences, parent cells and current expression. A new contraction gene appeared in a descendant; this did not establish useful muscle function. The imported three-cell checkpoint contained two genome-1 cells and a daughter with genome 300 and a duplicated light-uptake gene. The viewer rendered their connected body.
- Save locally, full reload, pause and Restore recovered the 187.3-cycle checkpoint and its exact displayed totals. Restore preserves the current pause state; restoring while running advances normally. Opening the older jointed lab displayed its supplied candidate and comparison rules. Return recovered the held autonomous world at 187.3 cycles.
- Desktop 1440×940 reported equal client/scroll widths of 1440. The requested 390×844 phone view reported 375/375 with its scrollbar, with readable cell programs and evolution counters and no horizontal overflow. The inspector was corrected to stack cell programs instead of squeezing them into parallel columns. See [desktop world and counters](screenshots/autonomous-desktop.jpg), [cell inspector](screenshots/autonomous-genome.jpg) and [phone](screenshots/autonomous-mobile.jpg). No final browser console warnings or errors were observed.

## Remaining scientific and performance limits

Selection is actual survival and reproduction, but these runs do not isolate which changes improve fitness. Gene expression and daughter retention establish a foundation; stable coordinated tissues, organs, organ systems and locomotion remain unproven. No timed upgrade guarantees their arrival. New functions still combine our defined physical/chemical primitives. Horizontal gene transfer is not sexual reproduction/meiosis, and division is an abstract genome-copying fission model rather than a molecular mitosis simulation.

There is no new sustained-throughput benchmark. Browser HUD samples near 60 FPS do not establish performance at the 1,024-cell ceiling. Rendering uses transient adapters for the existing 3D viewer; they do not participate in heredity or biology. Exact continuation is checked within the same build/runtime; native and WASM floating-point trajectories can differ.
