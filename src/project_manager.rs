use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Manages project-specific vector databases
pub struct ProjectManager {
    /// The base data directory for fallback/global database
    global_data_dir: PathBuf,
}

impl ProjectManager {
    pub fn new(global_data_dir: PathBuf) -> Self {
        Self { global_data_dir }
    }

    /// Detect the current project root by looking for common project markers
    pub fn detect_project_root() -> Option<PathBuf> {
        // Start from current working directory
        let cwd = env::current_dir().ok()?;
        Self::find_project_root(&cwd)
    }

    /// Find project root by looking for markers like .git, package.json, Cargo.toml, etc.
    fn find_project_root(start_path: &Path) -> Option<PathBuf> {
        let project_markers = [
            ".git",
            "package.json",
            "Cargo.toml",
            "pyproject.toml",
            "go.mod",
            "pom.xml",
            "build.gradle",
            ".project",
            "Gemfile",
            "composer.json",
        ];

        let mut current = start_path;
        loop {
            // Check if any project marker exists in current directory
            for marker in &project_markers {
                if current.join(marker).exists() {
                    return Some(current.to_path_buf());
                }
            }

            // Move up to parent directory
            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }

        None
    }

    /// Get the vector database path for the current context
    pub fn get_database_path(&self) -> Result<PathBuf> {
        // Try to detect project root
        if let Some(project_root) = Self::detect_project_root() {
            let coderag_dir = project_root.join(".coderag");

            // Create .coderag directory if it doesn't exist
            if !coderag_dir.exists() {
                fs::create_dir_all(&coderag_dir).with_context(|| {
                    format!("Failed to create .coderag directory at {:?}", coderag_dir)
                })?;

                // Add .coderag to .gitignore
                self.update_gitignore(&project_root)?;
            }

            Ok(coderag_dir.join("vectordb.json"))
        } else {
            // Fall back to global database
            Ok(self.global_data_dir.join("coderag_vectordb.json"))
        }
    }

    /// Update .gitignore to include .coderag directory
    fn update_gitignore(&self, project_root: &Path) -> Result<()> {
        let gitignore_path = project_root.join(".gitignore");

        // Read existing .gitignore content
        let mut content = if gitignore_path.exists() {
            fs::read_to_string(&gitignore_path)
                .with_context(|| format!("Failed to read .gitignore at {:?}", gitignore_path))?
        } else {
            String::new()
        };

        // Check if .coderag is already in .gitignore
        let coderag_entry = ".coderag/";
        if !content
            .lines()
            .any(|line| line.trim() == coderag_entry || line.trim() == ".coderag")
        {
            // Add .coderag entry
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str("\n# CodeRAG vector database\n");
            content.push_str(coderag_entry);
            content.push('\n');

            // Write updated .gitignore
            fs::write(&gitignore_path, content)
                .with_context(|| format!("Failed to write .gitignore at {:?}", gitignore_path))?;
        }

        Ok(())
    }

    /// Get information about the current project context
    pub fn get_project_info(&self) -> ProjectInfo {
        if let Some(project_root) = Self::detect_project_root() {
            let db_path = project_root.join(".coderag").join("vectordb.json");
            ProjectInfo {
                is_project: true,
                project_root: Some(project_root.clone()),
                database_path: db_path,
                project_name: project_root
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string()),
            }
        } else {
            ProjectInfo {
                is_project: false,
                project_root: None,
                database_path: self.global_data_dir.join("coderag_vectordb.json"),
                project_name: None,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub is_project: bool,
    pub project_root: Option<PathBuf>,
    pub database_path: PathBuf,
    pub project_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_project_detection() {
        // Create a temporary directory with a git repo
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        // Set current directory to temp dir
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        // Test project detection
        let project_root = ProjectManager::detect_project_root();
        assert!(project_root.is_some());

        // Canonicalize paths to handle macOS /private symlinks
        let detected_root = project_root.unwrap().canonicalize().unwrap();
        let expected_root = temp_dir.path().canonicalize().unwrap();
        assert_eq!(detected_root, expected_root);

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_gitignore_update() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ProjectManager::new(temp_dir.path().to_path_buf());

        // Test creating new .gitignore
        manager.update_gitignore(temp_dir.path()).unwrap();
        let gitignore_content = fs::read_to_string(temp_dir.path().join(".gitignore")).unwrap();
        assert!(gitignore_content.contains(".coderag/"));

        // Test updating existing .gitignore
        fs::write(temp_dir.path().join(".gitignore"), "node_modules/\n").unwrap();
        manager.update_gitignore(temp_dir.path()).unwrap();
        let gitignore_content = fs::read_to_string(temp_dir.path().join(".gitignore")).unwrap();
        assert!(gitignore_content.contains("node_modules/"));
        assert!(gitignore_content.contains(".coderag/"));
    }
}
