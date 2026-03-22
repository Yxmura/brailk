# VWC to Typst Converter

Convert VWC (Vlaamse Wiskundecode) — the linear mathematical notation used in Flemish education — to Typst.

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
