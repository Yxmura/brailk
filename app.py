"""
Flask web interface for VWC to Typst converter with live rendering.
"""

import base64
import os
import tempfile
from pathlib import Path

import typst
from flask import Flask, jsonify, render_template, request

from vwc_converter import vwc_to_typst

app = Flask(__name__)

# Cache directory for rendered images
CACHE_DIR = Path(tempfile.gettempdir()) / "vwc_typst_cache"
CACHE_DIR.mkdir(exist_ok=True)


def render_typst_to_image(
    typst_code: str, format: str = "svg", ppi: float = 144.0
) -> bytes:
    """
    Render Typst code to an image using the typst Python library.

    Args:
        typst_code: The Typst source code to render
        format: Output format (png or svg)
        ppi: Pixels per inch for PNG rendering

    Returns:
        Image data as bytes
    """
    # Create a temporary .typ file
    with tempfile.NamedTemporaryFile(mode="w", suffix=".typ", delete=False) as f:
        f.write(typst_code)
        temp_path = f.name

    try:
        # Compile to the requested format
        output_path = temp_path.replace(".typ", f".{format}")

        if format == "png":
            typst.compile(temp_path, output=output_path, format="png", ppi=ppi)
        elif format == "pdf":
            typst.compile(temp_path, output=output_path, format="pdf")
        else:
            typst.compile(temp_path, output=output_path, format="svg")

        # Read the result
        with open(output_path, "rb") as f:
            image_data = f.read()

        # Clean up
        os.unlink(output_path)

        return image_data

    finally:
        # Always clean up the temp file
        os.unlink(temp_path)


@app.route("/")
def index():
    """Render the main web interface."""
    return render_template("index.html")


@app.route("/convert", methods=["POST"])
def convert_endpoint():
    """
    POST /convert
    Body: { "vwc": "your VWC code", "display": false }
    Returns: { "typst": "$...$" }
    """
    data = request.get_json()
    if not data or "vwc" not in data:
        return jsonify({"error": "Missing 'vwc' field"}), 400

    try:
        typst_output = vwc_to_typst(data["vwc"], display=data.get("display", False))
        return jsonify({"typst": typst_output, "input": data["vwc"]})
    except Exception as e:
        return jsonify({"error": str(e)}), 500


@app.route("/convert-and-render", methods=["POST"])
def convert_and_render():
    """
    POST /convert-and-render
    Body: { "vwc": "your VWC code", "display": false }
    Returns: { "typst": "$...$", "image": "base64png..." }
    """
    data = request.get_json()
    if not data or "vwc" not in data:
        return jsonify({"error": "Missing 'vwc' field"}), 400

    vwc_input = data["vwc"]
    display = data.get("display", False)

    try:
        # Convert VWC to Typst
        typst_code = vwc_to_typst(vwc_input, display=display)

        # For rendering, we need to wrap inline math in a proper Typst document
        # Inline math: $...$ becomes a full document
        if not display:
            # Wrap in a document with some padding
            full_typst = f"#set page(width: auto, height: auto, margin: 1em)\n{align_and_center(typst_code)}"
        else:
            # Display math is already a block
            full_typst = f"#set page(width: auto, height: auto, margin: 1em)\n{align_and_center(typst_code)}"

        # Render to SVG (higher quality, scalable)
        image_data = render_typst_to_image(full_typst, format="svg")

        # SVG is text, so decode it directly (no base64 needed)
        svg_text = image_data.decode("utf-8")

        return jsonify(
            {
                "typst": typst_code,
                "full_typst": full_typst,  # For debugging
                "image": svg_text,
                "input": vwc_input,
            }
        )

    except Exception as e:
        import traceback

        return jsonify({"error": str(e), "traceback": traceback.format_exc()}), 500


def align_and_center(typst_code: str) -> str:
    """
    Ensure the math is properly aligned in the rendered output.
    For display math, it's already centered. For inline, we center it.
    """
    # Remove the $ wrapper for the document body
    # The typst_code comes wrapped like $...$ or $ ... $
    if typst_code.startswith("$ ") and typst_code.endswith(" $"):
        # Display math - keep as is
        return typst_code
    elif typst_code.startswith("$") and typst_code.endswith("$"):
        # Inline math - center it
        inner = typst_code[1:-1]
        return f"$ {inner} $"
    return typst_code


@app.route("/api/examples")
def examples():
    """Return example VWC codes for testing."""
    return jsonify(
        {
            "examples": [
                {"name": "Greek letters", "vwc": "μa+μb+μg"},
                {"name": "Fraction", "vwc": "$br x+1/x-1#"},
                {
                    "name": "Quadratic formula",
                    "vwc": "x=($br -b+$+-$wt b^2-4ac##)/(2a)",
                },
                {"name": "Square root", "vwc": "$wt 9"},
                {"name": "Cube root", "vwc": "^^3#$wt 8"},
                {"name": "Integral", "vwc": "$int_a#^b#f(x)dx"},
                {"name": "Matrix", "vwc": "$mat2.2\na c\nb d$#"},
                {"name": "Binomial", "vwc": "$com\nn\np$#"},
                {"name": "Set notation", "vwc": "$A x$elm$R: x^2$>=0"},
            ]
        }
    )


@app.route("/download-pdf", methods=["POST"])
def download_pdf():
    """
    POST /download-pdf
    Body: { "vwc": "your VWC code", "display": false }
    Returns: PDF file as attachment
    """
    data = request.get_json()
    if not data or "vwc" not in data:
        return jsonify({"error": "Missing 'vwc' field"}), 400

    vwc_input = data["vwc"]
    display = data.get("display", False)

    try:
        # Convert VWC to Typst
        typst_code = vwc_to_typst(vwc_input, display=display)

        # Wrap in proper Typst document
        full_typst = f"#set page(width: auto, height: auto, margin: 1em)\n{align_and_center(typst_code)}"

        # Render to PDF
        pdf_data = render_typst_to_image(full_typst, format="pdf")

        # Return as downloadable file
        import io

        from flask import send_file

        return send_file(
            io.BytesIO(pdf_data),
            mimetype="application/pdf",
            as_attachment=True,
            download_name="math.pdf",
        )

    except Exception as e:
        import traceback

        return jsonify({"error": str(e), "traceback": traceback.format_exc()}), 500


if __name__ == "__main__":
    print("Starting VWC to Typst Converter...")
    print(f"Open http://localhost:5000 in your browser")
    app.run(debug=True, host="0.0.0.0", port=5000)
