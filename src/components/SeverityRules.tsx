import { useState, useEffect, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { Project, SeverityRule, SeverityStatus, SeverityReadiness } from "../types";
import { severityApi } from "../api/tauri";

interface SeverityRulesProps {
  activeProject: Project | null;
}

export default function SeverityRules({ activeProject }: SeverityRulesProps) {
  const [rules, setRules] = useState<SeverityRule[]>([]);
  const [status, setStatus] = useState<SeverityStatus | null>(null);
  const [importing, setImporting] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [importWarnings, setImportWarnings] = useState<string[]>([]);

  const loadData = useCallback(async () => {
    if (!activeProject) return;
    setLoading(true);
    try {
      const [statusResult, rulesResult] = await Promise.all([
        severityApi.getStatus(activeProject.id),
        severityApi.listRules(activeProject.id),
      ]);
      setStatus(statusResult);
      setRules(rulesResult);
      setError(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [activeProject]);

  useEffect(() => { loadData(); }, [loadData]);

  const getReadiness = (): SeverityReadiness => {
    if (!status || !status.has_rules) return "MISSING";
    if (status.warning_count > 0) return "PARSE_WARNING";
    return "READY";
  };

  const handleImport = async () => {
    if (!activeProject) return;
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Excel", extensions: ["xlsx"] }],
      });
      if (!selected) return;
      setImporting(true);
      setError(null);
      setImportWarnings([]);
      const filePath = selected as string;
      const result = await severityApi.importRules(activeProject.id, filePath);
      setImportWarnings(result.warnings);
      await loadData();
    } catch (err) {
      setError(String(err));
    } finally {
      setImporting(false);
    }
  };

  if (!activeProject) {
    return (
      <div className="page-container">
        <div className="empty-state">
          <div className="empty-icon">📋</div>
          <h2>No Project Selected</h2>
          <p>Select a project to manage severity rules.</p>
        </div>
      </div>
    );
  }

  const readiness = getReadiness();

  return (
    <div className="page-container">
      <div className="page-header">
        <h2>Severity Rules</h2>
        <span className="project-context">{activeProject.name}</span>
      </div>

      <div className={`readiness-banner readiness-${readiness.toLowerCase()}`}>
        <span className="readiness-icon">
          {readiness === "READY" ? "✅" : readiness === "PARSE_WARNING" ? "⚠️" : "❌"}
        </span>
        <div className="readiness-info">
          <strong>
            {readiness === "READY" ? "Severity Rules Ready"
              : readiness === "PARSE_WARNING" ? "Severity Rules — Parse Warnings"
              : "No Severity Rules Imported"}
          </strong>
          <span className="readiness-detail">
            {readiness === "MISSING"
              ? "Authoritative severity assignment is unavailable until severity rules are imported for this project."
              : readiness === "PARSE_WARNING"
              ? `Imported with ${status?.warning_count} warning(s). Review before use.`
              : `${status?.rule_count} active rules from ${status?.source_filename}`}
          </span>
        </div>
      </div>

      {/* Summary Cards */}
      {status?.has_rules && (
        <div className="severity-summary-cards">
          {[
            { label: "Critical (P1)", count: rules.filter(r => r.severity_code === "P1").length, cls: "critical" },
            { label: "Major (P2)", count: rules.filter(r => r.severity_code === "P2").length, cls: "major" },
            { label: "Minor (P3)", count: rules.filter(r => r.severity_code === "P3").length, cls: "minor" },
            { label: "Trivial (P4)", count: rules.filter(r => r.severity_code === "P4").length, cls: "trivial" },
          ].map(card => (
            <div key={card.label} className={`severity-summary-card ${card.cls}`}>
              <div className="severity-summary-count">{card.count}</div>
              <div className="severity-summary-label">{card.label}</div>
            </div>
          ))}
        </div>
      )}

      {status?.has_rules && (
        <div className="source-info">
          <div className="source-field"><label>Source File:</label><span>{status.source_filename}</span></div>
          <div className="source-field"><label>Source Sheet:</label><span>{status.source_sheet}</span></div>
          <div className="source-field"><label>Active Rules:</label><span>{status.rule_count}</span></div>
          <div className="source-field"><label>Import Status:</label><span>{status.import_status}</span></div>
        </div>
      )}

      <div className="actions-bar">
        <button className="btn-primary" onClick={handleImport} disabled={importing}>
          {importing ? "Importing..." : status?.has_rules ? "Replace Rules Source" : "Import Severity Rules"}
        </button>
      </div>

      {error && (
        <div className="error-message">
          <span>{error}</span>
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      {importWarnings.length > 0 && (
        <div className="warnings-panel">
          <h4>Parser Warnings</h4>
          <ul>{importWarnings.map((w, i) => <li key={i}>{w}</li>)}</ul>
        </div>
      )}

      {loading ? (
        <div className="loading-indicator">Loading rules...</div>
      ) : readiness === "MISSING" ? (
        <div className="empty-state-inline">
          <div className="empty-icon">📖</div>
          <h3>No Severity Rules</h3>
          <p>Import a severity rules workbook (XLSX) to enable authoritative severity
            assignment for this project. The workbook must contain a "BUG info" sheet
            with canonical rule records.</p>
          <p className="empty-note">Without imported severity rules, the system cannot perform deterministic
            severity validation. Issues will remain in a review-required state.</p>
        </div>
      ) : (
        <div className="rules-table-container">
          <table className="rules-table">
            <thead>
              <tr><th>Row</th><th>Category</th><th>Sub-category</th><th>Bug Type</th><th>Severity</th></tr>
            </thead>
            <tbody>
              {rules.map((rule) => (
                <tr key={rule.id}>
                  <td className="row-num">{rule.source_row}</td>
                  <td>{rule.main_category_raw}</td>
                  <td>{rule.subcategory_raw}</td>
                  <td>{rule.canonical_bug_type}</td>
                  <td><span className={`severity-badge severity-${rule.severity_code.toLowerCase()}`}>{rule.severity_exact}</span></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}