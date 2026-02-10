"""
Test suite for VWC to Typst converter.
"""

from vwc_converter import vwc_to_typst

TEST_CASES = [
    # Greek letters
    ("μa", "alpha", "Greek alpha"),
    ("μb+μg", "beta", "Greek beta and gamma"),
    ("μA", "Alpha", "Greek uppercase"),
    # Basic operations
    ("x^2", "x^2", "Simple superscript"),
    ("a_i", "a_i", "Simple subscript"),
    ("a_n-1#", "a_(n-1)", "Complex subscript with hash"),
    ("x_i#^j#", "x_i^j", "Subscript and superscript"),
    # Roots
    ("$wt 9", "sqrt(9)", "Square root"),
    ("$wt 3x+2#", "sqrt(3 x+2)", "Square root with expression"),
    ("^^3#$wt 8", "root(3, 8)", "Cube root"),
    # Fractions
    ("$br 2x+3/y#", "frac(2 x+3, y)", "Simple fraction"),
    ("$br 24+35/12*8#", "frac(24+35, 12*8)", "Fraction with operations"),
    # Dollar codes
    ("$N", "NN", "Natural numbers"),
    ("$Z", "ZZ", "Integers"),
    ("$R", "RR", "Reals"),
    ("$elm", "in", "Element of"),
    ("$inf", "infinity", "Infinity"),
    ("$int", "integral", "Integral"),
    ("$<=", "lt.eq", "Less than or equal"),
    ("$>=", "gt.eq", "Greater than or equal"),
    ("$nt=", "eq.not", "Not equal"),
    # Systems
    ("$st2\n2x+3y=14\nx-2y=0$#", "cases(", "System of equations"),
    # Matrices
    ("$mat2.2\n1  2\n3  4$#", "mat(", "2x2 matrix"),
    # Determinants
    ("$det2.2\n1  2\n3  4$#", 'mat(delim: "|"', "Determinant"),
    # Combinations
    ("$com\nn\np$#", "binom(", "Binomial coefficient"),
    # Line forms
    ("$bgb AB#", "overline(A B)", "Overline accent"),
    ("$hkb x#", "accent(x, hat)", "Hat accent"),
    # Sets and logic
    ("$A x $elm $R", "forall space x space in space RR", "For all x in R"),
    ("$E", "exists", "Exists"),
    ("$O", "nothing", "Empty set"),
]


def run_tests(verbose=False):
    """Run all test cases and return results."""
    passed = 0
    failed = 0
    failures = []

    print(f"Running {len(TEST_CASES)} tests...")
    print("-" * 60)

    for vwc_input, expected, description in TEST_CASES:
        result = vwc_to_typst(vwc_input)
        ok = expected in result

        if ok:
            passed += 1
            status = "OK"
        else:
            failed += 1
            status = "FAIL"
            failures.append((vwc_input, expected, result, description))

        if verbose or not ok:
            print(f"  [{status}] {description}")
            if not ok:
                print(f"       Input:    {repr(vwc_input)}")
                print(f"       Expected: {repr(expected)} in output")
                print(f"       Got:      {repr(result)}")

    print("-" * 60)
    print(f"  {passed} passed, {failed} failed out of {len(TEST_CASES)}")

    return passed, failed, failures


def run_complex_tests():
    """Run more complex test cases."""
    print("\nComplex test cases:")
    print("-" * 60)

    complex_tests = [
        (r"μp$=3,14", "pi", "Pi approximation"),
        ("$int_a#^b#f(x)dx", "integral_a^b", "Integral with bounds"),
        ("$br $br a/b+c#//d/e+f#", "$frac(frac(a, b+c), d/e+f)$", "Nested fractions"),
        ("^^n#$wt x^2+1#", "root(n, x^2+1)", "Nth root of expression"),
    ]

    passed = 0
    for vwc_input, expected, description in complex_tests:
        result = vwc_to_typst(vwc_input)
        ok = expected in result
        status = "OK" if ok else "FAIL"

        if ok:
            passed += 1

        print(f"  [{status}] {description}")
        print(f"       {repr(vwc_input)} -> {result}")

    print(f"\n{passed}/{len(complex_tests)} complex tests passed")


def run_the_beast_test():
    """no running from this one"""
    print("\nRunning the beast test:")
    print("-" * 60)

    # Corrected VWC input using proper characters:
    # - \xa8 = ¨ (U+00A8, spatievuller, NOT combining diaeresis U+0308)
    # - \xb5 = µ (U+00B5, micro sign, works same as Greek mu U+03BC)
    # - $wt for roots (NOT #wt)
    # - $d for partial derivative (NOT μD)
    # - μS = Sigma (NOT sum)
    # - i is just the letter i (NOT μi which would be iota)
    beast_vwc = (
        "$int_0#^µp/2#$br $br sin\xa8µa/$wt 1-cos^2\xa8µa## + "
        "$br e^iµl#/µS_k\xa8=1#^n# $br µa_k^2/$wt k####"
        "//1/2# = $br 1/2# $br $br $wt 5#+1//2#^$nt=2# - "
        "$br $d_x#^2#f/$d_y#^2#g#//1# $asa $mat3.3\n"
        "1  0  0\n"
        "0  µp/4#  0\n"
        "0  0  1$# $plr $br det(A-µlI)#//det(B)#"
    )

    result = vwc_to_typst(beast_vwc)

    expected = (
        "$integral_0^(pi/2)frac(frac(sin alpha, sqrt(1-cos^2 alpha))"
        " space + space frac(e^(i lambda), Sigma_(k =1)^n space "
        "frac(alpha_k^2, sqrt(k))), 1/2) space = space frac(1, 2)"
        " space frac(frac(sqrt(5)+1, 2)^(eq.not 2) space - space "
        "frac(partial_x^2 f, partial_y^2 g), 1) space <==> space "
        "mat(1, 0, 0; 0, pi/4, 0; 0, 0, 1) space arrow.r space "
        "frac(det(A-lambda I), det(B))$"
    )

    ok = result == expected
    print("".join(beast_vwc))
    if not ok:
        print(f"\nExpected:\n{expected}")
    print("THE BEAST has passed" if ok else "Failed THE BEAST")


if __name__ == "__main__":
    import sys

    verbose = "-v" in sys.argv or "--verbose" in sys.argv
    passed, failed, _ = run_tests(verbose=verbose)
    run_complex_tests()
    run_the_beast_test()

    sys.exit(0 if failed == 0 else 1)
