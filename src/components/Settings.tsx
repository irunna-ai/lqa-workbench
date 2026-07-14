import type { NavigationPage } from "../types";

interface SettingsProps {
  activeTab: NavigationPage;
}

export default function Settings({ activeTab }: SettingsProps) {
  return (
    <>
      {activeTab === "settings-ai" && (
        <div className="settings-section">
          <h3>AI Model Configuration</h3>
            <div className="settings-notice">
              <p>
                Configure your AI model for analysis.
              </p>
              <ul>
                <li>Configure Ollama provider connection</li>
                <li>View installed local models</li>
                <li>Check model capabilities for image and text analysis</li>
                <li>Test model connectivity</li>
              </ul>
            </div>
            <div className="settings-field">
              <label>Provider</label>
              <input type="text" value="Ollama (localhost)" disabled />
            </div>
            <div className="settings-field">
              <label>Status</label>
              <span className="status-badge status-unknown">Not Connected</span>
            </div>
          </div>
        )}

        {activeTab === "settings-processing" && (
          <div className="settings-section">
            <h3>Processing Configuration</h3>
            <div className="settings-notice">
              <p>
                Configure processing parameters.
              </p>
              <ul>
                <li>Video candidate extraction parameters</li>
                <li>Frame extraction quality and sampling</li>
                <li>Cache management</li>
                <li>Concurrency limits</li>
              </ul>
            </div>
          </div>
        )}

        {activeTab === "settings-template" && (
          <div className="settings-section">
            <h3>Export Template Configuration</h3>
            <div className="settings-notice">
              <p>
                Configure export templates.
              </p>
              <ul>
                <li>Inspect imported client XLSX templates</li>
                <li>View detected report sheets and headers</li>
                <li>Validate template structure</li>
              </ul>
            </div>
          </div>
        )}
    </>
  );
}