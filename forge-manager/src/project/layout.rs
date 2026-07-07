// Copyright Andrey Zelenskiy, 2024-2026

use std::{fs::create_dir_all, path::PathBuf};

use crate::{errors::ManagerError, run::RunId, ManagerResult};

#[derive(Debug, Clone)]
pub struct ProjectLayout(PathBuf);

impl ProjectLayout {
    /// Create a new layout manager from project root
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    // Directory paths

    /// Project root directory path
    pub fn root_dir(&self) -> &PathBuf {
        &self.0
    }

    /// Path to directory with all simulation runs
    pub fn runs_dir(&self) -> PathBuf {
        self.0.join("runs")
    }

    /// Path to specific simulation run
    pub fn run_dir(&self, run_id: &RunId) -> PathBuf {
        self.0.join(format!("runs/{run_id}"))
    }

    /// Path to the directory with analysis tools
    pub fn analysis_dir(&self) -> PathBuf {
        self.0.join("analysis")
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

            // root/analysis/
            create_dir_all(self.analysis_dir())?;

            Ok(())
        }
    }
}
