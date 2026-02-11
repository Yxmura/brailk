"""
VWC (Vlaamse Wiskundecode) to Typst converter.

Package structure:
- constants.py: All lookup tables (Greek letters, dollar codes, etc.)
- converter.py: Core conversion logic (VWCConverter class)
- exceptions.py: Custom exception classes
- validator.py: Syntax validation
"""

from .converter import VWCConverter, vwc_to_typst
from .exceptions import (
    VWCError,
    VWCSyntaxError,
    VWCValidationError,
    VWCNestingError,
    VWCUnclosedStructureError,
    VWCMismatchedCloserError,
)
from .validator import validate_vwc, get_validation_errors, VWCValidator

__version__ = "1.0.0"
__all__ = [
    "VWCConverter",
    "vwc_to_typst",
    "VWCError",
    "VWCSyntaxError",
    "VWCValidationError",
    "VWCNestingError",
    "VWCUnclosedStructureError",
    "VWCMismatchedCloserError",
    "validate_vwc",
    "get_validation_errors",
    "VWCValidator",
]
