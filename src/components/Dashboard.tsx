import { useState, useEffect } from "react";
import type { Project, NavigationPage } from "../types";
import type { SeverityStatus, GlossaryStatus, ReferenceStatus } from "../types";
import { severityApi, glossaryApi, referenceApi } from "../api/tauri";

interface DashboardProps {
  activeProject: Project | null;
  onNavigate: (page: NavigationPage) => void;
  onCreateProject: () => void;
}

export default function Dashboard({ activeProject, onNavigate, onCreateProject }: DashboardProps) {
  const [severityStatus, setSeverityStatus] = useState<SeverityStatus | null>(null);
  const [glossaryStatus, setGlossaryStatus] = useState<GlossaryStatus | null>(null);
  const [referenceStatus, setReferenceStatus] = useState<ReferenceStatus | null>(null);

  useEffect(() => {
    if (activeProject) {
      severityApi.getStatus(activeProject.id).then(setSeverityStatus).catch(() => setSeverityStatus(null));
      glossaryApi.getStatus(activeProject.id).then(setGlossaryStatus).catch(() => setGlossaryStatus(null));
      referenceApi.getStatus(activeProject.id).then(setReferenceStatus).catch(() => setReferenceStatus(null));
    } else {
      setSeverityStatus(null);
      setGlossaryStatus(null);
      setReferenceStatus(null);
    }
  }, [activeProject]);

  // No project selected state
  if (!activeProject) {
    return (
      <div className="dashboard">
        <div className="empty-state">
          <div className="empty-icon">📋</div>
          <h2>Welcome to QAIVRA</h2>
          <p className="empty-description">
            Create or select a project to begin your Localization QA workflow.
          </p>
          <button className="btn-primary" onClick={onCreateProject}>
            Create Your First Project
          </button>
          <div className="empty-info">
            <h3>Getting Started</h3>
            <ul>
              <li>Create a project with your target language and game details</li>
              <li>Import severity rules from your client's reference workbook</li>
              <li>Load your glossary for terminology consistency</li>
              <li>Analyze screenshots and gameplay video for localization issues</li>
              <li>Export client-compatible LQA reports</li>
            </ul>
          </div>
        </div>
      </div>
    );
  }

  // Project selected - empty dashboard
  return (
    <div className="dashboard">
      <div className="dashboard-header">
        <h2>{activeProject.name}</h2>
        <div className="project-meta">
          {activeProject.target_language && (
            <span className="meta-tag">Language: {activeProject.target_language}</span>
          )}
          {activeProject.genre && <span className="meta-tag">Genre: {activeProject.genre}</span>}
          {activeProject.platform && (
            <span className="meta-tag">Platform: {activeProject.platform}</span>
          )}
        </div>
      </div>

      <div className="dashboard-grid">
        {/* Project Brain Status Cards */}
        <div className="status-card">
          <h3>Severity Rules</h3>
          {severityStatus?.has_rules ? (
            <div className="status-ready">
              <span className="status-icon">✅</span>
              <p>{severityStatus.rule_count} rules loaded</p>
              <span className="status-source">Source: {severityStatus.source_filename}</span>
              <button
                className="btn-secondary"
                onClick={() => onNavigate("knowledge-rules")}
              >
                View Rules
              </button>
            </div>
          ) : (
            <div className="status-empty">
              <span className="status-icon">⚠️</span>
              <p>No rules loaded</p>
              <button
                className="btn-secondary"
                onClick={() => onNavigate("knowledge-rules")}
              >
                Import Rules
              </button>
            </div>
          )}
        </div>

        <div className="status-card">
          <h3>Glossary</h3>
          {glossaryStatus?.has_glossary ? (
            <div className="status-ready">
              <span className="status-icon">✅</span>
              <p>{glossaryStatus.entry_count} terms loaded</p>
              <span className="status-source">
                {glossaryStatus.dnt_count} DNT, {glossaryStatus.banned_count} banned
              </span>
              <button className="btn-secondary" onClick={() => onNavigate("knowledge-glossary")}>
                View Glossary
              </button>
            </div>
          ) : (
            <div className="status-empty">
              <span className="status-icon">📖</span>
              <p>No glossary loaded</p>
              <button className="btn-secondary" onClick={() => onNavigate("knowledge-glossary")}>
                Import Glossary
              </button>
            </div>
          )}
        </div>

        <div className="status-card">
          <h3>Reference Documents</h3>
          {referenceStatus?.has_references ? (
            <div className="status-ready">
              <span className="status-icon">✅</span>
              <p>{referenceStatus.chunk_count} chunks from {referenceStatus.source_count} doc(s)</p>
              <button className="btn-secondary" onClick={() => onNavigate("knowledge-references")}>
                View References
              </button>
            </div>
          ) : (
            <div className="status-empty">
              <span className="status-icon">📄</span>
              <p>No documents loaded</p>
              <button className="btn-secondary" onClick={() => onNavigate("knowledge-references")}>
                Import Documents
              </button>
            </div>
          )}
        </div>

        <div className="status-card">
          <h3>AI Model</h3>
          <div className="status-empty">
            <span className="status-icon">🤖</span>
            <p>Not configured</p>
            <button
              className="btn-secondary"
              onClick={() => onNavigate("settings-ai")}
            >
              Configure
            </button>
          </div>
        </div>
      </div>

      <div className="dashboard-section">
        <h3>Analysis Summary</h3>
        <div className="summary-empty">
          <p>No analysis data yet. Upload images or video to begin analysis.</p>
          <div className="summary-actions">
            <button className="btn-secondary" onClick={() => onNavigate("analyze-image")}>
              Analyze Images
            </button>
            <button className="btn-secondary" onClick={() => onNavigate("analyze-video")}>
              Analyze Video
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}