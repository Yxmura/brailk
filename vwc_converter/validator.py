"""
VWC syntax validator - ensures structures are properly closed and nested.
"""

from typing import List, NamedTuple, Optional, Tuple

from .exceptions import (
    VWCUnclosedStructureError,
    VWCNestingError,
    VWCMismatchedCloserError,
)


class StructureToken(NamedTuple):
    """Represents a structural element in VWC code."""

    type: str  # 'open' or 'close'
    code: str  # e.g., '$br', '$wt', '#', '$#'
    position: int
    line: int
    column: int


class VWCValidator:
    """
    Validates VWC syntax before conversion.

    Checks for:
    - Unclosed structures ($br, $wt, etc. without matching #)
    - Improper nesting (starting new structure before closing previous)
    - Mismatched closers
    """

    # Structure opening codes that require # to close
    OPENING_CODES = {"$br", "$wt", "$mat", "$det", "$com", "$st"}

    # Codes that close structures
    CLOSING_CODES = {"#", "$#"}

    # All structural codes
    STRUCTURAL_CODES = OPENING_CODES | CLOSING_CODES

    def __init__(self, vwc_input: str):
        self.input = vwc_input
        self.length = len(vwc_input)
        self.pos = 0
        self.line = 1
        self.column = 1

    def validate(self) -> None:
        """
        Main validation entry point.

        Raises:
            VWCUnclosedStructureError: If a structure isn't closed
            VWCNestingError: If structures are improperly nested
            VWCMismatchedCloserError: If closers don't match properly
        """
        structures = self._tokenize_structures()
        self._validate_structure_balance(structures)
        self._validate_nesting(structures)

    def _tokenize_structures(self) -> List[StructureToken]:
        """
        Scan input and identify all structural tokens.

        Returns:
            List of StructureToken objects
        """
        structures = []

        while self.pos < self.length:
            char = self.input[self.pos]

            # Track line/column for error reporting
            if char == "\n":
                self.line += 1
                self.column = 1
            else:
                self.column += 1

            # Check for structural codes
            found_code = self._match_structural_code()

            if found_code:
                token_type = "close" if found_code in self.CLOSING_CODES else "open"
                structures.append(
                    StructureToken(
                        type=token_type,
                        code=found_code,
                        position=self.pos,
                        line=self.line,
                        column=self.column - len(found_code),
                    )
                )
                self.pos += len(found_code)
            else:
                self.pos += 1

        return structures

    def _match_structural_code(self) -> Optional[str]:
        """
        Check if current position matches a structural code.

        Returns:
            The matched code if found, None otherwise
        """
        remaining = self.input[self.pos :]

        # Try longer codes first (e.g., $# before #)
        for code in sorted(self.STRUCTURAL_CODES, key=len, reverse=True):
            if remaining.startswith(code):
                return code

        return None

    def _validate_structure_balance(self, structures: List[StructureToken]) -> None:
        """
        Check that all opening codes have corresponding closers.

        Args:
            structures: List of structural tokens

        Raises:
            VWCUnclosedStructureError: If balance is incorrect
        """
        # Simple count check first
        opens = sum(1 for s in structures if s.type == "open")
        closes = sum(1 for s in structures if s.type == "close")

        if opens > closes:
            # Find the last unclosed structure
            stack = []
            for s in structures:
                if s.type == "open":
                    stack.append(s)
                elif s.type == "close" and stack:
                    stack.pop()

            if stack:
                last_unclosed = stack[-1]
                raise VWCUnclosedStructureError(
                    last_unclosed.code, last_unclosed.position
                )
        elif closes > opens:
            # Find first extra closer
            depth = 0
            for s in structures:
                if s.type == "open":
                    depth += 1
                elif s.type == "close":
                    if depth == 0:
                        raise VWCMismatchedCloserError("niets", s.code, s.position)
                    depth -= 1

    def _validate_nesting(self, structures: List[StructureToken]) -> None:
        """
        Validate that structures are properly nested (no overlapping).

        Args:
            structures: List of structural tokens

        Raises:
            VWCNestingError: If nesting is invalid
        """
        stack: List[StructureToken] = []

        for token in structures:
            if token.type == "open":
                # Check if we're trying to start same structure type (likely forgot to close)
                if stack and stack[-1].code == token.code:
                    raise VWCNestingError(stack[-1].code, token.code, token.position)
                stack.append(token)

            elif token.type == "close":
                if not stack:
                    raise VWCMismatchedCloserError("niets", token.code, token.position)
                # Pop the matching opener
                stack.pop()


def validate_vwc(vwc_input: str) -> None:
    """
    Convenience function to validate VWC input.

    Args:
        vwc_input: The VWC code to validate

    Raises:
        VWCUnclosedStructureError: If structures aren't closed
        VWCNestingError: If improper nesting detected
        VWCMismatchedCloserError: If closers don't match
    """
    validator = VWCValidator(vwc_input)
    validator.validate()


def get_validation_errors(vwc_input: str) -> List[dict]:
    """
    Validate and return list of errors without raising.

    Args:
        vwc_input: The VWC code to validate

    Returns:
        List of error dictionaries, empty if valid
    """
    try:
        validate_vwc(vwc_input)
        return []
    except (VWCUnclosedStructureError, VWCNestingError, VWCMismatchedCloserError) as e:
        return [e.to_dict()]
    except Exception as e:
        return [{"error_type": "unknown", "message": str(e)}]
