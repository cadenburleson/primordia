export const GENES = ['photo', 'chemo', 'mobility', 'adhesion', 'armor', 'thermal', 'fertility', 'sex', 'shape', 'branching', 'specialization', 'pigment', 'muscle', 'skeleton', 'sensing', 'predation'] as const;
export type Gene = typeof GENES[number];
export const GI = Object.fromEntries(GENES.map((name, i) => [name, i])) as Record<Gene, number>;
export const WORLD_SIZE = 64;
export const GRID = 40;
export const MAX_POPULATION = 12000;
export const STRIDE = 12;
export const CELL_STRIDE = 9;
export const LINK_STRIDE = 6;
export const JOINT_STRIDE = 9;
export const MAX_BODY_CELLS = 64;
export const TISSUES = ['Metabolic','Support','Muscle','Sensory','Digestive','Reproductive'] as const;
export type WorldKind = 'tidal' | 'volcanic' | 'alien' | 'lab';
export interface WorldConfig {
  seed: number; kind: WorldKind; name: string; sunlight: number; nutrients: number;
  minerals: number; temperature: number; radiation: number; solvent: number;
  mutation: number; capacity: number; worldSize?:number; worldShape?:"planar"|"sphere"; founderSpread?:"local"|"global"; initialCells?: number; physicalAssay?: 'full'|'passive'|'scarcity'; resourceLayout?:'diffuse'|'patchy'; origin: 'seeded' | 'chemistry';
}
export const PRESETS: Record<Exclude<WorldKind,'lab'>, WorldConfig> = {
  tidal: { seed: 8042, kind: 'tidal', name: 'The tidal garden', sunlight: .75, nutrients: .7, minerals: .45, temperature: .48, radiation: .18, solvent: .85, mutation: .09, capacity: 8192, initialCells: 256, worldSize:1024, worldShape:"sphere", founderSpread:"local", physicalAssay: 'full', resourceLayout:'patchy', origin: 'seeded' },
  volcanic: { seed: 1197, kind: 'volcanic', name: 'The ember sea', sunlight: .2, nutrients: .55, minerals: .94, temperature: .81, radiation: .32, solvent: .65, mutation: .11, capacity: 8192, initialCells: 256, worldSize:1024, worldShape:"sphere", founderSpread:"local", physicalAssay: 'full', resourceLayout:'patchy', origin: 'seeded' },
  alien: { seed: 6219, kind: 'alien', name: 'The violet frontier', sunlight: .42, nutrients: .45, minerals: .74, temperature: .25, radiation: .64, solvent: .63, mutation: .14, capacity: 8192, initialCells: 256, worldSize:1024, worldShape:"sphere", founderSpread:"local", physicalAssay: 'full', resourceLayout:'patchy', origin: 'seeded' },
};
export interface Organism {
  id: number; parent: number; mate: number; lineage: number; generation: number;
  x: number; z: number; vx: number; vz: number; heading: number; energy: number; age: number;
  cells: number; genes: Float32Array; cooldown: number; alive: boolean;
}
export interface Lineage {
  id: number; parent: number; name: string; born: number; generation: number;
  color: string; ancestor: number[]; peak: number; representative?: number;
}
export interface LifeEvent { time: number; text: string; type: 'origin' | 'birth' | 'colony' | 'sex' | 'extinction' | 'world' | 'motion' | 'feeding' | 'tissue' | 'milestone' }
export interface HistoryPoint { time: number; population: number; cells: number; colonies: number; diversity: number }
export interface Stats {
  time: number; population: number; cells: number; colonies: number; diversity: number;
  snapshotMs?:number; actualSpeed?: number; requestedSpeed?: number; generation: number; births: number; deaths: number; mating: number; feedings: number;
  meanGenes: number[]; nutrient: number; capped: boolean; computeMs: number;
  articulated: number; contractile: number; jointed:number; cellBudget:number; cellBudgetReached:boolean; regulated:number; injured:number;
}
export interface V3 { x:number; y:number; z:number }
export interface PhysicalCell { p:V3; v:V3; rest:V3; radius:number; role:number; contact:boolean; health:number; reserve:number; signal:number[] }
export interface PhysicalLink { a:number; b:number; rest:number; target:number; kind:number; activation:number; strain:number; integrity:number; purpose:number }
export interface Anatomy {traits:number[];network:number[][]}
export interface PhysicalJoint {a:number;b:number;c:number;segments:number[];enabled:boolean;min:number;max:number;angle:number;flexion:number;extension:number;state:number[];work:number;limit_error:number}
export interface PhysicalBody {anatomy:Anatomy|null;joints:PhysicalJoint[]; nodes:PhysicalCell[]; links:PhysicalLink[]; clock:number; work:number; path:number; contacts:number; steps:number; max_strain:number; regulated:boolean; differentiations:number; injuries:number; repair_work:number }
export interface TrialResult { generation:number; candidate:number; distance:number; work:number; score:number; contacts:number; steps:number; supported:number; viable:boolean; joints:number; joint_error:number; alternations:number }
export interface LabView {jointed:boolean;motorsEnabled:boolean;alternations:number;jointError:number; replaying:boolean; generation:number; candidate:number; candidates:number; elapsed:number; duration:number; phase:string; distance:number; work:number; baseline:TrialResult|null; champion:TrialResult|null; progress:TrialResult[]; results:TrialResult[]; trace:[number,number,number][] }
export interface CompetingFamily {id:number;name:string;introduced:number;living:number;cells:number;births:number;exchanges:number;meals:number;deaths:number;generation:number;jointed:number;meanPredation:number;meanSensing:number;meanMuscle:number;controllerChange:number;representative:number|null}
export interface BirthRecord {time:number;child:number;parent:number;mate:number;family:number;generation:number}
export interface CompetitionView {directionalSensing:boolean;foodEnergy:number;peakGeneration:number;renewal:number;families:CompetingFamily[];ancestry:BirthRecord[];peakJointed:number;largestBody:number;foodPoorFraction:number}
export interface BodyStores {water:number;solutes:number[];gas:number[];matrix:number;fibers:number;polymers:{sequence:number[];amount:number}[]}
export interface MaterialInventory {biomass:number;fuel:number;mineral:number;lipid:number;proteins:number[];body?:BodyStores}
export interface PhysicsView {domain:{width:number;grid:number;rows:number;sites:number;spacing:number;radius:number;shape:"planar"|"sphere";area:number;boundary:string};worldFlow?:number[][]|null;assay: 'full'|'passive'; joints:number; angularWork:number;organic:number;mineral:number;chemicalEnergy:number;photonEnergy:number;heat:number;mechanicalInput:number;massError:number;mineralError:number;energyError:number;fluidSpeed:number;boundEvents:number;dissociations:number;proteinMass:number;boundMineral:number;flow:[number,number,number][]}
export interface CellPhysiology {excitation:number;recovery:number;nutrient_memory:number;food_change:number;injury:number;strain:number;exposure:number;ground:boolean;light:number;neural_input:number;pressure?:number;pressure_gradient?:number;germline?:number;light_contrast?:number;
  /** 0 for a stem cell, 1 to 3 once committed to a fate inside a body. */
  fate?:number;
  /** Light harvest kept up by a captured cell living inside this one. */
  symbiont?:number}
export interface SystemView {particles:{p:V3;radius:number;mineral:number}[];ledger:{waterFlow:number;soluteFlow:number;membraneExchange:number;absorbed:number;filtered:number;gasExchange:number;respired:number;pumpWork:number;matrixBuilt:number;fiberBuilt:number;mineralRemodeled:number;polymersBuilt:number;fidelityWork:number;matrixMeals:number};fluidCells:number;fluidConnections:number;internalWater:number;internalSolutes:number;matrix:number;fibers:number;polymerSpecies:number;polymerMass:number;gasError:number;germlineCells:number;pressureMax:number}
export interface PhysiologyView {ledger:{lipid_stored:number;lipid_used:number;fuel_transported:number;energy_transported:number;signal_work:number;repairs:number;digested:number;excreted:number;decomposed:number;spikes:number;contact_changes:number};regions:{function:string;cells:number;regions:number;largest:number}[];mixedBodies:number;lipid:number;excited:number;injured:number;grounded:number}
export interface PlanetView {seasonStrength?:number;seasonPeriod?:number;cloudCover?:number;stormRadius?:number;waterDepth?:number[];sunDirection?:number[];day:number;season:number;sunlight:number;temperature:number;weather:string;storm:number;stormCenter:number[];water:number;snow:number;atmosphere:number;waterError:number;precipitation:number;evaporation:number;snowmelt:number;runoff:number;eruptions:number;mineralInput:number;relief:number[];snowCover:number[];heat:number[];terrainVersion:number}
export interface CellLifecycle {root:number;depth:number;phase:"somatic"|"gamete"|"zygote";mate:number;homolog:{id:number;genes:ProgramCell["genes"]}|null}
export interface DevelopmentView {gametesProduced:number;zygotes:number;meioses:number;livingGametes:number;diploidCells:number;synapses:number;transmissions:number;adaptations:number;sensoryWork:number;neuralWork:number;fusionWork:number;recent:{time:number;child:number;parent:number;mate:number;generation:number}[]}
export interface ProgramCell {lifecycle?:CellLifecycle;state?:CellPhysiology;material?:MaterialInventory|null;id:number;parent:number;genome:number;generation:number;genes:{action:number;promoter:number[];bias:number;strength:number;fates?:number}[];energy:number;chemicals:number[];expression:number[]}
export type BreedingGoal='bodies'|'swimming'|'movement'|'support'|'senses'|'hunting'|'defence';
/** Creator commands. They change selection and variation, never genes or materials. */
export type SteerAction =
  | {kind:'mutation';scale:number}
  | {kind:'mixing';scale:number}
  | {kind:'sharing';scale:number}
  | {kind:'seasons';depth:number;length:number}
  | {kind:'light';scale:number}
  | {kind:'vents';scale:number}
  | {kind:'goal';goal:BreedingGoal|'';strength:number;period:number}
  | {kind:'favour';body:number;pressure:number}
  | {kind:'unfavour'}
  | {kind:'storm';scale:number;duration:number}
  | {kind:'winter';depth:number;duration:number}
  | {kind:'meteor';x:number;z:number;radius:number}
  | {kind:'dying';fraction:number}
  | {kind:'nourish';amount:number};
/** stage: 0 absent · 1 gene in the pool · 2 working somewhere · 3 established. */
export interface LadderRung {key:string;stage:0|1|2|3;progress:number;genes:number;built:number;count:number;first:(number|null)[]}
export interface SteeringView {mutation:number;mutationFactor:number;mixing:number;light:number;vents:number;
  /** How readily touching cells pass a gene across; 0 is off. */
  sharing?:number;
  /** Share of light lost at midwinter, the length of a year in cycles, and the share of light arriving now. */
  seasonDepth?:number;seasonLength?:number;season?:number;
  /** Living cells by how their body feeds: light, dissolved fuel, remains, vents, herbivore, omnivore, carnivore. */
  diet:number[];
  /** Living cells by kind of life: simple (no wall, mouth or motor), plant-like, fungus-like, animal-like. */
  kinds?:number[];
  /** Times a swallowed cell was first kept alive inside its host. */
  captures:number;losses:[number,number,number];storm:number;goal:BreedingGoal|'';strength:number;period:number;nextRound:number;rounds:number;culled:number;note:''|'selecting'|'no-variation'|'recovering'|'too-few';scored:number;bodies:number;favoured:number;favourPressure:number;favourCulled:number;winter:number;ladder:LadderRung[];milestones:{key:string;stage:number;time:number;generation:number}[]}
export interface AutonomousView {steering?:SteeringView|null;systems?:SystemView|null;development?:DevelopmentView|null;resources?:{detritus:number[];mineral:number[];waste?:number[];light?:number[];area?:number[]};feeding?:{time:number;attacker:number;prey:number;attacker_genome:number;prey_genome:number;material:string;amount:number}[];observationControl?:string;physiology?:PhysiologyView;planet?:PlanetView|null;detritus?:number;waste?:number;physics?:PhysicsView|null;actions:string[];expressed:number[];divisions:number;attachedDivisions:number;pointEdits:number;duplications:number;deletions:number;innovations:number;exchanges:number;foodEnergy:number;motorWork:number;peakBody:number;peakGeneration:number;genomeMin:number;genomeMax:number;genomes:number;recent:{time:number;parent:number;child:number;genome:number;generation:number;attached:boolean;edits:number}[]}
export interface Snapshot {
  type: 'snapshot'; nursery?:{parentName:string}|null; selection?:number; paused?:boolean; assessment?:Assessment; canUndoWorld?:boolean; config: WorldConfig; data: Float32Array; ids: Uint32Array;
  cellData:Float32Array; linkData:Float32Array; jointData:Float32Array; lab:LabView|null; study:boolean; competition:CompetitionView|null; autonomous?:AutonomousView|null;
  field: Float32Array; stats: Stats; lineages: (Lineage & { count: number })[];
  events: LifeEvent[]; history: HistoryPoint[]; selected: OrganismView | null;
}
export interface OrganismView {observation?:BodyObservation;programCells?:ProgramCell[]; id: number; lineage: number; name: string; generation: number; age: number; cells: number; energy: number; genes: number[]; development:number[]; regulation:{rules:number[][];growth:number[]}|null; anatomy:Anatomy|null; body:PhysicalBody; meals:number; feedingCooldown:number; family:number; offspring:number; parent: number; mate: number; x: number; z: number }
export interface SaveData {
  version: 1|2|3|4; config: WorldConfig; rng: number; time: number; next_id: number; next_lineage: number;
  births: number; deaths: number; mating: number; organisms: (Omit<Organism, 'genes'> & { genes: number[] })[];
  lineages: Record<string,Lineage>; field: number[]; events: LifeEvent[]; history: HistoryPoint[];
  milestones: string[]; history_clock: number;
}
export type WorldEdit =
  | {kind:'climate';sunlight:number;temperature:number;clouds:number}
  | {kind:'brush';tool:'raise'|'lower'|'flatten'|'fuel'|'mineral'|'detritus'|'water';x:number;z:number;radius:number;amount:number;aspect?:number;angle?:number;level?:number;points?:[number,number][]}
  | {kind:'seasons';strength:number;period:number}
  | {kind:'rain';x:number;z:number}
  | {kind:'genes';body:number;cell:number;genes:ProgramCell['genes']};
export type Command =
  | {type:'brush-stroke';id:number;start:boolean;end:boolean;settings:import('../brush').BrushSettings;points:[number,number][]}
  | {type:'nursery';action:'enter'|'leave'|'recover'}
  | {type:'steer';action:SteerAction}
  | {type:'display-priority';mode:'live'|'evolution'}
  | {type:'editor';action:'apply';edit:WorldEdit}
  | {type:'editor';action:'undo'}
  | {type:'editor';action:'founder'}
  | {type:'editor';action:'copy';id:number;name:string}
  | {type:'specimen';action:'capture';name:string;id:number}
  | {type:'specimen';action:'inspect';json:string}
  | {type:'specimen';action:'place';json:string;x:number;z:number}
  | { type: 'init'; config: WorldConfig; steering?: SteerAction[] }
  | { type: 'control'; paused?: boolean; speed?: number }
  | { type: 'visibility'; visible: boolean }
  | { type: 'config'; patch: Partial<WorldConfig> }
  | { type: 'seed'; count: number }
  | { type: 'autonomous' }
  | { type: 'cell-budget'; capacity: number }
  | { type: 'physical-study'; config: WorldConfig }
  | { type: 'return-physical' }
  | { type: 'undo-world' }
  | { type:'assessment'; action:'capture'|'run'|'restore'; control?:ComparisonControl; duration?:number }
  | { type: 'competition'; action:'start'|'introduce'|'sense-on'|'sense-off'; count:number }
  | { type: 'pulse' }
  | { type: 'select'; id: number | null }
  | { type: 'save' }
  | { type: 'load'; data: unknown; restore?:boolean }
  | { type: 'lab'; action:'start'|'joints'|'study'|'leave'|'replay'|'evolve'|'motors-on'|'motors-off' };

export type ComparisonControl='baseline'|'motors'|'transport'|'signals';
export interface SpecimenSummary {name:string;cells:number;generation:number;genomes:number;joints:number;bonds:number;synapses:number;sourceBody:number;sourceSeed:number;cycle:number;genes:number[];functions:number[];shape:number[][]}
export interface SavedSpecimen {id:string;savedAt:string;json:string;summary:SpecimenSummary}
export interface SpecimenMessage {type:'specimen';action:'capture'|'inspect';json:string;summary:SpecimenSummary}
export interface Flux {fuel:number;detritus:number;photosynthesis:number;prey:number;lost_to_predators:number;defense_blocked:number;minerals:number;waste:number;organic_waste:number;signal_a:number;signal_b:number;proteins:number;absorbed:number;filtered:number;respiration:number;water_transport:number;solute_transport:number;matrix:number;fibers:number;chemosynthesis?:number;wounded?:number;prey_animal?:number;
  /** Fuel a mouth cell passed to unarmed cells of its own body, and what those cells received. */
  shared?:number;fed?:number}
export interface BodyObservation {flux:Flux;stocks:number[];functions:number[];centroid:V3;membership:string;members:number;site:number;ambientFlow:V3;localResources:number[]}
export interface ComparisonResult {control:ComparisonControl;cycles:number;targetCycles:number;body:number;membershipChanged:boolean;bodyLost:boolean;distance:number|null;ambientTravel:number;work:number|null;food:number|null;prey:number|null;soluteTransport:number|null;births:number;deaths:number;cells:number;regions:number}
export interface Assessment {checkpoint:boolean;running:boolean;control:ComparisonControl|null;elapsed:number;duration:number;results:ComparisonResult[]}
