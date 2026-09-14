use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct GitSyncStatus {
    pub branch: String,
    pub remote_url: Option<String>,
    pub last_synced: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub is_syncing: bool,
}

impl Default for GitSyncStatus {
    fn default() -> Self {
        Self {
            branch: "main".to_string(),
            remote_url: None,
            last_synced: None,
            last_error: None,
            is_syncing: false,
        }
    }
}

enum GitCommand {
    CommitAndPush {
        data_path: PathBuf,
        message: String,
    },
    SyncNow {
        data_path: PathBuf,
    },
    SetRemote {
        data_path: PathBuf,
        url: String,
    },
    RefreshStatus {
        data_path: PathBuf,
    },
}

pub struct GitManager {
    tx: Sender<GitCommand>,
    status: Arc<Mutex<GitSyncStatus>>,
}

impl GitManager {
    pub fn new(data_path: &Path) -> Self {
        let (tx, rx) = channel::<GitCommand>();
        let initial_status = Arc::new(Mutex::new(read_git_status(data_path)));
        let status_clone = initial_status.clone();

        thread::Builder::new()
            .name("mote-git-worker".to_string())
            .spawn(move || {
                run_worker_loop(rx, status_clone);
            })
            .expect("failed to spawn mote-git-worker thread");

        Self {
            tx,
            status: initial_status,
        }
    }

    pub fn commit_and_push(&self, data_path: PathBuf, message: String) {
        let _ = self.tx.send(GitCommand::CommitAndPush { data_path, message });
    }

    pub fn sync_now(&self, data_path: PathBuf) {
        let _ = self.tx.send(GitCommand::SyncNow { data_path });
    }

    pub fn set_remote(&self, data_path: PathBuf, url: String) {
        let _ = self.tx.send(GitCommand::SetRemote { data_path, url });
    }

    pub fn refresh_status(&self, data_path: PathBuf) {
        let _ = self.tx.send(GitCommand::RefreshStatus { data_path });
    }

    pub fn get_status(&self) -> GitSyncStatus {
        self.status.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

fn macos_git_command(data_path: &Path) -> Command {
    let mut cmd = Command::new("git");
    let current_path = std::env::var("PATH").unwrap_or_default();
    let extended_path = format!("{}:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin", current_path);
    cmd.env("PATH", extended_path)
        .env("GIT_TERMINAL_PROMPT", "0")
        .current_dir(data_path);
    cmd
}

fn recover_index_lock(data_path: &Path) {
    let lock_file = data_path.join(".git").join("index.lock");
    if lock_file.exists() {
        // If lock file exists, remove it safely so future git calls succeed
        let _ = std::fs::remove_file(&lock_file);
    }
}

fn ensure_git_identity(data_path: &Path) {
    // Check user.name
    let name_check = macos_git_command(data_path)
        .args(["config", "user.name"])
        .output();
    let has_name = match name_check {
        Ok(ref o) => !o.stdout.is_empty(),
        Err(_) => false,
    };
    if !has_name {
        let _ = macos_git_command(data_path)
            .args(["config", "user.name", "Mote User"])
            .output();
    }

    // Check user.email
    let email_check = macos_git_command(data_path)
        .args(["config", "user.email"])
        .output();
    let has_email = match email_check {
        Ok(ref o) => !o.stdout.is_empty(),
        Err(_) => false,
    };
    if !has_email {
        let _ = macos_git_command(data_path)
            .args(["config", "user.email", "mote@local"])
            .output();
    }
}

fn read_git_status(data_path: &Path) -> GitSyncStatus {
    let mut status = GitSyncStatus::default();
    if !data_path.join(".git").exists() {
        return status;
    }

    // Branch
    if let Ok(o) = macos_git_command(data_path).args(["rev-parse", "--abbrev-ref", "HEAD"]).output() {
        if o.status.success() {
            let b = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !b.is_empty() {
                status.branch = b;
            }
        }
    }

    // Remote
    if let Ok(o) = macos_git_command(data_path).args(["remote", "get-url", "origin"]).output() {
        if o.status.success() {
            let u = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !u.is_empty() {
                status.remote_url = Some(u);
            }
        }
    }

    status
}

fn run_worker_loop(rx: Receiver<GitCommand>, status_arc: Arc<Mutex<GitSyncStatus>>) {
    while let Ok(cmd) = rx.recv() {
        match cmd {
            GitCommand::CommitAndPush { data_path, mut message } => {
                // Debounce / Coalesce: wait 1.5s for any further pending commits
                let start = Instant::now();
                while start.elapsed() < Duration::from_millis(1500) {
                    if let Ok(next) = rx.try_recv() {
                        match next {
                            GitCommand::CommitAndPush { data_path: p2, message: m2 } => {
                                if p2 == data_path {
                                    message = m2; // coalesce with newer message
                                }
                            }
                            other => {
                                // Execute other non-commit command immediately, then proceed
                                execute_command(other, &status_arc);
                            }
                        }
                    } else {
                        thread::sleep(Duration::from_millis(150));
                    }
                }

                execute_commit_and_push(&data_path, &message, &status_arc);
            }
            other => {
                execute_command(other, &status_arc);
            }
        }
    }
}

fn execute_command(cmd: GitCommand, status_arc: &Arc<Mutex<GitSyncStatus>>) {
    match cmd {
        GitCommand::SyncNow { data_path } => {
            execute_sync_now(&data_path, status_arc);
        }
        GitCommand::SetRemote { data_path, url } => {
            execute_set_remote(&data_path, &url, status_arc);
        }
        GitCommand::RefreshStatus { data_path } => {
            let updated = read_git_status(&data_path);
            if let Ok(mut lock) = status_arc.lock() {
                lock.branch = updated.branch;
                lock.remote_url = updated.remote_url;
            }
        }
        GitCommand::CommitAndPush { data_path, message } => {
            execute_commit_and_push(&data_path, &message, status_arc);
        }
    }
}

fn execute_commit_and_push(data_path: &Path, message: &str, status_arc: &Arc<Mutex<GitSyncStatus>>) {
    if !data_path.join(".git").exists() {
        return;
    }

    if let Ok(mut lock) = status_arc.lock() {
        lock.is_syncing = true;
    }

    recover_index_lock(data_path);
    ensure_git_identity(data_path);

    // Stage all
    let _ = macos_git_command(data_path).args(["add", "-A"]).output();

    // Check if there are staged changes
    let diff_output = macos_git_command(data_path)
        .args(["diff", "--cached", "--quiet"])
        .output();
    let has_staged_changes = match diff_output {
        Ok(o) => !o.status.success(), // exit code 1 means changes exist
        Err(_) => true,
    };

    if has_staged_changes {
        let commit_res = macos_git_command(data_path)
            .args(["commit", "-m", message, "--allow-empty-message", "--no-gpg-sign"])
            .output();

        if let Err(e) = commit_res {
            if let Ok(mut lock) = status_arc.lock() {
                lock.is_syncing = false;
                lock.last_error = Some(format!("Git commit error: {e}"));
            }
            return;
        }
    }

    // Push if remote exists
    push_to_remote(data_path, status_arc);
}

fn push_to_remote(data_path: &Path, status_arc: &Arc<Mutex<GitSyncStatus>>) {
    // Check current branch
    let branch = macos_git_command(data_path)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).trim().to_string()) } else { None })
        .unwrap_or_else(|| "main".to_string());

    // Check remote url
    let remote_url = macos_git_command(data_path)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()
        .and_then(|o| if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).trim().to_string()) } else { None });

    if remote_url.is_none() {
        if let Ok(mut lock) = status_arc.lock() {
            lock.branch = branch;
            lock.is_syncing = false;
            lock.last_error = None;
        }
        return;
    }

    // Attempt push
    let push_res = macos_git_command(data_path)
        .args(["push", "-u", "origin", &branch])
        .output();

    if let Ok(mut lock) = status_arc.lock() {
        lock.branch = branch;
        lock.remote_url = remote_url;
        lock.is_syncing = false;

        match push_res {
            Ok(o) if o.status.success() => {
                lock.last_synced = Some(Utc::now());
                lock.last_error = None;
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
                if !err.is_empty() {
                    // Extract meaningful line from git stderr
                    let first_err = err.lines().filter(|l| !l.starts_with("To ")).next().unwrap_or(&err);
                    lock.last_error = Some(first_err.to_string());
                } else {
                    lock.last_error = Some("Push failed without details".to_string());
                }
            }
            Err(e) => {
                lock.last_error = Some(format!("Push process error: {e}"));
            }
        }
    }
}

fn execute_sync_now(data_path: &Path, status_arc: &Arc<Mutex<GitSyncStatus>>) {
    if !data_path.join(".git").exists() {
        if let Ok(mut lock) = status_arc.lock() {
            lock.last_error = Some("Git repository not initialized in workspace".to_string());
        }
        return;
    }

    if let Ok(mut lock) = status_arc.lock() {
        lock.is_syncing = true;
    }

    recover_index_lock(data_path);
    ensure_git_identity(data_path);

    // Stage all
    let _ = macos_git_command(data_path).args(["add", "-A"]).output();

    // Check if staged
    let diff_output = macos_git_command(data_path)
        .args(["diff", "--cached", "--quiet"])
        .output();
    if diff_output.map(|o| !o.status.success()).unwrap_or(false) {
        let _ = macos_git_command(data_path)
            .args(["commit", "-m", "sync: manual sync", "--no-gpg-sign"])
            .output();
    }

    // Branch
    let branch = macos_git_command(data_path)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).trim().to_string()) } else { None })
        .unwrap_or_else(|| "main".to_string());

    // Pull with rebase first if remote exists
    let _ = macos_git_command(data_path)
        .args(["pull", "--rebase", "origin", &branch])
        .output();

    // Push
    push_to_remote(data_path, status_arc);
}

fn execute_set_remote(data_path: &Path, url: &str, status_arc: &Arc<Mutex<GitSyncStatus>>) {
    if !data_path.join(".git").exists() {
        return;
    }

    let check_remote = macos_git_command(data_path)
        .args(["remote"])
        .output();

    let has_origin = match check_remote {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains("origin"),
        Err(_) => false,
    };

    let result = if has_origin {
        macos_git_command(data_path)
            .args(["remote", "set-url", "origin", url])
            .output()
    } else {
        macos_git_command(data_path)
            .args(["remote", "add", "origin", url])
            .output()
    };

    if let Ok(mut lock) = status_arc.lock() {
        match result {
            Ok(o) if o.status.success() => {
                lock.remote_url = Some(url.to_string());
                lock.last_error = None;
            }
            Ok(o) => {
                lock.last_error = Some(String::from_utf8_lossy(&o.stderr).trim().to_string());
            }
            Err(e) => {
                lock.last_error = Some(format!("Failed to set remote: {e}"));
            }
        }
    }
}
