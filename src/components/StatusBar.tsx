import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface StatusItem {
  label: string;
  status: "ok" | "warn" | "err" | "unknown";
}

export default function StatusBar() {
  const [items, setItems] = useState<StatusItem[]>([
    { label: "SQLite", status: "ok" },
    { label: "Python", status: "unknown" },
    { label: "FFmpeg", status: "unknown" },
    { label: "AI Provider", status: "unknown" },
  ]);

  useEffect(() => {
    // Check system status on mount
    const checkStatus = async () => {
      try {
        const pythonOk = await invoke("check_python").then(() => true).catch(() => false);
        const ffmpegOk = await invoke("check_ffmpeg").then(() => true).catch(() => false);
        const aiOk = await invoke("ai_health").then(() => true).catch(() => false);
        setItems([
          { label: "SQLite", status: "ok" },
          { label: "Python", status: pythonOk ? "ok" : "err" },
          { label: "FFmpeg", status: ffmpegOk ? "ok" : "warn" },
          { label: "AI Provider", status: aiOk ? "ok" : "warn" },
        ]);
      } catch {
        // Keep default statuses
      }
    };
    checkStatus();
  }, []);

  return (
    <footer className="status-bar">
      <div className="status-bar-left">
        {items.map((item) => (
          <div key={item.label} className="status-item">
            <span className={`status-dot status-dot-${item.status}`} />
            <span>{item.label}</span>
          </div>
        ))}
      </div>
      <div className="status-bar-right">
        <span className="status-item" style={{ color: 'var(--text-secondary)' }}>v0.1.0</span>
      </div>
    </footer>
  );
}