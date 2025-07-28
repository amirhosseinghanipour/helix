use std::path::{Path, PathBuf};
use std::fs;

pub fn normalize_path(path: &Path) -> PathBuf {
    path.to_path_buf()
}

pub fn load_helixignore(repo_path: &Path) -> Vec<String> {
    let ignore_file = repo_path.join(".helixignore");
    if let Ok(content) = fs::read_to_string(&ignore_file) {
        content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.to_string())
            .collect()
    } else {
        Vec::new()
    }
}

pub fn is_ignored(path: &Path, repo_path: &Path) -> bool {
    let relative_path = get_relative_path(repo_path, path).unwrap_or_default();

    // Load .helixignore patterns
    let ignore_patterns = load_helixignore(repo_path);

    // Common ignore patterns (built-in)
    let built_in_patterns = [
        ".helix",
        ".git",
        "target",
        "node_modules",
        ".DS_Store",
        "*.tmp",
        "*.log",
        "*.swp",
        "*.swo",
        "*~",
        ".vscode",
        ".idea",
        "*.o",
        "*.so",
        "*.dylib",
        "*.dll",
        "*.exe",
        "*.pyc",
        "__pycache__",
        ".pytest_cache",
        "*.class",
        "*.jar",
        "*.war",
        "*.ear",
        "*.min.js",
        "*.min.css",
        "dist",
        "build",
        "out",
        "coverage",
        ".nyc_output",
        "*.lcov",
        ".env",
        ".env.local",
        ".env.*.local",
    ];

    // Check built-in patterns
    for pattern in &built_in_patterns {
        if matches_pattern(&relative_path, pattern) {
            return true;
        }
    }

    // Check .helixignore patterns
    for pattern in &ignore_patterns {
        if matches_pattern(&relative_path, pattern) {
            return true;
        }
    }

    false
}

fn matches_pattern(path: &str, pattern: &str) -> bool {
    // Handle simple patterns
    if pattern.starts_with("*.") {
        let ext = &pattern[2..];
        return path.ends_with(ext);
    }

    if pattern.ends_with("/") {
        // Directory pattern
        let dir_pattern = &pattern[..pattern.len() - 1];
        return path.contains(dir_pattern) && path.split('/').any(|part| part == dir_pattern);
    }

    if pattern.starts_with("/") {
        // Pattern from root
        let root_pattern = &pattern[1..];
        return path.starts_with(root_pattern);
    }

    if pattern.ends_with("*") {
        // Prefix pattern
        let prefix = &pattern[..pattern.len() - 1];
        return path.starts_with(prefix);
    }

    if pattern.starts_with("*") && pattern.ends_with("*") {
        // Contains pattern
        let contains = &pattern[1..pattern.len() - 1];
        return path.contains(contains);
    }

    // Exact match
    path == pattern || path.contains(pattern)
}

pub fn get_relative_path(base: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(base)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

#[allow(dead_code)]
pub fn should_track_file(path: &Path, repo_path: &Path) -> bool {
    // Don't track directories
    if path.is_dir() {
        return false;
    }

    // Don't track ignored files
    if is_ignored(path, repo_path) {
        return false;
    }

    // Don't track empty files (optional)
    if let Ok(metadata) = fs::metadata(path) {
        if metadata.len() == 0 {
            return false;
        }
    }

    true
}

#[allow(dead_code)]
pub fn collect_trackable_files(repo_path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(repo_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if should_track_file(&path, repo_path) {
                files.push(path);
            } else if path.is_dir() {
                // Recursively collect files from subdirectories
                files.extend(collect_trackable_files(&path));
            }
        }
    }
    
    files
}
