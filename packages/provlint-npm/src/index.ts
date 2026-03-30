export interface Diagnostic {
  code: string;
  message: string;
  severity: "error" | "warning" | "info";
  category: "schema" | "security" | "best-practice";
  span: Span;
  fix?: Fix;
}

export interface Span {
  startLine: number;
  startCol: number;
  endLine: number;
  endCol: number;
}

export interface Fix {
  description: string;
  replacement: string;
}

export interface RuleInfo {
  code: string;
  description: string;
  formats: Format[];
}

export type Format = "kickstart" | "autoyast" | "autoinstall";

export interface LintConfig {
  disabledRules?: string[];
}

export interface ProvLint {
  lint(content: string, format?: Format): Diagnostic[];
  lintWithConfig(content: string, format: Format, config: LintConfig): Diagnostic[];
  detectFormat(content: string): Format | null;
  getSupportedRules(): RuleInfo[];
}

let wasmInstance: any = null;

export async function initProvLint(): Promise<ProvLint> {
  if (wasmInstance) {
    return wasmInstance;
  }

  const wasm = await import("../wasm/provlint_wasm.js");
  await wasm.default();

  const inner = new wasm.WasmProvLint();

  wasmInstance = {
    lint(content: string, format?: Format): Diagnostic[] {
      return inner.lint(content, format ?? null) as Diagnostic[];
    },
    lintWithConfig(
      content: string,
      format: Format,
      config: LintConfig
    ): Diagnostic[] {
      return inner.lintWithConfig(content, format, config) as Diagnostic[];
    },
    detectFormat(content: string): Format | null {
      return (inner.detectFormat(content) as Format) ?? null;
    },
    getSupportedRules(): RuleInfo[] {
      return inner.getSupportedRules() as RuleInfo[];
    },
  };

  return wasmInstance;
}
