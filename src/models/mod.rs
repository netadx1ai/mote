use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Document,
    Note,
    Task,
    Folder,
    Project,
}

impl ItemType {
    pub fn as_str(&self) -> &str {
        match self {
            ItemType::Document => "document",
            ItemType::Note => "note",
            ItemType::Task => "task",
            ItemType::Folder => "folder",
            ItemType::Project => "project",
        }
    }
}

impl std::fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ItemType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "document" => Ok(ItemType::Document),
            "note" => Ok(ItemType::Note),
            "task" => Ok(ItemType::Task),
            "folder" => Ok(ItemType::Folder),
            "project" => Ok(ItemType::Project),
            _ => Err(format!("unknown item type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Done => "done",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            TaskStatus::Todo => "Todo",
            TaskStatus::InProgress => "In Progress",
            TaskStatus::Done => "Done",
            TaskStatus::Cancelled => "Cancelled",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            TaskStatus::Todo => "○",
            TaskStatus::InProgress => "◐",
            TaskStatus::Done => "●",
            TaskStatus::Cancelled => "✕",
        }
    }

    pub fn next(&self) -> TaskStatus {
        match self {
            TaskStatus::Todo => TaskStatus::InProgress,
            TaskStatus::InProgress => TaskStatus::Done,
            TaskStatus::Done => TaskStatus::Cancelled,
            TaskStatus::Cancelled => TaskStatus::Todo,
        }
    }
}

impl FromStr for TaskStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "todo" => Ok(TaskStatus::Todo),
            "in_progress" => Ok(TaskStatus::InProgress),
            "done" => Ok(TaskStatus::Done),
            "cancelled" => Ok(TaskStatus::Cancelled),
            _ => Err(format!("unknown status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    None,
    Low,
    Medium,
    High,
    Urgent,
}

impl TaskPriority {
    pub fn as_str(&self) -> &str {
        match self {
            TaskPriority::None => "none",
            TaskPriority::Low => "low",
            TaskPriority::Medium => "medium",
            TaskPriority::High => "high",
            TaskPriority::Urgent => "urgent",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            TaskPriority::None => "-",
            TaskPriority::Low => "Low",
            TaskPriority::Medium => "Med",
            TaskPriority::High => "High",
            TaskPriority::Urgent => "!!",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            TaskPriority::None => "#5a6577",
            TaskPriority::Low => "#64b5f6",
            TaskPriority::Medium => "#ffa726",
            TaskPriority::High => "#ef5350",
            TaskPriority::Urgent => "#ff1744",
        }
    }

    pub fn next(&self) -> TaskPriority {
        match self {
            TaskPriority::None => TaskPriority::Low,
            TaskPriority::Low => TaskPriority::Medium,
            TaskPriority::Medium => TaskPriority::High,
            TaskPriority::High => TaskPriority::Urgent,
            TaskPriority::Urgent => TaskPriority::None,
        }
    }
}

impl FromStr for TaskPriority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(TaskPriority::None),
            "low" => Ok(TaskPriority::Low),
            "medium" => Ok(TaskPriority::Medium),
            "high" => Ok(TaskPriority::High),
            "urgent" => Ok(TaskPriority::Urgent),
            _ => Err(format!("unknown priority: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub item_type: ItemType,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub content: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub file_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted: bool,
}

impl Item {
    /// Content to store in DB — None for file-backed items.
    pub fn db_content(&self) -> Option<&str> {
        if self.file_path.is_some() { None } else { self.content.as_deref() }
    }
}

#[derive(Debug, Clone)]
pub struct CreateItemRequest {
    pub title: String,
    pub item_type: ItemType,
    pub parent_id: Option<String>,
    pub content: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateItemRequest {
    pub id: String,
    pub title: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: Option<i32>,
    pub content: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub item_type: ItemType,
    pub snippet: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub workspace_path: Option<String>,
}

/// Task filter for UI views.
#[derive(Debug, Clone, PartialEq)]
pub enum TaskFilter {
    All,
    Todo,
    InProgress,
    Done,
}

impl TaskFilter {
    pub fn matches(&self, status: Option<&TaskStatus>) -> bool {
        match self {
            TaskFilter::All => true,
            TaskFilter::Todo => status == Some(&TaskStatus::Todo),
            TaskFilter::InProgress => status == Some(&TaskStatus::InProgress),
            TaskFilter::Done => status == Some(&TaskStatus::Done),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            TaskFilter::All => "All",
            TaskFilter::Todo => "Todo",
            TaskFilter::InProgress => "Active",
            TaskFilter::Done => "Done",
        }
    }
}
