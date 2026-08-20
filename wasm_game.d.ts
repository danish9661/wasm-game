/* tslint:disable */
/* eslint-disable */

/**
 * Machine-readable stats for the JS HUD / test harness.
 */
export function get_stats(): string;

/**
 * Result of the last completed readback ("pending" until the map callback runs).
 */
export function readback_stats(): string;

/**
 * Recompute canvas size on window resize.
 */
export function resize(): void;

export function start(): void;

/**
 * Called by JS every frame. JS owns the loop; wasm just steps.
 */
export function step(dt_seconds: number): void;

/**
 * Queue a GPU→CPU readback of the next rendered frame.
 */
export function trigger_readback(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly get_stats: (a: number) => void;
    readonly readback_stats: (a: number) => void;
    readonly start: () => void;
    readonly step: (a: number) => void;
    readonly trigger_readback: () => void;
    readonly resize: () => void;
    readonly __wasm_bindgen_func_elem_1410: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_361: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_361_3: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_361_4: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_272: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_274: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export5: (a: number, b: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
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
