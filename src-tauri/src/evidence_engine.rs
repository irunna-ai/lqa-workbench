//! QAIVRA Phase 6 - Visual Evidence Engine (pure domain + validation layer).
//!
//! This module is intentionally free of SQLite / Tauri dependencies so the
//! coordinate-validation and proposal logic can be unit-tested deterministically.
//!
//! Authority model: AI / OCR may *propose* an evidence region, but the reviewer
//! is the final authority. The engine never invents a decorative box when no
//! trustworthy grounding exists - it returns an empty proposal set instead.

use serde::{Deserialize, Serialize};

/// Smallest allowed normalized region edge (1% of the image).
pub const MIN_REGION_SIZE: f64 = 0.01;
/// Deterministic OCR-derived padding (1.5% of the image on each side).
pub const OCR_PADDING: f64 = 0.015;
/// Floating-point tolerance for bound checks.
const EPS: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationType {
    RedBox,
    RedBracket,
}

impl AnnotationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnnotationType::RedBox => "RED_BOX",
            AnnotationType::RedBracket => "RED_BRACKET",
        }
    }

    pub fn from_str(s: &str) -> Result<AnnotationType, String> {
        match s {
            "RED_BOX" => Ok(AnnotationType::RedBox),
            "RED_BRACKET" => Ok(AnnotationType::RedBracket),
            other => Err(format!("Invalid annotation_type: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateSpace {
    Normalized,
}

impl CoordinateSpace {
    pub fn as_str(&self) -> &'static str {
        match self {
            CoordinateSpace::Normalized => "NORMALIZED",
        }
    }

    pub fn from_str(s: &str) -> Result<CoordinateSpace, String> {
        match s {
            "NORMALIZED" => Ok(CoordinateSpace::Normalized),
            other => Err(format!("Invalid coordinate_space: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceOrigin {
    AiProposed,
    OcrDerived,
    UserCreated,
    UserEdited,
}

impl EvidenceOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            EvidenceOrigin::AiProposed => "AI_PROPOSED",
            EvidenceOrigin::OcrDerived => "OCR_DERIVED",
            EvidenceOrigin::UserCreated => "USER_CREATED",
            EvidenceOrigin::UserEdited => "USER_EDITED",
        }
    }

    pub fn from_str(s: &str) -> Result<EvidenceOrigin, String> {
        match s {
            "AI_PROPOSED" => Ok(EvidenceOrigin::AiProposed),
            "OCR_DERIVED" => Ok(EvidenceOrigin::OcrDerived),
            "USER_CREATED" => Ok(EvidenceOrigin::UserCreated),
            "USER_EDITED" => Ok(EvidenceOrigin::UserEdited),
            other => Err(format!("Invalid evidence origin: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewState {
    Proposed,
    Reviewed,
    Approved,
    Rejected,
}

impl ReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewState::Proposed => "PROPOSED",
            ReviewState::Reviewed => "REVIEWED",
            ReviewState::Approved => "APPROVED",
            ReviewState::Rejected => "REJECTED",
        }
    }

    pub fn from_str(s: &str) -> Result<ReviewState, String> {
        match s {
            "PROPOSED" => Ok(ReviewState::Proposed),
            "REVIEWED" => Ok(ReviewState::Reviewed),
            "APPROVED" => Ok(ReviewState::Approved),
            "REJECTED" => Ok(ReviewState::Rejected),
            other => Err(format!("Invalid review_state: '{}'", other)),
        }
    }
}

/// A normalized image region. All components are in [0, 1].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedRegion {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
// ---------------------------------------------------------------------------
// Coordinate validation engine (backend-authoritative)
// ---------------------------------------------------------------------------

/// Validate a normalized region. Rejects NaN, infinity, negatives, zero area,
/// and overflow beyond the unit square.
pub fn validate_normalized_region(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<NormalizedRegion, String> {
    if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() {
        return Err("Evidence coordinates must be finite (no NaN or infinity).".to_string());
    }
    if x < -EPS || y < -EPS {
        return Err("Evidence coordinates must be non-negative.".to_string());
    }
    if width <= 0.0 || height <= 0.0 {
        return Err("Evidence region must have positive width and height.".to_string());
    }
    if x + width > 1.0 + EPS {
        return Err("Evidence region overflows the image horizontally.".to_string());
    }
    if y + height > 1.0 + EPS {
        return Err("Evidence region overflows the image vertically.".to_string());
    }
    Ok(NormalizedRegion {
        x,
        y,
        width,
        height,
    })
}

/// Clamp a region to the unit square. Used only for explicitly allowed
/// proposal normalization, never to silently accept invalid user input.
pub fn clamp_region(region: &NormalizedRegion) -> NormalizedRegion {
    let x = x_clamp(region.x);
    let y = x_clamp(region.y);
    let max_w = (1.0 - x).max(0.0);
    let max_h = (1.0 - y).max(0.0);
    let width = region.width.clamp(0.0, max_w);
    let height = region.height.clamp(0.0, max_h);
    NormalizedRegion {
        x,
        y,
        width,
        height,
    }
}

fn x_clamp(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    v.clamp(0.0, 1.0)
}

/// Convert pixel geometry to a normalized region. `img_w`/`img_h` must be > 0.
pub fn pixel_to_normalized(
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
    img_w: f64,
    img_h: f64,
) -> Result<NormalizedRegion, String> {
    if img_w <= 0.0 || img_h <= 0.0 {
        return Err("Image dimensions must be positive for pixel conversion.".to_string());
    }
    validate_normalized_region(px / img_w, py / img_h, pw / img_w, ph / img_h)
}

/// Convert a normalized region back to pixel geometry for rendering.
pub fn normalized_to_pixel(
    region: &NormalizedRegion,
    img_w: f64,
    img_h: f64,
) -> (f64, f64, f64, f64) {
    (
        region.x * img_w,
        region.y * img_h,
        region.width * img_w,
        region.height * img_h,
    )
}

// ---------------------------------------------------------------------------
// Evidence proposal engine (deterministic, no invented boxes)
// ---------------------------------------------------------------------------

/// Lightweight issue view consumed by the proposal engine.
#[derive(Debug, Clone)]
pub struct IssueLite {
    pub detected_text: String,
    pub title: String,
    pub coord_system: String,
    pub bbox_x: f64,
    pub bbox_y: f64,
    pub bbox_w: f64,
    pub bbox_h: f64,
    pub confidence: f64,
}

/// Lightweight OCR region view (already normalized) consumed by the engine.
#[derive(Debug, Clone)]
pub struct OcrLite {
    pub text_normalized: String,
    pub bbox_x: f64,
    pub bbox_y: f64,
    pub bbox_width: f64,
    pub bbox_height: f64,
    pub ocr_confidence: f64,
}

/// A candidate evidence region proposed (not yet persisted) by the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCandidate {
    pub annotation_type: String,
    pub coordinate_space: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub origin: String,
    pub confidence: Option<f64>,
}

const PROPOSED_TYPE: &str = "RED_BOX";

/// Generate evidence candidates for an issue.
///
/// Priority:
/// 1. Ground to OCR geometry when the issue text matches OCR regions -> `OCR_DERIVED`.
/// 2. Otherwise, if the issue already carries a valid normalized bbox -> `AI_PROPOSED`.
/// 3. Otherwise, return an empty list (no trustworthy region - do NOT invent a box).
pub fn generate_proposals(issue: &IssueLite, ocr_regions: &[OcrLite]) -> Vec<EvidenceCandidate> {
    // 1) OCR grounding (preferred).
    let matched = match_ocr_regions(issue, ocr_regions);
    if let Some(merged) = merge_regions(&matched) {
        let padded = pad_region(merged);
        if validate_normalized_region(padded.x, padded.y, padded.width, padded.height).is_ok() {
            return vec![EvidenceCandidate {
                annotation_type: PROPOSED_TYPE.to_string(),
                coordinate_space: "NORMALIZED".to_string(),
                x: padded.x,
                y: padded.y,
                width: padded.width,
                height: padded.height,
                origin: EvidenceOrigin::OcrDerived.as_str().to_string(),
                confidence: average_confidence(&matched),
            }];
        }
    }

    // 2) Model-provided normalized bbox.
    if issue.coord_system == "normalized"
        && validate_normalized_region(issue.bbox_x, issue.bbox_y, issue.bbox_w, issue.bbox_h)
            .is_ok()
    {
        return vec![EvidenceCandidate {
            annotation_type: PROPOSED_TYPE.to_string(),
            coordinate_space: "NORMALIZED".to_string(),
            x: issue.bbox_x,
            y: issue.bbox_y,
            width: issue.bbox_w,
            height: issue.bbox_h,
            origin: EvidenceOrigin::AiProposed.as_str().to_string(),
            confidence: Some(issue.confidence),
        }];
    }

    // 3) No trustworthy region.
    vec![]
}

fn match_ocr_regions(issue: &IssueLite, ocr_regions: &[OcrLite]) -> Vec<OcrLite> {
    let query = normalize_text(&format!("{} {}", issue.detected_text, issue.title));
    let tokens: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .map(|t| t.to_string())
        .collect();
    if tokens.is_empty() {
        return vec![];
    }
    let mut matched = Vec::new();
    for r in ocr_regions {
        let rt = normalize_text(&r.text_normalized);
        if rt.is_empty() || rt.len() < 2 {
            continue;
        }
        let hit = tokens
            .iter()
            .any(|t| rt.contains(t.as_str()) || t.contains(&rt));
        if hit {
            matched.push(r.clone());
        }
    }
    matched
}

fn merge_regions(regions: &[OcrLite]) -> Option<NormalizedRegion> {
    if regions.is_empty() {
        return None;
    }
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for r in regions {
        min_x = min_x.min(r.bbox_x);
        min_y = min_y.min(r.bbox_y);
        max_x = max_x.max(r.bbox_x + r.bbox_width);
        max_y = max_y.max(r.bbox_y + r.bbox_height);
    }
    Some(NormalizedRegion {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(0.0),
        height: (max_y - min_y).max(0.0),
    })
}

fn pad_region(region: NormalizedRegion) -> NormalizedRegion {
    let x = (region.x - OCR_PADDING).max(0.0);
    let y = (region.y - OCR_PADDING).max(0.0);
    let width = (region.width + 2.0 * OCR_PADDING).min(1.0 - x);
    let height = (region.height + 2.0 * OCR_PADDING).min(1.0 - y);
    NormalizedRegion {
        x,
        y,
        width,
        height,
    }
}

fn average_confidence(regions: &[OcrLite]) -> Option<f64> {
    if regions.is_empty() {
        return None;
    }
    let sum: f64 = regions.iter().map(|r| r.ocr_confidence).sum();
    Some(sum / regions.len() as f64)
}

fn normalize_text(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_valid_region() {
        assert!(validate_normalized_region(0.1, 0.2, 0.3, 0.4).is_ok());
    }

    #[test]
    fn rejects_negative_coordinates() {
        assert!(validate_normalized_region(-0.1, 0.2, 0.3, 0.4).is_err());
        assert!(validate_normalized_region(0.1, -0.2, 0.3, 0.4).is_err());
    }

    #[test]
    fn rejects_nan() {
        assert!(validate_normalized_region(f64::NAN, 0.2, 0.3, 0.4).is_err());
    }

    #[test]
    fn rejects_infinity() {
        assert!(validate_normalized_region(f64::INFINITY, 0.2, 0.3, 0.4).is_err());
    }

    #[test]
    fn rejects_zero_width() {
        assert!(validate_normalized_region(0.1, 0.2, 0.0, 0.4).is_err());
    }

    #[test]
    fn rejects_zero_height() {
        assert!(validate_normalized_region(0.1, 0.2, 0.3, 0.0).is_err());
    }

    #[test]
    fn rejects_x_overflow() {
        assert!(validate_normalized_region(0.8, 0.1, 0.3, 0.2).is_err());
    }

    #[test]
    fn rejects_y_overflow() {
        assert!(validate_normalized_region(0.1, 0.8, 0.2, 0.3).is_err());
    }

    #[test]
    fn pixel_to_normalized_conversion() {
        let r = pixel_to_normalized(100.0, 50.0, 200.0, 100.0, 1000.0, 500.0).unwrap();
        assert!((r.x - 0.1).abs() < 1e-12);
        assert!((r.y - 0.1).abs() < 1e-12);
        assert!((r.width - 0.2).abs() < 1e-12);
        assert!((r.height - 0.2).abs() < 1e-12);
    }

    #[test]
    fn normalized_to_pixel_conversion() {
        let r = NormalizedRegion {
            x: 0.1,
            y: 0.2,
            width: 0.3,
            height: 0.4,
        };
        let (px, py, pw, ph) = normalized_to_pixel(&r, 1000.0, 500.0);
        assert!((px - 100.0).abs() < 1e-9);
        assert!((py - 100.0).abs() < 1e-9);
        assert!((pw - 300.0).abs() < 1e-9);
        assert!((ph - 200.0).abs() < 1e-9);
    }

    #[test]
    fn clamp_keeps_region_in_bounds() {
        let r = clamp_region(&NormalizedRegion {
            x: -0.2,
            y: 0.5,
            width: 0.8,
            height: 0.8,
        });
        assert!(r.x >= 0.0);
        assert!((r.x + r.width) <= 1.0 + EPS);
        assert!((r.y + r.height) <= 1.0 + EPS);
    }

    fn ocr(text: &str, x: f64, y: f64, w: f64, h: f64) -> OcrLite {
        OcrLite {
            text_normalized: text.to_lowercase(),
            bbox_x: x,
            bbox_y: y,
            bbox_width: w,
            bbox_height: h,
            ocr_confidence: 0.9,
        }
    }

    #[test]
    fn ocr_single_region_match() {
        let issue = IssueLite {
            detected_text: "Healt".to_string(),
            title: String::new(),
            coord_system: "normalized".to_string(),
            bbox_x: 0.0,
            bbox_y: 0.0,
            bbox_w: 0.0,
            bbox_h: 0.0,
            confidence: 0.0,
        };
        let regions = vec![ocr("Healt", 0.1, 0.1, 0.2, 0.05)];
        let proposals = generate_proposals(&issue, &regions);
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].origin, "OCR_DERIVED");
        assert!(proposals[0].x < 0.1);
        assert!((proposals[0].x + proposals[0].width) <= 1.0 + EPS);
    }

    #[test]
    fn ocr_multi_word_merged_region() {
        let issue = IssueLite {
            detected_text: "save game".to_string(),
            title: String::new(),
            coord_system: "normalized".to_string(),
            bbox_x: 0.0,
            bbox_y: 0.0,
            bbox_w: 0.0,
            bbox_h: 0.0,
            confidence: 0.0,
        };
        let regions = vec![
            ocr("Save", 0.1, 0.1, 0.2, 0.05),
            ocr("Game", 0.35, 0.1, 0.2, 0.05),
        ];
        let proposals = generate_proposals(&issue, &regions);
        assert_eq!(proposals.len(), 1);
        assert!((proposals[0].x + proposals[0].width) > 0.35);
    }

    #[test]
    fn ocr_no_match_returns_no_evidence() {
        let issue = IssueLite {
            detected_text: "zzzqqq".to_string(),
            title: String::new(),
            coord_system: "normalized".to_string(),
            bbox_x: 0.0,
            bbox_y: 0.0,
            bbox_w: 0.0,
            bbox_h: 0.0,
            confidence: 0.0,
        };
        let regions = vec![ocr("Healt", 0.1, 0.1, 0.2, 0.05)];
        assert!(generate_proposals(&issue, &regions).is_empty());
    }

    #[test]
    fn deterministic_padding_stays_in_bounds() {
        let r = pad_region(NormalizedRegion {
            x: 0.0,
            y: 0.0,
            width: 0.98,
            height: 0.98,
        });
        assert!(r.x >= 0.0);
        assert!((r.x + r.width) <= 1.0 + EPS);
        assert!((r.x + r.width) >= 0.98 - 1e-6);
    }

    #[test]
    fn ai_proposed_fallback_when_no_ocr() {
        let issue = IssueLite {
            detected_text: String::new(),
            title: String::new(),
            coord_system: "normalized".to_string(),
            bbox_x: 0.2,
            bbox_y: 0.2,
            bbox_w: 0.3,
            bbox_h: 0.3,
            confidence: 0.8,
        };
        let proposals = generate_proposals(&issue, &[]);
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].origin, "AI_PROPOSED");
        assert_eq!(proposals[0].confidence, Some(0.8));
    }

    #[test]
    fn no_region_invented_without_grounding() {
        let issue = IssueLite {
            detected_text: String::new(),
            title: String::new(),
            coord_system: "pixel".to_string(),
            bbox_x: 0.0,
            bbox_y: 0.0,
            bbox_w: 0.0,
            bbox_h: 0.0,
            confidence: 0.0,
        };
        assert!(generate_proposals(&issue, &[]).is_empty());
    }

    #[test]
    fn enum_validation_rejects_unknown() {
        assert!(AnnotationType::from_str("RED_BOX").is_ok());
        assert!(AnnotationType::from_str("RED_CIRCLE").is_err());
        assert!(ReviewState::from_str("APPROVED").is_ok());
        assert!(ReviewState::from_str("DONE").is_err());
        assert!(EvidenceOrigin::from_str("USER_EDITED").is_ok());
        assert!(EvidenceOrigin::from_str("BOT").is_err());
    }
}
