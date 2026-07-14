use rusqlite::{Connection, params};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

// Compatibility debt (intentionally preserved from pre-QAIVRA branding):
// - Database filename: "lqa_inspector.db" â€” changing would orphan Phase 1â€“3 data
// - Tauri bundle identifier: "com.lqa.inspector" â€” changing alters app data directory
// Note: Cargo crate name was renamed from "lqa-inspector" to "qaivra" for QAIVRA.exe branding.

#[derive(Debug, Serialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub target_language: String,
    pub genre: String,
    pub platform: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Initialize the database at the given path, creating the parent directory
/// and running all Phase 1 migrations.
pub fn initialize_db(db_path: &PathBuf) -> Result<Connection, String> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create db directory: {}", e))?;
    }

    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    run_migrations(&conn)?;
    Ok(conn)
}

/// Run all Phase 1 schema migrations.
fn run_migrations(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;"
    ).map_err(|e| format!("Failed to set pragmas: {}", e))?;

    // Migration 1: projects table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            target_language TEXT NOT NULL DEFAULT '',
            genre TEXT NOT NULL DEFAULT '',
            platform TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
        [],
    ).map_err(|e| format!("Migration 1 (projects) failed: {}", e))?;

    // Migration 2: knowledge_documents table (schema for Phase 2+)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS knowledge_documents (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            type TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT '',
            filename TEXT NOT NULL,
            stored_path TEXT NOT NULL,
            content_hash TEXT NOT NULL DEFAULT '',
            authority_rank INTEGER NOT NULL DEFAULT 0,
            parse_status TEXT NOT NULL DEFAULT 'pending',
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 2 (knowledge_documents) failed: {}", e))?;

    // Migration 3: migrations tracking
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
        [],
    ).map_err(|e| format!("Migration 3 (_migrations) failed: {}", e))?;

    // Migration 4: severity_rules table (Phase 2)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS severity_rules (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            document_id TEXT,
            source_sheet TEXT NOT NULL DEFAULT '',
            source_row INTEGER NOT NULL DEFAULT 0,
            main_category_raw TEXT NOT NULL DEFAULT '',
            subcategory_raw TEXT NOT NULL DEFAULT '',
            canonical_bug_type TEXT NOT NULL DEFAULT '',
            severity_exact TEXT NOT NULL DEFAULT '',
            severity_code TEXT NOT NULL DEFAULT '',
            severity_name TEXT NOT NULL DEFAULT '',
            description_raw TEXT NOT NULL DEFAULT '',
            source_filename TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 4 (severity_rules) failed: {}", e))?;

    // Migration 5: severity_imports tracking (Phase 2)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS severity_imports (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            source_filename TEXT NOT NULL,
            source_sheet TEXT NOT NULL DEFAULT '',
            rule_count INTEGER NOT NULL DEFAULT 0,
            warning_count INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'pending',
            error_message TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 5 (severity_imports) failed: {}", e))?;

    // Migration 6: glossary_entries (Phase 3)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS glossary_entries (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            document_id TEXT,
            source_term TEXT NOT NULL,
            target_term TEXT NOT NULL DEFAULT '',
            source_term_normalized TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'approved',
            do_not_translate INTEGER NOT NULL DEFAULT 0,
            banned INTEGER NOT NULL DEFAULT 0,
            notes TEXT NOT NULL DEFAULT '',
            source_filename TEXT NOT NULL DEFAULT '',
            source_sheet TEXT NOT NULL DEFAULT '',
            source_row INTEGER NOT NULL DEFAULT 0,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 6 (glossary_entries) failed: {}", e))?;

    // Migration 7: reference_chunks (Phase 3)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS reference_chunks (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            document_id TEXT,
            source_filename TEXT NOT NULL DEFAULT '',
            source_type TEXT NOT NULL DEFAULT '',
            source_page INTEGER NOT NULL DEFAULT 0,
            source_sheet TEXT NOT NULL DEFAULT '',
            source_row_start INTEGER NOT NULL DEFAULT 0,
            source_row_end INTEGER NOT NULL DEFAULT 0,
            heading TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL,
            chunk_order INTEGER NOT NULL DEFAULT 0,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 7 (reference_chunks) failed: {}", e))?;

    // Migration 8: glossary_conflicts (Phase 3)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS glossary_conflicts (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            source_term_normalized TEXT NOT NULL,
            resolution TEXT NOT NULL DEFAULT 'unresolved',
            chosen_entry_id TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            resolved_at TEXT,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 8 (glossary_conflicts) failed: {}", e))?;

    // Migration 9: glossary_imports tracking (Phase 3)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS glossary_imports (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            source_filename TEXT NOT NULL,
            source_type TEXT NOT NULL DEFAULT '',
            content_hash TEXT NOT NULL DEFAULT '',
            entry_count INTEGER NOT NULL DEFAULT 0,
            duplicate_count INTEGER NOT NULL DEFAULT 0,
            conflict_count INTEGER NOT NULL DEFAULT 0,
            warning_count INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'pending',
            error_message TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 9 (glossary_imports) failed: {}", e))?;

    // Migration 10: precedent_reports (Phase 3)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS precedent_reports (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            document_id TEXT,
            source_filename TEXT NOT NULL DEFAULT '',
            source_type TEXT NOT NULL DEFAULT '',
            sheet_name TEXT NOT NULL DEFAULT '',
            page_number INTEGER NOT NULL DEFAULT 0,
            row_start INTEGER NOT NULL DEFAULT 0,
            row_end INTEGER NOT NULL DEFAULT 0,
            heading TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL,
            issue_type TEXT NOT NULL DEFAULT '',
            severity_exact TEXT NOT NULL DEFAULT '',
            terminology_used TEXT NOT NULL DEFAULT '',
            chunk_order INTEGER NOT NULL DEFAULT 0,
            classification TEXT NOT NULL DEFAULT 'unclassified',
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 10 (precedent_reports) failed: {}", e))?;

    // Migration 11: ai_provider_config (Phase 4)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_provider_config (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            provider TEXT NOT NULL DEFAULT 'ollama',
            base_url TEXT NOT NULL DEFAULT 'http://localhost:11434',
            selected_model TEXT NOT NULL DEFAULT '',
            model_capabilities_json TEXT NOT NULL DEFAULT '{}',
            health_status TEXT NOT NULL DEFAULT 'unknown',
            last_health_check TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 11 (ai_provider_config) failed: {}", e))?;

    // Migration 12: analysis_jobs (Phase 4)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS analysis_jobs (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            media_id TEXT,
            mode TEXT NOT NULL DEFAULT 'FULL_LQA',
            status TEXT NOT NULL DEFAULT 'queued',
            stage TEXT NOT NULL DEFAULT '',
            progress_json TEXT NOT NULL DEFAULT '{}',
            provider TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL DEFAULT '',
            settings_json TEXT NOT NULL DEFAULT '{}',
            error_json TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            completed_at TEXT,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 12 (analysis_jobs) failed: {}", e))?;

    // Migration 13: ocr_regions (Phase 4)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ocr_regions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            media_id TEXT,
            frame_id TEXT,
            text_raw TEXT NOT NULL DEFAULT '',
            text_normalized TEXT NOT NULL DEFAULT '',
            bbox_x REAL NOT NULL DEFAULT 0.0,
            bbox_y REAL NOT NULL DEFAULT 0.0,
            bbox_width REAL NOT NULL DEFAULT 0.0,
            bbox_height REAL NOT NULL DEFAULT 0.0,
            ocr_confidence REAL NOT NULL DEFAULT 0.0,
            region_hash TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 13 (ocr_regions) failed: {}", e))?;

    // Migration 14: project_images (Phase 5)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS project_images (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            filename TEXT NOT NULL,
            file_path TEXT NOT NULL,
            mime_type TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            width INTEGER NOT NULL DEFAULT 0,
            height INTEGER NOT NULL DEFAULT 0,
            content_hash TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 14 (project_images) failed: {}", e))?;

    // Migration 15: image_analyses (Phase 5)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS image_analyses (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            image_id TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            analyzer_version TEXT NOT NULL DEFAULT '',
            severity_snapshot_id TEXT NOT NULL DEFAULT '',
            glossary_snapshot_id TEXT NOT NULL DEFAULT '',
            precedent_snapshot_ids TEXT NOT NULL DEFAULT '[]',
            reference_snapshot_ids TEXT NOT NULL DEFAULT '[]',
            error_message TEXT,
            proposal_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            completed_at TEXT,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (image_id) REFERENCES project_images(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 15 (image_analyses) failed: {}", e))?;

    // Migration 16: issue_proposals (Phase 5)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS issue_proposals (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            analysis_id TEXT NOT NULL,
            image_id TEXT NOT NULL,
            proposal_index INTEGER NOT NULL DEFAULT 0,
            issue_category TEXT NOT NULL DEFAULT '',
            issue_subcategory TEXT NOT NULL DEFAULT '',
            bug_type TEXT NOT NULL DEFAULT '',
            title TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            detected_text TEXT NOT NULL DEFAULT '',
            glossary_findings TEXT NOT NULL DEFAULT '',
            severity_candidate TEXT NOT NULL DEFAULT 'UNRESOLVED',
            severity_reasoning TEXT NOT NULL DEFAULT '',
            severity_rule_id TEXT NOT NULL DEFAULT '',
            confidence REAL NOT NULL DEFAULT 0.0,
            review_status TEXT NOT NULL DEFAULT 'pending',
            coord_system TEXT NOT NULL DEFAULT 'normalized',
            image_width INTEGER NOT NULL DEFAULT 0,
            image_height INTEGER NOT NULL DEFAULT 0,
            bbox_x REAL NOT NULL DEFAULT 0.0,
            bbox_y REAL NOT NULL DEFAULT 0.0,
            bbox_w REAL NOT NULL DEFAULT 0.0,
            bbox_h REAL NOT NULL DEFAULT 0.0,
            spatial_description TEXT NOT NULL DEFAULT '',
            spatial_confidence REAL NOT NULL DEFAULT 0.0,
            precedent_ids TEXT NOT NULL DEFAULT '[]',
            reference_ids TEXT NOT NULL DEFAULT '[]',
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (analysis_id) REFERENCES image_analyses(id) ON DELETE CASCADE,
            FOREIGN KEY (image_id) REFERENCES project_images(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 16 (issue_proposals) failed: {}", e))?;

    // Migration 17: add ocr_text column to project_images (Phase 5)
    conn.execute(
        "ALTER TABLE project_images ADD COLUMN ocr_text TEXT NOT NULL DEFAULT ''",
        [],
    ).map_err(|e| format!("Migration 17 (ocr_text column) failed: {}", e)).or_else(|e| {
        // Ignore "duplicate column" errors (already applied on existing DBs)
        if e.contains("duplicate column") { Ok(0) } else { Err(e) }
    })?;

    // Migration 18a: link OCR regions to a specific project image (Phase 6).
    // OCR regions were previously project-scoped only; evidence grounding
    // requires per-image OCR geometry. Existing rows default to ''.
    conn.execute(
        "ALTER TABLE ocr_regions ADD COLUMN image_id TEXT NOT NULL DEFAULT ''",
        [],
    ).map_err(|e| format!("Migration 18a (ocr_regions.image_id) failed: {}", e)).or_else(|e| {
        if e.contains("duplicate column") { Ok(0) } else { Err(e) }
    })?;

    // Migration 18b: evidence_annotations (Phase 6 Visual Evidence Engine).
    conn.execute(
        "CREATE TABLE IF NOT EXISTS evidence_annotations (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            issue_proposal_id TEXT NOT NULL,
            project_image_id TEXT NOT NULL,
            analysis_id TEXT NOT NULL DEFAULT '',
            annotation_type TEXT NOT NULL,
            coordinate_space TEXT NOT NULL DEFAULT 'NORMALIZED',
            x REAL NOT NULL DEFAULT 0.0,
            y REAL NOT NULL DEFAULT 0.0,
            width REAL NOT NULL DEFAULT 0.0,
            height REAL NOT NULL DEFAULT 0.0,
            origin TEXT NOT NULL,
            confidence REAL,
            review_state TEXT NOT NULL DEFAULT 'PROPOSED',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (issue_proposal_id) REFERENCES issue_proposals(id) ON DELETE CASCADE,
            FOREIGN KEY (project_image_id) REFERENCES project_images(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 18b (evidence_annotations) failed: {}", e))?;

    // Indexes for project / issue / image / analysis evidence lookups.
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_evidence_project ON evidence_annotations(project_id)",
        [],
    ).map_err(|e| format!("Migration 18c (evidence project index) failed: {}", e))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_evidence_issue ON evidence_annotations(issue_proposal_id)",
        [],
    ).map_err(|e| format!("Migration 18c (evidence issue index) failed: {}", e))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_evidence_image ON evidence_annotations(project_image_id)",
        [],
    ).map_err(|e| format!("Migration 18c (evidence image index) failed: {}", e))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_evidence_analysis ON evidence_annotations(analysis_id)",
        [],
    ).map_err(|e| format!("Migration 18c (evidence analysis index) failed: {}", e))?;

// Migration 19: project_videos (Phase 7)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS project_videos (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            original_filename TEXT NOT NULL,
            stored_path TEXT NOT NULL,
            content_hash TEXT NOT NULL DEFAULT '',
            file_size INTEGER NOT NULL DEFAULT 0,
            media_type TEXT NOT NULL DEFAULT '',
            duration_secs REAL NOT NULL DEFAULT 0.0,
            width INTEGER NOT NULL DEFAULT 0,
            height INTEGER NOT NULL DEFAULT 0,
            frame_rate REAL NOT NULL DEFAULT 0.0,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 19 (project_videos) failed: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_videos_project ON project_videos(project_id)",
        [],
    ).map_err(|e| format!("Migration 19a (videos project index) failed: {}", e))?;

    // Migration 20: video_extractions (Phase 7)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS video_extractions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            video_id TEXT NOT NULL,
            extraction_strategy TEXT NOT NULL DEFAULT 'interval',
            interval_secs REAL NOT NULL DEFAULT 1.0,
            status TEXT NOT NULL DEFAULT 'pending',
            extractor_version TEXT NOT NULL DEFAULT '',
            total_frames INTEGER NOT NULL DEFAULT 0,
            error_message TEXT,
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            completed_at TEXT,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (video_id) REFERENCES project_videos(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 20 (video_extractions) failed: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_extractions_video ON video_extractions(video_id)",
        [],
    ).map_err(|e| format!("Migration 20a (extractions video index) failed: {}", e))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_extractions_project ON video_extractions(project_id)",
        [],
    ).map_err(|e| format!("Migration 20b (extractions project index) failed: {}", e))?;

    // Migration 21: video_frames (Phase 7)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS video_frames (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            video_id TEXT NOT NULL,
            extraction_id TEXT NOT NULL,
            frame_index INTEGER NOT NULL,
            timestamp_ms INTEGER NOT NULL DEFAULT 0,
            timestamp_display TEXT NOT NULL DEFAULT '',
            image_path TEXT NOT NULL,
            content_hash TEXT NOT NULL DEFAULT '',
            width INTEGER NOT NULL DEFAULT 0,
            height INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (video_id) REFERENCES project_videos(id) ON DELETE CASCADE,
            FOREIGN KEY (extraction_id) REFERENCES video_extractions(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 21 (video_frames) failed: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_frames_video ON video_frames(video_id, frame_index)",
        [],
    ).map_err(|e| format!("Migration 21a (frames video index) failed: {}", e))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_frames_extraction ON video_frames(extraction_id)",
        [],
    ).map_err(|e| format!("Migration 21b (frames extraction index) failed: {}", e))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_frames_project ON video_frames(project_id)",
        [],
    ).map_err(|e| format!("Migration 21c (frames project index) failed: {}", e))?;
// Migration 22: add video_frame_id to project_images and frame_image_id to video_frames (Phase 7 bridge)
    conn.execute(
        "ALTER TABLE project_images ADD COLUMN video_frame_id TEXT NOT NULL DEFAULT ''",
        [],
    ).map_err(|e| format!("Migration 22a (project_images.video_frame_id) failed: {}", e)).or_else(|e| {
        if e.contains("duplicate column") { Ok(0) } else { Err(e) }
    })?;
    conn.execute(
        "ALTER TABLE video_frames ADD COLUMN frame_image_id TEXT NOT NULL DEFAULT ''",
        [],
    ).map_err(|e| format!("Migration 22b (video_frames.frame_image_id) failed: {}", e)).or_else(|e| {
        if e.contains("duplicate column") { Ok(0) } else { Err(e) }
    })?;
    // Migration 23: duplicate_relationships (Phase 8)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS duplicate_relationships (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            source_issue_id TEXT NOT NULL,
            duplicate_issue_id TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 0.0,
            match_signals TEXT NOT NULL DEFAULT '[]',
            confirmed INTEGER NOT NULL DEFAULT 0,
            confirmed_by TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (source_issue_id) REFERENCES issue_proposals(id) ON DELETE CASCADE,
            FOREIGN KEY (duplicate_issue_id) REFERENCES issue_proposals(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 23 (duplicate_relationships) failed: {}", e))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_dup_rels_project ON duplicate_relationships(project_id);",
        [],
    ).map_err(|e| format!("Migration 23a (dup_rels project index) failed: {}", e))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_dup_rels_source ON duplicate_relationships(source_issue_id);",
        [],
    ).map_err(|e| format!("Migration 23b (dup_rels source index) failed: {}", e))?;

    // Migration 24: known_issues (Phase 8)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS known_issues (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            title TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            category TEXT NOT NULL DEFAULT '',
            subcategory TEXT NOT NULL DEFAULT '',
            bug_type TEXT NOT NULL DEFAULT '',
            severity TEXT NOT NULL DEFAULT '',
            source_issue_id TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 24 (known_issues) failed: {}", e))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_known_issues_project ON known_issues(project_id);",
        [],
    ).map_err(|e| format!("Migration 24a (known_issues project index) failed: {}", e))?;

    // Migration 25: add lifecycle column to issue_proposals (Phase 8)
    conn.execute(
        "ALTER TABLE issue_proposals ADD COLUMN lifecycle TEXT NOT NULL DEFAULT 'proposed'",
        [],
    ).map_err(|e| format!("Migration 25 (issue_proposals.lifecycle) failed: {}", e)).or_else(|e| {
        if e.contains("duplicate column") { Ok(0) } else { Err(e) }
    })?;

    // Migration 26: builds (Phase 9)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS builds (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            version_label TEXT NOT NULL DEFAULT '',
            platform TEXT NOT NULL DEFAULT '',
            environment TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 26 (builds) failed: {}", e))?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_builds_project ON builds(project_id);", []).map_err(|e| format!("Migration 26a (builds idx) failed: {}", e))?;

    // Migration 27: add build_id to project_images (Phase 9)
    conn.execute("ALTER TABLE project_images ADD COLUMN build_id TEXT NOT NULL DEFAULT ''", []).map_err(|e| format!("Migration 27 (build_id on images) failed: {}", e)).or_else(|e| { if e.contains("duplicate column") { Ok(0) } else { Err(e) } })?;

    // Migration 28: add build_id to project_videos (Phase 9)
    conn.execute("ALTER TABLE project_videos ADD COLUMN build_id TEXT NOT NULL DEFAULT ''", []).map_err(|e| format!("Migration 28 (build_id on videos) failed: {}", e)).or_else(|e| { if e.contains("duplicate column") { Ok(0) } else { Err(e) } })?;

    // Migration 29: localization_records (Phase 10)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS localization_records (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            string_id TEXT NOT NULL DEFAULT '',
            source_text TEXT NOT NULL DEFAULT '',
            target_text TEXT NOT NULL DEFAULT '',
            locale TEXT NOT NULL DEFAULT '',
            context TEXT NOT NULL DEFAULT '',
            source_file TEXT NOT NULL DEFAULT '',
            source_row INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 29 (localization_records) failed: {}", e))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_loc_records_project ON localization_records(project_id);",
        [],
    ).map_err(|e| format!("Migration 29a (loc records idx) failed: {}", e))?;

    // Migration 30: string_mappings (Phase 10)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS string_mappings (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            image_id TEXT NOT NULL,
            ocr_text TEXT NOT NULL DEFAULT '',
            loc_record_id TEXT NOT NULL DEFAULT '',
            string_id TEXT NOT NULL DEFAULT '',
            mapping_method TEXT NOT NULL DEFAULT '',
            confidence REAL NOT NULL DEFAULT 0.0,
            confirmed INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (image_id) REFERENCES project_images(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 30 (string_mappings) failed: {}", e))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_str_mappings_project ON string_mappings(project_id);",
        [],
    ).map_err(|e| format!("Migration 30a (str mappings idx) failed: {}", e))?;

    // Migration 31: analysis_job_queue (Phase 11)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS analysis_job_queue (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            job_type TEXT NOT NULL DEFAULT 'image',
            source_id TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'queued',
            progress REAL NOT NULL DEFAULT 0.0,
            error_message TEXT,
            retry_count INTEGER NOT NULL DEFAULT 0,
            max_retries INTEGER NOT NULL DEFAULT 3,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            started_at TEXT,
            completed_at TEXT,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );",
        [],
    ).map_err(|e| format!("Migration 31 (analysis_job_queue) failed: {}", e))?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_jobs_project ON analysis_job_queue(project_id);",
        [],
    ).map_err(|e| format!("Migration 31a (jobs idx) failed: {}", e))?;

    // Migration 32: add job_id to image_analyses (Phase 11)
    conn.execute(
        "ALTER TABLE image_analyses ADD COLUMN job_id TEXT NOT NULL DEFAULT ''",
        [],
    ).map_err(|e| format!("Migration 32 (job_id on analyses) failed: {}", e)).or_else(|e| {
        if e.contains("duplicate column") { Ok(0) } else { Err(e) }
    })?;

    // Migration 33: tracker_configs (Phase 14)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tracker_configs (
            id TEXT PRIMARY KEY, project_id TEXT NOT NULL,
            connector_type TEXT NOT NULL DEFAULT 'mock', display_name TEXT NOT NULL DEFAULT '',
            base_url TEXT NOT NULL DEFAULT '', project_key TEXT NOT NULL DEFAULT '',
            field_mapping_json TEXT NOT NULL DEFAULT '{}', enabled INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );", [],
    ).map_err(|e| format!("Migration 33 (tracker_configs) failed: {}", e))?;

    // Migration 34: submission_records (Phase 14)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS submission_records (
            id TEXT PRIMARY KEY, qaivra_issue_id TEXT NOT NULL,
            tracker_config_id TEXT NOT NULL, external_id TEXT NOT NULL DEFAULT '',
            payload_fingerprint TEXT NOT NULL DEFAULT '', status TEXT NOT NULL DEFAULT 'submitted',
            submitted_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (qaivra_issue_id) REFERENCES issue_proposals(id) ON DELETE CASCADE,
            FOREIGN KEY (tracker_config_id) REFERENCES tracker_configs(id) ON DELETE CASCADE
        );", [],
    ).map_err(|e| format!("Migration 34 (submission_records) failed: {}", e))?;

    // Migration 35: audit_events (Phase 15)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS audit_events (
            id TEXT PRIMARY KEY, project_id TEXT NOT NULL,
            event_type TEXT NOT NULL DEFAULT '', entity_type TEXT NOT NULL DEFAULT '',
            entity_id TEXT NOT NULL DEFAULT '', safe_metadata_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );", [],
    ).map_err(|e| format!("Migration 35 (audit_events) failed: {}", e))?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_project ON audit_events(project_id);", [])
        .map_err(|e| format!("Migration 35a (audit idx) failed: {}", e))?;

    // Migration 36: extend ai_provider_config for multi-provider support (Alpha Polish)
    for (col, col_def) in [
        ("temperature", "REAL NOT NULL DEFAULT 0.0"),
        ("max_tokens", "INTEGER NOT NULL DEFAULT 4096"),
        ("timeout_secs", "INTEGER NOT NULL DEFAULT 120"),
        ("api_key_ref", "TEXT NOT NULL DEFAULT ''"),
        ("provider_display_name", "TEXT NOT NULL DEFAULT ''"),
    ] {
        conn.execute(
            &format!("ALTER TABLE ai_provider_config ADD COLUMN {} {}", col, col_def),
            [],
        ).map_err(|e| format!("Migration 36 (ai_provider_config.{}) failed: {}", col, e))
         .or_else(|e| { if e.contains("duplicate column") { Ok(0) } else { Err(e) } })?;
    }

    // Record applied migrations
    conn.execute(
        "INSERT OR IGNORE INTO _migrations (id, name) VALUES
            (1, 'create_projects'),
            (2, 'create_knowledge_documents'),
            (3, 'create_migrations_tracking'),
            (4, 'create_severity_rules'),
            (5, 'create_severity_imports'),
            (6, 'create_glossary_entries'),
            (7, 'create_reference_chunks'),
            (8, 'create_glossary_conflicts'),
            (9, 'create_glossary_imports'),
            (10, 'create_precedent_reports'),
            (11, 'create_ai_provider_config'),
            (12, 'create_analysis_jobs'),
            (13, 'create_ocr_regions'),
            (14, 'create_project_images'),
            (15, 'create_image_analyses'),
            (16, 'create_issue_proposals'),
            (17, 'add_image_ocr_text'),
            (18, 'create_evidence_annotations'),
            (19, 'create_project_videos'),
            (20, 'create_video_extractions'),
            (21, 'create_video_frames'),
            (22, 'add_video_frame_bridge'),
            (23, 'create_duplicate_relationships'),
            (24, 'create_known_issues'),
            (25, 'add_issue_lifecycle_column'),
            (26, 'create_builds'),
            (27, 'add_build_id_to_images'),
            (28, 'add_build_id_to_videos'),
            (29, 'create_localization_records'),
            (30, 'create_string_mappings'),
            (31, 'create_analysis_job_queue'),
            (32, 'add_job_id_to_analyses'),
            (33, 'create_tracker_configs'),
            (34, 'create_submission_records'),
            (35, 'create_audit_events'),
            (36, 'extend_ai_provider_config');",
        [],
    ).map_err(|e| format!("Failed to record migrations: {}", e))?;

    Ok(())
}

pub fn create_project(
    conn: &Connection,
    id: &str,
    name: &str,
    target_language: &str,
    genre: &str,
    platform: &str,
) -> Result<Project, String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO projects (id, name, target_language, genre, platform, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, name, target_language, genre, platform, now, now],
    )
    .map_err(|e| format!("Failed to create project: {}", e))?;

    Ok(Project {
        id: id.to_string(),
        name: name.to_string(),
        target_language: target_language.to_string(),
        genre: genre.to_string(),
        platform: platform.to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_projects(conn: &Connection) -> Result<Vec<Project>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, target_language, genre, platform, created_at, updated_at FROM projects ORDER BY created_at DESC")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                target_language: row.get(2)?,
                genre: row.get(3)?,
                platform: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query projects: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read project row: {}", e))?;

    Ok(projects)
}

pub fn get_project(conn: &Connection, id: &str) -> Result<Project, String> {
    conn.query_row(
        "SELECT id, name, target_language, genre, platform, created_at, updated_at FROM projects WHERE id = ?1",
        params![id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                target_language: row.get(2)?,
                genre: row.get(3)?,
                platform: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| format!("Project not found: {}", e))
}

pub fn delete_project(conn: &Connection, id: &str) -> Result<(), String> {
    let rows = conn
        .execute("DELETE FROM projects WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete project: {}", e))?;

    if rows == 0 {
        return Err("Project not found".to_string());
    }
    Ok(())
}

pub fn rename_project(conn: &Connection, id: &str, new_name: &str) -> Result<Project, String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_name, now, id],
    )
    .map_err(|e| format!("Failed to rename project: {}", e))?;

    get_project(conn, id)
}

// ---- Phase 2: Severity Rules ----

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeverityRule {
    pub id: String,
    pub project_id: String,
    pub source_sheet: String,
    pub source_row: i32,
    pub main_category_raw: String,
    pub subcategory_raw: String,
    pub canonical_bug_type: String,
    pub severity_exact: String,
    pub severity_code: String,
    pub severity_name: String,
    pub description_raw: String,
    pub source_filename: String,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeverityImportResult {
    pub import_id: String,
    pub rules_count: usize,
    pub warnings: Vec<String>,
    pub source_filename: String,
    pub source_sheet: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedRule {
    pub source_row: i32,
    pub main_category_raw: String,
    pub subcategory_raw: String,
    pub canonical_bug_type: String,
    pub severity_exact: String,
    pub severity_code: String,
    pub severity_name: String,
    pub description_raw: String,
    pub source_sheet: String,
    pub source_filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParseResult {
    pub success: bool,
    pub error: Option<String>,
    pub rules: Vec<ParsedRule>,
    pub warnings: Vec<String>,
    pub metadata: Option<ParseMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParseMetadata {
    pub filename: String,
    pub sheet_name: String,
    pub total_rules: usize,
}

use rusqlite::OptionalExtension;

/// Import parsed severity rules for a project. Deactivates previous rules first.
pub fn import_severity_rules(
    conn: &Connection,
    project_id: &str,
    parse_result: &ParseResult,
) -> Result<SeverityImportResult, String> {
    if !parse_result.success {
        return Err(parse_result.error.clone().unwrap_or_else(|| "Parse failed".to_string()));
    }

    let import_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    // Deactivate all existing rules for this project
    if let Err(e) = conn.execute(
        "UPDATE severity_rules SET active = 0 WHERE project_id = ?1",
        params![project_id],
    ) {
        conn.execute_batch("ROLLBACK;").ok();
        return Err(format!("Failed to deactivate old rules: {}", e));
    }

    // Insert new rules
    for rule in &parse_result.rules {
        let rule_id = uuid::Uuid::new_v4().to_string();
        if let Err(e) = conn.execute(
            "INSERT INTO severity_rules (
                id, project_id, source_sheet, source_row,
                main_category_raw, subcategory_raw, canonical_bug_type,
                severity_exact, severity_code, severity_name,
                description_raw, source_filename, active, created_at
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,1,?13)",
            params![
                rule_id, project_id, rule.source_sheet, rule.source_row,
                rule.main_category_raw, rule.subcategory_raw, rule.canonical_bug_type,
                rule.severity_exact, rule.severity_code, rule.severity_name,
                rule.description_raw, rule.source_filename, now,
            ],
        ) {
            conn.execute_batch("ROLLBACK;").ok();
            return Err(format!("Failed to insert rule row {}: {}", rule.source_row, e));
        }
    }

    // Record the import
    let meta = parse_result.metadata.as_ref();
    let source_filename = meta.map(|m| m.filename.clone()).unwrap_or_default();
    let source_sheet = meta.map(|m| m.sheet_name.clone()).unwrap_or_default();

    if let Err(e) = conn.execute(
        "INSERT INTO severity_imports (
            id, project_id, source_filename, source_sheet, rule_count, warning_count, status, created_at
        ) VALUES (?1,?2,?3,?4,?5,?6,'completed',?7)",
        params![
            import_id, project_id, source_filename, source_sheet,
            parse_result.rules.len(), parse_result.warnings.len(), now,
        ],
    ) {
        conn.execute_batch("ROLLBACK;").ok();
        return Err(format!("Failed to record import: {}", e));
    }

    conn.execute_batch("COMMIT;")
        .map_err(|e| format!("Failed to commit: {}", e))?;

    Ok(SeverityImportResult {
        import_id,
        rules_count: parse_result.rules.len(),
        warnings: parse_result.warnings.clone(),
        source_filename,
        source_sheet,
        status: "completed".to_string(),
    })
}

/// List active severity rules for a project.
pub fn list_severity_rules(conn: &Connection, project_id: &str) -> Result<Vec<SeverityRule>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, source_sheet, source_row,
                    main_category_raw, subcategory_raw, canonical_bug_type,
                    severity_exact, severity_code, severity_name,
                    description_raw, source_filename, active, created_at
             FROM severity_rules WHERE project_id = ?1 AND active = 1 ORDER BY source_row",
        )
        .map_err(|e| format!("Failed to prepare: {}", e))?;

    let rules = stmt
        .query_map(params![project_id], |row| {
            Ok(SeverityRule {
                id: row.get(0)?, project_id: row.get(1)?,
                source_sheet: row.get(2)?, source_row: row.get(3)?,
                main_category_raw: row.get(4)?, subcategory_raw: row.get(5)?,
                canonical_bug_type: row.get(6)?, severity_exact: row.get(7)?,
                severity_code: row.get(8)?, severity_name: row.get(9)?,
                description_raw: row.get(10)?, source_filename: row.get(11)?,
                active: row.get::<_, i32>(12)? != 0, created_at: row.get(13)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Collect error: {}", e))?;
    Ok(rules)
}

/// Get the most recent severity import for a project.
pub fn get_severity_import_status(
    conn: &Connection, project_id: &str,
) -> Result<Option<(String, String, i32, i32, String)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT source_filename, source_sheet, rule_count, warning_count, status
             FROM severity_imports WHERE project_id = ?1 ORDER BY created_at DESC LIMIT 1",
        )
        .map_err(|e| format!("Failed to prepare: {}", e))?;

    let result = stmt
        .query_row(params![project_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })
        .optional()
        .map_err(|e| format!("Query error: {}", e))?;
    Ok(result)
}

// ---- Phase 3: Glossary & Reference Knowledge ----

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlossaryEntry {
    pub id: String,
    pub project_id: String,
    pub source_term: String,
    pub target_term: String,
    pub source_term_normalized: String,
    pub status: String,
    pub do_not_translate: bool,
    pub banned: bool,
    pub notes: String,
    pub source_filename: String,
    pub source_sheet: String,
    pub source_row: i32,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedGlossaryEntry {
    pub source_term: String,
    pub target_term: String,
    pub status: String,
    pub do_not_translate: bool,
    pub banned: bool,
    pub notes: String,
    pub source_row: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlossaryImportResult {
    pub import_id: String,
    pub entry_count: usize,
    pub duplicate_count: usize,
    pub conflict_count: usize,
    pub warnings: Vec<String>,
    pub source_filename: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReferenceChunk {
    pub id: String,
    pub project_id: String,
    pub source_filename: String,
    pub source_type: String,
    pub source_page: i32,
    pub source_sheet: String,
    pub source_row_start: i32,
    pub source_row_end: i32,
    pub heading: String,
    pub content: String,
    pub chunk_order: i32,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedReferenceChunk {
    pub source_page: i32,
    pub source_sheet: String,
    pub source_row_start: i32,
    pub source_row_end: i32,
    pub heading: String,
    pub content: String,
    pub chunk_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceImportResult {
    pub import_id: String,
    pub chunk_count: usize,
    pub warnings: Vec<String>,
    pub source_filename: String,
    pub source_type: String,
    pub status: String,
}

// ---- Phase 3: Precedent Reports ----

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrecedentReport {
    pub id: String,
    pub project_id: String,
    pub source_filename: String,
    pub source_type: String,
    pub sheet_name: String,
    pub page_number: i32,
    pub row_start: i32,
    pub row_end: i32,
    pub heading: String,
    pub content: String,
    pub issue_type: String,
    pub severity_exact: String,
    pub terminology_used: String,
    pub chunk_order: i32,
    pub classification: String,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedPrecedentRow {
    pub sheet_name: String,
    pub page_number: i32,
    pub row_start: i32,
    pub row_end: i32,
    pub heading: String,
    pub content: String,
    pub issue_type: String,
    pub severity_exact: String,
    pub terminology_used: String,
    pub chunk_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrecedentImportResult {
    pub import_id: String,
    pub chunk_count: usize,
    pub warnings: Vec<String>,
    pub source_filename: String,
    pub source_type: String,
    pub classification: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlossaryConflictInfo {
    pub id: String,
    pub project_id: String,
    pub source_term_normalized: String,
    pub resolution: String,
    pub chosen_entry_id: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

/// Normalize a term for comparison: lowercase, trim, collapse whitespace.
pub fn normalize_for_comparison(term: &str) -> String {
    term.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Import glossary entries for a project with safe replacement.
pub fn import_glossary_entries(
    conn: &Connection,
    project_id: &str,
    source_filename: &str,
    source_type: &str,
    entries: &[ParsedGlossaryEntry],
    warnings: &[String],
) -> Result<GlossaryImportResult, String> {
    let import_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    if let Err(e) = conn.execute(
        "UPDATE glossary_entries SET active = 0 WHERE project_id = ?1",
        params![project_id],
    ) {
        conn.execute_batch("ROLLBACK;").ok();
        return Err(format!("Failed to deactivate old entries: {}", e));
    }

    if let Err(e) = conn.execute(
        "DELETE FROM glossary_conflicts WHERE project_id = ?1",
        params![project_id],
    ) {
        conn.execute_batch("ROLLBACK;").ok();
        return Err(format!("Failed to clear old conflicts: {}", e));
    }

    let mut duplicate_count = 0usize;
    let mut seen_normalized: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for entry in entries {
        let entry_id = uuid::Uuid::new_v4().to_string();
        let normalized = normalize_for_comparison(&entry.source_term);

        if seen_normalized.contains_key(&normalized) {
            duplicate_count += 1;
            // Persist the conflict for review
            if let Err(_e) = record_glossary_conflict(conn, project_id, &normalized) {
                // Non-fatal: conflict recording failure doesn't block import
            }
        }
        seen_normalized.insert(normalized.clone(), entry_id.clone());

        if let Err(e) = conn.execute(
            "INSERT INTO glossary_entries (
                id, project_id, source_term, target_term, source_term_normalized,
                status, do_not_translate, banned, notes,
                source_filename, source_sheet, source_row, active, created_at
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,1,?13)",
            params![
                entry_id, project_id, entry.source_term, entry.target_term, normalized,
                entry.status, entry.do_not_translate as i32, entry.banned as i32, entry.notes,
                source_filename, "", entry.source_row, now,
            ],
        ) {
            conn.execute_batch("ROLLBACK;").ok();
            return Err(format!("Failed to insert glossary entry: {}", e));
        }
    }

    if let Err(e) = conn.execute(
        "INSERT INTO glossary_imports (
            id, project_id, source_filename, source_type, content_hash,
            entry_count, duplicate_count, conflict_count, warning_count,
            status, created_at
        ) VALUES (?1,?2,?3,?4,'',?5,?6,0,?7,'completed',?8)",
        params![
            import_id, project_id, source_filename, source_type,
            entries.len(), duplicate_count, warnings.len(), now,
        ],
    ) {
        conn.execute_batch("ROLLBACK;").ok();
        return Err(format!("Failed to record import: {}", e));
    }

    conn.execute_batch("COMMIT;")
        .map_err(|e| format!("Failed to commit: {}", e))?;

    Ok(GlossaryImportResult {
        import_id,
        entry_count: entries.len(),
        duplicate_count,
        conflict_count: duplicate_count,
        warnings: warnings.to_vec(),
        source_filename: source_filename.to_string(),
        status: "completed".to_string(),
    })
}

/// List active glossary entries for a project.
pub fn list_glossary_entries(conn: &Connection, project_id: &str) -> Result<Vec<GlossaryEntry>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, source_term, target_term, source_term_normalized,
                status, do_not_translate, banned, notes,
                source_filename, source_sheet, source_row, active, created_at
         FROM glossary_entries WHERE project_id = ?1 AND active = 1 ORDER BY source_row",
    ).map_err(|e| format!("Failed to prepare: {}", e))?;

    let entries = stmt
        .query_map(params![project_id], |row| {
            Ok(GlossaryEntry {
                id: row.get(0)?, project_id: row.get(1)?,
                source_term: row.get(2)?, target_term: row.get(3)?,
                source_term_normalized: row.get(4)?, status: row.get(5)?,
                do_not_translate: row.get::<_, i32>(6)? != 0,
                banned: row.get::<_, i32>(7)? != 0,
                notes: row.get(8)?, source_filename: row.get(9)?,
                source_sheet: row.get(10)?, source_row: row.get(11)?,
                active: row.get::<_, i32>(12)? != 0, created_at: row.get(13)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Collect error: {}", e))?;
    Ok(entries)
}

/// Import reference chunks for a project.
pub fn import_reference_chunks(
    conn: &Connection,
    project_id: &str,
    source_filename: &str,
    source_type: &str,
    chunks: &[ParsedReferenceChunk],
    warnings: &[String],
) -> Result<ReferenceImportResult, String> {
    let import_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    if let Err(e) = conn.execute(
        "UPDATE reference_chunks SET active = 0 WHERE project_id = ?1 AND source_filename = ?2",
        params![project_id, source_filename],
    ) {
        conn.execute_batch("ROLLBACK;").ok();
        return Err(format!("Failed to deactivate old chunks: {}", e));
    }

    for chunk in chunks {
        let chunk_id = uuid::Uuid::new_v4().to_string();
        if let Err(e) = conn.execute(
            "INSERT INTO reference_chunks (
                id, project_id, source_filename, source_type,
                source_page, source_sheet, source_row_start, source_row_end,
                heading, content, chunk_order, active, created_at
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,1,?12)",
            params![
                chunk_id, project_id, source_filename, source_type,
                chunk.source_page, chunk.source_sheet, chunk.source_row_start, chunk.source_row_end,
                chunk.heading, chunk.content, chunk.chunk_order, now,
            ],
        ) {
            conn.execute_batch("ROLLBACK;").ok();
            return Err(format!("Failed to insert chunk: {}", e));
        }
    }

    conn.execute_batch("COMMIT;")
        .map_err(|e| format!("Failed to commit: {}", e))?;

    Ok(ReferenceImportResult {
        import_id,
        chunk_count: chunks.len(),
        warnings: warnings.to_vec(),
        source_filename: source_filename.to_string(),
        source_type: source_type.to_string(),
        status: "completed".to_string(),
    })
}

/// List active reference chunks for a project.
pub fn list_reference_chunks(conn: &Connection, project_id: &str) -> Result<Vec<ReferenceChunk>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, source_filename, source_type,
                source_page, source_sheet, source_row_start, source_row_end,
                heading, content, chunk_order, active, created_at
         FROM reference_chunks WHERE project_id = ?1 AND active = 1 ORDER BY chunk_order",
    ).map_err(|e| format!("Failed to prepare: {}", e))?;

    let chunks = stmt
        .query_map(params![project_id], |row| {
            Ok(ReferenceChunk {
                id: row.get(0)?, project_id: row.get(1)?,
                source_filename: row.get(2)?, source_type: row.get(3)?,
                source_page: row.get(4)?, source_sheet: row.get(5)?,
                source_row_start: row.get(6)?, source_row_end: row.get(7)?,
                heading: row.get(8)?, content: row.get(9)?,
                chunk_order: row.get(10)?, active: row.get::<_, i32>(11)? != 0,
                created_at: row.get(12)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Collect error: {}", e))?;
    Ok(chunks)
}

/// Search glossary entries by normalized term match.
pub fn search_glossary(conn: &Connection, project_id: &str, query: &str) -> Result<Vec<GlossaryEntry>, String> {
    let normalized_query = normalize_for_comparison(query);
    let pattern = format!("%{}%", normalized_query);
    let like_pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, project_id, source_term, target_term, source_term_normalized,
                status, do_not_translate, banned, notes,
                source_filename, source_sheet, source_row, active, created_at
         FROM glossary_entries
         WHERE project_id = ?1 AND active = 1
           AND (source_term_normalized LIKE ?2 OR source_term LIKE ?3 OR target_term LIKE ?3)
         ORDER BY source_row LIMIT 50",
    ).map_err(|e| format!("Failed to prepare: {}", e))?;

    let entries = stmt
        .query_map(params![project_id, pattern, like_pattern], |row| {
            Ok(GlossaryEntry {
                id: row.get(0)?, project_id: row.get(1)?,
                source_term: row.get(2)?, target_term: row.get(3)?,
                source_term_normalized: row.get(4)?, status: row.get(5)?,
                do_not_translate: row.get::<_, i32>(6)? != 0,
                banned: row.get::<_, i32>(7)? != 0,
                notes: row.get(8)?, source_filename: row.get(9)?,
                source_sheet: row.get(10)?, source_row: row.get(11)?,
                active: row.get::<_, i32>(12)? != 0, created_at: row.get(13)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Collect error: {}", e))?;
    Ok(entries)
}

/// Search reference chunks by keyword match.
pub fn search_reference_chunks(conn: &Connection, project_id: &str, query: &str) -> Result<Vec<ReferenceChunk>, String> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, project_id, source_filename, source_type,
                source_page, source_sheet, source_row_start, source_row_end,
                heading, content, chunk_order, active, created_at
         FROM reference_chunks
         WHERE project_id = ?1 AND active = 1 AND content LIKE ?2
         ORDER BY chunk_order LIMIT 20",
    ).map_err(|e| format!("Failed to prepare: {}", e))?;

    let chunks = stmt
        .query_map(params![project_id, pattern], |row| {
            Ok(ReferenceChunk {
                id: row.get(0)?, project_id: row.get(1)?,
                source_filename: row.get(2)?, source_type: row.get(3)?,
                source_page: row.get(4)?, source_sheet: row.get(5)?,
                source_row_start: row.get(6)?, source_row_end: row.get(7)?,
                heading: row.get(8)?, content: row.get(9)?,
                chunk_order: row.get(10)?, active: row.get::<_, i32>(11)? != 0,
                created_at: row.get(12)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Collect error: {}", e))?;
    Ok(chunks)
}

/// Import precedent report rows for a project. Classification must be explicit.
pub fn import_precedent_report(
    conn: &Connection, project_id: &str, source_filename: &str,
    source_type: &str, classification: &str, rows: &[ParsedPrecedentRow],
    warnings: &[String],
) -> Result<PrecedentImportResult, String> {
    if classification != "PRECEDENT" && classification != "APPROVED" {
        return Err(format!("Classification must be explicit, got: '{}'", classification));
    }
    let import_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute_batch("BEGIN TRANSACTION;").map_err(|e| format!("Begin failed: {}", e))?;
    if let Err(e) = conn.execute(
        "UPDATE precedent_reports SET active = 0 WHERE project_id = ?1 AND source_filename = ?2",
        params![project_id, source_filename],
    ) { conn.execute_batch("ROLLBACK;").ok(); return Err(format!("Deactivate failed: {}", e)); }
    for row in rows {
        let rid = uuid::Uuid::new_v4().to_string();
        if let Err(e) = conn.execute(
            "INSERT INTO precedent_reports (id, project_id, source_filename, source_type,
                sheet_name, page_number, row_start, row_end, heading, content, issue_type,
                severity_exact, terminology_used, chunk_order, classification, active, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,1,?16)",
            params![rid, project_id, source_filename, source_type,
                row.sheet_name, row.page_number, row.row_start, row.row_end,
                row.heading, row.content, row.issue_type, row.severity_exact,
                row.terminology_used, row.chunk_order, classification, now],
        ) { conn.execute_batch("ROLLBACK;").ok(); return Err(format!("Insert failed: {}", e)); }
    }
    conn.execute_batch("COMMIT;").map_err(|e| format!("Commit failed: {}", e))?;
    Ok(PrecedentImportResult {
        import_id, chunk_count: rows.len(), warnings: warnings.to_vec(),
        source_filename: source_filename.to_string(), source_type: source_type.to_string(),
        classification: classification.to_string(), status: "completed".to_string(),
    })
}

/// List active precedent reports for a project.
pub fn list_precedent_reports(conn: &Connection, project_id: &str) -> Result<Vec<PrecedentReport>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, source_filename, source_type, sheet_name, page_number,
                row_start, row_end, heading, content, issue_type, severity_exact,
                terminology_used, chunk_order, classification, active, created_at
         FROM precedent_reports WHERE project_id = ?1 AND active = 1 ORDER BY chunk_order",
    ).map_err(|e| format!("Failed to prepare: {}", e))?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(PrecedentReport {
            id: row.get(0)?, project_id: row.get(1)?, source_filename: row.get(2)?,
            source_type: row.get(3)?, sheet_name: row.get(4)?, page_number: row.get(5)?,
            row_start: row.get(6)?, row_end: row.get(7)?, heading: row.get(8)?,
            content: row.get(9)?, issue_type: row.get(10)?, severity_exact: row.get(11)?,
            terminology_used: row.get(12)?, chunk_order: row.get(13)?, classification: row.get(14)?,
            active: row.get::<_, i32>(15)? != 0, created_at: row.get(16)?,
        })
    }).map_err(|e| format!("Query error: {}", e))?
      .collect::<Result<Vec<_>, _>>().map_err(|e| format!("Collect error: {}", e))?;
    Ok(rows)
}

/// Search precedent reports by keyword match.
pub fn search_precedent_reports(conn: &Connection, project_id: &str, query: &str) -> Result<Vec<PrecedentReport>, String> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, project_id, source_filename, source_type, sheet_name, page_number,
                row_start, row_end, heading, content, issue_type, severity_exact,
                terminology_used, chunk_order, classification, active, created_at
         FROM precedent_reports
         WHERE project_id = ?1 AND active = 1 AND classification = 'PRECEDENT'
           AND (content LIKE ?2 OR issue_type LIKE ?2 OR severity_exact LIKE ?2 OR terminology_used LIKE ?2)
         ORDER BY chunk_order LIMIT 20",
    ).map_err(|e| format!("Failed to prepare: {}", e))?;
    let rows = stmt.query_map(params![project_id, pattern], |row| {
        Ok(PrecedentReport {
            id: row.get(0)?, project_id: row.get(1)?, source_filename: row.get(2)?,
            source_type: row.get(3)?, sheet_name: row.get(4)?, page_number: row.get(5)?,
            row_start: row.get(6)?, row_end: row.get(7)?, heading: row.get(8)?,
            content: row.get(9)?, issue_type: row.get(10)?, severity_exact: row.get(11)?,
            terminology_used: row.get(12)?, chunk_order: row.get(13)?, classification: row.get(14)?,
            active: row.get::<_, i32>(15)? != 0, created_at: row.get(16)?,
        })
    }).map_err(|e| format!("Query error: {}", e))?
      .collect::<Result<Vec<_>, _>>().map_err(|e| format!("Collect error: {}", e))?;
    Ok(rows)
}

/// Get glossary conflicts for a project.
pub fn get_glossary_conflicts(conn: &Connection, project_id: &str) -> Result<Vec<GlossaryConflictInfo>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, source_term_normalized, resolution,
                chosen_entry_id, created_at, resolved_at
         FROM glossary_conflicts WHERE project_id = ?1 ORDER BY created_at",
    ).map_err(|e| format!("Failed to prepare: {}", e))?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(GlossaryConflictInfo {
            id: row.get(0)?, project_id: row.get(1)?, source_term_normalized: row.get(2)?,
            resolution: row.get(3)?, chosen_entry_id: row.get(4)?,
            created_at: row.get(5)?, resolved_at: row.get(6)?,
        })
    }).map_err(|e| format!("Query error: {}", e))?
      .collect::<Result<Vec<_>, _>>().map_err(|e| format!("Collect error: {}", e))?;
    Ok(rows)
}

/// Count unresolved glossary conflicts for a project.
pub fn count_unresolved_conflicts(conn: &Connection, project_id: &str) -> Result<usize, String> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM glossary_conflicts WHERE project_id = ?1 AND resolution = 'unresolved'",
        params![project_id], |row| row.get(0),
    ).map_err(|e| format!("Query error: {}", e))?;
    Ok(count as usize)
}

/// Resolve a glossary conflict.
pub fn resolve_glossary_conflict(conn: &Connection, conflict_id: &str, chosen_entry_id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn.execute(
        "UPDATE glossary_conflicts SET resolution = 'resolved', chosen_entry_id = ?1, resolved_at = ?2 WHERE id = ?3",
        params![chosen_entry_id, now, conflict_id],
    ).map_err(|e| format!("Failed to resolve conflict: {}", e))?;
    if rows == 0 { return Err("Conflict not found".to_string()); }
    Ok(())
}

/// Record a glossary conflict for a project.
fn record_glossary_conflict(conn: &Connection, project_id: &str, normalized_term: &str) -> Result<(), String> {
    let cid = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO glossary_conflicts (id, project_id, source_term_normalized, resolution)
         VALUES (?1, ?2, ?3, 'unresolved')",
        params![cid, project_id, normalized_term],
    ).map_err(|e| format!("Failed to record conflict: {}", e))?;
    Ok(())
}

// ---- Phase 4: Local AI and OCR Foundation ----

/// AI provider configuration for a project.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiProviderConfig {
    pub id: String,
    pub project_id: String,
    pub provider: String,
    pub base_url: String,
    pub selected_model: String,
    pub model_capabilities_json: String,
    pub health_status: String,
    pub last_health_check: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub temperature: f64,
    pub max_tokens: i64,
    pub timeout_secs: i64,
    pub api_key_ref: String,
    pub provider_display_name: String,
}

/// Result of an Ollama health check.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaHealthResult {
    pub healthy: bool,
    pub version: String,
    pub base_url: String,
    pub error: Option<String>,
}

/// An installed Ollama model with capability metadata.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaModel {
    pub name: String,
    pub size_bytes: u64,
    pub parameter_size: String,
    pub family: String,
    pub capabilities: ModelCapabilities,
}

/// Capability flags for a model.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModelCapabilities {
    pub vision: bool,
    pub text_generation: bool,
    pub structured_output: bool,
}

/// Bounding box for an OCR text region (normalized 0..1 coordinates).
/// Used by Phase 5+ image analysis pipeline.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// OCR sidecar result: text regions extracted from an image.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrResult {
    pub success: bool,
    pub regions: Vec<OcrTextRegion>,
    pub warnings: Vec<String>,
    pub error: Option<String>,
}

/// A single OCR text region from the sidecar output.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrTextRegion {
    pub text: String,
    pub confidence: f64,
    pub bbox_x: f64,
    pub bbox_y: f64,
    pub bbox_width: f64,
    pub bbox_height: f64,
}

/// Validation result for structured AI output.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Candidate issue observation from AI analysis (validated input).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CandidateIssueInput {
    pub observed_text_exact: String,
    pub defect_summary: String,
    pub candidate_bug_type: String,
    pub defect_condition: String,
    pub suggested_text: String,
    pub confidence: f64,
    pub review_state_hint: String,
    pub affected_region_ids: Vec<String>,
    pub context_evidence: Vec<String>,
    pub glossary_dependencies: Vec<String>,
}

/// Save or update AI provider config for a project.
pub fn save_ai_provider_config(
    conn: &Connection,
    project_id: &str,
    provider: &str,
    base_url: &str,
    selected_model: &str,
    capabilities_json: &str,
    health_status: &str,
    temperature: f64,
    max_tokens: i64,
    timeout_secs: i64,
    api_key_ref: &str,
    provider_display_name: &str,
) -> Result<AiProviderConfig, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let existing: Option<String> = conn.query_row(
        "SELECT id FROM ai_provider_config WHERE project_id = ?1",
        params![project_id],
        |row| row.get(0),
    ).ok();

    if let Some(eid) = existing {
        conn.execute(
            "UPDATE ai_provider_config SET provider = ?2, base_url = ?3, selected_model = ?4,
             model_capabilities_json = ?5, health_status = ?6, updated_at = ?7, temperature = ?8, max_tokens = ?9, timeout_secs = ?10, api_key_ref = ?11, provider_display_name = ?12 WHERE id = ?1",
            params![eid, provider, base_url, selected_model, capabilities_json, health_status, now, temperature, max_tokens, timeout_secs, api_key_ref, provider_display_name],
        ).map_err(|e| format!("Failed to update AI config: {}", e))?;
        get_ai_provider_config(conn, project_id)
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO ai_provider_config (id, project_id, provider, base_url, selected_model,
             model_capabilities_json, health_status, temperature, max_tokens, timeout_secs, api_key_ref, provider_display_name, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?9, ?10, ?11, ?12, ?13, ?8, ?8)",
            params![id, project_id, provider, base_url, selected_model, capabilities_json, health_status, now, temperature, max_tokens, timeout_secs, api_key_ref, provider_display_name],
        ).map_err(|e| format!("Failed to save AI config: {}", e))?;
        get_ai_provider_config(conn, project_id)
    }
}

/// Get AI provider config for a project.
pub fn get_ai_provider_config(conn: &Connection, project_id: &str) -> Result<AiProviderConfig, String> {
    conn.query_row(
        "SELECT id, project_id, provider, base_url, selected_model,
                model_capabilities_json, health_status, last_health_check, created_at, updated_at,
                temperature, max_tokens, timeout_secs, api_key_ref, provider_display_name
         FROM ai_provider_config WHERE project_id = ?1",
        params![project_id],
        |row| Ok(AiProviderConfig {
            id: row.get(0)?,
            project_id: row.get(1)?,
            provider: row.get(2)?,
            base_url: row.get(3)?,
            selected_model: row.get(4)?,
            model_capabilities_json: row.get(5)?,
            health_status: row.get(6)?,
            last_health_check: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            temperature: row.get(10)?,
            max_tokens: row.get(11)?,
            timeout_secs: row.get(12)?,
            api_key_ref: row.get(13)?,
            provider_display_name: row.get(14)?,
        }),
    ).map_err(|e| format!("No AI provider config found: {}", e))
}

/// Update health check timestamp and status.
pub fn update_ai_health(conn: &Connection, project_id: &str, health_status: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE ai_provider_config SET health_status = ?2, last_health_check = ?3, updated_at = ?3 WHERE project_id = ?1",
        params![project_id, health_status, now],
    ).map_err(|e| format!("Failed to update health: {}", e))?;
    Ok(())
}

/// Simple hash for region deduplication (not cryptographic).
/// Used by Phase 5+ OCR storage pipeline.
#[allow(dead_code)]
fn simple_hash(text: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

/// Store OCR regions from a completed OCR job.
/// Called by Phase 5+ image analysis pipeline.
#[allow(dead_code)]
pub fn store_ocr_regions(
    conn: &Connection,
    project_id: &str,
    image_id: &str,
    regions: &[OcrTextRegion],
) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();
    for r in regions {
        let id = uuid::Uuid::new_v4().to_string();
        let region_hash = format!("{:x}", simple_hash(&r.text));
        let text_normalized = r.text.trim().to_lowercase();
        conn.execute(
            "INSERT INTO ocr_regions (id, project_id, image_id, text_raw, text_normalized,
             bbox_x, bbox_y, bbox_width, bbox_height, ocr_confidence, region_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![id, project_id, image_id, r.text, text_normalized, r.bbox_x, r.bbox_y,
                    r.bbox_width, r.bbox_height, r.confidence, region_hash],
        ).map_err(|e| format!("Failed to store OCR region: {}", e))?;
        ids.push(id);
    }
    Ok(ids)
}

/// A persisted OCR geometry row (normalized coords) for evidence grounding.
#[derive(Debug, Serialize, Clone)]
pub struct OcrRegionRow {
    pub id: String,
    pub text: String,
    pub text_normalized: String,
    pub bbox_x: f64,
    pub bbox_y: f64,
    pub bbox_width: f64,
    pub bbox_height: f64,
    pub ocr_confidence: f64,
}

/// Fetch OCR geometry regions for a specific project image (project-scoped).
pub fn get_ocr_regions(
    conn: &Connection,
    project_id: &str,
    image_id: &str,
) -> Result<Vec<OcrRegionRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, text_raw, text_normalized, bbox_x, bbox_y, bbox_width, bbox_height, ocr_confidence
             FROM ocr_regions WHERE project_id = ?1 AND image_id = ?2 ORDER BY created_at DESC",
        )
        .map_err(|e| format!("Failed to prepare OCR regions query: {}", e))?;
    let rows = stmt
        .query_map(params![project_id, image_id], |row| {
            Ok(OcrRegionRow {
                id: row.get(0)?,
                text: row.get(1)?,
                text_normalized: row.get(2)?,
                bbox_x: row.get(3)?,
                bbox_y: row.get(4)?,
                bbox_width: row.get(5)?,
                bbox_height: row.get(6)?,
                ocr_confidence: row.get(7)?,
            })
        })
        .map_err(|e| format!("OCR regions query error: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("OCR regions collect error: {}", e))?;
    Ok(rows)
}

/// Validate a candidate issue observation against canonical rules.
/// Invalid candidates cannot become confirmed issues â€” they must go to review.
pub fn validate_candidate_issue(
    candidate: &CandidateIssueInput,
    active_bug_types: &[String],
) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if candidate.observed_text_exact.trim().is_empty() {
        errors.push("observed_text_exact must not be empty".to_string());
    }
    if !candidate.candidate_bug_type.trim().is_empty()
        && !active_bug_types.contains(&candidate.candidate_bug_type)
    {
        errors.push(format!(
            "candidate_bug_type '{}' not found in active severity rules",
            candidate.candidate_bug_type
        ));
    }
    if candidate.confidence < 0.0 || candidate.confidence > 1.0 {
        errors.push(format!("confidence {} is out of valid range [0.0, 1.0]", candidate.confidence));
    }
    if candidate.defect_summary.trim().is_empty() {
        errors.push("defect_summary must not be empty".to_string());
    }
    if candidate.confidence < 0.3 && candidate.confidence > 0.0 {
        warnings.push(format!("Low confidence ({}) â€” issue will require manual review", candidate.confidence));
    }
    if candidate.suggested_text.trim().is_empty() {
        warnings.push("suggested_text is empty â€” correction not provided".to_string());
    }

    ValidationResult { valid: errors.is_empty(), errors, warnings }
}

// ===========================================================================
// Phase5: Image LQA Analysis â€” types and persistence
// ===========================================================================

/// A project screenshot/image registered for analysis (Phase 5).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectImage {
    pub id: String,
    pub project_id: String,
    pub filename: String,
    pub file_path: String,
    pub mime_type: String,
    pub file_size: i64,
    pub width: i32,
    pub height: i32,
    pub content_hash: String,
    pub ocr_text: String,
pub video_frame_id: String,
    pub active: bool,
    pub created_at: String,
}

/// Status of an image analysis job (Phase 5).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageAnalysis {
    pub id: String,
    pub project_id: String,
    pub image_id: String,
    pub status: String,
    pub analyzer_version: String,
    pub severity_snapshot_id: String,
    pub glossary_snapshot_id: String,
    pub precedent_snapshot_ids: String,
    pub reference_snapshot_ids: String,
    pub error_message: Option<String>,
    pub proposal_count: i32,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Atomic issue proposal produced by analysis (Phase 5).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IssueProposal {
    pub id: String,
    pub project_id: String,
    pub analysis_id: String,
    pub image_id: String,
    pub proposal_index: i32,
    pub issue_category: String,
    pub issue_subcategory: String,
    pub bug_type: String,
    pub title: String,
    pub description: String,
    pub detected_text: String,
    pub glossary_findings: String,
    pub severity_candidate: String,
    pub severity_reasoning: String,
    pub severity_rule_id: String,
    pub confidence: f64,
    pub review_status: String,
    pub coord_system: String,
    pub image_width: i32,
    pub image_height: i32,
    pub bbox_x: f64,
    pub bbox_y: f64,
    pub bbox_w: f64,
    pub bbox_h: f64,
    pub spatial_description: String,
    pub spatial_confidence: f64,
    pub precedent_ids: String,
    pub reference_ids: String,
    pub active: bool,
    pub lifecycle: String,
    pub created_at: String,
}

/// Register an uploaded image for a project (file copied to storage by caller).
pub fn upload_project_image(
    conn: &Connection,
    project_id: &str,
    filename: &str,
    file_path: &str,
    mime_type: &str,
    file_size: i64,
    width: i32,
    height: i32,
    content_hash: &str,
) -> Result<ProjectImage, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO project_images (id, project_id, filename, file_path, mime_type,
         file_size, width, height, content_hash, video_frame_id, active, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '', 1, ?10)",
        params![id, project_id, filename, file_path, mime_type, file_size, width, height, content_hash, now],
    ).map_err(|e| format!("Failed to insert project image: {}", e))?;
    get_project_image(conn, &id)
}
/// Register a video frame image as a project_image for Phase 5 analysis bridge.
pub fn register_frame_image(
    conn: &Connection,
    project_id: &str,
    frame_id: &str,
    filename: &str,
    file_path: &str,
    width: i32,
    height: i32,
    content_hash: &str,
) -> Result<ProjectImage, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let file_size = std::fs::metadata(file_path)
        .map(|m| m.len() as i64)
        .unwrap_or(0);
    let mime = "image/png";
    conn.execute(
        "INSERT INTO project_images (id, project_id, filename, file_path, mime_type,
         file_size, width, height, content_hash, video_frame_id, active, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1, ?11)",
        params![id, project_id, filename, file_path, mime, file_size, width, height, content_hash, frame_id, now],
    ).map_err(|e| format!("Failed to register frame image: {}", e))?;
    // Update the video_frames row to link back
    conn.execute(
        "UPDATE video_frames SET frame_image_id = ?1 WHERE id = ?2",
        params![id, frame_id],
    ).map_err(|e| format!("Failed to update frame image link: {}", e))?;
    get_project_image(conn, &id)
}

/// Fetch a single registered project image.
pub fn get_project_image(conn: &Connection, image_id: &str) -> Result<ProjectImage, String> {
    let img = conn.query_row(
        "SELECT id, project_id, filename, file_path, mime_type, file_size,
                width, height, content_hash, ocr_text, video_frame_id, active, created_at FROM project_images WHERE id = ?1",
        params![image_id],
        |row| Ok(ProjectImage {
            id: row.get(0)?, project_id: row.get(1)?, filename: row.get(2)?,
            file_path: row.get(3)?, mime_type: row.get(4)?, file_size: row.get(5)?,
            width: row.get(6)?, height: row.get(7)?, content_hash: row.get(8)?,
            ocr_text: row.get(9)?, video_frame_id: row.get(10)?, active: row.get::<_, i32>(11)? != 0, created_at: row.get(12)?,
        }),
    ).map_err(|e| format!("Project image not found ({}): {}", image_id, e))?;
    Ok(img)
}

/// List all registered images for a project (newest first).
pub fn list_project_images(conn: &Connection, project_id: &str) -> Result<Vec<ProjectImage>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, filename, file_path, mime_type, file_size,
                width, height, content_hash, ocr_text, video_frame_id, active, created_at
         FROM project_images WHERE project_id = ?1 ORDER BY created_at DESC",
    ).map_err(|e| format!("Failed to prepare images query: {}", e))?;
    let imgs = stmt.query_map(params![project_id], |row| Ok(ProjectImage {
        id: row.get(0)?, project_id: row.get(1)?, filename: row.get(2)?,
        file_path: row.get(3)?, mime_type: row.get(4)?, file_size: row.get(5)?,
        width: row.get(6)?, height: row.get(7)?, content_hash: row.get(8)?,
        ocr_text: row.get(9)?, video_frame_id: row.get(10)?, active: row.get::<_, i32>(11)? != 0, created_at: row.get(12)?,
    }))
    .map_err(|e| format!("Images query error: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Images collect error: {}", e))?;
    Ok(imgs)
}

/// Persist the OCR composite text for an image (used by terminology evaluation).
pub fn set_image_ocr_text(conn: &Connection, image_id: &str, ocr_text: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE project_images SET ocr_text = ?1 WHERE id = ?2",
        params![ocr_text, image_id],
    ).map_err(|e| format!("Failed to store OCR text: {}", e))?;
    Ok(())
}

// === PHASE5_DB_PART2 ===

/// Cross-image terminology consistency finding (Phase 5).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminologyFinding {
    pub kind: String,
    pub observed: String,
    pub expected: String,
    pub source_image_id: String,
    pub source_analysis_id: String,
    pub note: String,
}

/// Result payload returned by an analysis run (Phase 5).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageAnalysisResult {
    pub analysis: ImageAnalysis,
    pub proposals: Vec<IssueProposal>,
    pub terminology_findings: Vec<TerminologyFinding>,
}

/// Create an analysis job record in `running` state.
pub fn record_image_analysis(
    conn: &Connection,
    project_id: &str,
    image_id: &str,
    analyzer_version: &str,
    severity_snapshot_id: &str,
    glossary_snapshot_id: &str,
    precedent_snapshot_ids: &str,
    reference_snapshot_ids: &str,
) -> Result<ImageAnalysis, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO image_analyses (id, project_id, image_id, status, analyzer_version,
         severity_snapshot_id, glossary_snapshot_id, precedent_snapshot_ids,
         reference_snapshot_ids, proposal_count, created_at)
         VALUES (?1, ?2, ?3, 'running', ?4, ?5, ?6, ?7, ?8, 0, ?9)",
        params![id, project_id, image_id, analyzer_version, severity_snapshot_id,
                glossary_snapshot_id, precedent_snapshot_ids, reference_snapshot_ids, now],
    ).map_err(|e| format!("Failed to insert analysis: {}", e))?;
    get_image_analysis(conn, &id)
}

/// Finalize an analysis record (status + proposal count).
pub fn finalize_image_analysis(
    conn: &Connection,
    analysis_id: &str,
    status: &str,
    error_message: Option<&str>,
    proposal_count: i32,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE image_analyses SET status = ?1, error_message = ?2,
         proposal_count = ?3, completed_at = ?4 WHERE id = ?5",
        params![status, error_message, proposal_count, now, analysis_id],
    ).map_err(|e| format!("Failed to finalize analysis: {}", e))?;
    Ok(())
}

/// Fetch a single analysis record.
pub fn get_image_analysis(conn: &Connection, analysis_id: &str) -> Result<ImageAnalysis, String> {
    let a = conn.query_row(
        "SELECT id, project_id, image_id, status, analyzer_version, severity_snapshot_id,
                glossary_snapshot_id, precedent_snapshot_ids, reference_snapshot_ids,
                error_message, proposal_count, created_at, completed_at
         FROM image_analyses WHERE id = ?1",
        params![analysis_id],
        |row| Ok(ImageAnalysis {
            id: row.get(0)?, project_id: row.get(1)?, image_id: row.get(2)?,
            status: row.get(3)?, analyzer_version: row.get(4)?,
            severity_snapshot_id: row.get(5)?, glossary_snapshot_id: row.get(6)?,
            precedent_snapshot_ids: row.get(7)?, reference_snapshot_ids: row.get(8)?,
            error_message: row.get(9)?, proposal_count: row.get(10)?,
            created_at: row.get(11)?, completed_at: row.get(12)?,
        }),
    ).map_err(|e| format!("Analysis not found ({}): {}", analysis_id, e))?;
    Ok(a)
}

// === PHASE5_DB_PART3 ===

/// Insert an atomic issue proposal produced by analysis.
pub fn insert_issue_proposal(conn: &Connection, p: &IssueProposal) -> Result<(), String> {
    conn.execute(
        "INSERT INTO issue_proposals (
            id, project_id, analysis_id, image_id, proposal_index,
            issue_category, issue_subcategory, bug_type, title, description,
            detected_text, glossary_findings, severity_candidate, severity_reasoning,
            severity_rule_id, confidence, review_status, coord_system,
            image_width, image_height, bbox_x, bbox_y, bbox_w, bbox_h,
            spatial_description, spatial_confidence, precedent_ids, reference_ids,
            active, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,
                 ?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,1,?29)",
        params![
            p.id, p.project_id, p.analysis_id, p.image_id, p.proposal_index,
            p.issue_category, p.issue_subcategory, p.bug_type, p.title, p.description,
            p.detected_text, p.glossary_findings, p.severity_candidate, p.severity_reasoning,
            p.severity_rule_id, p.confidence, p.review_status, p.coord_system,
            p.image_width, p.image_height, p.bbox_x, p.bbox_y, p.bbox_w, p.bbox_h,
            p.spatial_description, p.spatial_confidence, p.precedent_ids, p.reference_ids, p.created_at
        ],
    ).map_err(|e| format!("Failed to insert issue proposal: {}", e))?;
    Ok(())
}

fn proposal_from_row(row: &rusqlite::Row) -> Result<IssueProposal, rusqlite::Error> {
    Ok(IssueProposal {
        id: row.get(0)?, project_id: row.get(1)?, analysis_id: row.get(2)?,
        image_id: row.get(3)?, proposal_index: row.get(4)?, issue_category: row.get(5)?,
        issue_subcategory: row.get(6)?, bug_type: row.get(7)?, title: row.get(8)?,
        description: row.get(9)?, detected_text: row.get(10)?, glossary_findings: row.get(11)?,
        severity_candidate: row.get(12)?, severity_reasoning: row.get(13)?,
        severity_rule_id: row.get(14)?, confidence: row.get(15)?, review_status: row.get(16)?,
        coord_system: row.get(17)?, image_width: row.get(18)?, image_height: row.get(19)?,
        bbox_x: row.get(20)?, bbox_y: row.get(21)?, bbox_w: row.get(22)?, bbox_h: row.get(23)?,
        spatial_description: row.get(24)?, spatial_confidence: row.get(25)?,
        precedent_ids: row.get(26)?, reference_ids: row.get(27)?,
        active: row.get::<_, i32>(28)? != 0, lifecycle: row.get(29)?, created_at: row.get(30)?,
    })
}

/// List issue proposals for a project, optionally filtered by image.
pub fn list_issue_proposals(
    conn: &Connection,
    project_id: &str,
    image_id: Option<&str>,
) -> Result<Vec<IssueProposal>, String> {
    const COLS: &str = "id, project_id, analysis_id, image_id, proposal_index, issue_category,
        issue_subcategory, bug_type, title, description, detected_text, glossary_findings,
        severity_candidate, severity_reasoning, severity_rule_id, confidence, review_status,
        coord_system, image_width, image_height, bbox_x, bbox_y, bbox_w, bbox_h,
        spatial_description, spatial_confidence, precedent_ids, reference_ids, active, lifecycle, created_at";

    let (sql, params): (&str, Vec<rusqlite::types::Value>) = match image_id {
        Some(iid) => (
            "SELECT {COLS} FROM issue_proposals WHERE project_id = ?1 AND image_id = ?2 AND active = 1 ORDER BY proposal_index ASC",
            vec![project_id.to_string().into(), iid.to_string().into()],
        ),
        None => (
            "SELECT {COLS} FROM issue_proposals WHERE project_id = ?1 AND active = 1 ORDER BY created_at DESC",
            vec![project_id.to_string().into()],
        ),
    };
    let sql = sql.replace("{COLS}", COLS);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Prepare: {}", e))?;
    let proposals = stmt
        .query_map(rusqlite::params_from_iter(params), proposal_from_row)
        .map_err(|e| format!("Proposals query error: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Proposals collect error: {}", e))?;
    Ok(proposals)
}

// === PHASE5_DB_PART4 ===

/// Fetch a single issue proposal.
pub fn get_issue_proposal(conn: &Connection, proposal_id: &str) -> Result<IssueProposal, String> {
    let p = conn.query_row(
        "SELECT id, project_id, analysis_id, image_id, proposal_index, issue_category,
                issue_subcategory, bug_type, title, description, detected_text, glossary_findings,
                severity_candidate, severity_reasoning, severity_rule_id, confidence, review_status,
                coord_system, image_width, image_height, bbox_x, bbox_y, bbox_w, bbox_h,
                spatial_description, spatial_confidence, precedent_ids, reference_ids, active, lifecycle, created_at
         FROM issue_proposals WHERE id = ?1",
        params![proposal_id],
        proposal_from_row,
    ).map_err(|e| format!("Issue proposal not found ({}): {}", proposal_id, e))?;
    Ok(p)
}

/// Update the human review status of an issue proposal.
pub fn update_issue_proposal_review(
    conn: &Connection,
    proposal_id: &str,
    review_status: &str,
) -> Result<IssueProposal, String> {
    let allowed = matches!(review_status, "pending" | "approved" | "rejected" | "review_required");
    if !allowed {
        return Err(format!("Invalid review_status: {}", review_status));
    }
    conn.execute(
        "UPDATE issue_proposals SET review_status = ?1 WHERE id = ?2",
        params![review_status, proposal_id],
    ).map_err(|e| format!("Failed to update review status: {}", e))?;
    get_issue_proposal(conn, proposal_id)
}

// ===========================================================================
// Phase 6: Visual Evidence Engine â€” persistence
// ===========================================================================

/// A persisted visual evidence annotation over a project image (Phase 6).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvidenceAnnotation {
    pub id: String,
    pub project_id: String,
    pub issue_proposal_id: String,
    pub project_image_id: String,
    pub analysis_id: String,
    pub annotation_type: String,
    pub coordinate_space: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub origin: String,
    pub confidence: Option<f64>,
    pub review_state: String,
    pub created_at: String,
    pub updated_at: String,
}

fn evidence_from_row(row: &rusqlite::Row) -> Result<EvidenceAnnotation, rusqlite::Error> {
    Ok(EvidenceAnnotation {
        id: row.get(0)?,
        project_id: row.get(1)?,
        issue_proposal_id: row.get(2)?,
        project_image_id: row.get(3)?,
        analysis_id: row.get(4)?,
        annotation_type: row.get(5)?,
        coordinate_space: row.get(6)?,
        x: row.get(7)?,
        y: row.get(8)?,
        width: row.get(9)?,
        height: row.get(10)?,
        origin: row.get(11)?,
        confidence: row.get(12)?,
        review_state: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

const EVIDENCE_COLS: &str = "id, project_id, issue_proposal_id, project_image_id, analysis_id,
    annotation_type, coordinate_space, x, y, width, height, origin, confidence, review_state,
    created_at, updated_at";

/// Persist a new evidence annotation. Caller must have validated coordinates,
/// project scope, and the issue/image/analysis relationships beforehand.
pub fn insert_evidence_annotation(
    conn: &Connection,
    a: &EvidenceAnnotation,
) -> Result<EvidenceAnnotation, String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO evidence_annotations (id, project_id, issue_proposal_id, project_image_id,
         analysis_id, annotation_type, coordinate_space, x, y, width, height, origin, confidence,
         review_state, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
        params![
            a.id, a.project_id, a.issue_proposal_id, a.project_image_id, a.analysis_id,
            a.annotation_type, a.coordinate_space, a.x, a.y, a.width, a.height, a.origin,
            a.confidence, a.review_state, now, now
        ],
    )
    .map_err(|e| format!("Failed to insert evidence annotation: {}", e))?;
    get_evidence_annotation(conn, &a.project_id, &a.id)
}

/// Fetch a single evidence annotation (project-scoped to prevent cross-project reads).
pub fn get_evidence_annotation(
    conn: &Connection,
    project_id: &str,
    annotation_id: &str,
) -> Result<EvidenceAnnotation, String> {
    let a = conn
        .query_row(
            &format!("SELECT {EVIDENCE_COLS} FROM evidence_annotations WHERE id = ?1 AND project_id = ?2"),
            params![annotation_id, project_id],
            evidence_from_row,
        )
        .map_err(|e| format!("Evidence annotation not found ({}): {}", annotation_id, e))?;
    Ok(a)
}

/// List evidence annotations for an issue (project-scoped).
pub fn list_evidence_by_issue(
    conn: &Connection,
    project_id: &str,
    issue_proposal_id: &str,
) -> Result<Vec<EvidenceAnnotation>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {EVIDENCE_COLS} FROM evidence_annotations WHERE project_id = ?1 AND issue_proposal_id = ?2 ORDER BY created_at ASC"
        ))
        .map_err(|e| format!("Failed to prepare evidence query: {}", e))?;
    let rows = stmt
        .query_map(params![project_id, issue_proposal_id], evidence_from_row)
        .map_err(|e| format!("Evidence query error: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Evidence collect error: {}", e))?;
    Ok(rows)
}

/// Update geometry/type/origin of an evidence annotation. Coordinates and
/// enums must be pre-validated by the caller. Origin is forced to USER_EDITED
/// and review_state to REVIEWED on edit.
pub fn update_evidence_annotation(
    conn: &Connection,
    a: &EvidenceAnnotation,
) -> Result<EvidenceAnnotation, String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE evidence_annotations SET annotation_type = ?1, x = ?2, y = ?3, width = ?4,
         height = ?5, origin = ?6, review_state = ?7, updated_at = ?8
         WHERE id = ?9 AND project_id = ?10",
        params![
            a.annotation_type, a.x, a.y, a.width, a.height, a.origin, a.review_state, now,
            a.id, a.project_id
        ],
    )
    .map_err(|e| format!("Failed to update evidence annotation: {}", e))?;
    get_evidence_annotation(conn, &a.project_id, &a.id)
}

/// Update only the review lifecycle state of an evidence annotation.
pub fn update_evidence_review_state(
    conn: &Connection,
    project_id: &str,
    annotation_id: &str,
    review_state: &str,
) -> Result<EvidenceAnnotation, String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE evidence_annotations SET review_state = ?1, updated_at = ?2 WHERE id = ?3 AND project_id = ?4",
        params![review_state, now, annotation_id, project_id],
    )
    .map_err(|e| format!("Failed to update evidence review state: {}", e))?;
    get_evidence_annotation(conn, project_id, annotation_id)
}

/// Deterministic snapshot id from active rule ids (provenance for final severity).
pub fn compute_snapshot_id(ids: &[String]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut sorted = ids.to_vec();
    sorted.sort();
    let joined = sorted.join("|");
    let mut hasher = DefaultHasher::new();
    joined.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Compute a content hash for a file (non-cryptographic, for dedupe/identity).
pub fn hash_file_contents(path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read file for hashing: {}", e))?;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    Ok(format!("{:x}", hasher.finish()))
}


// === SECURITY_GATE ===
// Image file-ingestion validation (Security Gate, STEP 4).
// Rejects non-files, symlinks, disallowed extensions, empty/oversized files,
// and content that does not match a supported image magic number.
// Extension alone is NEVER treated as proof of validity.

/// Result of a successful, safe image-upload validation.
pub struct ValidatedUpload {
    pub canonical_path: String,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
}

/// Maximum accepted image size: 50 MiB. Larger files are DoS-prone for
/// OCR/analysis and unlikely to be a real localization screenshot.
pub const MAX_IMAGE_BYTES: u64 = 50 * 1024 * 1024;

/// Allowed image extensions. Import is restricted to these.
const ALLOWED_IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "gif"];

/// Safe segment for filesystem path components (project ids, etc.).
/// Rejects empties, overly long, and any non `[A-Za-z0-9_-]` characters
/// (including path separators and `..`) to prevent path traversal.
pub fn is_safe_id_segment(s: &str) -> bool {
    if s.is_empty() || s.len() > 64 {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn mime_from_extension(path: &str) -> String {
    let lower = path.to_lowercase();
    if lower.ends_with(".png") {
        "image/png".to_string()
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg".to_string()
    } else if lower.ends_with(".webp") {
        "image/webp".to_string()
    } else if lower.ends_with(".bmp") {
        "image/bmp".to_string()
    } else if lower.ends_with(".gif") {
        "image/gif".to_string()
    } else {
        "image/octet-stream".to_string()
    }
}

fn is_allowed_image_ext(name: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| ALLOWED_IMAGE_EXTS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Validate that the leading bytes match a supported image container.
fn magic_matches_image(path: &str) -> bool {
    use std::io::Read;
    let mut f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buf = [0u8; 12];
    let n = f.read(&mut buf).unwrap_or(0);
    if n < 4 {
        return false;
    }
    // PNG
    if n >= 8 && &buf[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return true;
    }
    // JPEG
    if &buf[0..2] == [0xFF, 0xD8] {
        return true;
    }
    // GIF
    if &buf[0..3] == [0x47, 0x49, 0x46] {
        return true;
    }
    // BMP
    if &buf[0..2] == [0x42, 0x4D] {
        return true;
    }
    // WEBP: "RIFF" .... "WEBP"
    if n >= 12 && &buf[0..4] == *b"RIFF" && &buf[8..12] == *b"WEBP" {
        return true;
    }
    false
}

/// Validate an image selected for upload. Returns canonical, scoped details,
/// or a typed error describing the rejection.
pub fn validate_image_upload(source_path: &str) -> Result<ValidatedUpload, String> {
    // Reject symlinks explicitly: metadata() follows links, so also inspect
    // the raw link metadata.
    let link_meta = std::fs::symlink_metadata(source_path)
        .map_err(|e| format!("Cannot access selected file: {}", e))?;
    if link_meta.file_type().is_symlink() {
        return Err("Symbolic links are not allowed for image import.".to_string());
    }

    let meta = std::fs::metadata(source_path)
        .map_err(|e| format!("Cannot read selected file: {}", e))?;
    if meta.is_dir() {
        return Err("Selected path is a directory, not an image file.".to_string());
    }

    let canon = std::fs::canonicalize(source_path)
        .map_err(|e| format!("Cannot resolve selected path: {}", e))?;
    let canon_str = canon.to_string_lossy().to_string();

    let filename = canon
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Selected file has no usable name.".to_string())?
        .to_string();

    if !is_allowed_image_ext(&filename) {
        return Err(format!(
            "Unsupported image type '{}'. Allowed: png, jpg, jpeg, webp, bmp, gif.",
            filename
        ));
    }

    if meta.len() == 0 {
        return Err("Selected image file is empty.".to_string());
    }
    if meta.len() > MAX_IMAGE_BYTES {
        return Err(format!(
            "Image file exceeds the {} MiB limit.",
            MAX_IMAGE_BYTES / 1024 / 1024
        ));
    }

    if !magic_matches_image(&canon_str) {
        return Err("File content does not match a supported image format.".to_string());
    }

    let mime_type = mime_from_extension(&filename);
    Ok(ValidatedUpload {
        canonical_path: canon_str,
        filename,
        mime_type,
        file_size: meta.len() as i64,
    })
}

// === PHASE5_DB_END ===

// === PHASE7_DB_VIDEO ===

#[derive(Debug, Serialize, Clone)]
pub struct ProjectVideo {
    pub id: String,
    pub project_id: String,
    pub original_filename: String,
    pub stored_path: String,
    pub content_hash: String,
    pub file_size: i64,
    pub media_type: String,
    pub duration_secs: f64,
    pub width: i32,
    pub height: i32,
    pub frame_rate: f64,
    pub active: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct VideoExtraction {
    pub id: String,
    pub project_id: String,
    pub video_id: String,
    pub extraction_strategy: String,
    pub interval_secs: f64,
    pub status: String,
    pub extractor_version: String,
    pub total_frames: i32,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct VideoFrame {
    pub id: String,
    pub project_id: String,
    pub video_id: String,
    pub extraction_id: String,
    pub frame_index: i32,
    pub timestamp_ms: i64,
    pub timestamp_display: String,
    pub image_path: String,
    pub content_hash: String,
    pub width: i32,
    pub height: i32,
pub frame_image_id: String,
    pub created_at: String,
}




pub fn insert_project_video(
    conn: &Connection,
    project_id: &str,
    filename: &str,
    stored_path: &str,
    file_size: i64,
    duration_secs: f64,
    width: i32,
    height: i32,
    frame_rate: f64,
    media_type: &str,
) -> Result<ProjectVideo, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let hash = crate::video_engine::hash_file(std::path::Path::new(stored_path))?;
    conn.execute(
        "INSERT INTO project_videos (id, project_id, original_filename, stored_path, content_hash,
         file_size, media_type, duration_secs, width, height, frame_rate, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![id, project_id, filename, stored_path, hash, file_size, media_type,
            duration_secs, width, height, frame_rate, now],
    ).map_err(|e| format!("insert_project_video: {}", e))?;
    Ok(ProjectVideo { id, project_id: project_id.to_string(), original_filename: filename.to_string(),
        stored_path: stored_path.to_string(), content_hash: hash, file_size, media_type: media_type.to_string(),
        duration_secs, width, height, frame_rate, active: 1, created_at: now })
}

pub fn list_project_videos(conn: &Connection, project_id: &str) -> Result<Vec<ProjectVideo>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, original_filename, stored_path, content_hash, file_size, media_type,
         duration_secs, width, height, frame_rate, active, created_at
         FROM project_videos WHERE project_id=?1 AND active=1 ORDER BY created_at DESC"
    ).map_err(|e| format!("list_project_videos: {}", e))?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(ProjectVideo {
            id: row.get(0)?, project_id: row.get(1)?, original_filename: row.get(2)?,
            stored_path: row.get(3)?, content_hash: row.get(4)?, file_size: row.get(5)?,
            media_type: row.get(6)?, duration_secs: row.get(7)?, width: row.get(8)?,
            height: row.get(9)?, frame_rate: row.get(10)?, active: row.get(11)?,
            created_at: row.get(12)?,
        })
    }).map_err(|e| format!("list_project_videos: {}", e))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| format!("list_project_videos: {}", e))
}

pub fn get_project_video(conn: &Connection, video_id: &str) -> Result<ProjectVideo, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, original_filename, stored_path, content_hash, file_size, media_type,
         duration_secs, width, height, frame_rate, active, created_at
         FROM project_videos WHERE id=?1"
    ).map_err(|e| format!("get_project_video: {}", e))?;
    stmt.query_row(params![video_id], |row| {
        Ok(ProjectVideo {
            id: row.get(0)?, project_id: row.get(1)?, original_filename: row.get(2)?,
            stored_path: row.get(3)?, content_hash: row.get(4)?, file_size: row.get(5)?,
            media_type: row.get(6)?, duration_secs: row.get(7)?, width: row.get(8)?,
            height: row.get(9)?, frame_rate: row.get(10)?, active: row.get(11)?,
            created_at: row.get(12)?,
        })
    }).map_err(|e| format!("get_project_video: {}", e))
}
pub fn insert_video_extraction(
    conn: &Connection,
    project_id: &str,
    video_id: &str,
    interval_secs: f64,
    extractor_version: &str,
) -> Result<VideoExtraction, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO video_extractions (id, project_id, video_id, extraction_strategy, interval_secs,
         status, extractor_version, total_frames, started_at)
         VALUES (?1,?2,?3,'interval',?4,'running',?5,0,?6)",
        params![id, project_id, video_id, interval_secs, extractor_version, now],
    ).map_err(|e| format!("insert_video_extraction: {}", e))?;
    Ok(VideoExtraction { id, project_id: project_id.to_string(), video_id: video_id.to_string(),
        extraction_strategy: "interval".to_string(), interval_secs,
        status: "running".to_string(), extractor_version: extractor_version.to_string(),
        total_frames: 0, error_message: None, started_at: now, completed_at: None })
}

pub fn complete_video_extraction(
    conn: &Connection,
    extraction_id: &str,
    total_frames: i32,
    error_message: Option<&str>,
) -> Result<VideoExtraction, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let status = if error_message.is_some() { "failed" } else { "completed" };
    conn.execute(
        "UPDATE video_extractions SET status=?1, total_frames=?2, error_message=?3, completed_at=?4 WHERE id=?5",
        params![status, total_frames, error_message, now, extraction_id],
    ).map_err(|e| format!("complete_video_extraction: {}", e))?;
    get_video_extraction(conn, extraction_id)
}

pub fn get_video_extraction(conn: &Connection, extraction_id: &str) -> Result<VideoExtraction, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, video_id, extraction_strategy, interval_secs, status,
         extractor_version, total_frames, error_message, started_at, completed_at
         FROM video_extractions WHERE id=?1"
    ).map_err(|e| format!("get_video_extraction: {}", e))?;
    stmt.query_row(params![extraction_id], |row| {
        Ok(VideoExtraction {
            id: row.get(0)?, project_id: row.get(1)?, video_id: row.get(2)?,
            extraction_strategy: row.get(3)?, interval_secs: row.get(4)?,
            status: row.get(5)?, extractor_version: row.get(6)?,
            total_frames: row.get(7)?, error_message: row.get(8)?,
            started_at: row.get(9)?, completed_at: row.get(10)?,
        })
    }).map_err(|e| format!("get_video_extraction: {}", e))
}

pub fn list_video_extractions(conn: &Connection, video_id: &str) -> Result<Vec<VideoExtraction>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, video_id, extraction_strategy, interval_secs, status,
         extractor_version, total_frames, error_message, started_at, completed_at
         FROM video_extractions WHERE video_id=?1 ORDER BY started_at DESC"
    ).map_err(|e| format!("list_video_extractions: {}", e))?;
    let rows = stmt.query_map(params![video_id], |row| {
        Ok(VideoExtraction {
            id: row.get(0)?, project_id: row.get(1)?, video_id: row.get(2)?,
            extraction_strategy: row.get(3)?, interval_secs: row.get(4)?,
            status: row.get(5)?, extractor_version: row.get(6)?,
            total_frames: row.get(7)?, error_message: row.get(8)?,
            started_at: row.get(9)?, completed_at: row.get(10)?,
        })
    }).map_err(|e| format!("list_video_extractions: {}", e))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| format!("list_video_extractions: {}", e))
}
pub fn insert_video_frame(
    conn: &Connection,
    project_id: &str,
    video_id: &str,
    extraction_id: &str,
    frame_index: i32,
    timestamp_ms: i64,
    timestamp_display: &str,
    image_path: &str,
    width: i32,
    height: i32,
) -> Result<VideoFrame, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let hash = crate::video_engine::hash_file(std::path::Path::new(image_path))?;
    conn.execute(
        "INSERT INTO video_frames (id, project_id, video_id, extraction_id, frame_index,
         timestamp_ms, timestamp_display, image_path, content_hash, width, height, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![id, project_id, video_id, extraction_id, frame_index, timestamp_ms,
            timestamp_display, image_path, hash, width, height, now],
    ).map_err(|e| format!("insert_video_frame: {}", e))?;
    Ok(VideoFrame { id, project_id: project_id.to_string(), video_id: video_id.to_string(),
        extraction_id: extraction_id.to_string(), frame_index, timestamp_ms,
        timestamp_display: timestamp_display.to_string(), image_path: image_path.to_string(),
        content_hash: hash, width, height, frame_image_id: String::new(), created_at: now })
}

pub fn list_video_frames(
    conn: &Connection,
    extraction_id: &str,
) -> Result<Vec<VideoFrame>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, video_id, extraction_id, frame_index, timestamp_ms,
         timestamp_display, image_path, content_hash, width, height, frame_image_id, created_at
         FROM video_frames WHERE extraction_id=?1 ORDER BY frame_index ASC"
    ).map_err(|e| format!("list_video_frames: {}", e))?;
    let rows = stmt.query_map(params![extraction_id], |row| {
        Ok(VideoFrame {
            id: row.get(0)?, project_id: row.get(1)?, video_id: row.get(2)?,
            extraction_id: row.get(3)?, frame_index: row.get(4)?, timestamp_ms: row.get(5)?,
            timestamp_display: row.get(6)?, image_path: row.get(7)?,
            content_hash: row.get(8)?, width: row.get(9)?, height: row.get(10)?, frame_image_id: row.get(11)?, created_at: row.get(12)?,
        })
    }).map_err(|e| format!("list_video_frames: {}", e))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| format!("list_video_frames: {}", e))
}

pub fn get_video_frame(conn: &Connection, frame_id: &str) -> Result<VideoFrame, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, video_id, extraction_id, frame_index, timestamp_ms,
         timestamp_display, image_path, content_hash, width, height, frame_image_id, created_at
         FROM video_frames WHERE id=?1"
    ).map_err(|e| format!("get_video_frame: {}", e))?;
    stmt.query_row(params![frame_id], |row| {
        Ok(VideoFrame {
            id: row.get(0)?, project_id: row.get(1)?, video_id: row.get(2)?,
            extraction_id: row.get(3)?, frame_index: row.get(4)?, timestamp_ms: row.get(5)?,
            timestamp_display: row.get(6)?, image_path: row.get(7)?,
            content_hash: row.get(8)?, width: row.get(9)?, height: row.get(10)?, frame_image_id: row.get(11)?, created_at: row.get(12)?,
        })
    }).map_err(|e| format!("get_video_frame: {}", e))
}

// === PHASE7_DB_END ===
// === PHASE8_DB_START ===
/// Insert a duplicate relationship between two issues (Phase 8).
pub fn insert_duplicate_relationship(
    conn: &Connection, id: &str, project_id: &str,
    source_issue_id: &str, duplicate_issue_id: &str,
    confidence: f64, match_signals: &str,
) -> Result<DuplicateRelationship, String> {
    conn.execute(
        "INSERT INTO duplicate_relationships (id, project_id, source_issue_id, duplicate_issue_id, confidence, match_signals, confirmed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
        rusqlite::params![id, project_id, source_issue_id, duplicate_issue_id, confidence, match_signals],
    ).map_err(|e| format!("insert duplicate: {}", e))?;
    get_duplicate_relationship(conn, id)
}

pub fn get_duplicate_relationship(conn: &Connection, id: &str) -> Result<DuplicateRelationship, String> {
    conn.query_row(
        "SELECT id, project_id, source_issue_id, duplicate_issue_id, confidence, match_signals,
                confirmed, confirmed_by, created_at, updated_at FROM duplicate_relationships WHERE id=?1",
        rusqlite::params![id],
        |row| Ok(DuplicateRelationship {
            id: row.get(0)?, project_id: row.get(1)?, source_issue_id: row.get(2)?,
            duplicate_issue_id: row.get(3)?, confidence: row.get(4)?, match_signals: row.get(5)?,
            confirmed: row.get::<_,i32>(6)? != 0, confirmed_by: row.get(7)?,
            created_at: row.get(8)?, updated_at: row.get(9)?,
        }),
    ).map_err(|e| format!("get dup rel: {}", e))
}

pub fn list_duplicate_relationships(conn: &Connection, project_id: &str) -> Result<Vec<DuplicateRelationship>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, source_issue_id, duplicate_issue_id, confidence, match_signals,
                confirmed, confirmed_by, created_at, updated_at FROM duplicate_relationships WHERE project_id=?1 ORDER BY created_at DESC",
    ).map_err(|e| format!("prep dup list: {}", e))?;
    let rows = stmt.query_map(rusqlite::params![project_id], |row| Ok(DuplicateRelationship {
        id: row.get(0)?, project_id: row.get(1)?, source_issue_id: row.get(2)?,
        duplicate_issue_id: row.get(3)?, confidence: row.get(4)?, match_signals: row.get(5)?,
        confirmed: row.get::<_,i32>(6)? != 0, confirmed_by: row.get(7)?,
        created_at: row.get(8)?, updated_at: row.get(9)?,
    })).map_err(|e| format!("query dup: {}", e))?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e| format!("collect dup: {}", e))
}

pub fn confirm_duplicate_relationship(conn: &Connection, id: &str) -> Result<DuplicateRelationship, String> {
    conn.execute("UPDATE duplicate_relationships SET confirmed=1, confirmed_by='human', updated_at=datetime('now') WHERE id=?1",
        rusqlite::params![id]).map_err(|e| format!("confirm dup: {}", e))?;
    get_duplicate_relationship(conn, id)
}

pub fn find_duplicate_candidates(conn: &Connection, issue_id: &str, project_id: &str) -> Result<Vec<(DuplicateRelationship, IssueProposal)>, String> {
    let source = get_issue_proposal(conn, issue_id)?;
    let all = list_issue_proposals(conn, project_id, None)?;
    let mut candidates = Vec::new();
    for other in &all {
        if other.id == issue_id { continue; }
        let (confidence, signals) = crate::issue_lifecycle::score_duplicate_candidate(&source, other);
        if confidence > 0.3 {
            candidates.push((DuplicateRelationship {
                id: format!("dup-{}-{}", issue_id, other.id), project_id: project_id.into(),
                source_issue_id: issue_id.into(), duplicate_issue_id: other.id.clone(),
                confidence, match_signals: serde_json::to_string(&signals).unwrap_or_default(),
                confirmed: false, confirmed_by: String::new(),
                created_at: String::new(), updated_at: String::new(),
            }, other.clone()));
        }
    }
    Ok(candidates)
}
// --- Known Issues ---
pub fn create_known_issue(
    conn: &Connection, id: &str, project_id: &str, title: &str, description: &str,
    category: &str, subcategory: &str, bug_type: &str, severity: &str, source_issue_id: &str,
) -> Result<KnownIssue, String> {
    conn.execute(
        "INSERT INTO known_issues (id, project_id, title, description, category, subcategory, bug_type, severity, source_issue_id)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        rusqlite::params![id, project_id, title, description, category, subcategory, bug_type, severity, source_issue_id],
    ).map_err(|e| format!("create known: {}", e))?;
    get_known_issue(conn, id)
}

pub fn get_known_issue(conn: &Connection, id: &str) -> Result<KnownIssue, String> {
    conn.query_row(
        "SELECT id, project_id, title, description, category, subcategory, bug_type, severity,
                source_issue_id, active, created_at, updated_at FROM known_issues WHERE id=?1",
        rusqlite::params![id],
        |row| Ok(KnownIssue {
            id: row.get(0)?, project_id: row.get(1)?, title: row.get(2)?, description: row.get(3)?,
            category: row.get(4)?, subcategory: row.get(5)?, bug_type: row.get(6)?, severity: row.get(7)?,
            source_issue_id: row.get(8)?, active: row.get::<_,i32>(9)? != 0,
            created_at: row.get(10)?, updated_at: row.get(11)?,
        }),
    ).map_err(|e| format!("get known: {}", e))
}

pub fn list_known_issues(conn: &Connection, project_id: &str) -> Result<Vec<KnownIssue>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, description, category, subcategory, bug_type, severity,
                source_issue_id, active, created_at, updated_at FROM known_issues WHERE project_id=?1 AND active=1 ORDER BY created_at DESC",
    ).map_err(|e| format!("prep known list: {}", e))?;
    let rows = stmt.query_map(rusqlite::params![project_id], |row| Ok(KnownIssue {
        id: row.get(0)?, project_id: row.get(1)?, title: row.get(2)?, description: row.get(3)?,
        category: row.get(4)?, subcategory: row.get(5)?, bug_type: row.get(6)?, severity: row.get(7)?,
        source_issue_id: row.get(8)?, active: row.get::<_,i32>(9)? != 0,
        created_at: row.get(10)?, updated_at: row.get(11)?,
    })).map_err(|e| format!("query known: {}", e))?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e| format!("collect known: {}", e))
}

pub fn deactivate_known_issue(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("UPDATE known_issues SET active=0, updated_at=datetime('now') WHERE id=?1",
        rusqlite::params![id]).map_err(|e| format!("deactivate known: {}", e))?;
    Ok(())
}

pub fn update_issue_lifecycle(conn: &Connection, issue_id: &str, new_lifecycle: &str) -> Result<IssueProposal, String> {
    let issue = get_issue_proposal(conn, issue_id)?;
    let current = crate::issue_lifecycle::IssueLifecycle::from_str(&issue.lifecycle)
        .unwrap_or(crate::issue_lifecycle::IssueLifecycle::Proposed);
    let target = crate::issue_lifecycle::IssueLifecycle::from_str(new_lifecycle)
        .ok_or_else(|| format!("Invalid lifecycle: {}", new_lifecycle))?;
    if current.transition(&target).is_none() {
        return Err(format!("Invalid transition: {} -> {}", current.as_str(), target.as_str()));
    }
    conn.execute("UPDATE issue_proposals SET lifecycle=?1, review_status=?2 WHERE id=?3",
        rusqlite::params![new_lifecycle, new_lifecycle, issue_id]).map_err(|e| format!("lifecycle update: {}", e))?;
    get_issue_proposal(conn, issue_id)
}
// === PHASE11_DB ===

pub fn create_analysis_job(conn: &Connection, id: &str, project_id: &str, job_type: &str, source_id: &str) -> Result<AnalysisJob, String> {
    conn.execute(
        "INSERT INTO analysis_job_queue (id, project_id, job_type, source_id, status) VALUES (?1,?2,?3,?4,'queued')",
        rusqlite::params![id, project_id, job_type, source_id],
    ).map_err(|e| format!("create job: {}", e))?;
    get_analysis_job(conn, id)
}

pub fn get_analysis_job(conn: &Connection, id: &str) -> Result<AnalysisJob, String> {
    conn.query_row(
        "SELECT id,project_id,job_type,source_id,status,progress,error_message,retry_count,max_retries,created_at,started_at,completed_at FROM analysis_job_queue WHERE id=?1",
        rusqlite::params![id],
        |row| Ok(AnalysisJob {
            id: row.get(0)?, project_id: row.get(1)?, job_type: row.get(2)?, source_id: row.get(3)?,
            status: row.get(4)?, progress: row.get(5)?, error_message: row.get(6)?,
            retry_count: row.get(7)?, max_retries: row.get(8)?, created_at: row.get(9)?,
            started_at: row.get(10)?, completed_at: row.get(11)?,
        }),
    ).map_err(|e| format!("get job: {}", e))
}

pub fn list_analysis_jobs(conn: &Connection, project_id: &str) -> Result<Vec<AnalysisJob>, String> {
    let mut stmt = conn.prepare(
        "SELECT id,project_id,job_type,source_id,status,progress,error_message,retry_count,max_retries,created_at,started_at,completed_at FROM analysis_job_queue WHERE project_id=?1 ORDER BY created_at DESC"
    ).map_err(|e| format!("prep jobs: {}", e))?;
    let rows = stmt.query_map(rusqlite::params![project_id], |row| Ok(AnalysisJob {
        id: row.get(0)?, project_id: row.get(1)?, job_type: row.get(2)?, source_id: row.get(3)?,
        status: row.get(4)?, progress: row.get(5)?, error_message: row.get(6)?,
        retry_count: row.get(7)?, max_retries: row.get(8)?, created_at: row.get(9)?,
        started_at: row.get(10)?, completed_at: row.get(11)?,
    })).map_err(|e| format!("query jobs: {}", e))?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e| format!("collect jobs: {}", e))
}

pub fn update_job_status(conn: &Connection, id: &str, status: &str, error_msg: Option<&str>, progress: f64) -> Result<AnalysisJob, String> {
    let now = chrono::Utc::now().to_rfc3339();
    match status {
        "running" => conn.execute("UPDATE analysis_job_queue SET status=?1, started_at=?2 WHERE id=?3", rusqlite::params![status, now, id]).map_err(|e| format!("update job: {}", e))?,
        "completed" | "failed" => conn.execute("UPDATE analysis_job_queue SET status=?1, completed_at=?2, error_message=?3, progress=?4 WHERE id=?5", rusqlite::params![status, now, error_msg.unwrap_or(""), progress, id]).map_err(|e| format!("update job: {}", e))?,
        _ => conn.execute("UPDATE analysis_job_queue SET status=?1, error_message=?2, progress=?3 WHERE id=?4", rusqlite::params![status, error_msg.unwrap_or(""), progress, id]).map_err(|e| format!("update job: {}", e))?,
    };
    get_analysis_job(conn, id)
}

pub fn increment_job_retry(conn: &Connection, id: &str) -> Result<AnalysisJob, String> {
    conn.execute("UPDATE analysis_job_queue SET retry_count=retry_count+1, status='queued', error_message=NULL WHERE id=?1", rusqlite::params![id])
        .map_err(|e| format!("retry job: {}", e))?;
    get_analysis_job(conn, id)
}

// === PHASE10_DB ===

/// Import localization records from parsed CSV/JSON data.
pub fn import_localization_records(
    conn: &Connection, project_id: &str, source_file: &str,
    records: &[ParsedLocalizationRecord],
) -> Result<usize, String> {
    let mut count = 0;
    for r in records {
        let id = format!("loc-{}", uuid::Uuid::new_v4());
        conn.execute(
            "INSERT INTO localization_records (id, project_id, string_id, source_text, target_text, locale, context, source_file, source_row)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            rusqlite::params![id, project_id, r.string_id, r.source_text, r.target_text, r.locale, r.context, source_file, r.source_row],
        ).map_err(|e| format!("import loc record: {}", e))?;
        count += 1;
    }
    Ok(count)
}

pub fn list_localization_records(conn: &Connection, project_id: &str) -> Result<Vec<LocalizationRecord>, String> {
    let mut stmt = conn.prepare(
        "SELECT id,project_id,string_id,source_text,target_text,locale,context,source_file,source_row,created_at
         FROM localization_records WHERE project_id=?1 ORDER BY created_at"
    ).map_err(|e| format!("prep loc records: {}", e))?;
    let rows = stmt.query_map(rusqlite::params![project_id], |row| Ok(LocalizationRecord {
        id: row.get(0)?, project_id: row.get(1)?, string_id: row.get(2)?,
        source_text: row.get(3)?, target_text: row.get(4)?, locale: row.get(5)?,
        context: row.get(6)?, source_file: row.get(7)?, source_row: row.get(8)?, created_at: row.get(9)?,
    })).map_err(|e| format!("query loc records: {}", e))?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e| format!("collect loc records: {}", e))
}

/// Deterministic mapping between OCR text and localization records.
/// Returns candidates with confidence and method.
pub fn find_string_mapping_candidates(
    conn: &Connection, project_id: &str, ocr_text: &str, image_id: &str,
) -> Result<Vec<StringMapping>, String> {
    let records = list_localization_records(conn, project_id)?;
    let mut candidates = Vec::new();
    let ocr_norm = ocr_text.trim().to_lowercase();
    for rec in &records {
        let target_norm = rec.target_text.trim().to_lowercase();
        let mut conf = 0.0f64;
        let mut method = "";
        if ocr_norm == target_norm {
            conf = 1.0; method = "exact_match";
        } else if !ocr_norm.is_empty() && (target_norm.contains(&ocr_norm) || ocr_norm.contains(&target_norm)) {
            conf = 0.8; method = "substring_match";
        }
        if conf > 0.0 {
            candidates.push(StringMapping {
                id: format!("sm-{}", uuid::Uuid::new_v4()),
                project_id: project_id.to_string(), image_id: image_id.to_string(),
                ocr_text: ocr_text.to_string(), loc_record_id: rec.id.clone(),
                string_id: rec.string_id.clone(), mapping_method: method.to_string(),
                confidence: conf, confirmed: false, created_at: String::new(),
            });
        }
    }
    Ok(candidates)
}

pub fn insert_string_mapping(conn: &Connection, mapping: &StringMapping) -> Result<(), String> {
    conn.execute(
        "INSERT INTO string_mappings (id,project_id,image_id,ocr_text,loc_record_id,string_id,mapping_method,confidence,confirmed)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        rusqlite::params![mapping.id, mapping.project_id, mapping.image_id, mapping.ocr_text,
            mapping.loc_record_id, mapping.string_id, mapping.mapping_method, mapping.confidence,
            mapping.confirmed as i32],
    ).map_err(|e| format!("insert mapping: {}", e))?;
    Ok(())
}

pub fn confirm_string_mapping(conn: &Connection, mapping_id: &str) -> Result<(), String> {
    conn.execute("UPDATE string_mappings SET confirmed=1 WHERE id=?1", rusqlite::params![mapping_id])
        .map_err(|e| format!("confirm mapping: {}", e))?;
    Ok(())
}

pub fn list_string_mappings(conn: &Connection, project_id: &str) -> Result<Vec<StringMapping>, String> {
    let mut stmt = conn.prepare(
        "SELECT id,project_id,image_id,ocr_text,loc_record_id,string_id,mapping_method,confidence,confirmed,created_at
         FROM string_mappings WHERE project_id=?1 ORDER BY created_at DESC"
    ).map_err(|e| format!("prep mappings: {}", e))?;
    let rows = stmt.query_map(rusqlite::params![project_id], |row| Ok(StringMapping {
        id: row.get(0)?, project_id: row.get(1)?, image_id: row.get(2)?, ocr_text: row.get(3)?,
        loc_record_id: row.get(4)?, string_id: row.get(5)?, mapping_method: row.get(6)?,
        confidence: row.get(7)?, confirmed: row.get::<_,i32>(8)? != 0, created_at: row.get(9)?,
    })).map_err(|e| format!("query mappings: {}", e))?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e| format!("collect mappings: {}", e))
}

/// Parsed localization record for import.
#[derive(Debug, Clone)]
pub struct ParsedLocalizationRecord {
    pub string_id: String,
    pub source_text: String,
    pub target_text: String,
    pub locale: String,
    pub context: String,
    pub source_row: i32,
}

pub fn parse_csv_localization(data: &str) -> Result<Vec<ParsedLocalizationRecord>, String> {
    let mut records = Vec::new();
    let mut reader = csv::ReaderBuilder::new().has_headers(true).flexible(true).from_reader(data.as_bytes());
    for (i, result) in reader.records().enumerate() {
        let row = result.map_err(|e| format!("CSV parse error at row {}: {}", i, e))?;
        records.push(ParsedLocalizationRecord {
            string_id: row.get(0).unwrap_or("").to_string(),
            source_text: row.get(1).unwrap_or("").to_string(),
            target_text: row.get(2).unwrap_or("").to_string(),
            locale: row.get(3).unwrap_or("").to_string(),
            context: row.get(4).unwrap_or("").to_string(),
            source_row: i as i32 + 2,
        });
    }
    Ok(records)
}

pub fn parse_json_localization(data: &str) -> Result<Vec<ParsedLocalizationRecord>, String> {
    let parsed: serde_json::Value = serde_json::from_str(data).map_err(|e| format!("JSON parse error: {}", e))?;
    let arr = parsed.as_array().ok_or("JSON must be an array of objects")?;
    let mut records = Vec::new();
    for (i, item) in arr.iter().enumerate() {
        let obj = item.as_object().ok_or(format!("Item {} is not an object", i))?;
        records.push(ParsedLocalizationRecord {
            string_id: obj.get("string_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            source_text: obj.get("source_text").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            target_text: obj.get("target_text").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            locale: obj.get("locale").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            context: obj.get("context").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            source_row: i as i32 + 1,
        });
    }
    Ok(records)
}

// === PHASE9_DB ===
pub fn create_build(conn: &Connection, id: &str, project_id: &str, version_label: &str, platform: &str, environment: &str, notes: &str) -> Result<Build, String> {
    conn.execute("INSERT INTO builds (id,project_id,version_label,platform,environment,notes) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![id,project_id,version_label,platform,environment,notes])
        .map_err(|e| format!("create build: {}", e))?;
    get_build(conn, id)
}

pub fn get_build(conn: &Connection, id: &str) -> Result<Build, String> {
    conn.query_row("SELECT id,project_id,version_label,platform,environment,notes,created_at FROM builds WHERE id=?1",
        rusqlite::params![id],
        |row| Ok(Build { id: row.get(0)?, project_id: row.get(1)?, version_label: row.get(2)?, platform: row.get(3)?, environment: row.get(4)?, notes: row.get(5)?, created_at: row.get(6)? }),
    ).map_err(|e| format!("get build: {}", e))
}

pub fn list_builds(conn: &Connection, project_id: &str) -> Result<Vec<Build>, String> {
    let mut stmt = conn.prepare("SELECT id,project_id,version_label,platform,environment,notes,created_at FROM builds WHERE project_id=?1 ORDER BY created_at DESC")
        .map_err(|e| format!("prep builds: {}", e))?;
    let rows = stmt.query_map(rusqlite::params![project_id], |row| Ok(Build {
        id: row.get(0)?, project_id: row.get(1)?, version_label: row.get(2)?, platform: row.get(3)?,
        environment: row.get(4)?, notes: row.get(5)?, created_at: row.get(6)?,
    })).map_err(|e| format!("query builds: {}", e))?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e| format!("collect builds: {}", e))
}

pub fn assign_image_build(conn: &Connection, image_id: &str, build_id: &str) -> Result<(), String> {
    conn.execute("UPDATE project_images SET build_id=?1 WHERE id=?2", rusqlite::params![build_id, image_id])
        .map_err(|e| format!("assign image build: {}", e))?;
    Ok(())
}

pub fn assign_video_build(conn: &Connection, video_id: &str, build_id: &str) -> Result<(), String> {
    conn.execute("UPDATE project_videos SET build_id=?1 WHERE id=?2", rusqlite::params![build_id, video_id])
        .map_err(|e| format!("assign video build: {}", e))?;
    Ok(())
}

pub fn determine_regression(conn: &Connection, project_id: &str, current_build_id: &str, fingerprint: &str, current_issue_id: &str) -> Result<RegressionStatus, String> {
    let builds = list_builds(conn, project_id)?;
    let current_idx = match builds.iter().position(|b| b.id == current_build_id) { Some(i) => i, None => return Ok(RegressionStatus::New) };
    let all_issues = list_issue_proposals(conn, project_id, None)?;
    for issue in &all_issues {
        if issue.id == current_issue_id { continue; }
        let fp = crate::issue_lifecycle::compute_issue_fingerprint(&issue.detected_text, "", &issue.issue_category, &issue.issue_subcategory, &issue.bug_type);
        if fp == fingerprint {
            if issue.lifecycle == "approved" || issue.lifecycle == "resolved" {
                return Ok(RegressionStatus::Recurring);
            }
        }
    }
    if builds.len() > current_idx + 1 { Ok(RegressionStatus::PotentiallyFixed) } else { Ok(RegressionStatus::New) }
}
// === PHASE14_DB ===

pub fn create_tracker_config(conn: &Connection, id: &str, project_id: &str, connector_type: &str, display_name: &str, base_url: &str, project_key: &str, field_mapping_json: &str) -> Result<(), String> {
    conn.execute("INSERT INTO tracker_configs (id,project_id,connector_type,display_name,base_url,project_key,field_mapping_json,enabled) VALUES (?1,?2,?3,?4,?5,?6,?7,1)",
        rusqlite::params![id,project_id,connector_type,display_name,base_url,project_key,field_mapping_json])
        .map_err(|e| format!("create tracker config: {}", e))?;
    Ok(())
}

pub fn list_tracker_configs(conn: &Connection, project_id: &str) -> Result<Vec<crate::tracker_connector::TrackerConfig>, String> {
    let mut stmt = conn.prepare("SELECT id,project_id,connector_type,display_name,base_url,project_key,field_mapping_json,enabled,created_at FROM tracker_configs WHERE project_id=?1")
        .map_err(|e| format!("prep tracker: {}", e))?;
    let rows = stmt.query_map(rusqlite::params![project_id], |row| Ok(crate::tracker_connector::TrackerConfig {
        id: row.get(0)?, project_id: row.get(1)?, connector_type: row.get(2)?, display_name: row.get(3)?,
        base_url: row.get(4)?, project_key: row.get(5)?, field_mapping_json: row.get(6)?,
        enabled: row.get::<_,i32>(7)? != 0, created_at: row.get(8)?,
    })).map_err(|e| format!("query tracker: {}", e))?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e| format!("collect tracker: {}", e))
}

pub fn insert_submission_record(conn: &Connection, record: &crate::tracker_connector::SubmissionRecord) -> Result<(), String> {
    conn.execute("INSERT INTO submission_records (id,qaivra_issue_id,tracker_config_id,external_id,payload_fingerprint,status) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![record.id, record.qaivra_issue_id, record.tracker_config_id, record.external_id, record.payload_fingerprint, record.status])
        .map_err(|e| format!("insert submission: {}", e))?;
    Ok(())
}

pub fn list_submission_records(conn: &Connection, project_id: &str) -> Result<Vec<crate::tracker_connector::SubmissionRecord>, String> {
    let mut stmt = conn.prepare("SELECT s.id,s.qaivra_issue_id,s.tracker_config_id,s.external_id,s.payload_fingerprint,s.status,s.submitted_at FROM submission_records s JOIN tracker_configs t ON s.tracker_config_id=t.id WHERE t.project_id=?1")
        .map_err(|e| format!("prep submissions: {}", e))?;
    let rows = stmt.query_map(rusqlite::params![project_id], |row| Ok(crate::tracker_connector::SubmissionRecord {
        id: row.get(0)?, qaivra_issue_id: row.get(1)?, tracker_config_id: row.get(2)?,
        external_id: row.get(3)?, payload_fingerprint: row.get(4)?, status: row.get(5)?, submitted_at: row.get(6)?,
    })).map_err(|e| format!("query submissions: {}", e))?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e| format!("collect submissions: {}", e))
}

// === PHASE15_DB ===

pub fn record_audit_event(conn: &Connection, project_id: &str, event_type: &str, entity_type: &str, entity_id: &str, metadata_json: &str) -> Result<(), String> {
    let id = format!("audit-{}", uuid::Uuid::new_v4());
    conn.execute("INSERT INTO audit_events (id,project_id,event_type,entity_type,entity_id,safe_metadata_json) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![id,project_id,event_type,entity_type,entity_id,metadata_json])
        .map_err(|e| format!("record audit: {}", e))?;
    Ok(())
}

pub fn list_audit_events(conn: &Connection, project_id: &str) -> Result<Vec<AuditEvent>, String> {
    let mut stmt = conn.prepare("SELECT id,project_id,event_type,entity_type,entity_id,safe_metadata_json,created_at FROM audit_events WHERE project_id=?1 ORDER BY created_at DESC LIMIT 500")
        .map_err(|e| format!("prep audit: {}", e))?;
    let rows = stmt.query_map(rusqlite::params![project_id], |row| Ok(AuditEvent {
        id: row.get(0)?, project_id: row.get(1)?, event_type: row.get(2)?,
        entity_type: row.get(3)?, entity_id: row.get(4)?, safe_metadata_json: row.get(5)?, created_at: row.get(6)?,
    })).map_err(|e| format!("query audit: {}", e))?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e| format!("collect audit: {}", e))
}
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_and_list_projects() {
        let conn = test_db();
        let project = create_project(&conn, "test-id", "Test Project", "Indonesian", "RPG", "PC").unwrap();
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.target_language, "Indonesian");

        let projects = get_projects(&conn).unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].id, "test-id");
    }

    #[test]
    fn test_upload_project_image_leaves_video_frame_id_empty() {
        let conn = test_db();
        create_project(&conn, "p1", "Project 1", "Japanese", "Action", "Mobile").unwrap();
        let img = upload_project_image(
            &conn, "p1", "shot.png", "C:\\tmp\\shot.png", "image/png", 2048, 1920, 1080, "abc123",
        )
        .unwrap();
        // A non-video image upload must not bind a stray placeholder into
        // video_frame_id; it should remain empty so it is distinct from a real
        // video-frame link.
        assert_eq!(img.video_frame_id, "");
        assert_eq!(img.project_id, "p1");
        assert_eq!(img.filename, "shot.png");
        assert_eq!(img.width, 1920);
        assert_eq!(img.height, 1080);

        let fetched = get_project_image(&conn, &img.id).unwrap();
        assert_eq!(fetched.video_frame_id, "");
        let listed = list_project_images(&conn, "p1").unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].video_frame_id, "");
    }
    #[test]
    fn test_list_and_get_video_frame_reads_frame_image_id() {
        let conn = test_db();
        create_project(&conn, "p1", "Project 1", "Japanese", "Action", "Mobile").unwrap();
        let vpath = std::env::temp_dir().join(format!("qaivra_vid_{}.mp4", chrono::Utc::now().timestamp()));
        std::fs::write(&vpath, b"videodata").unwrap();
        let v = vpath.to_str().unwrap();
        let video = insert_project_video(&conn, "p1", "clip.mp4", v, 1234, 10.0, 1920, 1080, 30.0, "video/mp4").unwrap();
        let ext = insert_video_extraction(&conn, "p1", &video.id, 2.0, "v1").unwrap();
        let fpath = std::env::temp_dir().join(format!("qaivra_fr_{}.png", chrono::Utc::now().timestamp()));
        std::fs::write(&fpath, b"framedata").unwrap();
        let p = fpath.to_str().unwrap();
        let frame = insert_video_frame(&conn, "p1", &video.id, &ext.id, 0, 0, "0.00s", p, 1920, 1080).unwrap();
        let reg = register_frame_image(&conn, "p1", &frame.id, "frame_000.png", p, 1920, 1080, "h1").unwrap();
        // Regression: list/get_video_frames must surface frame_image_id (migration 22b).
        let fetched = get_video_frame(&conn, &frame.id).unwrap();
        assert_eq!(fetched.frame_image_id, reg.id);
        let frames = list_video_frames(&conn, &ext.id).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].frame_image_id, reg.id);
        let _ = std::fs::remove_file(&vpath);
        let _ = std::fs::remove_file(&fpath);
    }




    #[test]
    fn test_get_project() {
        let conn = test_db();
        create_project(&conn, "p1", "Project 1", "Japanese", "Action", "Mobile").unwrap();
        let project = get_project(&conn, "p1").unwrap();
        assert_eq!(project.name, "Project 1");
    }

    #[test]
    fn test_delete_project() {
        let conn = test_db();
        create_project(&conn, "p1", "To Delete", "", "", "").unwrap();
        delete_project(&conn, "p1").unwrap();
        let projects = get_projects(&conn).unwrap();
        assert_eq!(projects.len(), 0);
    }

    #[test]
    fn test_rename_project() {
        let conn = test_db();
        create_project(&conn, "p1", "Old Name", "", "", "").unwrap();
        let renamed = rename_project(&conn, "p1", "New Name").unwrap();
        assert_eq!(renamed.name, "New Name");
    }

    #[test]
    fn test_get_nonexistent_project() {
        let conn = test_db();
        let result = get_project(&conn, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent_project() {
        let conn = test_db();
        let result = delete_project(&conn, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_projects_ordering() {
        let conn = test_db();
        create_project(&conn, "p1", "First", "", "", "").unwrap();
        create_project(&conn, "p2", "Second", "", "", "").unwrap();
        let projects = get_projects(&conn).unwrap();
        assert_eq!(projects.len(), 2);
    }

    // ---- Phase 2: Severity Rules Tests ----

    use super::{ParsedRule, ParseResult, ParseMetadata, import_severity_rules, list_severity_rules};

    fn make_parse_result(rules: Vec<ParsedRule>, warnings: Vec<String>) -> ParseResult {
        ParseResult {
            success: true, error: None, rules, warnings,
            metadata: Some(ParseMetadata { filename: "test.xlsx".to_string(), sheet_name: "BUG info".to_string(), total_rules: 0 }),
        }
    }

    fn make_rule(row: i32, subcat: &str, severity: &str) -> ParsedRule {
        ParsedRule {
            source_row: row, main_category_raw: "Test Category".to_string(),
            subcategory_raw: subcat.to_string(), canonical_bug_type: subcat.to_string(),
            severity_exact: severity.to_string(),
            severity_code: severity.split_whitespace().next().unwrap_or("").to_string(),
            severity_name: severity.split_whitespace().skip(1).collect::<Vec<_>>().join(" "),
            description_raw: format!("Desc row {}", row),
            source_sheet: "BUG info".to_string(), source_filename: "test.xlsx".to_string(),
        }
    }

    #[test]
    fn test_import_severity_rules_basic() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        let rules = vec![make_rule(3, "SubA", "P1 Critical ä¸¥é‡"), make_rule(4, "SubA", "P2 Moderate ä¸€èˆ¬")];
        let result = import_severity_rules(&conn, "p1", &make_parse_result(rules, vec![])).unwrap();
        assert_eq!(result.rules_count, 2);
        let loaded = list_severity_rules(&conn, "p1").unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].severity_exact, "P1 Critical ä¸¥é‡");
    }

    #[test]
    fn test_severity_rules_project_isolation() {
        let conn = test_db();
        create_project(&conn, "p1", "A", "", "", "").unwrap();
        create_project(&conn, "p2", "B", "", "", "").unwrap();
        import_severity_rules(&conn, "p1", &make_parse_result(vec![make_rule(3, "CatA", "P1 Critical ä¸¥é‡")], vec![])).unwrap();
        import_severity_rules(&conn, "p2", &make_parse_result(vec![make_rule(3, "CatB", "P0 Extreme è‡´å‘½")], vec![])).unwrap();
        assert_eq!(list_severity_rules(&conn, "p1").unwrap()[0].severity_exact, "P1 Critical ä¸¥é‡");
        assert_eq!(list_severity_rules(&conn, "p2").unwrap()[0].severity_exact, "P0 Extreme è‡´å‘½");
    }

    #[test]
    fn test_replace_deactivates_old_rules() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        import_severity_rules(&conn, "p1", &make_parse_result(vec![make_rule(3, "Old", "P1 Critical ä¸¥é‡"), make_rule(4, "Old", "P2 Moderate ä¸€èˆ¬")], vec![])).unwrap();
        assert_eq!(list_severity_rules(&conn, "p1").unwrap().len(), 2);
        import_severity_rules(&conn, "p1", &make_parse_result(vec![make_rule(3, "New", "P0 Extreme è‡´å‘½")], vec![])).unwrap();
        let active = list_severity_rules(&conn, "p1").unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].canonical_bug_type, "New");
    }

    #[test]
    fn test_replace_does_not_affect_other_projects() {
        let conn = test_db();
        create_project(&conn, "p1", "A", "", "", "").unwrap();
        create_project(&conn, "p2", "B", "", "", "").unwrap();
        import_severity_rules(&conn, "p1", &make_parse_result(vec![make_rule(3, "CatA", "P1 Critical ä¸¥é‡")], vec![])).unwrap();
        import_severity_rules(&conn, "p2", &make_parse_result(vec![make_rule(3, "CatB", "P0 Extreme è‡´å‘½")], vec![])).unwrap();
        // Replace p1
        import_severity_rules(&conn, "p1", &make_parse_result(vec![make_rule(3, "NewA", "P2 Moderate ä¸€èˆ¬")], vec![])).unwrap();
        // p2 unchanged
        let p2 = list_severity_rules(&conn, "p2").unwrap();
        assert_eq!(p2.len(), 1);
        assert_eq!(p2[0].canonical_bug_type, "CatB");
    }

    #[test]
    fn test_import_preserves_unicode() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        import_severity_rules(&conn, "p1", &make_parse_result(vec![
            make_rule(3, "å˜é‡æˆ–å•ä½é”™è¯¯ Variable/Format Error", "P1 Critical ä¸¥é‡"),
            make_rule(5, "è¯¯è¯‘ Mistranslation", "P0 Extreme è‡´å‘½"),
        ], vec![])).unwrap();
        let loaded = list_severity_rules(&conn, "p1").unwrap();
        assert_eq!(loaded[0].subcategory_raw, "å˜é‡æˆ–å•ä½é”™è¯¯ Variable/Format Error");
        assert_eq!(loaded[1].severity_exact, "P0 Extreme è‡´å‘½");
    }

    #[test]
    fn test_failed_parse_no_import() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        let failed = ParseResult { success: false, error: Some("Missing sheet".to_string()), rules: vec![], warnings: vec![], metadata: None };
        assert!(import_severity_rules(&conn, "p1", &failed).is_err());
        assert_eq!(list_severity_rules(&conn, "p1").unwrap().len(), 0);
    }

    #[test]
    fn test_import_warnings_preserved() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        let warnings = vec!["Row 5: empty severity".to_string()];
        let result = import_severity_rules(&conn, "p1", &make_parse_result(vec![make_rule(3, "Cat", "P1 Critical ä¸¥é‡")], warnings)).unwrap();
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("Row 5"));
    }

    // ---- Phase 3: Precedent Reports Tests ----

    fn make_precedent_row(heading: &str, content: &str, issue: &str) -> ParsedPrecedentRow {
        ParsedPrecedentRow {
            sheet_name: "Report".to_string(), page_number: 0,
            row_start: 1, row_end: 2, heading: heading.to_string(),
            content: content.to_string(), issue_type: issue.to_string(),
            severity_exact: "P1 Critical".to_string(),
            terminology_used: "test term".to_string(), chunk_order: 0,
        }
    }

    #[test]
    fn test_import_precedent_explicit_classification() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        let rows = vec![make_precedent_row("H1", "Content", "Mistranslation")];
        let result = import_precedent_report(&conn, "p1", "report.xlsx", "xlsx", "PRECEDENT", &rows, &[]).unwrap();
        assert_eq!(result.chunk_count, 1);
        assert_eq!(result.classification, "PRECEDENT");
        let reports = list_precedent_reports(&conn, "p1").unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].classification, "PRECEDENT");
    }

    #[test]
    fn test_import_precedent_unclassified_rejected() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        let rows = vec![make_precedent_row("H1", "Content", "Error")];
        let result = import_precedent_report(&conn, "p1", "report.xlsx", "xlsx", "unclassified", &rows, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("explicit"));
    }

    #[test]
    fn test_precedent_project_isolation() {
        let conn = test_db();
        create_project(&conn, "p1", "A", "", "", "").unwrap();
        create_project(&conn, "p2", "B", "", "", "").unwrap();
        let rows1 = vec![make_precedent_row("H1", "Project A report", "Error")];
        let rows2 = vec![make_precedent_row("H2", "Project B report", "Bug")];
        import_precedent_report(&conn, "p1", "r1.xlsx", "xlsx", "PRECEDENT", &rows1, &[]).unwrap();
        import_precedent_report(&conn, "p2", "r2.xlsx", "xlsx", "PRECEDENT", &rows2, &[]).unwrap();
        let p1_reports = list_precedent_reports(&conn, "p1").unwrap();
        let p2_reports = list_precedent_reports(&conn, "p2").unwrap();
        assert_eq!(p1_reports.len(), 1);
        assert_eq!(p2_reports.len(), 1);
        assert!(p1_reports[0].content.contains("Project A"));
        assert!(p2_reports[0].content.contains("Project B"));
        // Cross-project search isolation
        let p1_search = search_precedent_reports(&conn, "p1", "Project B").unwrap();
        assert_eq!(p1_search.len(), 0);
    }

    #[test]
    fn test_precedent_search() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        let rows = vec![
            make_precedent_row("H1", "Mistranslation in quest dialog", "Mistranslation"),
            make_precedent_row("H2", "UI overlap issue", "Overlap"),
        ];
        import_precedent_report(&conn, "p1", "r.xlsx", "xlsx", "PRECEDENT", &rows, &[]).unwrap();
        let results = search_precedent_reports(&conn, "p1", "Mistranslation").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("quest dialog"));
    }

    #[test]
    fn test_precedent_does_not_override_severity_rules() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        import_severity_rules(&conn, "p1", &make_parse_result(vec![
            make_rule(3, "Mistranslation", "P1 Critical ä¸¥é‡"),
        ], vec![])).unwrap();
        let rows = vec![make_precedent_row("H1", "Mistranslation", "Mistranslation")];
        import_precedent_report(&conn, "p1", "old_report.xlsx", "xlsx", "PRECEDENT", &rows, &[]).unwrap();
        // Severity rules are authoritative
        let rules = list_severity_rules(&conn, "p1").unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].severity_exact, "P1 Critical ä¸¥é‡");
        // Precedent is independently retrievable but cannot override
        let precedents = search_precedent_reports(&conn, "p1", "Mistranslation").unwrap();
        assert_eq!(precedents.len(), 1);
    }
    #[test]
    fn test_glossary_conflict_persisted_on_duplicate() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        let entries = vec![
            ParsedGlossaryEntry { source_term: "Hello".to_string(), target_term: "Halo".to_string(),
                status: "approved".to_string(), do_not_translate: false, banned: false,
                notes: "".to_string(), source_row: 2 },
            ParsedGlossaryEntry { source_term: "hello".to_string(), target_term: "Hai".to_string(),
                status: "approved".to_string(), do_not_translate: false, banned: false,
                notes: "".to_string(), source_row: 3 },
        ];
        import_glossary_entries(&conn, "p1", "g.xlsx", "xlsx", &entries, &[]).unwrap();
        let conflicts = get_glossary_conflicts(&conn, "p1").unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].resolution, "unresolved");
    }

    #[test]
    fn test_glossary_conflict_resolution() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        let entries = vec![
            ParsedGlossaryEntry { source_term: "Hello".to_string(), target_term: "Halo".to_string(),
                status: "approved".to_string(), do_not_translate: false, banned: false,
                notes: "".to_string(), source_row: 2 },
            ParsedGlossaryEntry { source_term: "hello".to_string(), target_term: "Hai".to_string(),
                status: "approved".to_string(), do_not_translate: false, banned: false,
                notes: "".to_string(), source_row: 3 },
        ];
        import_glossary_entries(&conn, "p1", "g.xlsx", "xlsx", &entries, &[]).unwrap();
        let conflicts = get_glossary_conflicts(&conn, "p1").unwrap();
        assert_eq!(conflicts.len(), 1);
        resolve_glossary_conflict(&conn, &conflicts[0].id, "chosen-entry-id").unwrap();
        let resolved = get_glossary_conflicts(&conn, "p1").unwrap();
        assert_eq!(resolved[0].resolution, "resolved");
        assert_eq!(count_unresolved_conflicts(&conn, "p1").unwrap(), 0);
    }

    #[test]
    fn test_precedent_approved_classification_accepted() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        let rows = vec![make_precedent_row("H1", "Content", "Error")];
        let result = import_precedent_report(&conn, "p1", "r.xlsx", "xlsx", "APPROVED", &rows, &[]).unwrap();
        assert_eq!(result.classification, "APPROVED");
    }

    #[test]
    fn test_ai_provider_config_save_and_get() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        let config = save_ai_provider_config(
            &conn, "p1", "ollama", "http://localhost:11434",
            "llava:7b", r#"{"vision":true}"#, "healthy",
            0.0, 4096, 120, "", "Ollama",
        ).unwrap();
        assert_eq!(config.provider, "ollama");
        assert_eq!(config.base_url, "http://localhost:11434");
        assert_eq!(config.selected_model, "llava:7b");
        assert_eq!(config.health_status, "healthy");
    }

    #[test]
    fn test_ai_provider_config_update() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        save_ai_provider_config(&conn, "p1", "ollama", "http://localhost:11434", "", "{}", "unknown", 0.0, 4096, 120, "", "Ollama").unwrap();
        save_ai_provider_config(&conn, "p1", "ollama", "http://localhost:11434", "llava:7b", r#"{"vision":true}"#, "healthy", 0.0, 4096, 120, "", "Ollama").unwrap();
        let config = get_ai_provider_config(&conn, "p1").unwrap();
        assert_eq!(config.selected_model, "llava:7b");
        assert_eq!(config.health_status, "healthy");
    }

    #[test]
    fn test_update_ai_health() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        save_ai_provider_config(&conn, "p1", "ollama", "http://localhost:11434", "", "{}", "unknown", 0.0, 4096, 120, "", "Ollama").unwrap();
        update_ai_health(&conn, "p1", "healthy").unwrap();
        let config = get_ai_provider_config(&conn, "p1").unwrap();
        assert_eq!(config.health_status, "healthy");
        assert!(config.last_health_check.is_some());
    }

    #[test]
    fn test_store_ocr_regions() {
        let conn = test_db();
        create_project(&conn, "p1", "Test", "", "", "").unwrap();
        let regions = vec![
            OcrTextRegion { text: "BERAKHIR".to_string(), confidence: 0.95,
                bbox_x: 0.1, bbox_y: 0.2, bbox_width: 0.3, bbox_height: 0.1 },
            OcrTextRegion { text: "Mulai".to_string(), confidence: 0.87,
                bbox_x: 0.5, bbox_y: 0.6, bbox_width: 0.2, bbox_height: 0.05 },
        ];
        let ids = store_ocr_regions(&conn, "p1", "img1", &regions).unwrap();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_validate_candidate_valid() {
        let bug_types = vec!["Mistranslation".to_string(), "Grammar".to_string()];
        let candidate = CandidateIssueInput {
            observed_text_exact: "Helo Dunia".to_string(),
            defect_summary: "Misspelling in greeting".to_string(),
            candidate_bug_type: "Grammar".to_string(),
            defect_condition: "Spelling error".to_string(),
            suggested_text: "Hello Dunia".to_string(),
            confidence: 0.85,
            review_state_hint: "confirmed".to_string(),
            affected_region_ids: vec!["r1".to_string()],
            context_evidence: vec![],
            glossary_dependencies: vec![],
        };
        let result = validate_candidate_issue(&candidate, &bug_types);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_candidate_invalid_bug_type() {
        let bug_types = vec!["Mistranslation".to_string()];
        let candidate = CandidateIssueInput {
            observed_text_exact: "Some text".to_string(),
            defect_summary: "Issue found".to_string(),
            candidate_bug_type: "NonExistentType".to_string(),
            defect_condition: "".to_string(),
            suggested_text: "Fixed text".to_string(),
            confidence: 0.8,
            review_state_hint: "confirmed".to_string(),
            affected_region_ids: vec![],
            context_evidence: vec![],
            glossary_dependencies: vec![],
        };
        let result = validate_candidate_issue(&candidate, &bug_types);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("not found in active severity rules")));
    }

    #[test]
    fn test_validate_candidate_empty_observed_text() {
        let candidate = CandidateIssueInput {
            observed_text_exact: "".to_string(),
            defect_summary: "Issue".to_string(),
            candidate_bug_type: "".to_string(),
            defect_condition: "".to_string(),
            suggested_text: "Fix".to_string(),
            confidence: 0.5,
            review_state_hint: "".to_string(),
            affected_region_ids: vec![],
            context_evidence: vec![],
            glossary_dependencies: vec![],
        };
        let result = validate_candidate_issue(&candidate, &[]);
        assert!(!result.valid);
    }

    #[test]
    fn test_validate_candidate_low_confidence_warning() {
        let candidate = CandidateIssueInput {
            observed_text_exact: "Text".to_string(),
            defect_summary: "Issue".to_string(),
            candidate_bug_type: "".to_string(),
            defect_condition: "".to_string(),
            suggested_text: "Fix".to_string(),
            confidence: 0.15,
            review_state_hint: "".to_string(),
            affected_region_ids: vec![],
            context_evidence: vec![],
            glossary_dependencies: vec![],
        };
        let result = validate_candidate_issue(&candidate, &[]);
        assert!(result.valid);
        assert!(!result.warnings.is_empty());
    }
}


// === SECURITY_GATE_TESTS ===
#[cfg(test)]
mod security_gate_tests {
    use super::*;
    use std::io::Write;

    fn write_temp(name: &str, bytes: &[u8]) -> String {
        let p = std::env::temp_dir().join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(bytes).unwrap();
        p.to_string_lossy().to_string()
    }

    const PNG_MAGIC: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52,
    ];

    #[test]
    fn test_validate_accepts_real_png() {
        let p = write_temp(&format!("qaivra_sec_{}_ok.png", std::process::id()), PNG_MAGIC);
        let v = validate_image_upload(&p).expect("valid png should pass validation");
        assert_eq!(v.mime_type, "image/png");
        assert!(v.file_size > 0);
        assert!(v.canonical_path.ends_with(".png"));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn test_validate_rejects_bad_extension() {
        let p = write_temp(&format!("qaivra_sec_{}_bad.txt", std::process::id()), b"hello world");
        assert!(validate_image_upload(&p).is_err(), "non-image extension must be rejected");
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn test_validate_rejects_wrong_magic_for_png_ext() {
        let p = write_temp(
            &format!("qaivra_sec_{}_fake.png", std::process::id()),
            b"this is clearly not a png file",
        );
        assert!(
            validate_image_upload(&p).is_err(),
            "extension-only validation must NOT be trusted"
        );
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn test_validate_rejects_directory() {
        let dir = std::env::temp_dir().join(format!("qaivra_sec_dir_{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        assert!(validate_image_upload(&dir.to_string_lossy()).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_is_safe_id_segment() {
        assert!(is_safe_id_segment("abc-123_Xy"));
        assert!(!is_safe_id_segment(""));
        assert!(!is_safe_id_segment(".."));
        assert!(!is_safe_id_segment("a/../b"));
        assert!(!is_safe_id_segment("a\\b"));
        assert!(!is_safe_id_segment(&"x".repeat(200)));
    }
}




/// Duplicate relationship between two issue proposals (Phase 8).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DuplicateRelationship {
    pub id: String,
    pub project_id: String,
    pub source_issue_id: String,
    pub duplicate_issue_id: String,
    pub confidence: f64,
    pub match_signals: String,
    pub confirmed: bool,
    pub confirmed_by: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Project-scoped known issue (Phase 8).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnownIssue {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub subcategory: String,
    pub bug_type: String,
    pub severity: String,
    pub source_issue_id: String,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Build/version within a project (Phase 9).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Build {
    pub id: String,
    pub project_id: String,
    pub version_label: String,
    pub platform: String,
    pub environment: String,
    pub notes: String,
    pub created_at: String,
}

/// Regression status for an issue across builds (Phase 9).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RegressionStatus {
    New,
    Recurring,
    PotentiallyFixed,
    Fixed,
    Reopened,
}

impl RegressionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Recurring => "recurring",
            Self::PotentiallyFixed => "potentially_fixed",
            Self::Fixed => "fixed",
            Self::Reopened => "reopened",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "new" => Some(Self::New),
            "recurring" => Some(Self::Recurring),
            "potentially_fixed" => Some(Self::PotentiallyFixed),
            "fixed" => Some(Self::Fixed),
            "reopened" => Some(Self::Reopened),
            _ => None,
        }
    }
}
/// Localization record from imported data (Phase 10).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalizationRecord {
    pub id: String,
    pub project_id: String,
    pub string_id: String,
    pub source_text: String,
    pub target_text: String,
    pub locale: String,
    pub context: String,
    pub source_file: String,
    pub source_row: i32,
    pub created_at: String,
}

/// String mapping between OCR text and localization record (Phase 10).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StringMapping {
    pub id: String,
    pub project_id: String,
    pub image_id: String,
    pub ocr_text: String,
    pub loc_record_id: String,
    pub string_id: String,
    pub mapping_method: String,
    pub confidence: f64,
    pub confirmed: bool,
    pub created_at: String,
}
/// Analysis job in the queue (Phase 11).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisJob {
    pub id: String,
    pub project_id: String,
    pub job_type: String,
    pub source_id: String,
    pub status: String,
    pub progress: f64,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}
/// Audit event (Phase 15).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEvent {
    pub id: String, pub project_id: String, pub event_type: String,
    pub entity_type: String, pub entity_id: String, pub safe_metadata_json: String, pub created_at: String,
}
