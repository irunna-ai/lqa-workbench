// Phase 12: Professional LQA Report Export
use crate::db::{IssueProposal, list_issue_proposals};
use serde::Serialize;

#[derive(Serialize)]
struct ExportRow {
    issue_id: String,
    project: String,
    source_type: String,
    source_asset: String,
    bug_type: String,
    category: String,
    sub_category: String,
    severity: String,
    detected_text: String,
    description: String,
    lifecycle: String,
    confidence: f64,
    string_id: String,
}

/// Export approved/reviewable issues to a CSV file.
pub fn export_lqa_report(
    conn: &rusqlite::Connection,
    project_id: &str,
    project_name: &str,
    output_path: &str,
    filter_approved_only: bool,
) -> Result<String, String> {
    let issues = list_issue_proposals(conn, project_id, None)?;
    let report_issues: Vec<&IssueProposal> = if filter_approved_only {
        issues.iter().filter(|i| i.lifecycle == "approved").collect()
    } else {
        issues.iter().collect()
    };

    let rows: Vec<ExportRow> = report_issues.iter().map(|i| ExportRow {
        issue_id: i.id.clone(), project: project_name.to_string(),
        source_type: if i.image_id.contains("frame") { "video_frame" } else { "image" }.to_string(),
        source_asset: i.image_id.clone(), bug_type: i.bug_type.clone(),
        category: i.issue_category.clone(), sub_category: i.issue_subcategory.clone(),
        severity: i.severity_candidate.clone(), detected_text: i.detected_text.clone(),
        description: i.description.clone(), lifecycle: i.lifecycle.clone(),
        confidence: i.confidence, string_id: String::new(),
    }).collect();

    let mut wtr = csv::Writer::from_path(output_path).map_err(|e| format!("create csv: {}", e))?;
    for row in &rows {
        wtr.serialize(row).map_err(|e| format!("write row: {}", e))?;
    }
    wtr.flush().map_err(|e| format!("flush: {}", e))?;
    Ok(format!("Exported {} issues to {}", rows.len(), output_path))
}