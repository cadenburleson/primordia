# From cells to embodied evolution

The goal is an observable, heritable route from a replicating cell to differentiated physical bodies whose shape and control can change through selection. No timer creates an animal, and no animation substitutes for simulated locomotion.

The ecological route is **retained division → ingestion of smaller lifeforms → food-funded growth and offspring → differentiated support and contractile tissues → movement toward resources and mates**. These are opportunities subject to selection, rather than obligatory stages. A large body must earn enough energy to pay for itself. A useful movement program can spread through inheritance.

## Cell-local genetic rewrite (implemented v0.6)

The default app world now uses variable per-cell genomes, local regulatory expression, abstract fission, replication errors including gene duplication/deletion/insertion, contact gene transfer, physical retained-daughter bonds, energy transport and connected-component bodies. All founders are identical basic replicators. This bypasses the previous fixed organism chromosomes, tissue classifications and developmental grammar. See [CELLULAR_ENGINE.md](CELLULAR_ENGINE.md).

Verified acceptance: copied information and mutated sequence length; context-dependent expression under unchanged sequences; real paid division and retained graphs; finite stores and packed rendering; paid internal contraction without forward thrust; extinction without rescue; exact save continuation; three seeded ecological runs. Current bodies stay small. Tissue coordination, organ organization and evolved multicellular reproductive advantages remain unestablished. This architecture enables those experiments but does not award those outcomes.

The milestones below describe retained legacy implementations and earlier comparisons. Their supplied chromosomes are not added to the v0.6 autonomous world.

## Milestone 1 — physical developmental bodies (implemented)

- Retained division adds an individual cell particle. Cells form a bounded graph under a heritable developmental program.
- A 12-coefficient developmental chromosome controls branching, reach, elevation, width, curl, motor frequency/phase/distribution, contact feedback, and friction. It mutates and recombines alongside the existing metabolic chromosome.
- Differentiation produces metabolic, support, contractile, and sensory roles. Structural constraints carry load; compliant braces allow joints to flex; active links pull with bounded force under inherited oscillator parameters and contact feedback. Passive elasticity restores joint posture when muscles relax.
- Rust solves gravity, ground contact, friction, self-contact, water damping/buoyancy, and internal constraints at 60 substeps per cycle. Multicellular bodies receive no forward locomotion force.
- Distributed cells forage at their actual positions and share the body's energy budget. Hungry colonies physically ingest exposed cells from smaller foreign organisms; digestion transfers energy, removes prey cells, and recycles the remaining material. Sensory signals from food, prey and compatible mates bias inherited motor control. Contraction incurs work costs. Motion changes resource access and hence survival/reproduction.
- The world and specimen viewers display solver positions and active links. Save files preserve every particle, velocity, controller clock, and chromosome. Version-one cellular saves migrate without deleting their organisms.

Acceptance: stable developing bodies, no thrust from internal contraction in free space, active/passive comparison, bounded cells/links, reproduction inheritance, save continuation, native and actual WASM verification, visible browser validation.

## Milestone 2 — measurable selection for locomotion (implemented)

Add a separate **Locomotion lab**, inspired by evolved virtual creatures. A clearly identified supplied precursor grows from one cell. Trials use the same body solver and developmental grammar as the ecosystem. Each trial has 6 cycles of development, 2 of settling, and 12 of measurement. Twelve candidates comprise a generation.

Net horizontal travel, contraction work, viability, and load-bearing posture determine artificial selection. The elite is retested unchanged; descendants mutate morphology and control together. The viewer exposes the current phase, support/muscle counts, contact transitions, travel, work, baseline, and best measured score. Growth and settling displacement do not count toward fitness.

This assay demonstrates that movement can be selected. It is not evidence that an untouched ecosystem spontaneously evolves an animal. Crawling, shuffling, rolling, and stepping are possible mechanical outcomes; the interface reports measurements rather than awarding a “walking” badge.

Acceptance: reproducible nonzero net movement, comparison with inactive motors, improvement over the baseline under fixed conditions, measured support/contact behavior, exact resumed experiments/replays, and user-visible controls for opening/leaving the lab and watching the champion.

## Milestone 3 — richer development and tissue ecology (local regulation implemented)

The ecosystem now has a heritable regulatory chromosome with six response rules and four growth/transport/repair coefficients. Local food, stress, exposure, neighboring signals and axial position determine metabolic, support, muscle, sensory, digestive and reproductive cell states. Signals diffuse along intact connections. Fate changes affect absorption, ingestion, mechanical links and reproductive thresholds.

Every multicellular body holds per-cell energy stores. Actual-position uptake credits those stores; intact structural/adhesion connections transfer energy conservatively. Maintenance, contraction and repair consume it. Local starvation weakens cells even when another region retains food. Overstrain damages links, broken links cease constraining the body, and predation wounds the neighbors of a consumed terminal cell. Growth preserves existing health and damaged connections.

Healthy expressed support, adhesion and inherited growth coefficients determine carrying capacity up to 64 cells per body. Growth and introducing founders share a 64,000-cell world budget. Newborns inherit, mutate and recombine regulatory rules alongside morphology and metabolic traits.

**Tissue study** supplies 80 single cells including one precursor with stronger tissue potentials. It uses ordinary ecological growth, feeding, transport and survival/reproduction rather than movement-scored selection. The supplied starting genotype is labeled explicitly, and the previous ecosystem is held. The study can be saved/imported; its state retains the regulatory chromosome and all cell stores/signals/health. The older locomotion assay deliberately remains the same mechanics comparison.

Verified: local differentiation, rule-removal comparison, conserved transport, energy-paid repair, injury-reduced actuation, preservation of damage during growth, 64-cell graph bounds, three replicated supplied-precursor studies, valid sampled states and exact resumed WASM simulations.

Still required before this milestone is considered complete: fully mutable topology beyond the budding grammar, per-cell metabolic pathways/chemical resources, arbitrary tissue excision and fragmentation, internal necrosis/turnover, differentiated propagule construction and rigorously replicated tissue-fitness comparisons. The 64-cell cap is an engineering bound; it does not establish spontaneous growth to that size in an ordinary habitat.

## Milestone 4 — joint mechanics and control (implemented); sustained walking (remaining)

New founders carry an additional anatomy/control chromosome: five coefficients set bending limits, torque, segment stiffness and controller response, and sixteen coefficients form a two-unit recurrent network. Growth creates nearly inextensible mineralized branch segments and three-particle angular joints. Each joint has flexion/extension limits and two opposing finite-torque motors. The network senses joint angle, distal/proximal contact and resource/mate direction, with recurrent state and inherited oscillator timing. These coefficients mutate and recombine alongside development and local tissue regulation.

Healthy support expresses the joints. Muscle health, local energy stores and intact supporting connections gate actuation. Joint work consumes energy; existing injury and repair rules still apply. The solver produces internal forces, so environmental contact is needed for horizontal travel. Growth and saves preserve each joint's limits, state and work. The 3D views display solved segments, joint cuffs and paired motor attachments; the inspector reports angles, limits and activation.

**Jointed locomotion lab** grows a supplied precursor from one cell and selects measured travel using the same development/settling/measurement protocol. Disabling motors restarts the same genotype with actuation off and holds selection; reenabling starts a fresh trial. Three distributed-WASM seed trials improved distance from a 0.366-unit baseline to 1.88–5.68 units after ten generations. Inactive motors and a constant controller yielded 0.0065 and 0.00015 units respectively. See [joint-results.json](joint-results.json).

Verified: finite segment and angular mechanics, bounded joint limits, energy-paid paired actuation, free-space momentum, injury/store gating, structural-fracture collapse, 64-cell developmental bounds, inherited controllers, exact saved continuation, motor/controller comparisons and repeated selection. This establishes selectable controlled locomotion. It does not establish sustained walking: some champions move without alternating support, and support is intermittent.

Still required: rigid-body hinge axes and torsional limits beyond the current particle bending model; independently replicated sustained alternating support and clearance over rough terrain; navigation, feeding and mating advantages demonstrated in ecological comparisons; and richer control/topology that allows those behaviors to evolve without a designed gait. No human-like body is required. A walking claim requires those observations, not only contact transitions.

## Shared ecological population (implemented)

The competing ecosystem supplies multiple single-cell families with varied joint/control, metabolic, defensive and digestive potentials. They share finite patches and reproduce through the ecological rules. Family counters, mean inherited traits, expressed joint-body peaks, maximum descent and recent parent/mate birth records make population change observable. Founder count, nutrients, mutation pressure and population budget are user controls; additional introductions are manual. Save/restore includes the population and held ecosystem.

Acceptance: multiple mechanically active bodies in the same habitat; real ecological offspring, compatible mating and contact meals; changing inherited traits and unequal family reproduction; valid bounded graphs/stores/ancestry; exact saved continuation; extinction without automatic rescue; visible multi-organism rendering and controls. Supplied founder functions and resource renewal are disclosed.

Directional sensing can now be removed as an intervention while keeping genotypes, sensory costs and mechanical feedback. Paired equal-start headless runs report actual environmental uptake and reproduction; the interface supports save/restore comparisons. This tests cue effects in the ecological model, not improvement of evolved controllers over founders.

Remaining: independently controlled experiments showing that evolved navigation or hunting causes a fitness advantage, sustained terrain locomotion, general attack/defense beyond smaller-prey ingestion, and a long-term genome/ancestry archive. No timer or distance-score optimizer substitutes for ecological selection.

## Milestone 5 — habitats, scale, and open-ended experiments (future)

Add conserved chemical resources and selectable metabolic pathways, volumetric aquatic habitats, climate gradients, richer prey defenses and digestive organs, migration, long-term lineage archives, and headless batch experiments. Profile native Rust and browser WASM separately. Use threads/native packaging and selective physics resolution only after measuring bottlenecks; rendering LOD must not silently change biology.

Acceptance: reproducible ecological comparisons across multiple seeds, documented approximations, stable long-run populations without numerical explosions, archived ancestry and experiment exports, and measured throughput on the user's Mac.

## Scientific boundaries

The implemented model is artificial life. Mineralized connections are mechanical analogues of support tissue; they do not simulate osteogenesis. Paired angular motors and legacy contractile links are muscle analogues; they do not model protein motors or histology. The initial genotype contains allowed functions, and the development grammar is designed by us. Molecular abiogenesis, arbitrary new biochemistry, and guaranteed Earth-like anatomy are not implied.

References: [Karl Sims, Evolving Virtual Creatures](https://www.karlsims.com/papers/siggraph94.pdf) for joint evolution of morphology and control; [Macklin, Müller and Chentanez, XPBD](https://matthias-research.github.io/pages/publications/XPBD.pdf) for compliant constraint dynamics. Primordia uses its own bounded model and does not reproduce either paper's experiments.
