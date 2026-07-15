// Copyright Andrey Zelenskiy, 2024-2026

//! # Module for setting up simulation project
//!
//! This module provides methods for setting up the files and directory tree
//! of a numerical simulation project.
//! The methods emphasize data safety, providing several protocols for dealing
//! with existing data.

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    errors::ManagerError,
    run::{parameters::ParameterMap, Run, RunId, RunIdBuilder},
    ManagerResult,
};

pub(crate) mod manifest;
use manifest::ProjectManifest;

pub(crate) mod layout;
use layout::ProjectLayout;

pub(crate) mod build_methods;

mod registry;
use registry::{query::RunFilter, Registry};

/// Structure that manages the simulation project
#[derive(Default, Serialize, Deserialize)]
pub struct ProjectManager {
    /// Project configuration data
    manifest: ProjectManifest,
    /// Project path dependencies
    layout: ProjectLayout,
    /// Simulation run registry
    registry: Registry,
}

impl ProjectManager {
    /// Creates a new simulation project
    ///
    /// The resulting layout for project with path `project_root` is
    ///
    /// ``` ignore
    /// project_root/
    /// |     manifest.toml    # Project manifest file
    /// |
    /// +---- logs/            # Directory for error message logs
    /// |
    /// +---- runs/            # Directory with all simulation runs
    /// ```
    pub fn create(
        name: &str,
        author: &Option<String>,
        description: &Option<String>,
        path: &Path,
        dirs: Option<Vec<String>>,
    ) -> ManagerResult<Self> {
        if Self::exists(path) {
            Err(ManagerError::ProjectAlreadyExists(PathBuf::from(path)))
        } else {
            // Create a new project manager
            let manager = Self {
                manifest: ProjectManifest::new(name, author, description),
                layout: ProjectLayout::new(PathBuf::from(path), dirs),
                registry: Registry,
            };

            // Initialize project layout
            manager.layout.create_layout()?;

            // Write manifest.toml
            manager.manifest.write(path)?;

            Ok(manager)
        }
    }

    /// Loads a simulation project from file
    pub fn load(path: &Path) -> ManagerResult<Self> {
        if Self::exists(path) {
            let dirs_vec: Vec<String> = fs::read_dir(path)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_dir())
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|name| !["logs", "runs"].contains(&name.as_str()))
                .collect();

            let dirs = match dirs_vec.is_empty() {
                true => Some(dirs_vec),
                false => None,
            };

            Ok(Self {
                manifest: ProjectManifest::load(path)?,
                layout: ProjectLayout::new(PathBuf::from(path), dirs),
                registry: Registry,
            })
        } else {
            Err(ManagerError::ProjectNotFound(PathBuf::from(path)))
        }
    }

    /// Checks if a project is initialized at a given path
    pub fn exists(path: &Path) -> bool {
        path.with_file_name("manifest.toml").exists()
    }

    /// Project name
    pub fn name(&self) -> &str {
        &self.manifest.metadata.name
    }

    /// Project path
    pub fn path(&self) -> &Path {
        self.layout.root_dir()
    }

    /// Project author (if given)
    pub fn author(&self) -> &Option<String> {
        &self.manifest.metadata.author
    }

    /// Project description (if given)
    pub fn description(&self) -> &Option<String> {
        &self.manifest.metadata.description
    }

    /// User-specified directories
    pub fn other_dirs(&self) -> &Option<Vec<String>> {
        &self.layout.other_dirs
    }

    /// Registers a simulation run
    pub fn register_run(
        &self,
        id_builder: RunIdBuilder,
        parameters: &ParameterMap,
    ) -> ManagerResult<RunId> {
        self.registry.register(&self.layout, id_builder, parameters)
    }

    /// Returns simulation run information
    pub fn get_run(&self, id: &RunId) -> ManagerResult<Run> {
        self.registry.get(&self.layout, id)
    }

    /// Returns simulation run directory
    pub fn get_run_dir(&self, id: &RunId) -> PathBuf {
        self.layout.run_dir(id)
    }

    /// Returns a list of simulation runs that pass specific criteria
    pub fn query_runs(&self, filter: &RunFilter) -> ManagerResult<Vec<Run>> {
        self.registry.query(&self.layout, filter)
    }

    /// Updates the status of a simulation run
    pub fn update_run_status(
        &self,
        id: &RunId,
        status_str: &str,
    ) -> ManagerResult<()> {
        self.registry.update_status(&self.layout, id, status_str)
    }

    /// Log error to a file
    pub fn log_error<E: Error>(
        &self,
        error: E,
        id: Option<&RunId>,
    ) -> ManagerResult<()> {
        let log_name = match id {
            Some(run_id) => run_id.to_string(),
            None => Utc::now().format("%Y-%m-%d-%H-%M-%S").to_string(),
        };

        let log_path = self.layout.logs_dir().with_file_name(log_name);

        fs::write(&log_path, error.to_string()).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use crate::project::manifest::CURRENT_SCHEMA_VERSION;

    use super::*;
    use tempfile::tempdir;

    // Creates correct directory structure
    #[test]
    fn test_create_correct_layout() {
        // Get a path to a new temporary directory
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        assert!(
            ProjectManager::create("test", &None, &None, &path, None).is_ok(),
            "Failed to initialize a simple project directory"
        );

        assert!(&path.exists(), "Failed to initialize project directory");

        assert!(
            &path.with_file_name("manifest.toml").exists(),
            "Failed to create the manifest file"
        );

        assert!(
            &path.join("runs").exists(),
            "Failed to initialize root/runs/ directory"
        );

        assert!(
            &path.join("analysis").exists(),
            "Failed to initialize root/analysis/ directory"
        );
    }

    // Loading a created project returns identical config
    #[test]
    fn load_created_project() {
        // Get a path to a new temporary directory
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        let manager = ProjectManager::create("test", &None, &None, &path, None)
            .expect("Failed to initialize a simple project directory");

        let manager_load = ProjectManager::load(&path)
            .expect("Failed to load existing project");

        assert_eq!(
            manager.manifest, manager_load.manifest,
            "Loaded manifest differs from the created copy"
        );
    }

    // Creating project twice returns ProjectAlreadyExists
    #[test]
    fn test_create_twice() {
        // Get a path to a new temporary directory
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        assert!(
            ProjectManager::create("test", &None, &None, &path, None).is_ok(),
            "Failed to initialize a test project"
        );

        assert!(
            ProjectManager::create("test", &None, &None, &path, None)
                .is_err_and(|e| {
                    if let ManagerError::ProjectAlreadyExists(p) = e {
                        p == path
                    } else {
                        false
                    }
                }),
            "Unexpected second initialization of existing project"
        );
    }

    // Loading nonexistent path returns ProjectNotFound
    #[test]
    fn test_load_missing() {
        // Get a path to a new temporary directory
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        let result = ProjectManager::load(&path);

        assert!(
            result.is_err_and(|e| {
                if let ManagerError::ProjectNotFound(p) = e {
                    p == path
                } else {
                    false
                }
            }),
            "Unexpected load of project from non-existing source"
        );
    }

    // Old schema version returns SchemaMismatch
    #[test]
    fn load_different_schema() {
        // Get a path to a new temporary directory
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        let mut manager =
            ProjectManager::create("test", &None, &None, &path, None)
                .expect("Failed to initialize a simple project directory");

        manager.manifest.schema_version += 1;

        // This is an exceptional case for accessing schema version
        manager
            .manifest
            .write(&path)
            .expect("Failed to overwrite manifest.toml");

        assert!(ProjectManager::load(&path).is_err_and(|e| {
            if let ManagerError::SchemaMismatch {
                path,
                manifest_schema,
                current_schema,
            } = e
            {
                path == path
                    && manifest_schema == CURRENT_SCHEMA_VERSION + 1
                    && current_schema == CURRENT_SCHEMA_VERSION
            } else {
                false
            }
        }))
    }
}
