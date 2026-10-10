# Validation — 2026-10-09 / physical bodies and feeding

## Completed checks

- **20 native Rust tests passed.** Deterministic seeds, exact save continuation, inheritance and recombination, finite resources and recycling, caps, extinction, retained division, environmental selection, legacy migration, graph validation, body stability, motor work, and exact resumed selection experiments. New feeding checks verify actual contact, smaller nonkin prey, energy transfer, food-funded growth with no environmental uptake, and valid terminal-cell loss. Opposite sensory directions alter muscle activation while preserving horizontal center-of-mass motion in free space.
- **Distributed WebAssembly checks passed.** The actual browser module initialized, returned packed organism/particle/link buffers, evolved reproducing colonies, resumed exactly, and rejected corrupt data. A 100-cycle default run produced 502 organisms, 980 births, and 63 colonies.
- **Controlled locomotion comparison passed in distributed WASM.** Baseline active travel: 0.8911; inactive motors: 0.1045 with zero work; best after eight generations: 3.2526, work 8.4841, score 3.0999. The champion maintained supported posture during measurement and made 79 contact transitions. Baseline active work was 6.5418. All distances and work are model units. Detailed records are in [locomotion-results.json](locomotion-results.json).
- **Three 3,000-cycle ecosystem runs passed.** Sampled states were validated by save/import every 100 cycles. Ordinary founders produced ingestion events in all three worlds. Their genomes and cell counts were not edited. See the table below and [ecology-soak.json](ecology-soak.json).
- **TypeScript and production build passed.** Final compiled WASM: 414.96 kB (152.90 kB gzip); graphics/UI JavaScript: 590.68 kB (152.34 kB gzip). Vite emits its advisory for JavaScript exceeding 500 kB; compilation succeeds.
- **Production browser startup and mechanics passed.** The packaged worker, WASM and 3D body loaded with no captured errors/warnings. The champion inspection displayed 12 solved cells, 14 support links, 16 motor links, live contacts, contact transitions and work. Resuming at 1× visibly changed the body measurements; replay regrew the champion on repeated trials.
- **Save/restore across reload passed.** The lab was saved at 2069.0 cycles, with the champion measurements retained. After full page reload, Restore returned to that lab state. Return to ecosystem restored the held 0.0-cycle, 160-organism, 160-cell Tidal Garden. This was tested on the separate preview origin so the user's manual save in the development origin was preserved.
- **Main ecosystem inspection passed.** At 301.5 cycles, the browser displayed 208 organisms, 238 cells, 28 colonies and 54 cells consumed. A naturally developed three-cell colony showed one meal and its digestion state. The journal recorded ingestion at 9.5 cycles and physical contractile-link expression at 12.5 cycles. These observations establish early functions, not sustained animal locomotion.
- **Responsive layouts passed.** The production page had width/scroll width 1440/1440 at 1440×940 and 390/390 at 390×844. Both layouts rendered the solver body; the phone page had no horizontal overflow. New evidence is [desktop](screenshots/locomotion-desktop.jpg) and [mobile](screenshots/locomotion-mobile.jpg). The original PNG screenshots document the earlier version.

The save control confirmed local persistence and initiated export. The in-app browser's earlier download-event capture did not yield an exported file, so native download handling is not independently verified in Safari/Chrome. Portable JSON import and exact engine save continuation are verified.

## Ordinary ecological runs

All three runs used a 1,000-organism budget and sampled every 100 cycles.

| Habitat / seed | Survivors | Births | Cells consumed | Gene exchanges | Largest sampled body |
| --- | ---: | ---: | ---: | ---: | ---: |
| Tidal / 8042 | 236 | 7,235 | 213 | 1 | 4 |
| Volcanic / 1197 | 177 | 4,642 | 32 | 0 | 3 |
| Cold / 6219 | 144 | 3,827 | 46 | 1 | 4 |

No active muscle or support links were present in the 100-cycle samples in these three final runs. Short-lived expression can occur between samples, as observed in the separate browser journal. These runs still favored small organisms; they do not establish spontaneous evolution of large walking animals. [evolved-ecosystem.json](../examples/evolved-ecosystem.json) contains a genuine sampled four-cell ecosystem state. [locomotion-lab.json](../examples/locomotion-lab.json) is a clearly supplied-precursor artificial-selection champion, not an untouched ecological outcome.

## Native throughput samples

Two release runs of 200 fixed updates on this development Mac, while browser graphics were active:

| Initial organisms | First mean ms/update | Second mean ms/update | Survivors | Colonies |
| --- | ---: | ---: | ---: | ---: |
| 1,000 | 12.19 | 13.97 | 593 | 251 |
| 4,000 | 20.08 | 33.80 | 809 | 131 |
| 12,000 | 31.61 | 115.20 | 1,114 | 44 |

The first run overlapped build/validation work. Host load and these deliberately crowded developing populations make timings variable. The values average across births/deaths and changing physical-body counts. They are not sustained 12,000-organism browser or GPU throughput guarantees. The ordinary ecosystem browser sample displayed about 60 FPS and 0.6 ms/update with 208 survivors. That is a local observation, not a broad hardware benchmark. Worker batches bound compute per batch; requested acceleration can be lower than its label under load.

## Remaining scientific and engineering boundaries

Support and muscle links are simplified tissue analogues. They solve physical movement, but do not model vertebrate bone formation, histology, rigid angular joints, digestive organs, recurrent nervous systems, or molecular abiogenesis. The inherited budding grammar allows at most 24 cells per organism. Neighbor separation and kin recognition are explicit simplified rules. Energetics include a biomass allowance and open environmental input rather than conserved molecular chemistry.

The controlled assay establishes selectable physical motion and useful load-bearing support. Contact transitions and travel alone do not establish a full walking gait. Natural resource selection currently often favors simplicity; the next planned work is richer developmental regulation, tissue ecology, and independently measured sustained locomotion. There are no timed anatomy upgrades. The complete milestone plan is [EVOLUTION_PLAN.md](EVOLUTION_PLAN.md), and the exact current rules are [MODEL.md](MODEL.md).
