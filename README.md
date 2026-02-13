# NeuroPad

NeuroPad is a desktop-first polyglot notebook app with:
- Rust core + desktop orchestration (Tauri)
- Go kernel for Go code cells
- Ruby kernel for Ruby code cells
- Native `.npad` notebooks plus `.ipynb` import/export

## Monorepo Layout

- `apps/neuropad-desktop/`: Svelte UI + Tauri host app
- `crates/neuropad-core/`: notebook domain, `.npad`, `.ipynb`, metadata
- `crates/neuropad-ipc/`: shared kernel IPC envelope types
- `services/go-kernel/`: Go kernel process (JSON over stdio)
- `services/ruby-kernel/`: Ruby kernel process (JSON over stdio)
- `schemas/`: JSON schemas for notebook and IPC
- `scripts/`: helper scripts for local development

## Requirements

- Rust toolchain (stable)
- Go 1.22+
- Ruby 3.x+
- Node.js 20+ and npm

## Quick Start (Windows)

1. Build the Go kernel:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_kernels.ps1
```
2. Install desktop dependencies:
```powershell
cd apps/neuropad-desktop
npm install
```
3. Run desktop app:
```powershell
npm run tauri dev
```

## Current V1 Scope in this implementation

- Notebook model with markdown and `go`/`ruby` code cells
- Save/load native `.npad`
- Import/export `.ipynb` (common markdown/code/text outputs)
- Per-notebook Go+Ruby process management in Rust kernel manager
- Core Tauri commands for notebook and execution operations
- Optional local-AI command placeholder (`ai_generate_cell`)
