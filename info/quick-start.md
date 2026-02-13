# NeuroPad Quick Start (Windows)

## Prerequisites

- Rust toolchain (stable)
- Go 1.22+
- Ruby 3.x+
- Node.js 20+ and npm

## 1. Build kernels

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_kernels.ps1
```

## 2. Install desktop dependencies

```powershell
cd apps/neuropad-desktop
npm install
```

## 3. Run NeuroPad desktop app

```powershell
npm run tauri dev
```

## First-run usage

1. Set a notebook title.
2. Enter a full file path ending in `.npad` (for example: `C:\work\demo.npad`).
3. Click `New`.
4. Add cells with `+ Markdown`, `+ Go Cell`, or `+ Ruby Cell`.
5. Click `Run` on a code cell to execute it.
6. Click `Save` to write the notebook to disk.
