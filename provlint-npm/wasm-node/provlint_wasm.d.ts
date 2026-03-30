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
