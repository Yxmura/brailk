"""
Constants and lookup tables for VWC to Typst conversion.

All the mappings from VWC symbols/codes to Typst equivalents.
"""

# ============================================================================
# Greek letters: μ prefix
# ============================================================================

# Lowercase Greek letters
GREEK_LOWER = {
    "a": "alpha",
    "b": "beta",
    "g": "gamma",
    "d": "delta",
    "e": "epsilon",
    "z": "zeta",
    "ä": "eta",
    "h": "eta",  # Alternative
    "ô": "theta",
    "q": "theta",  # Alternative
    "i": "iota",
    "k": "kappa",
    "l": "lambda",
    "m": "mu",
    "n": "nu",
    "x": "xi",
    "o": "omicron",
    "p": "pi",
    "r": "rho",
    "s": "sigma",
    "t": "tau",
    "y": "upsilon",
    "f": "phi",
    "c": "chi",
    "j": "chi",  # Alternative
    "ç": "psi",
    "w": "psi",  # Alternative
    "ö": "omega",
    "v": "omega",  # Alternative
}

# Uppercase Greek letters
GREEK_UPPER = {
    "A": "Alpha",
    "B": "Beta",
    "G": "Gamma",
    "D": "Delta",
    "E": "Epsilon",
    "Z": "Zeta",
    "H": "Eta",
    "O": "Theta",
    "I": "Iota",
    "K": "Kappa",
    "L": "Lambda",
    "M": "Mu",
    "N": "Nu",
    "X": "Xi",
    "P": "Pi",
    "R": "Rho",
    "S": "Sigma",
    "T": "Tau",
    "Y": "Upsilon",
    "F": "Phi",
    "C": "Chi",
    "W": "Psi",
    "V": "Omega",
}

# Combined Greek dictionary
GREEK = {**GREEK_LOWER, **GREEK_UPPER}

# ============================================================================
# Dollar codes: $XYZ -> Typst
# ============================================================================

DOLLAR_CODES = {
    # Number sets
    "N": "NN",
    "Z": "ZZ",
    "Q": "QQ",
    "R": "RR",
    "C": "CC",
    # Operations
    "+-": "plus.minus",
    "-+": "minus.plus",
    "*": "times",
    "**": "ast",
    "/": "div",
    "na": "compose",
    # Relations
    "==": "equiv",
    "=": "approx",
    "<=": "lt.eq",
    ">=": "gt.eq",
    "!=": "eq.not",
    "<=>": "arrow.l.r.double",
    # Set theory
    "elm": "in",
    "belm": "ni",
    "dlv": "subset",
    "dlv=": "subset.eq",
    "bdlv": "supset",
    "bdlv=": "supset.eq",
    "ntelm": "in.not",
    "ntdlv": "subset.not",
    "dsn": "sect",
    "unie": "union",
    "vrs": "without",
    "O": "nothing",  # empty set
    # Logic
    "en": "and",
    "of": "or",
    "asa": "<==>",
    "pldr": "==>",
    "pldl": "arrow.l.double",
    # Arrows
    "plb": "arrow.t",
    "plo": "arrow.b",
    "pll": "arrow.l",
    "plr": "arrow.r",
    "pllr": "arrow.l.r",
    "pllb": "arrow.tl",
    "plrb": "arrow.tr",
    "plsb": "arrow.tr",
    "plso": "arrow.br",
    # Geometry
    "//": "parallel",
    "nt//": "parallel.not",
    "ldr": "perp",
    "ntldr": "perp.not",
    "slg": "tilde",
    "cgr": "tilde.equiv",
    "par": "parallelogram",
    "rhk": "square",
    "4kt": "square",
    "3hk": "triangle.stroked.t",
    "hk": "angle",
    "rh": "angle.right",
    "bol": "circle.filled",
    # Quantifiers & special
    "A": "forall",
    "E": "exists",
    "d": "partial",
    "inf": "infinity",
    "int": "integral",
    "%%": "permille",
    "krd": "hash",
    # Negations
    "nt=": "eq.not",
    "nt<": "lt.not",
    "nt>": "gt.not",
    "nt|": "divides.not",
}

# Structural codes (need special handling, not simple replacement)
STRUCTURAL_CODES = {"br", "wt", "st", "mat", "det", "com"}

# Line form codes -> Typst accent function
LINE_FORMS = {
    "bgb": "overline",
    "strb": "overline",
    "slgb": "tilde",
    "hkb": "hat",
    "plrb": "arrow",
    "pllb": "arrow.l",
}

# Sort by length (longest first) so "$<=" matches before "$<"
DOLLAR_CODES_SORTED = sorted(DOLLAR_CODES.keys(), key=len, reverse=True)
LINE_FORMS_SORTED = sorted(LINE_FORMS.keys(), key=len, reverse=True)
