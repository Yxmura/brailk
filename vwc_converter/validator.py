"""
VWC syntax validator - soft validation that lets converter handle most cases.
"""

from typing import List, NamedTuple, Optional


class ValidationIssue(NamedTuple):
    """A validation issue - severity determines how blocking it is."""

    severity: str  # 'error' (blocking) or 'warning' (non-blocking)
    message: str
    position: int


class StructureToken(NamedTuple):
    """Represents a structural element in VWC code."""

    type: str  # 'open' or 'close'
    code: str
    position: int


class VWCValidator:
    """
    Soft validator - returns issues but doesn't block conversion.

    Only raises hard errors for truly catastrophic cases.
    Most issues are returned as warnings to let the converter try anyway.
    """

    OPENING_CODES = {"$br", "$wt", "$mat", "$det", "$com", "$st"}
    CLOSING_CODES = {"#", "$#"}
    STRUCTURAL_CODES = OPENING_CODES | CLOSING_CODES

    def __init__(self, vwc_input: str):
        self.input = vwc_input
        self.length = len(vwc_input)
        self.pos = 0

    def get_issues(self) -> List[ValidationIssue]:
        """
        Scan input and return all issues (warnings + errors).

        Returns:
            List of ValidationIssue tuples - severity is 'warning' or 'error'
        """
        issues = []
        structures = self._tokenize_structures()

        # Only check for catastrophic issues (actual errors)
        # Missing # is just a warning - converter can handle it
        for issue in self._check_catastrophic_issues(structures):
            issues.append(issue)

        # Check for unbalanced stuff but make it a warning, not error
        for issue in self._check_balancing_warnings(structures):
            issues.append(issue)

        return issues

    def _tokenize_structures(self) -> List[StructureToken]:
        """Scan input and identify structural tokens."""
        structures = []
        while self.pos < self.length:
            found_code = self._match_structural_code()
            if found_code:
                token_type = "close" if found_code in self.CLOSING_CODES else "open"
                structures.append(
                    StructureToken(type=token_type, code=found_code, position=self.pos)
                )
                self.pos += len(found_code)
            else:
                self.pos += 1
        return structures

    def _match_structural_code(self) -> Optional[str]:
        """Check if current position matches a structural code."""
        remaining = self.input[self.pos :]
        for code in sorted(self.STRUCTURAL_CODES, key=len, reverse=True):
            if remaining.startswith(code):
                return code
        return None

    def _check_catastrophic_issues(
        self, structures: List[StructureToken]
    ) -> List[ValidationIssue]:
        """Only return errors for truly breaking issues."""
        issues = []

        # Check for impossible-to-parse patterns
        # Too many consecutive $ signs usually means broken input
        dollar_count = 0
        for i, char in enumerate(self.input):
            if char == "$":
                dollar_count += 1
                if dollar_count >= 5:
                    issues.append(
                        ValidationIssue(
                            "error", "Te veel $ symbolen - controllieer invoer", i
                        )
                    )
                    break
            else:
                dollar_count = 0

        # Check for completely broken structure (e.g., $br$$wt)
        consecutive_opens = 0
        for s in structures:
            if s.type == "open":
                consecutive_opens += 1
                if consecutive_opens >= 3:
                    issues.append(
                        ValidationIssue(
                            "error",
                            "Opeenvolgende structuren - mis je een #?",
                            s.position,
                        )
                    )
                    break
            else:
                consecutive_opens = 0

        return issues

    def _check_balancing_warnings(
        self, structures: List[StructureToken]
    ) -> List[ValidationIssue]:
        """Return warnings for unbalanced structures - not blocking."""
        issues = []

        opens = sum(1 for s in structures if s.type == "open")
        closes = sum(1 for s in structures if s.type == "close")

        # Only warn if the imbalance is significant
        diff = abs(opens - closes)
        if diff >= 2:  # Allow 1 mismatch, warn at 2+
            if opens > closes:
                issues.append(
                    ValidationIssue(
                        "warning",
                        "Mogelijk ontbreekt een # (kan nog steeds werken)",
                        self.length,
                    )
                )
            else:
                issues.append(
                    ValidationIssue(
                        "warning", "Extra # gevonden (wordt genegeerd)", self.length
                    )
                )

        return issues

    def has_blocking_errors(self) -> bool:
        """Check if there are any blocking errors."""
        issues = self.get_issues()
        return any(i.severity == "error" for i in issues)

    def get_warnings(self) -> List[ValidationIssue]:
        """Get only warnings (non-blocking issues)."""
        issues = self.get_issues()
        return [i for i in issues if i.severity == "warning"]


def validate_vwc_soft(vwc_input: str) -> List[ValidationIssue]:
    """
    Soft validation - returns issues but doesn't raise exceptions.

    Use this before conversion to show optional warnings.
    """
    if not vwc_input or not vwc_input.strip():
        return []
    validator = VWCValidator(vwc_input)
    return validator.get_issues()


def is_valid_enough(vwc_input: str) -> bool:
    """
    Check if input is "valid enough" to attempt conversion.

    Returns True unless there are blocking errors.
    """
    if not vwc_input or not vwc_input.strip():
        return False
    validator = VWCValidator(vwc_input)
    return not validator.has_blocking_errors()
