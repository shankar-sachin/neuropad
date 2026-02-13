# NeuroPad Application Guide

This document explains how the application is organized in this repository.

## Desktop app layer

- Path: `apps/neuropad-desktop/`
- UI framework: Svelte
- Host/runtime: Tauri
- UI calls backend operations using Tauri `invoke(...)` commands.

## Core domain layer

- Path: `crates/neuropad-core/`
- Handles notebook structure, metadata, and file conversions.
- Supports native `.npad` and `.ipynb` interoperability.

## IPC layer

- Path: `crates/neuropad-ipc/`
- Defines shared message envelopes/types used between app and kernels.

## Kernel services

- `services/go-kernel/`: Executes Go code over JSON/stdin-stdout IPC.
- `services/ruby-kernel/`: Executes Ruby code over JSON/stdin-stdout IPC.

## Data schemas

- `schemas/npad.schema.json`: Notebook schema.
- `schemas/ipc.schema.json`: IPC schema.

## Current V1 functional scope

- Markdown + Go + Ruby cells
- Save/load `.npad`
- Import/export `.ipynb`
- Execute code cells and capture text outputs
- Local AI command placeholder in backend (`ai_generate_cell`)
