mod db;
mod files;
pub mod backup;

pub use db::Database;
pub use files::FileManager;

use crate::models::*;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Position for reorder operations.
#[derive(Clone, Copy, PartialEq)]
pub enum ReorderPos {
    Before,
    After,
    Into,
}

/// Returns the data directory path within a workspace.
pub fn data_dir(workspace_path: &Path) -> PathBuf {
    workspace_path.join("mote-data")
}

pub struct Storage {
    pub(crate) db: Database,
    pub(crate) files: FileManager,
    pub data_path: PathBuf,
}

impl Storage {
    pub fn new(workspace_path: &Path) -> Result<Self, String> {
        let data_path = data_dir(workspace_path);
        maybe_migrate_legacy(workspace_path, &data_path);
        std::fs::create_dir_all(&data_path).map_err(|e| e.to_string())?;
        let db = Database::new(&data_path).map_err(|e| e.to_string())?;
        let files = FileManager::new(&data_path);
        // Initialize git repo in mote-data/ if not already present
        git_init(&data_path);
        Ok(Storage { db, files, data_path })
    }

    pub fn create_item(&self, req: CreateItemRequest) -> Result<Item, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let short_id = &id[..8]; // 8-char UUID prefix for unique filenames
        let now = chrono::Utc::now();

        let file_path = match req.item_type {
            ItemType::Document => {
                let slug = slugify(&req.title);
                let fname = format!("{slug}-{short_id}.md");
                let rel = if let Some(ref parent_id) = req.parent_id {
                    if let Ok(parent) = self.db.get_item(parent_id) {
                        if let Some(ref p) = parent.file_path {
                            let parent_dir = Path::new(p).parent().unwrap_or(Path::new("docs"));
                            parent_dir.join(&fname).to_string_lossy().to_string()
                        } else {
                            format!("docs/{fname}")
                        }
                    } else {
                        format!("docs/{fname}")
                    }
                } else {
                    format!("docs/{fname}")
                };
                Some(rel)
            }
            ItemType::Note => {
                let date = now.format("%Y-%m-%d");
                let slug = slugify(&req.title);
                Some(format!("notes/{date}-{slug}-{short_id}.md"))
            }
            ItemType::Project => {
                let slug = slugify(&req.title);
                let fname = format!("{slug}-{short_id}.md");
                Some(format!("projects/{fname}"))
            }
            _ => None,
        };

        if let Some(ref fp) = file_path {
            let content = req.content.as_deref().unwrap_or("");
            self.files.write_file(fp, content).map_err(|e| e.to_string())?;
        }

        let item = Item {
            id,
            title: req.title,
            item_type: req.item_type,
            parent_id: req.parent_id,
            sort_order: 0,
            content: req.content,
            status: req.status,
            priority: req.priority,
            file_path,
            created_at: now,
            updated_at: now,
            deleted: false,
        };

        self.db.insert_item(&item).map_err(|e| e.to_string())?;

        if let Some(ref content) = item.content {
            let _ = self.db.index_item(&item.id, item.item_type.as_str(), &item.title, content);
        }

        git_commit_async(self.data_path.clone(), format!("add: {}", item.title));
        Ok(item)
    }

    pub fn get_item(&self, id: &str) -> Result<Item, String> {
        let mut item = self.db.get_item(id).map_err(|e| e.to_string())?;
        if let Some(ref fp) = item.file_path {
            if let Ok(content) = self.files.read_file(fp) {
                item.content = Some(content);
            }
        }
        Ok(item)
    }

    pub fn update_item(&self, req: UpdateItemRequest) -> Result<Item, String> {
        // Only read from DB (skip file read if caller provides content)
        let mut item = self.db.get_item(&req.id).map_err(|e| e.to_string())?;
        let has_content_change = req.content.is_some() || req.title.is_some();

        if let Some(title) = req.title {
            item.title = title;
        }
        if let Some(parent_id) = req.parent_id {
            item.parent_id = Some(parent_id);
        }
        if let Some(sort_order) = req.sort_order {
            item.sort_order = sort_order;
        }
        if let Some(status) = req.status {
            item.status = Some(status);
        }
        if let Some(priority) = req.priority {
            item.priority = Some(priority);
        }
        if let Some(ref content) = req.content {
            item.content = Some(content.clone());
            if let Some(ref fp) = item.file_path {
                self.files.write_file(fp, content).map_err(|e| e.to_string())?;
            }
            let _ = self.db.index_item(&item.id, item.item_type.as_str(), &item.title, content);
        } else if item.file_path.is_some() {
            // Load file content for the returned item
            if let Some(ref fp) = item.file_path {
                if let Ok(content) = self.files.read_file(fp) {
                    item.content = Some(content);
                }
            }
        }

        item.updated_at = chrono::Utc::now();
        self.db.update_item(&item).map_err(|e| e.to_string())?;
        if has_content_change {
            git_commit_async(self.data_path.clone(), format!("edit: {}", item.title));
        }
        Ok(item)
    }

    pub fn delete_item(&self, id: &str) -> Result<(), String> {
        self.db.soft_delete(id).map_err(|e| e.to_string())?;
        let _ = self.db.remove_from_index(id);
        git_commit_async(self.data_path.clone(), format!("delete: {id}"));
        Ok(())
    }

    pub fn get_tree(&self) -> Result<Vec<Item>, String> {
        self.db.get_tree().map_err(|e| e.to_string())
    }

    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        self.db.search(query).map_err(|e| e.to_string())
    }

    pub fn move_item(&self, id: &str, new_parent_id: Option<&str>, sort_order: i32) -> Result<(), String> {
        self.db.move_item(id, new_parent_id, sort_order).map_err(|e| e.to_string())
    }

    /// Reorder an item relative to a target item.
    pub fn reorder_item(&self, item_id: &str, target_id: &str, pos: ReorderPos) -> Result<(), String> {
        match pos {
            ReorderPos::Into => {
                // Reparent into target container, at the top
                self.db.move_item(item_id, Some(target_id), 0).map_err(|e| e.to_string())?;
                let mut siblings = self.db.get_sibling_ids(Some(target_id)).map_err(|e| e.to_string())?;
                siblings.retain(|id| id != item_id);
                siblings.insert(0, item_id.to_string());
                let refs: Vec<&str> = siblings.iter().map(|s| s.as_str()).collect();
                self.db.reorder_siblings(&refs).map_err(|e| e.to_string())
            }
            ReorderPos::Before | ReorderPos::After => {
                // Get target's parent so we can place at same level
                let target = self.db.get_item(target_id).map_err(|e| e.to_string())?;
                let parent_id = target.parent_id.clone();

                // Move to same parent
                self.db.move_item(item_id, parent_id.as_deref(), 0).map_err(|e| e.to_string())?;

                // Get siblings, remove dragged item, insert at correct position
                let mut siblings = self.db.get_sibling_ids(parent_id.as_deref()).map_err(|e| e.to_string())?;
                siblings.retain(|id| id != item_id);

                let target_pos = siblings.iter().position(|id| id == target_id).unwrap_or(0);
                let insert_at = match pos {
                    ReorderPos::Before => target_pos,
                    ReorderPos::After => target_pos + 1,
                    _ => unreachable!(),
                };
                let insert_at = insert_at.min(siblings.len());
                siblings.insert(insert_at, item_id.to_string());

                let refs: Vec<&str> = siblings.iter().map(|s| s.as_str()).collect();
                self.db.reorder_siblings(&refs).map_err(|e| e.to_string())
            }
        }
    }

    /// Scan docs/ and notes/ for untracked .md files and import them into the DB.
    pub fn sync_filesystem(&self) -> Result<usize, String> {
        // Collect all file_paths already tracked in DB
        let tree = self.get_tree()?;
        let tracked: std::collections::HashSet<String> = tree.iter()
            .filter_map(|item| item.file_path.clone())
            .collect();

        let mut imported = 0;

        // Scan docs/
        for rel_path in self.files.list_md_files("docs") {
            if tracked.contains(&rel_path) { continue; }
            let content = self.files.read_file(&rel_path).unwrap_or_default();
            let title = title_from_content(&content, &rel_path);
            let item = self.create_item(CreateItemRequest {
                title,
                item_type: ItemType::Document,
                parent_id: None,
                content: Some(content),
                status: None,
                priority: None,
            })?;
            // The create_item wrote a new file, but we want to keep the original.
            // Overwrite the DB file_path to point to the existing file instead.
            self.db.update_file_path(&item.id, &rel_path).map_err(|e| e.to_string())?;
            // Delete the duplicate file that create_item wrote
            if let Some(ref fp) = item.file_path {
                if fp != &rel_path {
                    let _ = self.files.delete_file(fp);
                }
            }
            imported += 1;
        }

        // Scan notes/
        for rel_path in self.files.list_md_files("notes") {
            if tracked.contains(&rel_path) { continue; }
            let content = self.files.read_file(&rel_path).unwrap_or_default();
            let title = title_from_content(&content, &rel_path);
            let item = self.create_item(CreateItemRequest {
                title,
                item_type: ItemType::Note,
                parent_id: None,
                content: Some(content),
                status: None,
                priority: None,
            })?;
            self.db.update_file_path(&item.id, &rel_path).map_err(|e| e.to_string())?;
            if let Some(ref fp) = item.file_path {
                if fp != &rel_path {
                    let _ = self.files.delete_file(fp);
                }
            }
            imported += 1;
        }

        // Scan projects/
        for rel_path in self.files.list_md_files("projects") {
            if tracked.contains(&rel_path) { continue; }
            let content = self.files.read_file(&rel_path).unwrap_or_default();
            let title = title_from_content(&content, &rel_path);
            let item = self.create_item(CreateItemRequest {
                title,
                item_type: ItemType::Project,
                parent_id: None,
                content: Some(content),
                status: None,
                priority: None,
            })?;
            self.db.update_file_path(&item.id, &rel_path).map_err(|e| e.to_string())?;
            if let Some(ref fp) = item.file_path {
                if fp != &rel_path {
                    let _ = self.files.delete_file(fp);
                }
            }
            imported += 1;
        }

        Ok(imported)
    }

    /// Load file content for items that have file_path. Modifies items in place.
    pub fn load_file_contents(&self, items: &mut [Item]) {
        for item in items.iter_mut() {
            if let Some(ref fp) = item.file_path {
                if let Ok(content) = self.files.read_file(fp) {
                    item.content = Some(content);
                }
            }
        }
    }
}

/// Extract title from markdown content (first # heading) or filename.
fn title_from_content(content: &str, rel_path: &str) -> String {
    // Try first heading
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            let title = heading.trim();
            if !title.is_empty() {
                return title.to_string();
            }
        }
    }
    // Fall back to filename without extension
    std::path::Path::new(rel_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string())
}

/// Migrate legacy workspace layout (files at root) to mote-data/ subdirectory.
fn maybe_migrate_legacy(workspace_path: &Path, data_path: &Path) {
    let legacy_db = workspace_path.join(".mote.db");
    let new_db = data_path.join(".mote.db");
    if legacy_db.exists() && !new_db.exists() {
        let _ = std::fs::create_dir_all(data_path);
        let _ = std::fs::rename(&legacy_db, &new_db);
        let legacy_docs = workspace_path.join("docs");
        if legacy_docs.exists() {
            let _ = std::fs::rename(&legacy_docs, data_path.join("docs"));
        }
        let legacy_notes = workspace_path.join("notes");
        if legacy_notes.exists() {
            let _ = std::fs::rename(&legacy_notes, data_path.join("notes"));
        }
    }
}

fn slugify(s: &str) -> String {
    let raw: String = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    // Collapse consecutive dashes and trim
    raw.split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Initialize a git repo in the data directory if not already present.
fn git_init(data_path: &Path) {
    if data_path.join(".git").exists() {
        return;
    }
    let _ = Command::new("git")
        .args(["init"])
        .current_dir(data_path)
        .output();
    // Create .gitignore to skip the SQLite WAL/SHM temp files
    let gitignore = data_path.join(".gitignore");
    if !gitignore.exists() {
        let _ = std::fs::write(&gitignore, "*.db-wal\n*.db-shm\n*.db-journal\n");
    }
}

/// Auto-commit all changes in the data directory (runs in background thread).
pub fn git_commit_async(data_path: PathBuf, message: String) {
    std::thread::spawn(move || {
        // Stage all changes
        let _ = Command::new("git")
            .args(["add", "-A"])
            .current_dir(&data_path)
            .output();
        // Commit (no-op if nothing to commit)
        let _ = Command::new("git")
            .args(["commit", "-m", &message, "--allow-empty-message", "--no-gpg-sign"])
            .current_dir(&data_path)
            .output();
    });
}
