# Observing ecology and testing functional claims

The Ecology and Evidence tabs observe the main physical engine. Opening either tab does not start an experiment. Trials start only when you press a Run button; runtime exploration and performance testing belong to the user.

## Why a colony may remain stationary

The four supplied founder genes include photosynthesis and environmental uptake. A stationary light-harvesting colony can remain viable without hunting or migrating. Physical movement needs produced contractile material, a driving chemical/excitable signal, available ATP and suitable physical contact. Merely mutating a contraction-production gene does not supply a working controller or gait. Predation needs produced digestive material and actual contact with available prey; protective membranes and bound mineral can reduce extraction. There is no distance-based prey target or awarded movement score.

The planetary chemistry grid is capped at 160 × 80. A 64,000-unit circumference gives 400-unit nominal spacing while initial cells are about 0.44 units wide. A small colony can share one field site; its local resource gradient can be unresolved. Larger geometry alone does not create finer ecology or a reason to migrate. For an initial local-gradient comparison, the 256-unit sphere retains nominal 1.6-unit sites. This is a model-resolution choice, not a demonstrated recipe for evolved animals.

New worlds default to **Patchy food & shade**, under **Seed & placement → Ecology**. Initial environmental fuel is redistributed over deterministic resource patches without changing its total. A spatial light-transmission mask reduces photon input in shaded sites, including photosynthesis, light sensing and photon-powered carrier regeneration. All photon energy still enters the existing ledger. Depletion, transport, weather, death and reproduction continue normally; no genes, organs, fitness reward, rescue or adult body are supplied. Old saves default to diffuse ecology and retain their existing resource fields. New counter fields begin at zero. Extinction remains possible.

## Ecology inspection

- Select a body in **Life**, then open **Ecology**. The map offers fuel, detritus, minerals, oxidized waste and available light. Colors use a logarithmic scale; the legend supplies the actual numerical range. Inventories are divided by each site's area, so larger or polar sites are not mistaken for higher concentrations. Light is shown separately as relative incident availability.
- Click a map site for its quantities; **View inspected site** moves the camera there. **Show this layer in the world** displays the same area-normalized field using fixed-screen-size site markers. These markers show field locations, not material-particle sizes. Point spacing reflects actual grid resolution.
- Body fluxes record measured conversions and transfers rather than inferring a diet from genes. Direct fuel uptake, photosynthetic production, detritus assimilation, prey assimilation, mineral binding, waste, signaling material, protein/matrix/fiber production and internal transport are recorded. Compartment absorption can redistribute fuel already inside the body; it is not necessarily additional environmental food. Counter totals include currently living cells since their birth, not dead ancestors. Daughters and fused embryos begin new counters; counters are saved but are not inherited as biological material.
- Feeding records identify attacker/prey cells and their genome variants at contact, distinguish living biomass from matrix/fiber removal and record actual removed mass. Contacts within a body are allowed and are not automatically called inter-organism predation. A bounded list keeps the latest 24 records, combining repeated identical pairs within 0.5 cycles.
- Defense records the difference between available extraction with and without the modeled barrier/mineral factor, under the same substrate, storage and energy bounds. It measures suppression of attempted extraction, not prevented death or a successful escape.

## Matched checkpoint comparisons

1. Select an existing living body in the main world and open **Evidence**.
2. **Capture selected body & world** holds and pauses that exact state, including inventories, chromosomes, weather and RNG. The checkpoint and comparison results are session-local; save your ordinary world separately before refreshing.
3. Choose 5, 20 or 50 simulation cycles. Run a **baseline** and the control of interest. Every Run restores the same checkpoint first. You control speed and pause; hidden tabs suspend stepping. The worker pauses automatically at the cycle deadline, which can overshoot by one fixed tick. Results report the actual duration.
4. **Without actuation** disables linear and angular contraction plus contraction-driven pressure, using the engine's existing passive mechanics gates. Fluid, gravity and terrain still move cells.
5. **Without tissue transport** skips attached-cell fuel/ATP exchange and internal intercellular fluid/solute/gas circulation. Membrane exchange, metabolism and material synthesis remain active.
6. **Without excitable signals** disables adaptive contact transmission and excitable/refractory dynamics. Local gene regulation and secreted chemicals remain active; this is not an ablation of all sensing.
7. Compare displacement, estimated ambient fluid travel, contraction work, non-light substrate assimilation, prey assimilation, tissue solute transport, population births/deaths and surviving cells. Births/deaths are world totals, not offspring attributed to the selected body. Resource/population edits are blocked during an active trial. Body selection is held during a trial.
8. **Return to checkpoint** restores the original main state and pauses it. Resume to continue ordinary evolution. Export the JSON report before refreshing to keep observations and completed comparisons. Saving an intervention branch as the main world is blocked until it is restored.

Focal-body membership is checked at published snapshots. If a cell is added, lost, or the body splits/merges in those observations, displacement and cumulative body-flux differences are marked inconclusive. Brief changes entirely between samples can remain unobserved. Estimated ambient travel integrates sampled field-speed magnitude; it is not an exact passive-body trajectory or a correction for gravity and terrain. Compare repeated matched trials rather than interpreting translation alone as locomotion.

## What constitutes evidence

An organ candidate needs spatially organized, persistent specialization **and** demonstrated function that benefits the body. Current regional counts mark connected groups of at least three material-bearing cells. Their existence alone does not establish a gut, heart, brain or skeletal system. Transport-disabled comparisons can establish dependence on an intercellular transport mechanism; they do not identify a specific anatomical organ, and the ablation affects the whole world.

Useful movement needs persistent displacement that improves outcomes relative to passive controls. Walking additionally needs a repeatable support/contact sequence and propulsion, with drift, falling, growth and external forcing accounted for. The present metrics and matched controls make these claims testable but do not automatically certify them.

Adaptive signals and changing contact weights are not demonstrated intelligence. Behavioral claims require repeatable improvements on changing stimuli, appropriate sensing/signaling controls and heredity or reproducible learning. A control that changes many mechanisms or an unmatched duration cannot isolate the claimed cause.

## Validation boundary

Rust/WASM and TypeScript/Vite compilation are static/build checks. They do not establish that the latest counters, comparative dynamics, evolutionary outcomes or performance passed runtime validation. No simulation tests, browser simulation or benchmarks were run by the assistant for this revision. Existing native physical tests can be run by the user; earlier validation results do not validate this extension.
