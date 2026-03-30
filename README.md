# ProvLint

A fast, WASM-powered linter for Linux provisioning config files — **Kickstart**, **AutoYaST**, and **Autoinstall**.

Built in Rust. Runs everywhere: CLI, browser (WASM), and VS Code.

## Why ProvLint?

Unattended install configs are critical infrastructure — a typo in a kickstart file can brick a fleet of servers. ProvLint catches schema errors, security misconfigurations, and best-practice violations before you push to production.

- **15 lint rules** across schema, security, and best-practice categories
- **3 parsers** — Kickstart (line-based), AutoYaST (XML), Autoinstall (YAML)
- **Format auto-detection** from file content and extensions
- **Source spans** for precise error locations (line + column)
- **Fix suggestions** for common issues (e.g. `--plaintext` → `--iscrypted`)
- **323 KB** optimized WASM binary for browser embedding
- **21 tests** covering parsers, rules, and end-to-end scenarios

## Quick Start

### CLI

```bash
# Install from source
cargo install --path crates/provlint-cli

# Lint a kickstart file
provlint lint server.ks

#   [SEC-001] warning at line 5: Root password set with --plaintext; use --iscrypted with a hashed password instead
#   [SEC-002] warning at line 6: SELinux is disabled; consider using --enforcing or --permissive
#   [BP-007]  info at line 8: Bootloader has no password set; consider adding --password for physical security
#
#   Found 0 errors, 2 warnings, 1 info in server.ks
```

### WASM (Browser)

```typescript
const wasm = await import('/wasm/provlint_wasm.js');
await wasm.default('/wasm/provlint_wasm_bg.wasm');
const linter = new wasm.WasmProvLint();

const diagnostics = linter.lint(content, 'kickstart');
// [{ code: "SEC-001", severity: "warning", message: "Root password set with ...", span: { startLine: 5, ... } }]
```

### VS Code

Build and install the extension, then open any `.ks`, `.cfg`, `.xml`, or `.yaml` file — diagnostics appear inline as you type.

## Installation

### CLI

```bash
cargo install --path crates/provlint-cli
```

### VS Code Extension

```bash
cd packages/provlint-vscode
npm install && npm run build
npm run package
# Then in VS Code: Extensions > Install from VSIX > select provlint-0.1.0.vsix
```

### NPM Package

```bash
# Build WASM first
wasm-pack build crates/provlint-wasm --target web --out-dir ../../packages/provlint-npm/wasm
```

```typescript
import { initProvLint } from '@provlint/core';

const linter = await initProvLint();
const diagnostics = linter.lint(content, 'kickstart');
```

## CLI Reference

### `provlint lint`

Lint one or more provisioning files.

```
provlint lint [OPTIONS] <PATHS>...
```

| Flag | Description |
|------|-------------|
| `<PATHS>...` | Files or directories to lint |
| `-f, --format <FORMAT>` | Force format: `kickstart`, `autoyast`, `autoinstall` (auto-detected if omitted) |
| `-o, --output <OUTPUT>` | Output format: `text` (default) or `json` |
| `-r, --recursive` | Recurse into directories |
| `--disable <RULE>` | Disable specific rules by code (repeatable) |

**Examples:**

```bash
# Lint a single file
provlint lint my-kickstart.ks

# Lint an entire directory recursively
provlint lint templates/ --recursive

# JSON output for CI pipelines
provlint lint config.xml --output json

# Disable noisy rules
provlint lint server.ks --disable SEC-001 --disable BP-001

# Force format when extension is ambiguous
provlint lint cloud-init.cfg --format kickstart
```

**Exit codes:**
- `0` — no errors found (warnings/info don't affect exit code)
- `1` — one or more errors found

### `provlint rules`

List all supported rules with their codes, descriptions, and applicable formats.

```bash
provlint rules

# Code       Description                                                  Formats
# ------------------------------------------------------------------------------------------
# SCH-006    Duplicate directives where only one is allowed               kickstart
# SEC-001    Plaintext root password detected                             kickstart
# BP-004     No NTP time source configured                                kickstart
# ...
```

## Format Detection

ProvLint auto-detects the provisioning format using file extensions and content analysis:

| Extension | Format |
|-----------|--------|
| `.ks`, `.cfg` | Kickstart |
| `.xml` | AutoYaST |
| `.yaml`, `.yml` | Autoinstall (if contains `#cloud-config` or `autoinstall:`) |

**Content-based detection** kicks in when the extension is ambiguous. ProvLint looks for format-specific markers:

- **Kickstart** — directives like `rootpw`, `bootloader`, `%packages`, `#version=RHEL`
- **AutoYaST** — XML with `<?xml`, `<!DOCTYPE profile`, or `<profile`
- **Autoinstall** — YAML starting with `#cloud-config` or containing `autoinstall:`

## Supported Rules

### Schema Rules

Structural and format validation errors.

| Code | Severity | Formats | Description |
|------|----------|---------|-------------|
| SCH-002 | Error | autoinstall | Missing required `version` field or invalid structure |
| SCH-005 | Error | autoyast | Invalid XML structure or missing `<profile>` root element |
| SCH-006 | Warning | kickstart | Duplicate singleton directive (e.g. two `rootpw` lines) |

**SCH-006 singleton directives:** `bootloader`, `clearpart`, `autopart`, `firewall`, `selinux`, `rootpw`, `timezone`, `lang`, `keyboard`, `zerombr`, `ignoredisk`, `text`, `graphical`, `cmdline`, `eula`, `url`, `cdrom`, `skipx`

### Security Rules

Configuration patterns that weaken system security.

| Code | Severity | Formats | Description |
|------|----------|---------|-------------|
| SEC-001 | Warning | kickstart | Plaintext root password — use `--iscrypted` with a hashed password |
| SEC-002 | Warning | kickstart | SELinux disabled — consider `--enforcing` or `--permissive` |
| SEC-003 | Warning | kickstart | Firewall disabled — consider enabling with port rules |
| SEC-004 | Warning | all | SSH `PermitRootLogin` enabled — use key-based auth |
| SEC-005 | Warning | kickstart | Weak hash algorithm (MD5 `$1$` or SHA-256 `$5$`) — use SHA-512 `$6$` |
| SEC-007 | Info | all | HTTP repository URL — consider HTTPS |
| SEC-008 | Error | kickstart | Empty or default root password |

### Best Practice Rules

Recommendations for production-ready configurations.

| Code | Severity | Formats | Description |
|------|----------|---------|-------------|
| BP-001 | Info | kickstart, autoinstall | No swap partition defined |
| BP-004 | Info | kickstart | No NTP time source configured |
| BP-006 | Info | kickstart | Deprecated directive (`auth`/`authconfig` → `authselect`, `install` is implied) |
| BP-007 | Info | kickstart | No bootloader password set for physical security |
| BP-008 | Info | kickstart, autoinstall | No network hostname configured |

## VS Code Extension

### Activation

The extension activates automatically when:
- Opening `.ks`, `.cfg`, `.xml`, `.yaml`, or `.yml` files
- A workspace contains `*.ks` or `*.cfg` files

### Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `provlint.enabled` | boolean | `true` | Enable/disable ProvLint diagnostics |
| `provlint.disabledRules` | string[] | `[]` | Rule codes to suppress (e.g. `["BP-001", "SEC-005"]`) |
| `provlint.debounceMs` | number | `500` | Debounce delay in ms before linting after a change |

### Features

- **Inline diagnostics** — errors, warnings, and info markers appear directly in the editor
- **Quick fixes** — lightbulb actions for rules that provide fix suggestions
- **Disable rule** — inline code action to add a rule to `provlint.disabledRules`
- **Rule browser** — run `ProvLint: Show Rules` from the command palette to browse all 15 rules
- **Kickstart language** — registers `.ks` and `.cfg` as the Kickstart language with `#` comment support

## WASM / TypeScript API

### Types

```typescript
interface Diagnostic {
  code: string;                              // "SEC-001", "BP-007", etc.
  message: string;                           // Human-readable description
  severity: 'error' | 'warning' | 'info';
  category: 'schema' | 'security' | 'best-practice';
  span: Span;
  fix?: Fix;                                 // Present when an auto-fix is available
}

interface Span {
  startLine: number;    // 1-based
  startCol: number;     // 1-based
  endLine: number;
  endCol: number;
}

interface Fix {
  description: string;  // e.g. "Replace --plaintext with --iscrypted"
  replacement: string;  // The replacement text
}

type Format = 'kickstart' | 'autoyast' | 'autoinstall';
```

### Methods

```typescript
const linter = new WasmProvLint();

// Lint with auto-detection
linter.lint(content: string, format?: string | null): Diagnostic[]

// Lint with disabled rules
linter.lintWithConfig(content: string, format: string, config: { disabledRules?: string[] }): Diagnostic[]

// Detect format from content
linter.detectFormat(content: string): string | undefined

// List all rules
linter.getSupportedRules(): Array<{ code: string; description: string; formats: string[] }>
```

## Project Structure

```
provlint/
├── crates/
│   ├── provlint-core/          # Parsing engine, rules, diagnostics
│   │   └── src/
│   │       ├── parser/         # Kickstart, AutoYaST, Autoinstall parsers
│   │       ├── rules/          # Schema, security, best-practice rules
│   │       ├── diagnostic.rs   # Diagnostic, Severity, Fix types
│   │       ├── config.rs       # Format detection, LintConfig
│   │       ├── rule.rs         # LintRule trait, RuleRegistry
│   │       ├── span.rs         # Source span tracking
│   │       └── lib.rs          # Public API: ProvLint struct
│   ├── provlint-cli/           # CLI binary (clap)
│   └── provlint-wasm/          # WASM bindings (wasm-bindgen)
└── packages/
    ├── provlint-npm/           # TypeScript types + WASM loader
    └── provlint-vscode/        # VS Code extension
```

## Building from Source

### Prerequisites

- Rust 1.75+ with `wasm32-unknown-unknown` target
- wasm-pack 0.14+
- Node.js 18+

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack
```

### Build

```bash
# Run all tests (21 tests)
cargo test

# Build CLI (release)
cargo build --release -p provlint-cli

# Build WASM (browser target)
wasm-pack build crates/provlint-wasm --target web --out-dir ../../packages/provlint-npm/wasm

# Build WASM (Node.js target, for VS Code extension)
wasm-pack build crates/provlint-wasm --target nodejs --out-dir ../../packages/provlint-npm/wasm-node

# Build and package VS Code extension
cd packages/provlint-vscode
npm install && npm run build && npm run package
```

