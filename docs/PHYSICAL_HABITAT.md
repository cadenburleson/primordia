# Historical physical baseline

For the current organism preview on port 5194, see [v0.9 organism/ecosystem model](ORGANISMS_AND_ECOSYSTEM.md). The baseline description below predates the added pathways and planetary cycles.

# Physical habitat preview

This isolated copy extends Primordia 0.6 without modifying the main checkout or its running world. New worlds and the restart button use `World.start_physical()`. The original autonomous engine and supplied-trait comparison labs remain available through their APIs and older saves. Preview ports are 5192 (development) and 5193 (production).

## Physical equations and representation

The 40×40 horizontal solvent lattice uses a D2Q9 lattice Boltzmann BGK collision/streaming scheme. Equilibrium populations are w*rho*[1 + 3(c·u) + 4.5(c·u)^2 - 1.5(u·u)]. Relaxation time is 0.9; kinematic viscosity is (tau-0.5)/3 in lattice units. Grid spacing is 1.6 model units and one lattice update is 0.25 model cycles. The initial low-Mach vortex has no sustaining force. Halfway bounce-back at the outer square walls represents no-slip boundaries. The terrain is not a resolved fluid boundary.

Five dissolved species use conservative finite-volume donor-cell advection and Fickian diffusion across site faces. Four substeps bound the outgoing coefficients in the validated low-Mach domain. Outer solute walls are impermeable. Fuel, waste and two signaling substances share an abstract organic material inventory; dissolved mineral has a separate inventory. Signal decay converts signal into waste, rather than deleting it.

Cell positions remain three-dimensional. Pair springs and contact forces are equal and opposite. Cell mass and radius follow organic and mineral stores, using distinct material densities. Weight minus buoyancy acts vertically. Stokes drag uses 6*pi*viscosity*radius times fluid-relative velocity with implicit time stepping. Horizontal drag impulses are returned to solvent populations with a zero-mass, matching-momentum update. Vertical fluid velocity is not solved. Terrain contact and outer cell walls exchange momentum with an external substrate.

The viscosity and drag relations are physically grounded. Constants are nondimensional, chosen for stable experiments; they have not been calibrated to a particular molecule, organism, length or time scale. The low-Reynolds-number Stokes model is not a model for turbulent swimming or macroscopic running. Fluid flow is weakly compressible and two-dimensional; this is not a volumetric ocean simulation.

## Defined reduced chemistry

Every cell has biomass, reduced fuel, bound mineral, and ten protein/polymer inventories. Genes specify production rates and environmental promoter responses for a bounded set of catalysts/materials. Gene output is not directly stiffness or force. Protein activity is a saturating function of actual inventory relative to biomass. Production consumes cell fuel and ATP-like stored energy. Protein turnover returns material to fuel. Founder cells supply only the four basic pathways, and their initial catalysts and energy are explicitly initialized.

Defined reactions:

- Uptake transfers extracellular reduced fuel into a cell through synthesized transporters.
- Photosynthesis converts oxidized waste to reduced fuel with incoming photon energy.
- Respiration converts reduced fuel to extracellular waste and ATP-like energy.
- Biomass/protein/signal synthesis transfers reduced carbon between forms and consumes ATP-like energy.
- Signals diffuse, advect, and decay into waste with their remaining chemical energy going to the heat ledger.
- Mineral-binding catalysts associate dissolved minerals with the cell; a first-order dissociation reaction returns them. Bound material changes mass and bond stiffness. Minerals are not themselves free metabolic energy.
- Contact digestive catalysts cleave actual prey biomass into the consumer's fuel; mineral deposition reduces the modeled cleavage rate. No founder/kin immunity rule is supplied.
- Contact transport exchanges actual fuel across a real bond. Gene exchange copies a neighboring gene with an energy cost; it is not sexual reproduction.

These are designed coarse-grained reactions with chosen rate constants, not carbon/oxygen stoichiometry, force-field parameters, atomic electron chemistry or a molecular bonding solver. Adhesive polymer association supplies stochastic contact attachment and strain-sensitive dissociation. It is a molecular-inspired effective binding model, not a ReaxFF or quantum calculation. Water viscosity, material density, reaction heat and adhesion affinity do not emerge from atomistic interactions.

Contraction requires synthesized contractile material and environmental signal A, with signal B inhibiting activation. ATP-like energy pays the positive spring-energy change and an activation cost; starvation reduces activation. Mineralized bonds are effective elastic connections, not anatomically defined bones. Constant contraction does not establish a locomotor gait.

## Conservation and development

Organic material and mineral inventories are finite and closed. There is no uniform nutrient renewal or arbitrary corpse-resource bonus. Death returns the actual material inventory. Photosynthesis can regenerate fuel from waste but consumes tracked light energy. Habitat presets choose new starting pools, solvent availability and light input; they do not mutate traits during a run. Temperature/radiation-specific reaction kinetics, thermal damage and Brownian motion are not implemented in this preview.

Chemical-energy accounting uses three energy units per unit of reduced carbon (fuel, biomass, proteins and signals), zero for modeled oxidized waste, plus ATP-like stores. Consumed ATP goes to a heat or mechanical-input ledger; photons are explicit external input. The reported balance is chemical energy plus cumulative sinks minus initial energy and photon input. Mechanical input is a sink from the chemical budget, not the currently retained kinetic/spring energy. Numerical mechanics, drag dissipation, substrate work and stiffness changes do not have a complete thermodynamic ledger. Therefore a small chemical-energy residual is not proof of full mechanical energy conservation.

Growth requires biomass precursor and energy. Division requires sufficient biomass, a synthesized replication pathway and available energy. It splits every existing inventory and energy store, copies and mutates the genome, and separates daughter positions about their center. There is no target organism body plan, tissue stage, timed unlock or automatic rescue. Contact association can join unrelated cells into actual graph components. Engineering bounds remain 1,024 cells, 48 genes per cell, six bonds per cell, 256 recent division records. This preview removes the old fixed 160-cycle death cutoff in physical worlds; starvation and biomass loss cause death.

Cell homeostasis, catalysts, replication machinery and the regulatory vocabulary are still designed. This is a supplied-replicator artificial-life experiment, not abiogenesis or unlimited biochemical invention. Tissues, organs, organ systems, mating, bones, muscles and walking are not guaranteed outcomes. Demonstrating persistent specialization, functional collective movement and a heritable multicellular fitness advantage requires further experiments.

## Persistence and observation

The optional `autonomous.physical` state saves solute amounts, every fluid population and chemical budget. Each cell saves `material`. Older autonomous saves without these fields retain the prior rules. Load rejects malformed stores, invalid fluids and inconsistent budgets. Save/restore continuation is tested exactly within one engine build.

The inspector shows per-cell biomass, fuel, bound mineral and synthesized stores alongside its genome. The habitat panel reports mass/mineral/chemical-energy residuals. The layers button shows nutrient availability and measured horizontal currents; arrow lengths magnify velocity 18 times for readability. Water surface animation is decorative, not a rendering of the fluid velocity solver.

## References

- [Lattice Boltzmann methods, Scholarpedia](https://www.scholarpedia.org/article/Lattice_Boltzmann_Methods): D2Q9/BGK and bounce-back foundations.
- [COMSOL particle motion in fluids](https://doc.comsol.com/6.4/doc/com.comsol.help.particle/particle_ug_fluid_flow.08.36.html): Stokes drag and its low-Reynolds-number limits.
- [COMSOL Fickian diffusion](https://www.comsol.com/multiphysics/diffusion-equation): concentration transport by diffusion.
- [LAMMPS reactive force fields](https://docs.lammps.org/pair_reaxff.html): contrast with atomistic reactive bonding, which this preview does not implement.


## v0.8 shared mechanics and scale

Material bending, antagonistic paid angular actuation, shared physical assays, configurable starting populations/cell ceilings, measured high-speed scheduling and native batch execution are described in [SHARED_ENGINE_AND_SCALE.md](SHARED_ENGINE_AND_SCALE.md). This supersedes older fixed-population limits in this document. Legacy recurrent controllers and sexual mating remain separate comparisons.
