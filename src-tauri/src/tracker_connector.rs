// Phase 14: External Issue Tracker Connector Architecture
// Jira / TAPD / Mantis-Ready adapters with deterministic mock connector.
use serde::{Serialize, Deserialize};
use crate::db::IssueProposal;
use sha2::{Sha256, Digest};

/// Connector type enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectorType {
    Mock,
    Jira,
    Tapd,
    Mantis,
}

impl ConnectorType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mock" => Some(Self::Mock),
            "jira" => Some(Self::Jira),
            "tapd" => Some(Self::Tapd),
            "mantis" => Some(Self::Mantis),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self { Self::Mock => "mock", Self::Jira => "jira", Self::Tapd => "tapd", Self::Mantis => "mantis" }
    }
    pub fn available(&self) -> bool {
        matches!(self, Self::Mock) // Only mock is live; others scaffolded
    }
}

/// Tracker configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerConfig {
    pub id: String,
    pub project_id: String,
    pub connector_type: String,
    pub display_name: String,
    pub base_url: String,
    pub project_key: String,
    pub field_mapping_json: String,
    pub enabled: bool,
    pub created_at: String,
}

/// External submission record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionRecord {
    pub id: String,
    pub qaivra_issue_id: String,
    pub tracker_config_id: String,
    pub external_id: String,
    pub payload_fingerprint: String,
    pub status: String,
    pub submitted_at: String,
}

/// Field mapping from QAIVRA issue to tracker payload.
pub fn map_issue_to_payload(issue: &IssueProposal, field_mapping: &str) -> serde_json::Value {
    let mapping: serde_json::Value = serde_json::from_str(field_mapping).unwrap_or_default();
    let mut payload = serde_json::json!({});
    if let Some(obj) = mapping.as_object() {
        for (key, val) in obj {
            let v = match val.as_str().unwrap_or("") {
                "title" => &issue.title,
                "description" => &issue.description,
                "severity" => &issue.severity_candidate,
                "category" => &issue.issue_category,
                "bug_type" => &issue.bug_type,
                "source" => &issue.image_id,
                _ => "",
            };
            payload[key] = serde_json::Value::String(v.to_string());
        }
    }
    payload
}

/// Submit issue to the configured tracker.
pub fn submit_issue(
    config: &TrackerConfig,
    issue: &IssueProposal,
    previous_submissions: &[SubmissionRecord],
) -> Result<SubmissionRecord, String> {
    let ct = ConnectorType::from_str(&config.connector_type)
        .ok_or_else(|| format!("Unknown connector: {}", config.connector_type))?;
    if !ct.available() {
        return Err(format!("{} connector is not available (scaffold only)", ct.as_str()));
    }
    // Check for duplicate submissions
    for prev in previous_submissions {
        if prev.qaivra_issue_id == issue.id && prev.tracker_config_id == config.id {
            return Err(format!("Issue already submitted (external ID: {})", prev.external_id));
        }
    }
    match ct {
        ConnectorType::Mock => submit_mock(issue, config),
        _ => Err(format!("{} not implemented", ct.as_str())),
    }
}

fn submit_mock(issue: &IssueProposal, config: &TrackerConfig) -> Result<SubmissionRecord, String> {
    let payload = map_issue_to_payload(issue, &config.field_mapping_json);
    let fingerprint = format!("{:x}", Sha256::digest(serde_json::to_string(&payload).unwrap_or_default()));
    Ok(SubmissionRecord {
        id: format!("sub-{}", uuid::Uuid::new_v4()),
        qaivra_issue_id: issue.id.clone(),
        tracker_config_id: config.id.clone(),
        external_id: format!("MOCK-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("000")),
        payload_fingerprint: fingerprint,
        status: "submitted".to_string(),
        submitted_at: chrono::Utc::now().to_rfc3339(),
    })
}