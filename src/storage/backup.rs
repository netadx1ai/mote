use std::fs::{self, File};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

const MAX_RESTORE_BYTES: u64 = 1_073_741_824; // 1 GB
const MAX_RESTORE_ENTRIES: usize = 50_000;

/// Create a zip backup of the entire workspace directory.
pub fn create_backup(workspace_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let workspace_name = workspace_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "workspace".to_string());
    let filename = format!("mote-backup-{workspace_name}-{timestamp}.zip");
    let output_path = output_dir.join(&filename);

    let file = File::create(&output_path).map_err(|e| format!("Failed to create backup file: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    add_dir_to_zip(&mut zip, workspace_path, workspace_path, &options)?;

    zip.finish().map_err(|e| format!("Failed to finalize zip: {e}"))?;
    Ok(output_path)
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<File>,
    base: &Path,
    dir: &Path,
    options: &SimpleFileOptions,
) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read dir {}: {e}", dir.display()))?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = entry.metadata().map_err(|e| e.to_string())?;

        // Skip symlinks to prevent including files outside workspace
        if metadata.file_type().is_symlink() {
            continue;
        }

        let path = entry.path();
        let relative = path.strip_prefix(base).map_err(|e| e.to_string())?;
        let name = relative.to_string_lossy().to_string();

        // Skip build artifacts, git, and zip files
        if name.starts_with("target") || name.starts_with(".git") || name.ends_with(".zip") {
            continue;
        }

        if metadata.is_dir() {
            zip.add_directory(&format!("{name}/"), *options)
                .map_err(|e| format!("Failed to add dir to zip: {e}"))?;
            add_dir_to_zip(zip, base, &path, options)?;
        } else {
            zip.start_file(&name, *options)
                .map_err(|e| format!("Failed to start file in zip: {e}"))?;
            let mut f = File::open(&path).map_err(|e| format!("Failed to open {}: {e}", path.display()))?;
            std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Restore a zip backup into a target workspace directory.
/// Validates against zip slip, zip bombs, and symlinks.
pub fn restore_backup(zip_path: &Path, workspace_path: &Path) -> Result<(), String> {
    if !zip_path.exists() {
        return Err(format!("Backup file not found: {}", zip_path.display()));
    }

    fs::create_dir_all(workspace_path).map_err(|e| e.to_string())?;
    let canonical_ws = workspace_path.canonicalize().map_err(|e| e.to_string())?;

    let file = File::open(zip_path).map_err(|e| format!("Failed to open backup: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip file: {e}"))?;

    // Guard: max entries
    if archive.len() > MAX_RESTORE_ENTRIES {
        return Err(format!("Zip has too many entries ({} > {MAX_RESTORE_ENTRIES})", archive.len()));
    }

    // Validate all paths BEFORE extracting (no partial writes on failure)
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name();

        // Reject entries with path traversal components
        if name.contains("..") {
            return Err(format!("Zip entry contains path traversal: {name}"));
        }

        let out_path = canonical_ws.join(name);
        // Verify resolved path stays within workspace
        // For dirs that don't exist yet, check the string prefix
        if !out_path.starts_with(&canonical_ws) {
            return Err(format!("Zip entry escapes workspace: {name}"));
        }
    }

    // Clear existing content (keep .git if present)
    if let Ok(entries) = fs::read_dir(workspace_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == ".git" {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                let _ = fs::remove_dir_all(&path);
            } else {
                let _ = fs::remove_file(&path);
            }
        }
    }

    // Extract with size tracking
    let mut total_bytes: u64 = 0;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let out_path = canonical_ws.join(entry.name());

        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out_file = File::create(&out_path).map_err(|e| e.to_string())?;
            let written = std::io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
            total_bytes += written;
            if total_bytes > MAX_RESTORE_BYTES {
                return Err(format!("Restore aborted: extracted data exceeds {MAX_RESTORE_BYTES} bytes (possible zip bomb)"));
            }
        }
    }

    Ok(())
}

/// Export all items as a JSON array (portable data export).
pub fn export_json(storage: &crate::storage::Storage) -> Result<String, String> {
    let mut items = storage.get_tree()?;
    storage.load_file_contents(&mut items);
    serde_json::to_string_pretty(&items).map_err(|e| e.to_string())
}

/// Import items from a JSON export. Merges (skips duplicates by ID).
pub fn import_json(storage: &crate::storage::Storage, json: &str) -> Result<usize, String> {
    let items: Vec<crate::models::Item> =
        serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {e}"))?;

    let existing = storage.get_tree()?;
    let existing_ids: std::collections::HashSet<String> = existing.iter().map(|i| i.id.clone()).collect();

    let mut imported = 0;
    for item in items {
        if existing_ids.contains(&item.id) {
            continue;
        }

        // file write goes through FileManager which validates path traversal
        if let (Some(fp), Some(content)) = (&item.file_path, &item.content) {
            storage.files.write_file(fp, content).map_err(|e| e.to_string())?;
        }

        storage.db.insert_item(&item).map_err(|e| e.to_string())?;

        if let Some(ref content) = item.content {
            let _ = storage.db.index_item(&item.id, item.item_type.as_str(), &item.title, content);
        }

        imported += 1;
    }

    Ok(imported)
}
