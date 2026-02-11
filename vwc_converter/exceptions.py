"""
Custom exceptions for VWC parsing and conversion.
"""

from typing import Optional


class VWCError(Exception):
    """Base exception for all VWC-related errors."""

    pass


class VWCSyntaxError(VWCError):
    """
    Raised when VWC code has syntax errors (unclosed structures, invalid nesting, etc.).

    Attributes:
        message: Human-readable error description
        position: Character position where error occurred (optional)
        suggestion: Helpful hint on how to fix the error
    """

    def __init__(
        self,
        message: str,
        position: Optional[int] = None,
        suggestion: Optional[str] = None,
    ):
        self.message = message
        self.position = position
        self.suggestion = suggestion
        super().__init__(self._format_message())

    def _format_message(self) -> str:
        """Format the error message with position and suggestion."""
        parts = [self.message]
        if self.position is not None:
            parts.append(f" (positie {self.position})")
        if self.suggestion:
            parts.append(f". Tip: {self.suggestion}")
        return "".join(parts)

    def to_dict(self) -> dict:
        """Convert error to dictionary for JSON serialization."""
        return {
            "error_type": "syntax",
            "message": self.message,
            "position": self.position,
            "suggestion": self.suggestion,
        }


class VWCValidationError(VWCError):
    """Raised when VWC code fails validation checks."""

    pass


class VWCNestingError(VWCSyntaxError):
    """Raised when structures are improperly nested."""

    def __init__(
        self, outer_structure: str, inner_structure: str, position: Optional[int] = None
    ):
        message = f"Ongeldige nesting: je kunt geen {inner_structure} starten binnen een onafgesloten {outer_structure}"
        suggestion = f"Sluit eerst de {outer_structure} af met # voordat je {inner_structure} gebruikt"
        super().__init__(message, position, suggestion)
        self.outer_structure = outer_structure
        self.inner_structure = inner_structure


class VWCUnclosedStructureError(VWCSyntaxError):
    """Raised when a structure is not properly closed."""

    STRUCTURE_NAMES = {
        "$br": "breuk ($br)",
        "$wt": "wortel ($wt)",
        "$mat": "matrix ($mat)",
        "$det": "determinant ($det)",
        "$com": "combinatie ($com)",
        "$st": "stelsel ($st)",
        "^": "superscript",
        "_": "subscript",
    }

    def __init__(self, structure: str, position: Optional[int] = None):
        structure_name = self.STRUCTURE_NAMES.get(structure, structure)
        message = f"Niet afgesloten {structure_name}"
        suggestion = f"Voeg een # toe om de {structure_name} af te sluiten"
        super().__init__(message, position, suggestion)
        self.structure = structure


class VWCMismatchedCloserError(VWCSyntaxError):
    """Raised when closing markers don't match opening markers."""

    def __init__(self, expected: str, found: str, position: Optional[int] = None):
        message = (
            f"Verkeerde afsluiter: verwachtte # voor {expected}, maar vond {found}"
        )
        suggestion = (
            f"Zorg dat elke $br, $wt, etc. afgesloten wordt met # in de juiste volgorde"
        )
        super().__init__(message, position, suggestion)
        self.expected = expected
        self.found = found
