import { invoke } from "@tauri-apps/api/core";
import type { Project, CreateProjectInput, SeverityRule, SeverityImportResult, SeverityStatus,
  GlossaryEntry, GlossaryImportResult, GlossaryHeaderResult, ColumnMapping,
  ReferenceChunk, ReferenceImportResult, ReferenceStatus, GlossaryStatus, ProjectBrainResult,
  PrecedentReport, PrecedentImportResult, GlossaryConflict,
  EvidenceAnnotation, EvidenceCandidate, TestConnectionResult } from "../types";

/**
 * Invoke a Tauri command and return the result.
 * Wraps invoke() to provide consistent error handling.
 */
async function tauriCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    console.error(`Tauri command '${command}' failed:`, error);
    throw error;
  }
}

export const projectApi = {
  listProjects: () => tauriCommand<Project[]>("list_projects"),

  getProject: (id: string) => tauriCommand<Project>("get_project_by_id", { id }),

  createProject: (input: CreateProjectInput) =>
    tauriCommand<Project>("create_new_project", {
      name: input.name,
      targetLanguage: input.target_language,
      genre: input.genre,
      platform: input.platform,
    }),

  deleteProject: (id: string) => tauriCommand<void>("delete_project_by_id", { id }),

  renameProject: (id: string, newName: string) =>
    tauriCommand<Project>("rename_project_by_id", { id, newName }),
};

export const severityApi = {
  importRules: (projectId: string, workbookPath: string) =>
    tauriCommand<SeverityImportResult>("import_severity_rules_cmd", {
      projectId,
      workbookPath,
    }),

  listRules: (projectId: string) =>
    tauriCommand<SeverityRule[]>("list_severity_rules_cmd", { projectId }),

  getStatus: (projectId: string) =>
    tauriCommand<SeverityStatus>("get_severity_status_cmd", { projectId }),
};

export const glossaryApi = {
  parseHeaders: (filePath: string, sheetName?: string) =>
    tauriCommand<GlossaryHeaderResult>("parse_glossary_headers_cmd", { filePath, sheetName }),

  import: (projectId: string, filePath: string, columnMapping: ColumnMapping, sheetName?: string) =>
    tauriCommand<GlossaryImportResult>("import_glossary_cmd", { projectId, filePath, columnMapping, sheetName }),

  listEntries: (projectId: string) =>
    tauriCommand<GlossaryEntry[]>("list_glossary_entries_cmd", { projectId }),

  getStatus: (projectId: string) =>
    tauriCommand<GlossaryStatus>("get_glossary_status_cmd", { projectId }),
};

export const referenceApi = {
  import: (projectId: string, filePath: string) =>
    tauriCommand<ReferenceImportResult>("import_reference_doc_cmd", { projectId, filePath }),

  listChunks: (projectId: string) =>
    tauriCommand<ReferenceChunk[]>("list_reference_docs_cmd", { projectId }),

  getStatus: (projectId: string) =>
    tauriCommand<ReferenceStatus>("get_reference_status_cmd", { projectId }),
};

export const brainApi = {
  query: (projectId: string, query: string) =>
    tauriCommand<ProjectBrainResult>("query_project_brain_cmd", { projectId, query }),
};

export const precedentApi = {
  import: (projectId: string, filePath: string, classification: string) =>
    tauriCommand<PrecedentImportResult>("import_precedent_cmd", { projectId, filePath, classification }),

  list: (projectId: string) =>
    tauriCommand<PrecedentReport[]>("list_precedent_reports_cmd", { projectId }),
};

export const conflictApi = {
  list: (projectId: string) =>
    tauriCommand<GlossaryConflict[]>("get_glossary_conflicts_cmd", { projectId }),

  getCount: (projectId: string) =>
    tauriCommand<number>("get_glossary_conflict_count_cmd", { projectId }),

  resolve: (conflictId: string, chosenEntryId: string) =>
    tauriCommand<void>("resolve_glossary_conflict_cmd", { conflictId, chosenEntryId }),
};

export const aiApi = {
  checkHealth: (baseUrl: string) =>
    tauriCommand<import("../types").OllamaHealthResult>("check_ollama_health_cmd", { baseUrl }),

  listModels: (baseUrl: string) =>
    tauriCommand<import("../types").OllamaModel[]>("list_ollama_models_cmd", { baseUrl }),

  getModelCapabilities: (baseUrl: string, modelName: string) =>
    tauriCommand<import("../types").ModelCapabilities>("get_model_capabilities_cmd", { baseUrl, modelName }),

  saveConfig: (projectId: string, provider: string, baseUrl: string,
    selectedModel: string, capabilitiesJson: string, healthStatus: string,
    temperature: number, maxTokens: number, timeoutSecs: number,
    apiKeyRef: string, providerDisplayName: string) =>
    tauriCommand<import("../types").AiProviderConfig>("save_ai_config_cmd", {
      projectId, provider, baseUrl, selectedModel, capabilitiesJson, healthStatus,
      temperature, maxTokens, timeoutSecs, apiKeyRef, providerDisplayName
    }),

  getConfig: (projectId: string) =>
    tauriCommand<import("../types").AiProviderConfig>("get_ai_config_cmd", { projectId }),

  updateHealth: (projectId: string, healthStatus: string) =>
    tauriCommand<void>("update_ai_health_cmd", { projectId, healthStatus }),

  testConnection: (providerType: string, endpoint: string, apiKeyRef: string) =>
    tauriCommand<TestConnectionResult>("test_provider_connection_cmd", { providerType, endpoint, apiKeyRef }),
};

export const ocrApi = {
  runOcr: (imagePath: string) =>
    tauriCommand<import("../types").OcrResult>("run_ocr_cmd", { imagePath }),
};

export const validationApi = {
  validateCandidate: (candidate: import("../types").ValidationResult, projectId: string) =>
    tauriCommand<import("../types").ValidationResult>("validate_candidate_cmd", { candidate, projectId }),
};

export const imageApi = {
  uploadImage: (projectId: string, sourcePath: string) =>
    tauriCommand<import("../types").ProjectImage>("upload_image_cmd", {
      projectId,
      sourcePath,
    }),

  listImages: (projectId: string) =>
    tauriCommand<import("../types").ProjectImage[]>("list_images_cmd", { projectId }),

  analyzeImage: (projectId: string, imageId: string) =>
    tauriCommand<import("../types").ImageAnalysisResult>("analyze_image_cmd", {
      projectId,
      imageId,
    }),

  listIssues: (projectId: string, imageId?: string) =>
    tauriCommand<import("../types").IssueProposal[]>("list_issues_cmd", {
      projectId,
      imageId,
    }),

  getIssue: (proposalId: string) =>
    tauriCommand<import("../types").IssueProposal>("get_issue_cmd", { proposalId }),

  updateIssueReview: (proposalId: string, reviewStatus: string) =>
    tauriCommand<import("../types").IssueProposal>("update_issue_review_cmd", {
      proposalId,
      reviewStatus,
    }),

  getAnalysis: (analysisId: string) =>
    tauriCommand<import("../types").ImageAnalysis>("get_analysis_cmd", { analysisId }),

  // Phase 8: Issue Lifecycle, Duplicates, Known Issues
  updateIssueLifecycle: (issueId: string, newLifecycle: string) =>
    tauriCommand<import("../types").IssueProposal>("update_issue_lifecycle_cmd", { issueId, newLifecycle }),

  findDuplicateCandidates: (projectId: string, issueId: string) =>
    tauriCommand<[import("../types").DuplicateRelationship, import("../types").IssueProposal][]>("find_duplicate_candidates_cmd", { projectId, issueId }),

  confirmDuplicate: (relationshipId: string) =>
    tauriCommand<import("../types").DuplicateRelationship>("confirm_duplicate_cmd", { relationshipId }),

  listDuplicateRelationships: (projectId: string) =>
    tauriCommand<import("../types").DuplicateRelationship[]>("list_duplicate_relationships_cmd", { projectId }),

  createKnownIssue: (projectId: string, title: string, description: string, category: string, subcategory: string, bugType: string, severity: string, sourceIssueId: string) =>
    tauriCommand<import("../types").KnownIssue>("create_known_issue_cmd", { id: "ki-" + crypto.randomUUID(), projectId, title, description, category, subcategory, bugType, severity, sourceIssueId }),

  listKnownIssues: (projectId: string) =>
    tauriCommand<import("../types").KnownIssue[]>("list_known_issues_cmd", { projectId }),

  deactivateKnownIssue: (id: string) =>
    tauriCommand<void>("deactivate_known_issue_cmd", { id }),
};

export const evidenceApi = {
  generateProposals: (projectId: string, issueProposalId: string) =>
    tauriCommand<EvidenceCandidate[]>("generate_evidence_proposals_cmd", {
      projectId,
      issueProposalId,
    }),

  listByIssue: (projectId: string, issueProposalId: string) =>
    tauriCommand<EvidenceAnnotation[]>("list_evidence_by_issue_cmd", {
      projectId,
      issueProposalId,
    }),

  getAnnotation: (projectId: string, annotationId: string) =>
    tauriCommand<EvidenceAnnotation>("get_evidence_annotation_cmd", {
      projectId,
      annotationId,
    }),

  createAnnotation: (
    projectId: string,
    issueProposalId: string,
    annotationType: string,
    x: number,
    y: number,
    width: number,
    height: number,
    origin: string,
  ) =>
    tauriCommand<EvidenceAnnotation>("create_evidence_annotation_cmd", {
      projectId,
      issueProposalId,
      annotationType,
      x,
      y,
      width,
      height,
      origin,
    }),

  updateAnnotation: (
    projectId: string,
    annotationId: string,
    annotationType: string,
    x: number,
    y: number,
    width: number,
    height: number,
  ) =>
    tauriCommand<EvidenceAnnotation>("update_evidence_annotation_cmd", {
      projectId,
      annotationId,
      annotationType,
      x,
      y,
      width,
      height,
    }),

  updateReview: (projectId: string, annotationId: string, reviewState: string) =>
    tauriCommand<EvidenceAnnotation>("update_evidence_review_cmd", {
      projectId,
      annotationId,
      reviewState,
    }),


};

export const videoApi = {
  uploadVideo: (projectId: string, sourcePath: string) =>
    tauriCommand<import('../types').ProjectVideo>('upload_video_cmd', { projectId, sourcePath }),

  listVideos: (projectId: string) =>
    tauriCommand<import('../types').ProjectVideo[]>('list_videos_cmd', { projectId }),

  getVideo: (videoId: string) =>
    tauriCommand<import('../types').ProjectVideo>('get_video_cmd', { videoId }),

  extractFrames: (projectId: string, videoId: string, intervalSecs: number) =>
    tauriCommand<import('../types').VideoExtraction>('extract_frames_cmd', {
      projectId, videoId, intervalSecs,
    }),

  getExtraction: (extractionId: string) =>
    tauriCommand<import('../types').VideoExtraction>('get_extraction_cmd', { extractionId }),

  listExtractions: (videoId: string) =>
    tauriCommand<import('../types').VideoExtraction[]>('list_extractions_cmd', { videoId }),

  listFrames: (extractionId: string) =>
    tauriCommand<import('../types').VideoFrame[]>('list_frames_cmd', { extractionId }),
analyzeFrame: (projectId: string, frameId: string) =>
    tauriCommand<import('../types').ImageAnalysisResult>('analyze_video_frame_cmd', { projectId, frameId }),

  getFrame: (frameId: string) =>
    tauriCommand<import('../types').VideoFrame>('get_video_frame_cmd', { frameId }),

  crossFrameConsistency: (projectId: string, frameIds: string[]) =>
    tauriCommand<import('../types').TerminologyFinding[]>('cross_frame_consistency_cmd', { projectId, frameIds }),
};

export interface BuildInfo {
  version: string;
  build: string;
  channel: string;
  commit: string;
  commit_short: string;
  branch: string;
  tag: string;
  timestamp: string;
  build_machine: { hostname: string; os: string; arch: string };
  toolchain: { rustc: string; node: string; npm: string; python: string };
  tests: {
    cargo_test: { passed: number; failed: number };
    vitest: { passed: number; failed: number };
  };
  artifacts: {
    executable: { filename: string; size_bytes: number; sha256: string };
    installer: { filename: string; size_bytes: number; sha256: string };
  };
}

export const buildApi = {
  getBuildInfo: (): Promise<BuildInfo> =>
    tauriCommand<string>("get_build_info_cmd").then((json) => {
      try {
        return JSON.parse(json) as BuildInfo;
      } catch {
        throw new Error("Build metadata unavailable.");
      }
    }),
};
