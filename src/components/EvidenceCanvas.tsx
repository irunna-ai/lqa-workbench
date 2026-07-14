import { useState, useEffect, useRef, useCallback } from "react";
import type { EvidenceAnnotation } from "../types";

interface Props {
  imageSrc: string;
  imageWidth: number;
  imageHeight: number;
  annotations: EvidenceAnnotation[];
  selectedAnnotationId: string | null;
  onSelectAnnotation: (id: string) => void;
  onUpdateAnnotation: (id: string, x: number, y: number, width: number, height: number) => void;
  readOnly?: boolean;
}

export default function EvidenceCanvas({
  imageSrc,
  imageWidth,
  imageHeight,
  annotations,
  selectedAnnotationId,
  onSelectAnnotation,
  onUpdateAnnotation,
  readOnly = false,
}: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [renderedSize, setRenderedSize] = useState({ width: 0, height: 0, offsetX: 0, offsetY: 0 });
  const [dragging, setDragging] = useState<{
    annotationId: string;
    handle: "move" | "tl" | "tr" | "bl" | "br";
    startX: number; startY: number;
    startBox: { x: number; y: number; width: number; height: number };
  } | null>(null);

  const updateRenderedSize = useCallback(() => {
    const el = containerRef.current?.querySelector<HTMLImageElement>(".evidence-image");
    if (!el || !imageWidth || !imageHeight) return;
    const rect = el.getBoundingClientRect();
    const containerRect = containerRef.current!.getBoundingClientRect();
    setRenderedSize({
      width: rect.width,
      height: rect.height,
      offsetX: rect.left - containerRect.left,
      offsetY: rect.top - containerRect.top,
    });
  }, [imageWidth, imageHeight]);

  useEffect(() => {
    updateRenderedSize();
    window.addEventListener("resize", updateRenderedSize);
    return () => window.removeEventListener("resize", updateRenderedSize);
  }, [updateRenderedSize]);

  const normToPixel = (ann: EvidenceAnnotation) => ({
    left: ann.x * renderedSize.width + renderedSize.offsetX,
    top: ann.y * renderedSize.height + renderedSize.offsetY,
    width: ann.width * renderedSize.width,
    height: ann.height * renderedSize.height,
  });

  const handlePointerDown = (e: React.PointerEvent, annotationId: string, handle: "move" | "tl" | "tr" | "bl" | "br") => {
    if (readOnly) return;
    e.stopPropagation();
    e.preventDefault();
    const ann = annotations.find((a) => a.id === annotationId);
    if (!ann) return;
    setDragging({
      annotationId, handle,
      startX: e.clientX, startY: e.clientY,
      startBox: { x: ann.x, y: ann.y, width: ann.width, height: ann.height },
    });
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  };

  const handlePointerMove = (e: React.PointerEvent) => {
    if (!dragging || !renderedSize.width) return;
    const dx = (e.clientX - dragging.startX) / renderedSize.width;
    const dy = (e.clientY - dragging.startY) / renderedSize.height;
    let { x, y, width, height } = dragging.startBox;
    const minSize = 0.02;
    if (dragging.handle === "move") {
      x = Math.max(0, Math.min(1 - width, x + dx));
      y = Math.max(0, Math.min(1 - height, y + dy));
    } else if (dragging.handle === "tl") {
      const nx = Math.max(0, Math.min(x + width - minSize, x + dx));
      const ny = Math.max(0, Math.min(y + height - minSize, y + dy));
      width += x - nx; height += y - ny; x = nx; y = ny;
    } else if (dragging.handle === "tr") {
      width = Math.max(minSize, Math.min(1 - x, width + dx));
      const ny = Math.max(0, Math.min(y + height - minSize, y + dy));
      height += y - ny; y = ny;
    } else if (dragging.handle === "bl") {
      const nx = Math.max(0, Math.min(x + width - minSize, x + dx));
      width += x - nx; x = nx;
      height = Math.max(minSize, Math.min(1 - y, height + dy));
    } else if (dragging.handle === "br") {
      width = Math.max(minSize, Math.min(1 - x, width + dx));
      height = Math.max(minSize, Math.min(1 - y, height + dy));
    }
    onUpdateAnnotation(dragging.annotationId, x, y, width, height);
  };

  const handlePointerUp = (e: React.PointerEvent) => {
    if (!dragging) return;
    (e.target as HTMLElement).releasePointerCapture(e.pointerId);
    setDragging(null);
  };
  return (
    <div ref={containerRef} className="evidence-canvas" style={{ position: "relative", display: "inline-block", maxWidth: "100%" }}>
      <img
        src={imageSrc}
        className="evidence-image"
        style={{ maxWidth: "100%", display: "block", userSelect: "none" }}
        onLoad={updateRenderedSize}
        draggable={false}
        alt="evidence"
      />
      {annotations.map((ann) => {
        const px = normToPixel(ann);
        const isSelected = ann.id === selectedAnnotationId;
        const isRedBox = ann.annotation_type === "RED_BOX";
        const isRedBracket = ann.annotation_type === "RED_BRACKET";
        const handleSize = 8;
        return (
          <div
            key={ann.id}
            onPointerDown={(e) => { onSelectAnnotation(ann.id); }}
            style={{ position: "absolute", left: px.left, top: px.top, width: px.width, height: px.height, cursor: readOnly ? "default" : "move", pointerEvents: "auto" }}
          >
            {isRedBox && (
              <div style={{ position: "absolute", inset: 0, border: `2px solid ${isSelected ? "#ff0000" : "#cc0000"}`, background: "rgba(255,0,0,0.05)", pointerEvents: "none" }} />
            )}
            {isRedBracket && (
              <>
                <div style={{ position: "absolute", left: 0, top: 0, width: 12, height: 2, background: "#ff0000", pointerEvents: "none" }} />
                <div style={{ position: "absolute", left: 0, top: 0, width: 2, height: 12, background: "#ff0000", pointerEvents: "none" }} />
                <div style={{ position: "absolute", right: 0, top: 0, width: 12, height: 2, background: "#ff0000", pointerEvents: "none" }} />
                <div style={{ position: "absolute", right: 0, top: 0, width: 2, height: 12, background: "#ff0000", pointerEvents: "none" }} />
                <div style={{ position: "absolute", left: 0, bottom: 0, width: 12, height: 2, background: "#ff0000", pointerEvents: "none" }} />
                <div style={{ position: "absolute", left: 0, bottom: 0, width: 2, height: 12, background: "#ff0000", pointerEvents: "none" }} />
                <div style={{ position: "absolute", right: 0, bottom: 0, width: 12, height: 2, background: "#ff0000", pointerEvents: "none" }} />
                <div style={{ position: "absolute", right: 0, bottom: 0, width: 2, height: 12, background: "#ff0000", pointerEvents: "none" }} />
              </>
            )}
            {isSelected && !readOnly && (
              <>
                <div onPointerDown={(e) => handlePointerDown(e, ann.id, "tl")} style={{ position: "absolute", left: -handleSize/2, top: -handleSize/2, width: handleSize, height: handleSize, background: "#ff0000", cursor: "nw-resize", borderRadius: "50%" }} />
                <div onPointerDown={(e) => handlePointerDown(e, ann.id, "tr")} style={{ position: "absolute", right: -handleSize/2, top: -handleSize/2, width: handleSize, height: handleSize, background: "#ff0000", cursor: "ne-resize", borderRadius: "50%" }} />
                <div onPointerDown={(e) => handlePointerDown(e, ann.id, "bl")} style={{ position: "absolute", left: -handleSize/2, bottom: -handleSize/2, width: handleSize, height: handleSize, background: "#ff0000", cursor: "sw-resize", borderRadius: "50%" }} />
                <div onPointerDown={(e) => handlePointerDown(e, ann.id, "br")} style={{ position: "absolute", right: -handleSize/2, bottom: -handleSize/2, width: handleSize, height: handleSize, background: "#ff0000", cursor: "se-resize", borderRadius: "50%" }} />
              </>
            )}
            <div style={{ position: "absolute", top: -16, left: 0, fontSize: 10, color: "#ff4444", whiteSpace: "nowrap", pointerEvents: "none", background: "rgba(0,0,0,0.6)", padding: "1px 4px", borderRadius: 2 }}>
              {ann.origin.replace("_", " ")}
            </div>
          </div>
        );
      })}
      {dragging && (
        <div onPointerMove={handlePointerMove} onPointerUp={handlePointerUp} style={{ position: "fixed", inset: 0, zIndex: 9999, cursor: "crosshair" }} />
      )}
    </div>
  );
}
