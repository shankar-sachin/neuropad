# Portable Ruby Runtime

Place a portable Windows Ruby runtime in this folder so builds can bundle it.

Expected executable path:

`services/ruby-portable/bin/ruby.exe`

If this runtime is present, NeuroPad will launch Ruby cells with this embedded executable.
If not present, NeuroPad falls back to system `ruby` on PATH.
