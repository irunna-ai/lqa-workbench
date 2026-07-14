import { useState, useEffect, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { Project, ReferenceChunk, ReferenceStatus } from "../types";
import { referenceApi } from "../api/tauri";

interface ReferenceDocsProps { activeProject: Project | null; }

export default function ReferenceDocs({ activeProject }: ReferenceDocsProps) {
  const [chunks, setChunks] = useState<ReferenceChunk[]>([]);
  const [status, setStatus] = useState<ReferenceStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);
  const [importWarnings, setImportWarnings] = useState<string[]>([]);
  const [expandedChunk, setExpandedChunk] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    if (!activeProject) return;
    setLoading(true);
    try {
      const [s, c] = await Promise.all([
        referenceApi.getStatus(activeProject.id),
        referenceApi.listChunks(activeProject.id),
      ]);
      setStatus(s); setChunks(c); setError(null);
    } catch (err) { setError(String(err)); }
    finally { setLoading(false); }
  }, [activeProject]);

  useEffect(() => { loadData(); }, [loadData]);

  const handleImport = async () => {
    if (!activeProject) return;
    try {
      const selected = await open({ multiple: false,
        filters: [{ name: "Documents", extensions: ["xlsx", "csv", "docx", "pdf", "txt"] }] });
      if (!selected) return;
      setImporting(true); setError(null); setImportWarnings([]);
      const result = await referenceApi.import(activeProject.id, selected as string);
      setImportWarnings(result.warnings);
      await loadData();
    } catch (err) { setError(String(err)); }
    finally { setImporting(false); }
  };

  if (!activeProject) {
    return (<div className="page-container"><div className="empty-state">
      <div className="empty-icon">📄</div><h2>No Project Selected</h2>
      <p>Select a project to manage reference documents.</p></div></div>);
  }

  // Group chunks by source filename
  const grouped = chunks.reduce<Record<string, ReferenceChunk[]>>((acc, c) => {
    (acc[c.source_filename] = acc[c.source_filename] || []).push(c);
    return acc;
  }, {});

  return (
    <div className="page-container">
      <div className="page-header">
        <h2>Reference Documents</h2>
        <span className="project-context">{activeProject.name}</span>
      </div>

      <div className={`readiness-banner readiness-${status?.has_references ? "ready" : "missing"}`}>
        <span className="readiness-icon">{status?.has_references ? "✅" : "📄"}</span>
        <div className="readiness-info">
          <strong>{status?.has_references ? "Reference Knowledge Available" : "No Reference Documents"}</strong>
          <span className="readiness-detail">
            {status?.has_references
              ? `${status.chunk_count} chunks from ${status.source_count} document(s)`
              : "Import reference documents to provide project context knowledge."}
          </span>
        </div>
      </div>

      <div className="actions-bar">
        <button className="btn-primary" onClick={handleImport} disabled={importing}>
          {importing ? "Importing..." : "Import Reference Document"}
        </button>
      </div>

      {error && <div className="error-message"><span>{error}</span><button onClick={() => setError(null)}>×</button></div>}
      {importWarnings.length > 0 && <div className="warnings-panel"><h4>Parser Warnings</h4>
        <ul>{importWarnings.map((w, i) => <li key={i}>{w}</li>)}</ul></div>}

      {loading ? <div className="loading-indicator">Loading reference documents...</div>
       : !status?.has_references ? (
        <div className="empty-state-inline">
          <div className="empty-icon">📄</div><h3>No Reference Documents</h3>
          <p>Import game documentation, localization guides, style guides, or other reference material.</p>
          <p className="empty-note">Supported formats: XLSX, CSV, DOCX, PDF, TXT</p>
        </div>
      ) : (
        Object.entries(grouped).map(([filename, fileChunks]) => (
          <div key={filename} className="reference-group">
            <h3 className="reference-filename">📄 {filename}
              <span className="chunk-count">{fileChunks.length} chunk(s)</span>
              <span className="source-type-badge">{fileChunks[0].source_type}</span></h3>
            <div className="chunks-list">
              {fileChunks.map((chunk) => (
                <div key={chunk.id} className={`chunk-card ${expandedChunk === chunk.id ? "expanded" : ""}`}
                  onClick={() => setExpandedChunk(expandedChunk === chunk.id ? null : chunk.id)}>
                  <div className="chunk-header">
                    <span className="chunk-order">#{chunk.chunk_order}</span>
                    {chunk.heading && <span className="chunk-heading">{chunk.heading}</span>}
                    <span className="chunk-provenance">
                      {chunk.source_page > 0 && `Page ${chunk.source_page}`}
                      {chunk.source_sheet && `Sheet: ${chunk.source_sheet}`}
                      {chunk.source_row_start > 0 && ` Rows ${chunk.source_row_start}-${chunk.source_row_end}`}
                    </span>
                  </div>
                  {expandedChunk === chunk.id && (
                    <div className="chunk-content">{chunk.content}</div>
                  )}
                </div>
              ))}
            </div>
          </div>
        ))
      )}
    </div>
  );
}