/* tslint:disable */
/* eslint-disable */

export class WasmProvLint {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Detect the format of content.
     */
    detectFormat(content: string): string | undefined;
    /**
     * Get all supported rules.
     */
    getSupportedRules(): any;
    /**
     * Lint content with optional format string.
     * Returns a JSON array of diagnostics.
     */
    lint(content: string, format?: string | null): any;
    /**
     * Lint with configuration (disabled rules).
     */
    lintWithConfig(content: string, format: string, config: any): any;
    constructor();
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmprovlint_free: (a: number, b: number) => void;
    readonly wasmprovlint_detectFormat: (a: number, b: number, c: number) => [number, number];
    readonly wasmprovlint_getSupportedRules: (a: number) => any;
    readonly wasmprovlint_lint: (a: number, b: number, c: number, d: number, e: number) => any;
    readonly wasmprovlint_lintWithConfig: (a: number, b: number, c: number, d: number, e: number, f: any) => any;
    readonly wasmprovlint_new: () => number;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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
