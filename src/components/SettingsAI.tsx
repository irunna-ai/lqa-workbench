import { useState, useEffect, useCallback } from "react";
import type { Project, OllamaHealthResult, OllamaModel, AiProviderConfig, TestConnectionResult } from "../types";
import { aiApi } from "../api/tauri";
import { useToast } from "./Toast";

interface SettingsAIProps { activeProject: Project | null; }

const PROVIDERS = [
  { id: "ollama", name: "Ollama", endpoint: "http://localhost:11434", needsKey: false, desc: "Local models via Ollama" },
  { id: "openai", name: "OpenAI", endpoint: "https://api.openai.com/v1", needsKey: true, desc: "GPT-4o, GPT-4, etc." },
  { id: "gemini", name: "Gemini", endpoint: "https://generativelanguage.googleapis.com/v1beta", needsKey: true, desc: "Google Gemini models" },
  { id: "claude", name: "Claude", endpoint: "https://api.anthropic.com/v1", needsKey: true, desc: "Anthropic Claude models" },
  { id: "openrouter", name: "OpenRouter", endpoint: "https://openrouter.ai/api/v1", needsKey: true, desc: "Multi-provider gateway" },
  { id: "lmstudio", name: "LM Studio", endpoint: "http://localhost:1234/v1", needsKey: false, desc: "Local models via LM Studio" },
  { id: "openai_compatible", name: "Custom", endpoint: "", needsKey: true, desc: "Any OpenAI-compatible API" },
] as const;

export default function SettingsAI({ activeProject }: SettingsAIProps) {
  const [config, setConfig] = useState<AiProviderConfig | null>(null);
  const [provider, setProvider] = useState("ollama");
  const [endpoint, setEndpoint] = useState("http://localhost:11434");
  const [apiKey, setApiKey] = useState("");
  const [selectedModel, setSelectedModel] = useState("");
  const [temperature, setTemperature] = useState(0.0);
  const [maxTokens, setMaxTokens] = useState(4096);
  const [timeoutSecs, setTimeoutSecs] = useState(120);
  const [models, setModels] = useState<OllamaModel[]>([]);
  const [ollamaHealth, setOllamaHealth] = useState<OllamaHealthResult | null>(null);
  const [testResult, setTestResult] = useState<TestConnectionResult | null>(null);
  const [checking, setChecking] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const toast = useToast();

  const loadConfig = useCallback(async () => {
    if (!activeProject) return;
    try {
      const c = await aiApi.getConfig(activeProject.id);
      setConfig(c);
      setProvider(c.provider || "ollama");
      setEndpoint(c.base_url || "");
      setSelectedModel(c.selected_model || "");
      setTemperature(c.temperature ?? 0.0);
      setMaxTokens(c.max_tokens ?? 4096);
      setTimeoutSecs(c.timeout_secs ?? 120);
      if (c.api_key_ref) setApiKey("masked");
    } catch { /* No config yet */ }
  }, [activeProject]);

  useEffect(() => { loadConfig(); setTestResult(null); setModels([]); setOllamaHealth(null); }, [loadConfig]);

  const currentProvider = PROVIDERS.find(p => p.id === provider) || PROVIDERS[0];

  const handleProviderChange = (newProvider: string) => {
    setProvider(newProvider);
    const p = PROVIDERS.find(pr => pr.id === newProvider);
    if (p) setEndpoint(p.endpoint);
    setModels([]); setOllamaHealth(null); setTestResult(null);
    if (newProvider === "ollama" || newProvider === "lmstudio") setApiKey("");
  };

  const handleTestConnection = async () => {
    setChecking(true); setError(null); setTestResult(null);
    try {
      const keyRef = apiKey && apiKey !== "masked"
        ? `apikey_${activeProject?.id}_${provider}` : (config?.api_key_ref || "");
      const result = await aiApi.testConnection(provider, endpoint, keyRef);
      setTestResult(result);
      if (result.success && provider === "ollama") {
        try {
          const h = await aiApi.checkHealth(endpoint);
          setOllamaHealth(h);
          if (h.healthy) setModels(await aiApi.listModels(endpoint));
        } catch { /* ignore */ }
      }
      if (result.success) {
        toast.addToast("success", "Connection OK", `${currentProvider.name} - ${result.latency_ms}ms`);
      } else {
        toast.addToast("error", "Connection Failed", result.message);
      }
    } catch (err) {
      setError(String(err));
      toast.addToast("error", "Connection Error", String(err));
    } finally { setChecking(false); }
  };

  const handleSave = async () => {
    if (!activeProject) return;
    setSaving(true); setError(null);
    try {
      let keyRef = config?.api_key_ref || "";
      if (currentProvider.needsKey && apiKey && apiKey !== "masked") {
        keyRef = `apikey_${activeProject.id}_${provider}`;
      }
      const caps = JSON.stringify(selectedModel ? models.find(m => m.name === selectedModel)?.capabilities || {} : {});
      const healthStatus = testResult?.success ? "healthy" : "unknown";
      await aiApi.saveConfig(activeProject.id, provider, endpoint, selectedModel,
        caps, healthStatus, temperature, maxTokens, timeoutSecs, keyRef, currentProvider.name);
      await loadConfig();
      toast.addToast("success", "Saved", `${currentProvider.name} configuration saved`);
    } catch (err) { setError(String(err)); toast.addToast("error", "Save Failed", String(err)); }
    finally { setSaving(false); }
  };


  if (!activeProject) return (
    <div className="page-container">
      <div className="empty-state">
        <div className="empty-icon">&#129302;</div>
        <h2>No Project Selected</h2>
        <p>Select a project to configure AI settings.</p>
      </div>
    </div>
  );

  return (
    <div className="settings">
      <div className="page-header">
        <h2>AI Provider Configuration</h2>
        <span className="project-context">{activeProject.name}</span>
      </div>

      <div className="settings-section">
        <h3>Provider</h3>
        <div className="provider-grid">
          {PROVIDERS.map(p => (
            <div key={p.id}
              className={`provider-card ${provider === p.id ? "active" : ""}`}
              onClick={() => handleProviderChange(p.id)}>
              <div className="provider-name">{p.name}</div>
              <div className="provider-desc">{p.desc}</div>
            </div>
          ))}
        </div>
      </div>

      <div className="settings-section">
        <h3>Connection</h3>
        <div className="settings-field">
          <label>Endpoint</label>
          <input type="text" value={endpoint} onChange={e => setEndpoint(e.target.value)}
            placeholder={currentProvider.endpoint} className="settings-input" />
        </div>
        {currentProvider.needsKey && (
          <div className="settings-field">
            <label>API Key</label>
            <input type="password" value={apiKey} onChange={e => setApiKey(e.target.value)}
              placeholder="sk-..." className="settings-input" />
            <span className="field-hint">Stored securely in OS keyring</span>
          </div>
        )}
        <div className="settings-field" style={{ display: "flex", alignItems: "center", gap: "0.75rem" }}>
          <button className="btn-secondary" onClick={handleTestConnection} disabled={checking || !endpoint}>
            {checking ? "Testing..." : "Test Connection"}
          </button>
          {testResult && (
            <>
              <span className={testResult.success ? "status-badge status-healthy" : "status-badge status-error"}>
                {testResult.success ? "Connected" : "Failed"}
              </span>
              {testResult.success && (
                <span style={{ fontSize: "0.75rem", color: "var(--text-secondary)", background: "var(--bg-overlay)", padding: "2px 8px", borderRadius: "var(--radius-sm)" }}>
                  {testResult.latency_ms}ms
                </span>
              )}
              {testResult.version && (
                <span style={{ fontSize: "0.75rem", color: "var(--text-muted)" }}>v{testResult.version}</span>
              )}
              {!testResult.success && testResult.message && (
                <span style={{ fontSize: "0.75rem", color: "var(--danger)" }}>{testResult.message}</span>
              )}
            </>
          )}
        </div>
        {provider === "ollama" && ollamaHealth && (
          <div className="settings-field">
            <label>Ollama Status</label>
            <span className={ollamaHealth.healthy ? "status-badge status-healthy" : "status-badge status-error"}>
              {ollamaHealth.healthy ? `Connected (v${ollamaHealth.version})` : "Not Connected"}
            </span>
          </div>
        )}
      </div>

      <div className="settings-section">
        <h3>Model</h3>
        {provider === "ollama" && models.length > 0 ? (
          <>
            <div className="settings-field">
              <label>Select Model</label>
              <select value={selectedModel} onChange={e => setSelectedModel(e.target.value)} className="settings-select">
                <option value="">Select a model</option>
                {models.map(m => (
                  <option key={m.name} value={m.name}>
                    {m.name}{m.parameter_size ? ` (${m.parameter_size})` : ""}{m.capabilities.vision ? " [Vision]" : ""}
                  </option>
                ))}
              </select>
            </div>
            <div className="model-table-container">
              <table className="model-table">
                <thead><tr><th>Model</th><th>Size</th><th>Family</th><th>Vision</th><th>Text</th></tr></thead>
                <tbody>{models.map(m => (
                  <tr key={m.name} className={m.name === selectedModel ? "selected" : ""}>
                    <td><strong>{m.name}</strong></td>
                    <td>{m.parameter_size || "-"}</td>
                    <td>{m.family || "-"}</td>
                    <td>{m.capabilities.vision ? "Yes" : "-"}</td>
                    <td>Yes</td>
                  </tr>
                ))}</tbody>
              </table>
            </div>
          </>
        ) : (
          <div className="settings-field">
            <label>Model Name</label>
            <input type="text" value={selectedModel} onChange={e => setSelectedModel(e.target.value)}
              placeholder={provider === "ollama" ? "Select after testing connection" : "e.g. gpt-4o, claude-3-opus"}
              className="settings-input" />
          </div>
        )}
      </div>

      <div className="settings-section">
        <h3>Parameters</h3>
        <div className="ai-params-grid">
          <div className="settings-field">
            <label>Temperature ({temperature.toFixed(1)})</label>
            <input type="range" min="0" max="2" step="0.1" value={temperature}
              onChange={e => setTemperature(parseFloat(e.target.value))} style={{ width: "100%" }} />
          </div>
          <div className="settings-field">
            <label>Max Tokens</label>
            <input type="number" value={maxTokens} onChange={e => setMaxTokens(parseInt(e.target.value) || 4096)}
              className="settings-input" min={1} max={128000} />
          </div>
          <div className="settings-field">
            <label>Timeout (seconds)</label>
            <input type="number" value={timeoutSecs} onChange={e => setTimeoutSecs(parseInt(e.target.value) || 120)}
              className="settings-input" min={5} max={600} />
          </div>
        </div>
      </div>

      <div className="actions-bar">
        <button className="btn-primary" onClick={handleSave} disabled={saving || !endpoint}>
          {saving ? "Saving..." : "Save Configuration"}
        </button>
        {config?.selected_model && (
          <span className="config-saved-hint">
            Currently: {config.provider_display_name || config.provider} - {config.selected_model}
          </span>
        )}
      </div>
      {error && (
        <div className="error-message">
          <span>{error}</span>
          <button onClick={() => setError(null)}>&times;</button>
        </div>
      )}
    </div>
  );
}
