> Historical v0.3 record. Reports and example files at the shared paths have since been regenerated for v0.4; the measurements below describe the earlier revision.

# Validation — 2026-10-09 / local regulation and tissue ecology (v0.3)

## Completed checks

- **25 native Rust tests passed in release mode.** Existing deterministic evolution, inheritance, save/resume, environmental selection, contact feeding and physics checks remain. Five tissue checks cover local differentiation and a regulatory-rule comparison, conservative energy transport, energy-paid repair, injury-reduced actuation, damage retained through growth, and valid extreme 64-cell graphs with support-dependent carrying capacity.
- **Distributed WebAssembly checks passed.** The browser module initialized, returned the expanded particle/link buffers, reproduced, developed colonies, resumed exactly and rejected malformed data. A 100-cycle default run produced 603 organisms, 1,153 births and 57 colonies. Version-one saves migrate without silently adding regulatory programs to legacy organisms.
- **Three tissue studies passed using the actual distributed WASM.** Each begins with 80 single cells, including one explicitly supplied precursor. States were sampled every 10 cycles for 120 cycles. Checks cover body growth, changing roles, support expression, reserves matching total energy, valid save/import and exact resumed simulations. The final rebuilt module repeated these studies with identical results. See [tissue-results.json](tissue-results.json) and the table below.
- **The locomotion comparison remains reproducible.** Baseline active travel 0.8911 versus inactive travel 0.1045 with zero motor work; the best after eight generations traveled 3.2526, spent 8.4841 work and scored 3.0999. It remained supported and made 79 contact transitions. Baseline work was 6.5418. These are model units in an artificial-selection assay. See [locomotion-results.json](locomotion-results.json).
- **Three ordinary 3,000-cycle ecosystem runs passed.** Sampled saves were validated every 100 cycles. Untouched founders reproduced, exchanged traits and consumed cells; large differentiated bodies did not persist in those samples. See [ecology-soak.json](ecology-soak.json) and the ordinary habitat table below.
- **Rust formatting, TypeScript and the final production build passed.** WASM is 452.54 kB (164.73 kB gzip); graphics/UI JavaScript is 594.06 kB (153.44 kB gzip). Vite emits its advisory for JavaScript exceeding 500 kB; compilation succeeds.
- **Live study controls and inspection passed.** Open tissue study introduced 80 single cells. At 14 cycles the supplied lineage had six cells with support/sensory expression. The 20-cycle checkpoint imported with eight solved cells, ten support links, five support cells, three sensory cells, twelve fate changes and 84.5 total energy. Its stores ranged from 8.04 to 11.66 per cell. Return to ecosystem recovered the held ordinary world.
- **Production save/restore across reload passed.** On the separate preview origin, Save world persisted the 20-cycle tissue study. After a full page reload, Restore recovered its eight-cell selection, roles, energy and stores. Return to ecosystem recovered the held 10.5-cycle ordinary world with 319 organisms and 373 cells. The development-origin manual save was not overwritten by this test.
- **Responsive rendering passed.** Desktop width/scroll width were 1440/1440. Phone checks at 390×844 had no horizontal overflow (390/390 after settling; scrollbar-constrained content was also observed at 375 pixels). The tissue table remained readable. A camera-fit correction prevents the larger specimen from clipping in its phone panel and fits the full rotating body. [Desktop](screenshots/tissue-desktop.jpg) and [mobile](screenshots/tissue-mobile.jpg) record the tissue checkpoint. No errors/warnings were captured in either the development or production inspection tabs.

Local persistence, portable JSON import and exact engine continuation are verified. The save UI initiated export, but native download handling is not independently verified in Safari/Chrome.

## Controlled tissue studies

These results establish mechanisms under a supplied starting genotype, not spontaneous invention of anatomy. All studies ran for 120 cycles. Repair energy is cumulative work for the most repaired sampled body, in abstract units. Largest size and fate-change counts are sampled maxima; muscle/support observations indicate at least one sampled expressed link.

| Seed | Largest body | Fate changes | Repair energy | Muscle observed | Support observed | Births | Cells consumed |
| --- | ---: | ---: | ---: | --- | --- | ---: | ---: |
| 8042 | 8 | 19 | 0.0497 | No | Yes | 1,348 | 38 |
| 1197 | 11 | 37 | 0.3756 | Yes | Yes | 1,434 | 47 |
| 6219 | 10 | 19 | 0.2385 | Yes | Yes | 1,392 | 26 |

[examples/tissue-study.json](../examples/tissue-study.json) is an actual grown eight-cell study checkpoint. Body counts and genomes were not edited after introducing its labeled precursor. The 64-cell graph/carrying-capacity checks validate an engineering bound, not an observed ecological outcome.

## Ordinary habitats

All runs used a 1,000-organism budget for 3,000 cycles and sampled every 100 cycles.

| Habitat / seed | Survivors | Births | Cells consumed | Gene exchanges | Largest body |
| --- | ---: | ---: | ---: | ---: | ---: |
| Tidal / 8042 | 238 | 7,251 | 198 | 266 | 3 |
| Volcanic / 1197 | 167 | 5,193 | 17 | 6 | 3 |
| Cold / 6219 | 144 | 4,923 | 10 | 2 | 4 |

No active muscle or support links appeared in these sampled states. Ordinary ecological selection still favored small organisms. These runs do not demonstrate spontaneous evolution of large walking animals. [evolved-ecosystem.json](../examples/evolved-ecosystem.json) preserves an unedited ecological sample; [locomotion-lab.json](../examples/locomotion-lab.json) preserves a separately supplied-precursor artificial-selection champion.

## Performance and limits

This revision was tested while the host was heavily loaded. No new throughput or sustained FPS claim is made. Population budgets remain 1,000/4,000/12,000 organisms, with a shared 64,000-cell budget and 64 cells/256 links per body. Far rendering reduces visible detail without reducing simulated cell count. Requested acceleration can run below its label under load.

Earlier physical-body validation, screenshots and native timing samples are archived in [VALIDATION_V02.md](VALIDATION_V02.md). Those measurements describe the prior revision and are not v0.3 performance guarantees.

Support and muscle links are simplified tissue analogues. Local numeric response rules are heritable but are not molecular gene regulation. Detailed bones, angular joints, organs, arbitrary topology/fragmentation, neural networks, conserved chemistry and sustained ecological walking remain future work. There are no timed anatomy upgrades. See [EVOLUTION_PLAN.md](EVOLUTION_PLAN.md) for the remaining acceptance criteria and [MODEL.md](MODEL.md) for exact assumptions.
