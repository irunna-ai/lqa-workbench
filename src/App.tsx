import { useState, useEffect, useCallback } from "react";
import Sidebar from "./components/Sidebar";
import Dashboard from "./components/Dashboard";
import SettingsAI from "./components/SettingsAI";
import SettingsLayout from "./components/SettingsLayout";
import Settings from "./components/Settings";
import SeverityRules from "./components/SeverityRules";
import Glossary from "./components/Glossary";
import ReferenceDocs from "./components/ReferenceDocs";
import ProjectBrain from "./components/ProjectBrain";
import ImageAnalysis from "./components/ImageAnalysis";
import VideoAnalysis from "./components/VideoAnalysis";
import Issues from "./components/Issues";
import ProjectDialog from "./components/ProjectDialog";
import AboutDialog from "./components/AboutDialog";
import StatusBar from "./components/StatusBar";
import CommandPalette from "./components/CommandPalette";
import ShortcutDialog from "./components/ShortcutDialog";
import { ToastProvider } from "./components/Toast";
import type { Project, NavigationPage, CreateProjectInput } from "./types";
import { projectApi } from "./api/tauri";
import "./styles.css";

export default function App() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [activeProject, setActiveProject] = useState<Project | null>(null);
  const [currentPage, setCurrentPage] = useState<NavigationPage>("dashboard");
  const [settingsTab, setSettingsTab] = useState<NavigationPage>("settings-ai");
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAboutDialog, setShowAboutDialog] = useState(false);
  const [showCmdPalette, setShowCmdPalette] = useState(false);
  const [showShortcuts, setShowShortcuts] = useState(false);

  const loadProjects = useCallback(async () => {
    try {
      const list = await projectApi.listProjects();
      setProjects(list);
      setError(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        setShowCmdPalette((prev) => !prev);
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "n") {
        e.preventDefault();
        setShowCreateDialog(true);
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key === ",") {
        e.preventDefault();
        setCurrentPage("settings");
        setSettingsTab("settings-ai");
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "/") {
        e.preventDefault();
        setShowShortcuts((prev) => !prev);
        return;
      }
      if (e.key === "F1") {
        e.preventDefault();
        setShowShortcuts((prev) => !prev);
        return;
      }
      if (e.key === "Escape") {
        if (showCmdPalette) setShowCmdPalette(false);
        else if (showShortcuts) setShowShortcuts(false);
        else if (showAboutDialog) setShowAboutDialog(false);
        else if (showCreateDialog) setShowCreateDialog(false);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [showCmdPalette, showAboutDialog, showCreateDialog, showShortcuts]);

  const handleCreateProject = async (input: CreateProjectInput) => {
    try {
      const project = await projectApi.createProject(input);
      setProjects((prev) => [project, ...prev]);
      setActiveProject(project);
      setCurrentPage("dashboard");
      setShowCreateDialog(false);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleSelectProject = (project: Project) => {
    setActiveProject(project);
    setCurrentPage("dashboard");
  };

  const handleDeleteProject = async (id: string) => {
    try {
      await projectApi.deleteProject(id);
      setProjects((prev) => prev.filter((p) => p.id !== id));
      if (activeProject?.id === id) {
        setActiveProject(null);
      }
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleSettingsTabChange = useCallback((tab: NavigationPage) => {
    setSettingsTab(tab);
  }, []);

  const renderPage = () => {
    // Unified settings page
    if (currentPage === "settings" || currentPage === "settings-ai" || currentPage === "settings-processing" || currentPage === "settings-template") {
      return (
        <SettingsLayout activeTab={settingsTab} onTabChange={handleSettingsTabChange}>
          {settingsTab === "settings-ai" && <SettingsAI activeProject={activeProject} />}
          {settingsTab === "settings-processing" && <Settings activeTab="settings-processing" />}
          {settingsTab === "settings-template" && <Settings activeTab="settings-template" />}
        </SettingsLayout>
      );
    }

    switch (currentPage) {
      case "dashboard":
        return (
          <Dashboard
            activeProject={activeProject}
            onNavigate={setCurrentPage}
            onCreateProject={() => setShowCreateDialog(true)}
          />
        );
      case "knowledge-rules":
        return <SeverityRules activeProject={activeProject} />;
      case "knowledge-glossary":
        return <Glossary activeProject={activeProject} />;
      case "knowledge-references":
        return <ReferenceDocs activeProject={activeProject} />;
      case "knowledge-brain":
        return <ProjectBrain activeProject={activeProject} />;
      case "analyze-image":
        return <ImageAnalysis activeProject={activeProject} onNavigate={setCurrentPage} />;
      case "analyze-video":
        return <VideoAnalysis activeProject={activeProject} />;
      case "issues":
        return <Issues activeProject={activeProject} onNavigate={setCurrentPage} />;
      default:
        return (
          <Dashboard
            activeProject={activeProject}
            onNavigate={setCurrentPage}
            onCreateProject={() => setShowCreateDialog(true)}
          />
        );
    }
  };

  if (loading) {
    return (
      <div className="app-loading">
        <div className="loading-spinner" />
        <p>Loading QAIVRA...</p>
      </div>
    );
  }

  return (
    <ToastProvider>
      <div className="app">
        {error && (
          <div className="error-banner">
            <span>{error}</span>
            <button onClick={() => setError(null)}>×</button>
          </div>
        )}
        <Sidebar
          projects={projects}
          activeProject={activeProject}
          currentPage={currentPage}
          onSelectProject={handleSelectProject}
          onCreateProject={() => setShowCreateDialog(true)}
          onDeleteProject={handleDeleteProject}
          onNavigate={setCurrentPage}
          onAbout={() => setShowAboutDialog(true)}
          onShortcuts={() => setShowShortcuts(true)}
        />
        <div style={{ display: "flex", flexDirection: "column", flex: 1, minWidth: 0 }}>
          <main className="content">{renderPage()}</main>
          <StatusBar />
        </div>
        {showCreateDialog && (
          <ProjectDialog
            onClose={() => setShowCreateDialog(false)}
            onSubmit={handleCreateProject}
          />
        )}
        <AboutDialog
          open={showAboutDialog}
          onClose={() => setShowAboutDialog(false)}
        />
        <CommandPalette
          open={showCmdPalette}
          onClose={() => setShowCmdPalette(false)}
          onNavigate={(page) => {
            if (page === "settings-ai" || page === "settings" || page === "settings-processing" || page === "settings-template") {
              setCurrentPage("settings");
              setSettingsTab(page === "settings" ? "settings-ai" : page);
            } else {
              setCurrentPage(page);
            }
            setShowCmdPalette(false);
          }}
          onCreateProject={() => { setShowCreateDialog(true); setShowCmdPalette(false); }}
          onAbout={() => { setShowAboutDialog(true); setShowCmdPalette(false); }}
          onShortcuts={() => { setShowShortcuts(true); setShowCmdPalette(false); }}
        />
        <ShortcutDialog
          open={showShortcuts}
          onClose={() => setShowShortcuts(false)}
        />
      </div>
    </ToastProvider>
  );
}