#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod db;
mod ai_provider;
mod ocr_adapter;
mod image_analysis;
mod evidence_engine;
mod video_engine;
mod issue_lifecycle;
mod export_engine;
mod provider_router;
mod tracker_connector;

use db::{initialize_db, create_project, get_projects, get_project, delete_project, rename_project,
         Project, SeverityRule, SeverityImportResult, ParseResult, import_severity_rules,
         list_severity_rules, get_severity_import_status,
         GlossaryEntry, GlossaryImportResult as DbGlossaryImportResult,
         ParsedGlossaryEntry, ReferenceChunk, ParsedReferenceChunk,
         ReferenceImportResult as DbReferenceImportResult,
         import_glossary_entries, list_glossary_entries,
         import_reference_chunks, list_reference_chunks,
         search_glossary, search_reference_chunks,
         PrecedentReport, PrecedentImportResult as DbPrecedentImportResult,
         ParsedPrecedentRow, GlossaryConflictInfo,
         import_precedent_report, list_precedent_reports,
         search_precedent_reports, get_glossary_conflicts,
         count_unresolved_conflicts, resolve_glossary_conflict,
         AiProviderConfig, OllamaHealthResult, OllamaModel, OcrResult,
         save_ai_provider_config, get_ai_provider_config, update_ai_health,
         validate_candidate_issue, CandidateIssueInput, ValidationResult,
         ProjectImage, ImageAnalysis, ImageAnalysisResult, IssueProposal, TerminologyFinding,
         upload_project_image, list_project_images, list_issue_proposals,
         get_issue_proposal, update_issue_proposal_review, get_image_analysis,
         EvidenceAnnotation, get_ocr_regions, get_project_image, insert_evidence_annotation,
         get_evidence_annotation, list_evidence_by_issue, update_evidence_annotation,
         update_evidence_review_state,
         ProjectVideo, VideoExtraction, VideoFrame,
         insert_project_video, list_project_videos, get_project_video,
         insert_video_extraction, complete_video_extraction, get_video_extraction,
         list_video_extractions, insert_video_frame, list_video_frames, get_video_frame, register_frame_image,
         DuplicateRelationship, KnownIssue,
         insert_duplicate_relationship, get_duplicate_relationship, list_duplicate_relationships,
         confirm_duplicate_relationship, find_duplicate_candidates,
         create_known_issue, get_known_issue, list_known_issues, deactivate_known_issue,
         update_issue_lifecycle,
         Build, RegressionStatus,
         create_build, get_build, list_builds, assign_image_build, assign_video_build, determine_regression,
         LocalizationRecord, StringMapping, ParsedLocalizationRecord,
         import_localization_records, list_localization_records,
         find_string_mapping_candidates, insert_string_mapping, confirm_string_mapping, list_string_mappings,
         parse_csv_localization, parse_json_localization,
         AnalysisJob, create_analysis_job, get_analysis_job, list_analysis_jobs, update_job_status, increment_job_retry,
         create_tracker_config, list_tracker_configs, insert_submission_record, list_submission_records,
         AuditEvent, record_audit_event, list_audit_events};
use evidence_engine::{EvidenceCandidate, IssueLite, OcrLite, AnnotationType, CoordinateSpace,
         EvidenceOrigin, ReviewState, validate_normalized_region};
use video_engine::{validate_video_path, probe_video, validate_interval, compute_frame_count,
         format_timestamp};
use rusqlite::Connection;
use std::sync::Mutex;
use std::process::Command as StdCommand;
use tauri::{Manager, State};

struct AppState {
    db: Mutex<Connection>,
}

#[tauri::command]
async fn create_new_project(
    name: String,
    target_language: String,
    genre: String,
    platform: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let conn = state.db.lock().unwrap();
    create_project(&conn, &id, &name, &target_language, &genre, &platform)
}

#[tauri::command]
async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().unwrap();
    get_projects(&conn)
}

#[tauri::command]
async fn get_project_by_id(id: String, state: State<'_, AppState>) -> Result<Project, String> {
    let conn = state.db.lock().unwrap();
    get_project(&conn, &id)
}

#[tauri::command]
async fn delete_project_by_id(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    delete_project(&conn, &id)
}

#[tauri::command]
async fn rename_project_by_id(
    id: String,
    new_name: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let conn = state.db.lock().unwrap();
    rename_project(&conn, &id, &new_name)
}

#[tauri::command]
async fn import_severity_rules_cmd(
    project_id: String,
    workbook_path: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<SeverityImportResult, String> {
    // Locate the Python parser script
    let parser_path = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?
        .join("parsers")
        .join("severity_rules_parser.py");

    // If parser not in resource dir, try relative to the executable
    let parser_path = if parser_path.exists() {
        parser_path
    } else {
        // Fallback: look relative to the app's current directory
        let exe_dir = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?
            .parent()
            .ok_or("No exe parent dir")?
            .to_path_buf();
        let candidate = exe_dir.join("parsers").join("severity_rules_parser.py");
        if candidate.exists() {
            candidate
        } else {
            // Development mode: look in project root
            let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("parsers")
                .join("severity_rules_parser.py");
            if dev_path.exists() {
                dev_path
            } else {
                return Err(format!("Parser not found. Tried: {:?}", parser_path));
            }
        }
    };

    // Invoke Python parser (PYTHONIOENCODING ensures UTF-8 stdout on Windows)
    let output = StdCommand::new("uv")
        .env("PYTHONIOENCODING", "utf-8")
        .args([
            "run",
            "--with", "openpyxl",
            "python",
            parser_path.to_str().ok_or("Invalid parser path")?,
            &workbook_path,
        ])
        .output()
        .map_err(|e| format!("Failed to execute parser: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Parser failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parse_result: ParseResult = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse parser output: {}. Raw: {}", e, &stdout[..stdout.len().min(200)]))?;

    if !parse_result.success {
        return Err(parse_result.error.unwrap_or_else(|| "Parse failed".to_string()));
    }

    let conn = state.db.lock().unwrap();
    import_severity_rules(&conn, &project_id, &parse_result)
}

#[tauri::command]
async fn list_severity_rules_cmd(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SeverityRule>, String> {
    let conn = state.db.lock().unwrap();
    list_severity_rules(&conn, &project_id)
}

#[derive(serde::Serialize)]
struct SeverityStatus {
    has_rules: bool,
    rule_count: usize,
    source_filename: String,
    source_sheet: String,
    warning_count: i32,
    import_status: String,
}

#[tauri::command]
async fn get_severity_status_cmd(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<SeverityStatus, String> {
    let conn = state.db.lock().unwrap();

    let rules = list_severity_rules(&conn, &project_id)?;
    let rule_count = rules.len();

    let import_info = get_severity_import_status(&conn, &project_id)?;

    let (source_filename, source_sheet, warning_count, import_status) = if let Some((fn_, sh, _rc, wc, st)) = import_info {
        (fn_, sh, wc, st)
    } else {
        (String::new(), String::new(), 0, "none".to_string())
    };

    Ok(SeverityStatus {
        has_rules: rule_count > 0,
        rule_count,
        source_filename,
        source_sheet,
        warning_count,
        import_status,
    })
}

// ---- Phase 3: Glossary & Reference Commands ----

fn find_parser_script(_app: &tauri::AppHandle, script_name: &str) -> Result<std::path::PathBuf, String> {
    let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("parsers").join(script_name);
    if dev_path.exists() { return Ok(dev_path); }
    Err(format!("Parser '{}' not found", script_name))
}

#[derive(serde::Deserialize, serde::Serialize)]
struct GlossaryHeaderResult {
    success: bool, error: Option<String>, filename: Option<String>,
    source_type: Option<String>, headers: Option<Vec<String>>,
    sample_rows: Option<Vec<Vec<String>>>, sheets: Option<Vec<String>>,
    active_sheet: Option<String>, requires_mapping: Option<bool>, note: Option<String>,
}

#[derive(serde::Deserialize)]
struct GlossaryImportResponse {
    success: bool, error: Option<String>,
    entries: Option<Vec<ParsedGlossaryEntry>>, warnings: Option<Vec<String>>,
}

#[derive(serde::Deserialize)]
struct ReferenceExtractResponse {
    success: bool, error: Option<String>,
    chunks: Option<Vec<ParsedReferenceChunk>>, warnings: Option<Vec<String>>,
}

#[tauri::command]
async fn parse_glossary_headers_cmd(file_path: String, sheet_name: Option<String>, app: tauri::AppHandle)
    -> Result<GlossaryHeaderResult, String>
{
    let parser_path = find_parser_script(&app, "glossary_parser.py")?;
    let mut args = vec!["run", "--with", "openpyxl", "python",
                        parser_path.to_str().ok_or("Invalid path")?, "parse_headers", &file_path];
    if let Some(ref s) = sheet_name { args.push(s); }
    let output = StdCommand::new("uv").env("PYTHONIOENCODING", "utf-8").args(&args).output()
        .map_err(|e| format!("Failed to execute parser: {}", e))?;
    if !output.status.success() { return Err(format!("Parser failed: {}", String::from_utf8_lossy(&output.stderr))); }
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).map_err(|e| format!("Parse error: {}", e))
}

#[tauri::command]
async fn import_glossary_cmd(project_id: String, file_path: String, column_mapping: serde_json::Value,
    sheet_name: Option<String>, state: State<'_, AppState>, app: tauri::AppHandle)
    -> Result<DbGlossaryImportResult, String>
{
    let parser_path = find_parser_script(&app, "glossary_parser.py")?;
    let mapping_str = serde_json::to_string(&column_mapping).unwrap_or_default();
    let mut args = vec!["run", "--with", "openpyxl", "python",
                        parser_path.to_str().ok_or("Invalid path")?,
                        "import_with_mapping", &file_path, &mapping_str];
    if let Some(ref s) = sheet_name { args.push(s); }
    let output = StdCommand::new("uv").env("PYTHONIOENCODING", "utf-8").args(&args).output()
        .map_err(|e| format!("Failed to execute parser: {}", e))?;
    if !output.status.success() { return Err(format!("Parser failed: {}", String::from_utf8_lossy(&output.stderr))); }
    let result: GlossaryImportResponse = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .map_err(|e| format!("Parse error: {}", e))?;
    if !result.success { return Err(result.error.unwrap_or_else(|| "Import failed".to_string())); }
    let entries = result.entries.unwrap_or_default();
    let warnings = result.warnings.unwrap_or_default();
    let filename = std::path::Path::new(&file_path).file_name().unwrap_or_default().to_string_lossy().to_string();
    let conn = state.db.lock().unwrap();
    import_glossary_entries(&conn, &project_id, &filename, "xlsx", &entries, &warnings)
}

#[tauri::command]
async fn import_reference_doc_cmd(project_id: String, file_path: String,
    state: State<'_, AppState>, app: tauri::AppHandle)
    -> Result<DbReferenceImportResult, String>
{
    let parser_path = find_parser_script(&app, "glossary_parser.py")?;
    let output = StdCommand::new("uv").env("PYTHONIOENCODING", "utf-8").args([
        "run", "--with", "openpyxl", "--with", "python-docx", "--with", "pypdf",
        "python", parser_path.to_str().ok_or("Invalid path")?, "extract_text", &file_path,
    ]).output().map_err(|e| format!("Failed to execute parser: {}", e))?;
    if !output.status.success() { return Err(format!("Parser failed: {}", String::from_utf8_lossy(&output.stderr))); }
    let result: ReferenceExtractResponse = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .map_err(|e| format!("Parse error: {}", e))?;
    if !result.success { return Err(result.error.unwrap_or_else(|| "Extract failed".to_string())); }
    let chunks = result.chunks.unwrap_or_default();
    let warnings = result.warnings.unwrap_or_default();
    let filename = std::path::Path::new(&file_path).file_name().unwrap_or_default().to_string_lossy().to_string();
    let ext = std::path::Path::new(&file_path).extension().unwrap_or_default().to_string_lossy().to_string();
    let conn = state.db.lock().unwrap();
    import_reference_chunks(&conn, &project_id, &filename, &ext, &chunks, &warnings)
}

#[tauri::command]
async fn list_glossary_entries_cmd(project_id: String, state: State<'_, AppState>)
    -> Result<Vec<GlossaryEntry>, String> { list_glossary_entries(&state.db.lock().unwrap(), &project_id) }

#[tauri::command]
async fn list_reference_docs_cmd(project_id: String, state: State<'_, AppState>)
    -> Result<Vec<ReferenceChunk>, String> { list_reference_chunks(&state.db.lock().unwrap(), &project_id) }

#[derive(serde::Serialize)]
struct GlossaryStatus {
    has_glossary: bool, entry_count: usize, dnt_count: usize, banned_count: usize,
    conflict_count: usize, source_filename: String, import_status: String,
}

#[tauri::command]
async fn get_glossary_status_cmd(project_id: String, state: State<'_, AppState>)
    -> Result<GlossaryStatus, String>
{
    let conn = state.db.lock().unwrap();
    let entries = list_glossary_entries(&conn, &project_id)?;
    let ec = entries.len();
    let conflicts = count_unresolved_conflicts(&conn, &project_id).unwrap_or(0);
    Ok(GlossaryStatus {
        has_glossary: ec > 0, entry_count: ec,
        dnt_count: entries.iter().filter(|e| e.do_not_translate).count(),
        banned_count: entries.iter().filter(|e| e.banned).count(),
        conflict_count: conflicts,
        source_filename: entries.first().map(|e| e.source_filename.clone()).unwrap_or_default(),
        import_status: if ec > 0 { "completed" } else { "none" }.to_string(),
    })
}

#[derive(serde::Serialize)]
struct ReferenceStatus { has_references: bool, chunk_count: usize, source_count: usize, import_status: String }

#[tauri::command]
async fn get_reference_status_cmd(project_id: String, state: State<'_, AppState>)
    -> Result<ReferenceStatus, String>
{
    let conn = state.db.lock().unwrap();
    let chunks = list_reference_chunks(&conn, &project_id)?;
    let cc = chunks.len();
    let sc = chunks.iter().map(|c| &c.source_filename).collect::<std::collections::HashSet<_>>().len();
    Ok(ReferenceStatus {
        has_references: cc > 0, chunk_count: cc, source_count: sc,
        import_status: if cc > 0 { "completed" } else { "none" }.to_string(),
    })
}

#[derive(serde::Serialize)]
struct ProjectBrainResult {
    glossary_results: Vec<GlossaryEntry>, reference_results: Vec<ReferenceChunk>,
    severity_results: Vec<SeverityRule>, precedent_results: Vec<PrecedentReport>,
    result_count: usize,
}

#[tauri::command]
async fn query_project_brain_cmd(project_id: String, query: String, state: State<'_, AppState>)
    -> Result<ProjectBrainResult, String>
{
    let conn = state.db.lock().unwrap();
    let gr = search_glossary(&conn, &project_id, &query).unwrap_or_default();
    let rr = search_reference_chunks(&conn, &project_id, &query).unwrap_or_default();
    let all_rules = list_severity_rules(&conn, &project_id).unwrap_or_default();
    let ql = query.to_lowercase();
    let sr: Vec<SeverityRule> = all_rules.into_iter()
        .filter(|r| r.canonical_bug_type.to_lowercase().contains(&ql)
                  || r.description_raw.to_lowercase().contains(&ql))
        .collect();
    let pr = search_precedent_reports(&conn, &project_id, &query).unwrap_or_default();
    let rc = gr.len() + rr.len() + sr.len() + pr.len();
    Ok(ProjectBrainResult { glossary_results: gr, reference_results: rr, severity_results: sr, precedent_results: pr, result_count: rc })
}

#[tauri::command]
async fn import_precedent_cmd(
    project_id: String, file_path: String, classification: String,
    state: State<'_, AppState>, app: tauri::AppHandle,
) -> Result<DbPrecedentImportResult, String> {
    let parser_path = find_parser_script(&app, "glossary_parser.py")?;
    let output = StdCommand::new("uv").env("PYTHONIOENCODING", "utf-8").args([
        "run", "--with", "openpyxl", "--with", "python-docx", "--with", "pypdf",
        "python", parser_path.to_str().ok_or("Invalid path")?, "extract_text", &file_path,
    ]).output().map_err(|e| format!("Failed to execute parser: {}", e))?;
    if !output.status.success() { return Err(format!("Parser failed: {}", String::from_utf8_lossy(&output.stderr))); }
    let result: ReferenceExtractResponse = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .map_err(|e| format!("Parse error: {}", e))?;
    if !result.success { return Err(result.error.unwrap_or_else(|| "Extract failed".to_string())); }
    let chunks = result.chunks.unwrap_or_default();
    let warnings = result.warnings.unwrap_or_default();
    let filename = std::path::Path::new(&file_path).file_name().unwrap_or_default().to_string_lossy().to_string();
    let ext = std::path::Path::new(&file_path).extension().unwrap_or_default().to_string_lossy().to_string();
    let parsed_rows: Vec<ParsedPrecedentRow> = chunks.into_iter().map(|c| ParsedPrecedentRow {
        sheet_name: c.source_sheet.clone(), page_number: c.source_page,
        row_start: c.source_row_start, row_end: c.source_row_end,
        heading: c.heading.clone(), content: c.content.clone(),
        issue_type: String::new(), severity_exact: String::new(),
        terminology_used: String::new(), chunk_order: c.chunk_order,
    }).collect();
    let conn = state.db.lock().unwrap();
    import_precedent_report(&conn, &project_id, &filename, &ext, &classification, &parsed_rows, &warnings)
}

#[tauri::command]
async fn list_precedent_reports_cmd(project_id: String, state: State<'_, AppState>)
    -> Result<Vec<PrecedentReport>, String> { list_precedent_reports(&state.db.lock().unwrap(), &project_id) }

#[tauri::command]
async fn get_glossary_conflicts_cmd(project_id: String, state: State<'_, AppState>)
    -> Result<Vec<GlossaryConflictInfo>, String> { get_glossary_conflicts(&state.db.lock().unwrap(), &project_id) }

#[tauri::command]
async fn get_glossary_conflict_count_cmd(project_id: String, state: State<'_, AppState>)
    -> Result<usize, String> { count_unresolved_conflicts(&state.db.lock().unwrap(), &project_id) }

#[tauri::command]
async fn resolve_glossary_conflict_cmd(conflict_id: String, chosen_entry_id: String, state: State<'_, AppState>)
    -> Result<(), String> { resolve_glossary_conflict(&state.db.lock().unwrap(), &conflict_id, &chosen_entry_id) }

// ---- Phase 4: Local AI and OCR Foundation Commands ----

#[tauri::command]
async fn check_ollama_health_cmd(base_url: String) -> Result<OllamaHealthResult, String> {
    Ok(ai_provider::check_health(&base_url))
}

#[tauri::command]
async fn list_ollama_models_cmd(base_url: String) -> Result<Vec<OllamaModel>, String> {
    ai_provider::list_models(&base_url)
}

#[tauri::command]
async fn get_model_capabilities_cmd(base_url: String, model_name: String) -> Result<db::ModelCapabilities, String> {
    ai_provider::get_model_capabilities(&base_url, &model_name)
}

#[tauri::command]
async fn save_ai_config_cmd(
    project_id: String, provider: String, base_url: String,
    selected_model: String, capabilities_json: String, health_status: String,
    temperature: f64, max_tokens: i64, timeout_secs: i64,
    api_key_ref: String, provider_display_name: String,
    state: State<'_, AppState>,
) -> Result<AiProviderConfig, String> {
    save_ai_provider_config(&state.db.lock().unwrap(), &project_id, &provider, &base_url,
        &selected_model, &capabilities_json, &health_status,
        temperature, max_tokens, timeout_secs, &api_key_ref, &provider_display_name)
}

#[tauri::command]
async fn get_ai_config_cmd(project_id: String, state: State<'_, AppState>) -> Result<AiProviderConfig, String> {
    get_ai_provider_config(&state.db.lock().unwrap(), &project_id)
}

#[tauri::command]
async fn update_ai_health_cmd(project_id: String, health_status: String, state: State<'_, AppState>) -> Result<(), String> {
    update_ai_health(&state.db.lock().unwrap(), &project_id, &health_status)
}

#[tauri::command]
async fn run_ocr_cmd(image_path: String, app: tauri::AppHandle) -> Result<OcrResult, String> {
    let parser_dir = app.path().resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?.join("parsers");
    let parser_dir = if parser_dir.exists() { parser_dir } else {
        std::env::current_exe().map_err(|e| format!("Failed to get exe path: {}", e))?
            .parent().ok_or("No exe parent dir")?.to_path_buf().join("parsers")
    };
    ocr_adapter::run_ocr(&image_path, &parser_dir)
}

#[tauri::command]
async fn validate_candidate_cmd(
    candidate: CandidateIssueInput, project_id: String, state: State<'_, AppState>,
) -> Result<ValidationResult, String> {
    let conn = state.db.lock().unwrap();
    let rules = list_severity_rules(&conn, &project_id).unwrap_or_default();
    let bug_types: Vec<String> = rules.iter().map(|r| r.canonical_bug_type.clone()).collect();
    Ok(validate_candidate_issue(&candidate, &bug_types))
}

/// Resolve the OCR/parser scripts directory (resource dir, else exe-relative).
fn parser_dir_for(app: &tauri::AppHandle) -> std::path::PathBuf {
    let res = app
        .path()
        .resource_dir()
        .map(|d| d.join("parsers"))
        .unwrap_or_else(|_| std::path::PathBuf::new());
    if res.exists() {
        res
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("parsers"))
            .unwrap_or(res)
    }
}

/// Image store directory for a project: <app_data>/images/<project_id>
/// `project_id` is validated as a safe path segment to prevent traversal.
fn image_store_dir(app: &tauri::AppHandle, project_id: &str) -> Result<std::path::PathBuf, String> {
    if !db::is_safe_id_segment(project_id) {
        return Err(format!("Invalid project identifier: {}", project_id));
    }
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let dir = base.join("images").join(project_id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create image dir: {}", e))?;
    Ok(dir)
}

// Image mime/extension handling moved to `db::validate_image_upload`
// (Security Gate: extension is validated against content magic bytes).

#[tauri::command]
async fn upload_image_cmd(
    project_id: String,
    source_path: String,
    app: tauri::AppHandle,
) -> Result<ProjectImage, String> {
    // Validate BEFORE any filesystem write: extension + magic-byte + size +
    // symlink checks. Returns a canonical, scoped path we control.
    let validated = db::validate_image_upload(&source_path)?;
    let store = image_store_dir(&app, &project_id)?;
    // UUID-prefix the stored filename to avoid collisions and any crafted-name issues.
    let unique_name = format!("{}-{}", uuid::Uuid::new_v4(), validated.filename);
    let dest = store.join(&unique_name);
    std::fs::copy(&validated.canonical_path, &dest)
        .map_err(|e| format!("Failed to copy image: {}", e))?;

    let content_hash = db::hash_file_contents(&dest.to_string_lossy())?;
    let state = app.state::<AppState>();
    // Read actual image dimensions for proper thumbnail rendering and
    // evidence annotation spatial calculations.
    let (width, height) = match image::io::Reader::open(&dest) {
        Ok(reader) => match reader.into_dimensions() {
            Ok(dims) => (dims.0 as i32, dims.1 as i32),
            Err(_) => (0, 0),
        },
        Err(_) => (0, 0),
    };
    let conn = state.db.lock().unwrap();
    upload_project_image(
        &conn, &project_id, &unique_name, &dest.to_string_lossy(),
        &validated.mime_type, validated.file_size, width, height, &content_hash,
    )
}

#[tauri::command]
async fn list_images_cmd(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ProjectImage>, String> {
    list_project_images(&state.db.lock().unwrap(), &project_id)
}

#[tauri::command]
async fn analyze_image_cmd(
    project_id: String,
    image_id: String,
    app: tauri::AppHandle,
) -> Result<ImageAnalysisResult, String> {
    let parser_dir = parser_dir_for(&app);
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    let ai_config = db::get_ai_provider_config(&conn, &project_id)
        .map_err(|_| "No AI model configured for this project. Configure one in Settings â†’ AI Model.".to_string())?;
    image_analysis::run_image_analysis(&conn, &project_id, &image_id, &parser_dir, &ai_config)
}

#[tauri::command]
async fn list_issues_cmd(
    project_id: String,
    image_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<IssueProposal>, String> {
    list_issue_proposals(&state.db.lock().unwrap(), &project_id, image_id.as_deref())
}

#[tauri::command]
async fn get_issue_cmd(
    proposal_id: String,
    state: State<'_, AppState>,
) -> Result<IssueProposal, String> {
    get_issue_proposal(&state.db.lock().unwrap(), &proposal_id)
}

#[tauri::command]
async fn update_issue_review_cmd(
    proposal_id: String,
    review_status: String,
    state: State<'_, AppState>,
) -> Result<IssueProposal, String> {
    update_issue_proposal_review(&state.db.lock().unwrap(), &proposal_id, &review_status)
}

#[tauri::command]
async fn get_analysis_cmd(
    analysis_id: String,
    state: State<'_, AppState>,
) -> Result<ImageAnalysis, String> {
    get_image_analysis(&state.db.lock().unwrap(), &analysis_id)
}

// ---- Phase 6: Visual Evidence Engine Commands ----

#[tauri::command]
async fn generate_evidence_proposals_cmd(
    project_id: String,
    issue_proposal_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<EvidenceCandidate>, String> {
    let conn = state.db.lock().unwrap();
    let issue = get_issue_proposal(&conn, &issue_proposal_id)?;
    if issue.project_id != project_id {
        return Err("Issue proposal does not belong to the provided project.".to_string());
    }
    let ocr = get_ocr_regions(&conn, &project_id, &issue.image_id)?;
    let issue_lite = IssueLite {
        detected_text: issue.detected_text.clone(),
        title: issue.title.clone(),
        coord_system: issue.coord_system.clone(),
        bbox_x: issue.bbox_x,
        bbox_y: issue.bbox_y,
        bbox_w: issue.bbox_w,
        bbox_h: issue.bbox_h,
        confidence: issue.confidence,
    };
    let ocr_lite: Vec<OcrLite> = ocr
        .iter()
        .map(|r| OcrLite {
            text_normalized: r.text_normalized.clone(),
            bbox_x: r.bbox_x,
            bbox_y: r.bbox_y,
            bbox_width: r.bbox_width,
            bbox_height: r.bbox_height,
            ocr_confidence: r.ocr_confidence,
        })
        .collect();
    Ok(evidence_engine::generate_proposals(&issue_lite, &ocr_lite))
}

#[tauri::command]
async fn list_evidence_by_issue_cmd(
    project_id: String,
    issue_proposal_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<EvidenceAnnotation>, String> {
    let conn = state.db.lock().unwrap();
    let issue = get_issue_proposal(&conn, &issue_proposal_id)?;
    if issue.project_id != project_id {
        return Err("Issue proposal does not belong to the provided project.".to_string());
    }
    list_evidence_by_issue(&conn, &project_id, &issue_proposal_id)
}

#[tauri::command]
async fn get_evidence_annotation_cmd(
    project_id: String,
    annotation_id: String,
    state: State<'_, AppState>,
) -> Result<EvidenceAnnotation, String> {
    let conn = state.db.lock().unwrap();
    get_evidence_annotation(&conn, &project_id, &annotation_id)
}

#[tauri::command]
async fn create_evidence_annotation_cmd(
    project_id: String,
    issue_proposal_id: String,
    annotation_type: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    origin: String,
    state: State<'_, AppState>,
) -> Result<EvidenceAnnotation, String> {
    let conn = state.db.lock().unwrap();
    // Validate project scope and issue/image relationships before mutation.
    let issue = get_issue_proposal(&conn, &issue_proposal_id)?;
    if issue.project_id != project_id {
        return Err("Issue proposal does not belong to the provided project.".to_string());
    }
    let image = get_project_image(&conn, &issue.image_id)?;
    if image.project_id != project_id {
        return Err("Source image does not belong to the provided project.".to_string());
    }
    // Validate enumerated contracts (unknown values rejected).
    let _ = AnnotationType::from_str(&annotation_type).map_err(|e| format!("{}", e))?;
    let origin_enum = EvidenceOrigin::from_str(&origin).map_err(|e| format!("{}", e))?;
    // Backend-authoritative coordinate validation.
    validate_normalized_region(x, y, width, height)
        .map_err(|e| format!("Invalid evidence coordinates: {}", e))?;

    let review_state = match origin_enum {
        EvidenceOrigin::UserCreated => ReviewState::Reviewed.as_str().to_string(),
        _ => ReviewState::Proposed.as_str().to_string(),
    };
    let ann = EvidenceAnnotation {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: project_id.clone(),
        issue_proposal_id: issue_proposal_id.clone(),
        project_image_id: issue.image_id.clone(),
        analysis_id: issue.analysis_id.clone(),
        annotation_type,
        coordinate_space: CoordinateSpace::Normalized.as_str().to_string(),
        x,
        y,
        width,
        height,
        origin,
        confidence: None,
        review_state,
        created_at: String::new(),
        updated_at: String::new(),
    };
    insert_evidence_annotation(&conn, &ann)
}

#[tauri::command]
async fn update_evidence_annotation_cmd(
    project_id: String,
    annotation_id: String,
    annotation_type: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    state: State<'_, AppState>,
) -> Result<EvidenceAnnotation, String> {
    let conn = state.db.lock().unwrap();
    // Project scope.
    let existing = get_evidence_annotation(&conn, &project_id, &annotation_id)?;
    // Relationship check: linked issue must belong to the project.
    let issue = get_issue_proposal(&conn, &existing.issue_proposal_id)?;
    if issue.project_id != project_id {
        return Err("Evidence issue does not belong to the provided project.".to_string());
    }
    // Validate contracts.
    let _ = AnnotationType::from_str(&annotation_type).map_err(|e| format!("{}", e))?;
    validate_normalized_region(x, y, width, height)
        .map_err(|e| format!("Invalid evidence coordinates: {}", e))?;
    let ann = EvidenceAnnotation {
        id: existing.id.clone(),
        project_id: existing.project_id.clone(),
        issue_proposal_id: existing.issue_proposal_id.clone(),
        project_image_id: existing.project_image_id.clone(),
        analysis_id: existing.analysis_id.clone(),
        annotation_type,
        coordinate_space: existing.coordinate_space.clone(),
        x,
        y,
        width,
        height,
        origin: EvidenceOrigin::UserEdited.as_str().to_string(),
        confidence: existing.confidence,
        review_state: ReviewState::Reviewed.as_str().to_string(),
        created_at: existing.created_at.clone(),
        updated_at: String::new(),
    };
    update_evidence_annotation(&conn, &ann)
}

#[tauri::command]
async fn update_evidence_review_cmd(
    project_id: String,
    annotation_id: String,
    review_state: String,
    state: State<'_, AppState>,
) -> Result<EvidenceAnnotation, String> {
    let conn = state.db.lock().unwrap();
    // Project scope check (also errors if the annotation does not exist).
    let _ = get_evidence_annotation(&conn, &project_id, &annotation_id)?;
    let _ = ReviewState::from_str(&review_state).map_err(|e| format!("{}", e))?;
    update_evidence_review_state(&conn, &project_id, &annotation_id, &review_state)
}

// === PHASE 13 PROVIDER / BYOK COMMANDS ===

#[tauri::command]
async fn save_provider_key_cmd(
    service: String, username: String, key: String,
) -> Result<(), String> {
    provider_router::store_api_key(&service, &username, &key)
}

#[tauri::command]
async fn get_provider_key_cmd(
    service: String, username: String,
) -> Result<String, String> {
    provider_router::get_api_key(&service, &username)
}

#[tauri::command]
async fn test_provider_connection_cmd(
    provider_type: String, endpoint: String, api_key_ref: String,
) -> Result<provider_router::TestConnectionResult, String> {
    Ok(provider_router::test_connection(&provider_type, &endpoint, &api_key_ref))
}

// === PHASE 12 EXPORT COMMANDS ===

#[tauri::command]
async fn export_report_cmd(
    project_id: String, project_name: String, output_path: String, filter_approved_only: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    export_engine::export_lqa_report(&state.db.lock().unwrap(), &project_id, &project_name, &output_path, filter_approved_only)
}

// === PHASE 11 BATCH JOB QUEUE COMMANDS ===

#[tauri::command]
async fn queue_analysis_job_cmd(
    project_id: String, job_type: String, source_id: String,
    state: State<'_, AppState>,
) -> Result<AnalysisJob, String> {
    let id = format!("job-{}", uuid::Uuid::new_v4());
    create_analysis_job(&state.db.lock().unwrap(), &id, &project_id, &job_type, &source_id)
}

#[tauri::command]
async fn list_analysis_jobs_cmd(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<AnalysisJob>, String> {
    list_analysis_jobs(&state.db.lock().unwrap(), &project_id)
}

#[tauri::command]
async fn update_job_status_cmd(
    job_id: String, status: String, error_message: Option<String>, progress: f64,
    state: State<'_, AppState>,
) -> Result<AnalysisJob, String> {
    update_job_status(&state.db.lock().unwrap(), &job_id, &status, error_message.as_deref(), progress)
}

#[tauri::command]
async fn retry_job_cmd(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<AnalysisJob, String> {
    increment_job_retry(&state.db.lock().unwrap(), &job_id)
}

// === PHASE 10 LOCALIZATION DATA COMMANDS ===

#[tauri::command]
async fn import_localization_cmd(
    project_id: String, source_file: String, format: String, data: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let records = match format.as_str() {
        "csv" => parse_csv_localization(&data)?,
        "json" => parse_json_localization(&data)?,
        _ => return Err(format!("Unsupported format: {}", format)),
    };
    import_localization_records(&state.db.lock().unwrap(), &project_id, &source_file, &records)
}

#[tauri::command]
async fn list_localization_records_cmd(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<LocalizationRecord>, String> {
    list_localization_records(&state.db.lock().unwrap(), &project_id)
}

#[tauri::command]
async fn find_string_mappings_cmd(
    project_id: String, ocr_text: String, image_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<StringMapping>, String> {
    find_string_mapping_candidates(&state.db.lock().unwrap(), &project_id, &ocr_text, &image_id)
}

#[tauri::command]
async fn confirm_string_mapping_cmd(
    mapping_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    confirm_string_mapping(&state.db.lock().unwrap(), &mapping_id)
}

#[tauri::command]
async fn list_string_mappings_cmd(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<StringMapping>, String> {
    list_string_mappings(&state.db.lock().unwrap(), &project_id)
}

// === PHASE 9 BUILD/VERSION COMMANDS ===

#[tauri::command]
async fn create_build_cmd(
    project_id: String, version_label: String, platform: String, environment: String, notes: String,
    state: State<'_, AppState>,
) -> Result<Build, String> {
    let id = format!("bld-{}", uuid::Uuid::new_v4());
    create_build(&state.db.lock().unwrap(), &id, &project_id, &version_label, &platform, &environment, &notes)
}

#[tauri::command]
async fn list_builds_cmd(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Build>, String> {
    list_builds(&state.db.lock().unwrap(), &project_id)
}

#[tauri::command]
async fn assign_image_build_cmd(
    image_id: String, build_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    assign_image_build(&state.db.lock().unwrap(), &image_id, &build_id)
}

#[tauri::command]
async fn assign_video_build_cmd(
    video_id: String, build_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    assign_video_build(&state.db.lock().unwrap(), &video_id, &build_id)
}

#[tauri::command]
async fn determine_regression_cmd(
    project_id: String, current_build_id: String, fingerprint: String, current_issue_id: String,
    state: State<'_, AppState>,
) -> Result<RegressionStatus, String> {
    determine_regression(&state.db.lock().unwrap(), &project_id, &current_build_id, &fingerprint, &current_issue_id)
}

// === PHASE 8 ISSUE LIFECYCLE COMMANDS ===

#[tauri::command]
async fn find_duplicate_candidates_cmd(
    project_id: String,
    issue_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<(DuplicateRelationship, IssueProposal)>, String> {
    find_duplicate_candidates(&state.db.lock().unwrap(), &issue_id, &project_id)
}

#[tauri::command]
async fn confirm_duplicate_cmd(
    relationship_id: String,
    state: State<'_, AppState>,
) -> Result<DuplicateRelationship, String> {
    confirm_duplicate_relationship(&state.db.lock().unwrap(), &relationship_id)
}

#[tauri::command]
async fn list_duplicate_relationships_cmd(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<DuplicateRelationship>, String> {
    list_duplicate_relationships(&state.db.lock().unwrap(), &project_id)
}

#[tauri::command]
async fn create_known_issue_cmd(
    id: String, project_id: String, title: String, description: String,
    category: String, subcategory: String, bug_type: String, severity: String,
    source_issue_id: String,
    state: State<'_, AppState>,
) -> Result<KnownIssue, String> {
    create_known_issue(&state.db.lock().unwrap(), &id, &project_id, &title, &description,
        &category, &subcategory, &bug_type, &severity, &source_issue_id)
}

#[tauri::command]
async fn list_known_issues_cmd(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<KnownIssue>, String> {
    list_known_issues(&state.db.lock().unwrap(), &project_id)
}

#[tauri::command]
async fn deactivate_known_issue_cmd(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    deactivate_known_issue(&state.db.lock().unwrap(), &id)
}

#[tauri::command]
async fn update_issue_lifecycle_cmd(
    issue_id: String,
    new_lifecycle: String,
    state: State<'_, AppState>,
) -> Result<IssueProposal, String> {
    update_issue_lifecycle(&state.db.lock().unwrap(), &issue_id, &new_lifecycle)
}
// === PHASE 7 VIDEO COMMANDS ===

#[tauri::command]
async fn upload_video_cmd(
    project_id: String,
    source_path: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<ProjectVideo, String> {
    let canonical = validate_video_path(std::path::Path::new(&source_path))?;
    if !crate::db::is_safe_id_segment(&project_id) {
        return Err("Invalid project_id".to_string());
    }
    let probe = probe_video(&canonical)?;
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("app data dir: {}", e))?;
    let project_dir = app_data_dir.join("projects").join(&project_id).join("videos");
    std::fs::create_dir_all(&project_dir)
        .map_err(|e| format!("create video dir: {}", e))?;
    let filename = canonical.file_name().and_then(|n| n.to_str()).unwrap_or("video");
    let ext = canonical.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
    let stored_name = format!("{}.{}", uuid::Uuid::new_v4(), ext);
    let dest = project_dir.join(&stored_name);
    std::fs::copy(&canonical, &dest).map_err(|e| format!("copy video: {}", e))?;
    let file_size = std::fs::metadata(&dest)
        .map_err(|e| format!("metadata: {}", e))?.len() as i64;
    let stored_path = dest.to_str().ok_or("invalid stored path")?.to_string();
    let conn = state.db.lock().unwrap();
    insert_project_video(&conn, &project_id, filename, &stored_path, file_size,
        probe.duration_secs, probe.width, probe.height, probe.frame_rate, &probe.media_type)
}

#[tauri::command]
async fn list_videos_cmd(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ProjectVideo>, String> {
    let conn = state.db.lock().unwrap();
    list_project_videos(&conn, &project_id)
}

#[tauri::command]
async fn get_video_cmd(
    video_id: String,
    state: State<'_, AppState>,
) -> Result<ProjectVideo, String> {
    let conn = state.db.lock().unwrap();
    get_project_video(&conn, &video_id)
}

#[tauri::command]
async fn extract_frames_cmd(
    project_id: String,
    video_id: String,
    interval_secs: f64,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<VideoExtraction, String> {
    validate_interval(interval_secs)?;
    let conn = state.db.lock().unwrap();
    let video = get_project_video(&conn, &video_id)?;
    if video.project_id != project_id { return Err("Project mismatch".to_string()); }
    let extraction = insert_video_extraction(&conn, &project_id, &video_id, interval_secs, "qaivra-phase7-1.0")?;
    drop(conn);

    let frame_count = compute_frame_count(video.duration_secs, interval_secs);
    if frame_count == 0 {
        let c2 = state.db.lock().unwrap();
        return complete_video_extraction(&c2, &extraction.id, 0,
            Some("No frames to extract (duration too short for interval)"));
    }

    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("app data dir: {}", e))?;
    let frames_dir = app_data_dir.join("projects").join(&project_id).join("frames").join(&extraction.id);
    std::fs::create_dir_all(&frames_dir).map_err(|e| format!("create frames dir: {}", e))?;
    let frames_pattern = frames_dir.join("frame_%06d.png");
    let pattern_str = frames_pattern.to_str().ok_or("invalid frames pattern")?;

    let output = std::process::Command::new("ffmpeg")
        .args(["-i", &video.stored_path, "-vf", &format!("fps=1/{}", interval_secs),
            "-frames:v", &frame_count.to_string(), "-vsync", "0", pattern_str, "-y"])
        .output()
        .map_err(|e| {
            let c = state.db.lock().unwrap();
            let _ = complete_video_extraction(&c, &extraction.id, 0,
                Some(&format!("FFmpeg not found: {}. Install FFmpeg.", e)));
            format!("FFmpeg not found: {}. Install FFmpeg.", e)
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let c2 = state.db.lock().unwrap();
        let _ = complete_video_extraction(&c2, &extraction.id, 0, Some(&format!("FFmpeg: {}", stderr)));
        return Err(format!("FFmpeg extraction failed: {}", stderr));
    }

    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(&frames_dir)
        .map_err(|e| format!("read frames dir: {}", e))?
        .filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_file()).collect();
    entries.sort();

    let mut total_frames = 0_i32;
    let c2 = state.db.lock().unwrap();
    for (idx, entry) in entries.iter().enumerate() {
        let frame_idx = idx as i32;
        let timestamp_ms = (frame_idx as f64 * interval_secs * 1000.0) as i64;
        let ts_display = format_timestamp(timestamp_ms);
        let path_str = entry.to_str().unwrap_or("");
        let frame = insert_video_frame(&c2, &project_id, &video_id, &extraction.id,
            frame_idx, timestamp_ms, &ts_display, path_str, video.width, video.height)?;
        // Phase 7 bridge: register frame as project_image for Phase 5/6 pipeline
        let fname = format!("frame_{:05}.png", frame_idx);
        let _ = register_frame_image(&c2, &project_id, &frame.id, &fname, path_str, video.width, video.height, "");
        total_frames += 1;
    }
    complete_video_extraction(&c2, &extraction.id, total_frames, None)
}

#[tauri::command]
async fn get_extraction_cmd(
    extraction_id: String,
    state: State<'_, AppState>,
) -> Result<VideoExtraction, String> {
    let conn = state.db.lock().unwrap();
    get_video_extraction(&conn, &extraction_id)
}

#[tauri::command]
async fn list_extractions_cmd(
    video_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<VideoExtraction>, String> {
    let conn = state.db.lock().unwrap();
    list_video_extractions(&conn, &video_id)
}

#[tauri::command]
async fn list_frames_cmd(
    extraction_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<VideoFrame>, String> {
    let conn = state.db.lock().unwrap();
    list_video_frames(&conn, &extraction_id)
}

#[tauri::command]
async fn analyze_video_frame_cmd(
    project_id: String,
    frame_id: String,
    app: tauri::AppHandle,
) -> Result<ImageAnalysisResult, String> {
    let parser_dir = parser_dir_for(&app);
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    let ai_config = db::get_ai_provider_config(&conn, &project_id)
        .map_err(|_| "No AI model configured for this project. Configure one in Settings → AI Model.".to_string())?;
    // Get the frame's registered project_image ID
    let frame = get_video_frame(&conn, &frame_id)?;
    if frame.project_id != project_id {
        return Err("Frame does not belong to the provided project.".to_string());
    }
    let image_id = if frame.frame_image_id.is_empty() {
        // Register the frame image if not already done
        let fname = format!("frame_{:05}.png", frame.frame_index);
        let img = register_frame_image(&conn, &project_id, &frame.id, &fname, &frame.image_path, frame.width, frame.height, &frame.content_hash)?;
        img.id
    } else {
        frame.frame_image_id.clone()
    };
    image_analysis::run_image_analysis(&conn, &project_id, &image_id, &parser_dir, &ai_config)
}

#[tauri::command]
async fn cross_frame_consistency_cmd(
    project_id: String,
    frame_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<Vec<TerminologyFinding>, String> {
    let conn = state.db.lock().unwrap();
    let glossary = db::list_glossary_entries(&conn, &project_id).unwrap_or_default();
    let mut images_ocr: Vec<(String, String)> = Vec::new();
    for fid in &frame_ids {
        let frame = db::get_video_frame(&conn, fid)?;
        if frame.project_id != project_id {
            return Err(format!("Frame {} does not belong to project {}", fid, project_id));
        }
        let img_id = if frame.frame_image_id.is_empty() {
            continue;
        } else {
            &frame.frame_image_id
        };
        let img = db::get_project_image(&conn, img_id)?;
        images_ocr.push((fid.clone(), img.ocr_text.clone()));
    }
    Ok(image_analysis::evaluate_cross_image_consistency(&images_ocr, &glossary))
}
#[tauri::command]
fn get_build_info_cmd() -> Result<String, String> {
    // Embedded by release pipeline via include_str!
    let embedded = include_str!("../build_info.json");
    Ok(embedded.to_string())
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {}", e))?;

            let db_path = app_data_dir.join("lqa_inspector.db");
            let conn = initialize_db(&db_path)?;

            app.manage(AppState {
                db: Mutex::new(conn),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_new_project,
            list_projects,
            get_project_by_id,
            delete_project_by_id,
            rename_project_by_id,
            import_severity_rules_cmd,
            list_severity_rules_cmd,
            get_severity_status_cmd,
            parse_glossary_headers_cmd,
            import_glossary_cmd,
            import_reference_doc_cmd,
            list_glossary_entries_cmd,
            list_reference_docs_cmd,
            get_glossary_status_cmd,
            get_reference_status_cmd,
            query_project_brain_cmd,
            import_precedent_cmd,
            list_precedent_reports_cmd,
            get_glossary_conflicts_cmd,
            get_glossary_conflict_count_cmd,
            resolve_glossary_conflict_cmd,
            check_ollama_health_cmd,
            list_ollama_models_cmd,
            get_model_capabilities_cmd,
            save_ai_config_cmd,
            get_ai_config_cmd,
            update_ai_health_cmd,
            run_ocr_cmd,
            validate_candidate_cmd,
            upload_image_cmd,
            list_images_cmd,
            analyze_image_cmd,
            list_issues_cmd,
            get_issue_cmd,
            update_issue_review_cmd,
            get_analysis_cmd,
            generate_evidence_proposals_cmd,
            list_evidence_by_issue_cmd,
            get_evidence_annotation_cmd,
            create_evidence_annotation_cmd,
            update_evidence_annotation_cmd,
            update_evidence_review_cmd,
            upload_video_cmd,
            list_videos_cmd,
            get_video_cmd,
            extract_frames_cmd,
            get_extraction_cmd,
            list_extractions_cmd,
            list_frames_cmd,
            analyze_video_frame_cmd,
            cross_frame_consistency_cmd,
            find_duplicate_candidates_cmd,
            confirm_duplicate_cmd,
            list_duplicate_relationships_cmd,
            create_known_issue_cmd,
            list_known_issues_cmd,
            deactivate_known_issue_cmd,
            update_issue_lifecycle_cmd,
            get_build_info_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
