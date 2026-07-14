import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import type { NavigationPage } from "../types";

interface CommandItem {
  id: string;
  label: string;
  icon: string;
  shortcut?: string;
  group: string;
  action: () => void;
}

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
  onNavigate: (page: NavigationPage) => void;
  onCreateProject: () => void;
  onAbout: () => void;
  onShortcuts?: () => void;
}

export default function CommandPalette({
  open,
  onClose,
  onNavigate,
  onCreateProject,
  onAbout,
  onShortcuts,
}: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const allItems = useMemo<CommandItem[]>(() => [
    { id: "dashboard", label: "Go to Dashboard", icon: "◈", group: "Navigate", action: () => onNavigate("dashboard") },
    { id: "analyze-image", label: "Analyze Image", icon: "◉", group: "Navigate", action: () => onNavigate("analyze-image") },
    { id: "analyze-video", label: "Analyze Video", icon: "▶", group: "Navigate", action: () => onNavigate("analyze-video") },
    { id: "issues", label: "View Issues", icon: "◼", group: "Navigate", action: () => onNavigate("issues") },
    { id: "knowledge-rules", label: "Severity Rules", icon: "◎", group: "Navigate", action: () => onNavigate("knowledge-rules") },
    { id: "knowledge-glossary", label: "Glossary", icon: "☰", group: "Navigate", action: () => onNavigate("knowledge-glossary") },
    { id: "knowledge-references", label: "Reference Docs", icon: "⊞", group: "Navigate", action: () => onNavigate("knowledge-references") },
    { id: "knowledge-brain", label: "Ask Project Brain", icon: "⚡", group: "Navigate", action: () => onNavigate("knowledge-brain") },
    { id: "settings-ai", label: "AI Model Settings", icon: "◆", group: "Navigate", action: () => { onNavigate("settings"); /* Tab set in parent */ } },
    { id: "settings-processing", label: "Processing Settings", icon: "⚙", group: "Navigate", action: () => { onNavigate("settings-processing"); } },
    { id: "settings-template", label: "Export Template", icon: "↗", group: "Navigate", action: () => { onNavigate("settings-template"); } },
    { id: "open-settings", label: "Open Settings", icon: "⚙", shortcut: "Ctrl+,", group: "Actions", action: () => { onNavigate("settings"); } },
    { id: "new-project", label: "New Project", icon: "+", shortcut: "Ctrl+N", group: "Actions", action: () => { onCreateProject(); onClose(); } },
    { id: "about", label: "About QAIVRA", icon: "ℹ", group: "Actions", action: () => { onAbout(); onClose(); } },
    { id: "shortcuts", label: "Keyboard Shortcuts", icon: "⌨", shortcut: "Ctrl+/", group: "Actions", action: () => { onShortcuts?.(); onClose(); } },
  ], [onNavigate, onCreateProject, onAbout, onShortcuts]);

  const filtered = useMemo(() => {
    if (!query.trim()) return allItems;
    const q = query.toLowerCase();
    return allItems.filter((item) => item.label.toLowerCase().includes(q));
  }, [query, allItems]);

  const grouped = useMemo(() => {
    const groups = new Map<string, CommandItem[]>();
    for (const item of filtered) {
      const arr = groups.get(item.group) || [];
      arr.push(item);
      groups.set(item.group, arr);
    }
    return groups;
  }, [filtered]);

  useEffect(() => {
    if (open) {
      setQuery("");
      setActiveIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  const execute = useCallback((item: CommandItem) => {
    item.action();
    onClose();
  }, [onClose]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActiveIndex((i) => Math.min(i + 1, filtered.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActiveIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter" && filtered[activeIndex]) {
      e.preventDefault();
      execute(filtered[activeIndex]);
    } else if (e.key === "Escape") {
      onClose();
    }
  }, [filtered, activeIndex, execute, onClose]);

  // Keep activeIndex in bounds
  useEffect(() => {
    if (activeIndex >= filtered.length) setActiveIndex(Math.max(0, filtered.length - 1));
  }, [filtered.length, activeIndex]);

  // Scroll active item into view
  useEffect(() => {
    const el = listRef.current?.querySelector(".cmd-item.active");
    el?.scrollIntoView({ block: "nearest" });
  }, [activeIndex]);

  if (!open) return null;

  let flatIndex = 0;

  return (
    <div className="cmd-palette-overlay" onClick={onClose}>
      <div className="cmd-palette" onClick={(e) => e.stopPropagation()} onKeyDown={handleKeyDown}>
        <div className="cmd-input-wrap">
          <input
            ref={inputRef}
            className="cmd-input"
            type="text"
            placeholder="Search commands, pages, actions..."
            value={query}
            onChange={(e) => { setQuery(e.target.value); setActiveIndex(0); }}
          />
        </div>
        <div className="cmd-results" ref={listRef}>
          {filtered.length === 0 ? (
            <div className="cmd-empty">No results for &quot;{query}&quot;</div>
          ) : (
            Array.from(grouped.entries()).map(([group, items]) => (
              <div key={group}>
                <div className="cmd-group-label">{group}</div>
                {items.map((item) => {
                  const idx = flatIndex++;
                  return (
                    <div
                      key={item.id}
                      className={`cmd-item ${idx === activeIndex ? "active" : ""}`}
                      onClick={() => execute(item)}
                      onMouseEnter={() => setActiveIndex(idx)}
                    >
                      <span className="cmd-item-icon">{item.icon}</span>
                      <span className="cmd-item-label">{item.label}</span>
                      {item.shortcut && <span className="cmd-item-shortcut">{item.shortcut}</span>}
                    </div>
                  );
                })}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}