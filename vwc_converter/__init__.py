"""
VWC (Vlaamse Wiskundecode) to Typst converter.

Package structure:
- constants.py: All lookup tables (Greek letters, dollar codes, etc.)
- converter.py: Core conversion logic (VWCConverter class)
"""

from .converter import VWCConverter, vwc_to_typst

__version__ = "1.0.0"
__all__ = ["VWCConverter", "vwc_to_typst"]
