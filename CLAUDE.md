# Lissajous build notes

- Maths in `core/src/lib.rs`: shared `fixed::sin` (Q16.16, 4096 units/turn). A:B is reduced by gcd so the curve closes after one turn (`Curve::len()` segments); step is a power of two so `point(len()) == point(0)` exactly.
- Points are computed from the index (`Curve::point`), no buffer. `Tracer::advance` returns each segment index once; the shell draws only new segments per pump tick.
- The pump runs only when foreground and `App::animating()`; `main.rs` re-evaluates after every message.
- No storage. Back = Backspace/F4; `∴` is never delivered.
- Tests: `cd core && cargo test`.
