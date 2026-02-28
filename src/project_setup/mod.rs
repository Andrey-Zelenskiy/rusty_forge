// Copyright Andrey Zelenskiy, 2024-2026

//! # Module for setting up simulation project
//!
//! This module provides methods for setting up the files and directory tree
//! of a numerical simulation project.
//! The methods emphasize data safety, providing several protocols for dealing
//! with existing data.
//!
//! ## Example
//!
//! ```rust
//! use rusty_forge::project_setup::ProjectManager;
//! ```

use std::{
    fs,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

use config::Config;
use regex::Regex;

use serde::Deserialize;

mod manifest;
use manifest::ProjectManifest;

/// Project directory initializer
#[derive(Deserialize, Debug)]
pub struct ProjectManager {
    /// Path to the project directory
    path: PathBuf,
    /// Recovery mode option for restarting a simulation from the last
    /// checkpoint
    #[serde(default)]
    recovery_mode: bool,
    /// Template structure of project directory
    directory_structure: DirectoryStructure,
    /// Type of behaviour if project path already exists
    data_protocol: DataProtocol,
}

/// Project directory tree formats
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum DirectoryStructure {
    /// Create a series of subdirectories
    Series { n_runs: usize },
    /// Store data directly in the project directory, specified by the path
    Simple,
    /// Create a subdirectory labelled by /yyyy-mm-dd/
    Timestamped,
}

/// Protocol for dealing with files that already exist
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum DataProtocol {
    /// Copies duplicate data to a timestamped folder
    Archive,
    /// Interrupts the program if data with duplicate names already exist
    Panic,
}

impl ProjectManager {
    /// Simple project structure: new directory without timestamps or series,
    /// panics if already exists
    ///
    /// ## Examples
    ///
    /// ```
    /// use rusty_forge::project_setup::ProjectManager;
    ///
    /// let project_manager = ProjectManager::new_simple("/tmp/simulation_project");
    ///
    /// assert!(!project_manager.is_recovery());
    /// ```
    pub fn new_simple<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: PathBuf::from(path.as_ref()),
            recovery_mode: false,
            directory_structure: DirectoryStructure::Simple,
            data_protocol: DataProtocol::Panic,
        }
    }

    /// Initializes simulation directory
    ///
    ///
    pub fn initialize_project(&mut self) -> Result<(), Error> {
        // Check if the directory already exists, in which case execute data
        // protocol
        if self.exists() {
            self.execute_data_protocol()?;
        }

        self.initialize_directory_structure()?;

        // Check the manifest file for date, complition status
        // If no manifest, check that the directory is empty
        // If empty, Ok(()), otherwise ErrorKind::DirectoryNotEmpty

        // Timestamped: check if timestamps exist. If yes, create a new one
        // and return Ok(()). Otherwise, first copy the old data into a
        // timestamped directory.

        // Recovery: if manifest status is Completed, return
        // ErrorKind::Other, "Simulation "
        Ok(())
    }

    /// Returns true if the project manager is in the recovery mode
    pub fn is_recovery(&self) -> bool {
        self.recovery_mode
    }

    // Checks if the project directory already exists
    fn exists(&self) -> bool {
        self.path.is_dir()
    }

    // Checks if the project directory is timestamped
    fn is_timestamped(&self) -> bool {
        let mut timestamped = false;

        if self.exists() {
            // Regex for the timestamps
            let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();

            // Get items of the directory
            if let Ok(entries) = fs::read_dir(&self.path) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        // Only care about directories
                        if file_type.is_dir() {
                            if let Some(name) = entry.file_name().to_str() {
                                // Check if directory name matches timestamp
                                // format yyyy-mm-dd
                                if re.is_match(name) {
                                    timestamped = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        timestamped
    }

    // Creates a new manifest file
    fn create_manifest(&self) -> ProjectManifest {
        ProjectManifest::new()
    }

    // Loads a preexisting manifest file
    fn load_manifest(&self) -> Result<ProjectManifest, Error> {
        match Config::builder()
            .add_source(config::File::from(self.path.join("manifest.toml")))
            .build()
        {
            Ok(config) => {
                config.try_deserialize::<ProjectManifest>().map_err(|e| {
                    Error::new(
                        ErrorKind::InvalidData,
                        format!("Failed to deserialize manifest file: {e}"),
                    )
                })
            }
            Err(e) => Err(Error::new(
                ErrorKind::NotFound,
                format!(
                    "manifest.toml not found in the project directory: {e}"
                ),
            )),
        }
    }

    // Method for safely dealing with the existing data
    fn execute_data_protocol(&mut self) -> Result<(), Error> {
        // Check if the existing directory is empty
        // (no need for data protocols)
        let mut entries = fs::read_dir(&self.path)?;

        if entries.next().is_some() {
            match &self.data_protocol {
                DataProtocol::Archive => self.archive_protocol()?,
                DataProtocol::Panic => self.panic_protocol()?,
            }
        }

        Ok(())
    }

    // Archives old data in a timestamped directory
    fn archive_protocol(&mut self) -> Result<(), Error> {
        if !self.is_timestamped() {
            // Copy data to a timestamped directory
            let manifest = self.load_manifest()?;

            let timestamp = manifest.timestamp();

            // Create a temporary archive
            let staging_path = self.path.join(".tmp_archive");

            // Clean up any failed previous attempts
            if staging_path.exists() {
                fs::remove_dir_all(&staging_path)?;
            }

            fs::create_dir(&staging_path)?;

            // Snapshot existing files
            let entries: Vec<PathBuf> = fs::read_dir(&self.path)?
                .filter_map(|res| res.ok())
                .map(|e| e.path())
                .filter(|p| p != &staging_path)
                .collect();

            // Atomic renames into staging
            for path in entries {
                if let Some(name) = path.file_name() {
                    let destination = staging_path.join(name);
                    fs::rename(&path, &destination)?;
                }
            }

            // Atomic rename of the whole directory
            let archive_path = self.path.join(timestamp);

            fs::rename(&staging_path, &archive_path)?;
        }
        // Whether or not the data is archived, there will be timestamped
        // directories. To deal with this, we create a new timestamped
        // directory and append it to path, then make sure that
        // directory_structure is either Simple or Series.

        // Append the project path
        self.path = self.path.join(self.create_manifest().timestamp());

        // Create a new timestamp directory
        fs::create_dir(&self.path)?;

        // If neccessary, change directory structure
        if self.directory_structure == DirectoryStructure::Timestamped {
            self.directory_structure = DirectoryStructure::Simple;
        }

        Ok(())
    }

    // Always returns an error
    fn panic_protocol(&mut self) -> Result<(), Error> {
        Err(Error::new(
            ErrorKind::AlreadyExists,
            "Project directory already exists.",
        ))
    }

    // Creates the project directories
    fn initialize_directory_structure(&mut self) -> Result<(), Error> {
        // match self.directory_structure {
        //     DirectoryStructure::Series { n_runs } =>
        // }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn initialize_simple() {
        // Get a path to a new temporary directory
        let temp_dir =
            tempdir().expect("Failed to initialize a temporary directory.");
        let path = temp_dir.path().join("project");

        let mut project_manager = ProjectManager::new_simple(path);

        project_manager
            .initialize_project()
            .expect("Failed to initialize a simple project directory");
    }
}
