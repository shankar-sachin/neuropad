# NeuroPad Overview

NeuroPad is a desktop-first, polyglot notebook app.

## What it supports

- Notebook cells: Markdown and code.
- Code languages: Go and Ruby.
- Native format: `.npad`.
- Interop format: `.ipynb` import/export.
- Desktop shell: Tauri (Rust backend + Svelte UI).

## Core features in this repository

- Create/open/save notebooks.
- Add markdown, Go, or Ruby cells.
- Execute code cells and view text outputs.
- Manage per-notebook kernel processes for Go and Ruby.

## Project structure (high level)

- `apps/neuropad-desktop/`: Desktop app (Svelte + Tauri).
- `crates/neuropad-core/`: Notebook domain, metadata, file format logic.
- `crates/neuropad-ipc/`: Shared IPC envelope/types.
- `services/go-kernel/`: Go execution kernel.
- `services/ruby-kernel/`: Ruby execution kernel.
- `schemas/`: JSON schemas for notebook and IPC.
