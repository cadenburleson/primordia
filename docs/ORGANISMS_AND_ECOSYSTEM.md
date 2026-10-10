> Current connected implementation: [v0.12 systems](CONNECTED_SYSTEMS.md). Development, transport, materials, signaling, reproduction, particles and larger globe spans are assembled in the same physical engine. The material below records an earlier stage; it is not validation of v0.12.

Current biological extension: [Inherited development, adaptive contact signals and reproductive cycles](LIFE_ENGINE_ROADMAP.md). The v0.9 description below is historical; functional organs remain unproven.

Current curved-world extension: [Spherical scale and physical transport](SPHERICAL_SCALE.md). The earlier results below used the original 64-unit planar domain.

# Material physiology and a changing ecosystem — v0.9

This implementation is the active `primordia/` application on port 5194. Earlier implementations are preserved under `archive/`. This is an extension of the actual physical population engine; shared passive and scarcity assays use it too. The old supplied adult-body/recurrent-controller labs remain labeled legacy comparisons.

## Inherited development and coordinated bodies

Cells copy and mutate variable-length genomes at fission. There are sixteen available production reactions and sixteen promoter inputs. Forty-eight genes, six contacts per cell, three bending constraints per center cell and the configured living-cell budget remain engineering bounds. Extra pathways are absent in founders. Local chemistry, ATP, actual neighbors, cellular excitation/recovery, receptor-mediated resource change, injury/strain, exposure, lipid fraction and a local membrane-axis resource gradient regulate production. There is no complete resource map or remote prey target available to a cell.

Division splits existing biomass, fuel, proteins, minerals, lipid and energy. Division planes are now three-dimensional. Cells inherit their parent's chemical/regulatory state, while lifetime meal/work counters begin anew. Daughter attachment and stochastic contact association create actual connected bodies. Association can also join unrelated cells. There is no imposed organism size, limb grammar or organ stage. Cellular reproduction and contact gene copying are not sexual reproduction; organism-level reproductive bottlenecks, germ lines, stable body-wide genotypes and sexual mating remain future research mechanisms.

Mineral deposits change particle mass, size, buoyancy and elastic stiffness. Mineral-bearing adhesive contacts resist stretch/bending. Actual contractile polymers produce energy-paid changes to bond lengths and bend targets. Paired angular drives read local chemical A/B and excitable/refractory states. Forces remain internal and equal/opposite; displacement comes from interaction with the fluid and substrate. No target velocity or teleport toward food is applied. The original low-Reynolds-number horizontal fluid model remains, so this is not calibrated macroscopic running or a full 3D ocean.

## Reserves, sensing, transport, attack and repair

Lipid-like material stores reduced carbon when ATP and fuel are abundant, consumes ATP to store it, and releases actual fuel under shortage when the appropriate catalyst exists. It counts in conserved organic material and chemical energy. Its effective lower density changes radius/drag/buoyancy. It isn't a free energy multiplier or detailed vertebrate fat physiology.

Local receptors make resource change, damage and stretch available to regulatory programs. A short-lived nutrient average provides cellular adaptation, not learned intelligence. Synthesized excitable channels support a bounded nonlinear activation/recovery system, integrated in eight substeps. Existing adhesive contacts and carrier material couple the cells synchronously. Channel maintenance spends ATP; absent channel material cannot latch a free excitation-driven motor. Excitation and recovery also feed back into promoters, making locally coupled control possible without a supplied brain or oscillatory gait.

Contact carriers redistribute actual fuel by concentration difference and existing ATP-like stores, bounded by donor availability and recipient capacity. Floating-point ATP transfer losses go to heat. No global colony energy pool rescues a disconnected starving cell. Carrier material is already synthesized/maintained through the reaction budget; the exchange itself is modeled as passive facilitated transport.

Contact digestive catalysts remove actual prey biomass. Mineral deposits and protective membrane polymers reduce cleavage. Digestion is energy-gated, limited by available biomass and assimilation space, causes measured injury and yields 85% fuel / 15% organic excretion. There is no supplied predator category, kin immunity or kill upgrade. Repair catalyst consumes fuel and ATP to replace damaged biomass. Injury reduces material function, and terminal damage recycles the cell into the habitat. Strained contacts lose integrity; matrix material and ATP can repair them. Matrix-dependent substrate friction changes traction but does not inject propulsion.

The inspector displays actual stores, cellular excitation/recovery, wounds, strain, genes and material colors. Eight overlapping function classes are measured from actual material stores. A region is at least three connected cells bearing a function; a mixed body bears at least three functions. These thresholds affect observation only. A region is **not proof of a functional organ**, coordinated feeding apparatus or persistent differentiation. Anatomical organs must be demonstrated through repeatable function and heritable development, not inferred from a count or color.

## Resource and planetary cycles

The solute grid contains reduced fuel, oxidized waste, dissolved mineral, chemical A/B and organic detritus. Ten percent of transporter uptake is unassimilated organic excretion. Prey digestion adds its unassimilated fraction. Dead cells return fuel to the fuel pool, their remaining organic inventories to detritus and their minerals to the mineral pool. Detritus has lower modeled diffusivity than soluble fuel. A new inherited catalyst can cleave it into assimilable fuel with an ATP cost. Respiration produces oxidized waste. That oxidized pool has zero remaining chemical energy under this model; it cannot become food without an explicit source of energy. Photosynthesis reduces it using tracked photons.

Day forcing has a 48-cycle period and season forcing a 240-cycle period, in model time rather than Earth time. Light, moisture and temperature modify reaction rates. Stochastic storms transfer existing atmospheric water to localized liquid rain or snow, depending on temperature. Evaporation returns liquid to the atmospheric inventory; melting transfers snow to liquid. A bounded donor-cell runoff operator moves existing water downhill and entrains existing solutes. It conserves all six species; rain does not generate nutrients. This is an effective water/reservoir model, not atmospheric fluid dynamics, cloud microphysics, real catchment hydrology or resolved lakes.

Stochastic eruptions add smooth local ground displacement, transient effective heat and an explicitly tracked dissolved-mineral input. The physical contact solver and rendered terrain use the same interpolated relief. Volcanic heat can wound nearby cells. Mineral balance subtracts the geological input; carbon receives no volcanic bonus. Terrain displacement exchanges work with an external geological system; full mechanical/thermal energy accounting is not solved. Rain/snow particles are visual indicators of the simulated weather state, not individually simulated drops. Snow cover is drawn from the actual snow inventory.

Organic, mineral, chemical-energy and water residuals are displayed. The inherited chemical budget includes external photon energy and sinks for heat/paid mechanical work; it is not proof of total thermodynamic conservation of every mechanical/thermal process.

## Persistence and validation

Old physical saves pad 10-protein/10-expression arrays to 16, eight promoter weights to 16 and five solutes to six, using zeros for additions. Lipid and cellular/planetary state have explicit defaults. Existing genetic programs and molecules are preserved on import. New planetary state initializes when an old world first ticks. Malformed intermediate array lengths and invalid state/inventories are rejected. Futures change across engine versions; same-build save/restore continuation remains exact.

Native mechanism tests use deliberately constructed controlled fixtures to isolate storage, fission, transport, signal propagation, paid mechanical displacement, defense, wounding, repair, excretion/decomposition, rain/snow/melt/runoff, volcanic contact geometry and conserved balances. They also check corrupt state rejection, unchanged founder pathways and migration. Fixtures establish that mechanisms function; they are **not naturally evolved animals or demonstrations of adaptive foraging**.

The WASM ecological check starts only ordinary founders, samples conserved state and observed functions across multiple seeds, validates buffers, restores state and rejects corruptions. Results are saved to `organisms-results.json`; they report observed values without requiring organs, movement success or a particular evolutionary destination.

## Measured validation — October 9, 2026

- Exact-source native suite: **72 passed, zero failed**. Final WASM build, TypeScript checking, production build and fixed-step scheduler check passed.
- The native `physical_batch` runner also passed an eight-cycle smoke run with 64 ordinary founders and a 4,096-cell budget, validated its final state and exported a save. That short run took 0.153 seconds after compilation; it does not establish long-run throughput for a mature large population.
- Three ordinary populations, each beginning with 64 four-gene founders, ran for 250 model cycles. Tidal / cold alien / volcanic runs ended with 609 / 708 / 248 living cells, 54 / 36 / 31 multicellular colonies and maximum generations 8 / 7 / 10. No specialist population was supplied.
- Actual detritus breakdown occurred in tidal and volcanic runs (19.707 / 20.856 model units); cold-run breakdown remained zero. Small digestive transfers, repair and storage/transport activity occurred, with outcomes differing by seed/environment.
- The tidal run briefly contained one mixed-function body and six material joints. None of these runs produced measured excitation spikes. These results do **not** establish an organ system, evolved gait, adaptive foraging or a stable decomposer niche.
- Sampled absolute organic/mineral/chemical-energy/water residuals stayed below 1.6e-8 model units, with exact save continuation and corruption/migration checks passing. This covers the modeled chemical inventories, not complete geological thermodynamics.
- Browser smoke check: the isolated port-5194 preview reached 137.5 cycles with 941 cells, 97 colonies and generation 6; detritus overlay and Max scheduling worked. The preview was paused for inspection, with no browser console errors. Screenshot: `screenshots/organism-ecosystem.png`.
- The distributed WASM passed short scale checks with 256, 1,536, 4,096 and 8,192 founders under a 64,000-cell ceiling, including budget edits, finite rendering, conservation, exact continuation and old-save import. The sharp-angle/no-motor-material regression passed too.
- Performance is measured throughput, not the selected multiplier. The eight-tick Node/WASM samples measured roughly 321 / 538 / 1,584 / 2,174 ms per tick respectively on this machine during validation. These short samples are not isolated benchmarks; record details are in `scale-results.json`. A 64,000-cell ceiling is not evidence that 64,000 active cells run interactively, and Max cannot guarantee accelerated time with a large population.

## Research basis and remaining uncertainty

Cell-local regulation and coordination are motivated by artificial-development research, but these particular rates and bounded equations are designed approximations, not reproductions or validated predictions of those papers:

- [Real-time Evolution of Multicellularity with Artificial Gene Regulation](https://arxiv.org/abs/2305.12249) describes a shared artificial regulatory representation for cellular specialization and multicellular evolution.
- [Improving Evolvability of Morphologies and Controllers of Developmental Soft-Bodied Robots](https://www.frontiersin.org/journals/robotics-and-ai/articles/10.3389/frobt.2015.00033/full) investigates coupled development of morphologies and controllers. Its novelty-search optimization is not copied into this ecosystem's selection rules.
- [Cellular plasticity model for self-organized phenotypes in multi-cellular robots](https://www.nature.com/articles/s44182-025-00039-y) provides evidence that local reaction/diffusion-based interactions can organize mechanical properties and coordination in a designed robotic substrate. It does not establish emergence of biological organs from this simulator.

There is no claim of brains, synaptic learning, true bones/muscles/fat histology, organogenesis, multicellular sex, evolved walking, successful directed foraging, stable trophic niches or open-ended evolution. Those require further mechanisms and controlled ecological evidence. The engine now allows more functional interactions, and reports what actually occurs rather than awarding anatomical progress.
