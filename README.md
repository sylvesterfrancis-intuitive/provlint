# ProvLint

A fast, WASM-powered linter for Linux provisioning config files — **Kickstart**, **AutoYaST**, and **Autoinstall**.

Built in Rust, runs everywhere: CLI, browser (WASM), and VS Code.

## Features

- Real-time validation of unattended install configurations
- 15 lint rules across schema, security, and best-practice categories
- Format auto-detection from content and file extensions
- Source span tracking for precise error locations
- Optional fix suggestions for common issues
- 323 KB optimized WASM binary for browser embedding
- Debounced linting for editor integrations

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
# Install the generated .vsix file in VS Code
```

### WASM (Browser)

Copy the files from `packages/provlint-npm/wasm/` into your project's public directory, then:

```typescript
const wasm = await import('/wasm/provlint_wasm.js');
await wasm.default('/wasm/provlint_wasm_bg.wasm');
const linter = new wasm.WasmProvLint();

const diagnostics = linter.lint(content, 'kickstart');
```

## Usage

### CLI

```bash
# Lint a single file
provlint lint my-kickstart.ks

# Lint multiple files recursively
provlint lint templates/ --recursive

# JSON output
provlint lint config.xml --output json

# Disable specific rules
provlint lint server.ks --disable SEC-001 --disable BP-001

# List all supported rules
provlint rules
```

### VS Code Extension

The extension activates automatically for `.ks`, `.cfg`, `.xml`, `.yaml`, and `.yml` files. Diagnostics appear inline as you type.

Settings:

| Setting | Default | Description |
|---------|---------|-------------|
| `provlint.enabled` | `true` | Enable/disable linting |
| `provlint.disabledRules` | `[]` | Rule codes to suppress |
| `provlint.debounceMs` | `500` | Delay before linting on change |

## Supported Rules

### Schema

| Code | Description | Formats |
|------|-------------|---------|
| SCH-002 | Missing required fields or invalid structure | autoinstall |
| SCH-005 | Invalid AutoYaST XML structure | autoyast |
| SCH-006 | Duplicate directives where only one is allowed | kickstart |

### Security

| Code | Description | Formats |
|------|-------------|---------|
| SEC-001 | Plaintext root password detected | kickstart |
| SEC-002 | SELinux is disabled | kickstart |
| SEC-003 | Firewall is disabled | kickstart |
| SEC-004 | SSH PermitRootLogin is enabled | kickstart, autoinstall, autoyast |
| SEC-005 | Weak password hash algorithm | kickstart |
| SEC-007 | Unencrypted HTTP repository URL | kickstart, autoyast, autoinstall |
| SEC-008 | Empty or default password | kickstart |

### Best Practice

| Code | Description | Formats |
|------|-------------|---------|
| BP-001 | No swap partition defined | kickstart, autoinstall |
| BP-004 | No NTP time source configured | kickstart |
| BP-006 | Deprecated directive used | kickstart |
| BP-007 | No bootloader password set | kickstart |
| BP-008 | Missing network hostname | kickstart, autoinstall |

## Project Structure

```
provlint/
├── crates/
│   ├── provlint-core/     # Parsing engine, rules, diagnostics
│   ├── provlint-cli/      # Command-line interface (clap)
│   └── provlint-wasm/     # WASM bindings (wasm-bindgen)
└── packages/
    ├── provlint-npm/      # TypeScript types + WASM loader
    └── provlint-vscode/   # VS Code extension
```

## Building from Source

### Prerequisites

- Rust 1.75+ with `wasm32-unknown-unknown` target
- wasm-pack
- Node.js 18+

### Build

```bash
# Run tests
cargo test

# Build CLI
cargo build --release -p provlint-cli

# Build WASM (web target)
wasm-pack build crates/provlint-wasm --target web --out-dir ../../packages/provlint-npm/wasm

# Build WASM (Node.js target, for VS Code)
wasm-pack build crates/provlint-wasm --target nodejs --out-dir ../../packages/provlint-npm/wasm-node

# Build VS Code extension
cd packages/provlint-vscode
npm install && npm run build && npm run package
```

## License

AGPL-3.0 — see [LICENSE](LICENSE)
