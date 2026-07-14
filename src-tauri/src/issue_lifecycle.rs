// Phase 8: Issue Lifecycle, Duplicate Intelligence, Known Issues
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IssueLifecycle {
    Proposed, NeedsReview, Approved, Rejected, KnownIssue, Duplicate, Resolved,
}

impl IssueLifecycle {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "proposed" => Some(Self::Proposed), "needs_review" => Some(Self::NeedsReview),
            "approved" => Some(Self::Approved), "rejected" => Some(Self::Rejected),
            "known_issue" => Some(Self::KnownIssue), "duplicate" => Some(Self::Duplicate),
            "resolved" => Some(Self::Resolved), _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed", Self::NeedsReview => "needs_review",
            Self::Approved => "approved", Self::Rejected => "rejected",
            Self::KnownIssue => "known_issue", Self::Duplicate => "duplicate",
            Self::Resolved => "resolved",
        }
    }
    pub fn transition(&self, target: &IssueLifecycle) -> Option<IssueLifecycle> {
        match (self, target) {
            (_, _) if *self == IssueLifecycle::Proposed => Some(target.clone()),
            (IssueLifecycle::NeedsReview, t) if matches!(t, IssueLifecycle::Approved | IssueLifecycle::Rejected | IssueLifecycle::KnownIssue | IssueLifecycle::Duplicate) => Some(target.clone()),
            (IssueLifecycle::Approved, t) if matches!(t, IssueLifecycle::Resolved | IssueLifecycle::Rejected) => Some(target.clone()),
            (IssueLifecycle::Rejected, IssueLifecycle::NeedsReview) => Some(IssueLifecycle::NeedsReview),
            (IssueLifecycle::KnownIssue, t) if matches!(t, IssueLifecycle::NeedsReview | IssueLifecycle::Approved) => Some(target.clone()),
            (IssueLifecycle::Duplicate, IssueLifecycle::NeedsReview) => Some(IssueLifecycle::NeedsReview),
            (IssueLifecycle::Resolved, IssueLifecycle::NeedsReview) => Some(IssueLifecycle::NeedsReview),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateRelationship {
    pub id: String, pub project_id: String, pub source_issue_id: String,
    pub duplicate_issue_id: String, pub confidence: f64, pub match_signals: String,
    pub confirmed: bool, pub confirmed_by: String, pub created_at: String, pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownIssue {
    pub id: String, pub project_id: String, pub title: String, pub description: String,
    pub category: String, pub subcategory: String, pub bug_type: String, pub severity: String,
    pub source_issue_id: String, pub active: bool, pub created_at: String, pub updated_at: String,
}
pub fn compute_issue_fingerprint(
    detected_text: &str, target_text: &str, category: &str,
    subcategory: &str, bug_type: &str,
) -> String {
    let norm = |s: &str| -> String {
        s.to_lowercase().chars().filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ")
    };
    format!("{}|{}|{}|{}|{}", norm(detected_text), norm(target_text),
        norm(category), norm(subcategory), norm(bug_type))
}

pub fn score_duplicate_candidate(
    issue_a: &crate::db::IssueProposal, issue_b: &crate::db::IssueProposal,
) -> (f64, Vec<String>) {
    let mut score = 0.0f64; let mut signals = Vec::new();
    let lcase = |s: &str| s.to_lowercase();
    if !issue_a.issue_category.is_empty() && lcase(&issue_a.issue_category) == lcase(&issue_b.issue_category)
    { score += 0.2; signals.push("category_match".into()); }
    if !issue_a.issue_subcategory.is_empty() && lcase(&issue_a.issue_subcategory) == lcase(&issue_b.issue_subcategory)
    { score += 0.15; signals.push("subcategory_match".into()); }
    if !issue_a.bug_type.is_empty() && lcase(&issue_a.bug_type) == lcase(&issue_b.bug_type)
    { score += 0.2; signals.push("bug_type_match".into()); }
    let norm = |s: &str| -> String { s.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ") };
    let a_text = norm(&issue_a.detected_text); let b_text = norm(&issue_b.detected_text);
    if !a_text.is_empty() && a_text == b_text
    { score += 0.25; signals.push("detected_text_exact".into()); }
    else if !a_text.is_empty() && !b_text.is_empty() {
        let a_words: std::collections::HashSet<&str> = a_text.split_whitespace().collect();
        let b_words: std::collections::HashSet<&str> = b_text.split_whitespace().collect();
        let intersection = a_words.intersection(&b_words).count();
        let union = a_words.union(&b_words).count();
        if union > 0 { let j = intersection as f64 / union as f64; if j > 0.5 { score += 0.15 * j; signals.push(format!("text_jaccard_{:.2}", j)); } }
    }
    if issue_a.severity_candidate == issue_b.severity_candidate && issue_a.severity_candidate != "UNRESOLVED"
    { score += 0.1; signals.push("severity_match".into()); }
    if !issue_a.severity_rule_id.is_empty() && issue_a.severity_rule_id == issue_b.severity_rule_id
    { score += 0.1; signals.push("rule_match".into()); }
    (score.min(1.0), signals)
}

pub fn would_create_cycle(issue_id: &str, duplicate_of: &str, rels: &[DuplicateRelationship]) -> bool {
    if issue_id == duplicate_of { return true; }
    let mut graph: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for rel in rels { if rel.confirmed { graph.entry(rel.source_issue_id.as_str()).or_default().push(rel.duplicate_issue_id.as_str()); } }
    let mut visited = std::collections::HashSet::new(); let mut stack = vec![duplicate_of];
    while let Some(current) = stack.pop() {
        if current == issue_id { return true; }
        if !visited.insert(current) { continue; }
        if let Some(neighbors) = graph.get(current) { for n in neighbors { stack.push(n); } }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_lifecycle_transitions_valid() {
        assert!(IssueLifecycle::Proposed.transition(&IssueLifecycle::NeedsReview).is_some());
        assert!(IssueLifecycle::NeedsReview.transition(&IssueLifecycle::Approved).is_some());
        assert!(IssueLifecycle::Approved.transition(&IssueLifecycle::Resolved).is_some());
        assert!(IssueLifecycle::Rejected.transition(&IssueLifecycle::NeedsReview).is_some());
    }
    #[test] fn test_lifecycle_transitions_invalid() {
        assert!(IssueLifecycle::Approved.transition(&IssueLifecycle::Proposed).is_none());
        assert!(IssueLifecycle::Rejected.transition(&IssueLifecycle::Approved).is_none());
    }
    #[test] fn test_fingerprint() {
        let a = compute_issue_fingerprint("Hello", "World", "spelling", "typo", "Spelling");
        let b = compute_issue_fingerprint("Hello", "World", "spelling", "typo", "Spelling");
        assert_eq!(a, b);
    }
    #[test] fn test_no_cycle_self() { assert!(would_create_cycle("a", "a", &[])); }
    #[test] fn test_cycle_detection() {
        let rels = vec![DuplicateRelationship { id: "1".into(), project_id: "p".into(),
            source_issue_id: "b".into(), duplicate_issue_id: "c".into(), confidence: 0.9,
            match_signals: "[]".into(), confirmed: true, confirmed_by: "".into(),
            created_at: "".into(), updated_at: "".into() }];
        assert!(!would_create_cycle("a", "b", &rels));
    }
}