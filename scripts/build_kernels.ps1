$ErrorActionPreference = "Stop"

Write-Host "Building Go kernel..."
Push-Location services/go-kernel
go build -o go-kernel.exe .
Pop-Location

Write-Host "Python and Ruby kernels are script-based; no compilation needed."

if (Test-Path "services/ruby-portable/bin/ruby.exe") {
  Write-Host "Portable Ruby runtime found: services/ruby-portable/bin/ruby.exe"
} else {
  Write-Warning "Portable Ruby runtime not found. Ruby cells will require a system Ruby unless you place one at services/ruby-portable/"
}

if (Test-Path "services/python-portable/python.exe") {
  Write-Host "Portable Python runtime found: services/python-portable/python.exe"
} else {
  Write-Host "Portable Python runtime not found. Python cells will use system python if available."
}
