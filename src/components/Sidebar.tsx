import type { Project, NavigationPage } from "../types";

interface SidebarProps {
  projects: Project[];
  activeProject: Project | null;
  currentPage: NavigationPage;
  onSelectProject: (project: Project) => void;
  onCreateProject: () => void;
  onDeleteProject: (id: string) => void;
  onNavigate: (page: NavigationPage) => void;
  onAbout: () => void;
  onShortcuts?: () => void;
}

interface NavItem {
  id: NavigationPage;
  label: string;
  icon: React.ReactNode;
  disabled?: boolean;
}

interface NavGroup {
  title: string;
  items: NavItem[];
}

const I = ({ children }: { children: React.ReactNode }) => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">{children}</svg>
);

const dashboardIcon = <I><rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="3" y="14" width="7" height="7" /><rect x="14" y="14" width="7" height="7" /></I>;
const imageIcon = <I><rect x="3" y="3" width="18" height="18" rx="2" ry="2" /><circle cx="8.5" cy="8.5" r="1.5" /><polyline points="21 15 16 10 5 21" /></I>;
const videoIcon = <I><polygon points="23 7 16 12 23 17 23 7" /><rect x="1" y="5" width="15" height="14" rx="2" ry="2" /></I>;
const issuesIcon = <I><circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" /></I>;
const shieldIcon = <I><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" /></I>;
const bookIcon = <I><path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" /><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" /></I>;
const fileIcon = <I><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" /></I>;
const brainIcon = <I><path d="M12 2a7 7 0 0 1 7 7c0 2.4-1.2 4.5-3 5.7V17a1 1 0 0 1-1 1H9a1 1 0 0 1-1-1v-2.3A7 7 0 0 1 12 2z" /><path d="M9 21h6" /><path d="M10 17v4" /><path d="M14 17v4" /></I>;
const settingsIcon = <I><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" /></I>;

const NAV_GROUPS: NavGroup[] = [
  { title: "", items: [{ id: "dashboard", label: "Dashboard", icon: dashboardIcon }] },
  {
    title: "Analyze",
    items: [
      { id: "analyze-image", label: "Image", icon: imageIcon },
      { id: "analyze-video", label: "Video", icon: videoIcon },
    ],
  },
  { title: "", items: [{ id: "issues", label: "Issues", icon: issuesIcon }] },
  {
    title: "Knowledge",
    items: [
      { id: "knowledge-rules", label: "Severity Rules", icon: shieldIcon },
      { id: "knowledge-glossary", label: "Glossary", icon: bookIcon },
      { id: "knowledge-references", label: "Reference Docs", icon: fileIcon },
      { id: "knowledge-brain", label: "Ask Project Brain", icon: brainIcon },
    ],
  },
  {
    title: "",
    items: [
      { id: "settings", label: "Settings", icon: settingsIcon },
    ],
  },
];

export default function Sidebar({
  projects,
  activeProject,
  currentPage,
  onSelectProject,
  onCreateProject,
  onDeleteProject,
  onNavigate,
  onAbout,
  onShortcuts,
}: SidebarProps) {
  const isActive = (item: NavItem) => {
    if (currentPage === item.id) return true;
    if (item.id === "settings" && ["settings", "settings-ai", "settings-processing", "settings-template"].includes(currentPage)) return true;
    return false;
  };

  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <div className="sidebar-brand-mark" aria-hidden="true">Q</div>
        <span className="sidebar-brand-name">QAIVRA</span>
      </div>

      <div className="sidebar-section">
        <div className="section-header">
          <span className="section-label">Projects</span>
          <button className="btn-icon" onClick={onCreateProject} title="Create new project" aria-label="Create new project">+</button>
        </div>
        {projects.length === 0 ? (
          <div className="empty-hint">No projects yet</div>
        ) : (
          <ul className="project-list">
            {projects.map((project) => (
              <li key={project.id} className={"project-item" + (activeProject?.id === project.id ? " active" : "")}>
                <button className="project-btn" onClick={() => onSelectProject(project)} title={project.name}>
                  <span className="project-name">{project.name}</span>
                  {project.target_language && <span className="project-lang">{project.target_language}</span>}
                </button>
                <button className="btn-icon btn-delete" onClick={(e) => { e.stopPropagation(); onDeleteProject(project.id); }} title={"Delete " + project.name} aria-label={"Delete " + project.name}>
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" /></svg>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      <nav className="sidebar-nav">
        {NAV_GROUPS.map((group, gi) => (
          <div key={gi} className="nav-group">
            {group.title && <div className="nav-group-title">{group.title}</div>}
            <ul>
              {group.items.map((item) => (
                <li key={item.id}>
                  <button
                    className={"nav-item" + (isActive(item) ? " active" : "") + (item.disabled ? " disabled" : "")}
                    onClick={() => !item.disabled && onNavigate(item.id)}
                    disabled={item.disabled}
                    title={item.label}
                  >
                    <span className="nav-icon" aria-hidden="true">{item.icon}</span>
                    {item.label}
                  </button>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </nav>

      <div className="sidebar-footer">
        {onShortcuts && (
          <button className="about-btn" onClick={onShortcuts} >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" ><rect x="2" y="4" width="20" height="16" rx="2" ry="2" /><line x1="6" y1="8" x2="6.01" y2="8" /><line x1="10" y1="8" x2="10.01" y2="8" /><line x1="14" y1="8" x2="14.01" y2="8" /><line x1="18" y1="8" x2="18.01" y2="8" /></svg>
            Shortcuts
          </button>
        )}
        <button className="about-btn" onClick={onAbout}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" ><circle cx="12" cy="12" r="10" /><line x1="12" y1="16" x2="12" y2="12" /><line x1="12" y1="8" x2="12.01" y2="8" /></svg>
          About
        </button>
      </div>
    </aside>
  );
}
