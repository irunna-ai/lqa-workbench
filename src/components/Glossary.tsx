import { useState, useEffect, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { Project, GlossaryEntry, GlossaryStatus, GlossaryReadiness,
  GlossaryHeaderResult, ColumnMapping } from "../types";
import { glossaryApi } from "../api/tauri";

interface GlossaryProps { activeProject: Project | null; }

export default function Glossary({ activeProject }: GlossaryProps) {
  const [entries, setEntries] = useState<GlossaryEntry[]>([]);
  const [status, setStatus] = useState<GlossaryStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);
  const [headerResult, setHeaderResult] = useState<GlossaryHeaderResult | null>(null);
  const [mapping, setMapping] = useState<ColumnMapping>({ source_term_col: 0 });
  const [importWarnings, setImportWarnings] = useState<string[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [showMapping, setShowMapping] = useState(false);

  const loadData = useCallback(async () => {
    if (!activeProject) return;
    setLoading(true);
    try {
      const [s, e] = await Promise.all([
        glossaryApi.getStatus(activeProject.id),
        glossaryApi.listEntries(activeProject.id),
      ]);
      setStatus(s); setEntries(e); setError(null);
    } catch (err) { setError(String(err)); }
    finally { setLoading(false); }
  }, [activeProject]);

  useEffect(() => { loadData(); }, [loadData]);

  const readiness: GlossaryReadiness = (!status || !status.has_glossary) ? "MISSING"
    : (status.conflict_count > 0) ? "CONFLICT_REVIEW" : "READY";

  const handleStartImport = async () => {
    if (!activeProject) return;
    try {
      const selected = await open({ multiple: false,
        filters: [{ name: "Glossary", extensions: ["xlsx", "csv", "docx", "pdf", "txt"] }] });
      if (!selected) return;
      const fp = selected as string;
      setSelectedFile(fp); setError(null); setImportWarnings([]);
      const ext = fp.split(".").pop()?.toLowerCase();
      if (ext === "xlsx" || ext === "csv") {
        const h = await glossaryApi.parseHeaders(fp);
        setHeaderResult(h); setMapping({ source_term_col: 0 }); setShowMapping(true);
      } else {
        setImporting(true);
        const r = await glossaryApi.import(activeProject.id, fp, { source_term_col: 0 });
        setImportWarnings(r.warnings); await loadData();
      }
    } catch (err) { setError(String(err)); }
    finally { setImporting(false); }
  };

  const handleConfirmImport = async () => {
    if (!activeProject || !selectedFile) return;
    setImporting(true);
    try {
      const r = await glossaryApi.import(activeProject.id, selectedFile, mapping, headerResult?.active_sheet);
      setImportWarnings(r.warnings); setShowMapping(false); setHeaderResult(null);
      await loadData();
    } catch (err) { setError(String(err)); }
    finally { setImporting(false); }
  };

  if (!activeProject) {
    return (<div className="page-container"><div className="empty-state">
      <div className="empty-icon">📖</div><h2>No Project Selected</h2>
      <p>Select a project to manage the glossary.</p></div></div>);
  }

  if (showMapping && headerResult?.success && headerResult.headers) {
    const hdrs = headerResult.headers;
    const samples = headerResult.sample_rows || [];
    const mapField = (label: string, key: keyof ColumnMapping) => (
      <div className="mapping-field" key={key}>
        <label>{label}</label>
        <select value={mapping[key] ?? -1} onChange={e => {
          const v = Number(e.target.value);
          setMapping(m => ({...m, [key]: v >= 0 ? v : undefined}));
        }}>
          <option value={-1}>— Not mapped —</option>
          {hdrs.map((h, i) => <option key={i} value={i}>{i}: {h}</option>)}
        </select>
      </div>
    );
    return (
      <div className="page-container">
        <div className="page-header"><h2>Import Glossary — Column Mapping</h2>
          <span className="project-context">{activeProject.name}</span></div>
        <div className="source-info">
          <div className="source-field"><label>File:</label><span>{headerResult.filename}</span></div>
          {headerResult.active_sheet && <div className="source-field"><label>Sheet:</label><span>{headerResult.active_sheet}</span></div>}
        </div>
        <div className="mapping-section">
          <h3>Detected Columns</h3>
          <div className="headers-preview">{hdrs.map((h, i) => <span key={i} className="header-chip">{i}: {h}</span>)}</div>
          {samples.length > 0 && <div className="sample-table"><h4>Sample Data</h4>
            <table className="rules-table"><thead><tr>{hdrs.map((h, i) => <th key={i}>{h}</th>)}</tr></thead>
            <tbody>{samples.map((row, ri) => <tr key={ri}>{row.map((c, ci) => <td key={ci}>{c}</td>)}</tr>)}</tbody></table></div>}
          <h3>Map Columns</h3>
          <div className="mapping-fields">
            <div className="mapping-field"><label>Source Term (required) *</label>
              <select value={mapping.source_term_col} onChange={e => setMapping(m => ({...m, source_term_col: Number(e.target.value)}))}>
                {hdrs.map((h, i) => <option key={i} value={i}>{i}: {h}</option>)}</select></div>
            {mapField("Target Term / Translation", "target_term_col")}
            {mapField("Status", "status_col")}
            {mapField("Do Not Translate (DNT)", "dnt_col")}
            {mapField("Banned / Forbidden", "banned_col")}
            {mapField("Notes / Context", "notes_col")}
          </div>
        </div>
        <div className="actions-bar" style={{ display: "flex", gap: "0.5rem" }}>
          <button className="btn-primary" onClick={handleConfirmImport} disabled={importing}>
            {importing ? "Importing..." : "Confirm Import"}</button>
          <button className="btn-secondary" onClick={() => { setShowMapping(false); setHeaderResult(null); }}>Cancel</button>
        </div>
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <h2>Glossary</h2>
        <span className="project-context">{activeProject.name}</span>
      </div>

      <div className={`readiness-banner readiness-${readiness.toLowerCase()}`}>
        <span className="readiness-icon">{readiness === "READY" ? "✅" : "❌"}</span>
        <div className="readiness-info">
          <strong>{readiness === "READY" ? "Glossary Ready" : readiness === "CONFLICT_REVIEW" ? "Glossary - Conflicts Detected" : "No Glossary Loaded"}</strong>
          <span className="readiness-detail">
            {readiness === "MISSING"
              ? "Import a glossary to enable canonical terminology authority for this project."
              : readiness === "CONFLICT_REVIEW"
              ? `${status?.entry_count} terms loaded, ${status?.conflict_count} unresolved duplicate conflict(s). Review and resolve before analysis.`
              : `${status?.entry_count} terms (${status?.dnt_count} DNT, ${status?.banned_count} banned) from ${status?.source_filename}`}
          </span>
        </div>
      </div>

      {status?.has_glossary && (
        <div className="source-info">
          <div className="source-field"><label>Total Terms:</label><span>{status.entry_count}</span></div>
          <div className="source-field"><label>DNT Terms:</label><span>{status.dnt_count}</span></div>
          <div className="source-field"><label>Banned Terms:</label><span>{status.banned_count}</span></div>
          <div className="source-field"><label>Source File:</label><span>{status.source_filename}</span></div>
        </div>
      )}

      <div className="actions-bar">
        <button className="btn-primary" onClick={handleStartImport} disabled={importing}>
          {importing ? "Importing..." : status?.has_glossary ? "Replace Glossary" : "Import Glossary"}
        </button>
      </div>

      {error && <div className="error-message"><span>{error}</span><button onClick={() => setError(null)}>×</button></div>}
      {importWarnings.length > 0 && <div className="warnings-panel"><h4>Parser Warnings</h4>
        <ul>{importWarnings.map((w, i) => <li key={i}>{w}</li>)}</ul></div>}

      {loading ? <div className="loading-indicator">Loading glossary...</div>
       : readiness === "MISSING" ? (
        <div className="empty-state-inline">
          <div className="empty-icon">📖</div><h3>No Glossary</h3>
          <p>Import a glossary file (XLSX, CSV, DOCX, PDF, or TXT) to establish canonical terminology.</p>
          <p className="empty-note">Missing glossary does not mean terms are wrong — it means authority is unavailable.</p>
        </div>
      ) : (
        <div className="rules-table-container">
          <table className="rules-table">
            <thead><tr>
              <th>Row</th><th>Source Term</th><th>Target Term</th><th>Status</th><th>DNT</th><th>Banned</th><th>Notes</th>
            </tr></thead>
            <tbody>
              {entries.map((e) => (
                <tr key={e.id}>
                  <td className="row-num">{e.source_row}</td>
                  <td>{e.source_term}</td>
                  <td>{e.target_term}</td>
                  <td><span className={`status-badge status-${e.status}`}>{e.status}</span></td>
                  <td>{e.do_not_translate ? "✓" : ""}</td>
                  <td>{e.banned ? "🚫" : ""}</td>
                  <td className="notes-cell">{e.notes}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}