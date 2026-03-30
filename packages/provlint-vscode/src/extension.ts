import * as vscode from 'vscode';
import * as path from 'path';

interface ProvLintSpan {
  startLine: number;
  startCol: number;
  endLine: number;
  endCol: number;
}

interface ProvLintFix {
  description: string;
  replacement: string;
}

interface ProvLintDiagnostic {
  code: string;
  message: string;
  severity: 'error' | 'warning' | 'info';
  category: 'schema' | 'security' | 'best-practice';
  span: ProvLintSpan;
  fix?: ProvLintFix;
}

interface WasmLinter {
  lint(content: string, format?: string | null): ProvLintDiagnostic[];
  lintWithConfig(
    content: string,
    format: string,
    config: { disabledRules?: string[] },
  ): ProvLintDiagnostic[];
  detectFormat(content: string): string | undefined;
  getSupportedRules(): Array<{ code: string; description: string; formats: string[] }>;
}

let linterInstance: WasmLinter | null = null;

async function loadLinter(): Promise<WasmLinter> {
  if (linterInstance) return linterInstance;

  const wasmDir = path.join(__dirname, '..', 'wasm');
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  const wasmModule = require(path.join(wasmDir, 'provlint_wasm.js'));
  const instance = new wasmModule.WasmProvLint();
  linterInstance = instance;
  return instance;
}

function detectFormat(
  document: vscode.TextDocument,
  linter: WasmLinter,
): string | undefined {
  const ext = path.extname(document.fileName).toLowerCase();
  if (ext === '.ks' || ext === '.cfg') return 'kickstart';
  if (ext === '.xml') return 'autoyast';
  if (ext === '.yaml' || ext === '.yml') {
    // Check if it looks like autoinstall
    const text = document.getText();
    if (text.startsWith('#cloud-config') || text.includes('autoinstall:')) {
      return 'autoinstall';
    }
    return undefined;
  }
  // Fallback: let the WASM linter detect from content
  return linter.detectFormat(document.getText());
}

function severityToVscode(severity: string): vscode.DiagnosticSeverity {
  switch (severity) {
    case 'error':
      return vscode.DiagnosticSeverity.Error;
    case 'warning':
      return vscode.DiagnosticSeverity.Warning;
    default:
      return vscode.DiagnosticSeverity.Information;
  }
}

export function activate(context: vscode.ExtensionContext) {
  const diagnosticCollection = vscode.languages.createDiagnosticCollection('provlint');
  context.subscriptions.push(diagnosticCollection);

  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  async function lintDocument(document: vscode.TextDocument) {
    const config = vscode.workspace.getConfiguration('provlint');
    if (!config.get<boolean>('enabled', true)) {
      diagnosticCollection.delete(document.uri);
      return;
    }

    let linter: WasmLinter;
    try {
      linter = await loadLinter();
    } catch (err) {
      console.error('[provlint] Failed to load WASM:', err);
      return;
    }

    const format = detectFormat(document, linter);
    if (!format) {
      diagnosticCollection.delete(document.uri);
      return;
    }

    const text = document.getText();
    const disabledRules = config.get<string[]>('disabledRules', []);

    let results: ProvLintDiagnostic[];
    try {
      if (disabledRules.length > 0) {
        results = linter.lintWithConfig(text, format, { disabledRules });
      } else {
        results = linter.lint(text, format);
      }
    } catch (err) {
      console.error('[provlint] Lint error:', err);
      return;
    }

    const diagnostics: vscode.Diagnostic[] = results.map((d) => {
      const range = new vscode.Range(
        Math.max(0, d.span.startLine - 1),
        Math.max(0, d.span.startCol - 1),
        Math.max(0, d.span.endLine - 1),
        Math.max(0, d.span.endCol - 1 || Number.MAX_SAFE_INTEGER),
      );
      const diag = new vscode.Diagnostic(
        range,
        d.message,
        severityToVscode(d.severity),
      );
      diag.code = d.code;
      diag.source = 'provlint';
      if (d.fix) {
        diag.tags = [];
      }
      return diag;
    });

    diagnosticCollection.set(document.uri, diagnostics);
  }

  function scheduleLint(document: vscode.TextDocument) {
    if (debounceTimer) clearTimeout(debounceTimer);
    const debounceMs = vscode.workspace
      .getConfiguration('provlint')
      .get<number>('debounceMs', 500);
    debounceTimer = setTimeout(() => lintDocument(document), debounceMs);
  }

  // Lint on open
  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((doc) => scheduleLint(doc)),
  );

  // Lint on change
  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((e) => scheduleLint(e.document)),
  );

  // Lint on save
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((doc) => lintDocument(doc)),
  );

  // Clean up when document is closed
  context.subscriptions.push(
    vscode.workspace.onDidCloseTextDocument((doc) => {
      diagnosticCollection.delete(doc.uri);
    }),
  );

  // Register code action provider for quick fixes
  context.subscriptions.push(
    vscode.languages.registerCodeActionsProvider(
      [
        { scheme: 'file', language: 'xml' },
        { scheme: 'file', language: 'yaml' },
        { scheme: 'file', language: 'plaintext' },
        { scheme: 'file', language: 'kickstart' },
      ],
      new ProvLintCodeActionProvider(),
      { providedCodeActionKinds: [vscode.CodeActionKind.QuickFix] },
    ),
  );

  // Lint all already-open documents
  vscode.workspace.textDocuments.forEach((doc) => scheduleLint(doc));

  // Register command to show all supported rules
  context.subscriptions.push(
    vscode.commands.registerCommand('provlint.showRules', async () => {
      try {
        const linter = await loadLinter();
        const rules = linter.getSupportedRules();
        const items = rules.map(
          (r: { code: string; description: string; formats: string[] }) => ({
            label: r.code,
            description: r.description,
            detail: `Formats: ${r.formats.join(', ')}`,
          }),
        );
        vscode.window.showQuickPick(items, {
          placeHolder: 'ProvLint supported rules',
        });
      } catch {
        vscode.window.showErrorMessage('Failed to load ProvLint rules');
      }
    }),
  );
}

class ProvLintCodeActionProvider implements vscode.CodeActionProvider {
  async provideCodeActions(
    document: vscode.TextDocument,
    _range: vscode.Range,
    context: vscode.CodeActionContext,
  ): Promise<vscode.CodeAction[]> {
    const actions: vscode.CodeAction[] = [];

    let linter: WasmLinter;
    try {
      linter = await loadLinter();
    } catch {
      return actions;
    }

    const format = detectFormat(document, linter);
    if (!format) return actions;

    const config = vscode.workspace.getConfiguration('provlint');
    const disabledRules = config.get<string[]>('disabledRules', []);

    let results: ProvLintDiagnostic[];
    try {
      if (disabledRules.length > 0) {
        results = linter.lintWithConfig(document.getText(), format, {
          disabledRules,
        });
      } else {
        results = linter.lint(document.getText(), format);
      }
    } catch {
      return actions;
    }

    for (const diag of context.diagnostics) {
      if (diag.source !== 'provlint') continue;

      const match = results.find((r) => r.code === diag.code && r.fix);
      if (!match?.fix) continue;

      const fix = new vscode.CodeAction(
        match.fix.description,
        vscode.CodeActionKind.QuickFix,
      );
      fix.edit = new vscode.WorkspaceEdit();
      fix.edit.replace(document.uri, diag.range, match.fix.replacement);
      fix.diagnostics = [diag];
      fix.isPreferred = true;
      actions.push(fix);

      // Also offer to disable the rule
      const disableAction = new vscode.CodeAction(
        `Disable rule ${diag.code}`,
        vscode.CodeActionKind.QuickFix,
      );
      disableAction.command = {
        command: 'provlint.disableRule',
        title: `Disable ${diag.code}`,
        arguments: [diag.code],
      };
      disableAction.diagnostics = [diag];
      actions.push(disableAction);
    }

    return actions;
  }
}

export function deactivate() {
  linterInstance = null;
}
