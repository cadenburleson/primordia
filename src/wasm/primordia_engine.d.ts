/* tslint:disable */
/* eslint-disable */

export class World {
    free(): void;
    [Symbol.dispose](): void;
    capture_specimen(selected: number, name: string): string;
    cell_data(): Float32Array;
    /**
     * The leading living body on each measure, as a JSON list.
     */
    champions(): string;
    configure(json: string): void;
    /**
     * Reuse the body projection already prepared for this display snapshot.
     */
    display_metadata(selected: number, display: World): string;
    display_world(): World;
    field(): Float32Array;
    founder_specimen(): string;
    ids(): Uint32Array;
    inspect_specimen(json: string): string;
    intervene(json: string): string;
    joint_data(): Float32Array;
    link_data(): Float32Array;
    load(json: string): void;
    metadata(selected: number): string;
    constructor(json: string);
    observation_control(control: string): void;
    /**
     * Scatter copies of an organism around painted points; returns how many fitted.
     */
    paint_life(json: string, points: Float32Array, radius: number, per_point: number): number;
    place_specimen(json: string, x: number, z: number): number;
    pulse(): void;
    render_data(): Float32Array;
    replay_best(enabled: boolean): void;
    save(): string;
    seed(count: number): void;
    seed_competitors(count: number): void;
    /**
     * Engineering budget only: does not change chemistry, genes or timestep.
     */
    set_cell_budget(capacity: number): void;
    set_directional_sensing(enabled: boolean): void;
    set_motors_enabled(enabled: boolean): void;
    simulation_time(): number;
    /**
     * One founder of a starter species, as an organism snapshot.
     */
    species_specimen(species: string): string;
    start_autonomous(): void;
    start_joint_lab(): void;
    start_lab(): void;
    start_physical(): void;
    start_population(count: number): void;
    start_tissue_study(): void;
    /**
     * Creator steering: mutation rate, breeding goals, favoured lineages and
     * disasters. Changes selection and variation, never genes or materials.
     */
    steer(json: string): string;
    step(count: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_world_free: (a: number, b: number) => void;
    readonly world_capture_specimen: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly world_cell_data: (a: number) => [number, number];
    readonly world_champions: (a: number) => [number, number];
    readonly world_configure: (a: number, b: number, c: number) => [number, number];
    readonly world_display_metadata: (a: number, b: number, c: number) => [number, number];
    readonly world_display_world: (a: number) => number;
    readonly world_field: (a: number) => [number, number];
    readonly world_founder_specimen: (a: number) => [number, number, number, number];
    readonly world_ids: (a: number) => [number, number];
    readonly world_inspect_specimen: (a: number, b: number, c: number) => [number, number, number, number];
    readonly world_intervene: (a: number, b: number, c: number) => [number, number, number, number];
    readonly world_joint_data: (a: number) => [number, number];
    readonly world_link_data: (a: number) => [number, number];
    readonly world_load: (a: number, b: number, c: number) => [number, number];
    readonly world_metadata: (a: number, b: number) => [number, number];
    readonly world_new: (a: number, b: number) => [number, number, number];
    readonly world_observation_control: (a: number, b: number, c: number) => [number, number];
    readonly world_paint_life: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly world_place_specimen: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly world_pulse: (a: number) => void;
    readonly world_render_data: (a: number) => [number, number];
    readonly world_replay_best: (a: number, b: number) => void;
    readonly world_save: (a: number) => [number, number];
    readonly world_seed: (a: number, b: number) => void;
    readonly world_seed_competitors: (a: number, b: number) => void;
    readonly world_set_cell_budget: (a: number, b: number) => [number, number];
    readonly world_set_directional_sensing: (a: number, b: number) => void;
    readonly world_set_motors_enabled: (a: number, b: number) => void;
    readonly world_simulation_time: (a: number) => number;
    readonly world_species_specimen: (a: number, b: number, c: number) => [number, number, number, number];
    readonly world_start_autonomous: (a: number) => void;
    readonly world_start_joint_lab: (a: number) => void;
    readonly world_start_lab: (a: number) => void;
    readonly world_start_physical: (a: number) => void;
    readonly world_start_population: (a: number, b: number) => void;
    readonly world_start_tissue_study: (a: number) => void;
    readonly world_steer: (a: number, b: number, c: number) => [number, number, number, number];
    readonly world_step: (a: number, b: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
