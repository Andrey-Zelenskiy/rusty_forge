// Copyright Andrey Zelenskiy, 2024-2026

use std::{fs::create_dir_all, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{errors::ManagerError, run::RunId, ManagerResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLayout {
    path: PathBuf,
    pub other_dirs: Option<Vec<String>>,
}

impl Default for ProjectLayout {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./"),
            other_dirs: None,
        }
    }
}

impl ProjectLayout {
    /// Create a new layout manager from project root
    pub fn new(path: PathBuf, other_dirs: Option<Vec<String>>) -> Self {
        Self { path, other_dirs }
    }

    // Directory paths

    /// Project root directory path
    pub fn root_dir(&self) -> &PathBuf {
        &self.path
    }

    /// Path to directory with all simulation runs
    pub fn runs_dir(&self) -> PathBuf {
        self.path.join("runs")
    }

    /// Path to specific simulation run
    pub fn run_dir(&self, run_id: &RunId) -> PathBuf {
        self.path.join(format!("runs/{run_id}"))
    }

    /// Path to the directory with error logs
    pub fn logs_dir(&self) -> PathBuf {
        self.path.join("logs")
    }

    // Filesystem operations

    /// Initialize project layout
    pub fn create_layout(&self) -> ManagerResult<()> {
        if self.root_dir().exists() {
            Err(ManagerError::ProjectAlreadyExists(PathBuf::from(
                self.root_dir(),
            )))
        } else {
            // root/runs/
            create_dir_all(self.runs_dir())?;

            // root/logs/
            create_dir_all(self.logs_dir())?;

            Ok(())
        }
    }
}
