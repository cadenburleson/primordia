# Curved terrain and world scale — preview 0.10

The current packaged source is v0.12. It adds 1,024, 8,192 and 64,000 spherical circumferences, with a new 1,024-unit/local-origin preset default. Large planetary fields remain capped at 160×80 sites, while cell dimensions and contact mechanics keep their existing individual scale. See [current setup and limits](CONNECTED_SYSTEMS.md). The v0.10 results below are historical and do not validate the newer implementation or larger spans.

The physical ecosystem can now run on a sphere or a larger planar habitat. This is implemented in the simulation, contact geometry, chemical fields, persistence and renderer. A spherical appearance alone would not provide these behaviors.

## Controls and units

Choose **World geometry**, **World span**, and **Starting distribution**, then **Create physical world**. The browser default is a sphere with circumference 128 model units and founders distributed across its surface. Existing files retain their original geometry. Geometry is a new-world choice; the living-cell ceiling can still be raised during a run.

| Span | Planar field | Spherical field | Sphere radius |
| --- | --- | --- | --- |
| 64 | 40 × 40 | 40 × 20 | 10.19 |
| 128 | 80 × 80 | 80 × 40 | 20.37 |
| 256 | 160 × 160 | 160 × 80 | 40.74 |

Span is square width for a plane and circumference for a sphere. The same span therefore does not imply the same area or carrying capacity. Ordinary founder cells begin approximately 0.44 units across. Increasing span adds habitat and field sites, rather than enlarging cells. Planar field spacing is 1.6 units; spherical meridional spacing is 1.6 and longitudinal spacing decreases toward the poles. Actual spherical cell areas are used in water depth, molecular concentration and transfers.

These are model units. They have no verified conversion to meters, micrometers, seconds, terrestrial climate or evolutionary timescales. Changing simulation speed requests more physical ticks, not a larger timestep or an anatomical advancement.

## Physical interactions

- Cell mechanics remain Cartesian in three dimensions. Broad-phase contacts use neighboring 3D spatial bins, so touching cells can interact across coordinate seams and polar regions.
- Spherical gravity points inward. Its strength follows an inverse-square radius dependence, with reference surface acceleration 0.3 in model units. Buoyancy depends on actual local immersion. Terrain support uses the same height function as the rendered globe, with a local slope normal, contact friction and paid internal contraction.
- Terrain comes from a continuous directional noise function, plus persistent volcanic relief. Longitude wraps; neither the longitude seam nor the poles is a cell wall.
- A viscous, reduced shallow surface-water layer responds to differences in free-surface height. Water depth is water volume divided by surface-cell area. Transfers carry the donor's water, six chemical inventories and flow momentum into adjacent regions. Bed drag and viscosity dissipate motion.
- Longitude neighbors wrap. Opposite meridians connect at both polar caps. Pairwise diffusion uses surface distance and concentration. Small substeps and donor limits keep inventories nonnegative while preserving transferred amounts.
- Cellular drag couples tangential motion to local water, including the opposite reaction to the fluid. Dry cells do not receive water drag. Radial fluid motion is constrained by the surface-layer approximation.
- A changing sun direction gives local day and night. Latitude and illumination affect effective temperature, reaction rates and rain versus snow. Conserved precipitation, evaporation and snowmelt enter the same water inventory that the flow solver transports. Volcanism changes relief and contributes explicitly tracked minerals.
- Fission places daughters in the local tangent/radial frame. Global founders are sampled uniformly in sphere area. A local-origin setup remains available.

The resource overlay, surface-water mesh, actual cell positions, flow vectors, inspection and camera scale use the selected geometry. Cells behind solid terrain are excluded from picking. Save files contain geometry and spherical fluid state; inconsistent dimensions are rejected.

## What is approximate

This is a reduced artificial-life simulation, not a faithful reproduction of all real-life physics. Spherical water is a shallow surface layer with simplified pressure, viscosity, bed interaction and polar connectivity. It is not full 3D Navier–Stokes turbulence, an ocean circulation model, erosion, atmospheric dynamics, plate tectonics, or gravitational interaction between arbitrary celestial bodies. The planar habitat retains the earlier horizontal lattice fluid; it does not acquire the spherical water solver merely by being enlarged.

The latitude-longitude grid has small polar cells. Donor limiting stabilizes them but reduces numerical accuracy at high flow speeds. The solver is not spatially adaptive and does not stream planet chunks. It conserves transferred water and chemical inventories; it does not assert an exact total angular-momentum or mechanical/thermal energy budget for the entire planet. Terrain constraints and bed drag exchange or dissipate mechanical momentum/energy. The chemical energy ledger is separate from that unresolved planetary energy budget.

The chemistry still uses the implemented reaction vocabulary. This change does not supply adult organisms, organs, brains, advanced movement, or new chemistry. It changes the physical environment in which ordinary founders reproduce, mutate and compete. Larger populations and habitats provide more opportunities but do not guarantee increasing complexity.

## Validation

Run:

```sh
cargo test --profile preview --manifest-path engine/Cargo.toml --lib
npm run build:wasm
npm run check:geometry
npm run build
```

Native fixtures check inward gravity and support at different globe orientations; continuous longitude geometry; analytic spherical area; hydrostatic rest; all-species water transport across longitude; positive, conservative polar diffusion; ordinary populations in all six size/shape combinations; exact continuation; and corrupt geometry rejection. Existing planar physical, cellular and ecosystem tests remain in the suite.

The browser-engine protocol and results are in [geometry-results.json](geometry-results.json). It checks the compiled WASM, dynamic field/render-buffer dimensions, finite values, organic/mineral/chemical-energy/water budgets, exact resume, shared physical assays and migration of an earlier planar ecosystem. A longer sphere run starts with 64 ordinary four-gene founders. These checks do not demonstrate evolved anatomical organs or intelligence.

For a native experiment using the same physical engine:

```sh
cargo run --profile preview --manifest-path engine/Cargo.toml --bin physical_batch -- --shape sphere --size 128 --spread global --founders 64 --cells 8192 --cycles 50 --output /tmp/spherical-experiment.json
```

Import the file with **Open** in the preview. Avoid comparing different shapes as if their surface area, total initial resource inventory or population density were identical.

Verified current-build results: 80 native tests passed; all six WASM geometry combinations, shared assays, old-save migration and exact continuation passed; production build and scheduler checks passed. The 64-founder sphere reached 319 cells, 64 colonies and generation 3 after 50 cycles. A separate 256-founder browser globe reached 2,850 cells, 252 colonies and generation 8 at 217.75 cycles, with tracked errors below 1e-7 and no logged console errors. Overview and radial cell-inspection screenshots are in `docs/screenshots/`. These observations do not establish evolved organs or advanced movement. Timings in the results were collected under heavy host load and are not representative performance benchmarks.

The native batch command also compiled and completed a two-cycle spherical smoke run. The earlier browser ecosystem was restored to its preserved 137.5-cycle planar save and left paused; the separate globe preview is paused at 217.75 cycles. No local save was overwritten by the globe validation.
