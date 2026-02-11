"""
Core VWC to Typst converter implementation.

The VWCConverter class scans through VWC input character by character,
recognizes patterns, and builds Typst output.
"""

from .constants import (
    DOLLAR_CODES,
    DOLLAR_CODES_SORTED,
    GREEK,
    LINE_FORMS,
    LINE_FORMS_SORTED,
    STRUCTURAL_CODES,
)


# Math function names that Typst recognizes as built-in.
# These must NOT be split into separate letters.
MATH_FUNCTIONS = {
    "log",
    "ln",
    "lg",
    "exp",
    "sin",
    "cos",
    "tan",
    "cot",
    "sec",
    "csc",
    "arcsin",
    "arccos",
    "arctan",
    "arccot",
    "sinh",
    "cosh",
    "tanh",
    "coth",
    "lim",
    "max",
    "min",
    "sup",
    "inf",
    "det",
    "dim",
    "ker",
    "mod",
    "gcd",
    "deg",
    "arg",
}


class VWCConverter:
    """
    Converts a VWC string to Typst.

    Scan through the input string character by character. When we see
    something special (like $ or μ), handle it. Otherwise pass through.

    For nested structures (fractions, roots), we use recursive conversion
    of the inner parts.
    """

    def __init__(self, vwc_input: str):
        self.input = vwc_input
        self.pos = 0
        self.output = []  # We'll join this at the end

    # ------------------------------------------------------------------
    # Position/reading utilities
    # ------------------------------------------------------------------

    def peek(self, offset=0) -> str:
        """Look at a character without moving forward. Returns '' if past end."""
        idx = self.pos + offset
        if idx < len(self.input):
            return self.input[idx]
        return ""

    def _try_math_function(self) -> str | None:
        """If current position starts with a known math function name
        followed by a non-letter (or end of input), consume and return it.
        Returns None if no match."""
        remaining = self.input[self.pos :]
        for func in sorted(MATH_FUNCTIONS, key=len, reverse=True):
            if remaining.startswith(func):
                # Must be followed by non-letter (so 'logy' doesn't match 'log')
                after = self.pos + len(func)
                if after >= len(self.input) or not self.input[after].isalpha():
                    self.advance(len(func))
                    return func
        return None

    def _peek_math_function(self) -> bool:
        """Check if current position starts with a math function name
        without consuming anything."""
        remaining = self.input[self.pos :]
        for func in sorted(MATH_FUNCTIONS, key=len, reverse=True):
            if remaining.startswith(func):
                after = self.pos + len(func)
                if after >= len(self.input) or not self.input[after].isalpha():
                    return True
        return False

    def advance(self, n=1):
        """Move cursor forward by n characters."""
        self.pos += n

    def eat(self, n=1) -> str:
        """Read n characters and move forward."""
        result = self.input[self.pos : self.pos + n]
        self.pos += n
        return result

    def starts_with(self, s: str) -> bool:
        """Check if remaining input starts with string s."""
        return self.input[self.pos : self.pos + len(s)] == s

    def emit(self, s: str):
        """Add string to output."""
        self.output.append(s)

    # ------------------------------------------------------------------
    # Main conversion loop
    # ------------------------------------------------------------------

    def convert(self) -> str:
        """Main entry point. Returns the Typst string."""
        # Validate syntax before conversion
        from .validator import validate_vwc

        validate_vwc(self.input)

        while self.pos < len(self.input):
            ch = self.peek()

            if ch == "μ" or ch == "µ":  # U+03BC (Greek mu) or U+00B5 (micro sign)
                self.handle_greek()
            elif ch == "$":
                self.handle_dollar()
            elif ch == "¨":
                self.handle_spatievuller()
            elif ch == "^" and self.peek(1) == "^":
                self.handle_left_superscript()
            elif ch == "^":
                self.handle_superscript()
            elif ch == "_":
                self.handle_subscript()
            elif ch == "#":
                # Closing marker - handled by each structure's method
                # If we hit a stray #, just skip it
                self.advance()
            elif ch == "\n":
                # Convert newline to linebreak in math mode
                self.advance()
                self.emit(" \\ ")
            elif ch == " ":
                # Space in VWC input = deliberate word/term break.
                # Use Typst's `space` so it's visibly wider than the thin
                # spaces between split letters (e.g. "A B C").
                self.advance()
                self.emit(" space ")
            else:
                # Regular character - check if we're at a known math function
                # name first (log, sin, cos, etc.) — those stay as one token.
                func = self._try_math_function()
                if func:
                    self.emit(func)
                    # Add space after if next char is a letter
                    if self.pos < len(self.input) and self.input[self.pos].isalpha():
                        self.emit(" ")
                else:
                    ch = self.eat()
                    self.emit(ch)

                    # After a digit or letter, split following letters with spaces
                    # e.g., "2bc" -> "2 b c", "ABC" -> "A B C"
                    # In Typst math, consecutive letters form a single variable
                    # name, but in VWC each letter is always a separate symbol.
                    if (
                        (ch.isdigit() or ch.isalpha())
                        and self.peek()
                        and self.peek().isalpha()
                    ):
                        while self.peek() and self.peek().isalpha():
                            # Stop splitting if next char is a Greek prefix (μ/µ),
                            # a math function name, or a $ (dollar code).
                            if self.peek() in "μµ":
                                break
                            if self._peek_math_function():
                                break
                            next_letter = self.eat()
                            self.emit(" " + next_letter)

        return "".join(self.output)

    # ------------------------------------------------------------------
    # Handlers
    # ------------------------------------------------------------------

    def handle_greek(self):
        """μ followed by one character -> Greek letter in Typst."""
        self.advance()  # skip the μ
        if self.pos < len(self.input):
            next_char = self.eat()
            typst_name = GREEK.get(next_char, f"μ{next_char}")
            # Add space before if previous output ends with alphanumeric
            if self.output:
                last = self.output[-1]
                if last and last[-1].isalnum():
                    self.emit(" ")
            self.emit(typst_name)
            # Add space after if next char is alphanumeric
            if self.pos < len(self.input) and self.input[self.pos].isalnum():
                self.emit(" ")
        else:
            self.emit("μ")

    def handle_dollar(self):
        """$ is the entry key for special symbols and structures."""
        self.advance()  # skip the $

        # Check structural forms first (these need special parsing)
        if self.starts_with("br"):
            self.handle_fraction()
            return
        if self.starts_with("wt"):
            self.handle_root()
            return
        if self.starts_with("st"):
            self.handle_system()
            return
        if self.starts_with("mat"):
            self.handle_matrix()
            return
        if self.starts_with("det"):
            self.handle_determinant()
            return
        if self.starts_with("com"):
            self.handle_combination()
            return
        if self.starts_with("#"):
            # $# is end of a vormschema block
            self.advance()
            return

        # Check line forms (accents)
        for code in LINE_FORMS_SORTED:
            if self.starts_with(code):
                self.advance(len(code))
                self.handle_line_form(LINE_FORMS[code])
                return

        # Check generic dollar codes (longest match first)
        for code in DOLLAR_CODES_SORTED:
            if self.starts_with(code):
                self.advance(len(code))
                symbol = DOLLAR_CODES[code]
                # Ensure spacing around symbols so Typst doesn't merge tokens.
                # Add space before if previous output ends with an alphanumeric char.
                if self.output:
                    last = self.output[-1]
                    if last and last[-1].isalnum():
                        self.emit(" ")
                self.emit(symbol)
                # Add space after if next char is alphanumeric (not a space,
                # operator, or end of input).
                if self.pos < len(self.input) and self.input[self.pos].isalnum():
                    self.emit(" ")
                return

        # Unknown dollar code - keep the $ and move on
        self.emit("$")

    def handle_fraction(self):
        """
        $br marks a complex fraction.
        Everything up to / is the numerator, everything after / until # is denominator.
        """
        self.advance(2)  # skip "br"

        if self.peek() == " ":
            self.advance()

        numerator = self.collect_until_fraction_slash()
        denominator = self.collect_until_hash(track_sub_super=True)

        num_typst = VWCConverter(numerator).convert()
        den_typst = VWCConverter(denominator).convert()

        self.emit(f"frac({num_typst}, {den_typst})")

    def handle_root(self):
        """$wt starts a square root. Content until #."""
        self.advance(2)  # skip "wt"

        if self.peek() == " ":
            self.advance()

        radicand = self.collect_until_hash_or_single()
        radicand_typst = VWCConverter(radicand).convert()

        self.emit(f"sqrt({radicand_typst})")

    def handle_left_superscript(self):
        """
        ^^ starts a left superscript.
        Used for nth root index or log base.
        """
        self.advance(2)  # skip "^^"

        index = self.collect_until_hash()
        index_typst = VWCConverter(index).convert()

        if self.starts_with("$wt"):
            self.advance(3)
            if self.peek() == " ":
                self.advance()
            radicand = self.collect_until_hash_or_single()
            radicand_typst = VWCConverter(radicand).convert()
            self.emit(f"root({index_typst}, {radicand_typst})")
        elif self.starts_with("log"):
            self.advance(3)
            self.emit(f"log_{index_typst}")
        else:
            self.emit(f"attach(tl: {index_typst})")

    def handle_superscript(self):
        """^ followed by content, optionally closed by #."""
        self.advance()
        content = self.collect_sub_or_super()
        self.emit(f"^({content})" if len(content) > 1 else f"^{content}")
        # Add space after if next char is a letter to prevent token merging
        if self.pos < len(self.input) and self.input[self.pos].isalpha():
            self.emit(" ")

    def handle_subscript(self):
        """_ followed by content, optionally closed by #."""
        self.advance()
        content = self.collect_sub_or_super()
        self.emit(f"_({content})" if len(content) > 1 else f"_{content}")
        # Add space after if next char is a letter to prevent token merging
        if self.pos < len(self.input) and self.input[self.pos].isalpha():
            self.emit(" ")

    def handle_system(self):
        """$stN followed by N lines, ended by $#."""
        self.advance(2)  # skip "st"

        num_str = ""
        while self.peek().isdigit():
            num_str += self.eat()
        num_eqs = int(num_str) if num_str else 2

        content = self.collect_until_dollar_hash()
        lines = [line.strip() for line in content.strip().split("\n") if line.strip()]
        converted_lines = [VWCConverter(line).convert() for line in lines[:num_eqs]]

        self.emit("cases(" + ", ".join(converted_lines) + ")")

    def handle_matrix(self):
        """$matR.C with rows separated by newlines, columns by two spaces."""
        self.advance(3)  # skip "mat"

        dims = ""
        while self.peek() and self.peek() in "0123456789.x":
            dims += self.eat()

        content = self.collect_until_dollar_hash()
        rows = [row.strip() for row in content.strip().split("\n") if row.strip()]

        typst_rows = []
        for row in rows:
            cols = [c.strip() for c in row.replace("  ", ",").split(",") if c.strip()]
            converted_cols = [VWCConverter(c).convert() for c in cols]
            typst_rows.append(", ".join(converted_cols))

        self.emit("mat(" + "; ".join(typst_rows) + ")")

    def handle_determinant(self):
        """Same as matrix but with | delimiters."""
        self.advance(3)  # skip "det"

        dims = ""
        while self.peek() and self.peek() in "0123456789.x":
            dims += self.eat()

        content = self.collect_until_dollar_hash()
        rows = [row.strip() for row in content.strip().split("\n") if row.strip()]

        typst_rows = []
        for row in rows:
            cols = [c.strip() for c in row.replace("  ", ",").split(",") if c.strip()]
            converted_cols = [VWCConverter(c).convert() for c in cols]
            typst_rows.append(", ".join(converted_cols))

        self.emit('mat(delim: "|", ' + "; ".join(typst_rows) + ")")

    def handle_combination(self):
        """$com followed by two lines (n and p), ended by $#."""
        self.advance(3)  # skip "com"

        content = self.collect_until_dollar_hash()
        lines = [line.strip() for line in content.strip().split("\n") if line.strip()]

        if len(lines) >= 2:
            n = VWCConverter(lines[0]).convert()
            p = VWCConverter(lines[1]).convert()
            self.emit(f"binom({n}, {p})")
        elif len(lines) == 1:
            self.emit(f"binom({VWCConverter(lines[0]).convert()}, )")

    def handle_line_form(self, typst_accent: str):
        """Line form: $XYZ content# -> accent or overline."""
        if self.peek() == " ":
            self.advance()

        content = self.collect_until_hash()
        content_typst = VWCConverter(content).convert()

        if typst_accent == "overline":
            self.emit(f"overline({content_typst})")
        else:
            self.emit(f"accent({content_typst}, {typst_accent})")

    def handle_spatievuller(self):
        """The ¨ character is a spacing filler. Convert to a space."""
        self.advance()
        self.emit(" ")

    # ------------------------------------------------------------------
    # Collection utilities
    # ------------------------------------------------------------------

    @staticmethod
    def _sub_super_has_hash(text: str, pos: int) -> bool:
        """Check if a ^ or _ at `pos` in `text` has an argument closed by #.

        Scans forward from pos+1 looking for a # at nesting depth 0.
        Returns True if found before another ^/_ at depth 0 or end of text.
        Tracks $br and $wt as depth-incrementing structures.

        Single-char superscripts (^x followed by a non-alnum boundary) are
        never considered to have a # closer — they grab one char and stop.
        """
        # Count the number of "content tokens" between ^ and the first # at
        # depth 0.  A single alphanumeric char that is the ONLY token before #
        # means this is a single-char sub/super — it doesn't consume a #.
        # Multiple tokens mean it IS a #-terminated argument.
        #
        # Special case: ^x¨ (single char + spatievuller) — if the spatievuller
        # is followed by a new term start (letter, digit, Greek prefix, $),
        # then ^x is single-char and ¨ starts new content.  If followed by
        # an operator (=, +, -, <, >, etc.), the spatievuller is a space
        # inside the sub/super argument (e.g. _k¨=1#).
        first = pos + 1
        if first < len(text):
            ch1 = text[first]
            if ch1.isalnum():
                second = first + 1
                if second >= len(text):
                    return False  # single char at end of text
                ch2 = text[second]
                if not ch2.isalnum() and ch2 not in "¨μµ#":
                    return False  # ^x followed by operator/boundary (not #)
                if ch2 == "¨":
                    # ^x¨ — check what follows the spatievuller
                    third = second + 1
                    if third >= len(text):
                        return False
                    ch3 = text[third]
                    # If next is an operator (=, +, -, etc.), the ¨ is inside
                    # the argument: _k¨=1# means _(k=1)
                    # If next is a term start (letter, digit, μ, $), the ¨
                    # is a separator: ^2¨μa means ^2 then space then μa
                    if ch3.isalnum() or ch3 in "μµ$":
                        return False  # new term — single-char super

        scan = pos + 1
        depth = 0
        while scan < len(text):
            chunk = text[scan:]
            if chunk.startswith("$br") or chunk.startswith("$wt"):
                depth += 1
                scan += 3
                continue
            ch = text[scan]
            if ch == "#":
                if depth == 0:
                    return True
                depth -= 1
            elif ch in "^_" and depth == 0 and scan > pos + 1:
                return False
            scan += 1
        return False

    def collect_until_fraction_slash(self) -> str:
        """Collect until / at current nesting level (depth tracked via $br/$wt/#).

        At depth 0, we look for // (double slash) which separates outer fraction.
        At depth > 0, we look for / (single slash) which separates inner fraction.

        Also tracks ^/_ with # closers as depth increments so that
        superscript/subscript # closers don't confuse the depth count.
        """
        depth = 0
        collected = []

        while self.pos < len(self.input):
            if self.starts_with("$br") or self.starts_with("$wt"):
                depth += 1
                collected.append(self.eat(3))
                continue

            ch = self.peek()

            # Track multi-char superscripts/subscripts that consume a #
            if (ch == "^" or ch == "_") and self._sub_super_has_hash(
                self.input, self.pos
            ):
                depth += 1
                collected.append(self.eat())
                continue

            if ch == "#" and depth > 0:
                depth -= 1
                collected.append(self.eat())
                continue

            # Check for fraction slash:
            # At depth 0: // means outer nested fraction, / means simple fraction
            # At depth > 0: don't return here - let inner fraction handle its own /
            if depth == 0:
                # At top level, check for // first (outer nested fraction separator)
                if self.starts_with("//"):
                    self.advance(2)  # Skip both slashes
                    return "".join(collected)
                # Single / at depth 0 is simple fraction separator
                elif self.peek() == "/":
                    self.advance()  # Skip single slash
                    return "".join(collected)
            # Note: when depth > 0, we DON'T return on /
            # The inner fraction's handle_fraction() will deal with its own /
            # We just keep collecting until we hit // at depth 0

            collected.append(self.eat())

        return "".join(collected)

    def collect_until_hash(self, track_sub_super: bool = False) -> str:
        """Collect until # at current nesting level.

        Args:
            track_sub_super: If True, track ^/_ with # closers as depth
                increments.  Needed when the enclosing structure (like a
                fraction denominator) contains ^/_ that consume their own #.
        """
        depth = 0
        collected = []

        while self.pos < len(self.input):
            if self.starts_with("$br") or self.starts_with("$wt"):
                depth += 1
                collected.append(self.eat(3))
                continue

            ch = self.peek()
            # Multi-char superscripts/subscripts consume a # to close
            if (
                track_sub_super
                and (ch == "^" or ch == "_")
                and self._sub_super_has_hash(self.input, self.pos)
            ):
                depth += 1
                collected.append(self.eat())
                continue

            if ch == "#":
                if depth == 0:
                    self.advance()
                    return "".join(collected)
                else:
                    depth -= 1
                    collected.append(self.eat())
                    continue

            collected.append(self.eat())

        return "".join(collected)

    def collect_until_hash_or_single(self) -> str:
        """
        For roots: if single token (no # found soon), grab that.
        Otherwise collect until #.

        Tracks $br, $wt, and ^/_ with # closers for depth so that
        the root's own # closer is correctly identified.
        """
        search_pos = self.pos
        depth = 0
        found_hash = False

        while search_pos < len(self.input):
            chunk = self.input[search_pos:]
            if chunk.startswith("$br") or chunk.startswith("$wt"):
                depth += 1
                search_pos += 3
                continue
            ch = self.input[search_pos]
            if (ch == "^" or ch == "_") and self._sub_super_has_hash(
                self.input, search_pos
            ):
                depth += 1
                search_pos += 1
                continue
            if ch == "#":
                if depth == 0:
                    found_hash = True
                    break
                depth -= 1
            search_pos += 1

        if found_hash:
            return self.collect_until_hash(track_sub_super=True)
        else:
            # No closing # found — grab everything remaining
            remaining = self.input[self.pos :]
            self.pos = len(self.input)
            return remaining

    def collect_sub_or_super(self) -> str:
        """Collect subscript/superscript content."""
        search_pos = self.pos
        depth = 0
        has_closing_hash = False

        while search_pos < len(self.input):
            chunk = self.input[search_pos:]
            if chunk.startswith("$br") or chunk.startswith("$wt"):
                depth += 1
                search_pos += 3
                continue
            ch = self.input[search_pos]
            if ch == "#":
                if depth == 0:
                    has_closing_hash = True
                    break
                depth -= 1
            if ch in "^_" and depth == 0 and search_pos > self.pos:
                break
            search_pos += 1

        if has_closing_hash and search_pos > self.pos:
            collected = []
            depth = 0
            while self.pos < len(self.input):
                if self.peek() == "#" and depth == 0:
                    self.advance()
                    break
                if self.starts_with("$br") or self.starts_with("$wt"):
                    depth += 1
                    collected.append(self.eat(3))
                    continue
                if self.peek() == "#" and depth > 0:
                    depth -= 1
                    collected.append(self.eat())
                    continue
                collected.append(self.eat())

            inner = "".join(collected)
            return VWCConverter(inner).convert()
        else:
            if self.pos < len(self.input) and self.peek().isalnum():
                return self.eat()
            elif self.pos < len(self.input):
                return self.eat()
            return ""

    def collect_until_dollar_hash(self) -> str:
        """Collect everything until $# (end of vormschema)."""
        collected = []
        while self.pos < len(self.input):
            if self.starts_with("$#"):
                self.advance(2)
                return "".join(collected)
            collected.append(self.eat())
        return "".join(collected)


def vwc_to_typst(vwc_input: str, display: bool = False) -> str:
    """
    Convert VWC input to Typst output.

    Args:
        vwc_input: The VWC string to convert
        display: If True, wrap in display math ($ ... $)

    Returns:
        Typst string
    """
    converter = VWCConverter(vwc_input)
    typst_body = converter.convert()

    if display:
        return f"$ {typst_body} $"
    else:
        return f"${typst_body}$"
