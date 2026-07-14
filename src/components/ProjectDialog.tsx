import { useState } from "react";
import type { CreateProjectInput } from "../types";

interface ProjectDialogProps {
  onClose: () => void;
  onSubmit: (input: CreateProjectInput) => void;
}

const COMMON_LANGUAGES = [
  "Indonesian",
  "Japanese",
  "Korean",
  "Chinese (Simplified)",
  "Chinese (Traditional)",
  "Thai",
  "Vietnamese",
  "Portuguese (Brazil)",
  "German",
  "French",
  "Spanish",
  "Italian",
  "Russian",
  "Arabic",
  "Turkish",
  "Polish",
];

const COMMON_GENRES = [
  "RPG",
  "Action",
  "Adventure",
  "Strategy",
  "Puzzle",
  "Simulation",
  "FPS/TPS",
  "Sports",
  "Racing",
  "MMORPG",
  "Casual",
  "Horror",
  "Visual Novel",
];

const COMMON_PLATFORMS = [
  "PC",
  "Mobile (iOS/Android)",
  "PlayStation",
  "Xbox",
  "Nintendo Switch",
  "Multi-platform",
];

export default function ProjectDialog({ onClose, onSubmit }: ProjectDialogProps) {
  const [name, setName] = useState("");
  const [targetLanguage, setTargetLanguage] = useState("");
  const [genre, setGenre] = useState("");
  const [platform, setPlatform] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;

    onSubmit({
      name: name.trim(),
      target_language: targetLanguage,
      genre,
      platform,
    });
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>Create New Project</h2>
          <button className="btn-icon" onClick={onClose} aria-label="Close">
            ×
          </button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="dialog-body">
            <div className="form-field">
              <label htmlFor="project-name">
                Project Name <span className="required">*</span>
              </label>
              <input
                id="project-name"
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g., Indonesian Game LQA - Build 2.1"
                autoFocus
                required
              />
            </div>

            <div className="form-field">
              <label htmlFor="target-language">Target Language</label>
              <select
                id="target-language"
                value={targetLanguage}
                onChange={(e) => setTargetLanguage(e.target.value)}
              >
                <option value="">Select language...</option>
                {COMMON_LANGUAGES.map((lang) => (
                  <option key={lang} value={lang}>
                    {lang}
                  </option>
                ))}
              </select>
            </div>

            <div className="form-field">
              <label htmlFor="genre">Game Genre</label>
              <select id="genre" value={genre} onChange={(e) => setGenre(e.target.value)}>
                <option value="">Select genre...</option>
                {COMMON_GENRES.map((g) => (
                  <option key={g} value={g}>
                    {g}
                  </option>
                ))}
              </select>
            </div>

            <div className="form-field">
              <label htmlFor="platform">Platform</label>
              <select
                id="platform"
                value={platform}
                onChange={(e) => setPlatform(e.target.value)}
              >
                <option value="">Select platform...</option>
                {COMMON_PLATFORMS.map((p) => (
                  <option key={p} value={p}>
                    {p}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="dialog-footer">
            <button type="button" className="btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="btn-primary" disabled={!name.trim()}>
              Create Project
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}