# Lissajous Machine

A parametric Lissajous curve generator for the [Precursor](https://precursor.dev) handheld, rendering classic frequency-ratio patterns on its 336×536 monochrome 1-bit display.

## Features

- **Adjustable frequency ratios** — A:B from 1:1 to 9:9
- **Phase control** — 16 steps of phase offset (22.5° increments)
- **10 built-in presets** — Circle, Figure-8, Trefoil, Clover, and more
- **Ratio table** — 6×6 interactive grid to explore all simple ratios
- **Fixed-point math** — 16.16 integer arithmetic, no floating-point

## Controls

| Key | Action |
|-----|--------|
| Up/Down | Navigate menu / browse gallery / table rows |
| Left/Right | Adjust parameters / table columns |
| Enter | Draw curve / select preset / select ratio |
| Left/Menu | Return to previous screen |
| Menu | Quit from main menu |

## Architecture

- `math.rs` — Fixed-point 16.16 arithmetic, sin/cos approximation, Lissajous parametric equations, GCD/LCM for periodicity
- `app.rs` — 4-state machine (Menu → Drawing → Gallery → Table), 10 presets
- `ui.rs` — Menu, curve rendering, scrollable gallery, interactive ratio grid
- `main.rs` — Standard Xous event loop

## Presets

| Name | A:B | Phase |
|------|-----|-------|
| Circle | 1:1 | 90° |
| Figure-8 | 1:2 | 0° |
| Bowtie | 2:3 | 0° |
| Trefoil | 3:2 | 90° |
| Clover | 3:4 | 0° |
| Star Knot | 5:4 | 90° |
| Diamond | 1:2 | 90° |
| Infinity | 2:1 | 90° |
| Ribbon | 3:5 | 45° |
| Atomic | 5:6 | 0° |

---

## Development

This app was developed using the methodology described in [xous-dev-toolkit](https://github.com/tbcolby/xous-dev-toolkit) — an LLM-assisted approach to Precursor app development on macOS ARM64.

---

## Author

Made by Tyler Colby — [Colby's Data Movers, LLC](https://colbysdatamovers.com)

Contact: [tyler@colbysdatamovers.com](mailto:tyler@colbysdatamovers.com) | [GitHub Issues](https://github.com/tbcolby/precursor-lissajous/issues)

---

## License

Licensed under the Apache License, Version 2.0.

See [LICENSE](LICENSE) for the full text.
