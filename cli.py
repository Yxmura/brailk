#!/usr/bin/env python3
"""
CLI for VWC to Typst converter.

Usage:
    python cli.py "your VWC code here"
    python cli.py -f input.vwc
    python cli.py --interactive
"""

import argparse
import sys
from vwc_converter import vwc_to_typst


def main():
    parser = argparse.ArgumentParser(
        description="Convert VWC (Vlaamse Wiskundecode) to Typst"
    )
    parser.add_argument(
        "input", nargs="?", help="VWC code to convert (or use -f for file input)"
    )
    parser.add_argument("-f", "--file", help="Read VWC from file")
    parser.add_argument(
        "-d",
        "--display",
        action="store_true",
        help="Use display math mode (adds spaces around content)",
    )
    parser.add_argument(
        "-i",
        "--interactive",
        action="store_true",
        help="Interactive mode (type VWC, get Typst)",
    )
    parser.add_argument(
        "--no-wrap", action="store_true", help="Output raw Typst without $...$ wrapper"
    )

    args = parser.parse_args()

    if args.interactive:
        print("VWC to Typst Converter - Interactive Mode")
        print("Type 'quit' or press Ctrl+D to exit")
        print("-" * 50)

        while True:
            try:
                vwc = input("\nVWC > ")
                if vwc.lower() in ("quit", "exit", "q"):
                    break

                result = vwc_to_typst(vwc, display=args.display)

                if args.no_wrap:
                    # Remove $...$ wrapper
                    result = result.strip("$")

                print(f"Typst: {result}")

            except EOFError:
                print()
                break
            except KeyboardInterrupt:
                print()
                break

    elif args.file:
        try:
            with open(args.file, "r", encoding="utf-8") as f:
                vwc = f.read()

            result = vwc_to_typst(vwc, display=args.display)

            if args.no_wrap:
                result = result.strip("$")

            print(result)

        except FileNotFoundError:
            print(f"Error: File not found: {args.file}", file=sys.stderr)
            sys.exit(1)
        except Exception as e:
            print(f"Error: {e}", file=sys.stderr)
            sys.exit(1)

    elif args.input:
        result = vwc_to_typst(args.input, display=args.display)

        if args.no_wrap:
            result = result.strip("$")

        print(result)

    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
