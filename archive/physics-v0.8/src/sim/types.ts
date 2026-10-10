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
  mutation: number; capacity: number; initialCells?: number; physicalAssay?: 'full'|'passive'|'scarcity'; origin: 'seeded' | 'chemistry';
}
export const PRESETS: Record<Exclude<WorldKind,'lab'>, WorldConfig> = {
  tidal: { seed: 8042, kind: 'tidal', name: 'The tidal garden', sunlight: .75, nutrients: .7, minerals: .45, temperature: .48, radiation: .18, solvent: .85, mutation: .09, capacity: 8192, initialCells: 256, physicalAssay: 'full', origin: 'seeded' },
  volcanic: { seed: 1197, kind: 'volcanic', name: 'The ember sea', sunlight: .2, nutrients: .55, minerals: .94, temperature: .81, radiation: .32, solvent: .65, mutation: .11, capacity: 8192, initialCells: 256, physicalAssay: 'full', origin: 'seeded' },
  alien: { seed: 6219, kind: 'alien', name: 'The violet frontier', sunlight: .42, nutrients: .45, minerals: .74, temperature: .25, radiation: .64, solvent: .63, mutation: .14, capacity: 8192, initialCells: 256, physicalAssay: 'full', origin: 'seeded' },
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
export interface LifeEvent { time: number; text: string; type: 'origin' | 'birth' | 'colony' | 'sex' | 'extinction' | 'world' | 'motion' | 'feeding' | 'tissue' }
export interface HistoryPoint { time: number; population: number; cells: number; colonies: number; diversity: number }
export interface Stats {
  time: number; population: number; cells: number; colonies: number; diversity: number;
  actualSpeed?: number; requestedSpeed?: number; generation: number; births: number; deaths: number; mating: number; feedings: number;
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
export interface MaterialInventory {biomass:number;fuel:number;mineral:number;proteins:number[]}
export interface PhysicsView {assay: 'full'|'passive'; joints:number; angularWork:number;organic:number;mineral:number;chemicalEnergy:number;photonEnergy:number;heat:number;mechanicalInput:number;massError:number;mineralError:number;energyError:number;fluidSpeed:number;boundEvents:number;dissociations:number;proteinMass:number;boundMineral:number;flow:[number,number,number][]}
export interface ProgramCell {material?:MaterialInventory|null;id:number;parent:number;genome:number;generation:number;genes:{action:number;promoter:number[];bias:number;strength:number}[];energy:number;chemicals:number[];expression:number[]}
export interface AutonomousView {physics?:PhysicsView|null;actions:string[];expressed:number[];divisions:number;attachedDivisions:number;pointEdits:number;duplications:number;deletions:number;innovations:number;exchanges:number;foodEnergy:number;motorWork:number;peakBody:number;peakGeneration:number;genomeMin:number;genomeMax:number;genomes:number;recent:{time:number;parent:number;child:number;genome:number;generation:number;attached:boolean;edits:number}[]}
export interface Snapshot {
  type: 'snapshot'; canUndoWorld?:boolean; config: WorldConfig; data: Float32Array; ids: Uint32Array;
  cellData:Float32Array; linkData:Float32Array; jointData:Float32Array; lab:LabView|null; study:boolean; competition:CompetitionView|null; autonomous?:AutonomousView|null;
  field: Float32Array; stats: Stats; lineages: (Lineage & { count: number })[];
  events: LifeEvent[]; history: HistoryPoint[]; selected: OrganismView | null;
}
export interface OrganismView {programCells?:ProgramCell[]; id: number; lineage: number; name: string; generation: number; age: number; cells: number; energy: number; genes: number[]; development:number[]; regulation:{rules:number[][];growth:number[]}|null; anatomy:Anatomy|null; body:PhysicalBody; meals:number; feedingCooldown:number; family:number; offspring:number; parent: number; mate: number; x: number; z: number }
export interface SaveData {
  version: 1|2|3|4; config: WorldConfig; rng: number; time: number; next_id: number; next_lineage: number;
  births: number; deaths: number; mating: number; organisms: (Omit<Organism, 'genes'> & { genes: number[] })[];
  lineages: Record<string,Lineage>; field: number[]; events: LifeEvent[]; history: HistoryPoint[];
  milestones: string[]; history_clock: number;
}
export type Command =
  | { type: 'init'; config: WorldConfig }
  | { type: 'control'; paused?: boolean; speed?: number }
  | { type: 'config'; patch: Partial<WorldConfig> }
  | { type: 'seed'; count: number }
  | { type: 'autonomous' }
  | { type: 'cell-budget'; capacity: number }
  | { type: 'physical-study'; config: WorldConfig }
  | { type: 'return-physical' }
  | { type: 'undo-world' }
  | { type: 'competition'; action:'start'|'introduce'|'sense-on'|'sense-off'; count:number }
  | { type: 'pulse' }
  | { type: 'select'; id: number | null }
  | { type: 'save' }
  | { type: 'load'; data: unknown; restore?:boolean }
  | { type: 'lab'; action:'start'|'joints'|'study'|'leave'|'replay'|'evolve'|'motors-on'|'motors-off' };
