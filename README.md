# precursor-lissajous

Lissajous curves for [Precursor](https://precursor.dev): `x = sin(a t + phase)`, `y = sin(b t)` drawn as an animated pen on the monochrome display, with presets and a 9x9 table of thumbnails to explore ratios.

## Features

- Frequencies A and B from 1 to 9, phase in 16 steps of 22.5 degrees.
- 10 presets (Circle, Figure-8, Trefoil, Star Knot, ...).
- Ratio table: 81 thumbnails (column = x frequency, row = y frequency). Enter draws the selected ratio with your current phase.
- The curve is traced a few dozen segments per tick and only the new segments are drawn, so nothing is drawn twice. The pen stops when the app is in the background or the curve is finished.
- Integer maths only (shared Q16.16 sine table from `precursor-appkit-core`); no floating point.

## How the curve is computed

A:B is divided by gcd(A,B) first, which gives the same figure that closes after exactly one turn (an earlier version ran `lcm(A,B)` turns and retraced the figure up to 81 times). Points are computed on demand from the index, so memory does not grow with the curve. The step between samples shrinks as frequency rises (2 to 16 turn units of 4096), keeping segments under 16 px.

## Keys

| Screen | Keys |
|---|---|
| Menu | Up/Down field, Left/Right change A, B or phase, Enter on Draw / Presets / Ratio table, Backspace or F4 quit |
| Drawing | Left/Right change phase (restarts the pen), Enter restart, Backspace or F4 back |
| Presets | Up/Down, Enter draws, Backspace or F4 back |
| Ratio table | Arrows move, Enter draws, Backspace or F4 back |

The menu key (centre of the D-pad) is consumed by Xous and never reaches the app.

## Storage

None. Nothing is saved.

## Limitations

- Layout (thumbnail size, header and footer fit) has not been checked on a device. Renode capture not yet run: `capture.py` is written but not executed; verification so far is the hosted build, clippy, lint and host tests.
- Equal amplitude on both axes, so a Circle stays round whatever the canvas shape.
- 81 thumbnails of 64 segments cost about 160 batched GAM messages per redraw; the table redraws on each key press.

## Building

```bash
bin/integrate --xous-root ~/xous-core --grade ~/Dev/precursor/precursor-lissajous
```

Host tests alone: `cd core && cargo test`.

## Layout

```
core/src/lib.rs     params, presets, Curve (on-demand points), Tracer (incremental pen)
core/src/tests.rs   closure, no-retrace, radius, float-reference and bound tests
src/main.rs         GAM registration, pump control (foreground and animating only)
src/app.rs          key mapping and drawing
capture.py          Renode capture sequence (not yet run)
```

## Author

Tyler Colby, [Colby's Data Movers, LLC](https://colbysdatamovers.com). Licensed under the Apache License, Version 2.0 (see [LICENSE](LICENSE)).
