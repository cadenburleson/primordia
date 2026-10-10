/* tslint:disable */
/* eslint-disable */

export class World {
    free(): void;
    [Symbol.dispose](): void;
    cell_data(): Float32Array;
    configure(json: string): void;
    field(): Float32Array;
    ids(): Uint32Array;
    joint_data(): Float32Array;
    link_data(): Float32Array;
    load(json: string): void;
    metadata(selected: number): string;
    constructor(json: string);
    pulse(): void;
    render_data(): Float32Array;
    replay_best(enabled: boolean): void;
    save(): string;
    seed(count: number): void;
    seed_competitors(count: number): void;
    set_directional_sensing(enabled: boolean): void;
    set_motors_enabled(enabled: boolean): void;
    start_autonomous(): void;
    start_joint_lab(): void;
    start_lab(): void;
    start_population(count: number): void;
    start_tissue_study(): void;
    step(count: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_world_free: (a: number, b: number) => void;
    readonly world_cell_data: (a: number) => [number, number];
    readonly world_configure: (a: number, b: number, c: number) => [number, number];
    readonly world_field: (a: number) => [number, number];
    readonly world_ids: (a: number) => [number, number];
    readonly world_joint_data: (a: number) => [number, number];
    readonly world_link_data: (a: number) => [number, number];
    readonly world_load: (a: number, b: number, c: number) => [number, number];
    readonly world_metadata: (a: number, b: number) => [number, number];
    readonly world_new: (a: number, b: number) => [number, number, number];
    readonly world_pulse: (a: number) => void;
    readonly world_render_data: (a: number) => [number, number];
    readonly world_replay_best: (a: number, b: number) => void;
    readonly world_save: (a: number) => [number, number];
    readonly world_seed: (a: number, b: number) => void;
    readonly world_seed_competitors: (a: number, b: number) => void;
    readonly world_set_directional_sensing: (a: number, b: number) => void;
    readonly world_set_motors_enabled: (a: number, b: number) => void;
    readonly world_start_autonomous: (a: number) => void;
    readonly world_start_joint_lab: (a: number) => void;
    readonly world_start_lab: (a: number) => void;
    readonly world_start_population: (a: number, b: number) => void;
    readonly world_start_tissue_study: (a: number) => void;
    readonly world_step: (a: number, b: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
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
