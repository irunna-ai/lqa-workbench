import type { NavigationPage } from "../types";

interface SettingsTab {
  id: NavigationPage;
  label: string;
  available: boolean;
}

const SETTINGS_TABS: SettingsTab[] = [
  { id: "settings-ai", label: "AI Provider", available: true },
  { id: "settings-processing", label: "Processing", available: true },
  { id: "settings-template", label: "Export Template", available: true },
];

interface SettingsLayoutProps {
  activeTab: NavigationPage;
  onTabChange: (tab: NavigationPage) => void;
  children: React.ReactNode;
}

export default function SettingsLayout({ activeTab, onTabChange, children }: SettingsLayoutProps) {
  return (
    <div className="settings-layout">
      <div className="settings-header">
        <h2>Settings</h2>
        <p className="settings-subtitle">Configure AI providers, processing, and export templates.</p>
      </div>

      <div className="settings-tabs">
        {SETTINGS_TABS.map((tab) => (
          <button
            key={tab.id}
            className={`settings-tab ${activeTab === tab.id ? "active" : ""}`}
            onClick={() => onTabChange(tab.id)}
            title={tab.label}
          >
            {tab.label}
          </button>
        ))}
      </div>

      <div className="settings-content">
        {children}
      </div>
    </div>
  );
}