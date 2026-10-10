# Active world tools

World creation lives in **New world** in the header. A successful creation closes the setup dialog. The floating Shape drawer operates on the living world; habitat presets, seed, size, starting count and placement only appear in the creation dialog. Labs remain in their separate header dialog.

## Environment and surface

- **Apply environment** sets available sunlight, baseline temperature and persistent global cloud cover. Temperature is a normalized model value, not Celsius. Seasons, days and local conditions continue after resuming. Cloud cover attenuates light by up to 80% and cools the model baseline slightly; it adds no cloud meshes or postprocessing.
- **Place a rain system** pauses, then lets you click a surface location. A 12-cycle storm moves existing atmospheric water into surface water or snow when evolution resumes. The storm footprint spans at least three field spacings, so it remains resolved on large planets. There must be atmospheric water to precipitate.
- **Shape → Surface**: choosing a brush opens a compact canvas toolbar and pauses time. Hold the primary pointer and drag to paint a continuous stroke; the brush stays selected after release. Right-drag or Alt-drag orbits, the wheel zooms, and Done / Escape exits painting. The Size slider and `[` / `]` set the footprint diameter. Strength controls each distance-spaced dab. Transform stretches a circle into an ellipse and rotates it in the local east/north frame. A terrain-following outline previews the footprint. Raise, lower and flatten edit the actual relief used by terrain contact and surface flow. Relief is bounded to ±8 model units relative to the original seeded landscape. Flatten blends neighboring ground toward the fixed surface height where the stroke begins; an amount of 1 is full blending before edge falloff. Affected connected bodies are translated together clear of newly raised ground. It is an external intervention, not energy produced by the organism.
- Water, dissolved fuel, minerals and detritus are added to actual field stores with surface-area weighting and brush falloff. Amount is mass per unit area; for spherical water this corresponds to added depth. Added matter, chemical energy and water are included in conservation baselines and recorded in Notes. Existing planar habitats retain their reduced fixed water-contact/display-depth approximation; their added water changes the stored water and cycle rather than providing a resolved ocean.

Field resolution is unchanged. On a 64,000-unit planet the capped 160×80 field has 400-unit spacing; a resource/terrain brush cannot resolve microscopic variation within a field site. Transform shows spacing and enforces a minimum narrow radius of 0.75 spacings, including stretched brushes. The clicked site's field cell is always included. Terrain height differences are best inspected locally, not from orbit. Resource map legend values report the new measured inventories. Terrain and resource displays refresh after an edit while paused.

## Stroke processing and undo

Pointer paths are sampled by distance rather than event frequency. Spherical paths use the short great-circle arc, including across the longitude seam. A missed surface, pointer cancellation, focus loss or Escape ends the gesture. Touch paints with one finger; exit painting to use the ordinary touch camera gestures.

Only one worker batch is in flight, with at most 12 dabs per batch and 256 queued points. Input beyond that bound is reported as clipped; use shorter strokes. Batches are throttled while dragging, and previews publish at most four times per second plus the final state. Arming or hovering has no simulation work. A stroke serializes one undo checkpoint; each batch validates and clones physical state once, not once per mouse event. This bounds work but is not a measured frame-rate guarantee on large populations.

The worker locks other world actions until the drag is finalized. **Undo** restores the entire world from before the first successful batch, including time, RNG and organisms. If a later batch fails, earlier paint remains and the original stroke checkpoint stays available. Releasing or cancelling does not roll back a stroke. Save a world file for durable preservation.

## Seasonal experiments

Climate → Seasons exposes Steady (0), Gentle (0.5), Natural (1), and a variation slider through 2. Year length is 120, 240 or 960 simulation cycles in the UI (the engine validates 48–2,400). A season changes baseline temperature by up to ±0.18 × strength and sunlight by ±15% × strength, with the existing model bounds and weather/local modifiers. Solar inclination also follows the seasonal signal. Daily forcing, weather, rainfall, snow and runoff remain active when seasonal strength is zero.

Older saved planets default to strength 1 and 240-cycle years, preserving the previous forcing. Values persist in Planet state and undo. Changing the year length rephases the signal from current simulation time; it does not change genes, mutation frequency or evolutionary speed. The guide prepares a gentle long-year experiment; the user still applies it explicitly.

## Organisms

**Place a starting cell** supplies the same four-gene, single-cell founder used at world creation, then lets you place it on clear terrain. Click the button again for additional founders. It does not supply an adult body or specialized genes.

**Copy selected organism** freezes the entire connected body and arms placement without requiring collection storage. Copies retain chromosomes, materials, age, injuries, contacts and learned signaling, and receive fresh identities. Added cells must fit the living-cell limit. Existing collision and specimen validation applies. Collection entries can also be placed from the Collection tab.

**Edit selected cell genes** pauses and opens the chosen body's first 64 inspectable cells. It edits only one cell's primary chromosome: existing gene bias, strength, promoter weights, removal, or addition from the engine's chemical action vocabulary. It does not alter a homologous chromosome, inject proteins, instantaneously remodel tissue, or prescribe walking. Bias and promoter weights are bounded to −2…2; strength to 0…2; chromosomes retain 1–48 genes and 28 regulatory input weights. The edited cell receives a new genome identity. Subsequent expression, material synthesis and inheritance follow the engine's ordinary rules. The journal marks the change as manual; natural mutation and birth counters are unchanged.

Edits and introductions pause the world. Resume explicitly to observe responses. These tools are unavailable inside legacy studies, physical assays and matched-checkpoint comparisons. Editing invalidates an older functional-assay checkpoint, so future comparisons must capture the edited world anew.

**Undo last world edit** holds one session-local full checkpoint, including RNG, cells, resources, selection and time, from before an environment/terrain/resource/gene edit or specimen placement. Undo rewinds the whole world, including any subsequent evolution; it is not a reverse brush. Invalid actions do not replace the checkpoint. Switching worlds or entering a comparison clears it. Save world provides durable export/local storage.

## Verification boundary

This revision was compiled with one Rust build worker and checked with TypeScript and the production bundler. The small `check:brush-ui` controller check covers path sampling, seam wrapping, bounded messages, cancellation, batch failures and whole-stroke undo using fake worlds. No simulation, benchmark, browser interaction or native/WASM runtime test was run, at the user's request. Runtime correctness, evolved outcomes, visual appearance and large-world performance remain for the user's testing.
