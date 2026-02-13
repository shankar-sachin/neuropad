$ErrorActionPreference = "Stop"

Write-Host "Building Go kernel..."
Push-Location services/go-kernel
go build -o go-kernel.exe .
Pop-Location

Write-Host "Ruby kernel is script-based; no compilation needed."
