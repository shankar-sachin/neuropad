$ErrorActionPreference = "Stop"

Write-Host "Preparing kernels..."
powershell -ExecutionPolicy Bypass -File scripts/build_kernels.ps1

Write-Host "Starting desktop app..."
Push-Location apps/neuropad-desktop
npm run tauri dev
Pop-Location
