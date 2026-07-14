import { useState } from "react";
import type { Project, ProjectBrainResult } from "../types";
import { brainApi } from "../api/tauri";

interface ProjectBrainProps { activeProject: Project | null; }

export default function ProjectBrain({ activeProject }: ProjectBrainProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ProjectBrainResult | null>(null);
  const [searching, setSearching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasSearched, setHasSearched] = useState(false);

  const handleSearch = async () => {
    if (!activeProject || !query.trim()) return;
    setSearching(true); setError(null); setHasSearched(true);
    try {
      const r = await brainApi.query(activeProject.id, query.trim());
      setResults(r);
    } catch (err) { setError(String(err)); }
    finally { setSearching(false); }
  };

  if (!activeProject) {
    return (<div className="page-container"><div className="empty-state">
      <div className="empty-icon">🔍</div><h2>No Project Selected</h2>
      <p>Select a project to search Project Brain.</p></div></div>);
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <h2>Ask Project Brain</h2>
        <span className="project-context">{activeProject.name}</span>
      </div>

      <div className="brain-search-box">
        <input type="text" value={query} onChange={e => setQuery(e.target.value)}
          onKeyDown={e => e.key === "Enter" && handleSearch()}
          placeholder="Search project knowledge..."
          className="brain-input" />
        <button className="btn-primary" onClick={handleSearch} disabled={searching || !query.trim()}>
          {searching ? "Searching..." : "Search"}</button>
      </div>
      <p className="brain-description">
        Query project-scoped knowledge with exact provenance. Results are grounded in stored data — not AI-generated.</p>

      {error && <div className="error-message"><span>{error}</span><button onClick={() => setError(null)}>×</button></div>}

      {hasSearched && results && results.result_count === 0 && (
        <div className="empty-state-inline">
          <div className="empty-icon">🔍</div><h3>No Results Found</h3>
          <p>No project knowledge matched "{query}". No stored authority covers this topic.</p>
        </div>
      )}

      {results && results.result_count > 0 && (
        <div className="brain-results">
          <div className="results-summary">
            Found {results.result_count} result(s):
            {results.severity_results.length > 0 && <span className="result-type-badge">Severity: {results.severity_results.length}</span>}
            {results.glossary_results.length > 0 && <span className="result-type-badge">Glossary: {results.glossary_results.length}</span>}
            {results.reference_results.length > 0 && <span className="result-type-badge">References: {results.reference_results.length}</span>}
            {results.precedent_results.length > 0 && <span className="result-type-badge">Precedent: {results.precedent_results.length}</span>}
          </div>

          {results.severity_results.length > 0 && (
            <div className="result-section"><h3>📋 Severity Rules <span className="authority-label">SEVERITY AUTHORITY</span></h3>
              {results.severity_results.map(r => (
                <div key={r.id} className="result-card severity-result">
                  <div className="result-main">
                    <strong>{r.canonical_bug_type}</strong>
                    <span className={`severity-badge severity-${r.severity_code.toLowerCase()}`}>{r.severity_exact}</span>
                  </div>
                  {r.description_raw && <p className="result-desc">{r.description_raw}</p>}
                  <div className="result-provenance">📁 {r.source_filename} → {r.source_sheet} → Row {r.source_row}</div>
                </div>))}
            </div>)}

          {results.glossary_results.length > 0 && (
            <div className="result-section"><h3>📖 Glossary <span className="authority-label">GLOSSARY AUTHORITY</span></h3>
              {results.glossary_results.map(e => (
                <div key={e.id} className="result-card glossary-result">
                  <div className="result-main">
                    <span className="term-source">{e.source_term}</span>→
                    <span className="term-target">{e.target_term || "(no translation)"}</span>
                    {e.do_not_translate && <span className="dnt-badge">DNT</span>}
                    {e.banned && <span className="banned-badge">BANNED</span>}
                    <span className={`status-badge status-${e.status}`}>{e.status}</span>
                  </div>
                  {e.notes && <p className="result-notes">{e.notes}</p>}
                  <div className="result-provenance">📁 {e.source_filename} → Row {e.source_row}</div>
                </div>))}
            </div>)}

          {results.reference_results.length > 0 && (
            <div className="result-section"><h3>📄 Reference Knowledge <span className="authority-label">REFERENCE CONTEXT</span></h3>
              {results.reference_results.map(c => (
                <div key={c.id} className="result-card reference-result">
                  <div className="result-main">
                    {c.heading && <strong>{c.heading}</strong>}
                    <p className="result-content">{c.content.length > 300 ? c.content.slice(0, 300) + "..." : c.content}</p>
                  </div>
                  <div className="result-provenance">📁 {c.source_filename}{c.source_page > 0 ? ` → Page ${c.source_page}` : ""} → Chunk #{c.chunk_order}</div>
                </div>))}
            </div>)}

          {results.precedent_results.length > 0 && (
            <div className="result-section"><h3>📊 Precedent Reports <span className="authority-label">PRECEDENT</span></h3>
              {results.precedent_results.map(p => (
                <div key={p.id} className="result-card precedent-result">
                  <div className="result-main">
                    <span className="precedent-classification-badge">{p.classification}</span>
                    {p.heading && <strong>{p.heading}</strong>}
                    <p className="result-content">{p.content.length > 300 ? p.content.slice(0, 300) + "..." : p.content}</p>
                    {p.issue_type && <span className="result-issue-type">Type: {p.issue_type}</span>}
                    {p.severity_exact && <span className="result-severity">Severity: {p.severity_exact}</span>}
                  </div>
                  <div className="result-provenance">📁 {p.source_filename}{p.sheet_name ? ` → Sheet: ${p.sheet_name}` : ""}{p.page_number > 0 ? ` → Page ${p.page_number}` : ""} → Row {p.row_start}</div>
                </div>))}
            </div>)}
        </div>)}

      {!hasSearched && (
        <div className="brain-hints"><h3>Example Queries</h3>
          <div className="hint-chips">
            <button className="hint-chip" onClick={() => setQuery("P1 Critical")}>P1 Critical</button>
            <button className="hint-chip" onClick={() => setQuery("Overlap")}>Overlap</button>
            <button className="hint-chip" onClick={() => setQuery("Mistranslation")}>Mistranslation</button>
          </div>
        </div>)}
    </div>
  );
}