$ErrorActionPreference = "Stop"

Push-Location apps/neuropad-desktop
try {
  Write-Host "Attempting NSIS (.exe) installer build..."
  npm run tauri build -- --bundles nsis
  Write-Host "NSIS build succeeded."
} catch {
  Write-Warning "NSIS build failed. Attempting MSI fallback..."
  npm run tauri build -- --bundles msi
  Write-Host "MSI build succeeded."
}
finally {
  Pop-Location
}
