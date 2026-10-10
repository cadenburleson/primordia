# Validation — 2026-10-09 / angular mechanics and recurrent control (v0.4)

## Engine checks

- **33 native Rust tests passed.** Existing ecological, feeding, tissue and locomotion checks remain. New checks cover finite supported segments and joint limits, opposing muscle work, horizontal free-space momentum, sensed control, injury and empty-store gating, structural-fracture collapse, joint/controller continuation, inherited coefficients, and development through all 64 cells with bounded joint indices.
- **All distributed WebAssembly suites passed:** `check:wasm`, `check:locomotion`, `check:joints`, `check:tissue`, and `check:ecology`. These execute the actual browser module. Sampled saves validate, experiments resume exactly, packed cell/link/joint buffers agree with body graphs, and malformed ranges/indices/weights are rejected. An actual v0.3 checkpoint imports with its prior mechanics intact.
- **Jointed selection replicated across three seeds and ten generations.** Every candidate grows from one supplied cell to twelve under the same 6-cycle development, 2-cycle settling and 12-cycle measurement protocol. Travel before measurement is excluded. Inactive-motor and constant-controller comparisons use the starting program. See [joint-results.json](joint-results.json).
- **The original locomotion comparison remains reproducible.** Active baseline travel 0.8911 versus inactive 0.1045 with zero work; its selected champion travels 3.2526, spends 8.4841 work and scores 3.0999. See [locomotion-results.json](locomotion-results.json).
- **Rust formatting, TypeScript and production compilation passed.** The module is 504.33 kB / 181.11 kB gzip; graphics/UI JavaScript is about 600 kB / 155 kB gzip. Vite reports its existing large-chunk advisory; compilation succeeds.

## Jointed movement

Distances and work are abstract model units. Supported fraction measures elevated posture during samples; stance alternations count changes in the dominant contacting side. Neither measure certifies a walking gait.

| Seed | Baseline distance | Selected distance | Selected work | Supported fraction | Stance alternations | Maximum limit error |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 8042 | 0.3658 | 2.1162 | 2.4589 | 0.667 | 17 | 0.00373 rad |
| 1197 | 0.3658 | 5.6798 | 11.6464 | 0.750 | 10 | 0.00752 rad |
| 6219 | 0.3658 | 1.8821 | 4.7466 | 0.854 | 0 | 0.00867 rad |

The unactuated starting body travels 0.00647 units with zero motor work. Constant controller coefficients produce 0.000147 units and 0.0800 work. These comparisons establish controlled movement that selection can improve. The seed-6219 champion has no stance alternations; the seed-1197 champion has no contacts at its final sample. Sustained walking and rough-terrain traversal are not established.

[examples/jointed-lab.json](../examples/jointed-lab.json) is a seed-8042 champion replay at 13 trial cycles, with twelve actual grown cells and eight joints. Its genotype was selected by the assay; adult body counts were not edited. The initial precursor and allowable functions are intentionally supplied.

## Tissue and ordinary ecology

Supplied-precursor tissue studies ran 120 cycles, sampling every ten. Muscle observations include expressed linear links or enabled angular joints with paid work; support observations indicate expressed support links. The report records sampled maxima, not the history of every individual. See [tissue-results.json](tissue-results.json).

| Seed | Largest body | Fate changes | Repair energy | Muscle observed | Support observed | Births | Cells consumed |
| --- | ---: | ---: | ---: | --- | --- | ---: | ---: |
| 8042 | 16 | 36 | 0.02388 | Yes | Yes | 1,351 | 34 |
| 1197 | 9 | 34 | 0.00633 | Yes | Yes | 1,456 | 40 |
| 6219 | 6 | 9 | 0.02089 | No | Yes | 1,321 | 31 |

Stores sum to total organism energy; growth, local differentiation, transport and repair produced valid resumable states. [tissue-study.json](../examples/tissue-study.json) preserves a grown checkpoint. The 64-cell development test validates an engineering bound, not a naturally evolved 64-cell animal.

Three ordinary habitats used a 1,000-organism budget for 3,000 cycles, sampling every hundred. See [ecology-soak.json](ecology-soak.json).

| Habitat / seed | Survivors | Births | Cells consumed | Gene exchanges | Largest body |
| --- | ---: | ---: | ---: | ---: | ---: |
| Tidal / 8042 | 258 | 7,311 | 137 | 1 | 3 |
| Volcanic / 1197 | 161 | 6,140 | 32 | 3 | 3 |
| Cold / 6219 | 119 | 4,056 | 43 | 23 | 3 |

No active muscle or support was observed in these ordinary samples. Small organisms still dominate under these conditions. Supplied-precursor movement does not establish spontaneous macroevolution. [evolved-ecosystem.json](../examples/evolved-ecosystem.json) preserves an unedited ordinary ecological sample.

## Browser checks

The production preview was tested on a separate local origin, preserving the development-origin manual save.

- Opening the jointed lab starts one supplied cell; the live trial develops twelve cells and eight joints. The inspector shows sixteen paired motors, solved angles/ranges, activation, work and stance measurements. The 3D world and specimen show solved support segments, cuffs and opposing motor bands.
- The evolved champion imports. Disabling motors resets to one cell, holds selection, and regrows the same program. A developed inactive replay displayed zero flexor/extensor activation, zero work and about 0.01 units of net travel. Reenabling starts a fresh trial.
- Save world and Restore across a full reload recover the champion's twelve cells, eight joint ranges/angles, activation, work, generation and trial position. Exact recurrent/physical continuation is also verified directly by the WASM suites. Return to ecosystem recovered the held 282-cycle ordinary world with 193 organisms and 222 cells after reload. Native export initiation and local persistence work; download handling in Safari/Chrome is not independently tested.
- Desktop layout at 1440×940 has no horizontal overflow. The requested 390×844 phone view also has no overflow; its scrollbar-constrained capture reports 375/375 client/scroll width. The complete specimen fits its phone panel, and joint angle/activation rows remain readable. Mechanics and lab measurements span both columns on small screens. See [desktop](screenshots/joints-desktop.jpg), [mobile specimen](screenshots/joints-mobile-body.jpg) and [mobile joints](screenshots/joints-mobile.jpg), [mobile lab](screenshots/joints-mobile-lab.jpg) and [mobile world](screenshots/joints-mobile-world.jpg). An initial missing Bone icon warning was corrected; the final build produced no observed console errors or warnings.

## Limits and next acceptance criteria

This revision does not include a new throughput or sustained-FPS benchmark. The 64,000-cell world budget, 64 cells / 256 links / 60 joints per body, and rendering LOD bound work. Requested acceleration can fall below its label under load. Rendering detail does not change biology.

Exact determinism/resume is verified within each build/runtime; native and WASM trajectories can diverge numerically because of floating-point differences. New founders use the new anatomy program. Older imported organisms retain their previous rules.

The angular solver models bending between particle segments, with nearly inextensible support and energy-paid paired motors. It does not model anatomical hinge axes, torsional constraints, bone histology or molecular muscle biology. Remaining milestones include mutable topology, richer tissue/chemical resources and control, sustained terrain walking, and ecological advantages from navigation/feeding/mating shown across repeated trials. See [EVOLUTION_PLAN.md](EVOLUTION_PLAN.md) and [MODEL.md](MODEL.md).

Historical records are [v0.3](VALIDATION_V03.md) and [v0.2](VALIDATION_V02.md); they describe prior revisions rather than current performance guarantees.
