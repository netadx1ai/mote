mod db;
mod files;
pub mod backup;

pub use db::Database;
pub use files::FileManager;

use crate::models::*;
use std::path::Path;

pub struct Storage {
    pub(crate) db: Database,
    pub(crate) files: FileManager,
}

impl Storage {
    pub fn new(workspace_path: &Path) -> Result<Self, String> {
        let db = Database::new(workspace_path).map_err(|e| e.to_string())?;
        let files = FileManager::new(workspace_path);
        Ok(Storage { db, files })
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
        Ok(item)
    }

    pub fn delete_item(&self, id: &str) -> Result<(), String> {
        self.db.soft_delete(id).map_err(|e| e.to_string())?;
        let _ = self.db.remove_from_index(id);
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
