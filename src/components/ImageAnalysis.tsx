import { useState, useEffect, useCallback, useRef } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { Project, NavigationPage, ProjectImage, IssueProposal, TerminologyFinding, ImageAnalysisResult } from "../types";
import { imageApi } from "../api/tauri";
import EvidenceEditor from "./EvidenceEditor";

interface ImageAnalysisProps {
  activeProject: Project | null;
  onNavigate: (page: NavigationPage) => void;
}

type Tab = "upload" | "browse";

const SUPPORTED_IMAGE_FORMATS = ["png", "jpg", "jpeg", "webp", "bmp"];
const IMAGE_FORMATS: Record<string, string> = {
  png: "PNG", jpg: "JPEG", jpeg: "JPEG", webp: "WebP", bmp: "BMP",
};

function severityClass(sev: string): string {
  if (sev.startsWith("P1")) return "sev-p1";
  if (sev.startsWith("P2")) return "sev-p2";
  if (sev.startsWith("P3")) return "sev-p3";
  return "sev-unresolved";
}

function reviewClass(status: string): string {
  if (status === "approved") return "review-approved";
  if (status === "rejected") return "review-rejected";
  if (status === "review_required") return "review-required";
  return "review-pending";
}

export default function ImageAnalysis({ activeProject, onNavigate }: ImageAnalysisProps) {
  const [images, setImages] = useState<ProjectImage[]>([]);
  const [selectedImage, setSelectedImage] = useState<ProjectImage | null>(null);
  const [result, setResult] = useState<ImageAnalysisResult | null>(null);
  const [analyzingId, setAnalyzingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [evidenceIssueId, setEvidenceIssueId] = useState<string | null>(null);
  const [tab, setTab] = useState<Tab>("upload");
  const [isDragOver, setIsDragOver] = useState(false);
  const [selectedFileName, setSelectedFileName] = useState<string | null>(null);
  const dropZoneRef = useRef<HTMLDivElement>(null);

  const loadImages = async () => {
    if (!activeProject) { setImages([]); return; }
    try { const list = await imageApi.listImages(activeProject.id); setImages(list); }
    catch { setImages([]); }
  };

  useEffect(() => { loadImages(); setResult(null); setSelectedImage(null); }, [activeProject]);

  if (!activeProject) {
    return (
      <div className="page image-analysis video-page">
        <div className="image-empty-state">
          <div className="video-empty-icon">&#128247;</div>
          <h2>No Project Selected</h2>
          <p>Select or create a project to upload and analyze image files.</p>
        </div>
      </div>
    );
  }

  const handleUpload = async () => {
    setError(null);
    try {
      const selected = await open({ multiple: true, filters: [{ name: "Images", extensions: SUPPORTED_IMAGE_FORMATS }] });
      if (!selected || selected.length === 0) return;
      setBusy(true);
      for (const fp of selected as string[]) { await imageApi.uploadImage(activeProject.id, fp); }
      await loadImages(); setTab("browse");
    } catch (err) { setError(String(err)); }
    finally { setBusy(false); setSelectedFileName(null); }
  };

  const handleDragEnter = useCallback((e: React.DragEvent) => { e.preventDefault(); e.stopPropagation(); setIsDragOver(true); }, []);
  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault(); e.stopPropagation();
    if (dropZoneRef.current && !dropZoneRef.current.contains(e.relatedTarget as Node)) setIsDragOver(false);
  }, []);
  const handleDragOver = useCallback((e: React.DragEvent) => { e.preventDefault(); e.stopPropagation(); }, []);
  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault(); e.stopPropagation(); setIsDragOver(false);
    const files = Array.from(e.dataTransfer.files);
    if (files.length === 0) return;
    const filePaths = files.map((f: any) => f.path).filter(Boolean);
    if (filePaths.length === 0) return;
    setBusy(true); setError(null);
    try {
      for (const fp of filePaths) {
        const ext = fp.split('.').pop()?.toLowerCase();
        if (!ext || !SUPPORTED_IMAGE_FORMATS.includes(ext)) { setError("Unsupported: ." + ext); continue; }
        setSelectedFileName(fp.split('\\').pop() || fp);
        await imageApi.uploadImage(activeProject.id, fp);
      }
      await loadImages(); setTab("browse");
    } catch (err) { setError(String(err)); }
    finally { setBusy(false); setSelectedFileName(null); }
  }, [activeProject]);

  const handleAnalyze = async (image: ProjectImage) => {
    setError(null); setAnalyzingId(image.id);
    try { const res = await imageApi.analyzeImage(activeProject.id, image.id); setResult(res); setSelectedImage(image); }
    catch (err) { setError(String(err)); }
    finally { setAnalyzingId(null); }
  };

  const handleReview = async (proposal: IssueProposal, status: "approved" | "rejected") => {
    try {
      await imageApi.updateIssueReview(proposal.id, status);
      if (result) { setResult(await imageApi.analyzeImage(activeProject.id, result.analysis.image_id)); }
    } catch (err) { setError(String(err)); }
  };

  return (
    <div className="page image-analysis video-page">
      <div className="page-header">
        <h2>Image Analysis</h2>
        <span className="project-context">{activeProject.name}</span>
      </div>

      <div className="va-tabs">
        <button className={"va-tab " + (tab === "upload" ? "active" : "")} onClick={() => setTab("upload")}>Upload</button>
        <button className={"va-tab " + (tab === "browse" ? "active" : "")} onClick={() => setTab("browse")}>Images ({images.length})</button>
      </div>

      {error && <div className="va-error">{error}</div>}

      {tab === "upload" && (
        <div className="image-upload-section">
          <div
            ref={dropZoneRef}
            className={"image-drop-zone " + (isDragOver ? "image-drop-zone-active" : "")}
            onDragEnter={handleDragEnter}
            onDragLeave={handleDragLeave}
            onDragOver={handleDragOver}
            onDrop={handleDrop}
            onClick={handleUpload}
          >
            <div className="image-drop-content">
              {busy ? (
                <>
                  <div className="loading-spinner" style={{ width: 32, height: 32 }} />
                  <p className="image-drop-text">Uploading images...</p>
                  {selectedFileName && <p className="image-drop-subtext">{selectedFileName}</p>}
                </>
              ) : (
                <>
                  <div className="image-drop-icon">&#128247;</div>
                  <p className="image-drop-text">{isDragOver ? "Drop images here" : "Drag & drop images here"}</p>
                  <p className="image-drop-subtext">or click to browse files</p>
                  <div className="image-formats-list">
                    {SUPPORTED_IMAGE_FORMATS.map(fmt => (
                      <span key={fmt} className="image-format-badge">.{fmt}</span>
                    ))}
                  </div>
                </>
              )}
            </div>
          </div>

          <div className="image-supported-info">
            <h3>Supported Formats</h3>
            <div className="image-formats-grid">
              {Object.entries(IMAGE_FORMATS).map(([ext, label]) => (
                <div key={ext} className="image-format-item">
                  <span className="image-format-ext">.{ext}</span>
                  <span className="image-format-label">{label}</span>
                </div>
              ))}
            </div>
            <div className="image-max-info">
              <span className="image-max-label">Maximum size:</span>
              <span className="image-max-badge">50 MB</span>
            </div>
          </div>
        </div>
      )}

      {tab === "browse" && (
        <div className="va-section">
          <div className="va-layout">
            <div className="va-col">
              <h3>Images ({images.length})</h3>
              {images.length === 0 && (
                <div className="image-empty-state" >
                  <div className="video-empty-icon">&#128247;</div>
                  <p>No images uploaded yet</p>
                  <button className="btn btn-primary" onClick={() => setTab("upload")}>Upload Your First Image</button>
                </div>
              )}
              <ul className="va-list">
                {images.map((img) => (
                  <li key={img.id} className={"va-list-item " + (selectedImage?.id === img.id ? "selected" : "")}
                    onClick={() => { setSelectedImage(img); setResult(null); }}>
                    <strong>{img.filename}</strong>
                    <span className="va-meta">{img.width}x{img.height}</span>
                  </li>
                ))}
              </ul>
            </div>

            {selectedImage && (
              <div className="va-col">
                <h3>Image Info</h3>
                <div className="va-info">
                  <p><strong>File:</strong> {selectedImage.filename}</p>
                  <p><strong>Resolution:</strong> {selectedImage.width}x{selectedImage.height}</p>
                  <p><strong>Size:</strong> {(selectedImage.file_size / 1024).toFixed(1)} KB</p>
                </div>
                <div className="mt-3">
                  <button className="btn btn-primary" onClick={() => handleAnalyze(selectedImage)} disabled={analyzingId === selectedImage.id}>
                    {analyzingId === selectedImage.id ? "Analyzing..." : "Analyze Image"}
                  </button>
                </div>
                {selectedImage.file_path && (
                  <div className="mt-3">
                    <img src={convertFileSrc(selectedImage.file_path)} alt={selectedImage.filename} className="image-thumb" style={{ maxHeight: 200 }} />
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      )}

      {result && selectedImage && (
        <div className="analysis-result">
          <div className="analysis-header">
            <h3>Analysis Result</h3>
            <span className={"analysis-status status-" + result.analysis.status}>{result.analysis.status}</span>
            <span className="meta-tag">{result.proposals.length} proposals</span>
            <button className="btn-secondary" onClick={() => onNavigate("issues")}>View All Issues</button>
          </div>

          {result.terminology_findings.length > 0 && (
            <div className="terminology-findings">
              <h4>Terminology Consistency</h4>
              {result.terminology_findings.map((f: TerminologyFinding, i) => (
                <div className={"finding finding-" + f.kind} key={i}>
                  <span className="finding-kind">{f.kind}</span>
                  <span className="finding-note">{f.note}</span>
                </div>
              ))}
            </div>
          )}

          <div className="proposal-list">
            {result.proposals.length === 0 && <p className="empty-description">No issues proposed for this image.</p>}
            {result.proposals.map((p) => (
              <div className="proposal-card" key={p.id}>
                <div className="proposal-top">
                  <span className={"sev-badge " + severityClass(p.severity_candidate)}>{p.severity_candidate}</span>
                  <span className="proposal-bugtype">{p.bug_type}</span>
                  <span className={"review-badge " + reviewClass(p.review_status)}>{p.review_status}</span>
                </div>
                <p className="proposal-title">{p.title}</p>
                <p className="proposal-desc">{p.description}</p>
                {p.detected_text && <p className="proposal-detected">Detected: “{p.detected_text}”</p>}
                <p className="proposal-confidence">Confidence: {(p.confidence * 100).toFixed(0)}%</p>
                {p.severity_reasoning && <p className="proposal-reasoning">{p.severity_reasoning}</p>}
                {p.glossary_findings && <p className="proposal-glossary">Glossary: {p.glossary_findings}</p>}
                <div className="proposal-actions">
                  <button className="btn-secondary" onClick={() => handleReview(p, "approved")}>Approve</button>
                  <button className="btn-secondary" onClick={() => handleReview(p, "rejected")}>Reject</button>
                  <button className="btn-secondary" onClick={() => setEvidenceIssueId(evidenceIssueId === p.id ? null : p.id)}>
                    {evidenceIssueId === p.id ? "Hide Evidence" : "Evidence"}
                  </button>
                </div>
              </div>
            ))}
          </div>

          {evidenceIssueId && (() => {
            const evidenceIssue = result.proposals.find((p) => p.id === evidenceIssueId);
            const evidenceImage = images.find((img) => img.id === evidenceIssue?.image_id);
            if (!evidenceIssue || !evidenceImage) return null;
            return (
              <EvidenceEditor
                projectId={activeProject.id}
                issue={evidenceIssue}
                imageSrc={convertFileSrc(evidenceImage.file_path)}
                imageWidth={evidenceImage.width}
                imageHeight={evidenceImage.height}
                onClose={() => setEvidenceIssueId(null)}
              />
            );
          })()}
        </div>
      )}
    </div>
  );
}