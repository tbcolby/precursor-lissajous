# Lissajous Machine — Build Notes

## What This Is
Parametric Lissajous curve generator for Precursor. Renders x=sin(at+δ), y=sin(bt) patterns.

## Key Decisions
- **Reuses Spirograph's fixed-point math**: Same 16.16 FP, parabolic sin/cos, 1024 angle units
- **Ratio table**: Interactive 6×6 grid showing all simple frequency ratios — unique feature
- **Phase in 1/1024 units**: 256 = π/2 (90°). Steps by 64 in menu (≈22.5°)
- **LCM for period**: Unlike spirograph's GCD, Lissajous period = LCM(a,b) cycles
- **No storage**: Pure visual app

## Architecture
- 4 states: Menu, Drawing, Gallery, Table
- 10 presets + full ratio table exploration
- Shared math module pattern with Spirograph
