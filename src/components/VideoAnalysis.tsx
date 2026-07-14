import { useState, useEffect, useCallback, useRef } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { Project, ProjectVideo, VideoExtraction, VideoFrame, IssueProposal, ImageAnalysisResult, TerminologyFinding } from "../types";
import { videoApi, imageApi, evidenceApi } from "../api/tauri";
import EvidenceEditor from "./EvidenceEditor";
import { useToast } from "./Toast";

interface Props { activeProject: Project | null; }
type Tab = "upload" | "browse" | "frames";
const MIN_INTERVAL = 1; const MAX_INTERVAL = 30;
const SUPPORTED_VIDEO_FORMATS = ["mp4", "mov", "avi", "mkv", "webm"];
const VIDEO_FORMAT_EXTENSIONS: Record<string, string> = {
  mp4: "MP4 (H.264/H.265)", mov: "QuickTime (MOV)", avi: "AVI (Audio Video Interleave)",
  mkv: "MKV (Matroska Video)", webm: "WebM (VP8/VP9)",
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

export default function VideoAnalysis({ activeProject }: Props) {
  const [tab, setTab] = useState<Tab>("upload");
  const [videos, setVideos] = useState<ProjectVideo[]>([]);
  const [selectedVideo, setSelectedVideo] = useState<ProjectVideo | null>(null);
  const [extractions, setExtractions] = useState<VideoExtraction[]>([]);
  const [selectedExtraction, setSelectedExtraction] = useState<VideoExtraction | null>(null);
  const [frames, setFrames] = useState<VideoFrame[]>([]);
  const [selectedFrame, setSelectedFrame] = useState<VideoFrame | null>(null);
  const [interval, setInterval] = useState(2);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [uploadPath, setUploadPath] = useState("");
  const [uploadingVideo, setUploadingVideo] = useState(false);
  const [analyzingFrameId, setAnalyzingFrameId] = useState<string | null>(null);
  const [analysisResult, setAnalysisResult] = useState<ImageAnalysisResult | null>(null);
  const [issues, setIssues] = useState<IssueProposal[]>([]);
  const [selectedIssue, setSelectedIssue] = useState<IssueProposal | null>(null);
  const [crossFrameFindings, setCrossFrameFindings] = useState<TerminologyFinding[]>([]);
  const [isDragOver, setIsDragOver] = useState(false);
  const [selectedFileName, setSelectedFileName] = useState<string | null>(null);
  const dropZoneRef = useRef<HTMLDivElement>(null);
  const toast = useToast();

  const loadVideos = useCallback(async () => {
    if (!activeProject) return;
    try { setError(null); const list = await videoApi.listVideos(activeProject.id); setVideos(list); }
    catch (e) { setError(String(e)); }
  }, [activeProject]);

  useEffect(() => { loadVideos(); }, [loadVideos]);
  useEffect(() => {
    setSelectedVideo(null); setExtractions([]); setSelectedExtraction(null);
    setFrames([]); setSelectedFrame(null); setAnalysisResult(null);
    setIssues([]); setSelectedIssue(null); setCrossFrameFindings([]);
    setSelectedFileName(null); setUploadPath("");
  }, [activeProject]);

  const loadExtractions = async (videoId: string) => {
    try { setError(null); const list = await videoApi.listExtractions(videoId); setExtractions(list); }
    catch (e) { setError(String(e)); }
  };
  const loadFrames = async (extractionId: string) => {
    try { setError(null); const list = await videoApi.listFrames(extractionId); setFrames(list); }
    catch (e) { setError(String(e)); }
  };

  const validateVideoFile = (filePath: string): string | null => {
    const ext = filePath.split('.').pop()?.toLowerCase();
    if (!ext || !SUPPORTED_VIDEO_FORMATS.includes(ext)) {
      return `Unsupported format: .${ext || 'unknown'}. Supported: ${SUPPORTED_VIDEO_FORMATS.join(', ')}`;
    }
    return null;
  };

  const handleUploadFile = async (filePath: string) => {
    if (!activeProject) return;
    const validationError = validateVideoFile(filePath);
    if (validationError) { setError(validationError); toast.addToast("error", "Invalid Format", validationError); return; }
    setUploadingVideo(true); setError(null);
    setSelectedFileName(filePath.split('\\').pop() || filePath.split('/').pop() || filePath);
    try {
      const video = await videoApi.uploadVideo(activeProject.id, filePath);
      setVideos(prev => [video, ...prev]);
      setSelectedVideo(video); setTab("browse");
      toast.addToast("success", "Video Uploaded", `${video.original_filename} uploaded successfully`);
      setUploadPath(""); setSelectedFileName(null);
    } catch (err) { const msg = String(err); setError(msg); toast.addToast("error", "Upload Failed", msg); }
    finally { setUploadingVideo(false); }
  };

  const handleUpload = async () => {
    if (!activeProject) return;
    setError(null);
    try {
      const selected = await open({ multiple: false, filters: [{ name: "Videos", extensions: SUPPORTED_VIDEO_FORMATS }] });
      if (!selected) return;
      const filePath = Array.isArray(selected) ? selected[0] : selected;
      await handleUploadFile(filePath);
    } catch (err) { const msg = String(err); setError(msg); toast.addToast("error", "Upload Failed", msg); }
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
    const filePath = (files[0] as any).path;
    if (!filePath) { toast.addToast("error", "Upload Error", "Could not determine file path. Use the upload button."); return; }
    await handleUploadFile(filePath);
  }, [activeProject]);

  const handleManualPathUpload = async () => {
    if (!activeProject || !uploadPath.trim()) return;
    await handleUploadFile(uploadPath.trim());
  };

  const handleSelectVideo = async (video: ProjectVideo) => {
    setSelectedVideo(video); setSelectedExtraction(null); setFrames([]); setSelectedFrame(null); setAnalysisResult(null); setIssues([]); setSelectedIssue(null); setCrossFrameFindings([]);
    await loadExtractions(video.id);
  };

  const handleExtract = async () => {
    if (!activeProject || !selectedVideo) return;
    setLoading(true); setError(null);
    try {
      const extraction = await videoApi.extractFrames(activeProject.id, selectedVideo.id, interval);
      setExtractions((prev) => [extraction, ...prev]); setSelectedExtraction(extraction);
      await loadFrames(extraction.id);
    } catch (e) { setError(String(e)); }
    finally { setLoading(false); }
  };

  const handleSelectExtraction = async (ext: VideoExtraction) => {
    setSelectedExtraction(ext); setSelectedFrame(null); setAnalysisResult(null); setIssues([]); setSelectedIssue(null); setCrossFrameFindings([]);
    await loadFrames(ext.id);
  };

  const handleSelectFrame = (frame: VideoFrame) => {
    setSelectedFrame(frame); setAnalysisResult(null); setIssues([]); setSelectedIssue(null);
  };

  const handleAnalyzeFrame = async () => {
    if (!activeProject || !selectedFrame) return;
    setAnalyzingFrameId(selectedFrame.id); setError(null); setAnalysisResult(null);
    try {
      const result = await videoApi.analyzeFrame(activeProject.id, selectedFrame.id);
      setAnalysisResult(result);
      setIssues(result.proposals);
    } catch (e) { setError(String(e)); }
    finally { setAnalyzingFrameId(null); }
  };

  const handleCrossFrameConsistency = async () => {
    if (!activeProject || frames.length < 2) return;
    setError(null);
    try {
      const frameIds = frames.map(f => f.id);
      const findings = await videoApi.crossFrameConsistency(activeProject.id, frameIds);
      setCrossFrameFindings(findings);
    } catch (e) { setError(String(e)); }
  };

  const fmtDur = (s: number) => `${Math.floor(s/60)}:${Math.floor(s%60).toString().padStart(2,"0")}`;
  const fmtSize = (b: number) => b < 1_048_576 ? `${(b/1024).toFixed(1)} KB` : `${(b/1_048_576).toFixed(1)} MB`;

  if (!activeProject) {
    return (
      <div className="page video-page">
        <div className="video-empty-state">
          <div className="video-empty-icon">&#127916;</div>
          <h2>No Project Selected</h2>
          <p>Select or create a project to upload and analyze video files.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="page video-page">
      <div className="page-header">
        <h2>Video Analysis</h2>
        <span className="project-context">{activeProject.name}</span>
      </div>

      <div className="va-tabs">
        <button className={`va-tab ${tab === "upload" ? "active" : ""}`} onClick={() => setTab("upload")}>Upload</button>
        <button className={`va-tab ${tab === "browse" ? "active" : ""}`} onClick={() => setTab("browse")}>Videos ({videos.length})</button>
        <button className={`va-tab ${tab === "frames" ? "active" : ""}`} onClick={() => setTab("frames")} disabled={!selectedExtraction}>Frames</button>
      </div>

      {error && <div className="va-error">{error}</div>}

      {tab === "upload" && (
        <div className="video-upload-section">
          <div
            ref={dropZoneRef}
            className={`video-drop-zone ${isDragOver ? "video-drop-zone-active" : ""}`}
            onDragEnter={handleDragEnter}
            onDragLeave={handleDragLeave}
            onDragOver={handleDragOver}
            onDrop={handleDrop}
            onClick={handleUpload}
          >
            <div className="video-drop-content">
              {uploadingVideo ? (
                <>
                  <div className="loading-spinner video-spinner" />
                  <p className="video-drop-text">Uploading video...</p>
                  {selectedFileName && <p className="video-drop-subtext">{selectedFileName}</p>}
                </>
              ) : (
                <>
                  <div className="video-drop-icon">&#127916;</div>
                  <p className="video-drop-text">
                    {isDragOver ? "Drop video here" : "Drag & drop a video file here"}
                  </p>
                  <p className="video-drop-subtext">or click to browse files</p>
                  <div className="video-formats-list">
                    {SUPPORTED_VIDEO_FORMATS.map(fmt => (
                      <span key={fmt} className="video-format-badge">.{fmt}</span>
                    ))}
                  </div>
                </>
              )}
            </div>
          </div>

          <div className="video-supported-info">
            <h3>Supported Formats</h3>
            <div className="video-formats-grid">
              {Object.entries(VIDEO_FORMAT_EXTENSIONS).map(([ext, label]) => (
                <div key={ext} className="video-format-item">
                  <span className="video-format-ext">.{ext}</span>
                  <span className="video-format-label">{label}</span>
                </div>
              ))}
            </div>
            <div className="video-ffmpeg-status">
              <span className="video-ffmpeg-label">FFmpeg Status:</span>
              <span className="video-ffmpeg-badge">Available</span>
            </div>
          </div>

          <div className="video-manual-input">
            <input type="text" className="settings-input" placeholder="Or enter video file path manually..."
              value={uploadPath} onChange={e => setUploadPath(e.target.value)}
              onKeyDown={e => e.key === "Enter" && handleManualPathUpload()} />
            <button className="btn btn-secondary" onClick={handleManualPathUpload} disabled={!uploadPath.trim() || uploadingVideo}>Upload</button>
          </div>
        </div>
      )}

      {tab === "browse" && (
        <div className="va-section">
          <div className="va-layout">
            <div className="va-col">
              <h3>Videos ({videos.length})</h3>
              {videos.length === 0 && (
                <div className="video-empty-state">
                  <div className="video-empty-icon">&#127916;</div>
                  <p>No videos uploaded yet</p>
                  <button className="btn btn-primary" onClick={() => setTab("upload")}>Upload Your First Video</button>
                </div>
              )}
              <ul className="va-list">
                {videos.map((v) => (
                  <li key={v.id} className={`va-list-item ${selectedVideo?.id === v.id ? "selected" : ""}`} onClick={() => handleSelectVideo(v)}>
                    <strong>{v.original_filename}</strong>
                    <span className="va-meta">{v.width}x{v.height} - {fmtDur(v.duration_secs)} - {fmtSize(v.file_size)}</span>
                  </li>
                ))}
              </ul>
            </div>

            {selectedVideo && (
              <div className="va-col">
                <h3>Video Info</h3>
                <div className="va-info">
                  <p><strong>File:</strong> {selectedVideo.original_filename}</p>
                  <p><strong>Resolution:</strong> {selectedVideo.width}x{selectedVideo.height}</p>
                  <p><strong>Duration:</strong> {fmtDur(selectedVideo.duration_secs)} ({selectedVideo.duration_secs}s)</p>
                  <p><strong>Size:</strong> {fmtSize(selectedVideo.file_size)}</p>
                  <p><strong>FPS:</strong> {selectedVideo.frame_rate}</p>
                </div>

                <div className="va-interval mt-3">
                  <label>Interval (s):</label>
                  <input type="range" min={MIN_INTERVAL} max={MAX_INTERVAL} value={interval} onChange={e => setInterval(parseInt(e.target.value))} />
                  <span>{interval}s</span>
                </div>
                <button className="btn btn-primary" onClick={handleExtract} disabled={loading}>{loading ? "Extracting..." : "Extract Frames"}</button>

                <h3 className="mt-4">Extractions</h3>
                {extractions.length === 0 && <p className="va-hint">No extractions yet.</p>}
                <ul className="va-list">
                  {extractions.map((ex) => (
                    <li key={ex.id} className={`va-list-item ${selectedExtraction?.id === ex.id ? "selected" : ""}`} onClick={() => handleSelectExtraction(ex)}>
                      <strong>{ex.extraction_strategy}</strong>
                      <span className="va-meta">{ex.interval_secs}s - {ex.total_frames} frames - {ex.status}</span>
                      {ex.error_message && <span className="va-error">{ex.error_message}</span>}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>
      )}

      {tab === "frames" && selectedExtraction && (
        <div className="va-section">
          <h3>Frames ({frames.length})</h3>
          <div className="va-frame-grid">
            {frames.map((f) => (
              <div key={f.id} className={`va-frame-card ${selectedFrame?.id === f.id ? "selected" : ""}`} onClick={() => handleSelectFrame(f)}>
                {f.image_path ? (
                  <img src={convertFileSrc(f.image_path)} alt={`Frame ${f.frame_index}`} className="va-frame-thumb" />
                ) : (
                  <div className="va-frame-thumb-placeholder">{f.frame_index}</div>
                )}
                <div className="va-frame-label">
                  <span>#{f.frame_index}</span>
                  <span>{f.timestamp_display}</span>
                </div>
              </div>
            ))}
          </div>
          {frames.length === 0 && <p className="va-hint">No frames extracted yet.</p>}

          <div className="mt-3">
            <button className="btn btn-secondary" onClick={handleCrossFrameConsistency} disabled={frames.length < 2}>Check Cross-Frame Consistency</button>
            {crossFrameFindings.length > 0 && (
              <div className="va-issues mt-2">
                <h4>Cross-Frame Findings ({crossFrameFindings.length})</h4>
                <ul className="va-list">
                  {crossFrameFindings.map((f, i) => (
                    <li key={i} className="va-list-item">
                      <strong>{f.kind}</strong>: {f.note}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>
      )}

      {selectedFrame && (
        <div className="va-preview">
          <h3>Selected Frame: #{selectedFrame.frame_index} @ {selectedFrame.timestamp_display}</h3>
          <p className="va-meta">Resolution: {selectedFrame.width}x{selectedFrame.height} - Hash: {selectedFrame.content_hash.substring(0, 16)}...</p>
          {selectedFrame.image_path && (
            <img src={convertFileSrc(selectedFrame.image_path)} alt={`Frame ${selectedFrame.frame_index}`} className="va-preview-img" />
          )}
          <div className="mt-3">
            <button className="btn btn-primary" onClick={handleAnalyzeFrame} disabled={analyzingFrameId === selectedFrame.id}>
              {analyzingFrameId === selectedFrame.id ? "Analyzing..." : "Analyze Frame"}
            </button>
          </div>

          {analysisResult && (
            <div className="va-analysis-result mt-4">
              <h4>Analysis Result</h4>
              <p>Status: <strong>{analysisResult.analysis.status}</strong> | Issues: {analysisResult.proposals.length}</p>
            </div>
          )}

          {issues.length > 0 && (
            <div className="va-issues mt-4">
              <h4>Issue Proposals ({issues.length})</h4>
              <ul className="va-list">
                {issues.map((issue) => (
                  <li key={issue.id} className={`va-list-item ${selectedIssue?.id === issue.id ? "selected" : ""}`} onClick={() => setSelectedIssue(issue)}>
                    <span className={`severity-tag ${severityClass(issue.severity_candidate)}`}>{issue.severity_candidate}</span>
                    <strong>{issue.title}</strong>
                    <span className={`review-tag ${reviewClass(issue.review_status)}`}>{issue.review_status}</span>
                    <span className="va-meta">{issue.bug_type}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {selectedIssue && (
            <div className="va-evidence mt-4">
              <h4>Evidence: {selectedIssue.title}</h4>
              <EvidenceEditor
                projectId={activeProject.id}
                issue={selectedIssue}
                imageSrc={selectedFrame.image_path ? convertFileSrc(selectedFrame.image_path) : ''}
                imageWidth={selectedFrame.width}
                imageHeight={selectedFrame.height}
                onClose={() => setSelectedIssue(null)} />
            </div>
          )}
        </div>
      )}
    </div>
  );
}
