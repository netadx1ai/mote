use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct FileManager {
    workspace_path: PathBuf,
}

impl FileManager {
    pub fn new(workspace_path: &Path) -> Self {
        FileManager {
            workspace_path: workspace_path.to_path_buf(),
        }
    }

    /// Resolve a relative path within the workspace, rejecting traversal.
    fn safe_resolve(&self, relative_path: &str) -> io::Result<PathBuf> {
        // Reject obvious traversal patterns before joining
        if relative_path.contains("..") {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "path traversal detected: '..' not allowed",
            ));
        }

        let full_path = self.workspace_path.join(relative_path);

        // Canonicalize parent (which must exist or be created) and verify containment.
        // For reads, the file itself must exist; for writes, check the parent.
        let check_path = if full_path.exists() {
            full_path.canonicalize()?
        } else {
            // File doesn't exist yet (write case) — canonicalize the parent
            let parent = full_path.parent().unwrap_or(&self.workspace_path);
            if parent.exists() {
                let canonical_parent = parent.canonicalize()?;
                canonical_parent.join(full_path.file_name().unwrap_or_default())
            } else {
                // Parent also doesn't exist — rely on the string check above
                full_path.clone()
            }
        };

        let canonical_ws = self.workspace_path.canonicalize().unwrap_or_else(|_| self.workspace_path.clone());
        if !check_path.starts_with(&canonical_ws) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("path escapes workspace: {}", relative_path),
            ));
        }

        Ok(full_path)
    }

    pub fn write_file(&self, relative_path: &str, content: &str) -> io::Result<()> {
        let full_path = self.safe_resolve(relative_path)?;
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, content)
    }

    pub fn read_file(&self, relative_path: &str) -> io::Result<String> {
        let full_path = self.safe_resolve(relative_path)?;
        fs::read_to_string(&full_path)
    }

    pub fn ensure_dirs(&self) -> io::Result<()> {
        fs::create_dir_all(self.workspace_path.join("docs"))?;
        fs::create_dir_all(self.workspace_path.join("notes"))?;
        Ok(())
    }
}
