# Validation — 2026-10-09 / shared ecological population (v0.5)

## Shared population

The competing ecosystem starts 24, 48 or 96 supplied single cells from six founder families. Bodies develop through retained division; independent offspring and compatible mating follow the existing ecological rules. Movement has physical costs, finite patches supply food, and smaller unrelated bodies can be consumed at contact. Family totals, individual parent/mate IDs, offspring counts and the latest 256 birth records persist with the world. No movement score or automatic replacement selects descendants.

The default supplied habitat has nutrient renewal coefficient 0.35, scaled by Nutrients, compared with 0.035 in ordinary worlds. This makes continued competition possible while preserving local depletion. Lower nutrients can cause extinction. Starting metabolic, support, muscle and controller capabilities are supplied; this is not proof of spontaneous invention of those functions.

## Engine validation

- **All six distributed WASM suites passed:** core, locomotion, joints, tissue, ecology and population. Existing movement comparisons and three 3,000-cycle ordinary ecological runs remain valid.
- **37 native Rust tests passed.** New tests cover bounded single-cell introductions, extinction without rescue, actual compatible mating with inherited controller coefficients and parent credits, tiny-store repair conservation, and valid saves after ingestion reduces a previously updated prey to one cell.
- TypeScript and production compilation passed. The current module is 599.50 kB / 213.15 kB gzip; graphics/UI JavaScript is about 605 kB / 157 kB gzip. Vite retains its large-chunk advisory.
- Rust formatting passed. Strict save validation remains in place: repair charges are bounded by actual stores, and a consumed body's obsolete physical graph is removed when only one cell remains.

## Distributed population results

The actual browser WASM ran three seeds with 48 single-cell founders for 200 cycles, sampling every ten. Every sample produced a valid exact roundtrip; a further seventeen updates resumed with identical saved state and packed joints. Family births, meals, exchanges and living counts agree with ecological totals. Manual introductions added twelve actual single cells; unknown family references were rejected.

| Seed | Survivors | Births | Gene exchanges | Cells consumed | Peak generation | Peak jointed bodies | Largest body |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8042 | 47 | 157 | 29 | 14 | 9 | 28 | 22 |
| 1197 | 95 | 215 | 43 | 12 | 17 | 32 | 21 |
| 6219 | 46 | 168 | 33 | 21 | 7 | 32 | 27 |

Samples contained up to eight, eight and seven bodies with enabled joints and paid motor work respectively. Food-poor locations occurred in every seed. Family reproduction was unequal and hereditary/controller values varied. These observations establish a functioning shared ecological population; they do not isolate why a particular mutation became more frequent. See [population-results.json](population-results.json). [population-ecosystem.json](../examples/population-ecosystem.json) preserves an unedited seed-8042 checkpoint at 180 cycles, with 51 organisms, 263 cells, 153 independent births, 28 exchanges and 14 consumed cells.

## Browser validation

Production preview was tested on a separate origin, preserving the development-origin manual save.

- Opening Competing ecosystem while paused gave 48 single cells across six families and synchronized the supplied habitat controls. At 10×, a live browser run reached 335.8 cycles, 213 births, 46 exchanges and 14 consumed cells, with 40 living organisms and eight expressed jointed bodies.
- The grown checkpoint imported and displayed multiple physical organisms in the 3D world. Selecting the defense family inspected a 17-cell body with three enabled angular joints, six paired motors, 19.60 paid work and one independent offspring. Manual introduction changed 51/263 organisms/cells to 63/275 without advancing time. Nutrients changed 45→44→45 through its slider.
- Save locally and Restore after a full reload recovered the 180-cycle population, all family counters and recent ancestry. Return to ecosystem recovered the held ordinary world at 14.8 cycles with 405 organisms and 478 cells. Export initiation was observed; Safari/Chrome download handling was not independently tested.
- Desktop 1440×940 had equal client/scroll widths. The requested phone 390×844 view reported 375/375 with its scrollbar and showed readable family cards and ancestry without horizontal overflow. See [desktop](screenshots/population-desktop.jpg), [mobile totals](screenshots/population-mobile.jpg) and [mobile ancestry](screenshots/population-mobile-ancestry.jpg). No console errors or warnings were observed.

## Directional sensing intervention

After v0.5 population delivery, the competing panel gained a directional-sensing switch and cumulative environmental food-energy counter. Genotypes, sensory maintenance charges, random steering, local tissue regulation and joint/contact feedback are preserved when cues are disabled. The switch and counter save exactly; older population fields default to sensing on and zero recorded uptake.

Three paired identical 48-cell starts ran 120 cycles each, sampling valid exact saves every ten cycles. First-update organism energy and consumed fields match on/off before movement can change resource access. Continued saves agree exactly, invalid negative counters are rejected, and additive fields migrate. `check:navigation`, `check:population`, `check:wasm`, all 37 native tests and production compilation passed. Enabling serde_json's precise float parsing fixed a two-trillionths counter drift while retaining exact biological state.

| Seed | Food energy on / off | Births on / off | Survivors on / off |
| --- | ---: | ---: | ---: |
| 8042 | 13315.7 / 13021.6 | 137 / 141 | 68 / 54 |
| 1197 | 15229.4 / 13298.5 | 154 / 124 | 86 / 70 |
| 6219 | 14633.7 / 14243.2 | 142 / 126 | 56 / 57 |

Enabled cues yielded 2.3%, 14.5% and 2.7% more food energy. Offspring differences were -4, +30 and +16. These are cue-removal effects in a supplied population; they do not isolate food cues from prey/mate cues, demonstrate navigation accuracy or prove evolved controllers outperform their ancestors. See [navigation-results.json](navigation-results.json).

The final production browser toggle changed on/off while preserving the paused world. Save and Restore after full reload recovered sensing off, 26,212.1 displayed food energy, 220 births and 46 exchanges at 316.3 cycles. Desktop 1440/1440 and phone 375/375 client/scroll widths show no horizontal overflow; no console warnings or errors were observed. See [sensing desktop](screenshots/navigation-desktop.jpg) and [mobile](screenshots/navigation-mobile.jpg). The toggle retains a stable DOM element during live updates.

## Limits

Predation currently consumes smaller unrelated organisms at contact; equal-size combat and newly evolved weapons are not modeled. Population survival, gene exchanges and controller variation do not by themselves prove that a controller mutation improved resource navigation. Sustained walking, mutable anatomical topology and independently replicated causal fitness comparisons remain future acceptance criteria.

There is no new sustained-FPS or throughput benchmark in this revision. Simulation work remains bounded by the world/body budgets. Requested acceleration can fall below its label under load. Exact saved continuation is validated within each build/runtime; native and WASM floating-point trajectories can differ.

Prior movement, tissue and ordinary ecological protocols and caveats are preserved in [v0.4 validation](VALIDATION_V04.md). See [MODEL.md](MODEL.md) and [EVOLUTION_PLAN.md](EVOLUTION_PLAN.md) for the current model and remaining work.
