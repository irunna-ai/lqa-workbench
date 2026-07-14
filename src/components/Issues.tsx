import { useState, useEffect, useMemo } from "react";
import type { Project, NavigationPage, IssueProposal, DuplicateRelationship, KnownIssue } from "../types";
import { imageApi } from "../api/tauri";

interface IssuesProps {
  activeProject: Project | null;
  onNavigate?: (page: NavigationPage) => void;
}

type IssueTab = "all" | "needs_review" | "approved" | "rejected" | "known";

function severityClass(sev: string): string {
  if (sev.startsWith("P1")) return "sev-p1";
  if (sev.startsWith("P2")) return "sev-p2";
  if (sev.startsWith("P3")) return "sev-p3";
  return "sev-unresolved";
}

function lifecycleClass(state: string): string {
  const map: Record<string, string> = {
    proposed: "lc-proposed", needs_review: "lc-review",
    approved: "lc-approved", rejected: "lc-rejected",
    known_issue: "lc-known", duplicate: "lc-dup",
    resolved: "lc-resolved",
  };
  return map[state] || "lc-proposed";
}

export default function Issues({ activeProject }: IssuesProps) {
  const [issues, setIssues] = useState<IssueProposal[]>([]);
  const [selected, setSelected] = useState<IssueProposal | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [activeTab, setActiveTab] = useState<IssueTab>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [severityFilter, setSeverityFilter] = useState("all");
  const [duplicates, setDuplicates] = useState<[DuplicateRelationship, IssueProposal][]>([]);
  const [knownIssues, setKnownIssues] = useState<KnownIssue[]>([]);

  const load = async () => {
    if (!activeProject) { setIssues([]); return; }
    try {
      const list = await imageApi.listIssues(activeProject.id);
      setIssues(list);
      setSelected(null);
      setDuplicates([]);
      setKnownIssues([]);
    } catch { setIssues([]); }
  };

  const loadKnownIssues = async () => {
    if (!activeProject) return;
    try { setKnownIssues(await imageApi.listKnownIssues(activeProject.id)); } catch {}
  };

  useEffect(() => { load(); loadKnownIssues(); }, [activeProject]);

  const handleLifecycle = async (target: string) => {
    if (!selected) return;
    setError(null); setBusy(true);
    try {
      await imageApi.updateIssueLifecycle(selected.id, target);
      await load();
      setSelected(null);
    } catch (err) { setError(String(err)); }
    finally { setBusy(false); }
  };

  const handleFindDuplicates = async () => {
    if (!selected || !activeProject) return;
    setBusy(true);
    try {
      setDuplicates(await imageApi.findDuplicateCandidates(activeProject.id, selected.id));
    } catch (err) { setError(String(err)); }
    finally { setBusy(false); }
  };

  const handleConfirmDuplicate = async (relId: string) => {
    setBusy(true);
    try {
      await imageApi.confirmDuplicate(relId);
      await load();
      setDuplicates([]);
    } catch (err) { setError(String(err)); }
    finally { setBusy(false); }
  };

  const handleMarkKnown = async () => {
    if (!selected || !activeProject) return;
    setBusy(true);
    try {
      await imageApi.createKnownIssue(
        activeProject.id, selected.title, selected.description,
        selected.issue_category, selected.issue_subcategory, selected.bug_type,
        selected.severity_candidate, selected.id
      );
      await handleLifecycle("known_issue");
    } catch (err) { setError(String(err)); }
    finally { setBusy(false); }
  };

  const filtered = (() => {
    let result = issues;
    if (activeTab === "needs_review") result = result.filter(i => i.lifecycle === "needs_review" || i.lifecycle === "proposed");
    else if (activeTab === "approved") result = result.filter(i => i.lifecycle === "approved");
    else if (activeTab === "rejected") result = result.filter(i => i.lifecycle === "rejected");
    else if (activeTab === "known") result = result.filter(i => i.lifecycle === "known_issue");
    if (severityFilter !== "all") result = result.filter(i => i.severity_candidate.startsWith(severityFilter));
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter(i => i.title.toLowerCase().includes(q) || i.description.toLowerCase().includes(q) || i.bug_type.toLowerCase().includes(q));
    }
    return result;
  })();

  if (!activeProject) {
    return (
      <div className="page">
        <h2>Issues</h2>
        <div className="empty-state"><p>No Project Selected</p></div>
      </div>
    );
  }

  const handleReview = async (p: IssueProposal, status: "approved" | "rejected") => {
    setError(null);
    setBusy(true);
    try {
      await imageApi.updateIssueReview(p.id, status);
      await load();
      setSelected(null);
    } catch (err) { setError(String(err)); }
    finally { setBusy(false); }
  };

  return (
    <div className="page issues-page">
      <div className="issues-header">
        <h2>Issues</h2>
        <span className="issue-count-badge">{issues.length} total</span>
      </div>

      {/* Tabs */}
      <div className="issues-tabs">
        {([["all", "All"], ["needs_review", "Needs Review"], ["approved", "Approved"], ["rejected", "Rejected"], ["known", "Known Issues"]] as [IssueTab, string][]).map(([tab, label]) => (
          <button key={tab} className={activeTab === tab ? "active" : ""}
            onClick={() => { setActiveTab(tab); setSelected(null); }}>
            {label}{tab === "all" ? ` (${issues.length})` : ""}
          </button>
        ))}
      </div>

      {/* Search and Filters */}
      <div className="issues-toolbar">
        <input type="text" placeholder="Search issues..." value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)} />
        <select value={severityFilter} onChange={e => setSeverityFilter(e.target.value)}>
          <option value="all">All Severities</option>
          <option value="P1">P1 - Critical</option>
          <option value="P2">P2 - Major</option>
          <option value="P3">P3 - Minor</option>
          <option value="UNRESOLVED">Unresolved</option>
        </select>
      </div>

      {error && <div className="error-banner"><span>{error}</span></div>}

      <div className="issues-split">
        <div className="issues-list-col">
          {filtered.length === 0 ? (
            <div className="issue-detail-empty">
              <div style={{ fontSize: "2rem", marginBottom: "0.75rem", opacity: 0.5 }}>📋</div>
              <div>
                {issues.length === 0 ? "No issues found. Run an analysis to generate issue proposals." : "No issues match the current filters."}
              </div>
              {issues.length === 0 && activeProject && (
                <button className="btn-primary" style={{ marginTop: "0.75rem" }}
                  onClick={() => {/* navigate to image analysis */}}>
                  Start Image Analysis
                </button>
              )}
            </div>
          ) : (
            filtered.map((p) => (
              <button key={p.id}
                className={`issue-row ${selected?.id===p.id?"active":""}`}
                onClick={() => setSelected(p)}>
                <span className={`sev-badge ${severityClass(p.severity_candidate)}`}>{p.severity_candidate}</span>
                <span className="issue-title-col">
                  <span className="issue-title">{p.title}</span>
                  <span className="issue-subtitle">{p.bug_type}</span>
                </span>
                <span className={`lifecycle-badge ${lifecycleClass(p.lifecycle)}`}>{p.lifecycle}</span>
              </button>
            ))
          )}
        </div>

        <div className="issues-detail-col">
          {!selected ? (
            <div className="issue-detail-empty">
              <div className="empty-icon">🔍</div>
              <div>Select an issue to view details</div>
            </div>
          ) : (
            <div className="issue-detail">
              <div className="detail-badges">
                <span className={`sev-badge ${severityClass(selected.severity_candidate)}`}>{selected.severity_candidate}</span>
                <span className={`lifecycle-badge ${lifecycleClass(selected.lifecycle)}`}>{selected.lifecycle}</span>
                <span className="detail-bugtype">{selected.bug_type}</span>
              </div>
              <h3>{selected.title}</h3>
              <p className="detail-desc">{selected.description}</p>
              {selected.detected_text && (
                <p className="detail-detected"><strong>Detected:</strong> "{selected.detected_text}"</p>
              )}
              <div className="detail-meta">
                <div><strong>Category:</strong> {selected.issue_category} / {selected.issue_subcategory}</div>
                <div><strong>Confidence:</strong> {(selected.confidence * 100).toFixed(0)}%</div>
                <div><strong>Rule ID:</strong> {selected.severity_rule_id || "— (no matched rule) —"}</div>
                <div><strong>Provenance:</strong> analysis {selected.analysis_id} · image {selected.image_id}</div>
              </div>
              {selected.severity_reasoning && <p className="detail-reasoning">{selected.severity_reasoning}</p>}
              {selected.glossary_findings && <p className="detail-glossary"><strong>Glossary:</strong> {selected.glossary_findings}</p>}

              <div className="detail-actions">
                <strong>Lifecycle:</strong>
                <button className="btn-secondary" disabled={busy} onClick={() => handleLifecycle("needs_review")}>Needs Review</button>
                <button className="btn-secondary" disabled={busy} onClick={() => handleLifecycle("approved")}>Approve</button>
                <button className="btn-secondary" disabled={busy} onClick={() => handleLifecycle("rejected")}>Reject</button>
                <button className="btn-secondary" disabled={busy} onClick={handleMarkKnown}>Mark Known Issue</button>
                <button className="btn-secondary" disabled={busy} onClick={handleFindDuplicates}>Find Duplicates</button>
                {selected.lifecycle === "approved" && (
                  <button className="btn-secondary" disabled={busy} onClick={() => handleLifecycle("resolved")}>Resolve</button>
                )}
              </div>

              {duplicates.length > 0 && (
                <div className="duplicates-section">
                  <h4>Duplicate Candidates</h4>
                  {duplicates.map(([rel, dup]) => (
                    <div key={rel.id} className="dup-row">
                      <span className="dup-confidence">{(rel.confidence*100).toFixed(0)}% match</span>
                      <span>{dup.title} — {dup.bug_type}</span>
                      <span className={`sev-badge ${severityClass(dup.severity_candidate)}`}>{dup.severity_candidate}</span>
                      <button className="btn-small" disabled={busy} onClick={() => handleConfirmDuplicate(rel.id)}>Confirm Duplicate</button>
                    </div>
                  ))}
                </div>
              )}

              {knownIssues.length > 0 && (
                <div className="known-issues-section">
                  <h4>Known Issues</h4>
                  {knownIssues.map(ki => (
                    <div key={ki.id} className="ki-row">
                      <span>{ki.title}</span>
                      <span className={`sev-badge ${severityClass(ki.severity)}`}>{ki.severity}</span>
                      <span>{ki.category}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
