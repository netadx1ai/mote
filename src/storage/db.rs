use crate::models::*;
use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(workspace_path: &Path) -> Result<Self, rusqlite::Error> {
        let db_path = workspace_path.join(".mote.db");
        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS items (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                item_type TEXT NOT NULL,
                parent_id TEXT,
                sort_order INTEGER DEFAULT 0,
                content TEXT,
                status TEXT,
                priority TEXT,
                file_path TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted INTEGER DEFAULT 0,
                FOREIGN KEY (parent_id) REFERENCES items(id)
            );
            CREATE INDEX IF NOT EXISTS idx_items_parent ON items(parent_id);
            CREATE INDEX IF NOT EXISTS idx_items_type ON items(item_type);
            CREATE INDEX IF NOT EXISTS idx_items_deleted ON items(deleted);

            CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
                item_id UNINDEXED,
                item_type UNINDEXED,
                title,
                content,
                tokenize='porter unicode61'
            );
            ",
        )?;

        Ok(Database { conn: Mutex::new(conn) })
    }

    pub fn insert_item(&self, item: &Item) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO items (id, title, item_type, parent_id, sort_order, content, status, priority, file_path, created_at, updated_at, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                item.id,
                item.title,
                item.item_type.as_str(),
                item.parent_id,
                item.sort_order,
                item.db_content(),
                item.status.as_ref().map(|s| s.as_str()),
                item.priority.as_ref().map(|p| p.as_str()),
                item.file_path,
                item.created_at.to_rfc3339(),
                item.updated_at.to_rfc3339(),
                item.deleted as i32,
            ],
        )?;
        Ok(())
    }

    pub fn get_item(&self, id: &str) -> Result<Item, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, title, item_type, parent_id, sort_order, content, status, priority, file_path, created_at, updated_at, deleted
             FROM items WHERE id = ?1 AND deleted = 0",
            params![id],
            |row| Ok(row_to_item(row)),
        )
    }

    pub fn update_item(&self, item: &Item) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE items SET title = ?2, parent_id = ?3, sort_order = ?4, content = ?5, status = ?6, priority = ?7, updated_at = ?8
             WHERE id = ?1",
            params![
                item.id,
                item.title,
                item.parent_id,
                item.sort_order,
                item.db_content(),
                item.status.as_ref().map(|s| s.as_str()),
                item.priority.as_ref().map(|p| p.as_str()),
                item.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn soft_delete(&self, id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        // Recursive CTE to cascade to all descendants
        conn.execute(
            "WITH RECURSIVE descendants AS (
                SELECT id FROM items WHERE id = ?1
                UNION ALL
                SELECT i.id FROM items i JOIN descendants d ON i.parent_id = d.id
            )
            UPDATE items SET deleted = 1 WHERE id IN (SELECT id FROM descendants)",
            params![id],
        )?;
        Ok(())
    }

    pub fn move_item(&self, id: &str, new_parent_id: Option<&str>, sort_order: i32) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE items SET parent_id = ?2, sort_order = ?3, updated_at = ?4 WHERE id = ?1",
            params![id, new_parent_id, sort_order, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn get_tree(&self) -> Result<Vec<Item>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, item_type, parent_id, sort_order, content, status, priority, file_path, created_at, updated_at, deleted
             FROM items WHERE deleted = 0 ORDER BY sort_order, created_at",
        )?;
        let items = stmt
            .query_map([], |row| Ok(row_to_item(row)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    // --- FTS search (unified in same DB) ---

    pub fn index_item(&self, id: &str, item_type: &str, title: &str, content: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM search_index WHERE item_id = ?1", params![id])?;
        conn.execute(
            "INSERT INTO search_index (item_id, item_type, title, content) VALUES (?1, ?2, ?3, ?4)",
            params![id, item_type, title, content],
        )?;
        Ok(())
    }

    pub fn remove_from_index(&self, id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM search_index WHERE item_id = ?1", params![id])?;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        // Quote the query to prevent FTS5 operator injection
        let safe_query = format!("\"{}\"", query.replace('"', "\"\""));
        let mut stmt = conn.prepare(
            "SELECT item_id, item_type, title, snippet(search_index, 3, '<b>', '</b>', '...', 32), rank
             FROM search_index WHERE search_index MATCH ?1 ORDER BY rank LIMIT 20"
        )?;
        let results = stmt
            .query_map(params![safe_query], |row| {
                let rank: f64 = row.get(4)?;
                let type_str: String = row.get(1)?;
                Ok(SearchResult {
                    id: row.get(0)?,
                    title: row.get(2)?,
                    item_type: type_str.parse().unwrap_or(ItemType::Document),
                    snippet: row.get(3)?,
                    score: -rank,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(results)
    }
}

fn row_to_item(row: &rusqlite::Row) -> Item {
    let item_type_str: String = row.get(2).unwrap();
    let status_str: Option<String> = row.get(6).unwrap();
    let priority_str: Option<String> = row.get(7).unwrap();
    let created_str: String = row.get(9).unwrap();
    let updated_str: String = row.get(10).unwrap();
    let deleted_int: i32 = row.get(11).unwrap();

    Item {
        id: row.get(0).unwrap(),
        title: row.get(1).unwrap(),
        item_type: item_type_str.parse().unwrap_or(ItemType::Document),
        parent_id: row.get(3).unwrap(),
        sort_order: row.get(4).unwrap(),
        content: row.get(5).unwrap(),
        status: status_str.and_then(|s| s.parse().ok()),
        priority: priority_str.and_then(|p| p.parse().ok()),
        file_path: row.get(8).unwrap(),
        created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
        deleted: deleted_int != 0,
    }
}
