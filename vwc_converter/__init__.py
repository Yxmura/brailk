"""
VWC (Vlaamse Wiskundecode) to Typst converter.

Package structure:
- constants.py: All lookup tables (Greek letters, dollar codes, etc.)
- converter.py: Core conversion logic (VWCConverter class)
- exceptions.py: Custom exception classes
- validator.py: Soft syntax validation (warnings, not errors)
"""

from .converter import VWCConverter, vwc_to_typst
from .exceptions import VWCError, VWCSyntaxError
from .validator import validate_vwc_soft, is_valid_enough, ValidationIssue

__version__ = "1.0.0"
__all__ = [
    "VWCConverter",
    "vwc_to_typst",
    "VWCError",
    "VWCSyntaxError",
    "validate_vwc_soft",
    "is_valid_enough",
    "ValidationIssue",
]
