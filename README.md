# Brailk — VWC naar LaTeX Converter

Convert VWC (Vlaamse Wiskundecode) to KaTeX-compatible LaTeX. Entirely client-side — runs in the browser via WebAssembly.

## Stack

- **WASM** — Converter compiled from Rust via `wasm32-unknown-unknown` (68KB, zero imports)
- **JS** — Thin bridge (allocate WASM memory, call converter, parse JSON result)
- **KaTeX** — Client-side LaTeX rendering
- **HTML/CSS** — UI

No backend, no Python, no Pyodide. Open `index.html` directly from the filesystem.

## Usage

Open `index.html` in a browser. Type VWC code in the left panel, click "Omzetten". LaTeX renders on the right.

### VWC examples

| VWC | LaTeX |
|-----|-------|
| `x + y = 3` | `x + y = 3` |
| `$br x+1/x-1#` | `\frac{x+1}{x-1}` |
| `$wt x+1#` | `\sqrt{x+1}` |
| `μa` | `\alpha` |
| `sin(x)` | `\sin(x)` |
| `x^2#` | `x^{2}` |
| `$R` | `\mathbb{R}` |

## Development

The converter is written in Rust in `src/lib.rs`. Build:

```bash
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/vwc_converter.wasm static/
```

Run tests:

```bash
cargo test
```

## Supported Features

- Greek letters (`μa`–`μw`, `μG`–`μV`)
- Dollar code symbols (`$N`, `$Z`, `$R`, `$C`, `$inf`, `$int`, `$+-`, etc.)
- Superscripts/subscripts (`^`, `_`)
- Fractions (`$br ... / ... #`)
- Roots (`$wt ... #`, `^^n#$wt ... #` for indexed roots)
- Line forms (`$bgb`, `$hkb`, `$plrb`, `$pllb`)
- Math functions (`sin`, `cos`, `log`, `lim`, `det`, etc.)
- Text detection (multi-letter words → `\text{...}`)
- Spatiëvuller (`¨` → space)
