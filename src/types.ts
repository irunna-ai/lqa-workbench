/** Core domain types for QAIVRA */

export interface Project {
  id: string;
  name: string;
  target_language: string;
  genre: string;
  platform: string;
  created_at: string;
  updated_at: string;
}

export interface CreateProjectInput {
  name: string;
  target_language: string;
  genre: string;
  platform: string;
}

export type NavigationPage =
  | "dashboard"
  | "analyze-image"
  | "analyze-video"
  | "analyze-document"
  | "issues"
  | "knowledge-rules"
  | "knowledge-glossary"
  | "knowledge-references"
  | "knowledge-brain"
  | "export-evidence"
  | "export-reports"
  | "settings"
  | "settings-ai"
  | "settings-processing"
  | "settings-template";

/** Phase 2: Severity Rules types */

export interface SeverityRule {
  id: string;
  project_id: string;
  source_sheet: string;
  source_row: number;
  main_category_raw: string;
  subcategory_raw: string;
  canonical_bug_type: string;
  severity_exact: string;
  severity_code: string;
  severity_name: string;
  description_raw: string;
  source_filename: string;
  active: boolean;
  created_at: string;
}

export interface SeverityImportResult {
  import_id: string;
  rules_count: number;
  warnings: string[];
  source_filename: string;
  source_sheet: string;
  status: string;
}

export type SeverityReadiness = "MISSING" | "PARSE_WARNING" | "READY";

export interface SeverityStatus {
  has_rules: boolean;
  rule_count: number;
  source_filename: string;
  source_sheet: string;
  warning_count: number;
  import_status: string;
}

/** Phase 3: Glossary types */

export interface GlossaryEntry {
  id: string;
  project_id: string;
  source_term: string;
  target_term: string;
  source_term_normalized: string;
  status: string;
  do_not_translate: boolean;
  banned: boolean;
  notes: string;
  source_filename: string;
  source_sheet: string;
  source_row: number;
  active: boolean;
  created_at: string;
}

export type GlossaryReadiness = "MISSING" | "CONFLICT_REVIEW" | "READY";

export interface GlossaryStatus {
  has_glossary: boolean;
  entry_count: number;
  dnt_count: number;
  banned_count: number;
  conflict_count: number;
  source_filename: string;
  import_status: string;
}

export interface GlossaryHeaderResult {
  success: boolean;
  error?: string;
  filename?: string;
  source_type?: string;
  headers?: string[];
  sample_rows?: string[][];
  sheets?: string[];
  active_sheet?: string;
  requires_mapping?: boolean;
  note?: string;
}

export interface GlossaryImportResult {
  import_id: string;
  entry_count: number;
  duplicate_count: number;
  conflict_count: number;
  warnings: string[];
  source_filename: string;
  status: string;
}

export interface ColumnMapping {
  source_term_col: number;
  target_term_col?: number;
  status_col?: number;
  dnt_col?: number;
  banned_col?: number;
  notes_col?: number;
}

/** Phase 3: Reference Document types */

export interface ReferenceChunk {
  id: string;
  project_id: string;
  source_filename: string;
  source_type: string;
  source_page: number;
  source_sheet: string;
  source_row_start: number;
  source_row_end: number;
  heading: string;
  content: string;
  chunk_order: number;
  active: boolean;
  created_at: string;
}

export interface ReferenceStatus {
  has_references: boolean;
  chunk_count: number;
  source_count: number;
  import_status: string;
}

export interface ReferenceImportResult {
  import_id: string;
  chunk_count: number;
  warnings: string[];
  source_filename: string;
  source_type: string;
  status: string;
}

/** Phase 3: Project Brain types */

export interface PrecedentReport {
  id: string;
  project_id: string;
  source_filename: string;
  source_type: string;
  sheet_name: string;
  page_number: number;
  row_start: number;
  row_end: number;
  heading: string;
  content: string;
  issue_type: string;
  severity_exact: string;
  terminology_used: string;
  chunk_order: number;
  classification: string;
  active: boolean;
  created_at: string;
}

export interface PrecedentImportResult {
  import_id: string;
  chunk_count: number;
  warnings: string[];
  source_filename: string;
  source_type: string;
  classification: string;
  status: string;
}

export interface GlossaryConflict {
  id: string;
  project_id: string;
  source_term_normalized: string;
  resolution: string;
  chosen_entry_id: string | null;
  created_at: string;
  resolved_at: string | null;
}

export interface ProjectBrainResult {
  glossary_results: GlossaryEntry[];
  reference_results: ReferenceChunk[];
  severity_results: SeverityRule[];
  precedent_results: PrecedentReport[];
  result_count: number;
}

/** Phase 4: AI Provider types */

export interface AiProviderConfig {
  id: string;
  project_id: string;
  provider: string;
  base_url: string;
  selected_model: string;
  model_capabilities_json: string;
  health_status: string;
  last_health_check: string | null;
  created_at: string;
  updated_at: string;
  temperature: number;
  max_tokens: number;
  timeout_secs: number;
  api_key_ref: string;
  provider_display_name: string;
}

export interface OllamaHealthResult {
  healthy: boolean;
  version: string;
  base_url: string;
  error: string | null;
}

export interface OllamaModel {
  name: string;
  size_bytes: number;
  parameter_size: string;
  family: string;
  capabilities: ModelCapabilities;
}

export interface ModelCapabilities {
  vision: boolean;
  text_generation: boolean;
  structured_output: boolean;
}

export interface TestConnectionResult {
  success: boolean;
  latency_ms: number;
  message: string;
  version: string | null;
}

export interface OcrTextRegion {
  text: string;
  confidence: number;
  bbox_x: number;
  bbox_y: number;
  bbox_width: number;
  bbox_height: number;
}

export interface OcrResult {
  success: boolean;
  regions: OcrTextRegion[];
  warnings: string[];
  error: string | null;
}

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

/** Phase 5: Image LQA Analysis types */

export interface ProjectImage {
  id: string;
  project_id: string;
  filename: string;
  file_path: string;
  mime_type: string;
  file_size: number;
  width: number;
  height: number;
  content_hash: string;
  ocr_text: string;
  active: boolean;
  created_at: string;
}

export interface ImageAnalysis {
  id: string;
  project_id: string;
  image_id: string;
  status: string; // pending | running | completed | failed
  analyzer_version: string;
  severity_snapshot_id: string;
  glossary_snapshot_id: string;
  precedent_snapshot_ids: string;
  reference_snapshot_ids: string;
  error_message: string | null;
  proposal_count: number;
  created_at: string;
  completed_at: string | null;
}

export interface IssueProposal {
  id: string;
  project_id: string;
  analysis_id: string;
  image_id: string;
  proposal_index: number;
  issue_category: string;
  issue_subcategory: string;
  bug_type: string;
  title: string;
  description: string;
  detected_text: string;
  glossary_findings: string;
  severity_candidate: string; // UNRESOLVED | <matched rule severity>
  severity_reasoning: string;
  severity_rule_id: string;
  confidence: number;
  review_status: string; // pending | approved | rejected | review_required
  coord_system: string;
  image_width: number;
  image_height: number;
  bbox_x: number;
  bbox_y: number;
  bbox_w: number;
  bbox_h: number;
  spatial_description: string;
  spatial_confidence: number;
  precedent_ids: string;
  reference_ids: string;
  active: boolean;
  lifecycle: string;
  created_at: string;
}

export interface TerminologyFinding {
  kind: string; // banned_term | dnt_violation | mismatch
  observed: string;
  expected: string;
  source_image_id: string;
  source_analysis_id: string;
  note: string;
}

export interface ImageAnalysisResult {
  analysis: ImageAnalysis;
  proposals: IssueProposal[];
  terminology_findings: TerminologyFinding[];
}

/** Phase 6: Visual Evidence Engine types */

export interface EvidenceAnnotation {
  id: string;
  project_id: string;
  issue_proposal_id: string;
  project_image_id: string;
  analysis_id: string;
  annotation_type: string; // RED_BOX | RED_BRACKET
  coordinate_space: string; // NORMALIZED
  x: number;
  y: number;
  width: number;
  height: number;
  origin: string; // AI_PROPOSED | OCR_DERIVED | USER_CREATED | USER_EDITED
  confidence: number | null;
  review_state: string; // PROPOSED | REVIEWED | APPROVED | REJECTED
  created_at: string;
  updated_at: string;
}

export interface EvidenceCandidate {
  annotation_type: string;
  coordinate_space: string;
  x: number;
  y: number;
  width: number;
  height: number;
  origin: string;
  confidence: number | null;
}

/** Phase 7: Video types */

export interface ProjectVideo {
  id: string;
  project_id: string;
  original_filename: string;
  stored_path: string;
  content_hash: string;
  file_size: number;
  media_type: string;
  duration_secs: number;
  width: number;
  height: number;
  frame_rate: number;
  active: number;
  created_at: string;
}

export interface VideoExtraction {
  id: string;
  project_id: string;
  video_id: string;
  extraction_strategy: string;
  interval_secs: number;
  status: string;
  extractor_version: string;
  total_frames: number;
  error_message: string | null;
  started_at: string;
  completed_at: string | null;
}

export interface VideoFrame {
  id: string;
  project_id: string;
  video_id: string;
  extraction_id: string;
  frame_index: number;
  timestamp_ms: number;
  timestamp_display: string;
  image_path: string;
  content_hash: string;
  width: number;
  height: number;
  created_at: string;
}export interface DuplicateRelationship {
  id: string;
  project_id: string;
  source_issue_id: string;
  duplicate_issue_id: string;
  confidence: number;
  match_signals: string;
  confirmed: boolean;
  confirmed_by: string;
  created_at: string;
  updated_at: string;
}

export interface KnownIssue {
  id: string;
  project_id: string;
  title: string;
  description: string;
  category: string;
  subcategory: string;
  bug_type: string;
  severity: string;
  source_issue_id: string;
  active: boolean;
  created_at: string;
  updated_at: string;
}
export interface Build {
  id: string;
  project_id: string;
  version_label: string;
  platform: string;
  environment: string;
  notes: string;
  created_at: string;
}

export type RegressionStatus = 'new' | 'recurring' | 'potentially_fixed' | 'fixed' | 'reopened';
