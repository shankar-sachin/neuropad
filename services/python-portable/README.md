# Portable Python Runtime

Place a portable Windows Python runtime in this folder so builds can bundle it.

Expected executable path:

`services/python-portable/python.exe`

If this runtime is present, NeuroPad will launch Python cells with this embedded executable.
If not present, NeuroPad falls back to system `python` on PATH.
