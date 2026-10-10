# The artificial life model

The default world now uses the [v0.6 autonomous cellular engine](CELLULAR_ENGINE.md), with variable cell-local genomes and no assigned tissue roles. The fixed chromosomes and supplied-precursor studies documented below belong to the retained legacy ecosystem and comparison labs.

## What evolves

The unit of selection is an organism: one cell or an attached clone of cells. Each organism stores a hereditary 16-trait genome, a developmental chromosome, optional local regulatory rules and joint/control coefficients, energy, age, generation, body cell count, lineage, parent and possible mating parent. Genomes are numerical phenotypes rather than DNA sequences.

| Trait | Effect |
| --- | --- |
| Light harvesting | Improves nutrient conversion when starlight is available |
| Chemosynthesis | Improves conversion in mineral-rich regions |
| Mobility | Propels a single cell; multicellular motion is generated mechanically |
| Adhesion | Increases the probability that a daughter remains in its parent's body |
| Armor | Reduces radiation and ingestion risk; extends modeled lifespan |
| Thermal tolerance | Sets the preferred normalized temperature |
| Fertility | Reduces the energy threshold for reproduction |
| Gene exchange | Governs the probability/obligation of finding a compatible partner |
| Shape | Produces elongated, compact, or branching geometry, with different drag |
| Branching | Changes body packing and curvature; branched bodies gain collecting surface at a drag cost |
| Specialization | Improves multicellular metabolic and mechanical coordination |
| Pigment | Changes visible coloration; presently neutral |
| Contractile potential | Expresses paired angular motors; legacy bodies retain tension-only muscle links |
| Structural support | Expresses stiff load-bearing constraints and increases supported cell count |
| Sensing | Perceives nutrient patches, edible prey and compatible mates; biases internal muscle activation |
| Predation | Allows larger colonies to digest exposed cells of smaller foreign lineages on physical contact |

Founders begin as single cells with varied traits. Muscle, support, and specialization begin as weak potentials. Their expression is contingent on an actual body of at least four cells. The initial genomes already contain possible functions; the model does not invent arbitrary new biochemical machinery or an unbounded number of genes.

An additional 12-coefficient **developmental chromosome** controls branch count, reach, elevation, curl, width, motor frequency, phase, activation bias, contact friction, contact feedback, bilateral phase offset, and motor distribution. It mutates and recombines through the same reproductive events. Development uses a bounded budding grammar: four cells can form a braced core, and subsequent retained daughters form branching chains. There is no pre-authored animal mesh or prescribed walking animation, but this grammar is a designed constraint on what shapes can evolve.

## Environment and energy

The world is a circular patch of radius 29 in abstract length units, over a 40×40 nutrient field. Regions differ in resource capacity and mineral availability. Nutrients are consumed locally, regenerate at a bounded rate, and partially return when organisms die. Continuous environmental input means this is an open system; energy and biomass are not a stoichiometrically conserved molecular chemistry model.

For cell count `C`, the approximate collecting surface is `C^0.68`; maintenance scales with `C^0.78`. Light and mineral metabolic traits affect uptake. Solvent availability and thermal mismatch affect conversion efficiency. Adhesive groups receive a simplified cooperative-processing benefit proportional to `ln(C) × adhesion × (0.25 + 0.35 × minerals + 0.15 × nutrients)`. Specialization further improves uptake in colonies with at least four cells. Armor, sensing, expressed muscles and support, unused metabolic machinery, and motion carry costs. Latent muscle/support genes incur no tissue maintenance until their multicellular expression. This supplies tradeoffs; merely increasing every expressed function is not free.

Nutrient regeneration and thresholds are dimensionless illustrative parameters. Temperature is normalized 0–1, not degrees Celsius. Solvent is an availability/efficiency multiplier, not a named chemical. There is no oxygen/carbon chemistry, atmospheric diffusion, pH, ecological succession, or planetary climate simulation yet.

## Feeding and resource seeking

A hungry multicellular organism can ingest a terminal cell from a strictly smaller, unrelated organism when an actual body particle touches it in 3D. Ingestion probability depends on digestion expression, relative body size, armor and expressed structural defense. Each meal removes one prey cell, transfers its stored energy plus a fixed biomass allowance, returns the unassimilated fraction to the nutrient field, and starts a digestion cooldown. A consumed single cell dies once; a damaged colony retains a valid smaller graph. Food can fund the ordinary growth and reproductive rules. No adult body is introduced by a meal.

The assimilation fraction is `0.25 + 0.55 × predation`; digest time is `2 + 4 × (1 − predation)` cycles. This is an abstract ingestion mechanism, not digestive organ anatomy. Explicit avoidance of the same lineage is a simplified kin recognition assumption. Larger size opens smaller prey opportunities but increases maintenance; support can raise the body-size limit and reduce ingestion risk.

Sensory expression expands bounded neighborhood searches and samples distant nutrient patches. Hunger favors edible targets; gene exchange permits seeking compatible mates. The resulting direction biases contractile activation through inherited control coefficients. It does not teleport a colony, apply forward propulsion, or guarantee efficient navigation. Useful resource access affects energy, survival and offspring in the main ecosystem; the separate lab uses measured travel as artificial fitness.

## Reproduction and body growth

Sufficient energy and a completed cooldown permit a reproductive event. Adhesion can retain the divided daughter inside the organism, increasing its cell count at an energy cost. Larger colonies shed small multicellular propagules. Other divisions produce a one-cell daughter.

Reproduction inherits the parental genome. A per-gene chance, influenced by radiation, adds bounded random variation. A sufficiently gene-exchanging organism searches for a nearby, energized, compatible partner. Offspring can recombine parental traits. Highly obligate exchangers wait for a partner; intermediate strategies can divide without one. This is a toy recombination model, not a detailed model of animal mating or meiosis.

Lineages branch when mean squared trait distance from the lineage's reference ancestor exceeds 0.018. The UI's lineage counts are operational genetic clusters, not a scientifically validated species boundary. Gene compatibility uses a separate distance threshold.

## Local cell regulation, transport and tissue health

Newly introduced founders inherit six tissue-response rows of six signed coefficients, encoded in 0–1, plus four size/retention/transport/repair coefficients. These are synthetic regulatory rules. They mutate/recombine during offspring formation; attached daughters retain the colony genotype. The input cues are bias, graph exposure, local food signal, stress signal, neighbor differentiation signal and axial rest-position polarity. Food/stress/identity signals change with local sources and diffuse one connection neighborhood per ecology update. A modest hysteresis prevents rapid fate flicker. There is no fixed cell-index assignment in regulated bodies.

Tissue potentials remain gated by the metabolic genome; a response rule cannot invent absent biochemical functions. Support or muscle differentiation starts at four cells; digestion can be expressed in smaller colonies. Tissue fate selects stiff or active physical connections. Digestive cells ingest more effectively; metabolic cells can also ingest at a lower rate. Injured cells below 25% health cannot feed. Healthy reproductive tissue modestly lowers the energy threshold for offspring.

Incoming food credits the collecting cell. Transport exchanges reserves conservatively along intact adhesion/support connections, with donor-limited flux and inherited conductance. Local maintenance and motor work drain stores; deficits weaken cells. Muscle force is limited by local reserve availability, cell health and link integrity. Repair consumes cell energy and repairs damaged connections using energy from both endpoints. Excessive strain fatigues connections; integrity at or below 8% disables the constraint and its transport. Consumption of a terminal prey cell wounds neighboring cells. Growth retains existing health and link damage. Mechanical support also changes the modeled structural density in water.

Carrying capacity depends on adhesion, healthy support cells and inherited growth investment, up to 64 cells per body. Without healthy support the cap is at most 16. Retention probability is inherited; adults shed propagules through the existing reproduction rule. A 64,000-cell world budget bounds further growth, single-cell births and introductions. At fission/growth, available energy is redistributed proportionally across retained stores; propagules rebuild their body and begin with newly distributed stores. This is an explicit simplified reproductive reset, not literal tissue partition mechanics.

Single cells retain their scalar energy. Version-one/two worlds import without adding a regulatory program to their existing organisms. Version-three saves retain regulatory programs, stores, signals, health and link integrity. Version-four adds heritable joint/controller coefficients and physical joint state. Earlier imported organisms retain their prior anatomy; new founders use the current joint model. A save with malformed health/signals/weights or excessive body/world counts is rejected. Existing manual saves are preserved until the user saves again.

The **Tissue study** is a separate controlled ecological starting condition: 80 single cells, one supplied strong-tissue precursor, real ecosystem nutrient rules and survival/reproduction. It demonstrates the mechanisms without presenting its designed founder as a spontaneous evolutionary result. The previous ecosystem can be restored after exploring or importing a study.

## Physics and morphology

The Rust solver now represents each multicellular cell as an individual 3D particle with a position, velocity, radius, and differentiated role. Actual cell division adds particles; fission partitions the cell budget and offspring inherit the developmental program. New ecosystem bodies have per-cell energy stores and transport along intact adhesion/support connections. Cells forage at their actual field positions; metabolic cells absorb more efficiently than other tissues, scaled by local health. The displayed organism energy is the sum of those stores. Legacy saves and the controlled locomotion assay retain the earlier shared-energy, positional-role model.

New bodies carry an **anatomy/control chromosome** with five scalar traits and two rows of eight network weights. Traits control flexion/extension range, torque, mineral stiffness and response rate. Branch links are nearly inextensible support segments. Each cell after the four-cell core can create an angular joint between three connected cell centers. Its minimum and maximum angles depend on inherited traits and rest geometry; flexor and extensor constraints apply bounded opposing torques. This is an angular particle model, not a full rigid-body hinge with a torsional axis.

Two recurrent controller units sense bias, inherited oscillator timing, distal contact, normalized joint angle, sensed food/mate direction, proximal contact and both previous unit states. Signed weights are encoded in 0–1; bounded tanh output and response smoothing drive each motor. A healthy sensory cell enables resource direction input in regulated bodies. Each joint stores its own recurrent state. The anatomy chromosome mutates and recombines through ecological reproduction and controlled selection.

Healthy support and intact supporting links express a joint. Local muscle reserves, health and strength gate force. Work is estimated from angular force and realized angle change with a small isometric allowance, scaled into energy units; it is not calibrated biological joules. Existing tissue metabolism charges this work and repair consumes local stores. Fractured support disables the joint. Legacy bodies and the original locomotion assay retain stiff distance links, compliant braces and force-limited linear muscles under their previous timing/contact rules.

Each quarter-cycle includes 15 physics substeps and eight XPBD iterations for jointed bodies, four for legacy bodies. The body solver applies gravity, height-field contact and friction, cell self-contact, water damping and submerged-volume buoyancy, and basin boundaries. The Rust height function and rendered terrain use the same deterministic integer noise. Multicellular organisms receive **no forward propulsion force**; motion comes from internal actuation and environmental reactions. Neighbor separation remains a coarse body-level external contact approximation. Soft bodies can float; increasing structural density can make them sink. Single cells use an analytic surface equilibrium shared by drawing and ingestion. Decorative rocks and crystals are not individual colliders.

The renderer draws solved cell positions and link endpoints. Ivory denotes support, coral contractile cells/links, and blue sensory cells. Far graphics may display fewer cell instances, while every cell is still simulated. Rendering interpolates sampled poses; it does not drive the solver. Contact transitions alone do not prove walking, and moving bodies may crawl, shuffle, roll, or hop.

The support links are mechanical analogues of mineralized tissue, not a model of vertebrate bones. Angular joints limit bending between particle segments; cuffs and motor bands diagram their attachments. They do not model anatomical hinge axes or torsion. Mutable developmental topology, molecular regulation, detailed tissue constitutive laws, arbitrary fragmentation/necrosis and richer nervous systems remain future work.

### Locomotion experiment

The optional **Locomotion lab** holds the current ecosystem and supplies an explicitly designed single-cell precursor predisposed to adhesion and differentiation. Nutrients and daughter retention are controlled in this assay; it is artificial selection rather than the ecosystem's unrestricted natural survival/reproduction. Cells are added by division every half-cycle up to twelve. Development and settling take eight cycles; net travel, work, viable geometry, and supported posture are measured for the remaining twelve cycles. Growth displacement is excluded.

Twelve candidates form a generation. The elite is retested unchanged, and other descendants vary morphology and control together. Fitness is `net distance × (0.65 + 0.35 × fraction supported) − 0.018 × work`, with nonviable trials rejected. “Watch best body” regrows and replays the champion through the same solver while holding selection. Both experiments and replays can be saved and resumed. The held ecosystem is also stored under a separate browser-storage key for returning after loading a lab save.

The assay can demonstrate improved locomotion under these rules. It cannot establish spontaneous macroevolution in the ecosystem or guarantee a walking lineage. The **Jointed locomotion lab** uses the same trial windows with angular motors and recurrent control. Live inspection reports angle limits, flexor/extensor activation, work and dominant-side stance changes. Disabling motors restarts the same genotype, holds selection and measures an unactuated comparison; reenabling restarts again. A constant-network comparison is also measured by the test suite. Stance changes and intermittent support do not certify walking. See [locomotion-results.json](locomotion-results.json) and [joint-results.json](joint-results.json) for actual distributed-WASM results.

## Competing population experiment

**Competing ecosystem** supplies 24, 48 or 96 single cells from six founding families. Their initial numerical traits favor different combinations of light/mineral uptake, digestion, defense, sensing and mechanical investment. Every founder has local regulatory rules and a joint/controller program; no adult body is inserted. Small founder variation and subsequent mutation/recombination provide diversity. The initial population is concentrated in a shared habitat so local food and contact interactions matter.

The experiment runs the existing ecosystem loop: distributed food uptake, energy-paid movement/repair, starvation, contact ingestion of smaller unrelated organisms, retention, propagule formation and compatible mating. There are no movement-scored candidate trials or prescribed winners. The habitat starts with finite resource stocks and renewal coefficient 0.35, scaled by the Nutrients setting; ordinary habitats use 0.035. This supplied environment is chosen to support continued competition. It is not a calibrated planetary chemistry model. Reducing nutrients lowers capacity and renewal. Extinction is permitted and the toy abiogenesis rescue rule is disabled in this mode. New competing founders are introduced only through an explicit manual action.

A family identifier follows the reproducing parent through new lineage formation. Mates contribute hereditary traits, so family totals are not a full ancestry tree or permanent species labels. Cumulative family counters record independent offspring, gene exchanges, consumed cells and deaths, including extinct members. Individual offspring counts credit both parents when they mate. The last 256 birth records retain parent/mate links, generation, metabolic traits and joint/controller coefficients; the interface shows the most recent records. Older dead genomes outside that window are not archived.

The display compares current family sizes and mean digestion, sensing, muscle and controller variation from each supplied family program. Controller variation includes founder diversity, mutation and mating; it is not a fitness score. Jointed-body and generation peaks survive extinction in the saved experiment. Food-poor locations count living organism centers whose local patch holds less than 0.5 abstract resource units.

Changing reproductive success and inherited traits establish an ongoing ecological population experiment. They alone do not establish that a particular controller adaptation caused better navigation, or that an untouched world invented new anatomical/chemical functions. Attack currently means the existing contact-feeding rule, not arbitrary weapons or equal-size combat.

## Directional sensing comparison

The competing panel can disable directional sensing as an experimental intervention. This removes local nutrient-gradient steering and food/prey/mate direction signals sent to motor control. It preserves inherited genomes, perception neighborhoods, random steering, sensory maintenance costs, joint/contact feedback and local tissue regulation. Disabling mid-run changes the current world; restore the same saved state before the alternative run for a paired comparison. It does not remove reproductive compatibility or contact feeding.

Cumulative food energy sums actual environmental uptake multiplied by the existing energy conversion of three. It excludes ingestion, founder stores, nutrient pulses before uptake and internal transport. The value is historical and does not equal current stores. Old population files start this new counter at zero when imported. Sensing-enabled state and uptake totals persist in additive version-four fields.

`check:navigation` compares identical 48-cell starts across three seeds over 120 cycles. It reports food energy, births, mating, feeding, survivors and cells without rewarding navigation or requiring the enabled case to win. Population divergence alone does not establish evolved controllers are superior to founders; that requires a further common-environment inherited-program comparison.

## Origins, extinctions, and time

Seeded experiments introduce 160 founders manually. Chemistry-only experiments start with zero organisms. Every three cycles, while fewer than eight organisms survive, a suitability-dependent Bernoulli trial may introduce eight single-cell candidate replicators. That trial is a speculative convenience, not a molecular explanation of how the first cell formed. A world with no available solvent cannot sustain life. No complex form is introduced or unlocked based on time.

A cycle consists of four quarter-cycle updates. It has no mapping to Earth years. Acceleration requests more fixed updates; under heavy load the worker runs as quickly as its bounded batches permit. The display reports simulated time actually completed.

## Research inspiration

- Ratcliff et al., [Experimental evolution of multicellularity](https://pmc.ncbi.nlm.nih.gov/articles/PMC3277146/): postdivision adhesion, cluster-level selection, propagules, and early division of labor. The model borrows these conceptual mechanisms; it does not reproduce the yeast experiment quantitatively.
- Koschwanez et al., [Improved use of a public good selects for the evolution of undifferentiated multicellularity](https://pubmed.ncbi.nlm.nih.gov/23577233/): resource access can favor attached clusters. The simulator's multicellular metabolic benefit is a simplified analogue.
- Jacobeen et al., [Cellular packing, mechanical stress and the evolution of multicellularity](https://pubmed.ncbi.nlm.nih.gov/31723354/): shape and mechanics constrain body size. Primordia uses a bounded physical budding graph and body-size limits; it does not reproduce the experimentally measured tissue stresses.

## Further development

The full implementation plan and acceptance criteria are in [EVOLUTION_PLAN.md](EVOLUTION_PLAN.md). Physical bodies, local regulatory differentiation, cell-energy transport, damage/repair, larger supported growth and a measured locomotion assay are implemented. Angular limits, paired motors and simple recurrent controllers are implemented. Mutable developmental topology, rigid-body joint axes/torsion, richer nervous systems, sustained ecological walking, conserved chemistry and connected volumetric habitats remain later milestones.

Additional primary references: [Sims, Evolving Virtual Creatures](https://www.karlsims.com/papers/siggraph94.pdf) for joint evolution of morphology and control, and [Macklin et al., XPBD](https://matthias-research.github.io/pages/publications/XPBD.pdf) for compliant constraint dynamics. These motivate the architecture; Primordia does not reproduce their experiments quantitatively.
