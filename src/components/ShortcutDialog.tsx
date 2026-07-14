import { useEffect, useRef } from "react";

interface ShortcutDialogProps {
  open: boolean;
  onClose: () => void;
}

const SHORTCUTS = [
  { category: "Navigation", items: [
    { keys: ["Ctrl", "K"], desc: "Open Command Palette" },
    { keys: ["Ctrl", "N"], desc: "New Project" },
    { keys: ["Ctrl", ","], desc: "Open Settings" },
    { keys: ["Ctrl", "/"], desc: "Keyboard Shortcuts" },
    { keys: ["F1"], desc: "Keyboard Shortcuts" },
    { keys: ["Esc"], desc: "Close Dialog" },
  ]},
  { category: "Analysis", items: [
    { keys: ["Ctrl", "Shift", "A"], desc: "Analyze Image" },
    { keys: ["Ctrl", "R"], desc: "Refresh Current Page" },
  ]},
  { category: "General", items: [
    { keys: ["Delete"], desc: "Delete Selected Item" },
    { keys: ["↑", "↓"], desc: "Navigate Lists" },
    { keys: ["Enter"], desc: "Confirm Selection" },
  ]},
];

export default function ShortcutDialog({ open, onClose }: ShortcutDialogProps) {
  const overlayRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") { e.preventDefault(); onClose(); }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="dialog-overlay shortcut-dialog-overlay" ref={overlayRef} onClick={onClose}>
      <div className="dialog shortcut-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="shortcut-dialog-header">
          <h2>Keyboard Shortcuts</h2>
          <button className="shortcut-close" onClick={onClose} aria-label="Close">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>
        <div className="shortcut-dialog-body">
          {SHORTCUTS.map((group) => (
            <div className="shortcut-group" key={group.category}>
              <h3 className="shortcut-group-title">{group.category}</h3>
              <div className="shortcut-list">
                {group.items.map((item, i) => (
                  <div className="shortcut-row" key={i}>
                    <span className="shortcut-desc">{item.desc}</span>
                    <span className="shortcut-keys">
                      {item.keys.map((key, ki) => (
                        <span key={ki}>
                          <kbd className="shortcut-kbd">{key}</kbd>
                          {ki < item.keys.length - 1 && <span className="shortcut-sep">+</span>}
                        </span>
                      ))}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
        <div className="shortcut-dialog-footer">
          <span className="shortcut-hint">Press <kbd>Esc</kbd> to close</span>
        </div>
      </div>
    </div>
  );
}