#!/usr/bin/env python3
"""
OCR Adapter sidecar for QAIVRA.

Extracts text regions from images using Tesseract OCR.
Returns JSON with text, bounding boxes (normalized 0..1), and confidence.

Usage: python ocr_adapter.py <image_path>
Output: JSON to stdout
"""

import json
import sys
from pathlib import Path

# Ensure UTF-8 stdout on Windows (cp1252 cannot encode CJK/emoji/Indonesian chars)
if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')

try:
    import pytesseract
    from PIL import Image
except ImportError:
    print(json.dumps({
        "success": False,
        "regions": [],
        "warnings": ["pytesseract or Pillow not installed"],
        "error": "Missing dependencies: install pytesseract and Pillow"
    }))
    sys.exit(0)


def run_ocr(image_path: str) -> dict:
    """Run OCR on an image and return structured regions."""
    try:
        img = Image.open(image_path)
    except Exception as e:
        return {
            "success": False,
            "regions": [],
            "warnings": [],
            "error": f"Failed to open image: {e}"
        }

    width, height = img.size
    if width == 0 or height == 0:
        return {
            "success": False,
            "regions": [],
            "warnings": [],
            "error": "Image has zero dimensions"
        }

    try:
        data = pytesseract.image_to_data(img, output_type=pytesseract.Output.DICT)
    except Exception as e:
        return {
            "success": False,
            "regions": [],
            "warnings": [],
            "error": f"Tesseract failed: {e}"
        }

    regions = []
    n = len(data["text"])
    for i in range(n):
        text = data["text"][i].strip()
        if not text:
            continue
        conf = float(data["conf"][i])
        if conf < 0:
            continue

        x = data["left"][i] / width
        y = data["top"][i] / height
        w = data["width"][i] / width
        h = data["height"][i] / height

        # Clamp to [0, 1]
        x = max(0.0, min(1.0, x))
        y = max(0.0, min(1.0, y))
        w = max(0.0, min(1.0 - x, w))
        h = max(0.0, min(1.0 - y, h))

        regions.append({
            "text": text,
            "confidence": conf / 100.0,  # Normalize to 0..1
            "bbox_x": round(x, 6),
            "bbox_y": round(y, 6),
            "bbox_width": round(w, 6),
            "bbox_height": round(h, 6),
        })

    warnings = []
    if not regions:
        warnings.append("No text regions detected in image")

    return {
        "success": True,
        "regions": regions,
        "warnings": warnings,
        "error": None
    }


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(json.dumps({
            "success": False,
            "regions": [],
            "warnings": [],
            "error": "Usage: ocr_adapter.py <image_path>"
        }))
        sys.exit(1)

    result = run_ocr(sys.argv[1])
    print(json.dumps(result))
