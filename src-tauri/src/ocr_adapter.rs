/// OCR adapter for extracting text regions from images.
///
/// Uses a Python sidecar (Tesseract-based) for OCR processing.
/// Returns structured JSON with text, bounding boxes, and confidence.
///
/// Authority: OCR output is observational data, not authoritative text.
/// Low-confidence results must trigger text review states.

use crate::db::{OcrResult, OcrTextRegion};
use std::process::Command as StdCommand;

/// Run OCR on an image file using the Python sidecar.
pub fn run_ocr(image_path: &str, parser_dir: &std::path::Path) -> Result<OcrResult, String> {
    let script_path = parser_dir.join("ocr_adapter.py");
    if !script_path.exists() {
        return Err(format!("OCR script not found: {}", script_path.display()));
    }

    let output = StdCommand::new("uv")
        .env("PYTHONIOENCODING", "utf-8")
        .args(["run", "--with", "pytesseract", "--with", "Pillow",
               script_path.to_str().ok_or("Invalid script path")?,
               image_path])
        .output()
        .map_err(|e| format!("Failed to run OCR sidecar: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("OCR sidecar failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse OCR result: {}", e))
}

/// Create an empty OCR result for when OCR is unavailable.
#[allow(dead_code)]
pub fn empty_ocr_result() -> OcrResult {
    OcrResult {
        success: false,
        regions: vec![],
        warnings: vec!["OCR adapter not available".to_string()],
        error: Some("OCR sidecar not configured".to_string()),
    }
}

/// Validate OCR regions have usable confidence.
/// Used by Phase 5+ image analysis pipeline.
#[allow(dead_code)]
pub fn filter_confident_regions(regions: &[OcrTextRegion], min_confidence: f64) -> Vec<OcrTextRegion> {
    regions.iter()
        .filter(|r| r.confidence >= min_confidence && !r.text.trim().is_empty())
        .cloned()
        .collect()
}
