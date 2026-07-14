import { useState, useEffect } from "react";
import { evidenceApi } from "../api/tauri";
import type { EvidenceAnnotation, EvidenceCandidate, IssueProposal } from "../types";
import EvidenceCanvas from "./EvidenceCanvas";

interface Props {
  projectId: string;
  issue: IssueProposal;
  imageSrc: string;
  imageWidth: number;
  imageHeight: number;
  onClose: () => void;
}

export default function EvidenceEditor({ projectId, issue, imageSrc, imageWidth, imageHeight, onClose }: Props) {
  const [annotations, setAnnotations] = useState<EvidenceAnnotation[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [annotationType, setAnnotationType] = useState("RED_BOX");

  const loadAnnotations = async () => {
    try {
      const list = await evidenceApi.listByIssue(projectId, issue.id);
      setAnnotations(list);
      if (list.length > 0 && !selectedId) setSelectedId(list[0].id);
    } catch (e) { setError(String(e)); }
  };

  useEffect(() => { loadAnnotations(); }, [projectId, issue.id]);

  const handleGenerate = async () => {
    setLoading(true); setError(null);
    try {
      const candidates: EvidenceCandidate[] = await evidenceApi.generateProposals(projectId, issue.id);
      for (const c of candidates) {
        await evidenceApi.createAnnotation(projectId, issue.id, c.annotation_type, c.x, c.y, c.width, c.height, c.origin);
      }
      await loadAnnotations();
    } catch (e) { setError(String(e)); }
    setLoading(false);
  };

  const handleCreate = async () => {
    try {
      await evidenceApi.createAnnotation(projectId, issue.id, annotationType, 0.2, 0.2, 0.3, 0.3, "USER_CREATED");
      await loadAnnotations();
    } catch (e) { setError(String(e)); }
  };

  const handleUpdate = async (id: string, x: number, y: number, width: number, height: number) => {
    const ann = annotations.find((a) => a.id === id);
    if (!ann) return;
    try {
      await evidenceApi.updateAnnotation(projectId, id, ann.annotation_type, x, y, width, height);
      setAnnotations((prev) => prev.map((a) => (a.id === id ? { ...a, x, y, width, height } : a)));
    } catch (e) { setError(String(e)); }
  };

  const handleReview = async (id: string, state: string) => {
    try {
      await evidenceApi.updateReview(projectId, id, state);
      setAnnotations((prev) => prev.map((a) => (a.id === id ? { ...a, review_state: state } : a)));
    } catch (e) { setError(String(e)); }
  };

  const selected = annotations.find((a) => a.id === selectedId);
  return (
    <div className="evidence-editor" style={{ padding: 12, border: "1px solid #333", borderRadius: 6, background: "#1a1a1a", marginTop: 12 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
        <h3 style={{ margin: 0, fontSize: 14 }}>Evidence: {issue.title} ({annotations.length} annotation{annotations.length !== 1 ? "s" : ""})</h3>
        <button onClick={onClose} style={{ background: "none", border: "none", color: "#aaa", cursor: "pointer", fontSize: 16 }}>&#x2715;</button>
      </div>
      {error && <div style={{ color: "#ff4444", marginBottom: 8, fontSize: 12 }}>{error}</div>}
      <div style={{ marginBottom: 8, display: "flex", gap: 8, flexWrap: "wrap" }}>
        <button onClick={handleGenerate} disabled={loading} style={{ padding: "4px 12px", background: "#2a2a2a", color: "#ccc", border: "1px solid #555", borderRadius: 4, cursor: "pointer" }}>{loading ? "Generating..." : "Generate Proposals"}</button>
        <select value={annotationType} onChange={(e) => setAnnotationType(e.target.value)} style={{ padding: "4px 8px", background: "#2a2a2a", color: "#ccc", border: "1px solid #555", borderRadius: 4 }}>
          <option value="RED_BOX">Red Box</option>
          <option value="RED_BRACKET">Red Bracket</option>
        </select>
        <button onClick={handleCreate} style={{ padding: "4px 12px", background: "#2a2a2a", color: "#ccc", border: "1px solid #555", borderRadius: 4, cursor: "pointer" }}>+ New</button>
      </div>
      <EvidenceCanvas imageSrc={imageSrc} imageWidth={imageWidth} imageHeight={imageHeight} annotations={annotations} selectedAnnotationId={selectedId} onSelectAnnotation={setSelectedId} onUpdateAnnotation={handleUpdate} />
      {selected && (
        <div style={{ marginTop: 8, display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center", fontSize: 12 }}>
          <span style={{ color: "#888" }}>{selected.origin.replace("_", " ")} | {selected.annotation_type.replace("_", " ")}</span>
          <span style={{ color: selected.review_state === "APPROVED" ? "#4caf50" : selected.review_state === "REJECTED" ? "#ff4444" : "#ffaa00" }}>{selected.review_state}</span>
          <button onClick={() => handleReview(selected.id, "APPROVED")} disabled={selected.review_state === "APPROVED"} style={{ padding: "2px 10px", background: "#1b5e20", color: "#4caf50", border: "1px solid #2e7d32", borderRadius: 3, cursor: "pointer" }}>Approve</button>
          <button onClick={() => handleReview(selected.id, "REJECTED")} disabled={selected.review_state === "REJECTED"} style={{ padding: "2px 10px", background: "#4a0000", color: "#ff4444", border: "1px solid #7f0000", borderRadius: 3, cursor: "pointer" }}>Reject</button>
          <button onClick={() => handleReview(selected.id, "PROPOSED")} style={{ padding: "2px 10px", background: "#2a2a2a", color: "#ffaa00", border: "1px solid #555", borderRadius: 3, cursor: "pointer" }}>Reset</button>
        </div>
      )}
      {annotations.length === 0 && !loading && (
        <div style={{ color: "#666", fontSize: 12, marginTop: 8 }}>No evidence annotations. Click "Generate Proposals" or "+ New" to create.</div>
      )}
    </div>
  );
}
