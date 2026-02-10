# VWC to Typst Converter

Convert VWC (Vlaamse Wiskundecode) — the linear mathematical notation used in Flemish education — to Typst.

> **Note:** This code is simpler than it looks. Don't let the jargon scare you — it's just a loop with some if-statements. See [How It Works](#how-it-works-explained-simply) below for a step-by-step explanation.

## Project Structure

```
brailk/
├── vwc_converter/          # Core package
│   ├── __init__.py        # Package exports
│   ├── constants.py       # Lookup tables (Greek, dollar codes, etc.)
│   └── converter.py       # Main VWCConverter class
├── app.py                 # Flask API server
├── cli.py                 # Command-line interface
├── tests.py               # Test suite
└── README.md
```

## Usage

### As a Python library

```python
from vwc_converter import vwc_to_typst

result = vwc_to_typst("$br 2x+3/y#")
print(result)  # $frac(2x+3, y)$

# Display math (block mode)
result = vwc_to_typst("$int_a#^b#f(x)dx", display=True)
print(result)  # $ integral_a^bf(x)dx $
```

### Command line

```bash
# Convert inline
python cli.py "μa+μb"
# Output: $alpha+beta$

# Convert with display math
python cli.py --display "$br x+1/x-1#"
# Output: $ frac(x+1, x-1) $

# Interactive mode
python cli.py --interactive

# Read from file
python cli.py -f input.vwc --no-wrap
```

### Flask API

```bash
# Start server
python app.py

# Or with Flask CLI
export FLASK_APP=app.py
flask run
```

Then POST to `/convert`:

```bash
curl -X POST http://localhost:5000/convert \
  -H "Content-Type: application/json" \
  -d '{"vwc": "μp", "display": true}'
```

Response:
```json
{
  "typst": "$ pi $",
  "input": "μp"
}
```

## Running Tests

```bash
python tests.py              # Quick summary
python tests.py --verbose    # Show all results
```

## Supported Features

- Greek letters (μ prefix)
- All dollar code symbols ($N, $Z, $elm, $int, etc.)
- Superscripts and subscripts (^, _)
- Fractions ($br ... / ... #)
- Roots ($wt, ^^n#$wt)
- Systems of equations ($st)
- Matrices ($mat) and determinants ($det)
- Binomials ($com)
- Line forms/accents ($bgb, $hkb, etc.)
- Spatiëvuller (¨ → space)

## How It Works (Explained Simply)

Don't worry if this looks complicated — it's actually just a few simple ideas put together.

### The Main Loop

The converter looks at the VWC string one character at a time, from left to right. Think of it like reading a book:

```python
while not at the end:
    look at current character
    if it's "μ": handle Greek letter
    if it's "$": handle dollar code  
    if it's "^": handle superscript
    if it's "_": handle subscript
    otherwise: just copy the character
```

That's it. No magic.

### What "Handle" Means

When we see something special (like `$br` for a fraction), we do three things:

1. **Skip** the code (`$br`)
2. **Collect** everything between the code and its closing `#`
3. **Convert** that collected text to Typst, then wrap it in `frac(...)`

### Example: Converting a Fraction

Input: `$br 2x+3/y#`

What happens:
1. See `$` → look at what follows → it's `br` (fraction)
2. Collect everything until we hit `/` → `2x+3` (this is the numerator)
3. Collect everything until we hit `#` → `y` (this is the denominator)
4. Create a new converter for the numerator text `2x+3`
   - This gets converted to `2x+3` (just numbers and letters, nothing special)
5. Create a new converter for the denominator text `y`
   - This gets converted to `y`
6. Output: `frac(2x+3, y)`

The key idea: **We create new converters for the inner parts**. This handles nesting automatically.

### Nested Example: Fraction Inside Fraction

Input: `$br $br a/b# / c/d# #`

When we see the first `$br`, we start collecting. But then we see ANOTHER `$br` inside! 

We use a **counter** to track how deep we are:
- First `$br`: counter = 1
- Second `$br`: counter = 2
- First `#`: counter = 1 (we're still inside the outer fraction)
- Second `#`: counter = 0 (now we're done)

This way we know which `#` closes which fraction.

### The Code Structure

**`constants.py`** — Just big dictionaries:
- `GREEK = {"a": "alpha", "b": "beta", ...}` 
- `DOLLAR_CODES = {"N": "NN", "Z": "ZZ", ...}`

**`converter.py`** — The VWCConverter class with methods:
- `convert()` — the main loop that goes character by character
- `handle_greek()` — looks up `μa` → `alpha`
- `handle_dollar()` — figures out which $ code it is
- `handle_fraction()` — collects numerator and denominator
- `handle_root()` — handles `$wt` for square roots
- Plus methods for matrices, systems, etc.

Each "handler" does the same pattern: skip the marker, collect the content, convert the content, output the result.

### Why This Works

We don't build a tree or use any fancy data structures. We just:
1. Scan left to right
2. When we see something special, handle it
3. For nested stuff, use a counter to track depth
4. For the inner parts, create a new converter and let it do its job

It's like peeling an onion — each layer handles its own stuff, then passes the result up.

### Want to Understand the Code?

Start with `converter.py` and read the `convert()` method. It's literally a while loop with a bunch of if-statements. Everything else is just helper methods that do the collecting and converting.
