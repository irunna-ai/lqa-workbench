import { useState, useEffect } from "react";
import { buildApi, type BuildInfo } from "../api/tauri";

interface AboutDialogProps {
  open: boolean;
  onClose: () => void;
}

const REPO_PUBLIC = true;
const REPO_URL = "https://github.com/irunna-ai/lqa-workbench";

function formatBuildDate(timestamp: string): string {
  try {
    const d = new Date(timestamp);
    return d.toLocaleDateString("en-GB", {
      day: "2-digit", month: "short", year: "numeric",
    }) + " \u2022 " + d.toLocaleTimeString("en-GB", {
      hour: "2-digit", minute: "2-digit", hour12: false,
    }) + " WIB";
  } catch { return "\u2014"; }
}

export default function AboutDialog({ open, onClose }: AboutDialogProps) {
  const [info, setInfo] = useState<BuildInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [unavailable, setUnavailable] = useState(false);

  useEffect(() => {
    if (open && !info) {
      buildApi.getBuildInfo().then(setInfo).catch((err) => {
        const msg = String(err);
        if (msg.includes("Build metadata unavailable") || msg.includes("Unexpected token") || msg.includes("JSON")) {
          setUnavailable(true);
        } else {
          setError("Build metadata unavailable.");
        }
      });
    }
  }, [open, info]);

  useEffect(() => {
    if (!open) return;
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") { e.preventDefault(); onClose(); }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [open, onClose]);

  if (!open) return null;

  const shortHash = info?.commit_short ?? "\u2014";
  const totalTests = (info?.tests.cargo_test.passed ?? 0) + (info?.tests.vitest.passed ?? 0);
  const buildDate = info?.timestamp ? formatBuildDate(info.timestamp) : "\u2014";

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div
        className="dialog about-dialog"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="about-header">
          <div className="about-logo-row">
            <div className="about-logo-icon">
              <svg width="28" height="28" viewBox="0 0 28 28" fill="none">
                <rect width="28" height="28" rx="6" fill="var(--accent)" fillOpacity="0.15" />
                <text x="14" y="19" textAnchor="middle" fill="var(--accent)" fontSize="14" fontWeight="700">Q</text>
              </svg>
            </div>
            <div>
              <h2>QAIVRA</h2>
              <p className="about-subtitle">Professional Localization QA Workbench</p>
            </div>
          </div>
          {info && (
            <span className="about-version-badge">v{info.version}-build{info.build}</span>
          )}
        </div>

        {unavailable ? (
          <p className="about-loading">Build metadata unavailable.</p>
        ) : error ? (
          <p className="about-error">{error}</p>
        ) : !info ? (
          <p className="about-loading">Loading build information...</p>
        ) : (
          <div className="about-grid">
            <div className="about-section">
              <h3>Build</h3>
              <table className="about-table">
                <tbody>
                  <tr><td>Version</td><td><code>{info.version}</code></td></tr>
                  <tr><td>Build</td><td><code>{info.build}</code></td></tr>
                  <tr><td>Channel</td><td><span className="about-channel-badge">{info.channel}</span></td></tr>
                  <tr><td>Commit</td><td><code>{shortHash}</code></td></tr>
                  <tr><td>Git Tag</td><td><code>{info.tag}</code></td></tr>
                  <tr><td>Build Date</td><td>{buildDate}</td></tr>
                </tbody>
              </table>
            </div>

            <div className="about-section">
              <h3>Platform</h3>
              <table className="about-table">
                <tbody>
                  <tr><td>OS</td><td>{info.build_machine?.os ?? "�"}</td></tr>
                  <tr><td>Architecture</td><td>{info.build_machine?.arch ?? "�"}</td></tr>
                  <tr><td>Rust</td><td><code>{info.toolchain?.rustc?.split(" ").slice(0, 2).join(" ") ?? "�"}</code></td></tr>
                  <tr><td>Node</td><td><code>{info.toolchain?.node ?? "�"}</code></td></tr>
                  <tr><td>Python</td><td><code>{info.toolchain?.python ?? "�"}</code></td></tr>
                  <tr><td>Tests Passed</td><td><span className="about-tests-badge">{totalTests}</span></td></tr>
                </tbody>
              </table>
            </div>

            <div className="about-section about-section-full">
              <h3>Publisher</h3>
              <table className="about-table">
                <tbody>
                  <tr><td>Publisher</td><td>Irunna AI</td></tr>
                  <tr><td>Author</td><td>Iqbal Annur</td></tr>
                  <tr><td>License</td><td>MIT</td></tr>
                  <tr>
                    <td>Repository</td>
                    <td>
                      {REPO_PUBLIC ? (
                        <a href={REPO_URL} target="_blank" rel="noreferrer" className="about-repo-link">
                          {REPO_URL.replace("https://", "")}
                        </a>
                      ) : (
                        <span className="about-repo-private" title="Repository will become available after public beta.">
                          Private Repository
                        </span>
                      )}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>

            <div className="about-section about-section-full">
              <h3>File Hashes</h3>
              <table className="about-table">
                <tbody>
                  <tr>
                    <td>Executable</td>
                    <td><code className="about-hash">{info.artifacts?.executable?.sha256 ?? "�"}</code></td>
                  </tr>
                  <tr>
                    <td>Installer</td>
                    <td><code className="about-hash">{info.artifacts?.installer?.sha256 ?? "�"}</code></td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        )}

        <div className="about-actions">
          <button className="btn btn-primary" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
