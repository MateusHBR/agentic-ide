use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub head: String,
    pub is_main: bool,
    pub is_bare: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub worktrees: Vec<WorktreeInfo>,
}

pub fn list_worktrees(project_path: &str) -> Result<Vec<WorktreeInfo>, String> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_path = String::new();
    let mut current_head = String::new();
    let mut current_branch = String::new();
    let mut is_bare = false;
    let mut is_first = true;

    for line in stdout.lines() {
        if line.starts_with("worktree ") {
            if !current_path.is_empty() {
                worktrees.push(WorktreeInfo {
                    path: current_path.clone(),
                    branch: current_branch.clone(),
                    head: current_head.clone(),
                    is_main: is_first,
                    is_bare,
                });
                is_first = false;
            }
            current_path = line.trim_start_matches("worktree ").to_string();
            current_head.clear();
            current_branch.clear();
            is_bare = false;
        } else if line.starts_with("HEAD ") {
            current_head = line.trim_start_matches("HEAD ").to_string();
        } else if line.starts_with("branch ") {
            let branch = line.trim_start_matches("branch ");
            current_branch = branch
                .strip_prefix("refs/heads/")
                .unwrap_or(branch)
                .to_string();
        } else if line == "bare" {
            is_bare = true;
        } else if line == "detached" {
            current_branch = format!("(detached {})", &current_head[..8.min(current_head.len())]);
        }
    }

    if !current_path.is_empty() {
        worktrees.push(WorktreeInfo {
            path: current_path,
            branch: current_branch,
            head: current_head,
            is_main: is_first,
            is_bare,
        });
    }

    Ok(worktrees)
}

pub fn get_diff(worktree_path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["diff"])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to run git diff: {}", e))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn get_staged_diff(worktree_path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to run git diff: {}", e))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn get_file_diff(
    worktree_path: &str,
    file: &str,
    context_lines: u32,
    staged: bool,
) -> Result<String, String> {
    let context_arg = format!("-U{}", context_lines);
    let mut args = vec!["diff", &context_arg];
    if staged {
        args.push("--cached");
    }
    args.push("--");
    args.push(file);

    let output = Command::new("git")
        .args(&args)
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to run git diff: {}", e))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn get_log(worktree_path: &str, count: u32) -> Result<Vec<LogEntry>, String> {
    let output = Command::new("git")
        .args([
            "log",
            &format!("-{}", count),
            "--pretty=format:%H|%h|%an|%ar|%s",
        ])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to run git log: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<LogEntry> = stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(5, '|').collect();
            if parts.len() == 5 {
                Some(LogEntry {
                    hash: parts[0].to_string(),
                    short_hash: parts[1].to_string(),
                    author: parts[2].to_string(),
                    relative_time: parts[3].to_string(),
                    message: parts[4].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(entries)
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub relative_time: String,
    pub message: String,
}

pub fn get_status(worktree_path: &str) -> Result<Vec<FileStatus>, String> {
    let output = Command::new("git")
        .args(["status", "--porcelain=v1"])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to run git status: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<FileStatus> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let status = line[..2].to_string();
            let file = line[3..].to_string();
            FileStatus { status, file }
        })
        .collect();

    Ok(files)
}

#[derive(Debug, Clone, Serialize)]
pub struct FileStatus {
    pub status: String,
    pub file: String,
}

pub fn get_project_info(project_path: &str) -> Result<ProjectInfo, String> {
    let path = Path::new(project_path);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| project_path.to_string());

    let worktrees = list_worktrees(project_path)?;

    Ok(ProjectInfo {
        name,
        path: project_path.to_string(),
        worktrees,
    })
}

pub fn add_worktree(project_path: &str, path: &str, branch: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["worktree", "add", path, "-b", branch])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to add worktree: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn stage_file(worktree_path: &str, file: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["add", "--", file])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to stage file: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(())
}

pub fn unstage_file(worktree_path: &str, file: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["reset", "HEAD", "--", file])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to unstage file: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(())
}

pub fn remove_worktree(project_path: &str, worktree_path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["worktree", "remove", worktree_path])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to remove worktree: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}
