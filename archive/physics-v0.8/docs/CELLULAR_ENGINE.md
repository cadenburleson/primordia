# Autonomous cellular engine (v0.6)

This replaces the fixed organism trait/development chromosomes in the app's default world. The previous ecosystem and supplied-precursor labs remain loadable and available as separate comparisons. Their bone/muscle programs are not inserted into autonomous cells.

## Heredity and development

Each living cell stores a variable sequence of genes, its genome identifier, parent cell, generation, position/velocity, energy, two chemical concentrations and ten expressed action values. A gene contains an action target, eight signed promoter coefficients, bias and strength. Inputs are local food, local mineral availability, energy, actual bond degree, the two local chemical signals, age and movement. The same sequence can therefore express differently in different cells without a mutation. This is a designed regulatory language rather than literal DNA/protein synthesis.

The founder sequence has four genes: light uptake, mineral uptake, division investment and weak adhesion. All 64 initial cells have the same sequence, five supplied energy units and no bonds. No stiffness, contraction, secretion, transport or digestion gene is supplied. This is a supplied replicator, not molecular abiogenesis.

A cell with enough energy and expressed division investment can divide. Replication consumes one energy unit and divides the remaining store equally. The daughter copies the genome and chemical/expression state, starts a new age and receives a parent link. Each copied gene has a 2.5% chance of a promoter/bias/strength edit. Per replication there is a 4% duplication chance, 2.5% deletion chance and 8% new-gene insertion chance. New genes choose among physical/chemical actions, with random promoters. None of these probabilities rewards a needed function or reads a fitness score. Each changed sequence receives a new identifier. Deletion may remove a vital pathway and kill a descendant.

Daughter adhesion probability comes from the dividing cell's actual expression. If retained, a physical bond joins the two cells. There is no budding blueprint, body-size target, body-level genotype or tissue assignment. A body in the viewer is a connected component of surviving bonds. Injury/starvation or excessive separation can break bonds; a cell death removes its incident bonds and can fragment a body. Each constituent cell retains its own inherited genome. Genetic conflicts within a body are possible.

## Physical and chemical actions

The bounded action vocabulary is light/mineral uptake, adhesion, material stiffness, contraction, two chemical secretions, division investment, membrane transport and contact digestion. These are our model's building blocks, not an unlimited invention mechanism. Changing and combining their regulatory programs creates variation in cell behavior; it does not invent arbitrary biochemistry.

Actual cell positions consume a finite renewing nutrient field. Uptake converts field units to stored energy. Basal maintenance, gene count and expression have costs. Secreted chemical concentrations mix locally; they are abstract normalized regulatory signals, not conserved molecular inventories. Expressed membrane transport transfers existing energy along a bond conservatively and can permit paid, contact-mediated copying of a neighboring gene. Ordinary fission does not require a partner. Horizontal transfer is not meiosis or sexual reproduction.

Bonds apply equal and opposite spring forces. Stiffness and contraction depend on the cells' expression. Contraction shortens the current target and pays from both endpoint stores; insufficient stores reduce activation. Ground reaction, friction, damping, collisions and gravity act on cell particles. There is no nutrient-gradient steering, prescribed gait, angular joint program or forward force for a colony. The terrain is three-dimensional; cells generally settle near the habitat surface, so this is not a volumetric aquatic model.

Contact digestion extracts stored energy from an adjacent different founder's cell, with a stiffness-dependent defense. Part enters the receiver and the remainder returns to the field. Its counter records extraction events, not whole cells eaten. It does not manufacture a predator or weapons upgrade. Founder-related cells are protected by a designed kin rule.

## Selection, storage and limits

Actual survival and cell divisions determine which copied programs remain. No distance objective, candidate optimizer, automatic rescue or timed upgrade runs. The app fixes conditions and the replication-error protocol within an autonomous run; habitat presets start new runs. Environment controls are not trait controls. New worlds now use this engine by default.

Bounds are 1,024 living cells, 48 genes per cell, six retained attachments per dividing parent and 256 recent division records. Cells have an engineering age cutoff of 160 cycles. These bounds constrain possible outcomes; the simulation is not mathematically open-ended. The nutrient renewal coefficient is 0.16 scaled by the chosen habitat. Light, minerals, solvent and nutrient availability affect uptake; temperature/radiation-specific adaptation is not implemented in this engine. No climate parameter directs mutations. Imported legacy worlds retain their original rules.

Authoritative cell genomes, expression, signals, stores, bonds, counters and RNG are serialized in an additive version-four `autonomous` state. Generated organism/trait records are transient renderer adapters and never enter biological updates. Invalid genes, conflicting sequences under one genome ID, indices and numeric stores are rejected. The last 256 division records are not a complete genome archive.

Continuous stiffness or contractile expression is not sufficient evidence for bones or muscles. Tissues require persistent coordinated specialization; organs and organ systems require spatial organization and demonstrated collective function. The initial three 300-cycle tests produced at most three-cell connected components. They established variable inherited genomes and context-dependent expression, not organs, locomotion or an evolved multicellular fitness advantage. Replicated selection evidence for larger coordinated bodies is the next scientific milestone; it must be measured rather than forced with founder traits.
