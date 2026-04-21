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
//! use rusty_forge::ProjectManager;
//! ```

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use config::Config;

use regex::Regex;

use serde::Deserialize;

use crate::FileManager;

mod manifest;
use manifest::ProjectManifest;

/// Structure that manages project directory throughout simulation
#[derive(Deserialize, Debug)]
pub struct ProjectManager {
    /// Path to the project directory
    path: PathBuf,
    /// Recovery mode option for restarting a simulation from the last
    /// checkpoint
    #[serde(default)]
    recovery_mode: Option<usize>,
    /// Option to timestamp the name of the project directory with /yyy-mm-dd/
    #[serde(default)]
    timestamped: bool,
    /// Type of behaviour if project path already exists
    data_protocol: DataProtocol,
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
    /// Simple project structure: new directory without timestamps,
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
            recovery_mode: None,
            timestamped: false,
            data_protocol: DataProtocol::Panic,
        }
    }

    /// Initializes project, depending on the recovery option
    ///
    ///
    pub fn initialize_project(&mut self) -> Result<(), io::Error> {
        // Choose initialization based on the recovery option
        if self.is_recovery() {
            self.initialize_recovery()
        } else {
            self.initialize_new()
        }
    }

    /// Initializes log files in the project directory
    pub fn initialize_logs(
        &mut self,
        file_managers: &mut Vec<&mut FileManager>,
    ) -> Result<(), io::Error> {
        file_managers.iter_mut().try_for_each(|f| {
            // Add project directory to all files
            f.set_project_dir(self.path().join("logs"));

            f.initialize_open()
        })?;

        Ok(())
    }

    /// Initializes checkpoint files in the project directory
    pub fn initialize_checkpoints(
        &mut self,
        file_managers: &mut Vec<&mut FileManager>,
    ) -> Result<(), io::Error> {
        file_managers.iter_mut().try_for_each(|f| {
            // Add project directory to all files
            f.set_project_dir(self.path().join("checkpoints"));

            f.initialize_open()
        })?;

        Ok(())
    }

    /// Initializes output files in the project directory
    pub fn initialize_data(
        &mut self,
        file_managers: &mut Vec<&mut FileManager>,
    ) -> Result<(), io::Error> {
        file_managers.iter_mut().try_for_each(|f| {
            // Add project directory to all files
            f.set_project_dir(self.path().join("data"));

            f.initialize_append()
        })?;

        Ok(())
    }

    /// Returns reference to project's path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns true if the project manager is in the recovery mode
    pub fn is_recovery(&self) -> bool {
        self.recovery_mode.is_some()
    }

    // Checks if the project directory already exists
    fn exists(&self) -> bool {
        self.path.is_dir()
    }

    // Checks if the project directory is timestamped
    fn exists_and_timestamped(&self) -> bool {
        let mut timestamped = false;

        if self.exists() {
            // Regex for the timestamps
            let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();

            // Get items of the directory
            if let Ok(entries) = fs::read_dir(self.path()) {
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
    fn load_manifest(&self) -> Result<ProjectManifest, io::Error> {
        match Config::builder()
            .add_source(config::File::from(self.path.join("manifest.toml")))
            .build()
        {
            Ok(config) => {
                config.try_deserialize::<ProjectManifest>().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to deserialize manifest file: {e}"),
                    )
                })
            }
            Err(e) => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "manifest.toml not found in the project directory: {e}"
                ),
            )),
        }
    }

    // Prepare for a recovery of an existing project
    fn initialize_recovery(&mut self) -> Result<(), io::Error> {
        // Ensure that the directory exists
        if !self.exists() {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Project directory scheduled for recovery does \
                        not exist: {:#?}",
                    self.path
                ),
            ))?;
        }

        // Load manifest file
        //let manifest = self.load_manifest()?;

        // Determine the index of the last saved checkpoint
        // Locate the checkpoint
        // Create a new data directory
        // Return Config
        Ok(())
    }

    // Creates a new project, following data safety protocols
    fn initialize_new(&mut self) -> Result<(), io::Error> {
        // Check if the directory already exists, in which case execute data
        // protocol
        if self.exists() {
            self.execute_data_protocol()?;
        }

        // Create project directory and manifest file
        self.initialize_directory_structure()?;

        Ok(())
    }

    // Method for safely dealing with the existing data
    fn execute_data_protocol(&mut self) -> Result<(), io::Error> {
        // Check if the existing directory is empty
        // (if it is, no need for data protocols)
        let mut entries = fs::read_dir(self.path())?;

        if entries.next().is_some() {
            match &self.data_protocol {
                DataProtocol::Archive => self.archive_protocol()?,
                DataProtocol::Panic => self.panic_protocol()?,
            }
        }

        Ok(())
    }

    // Archives old data in a timestamped directory
    fn archive_protocol(&mut self) -> Result<(), io::Error> {
        if !self.exists_and_timestamped() {
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
            let entries: Vec<PathBuf> = fs::read_dir(self.path())?
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
        // directory and append it to path

        // Append the project path
        self.path = self.path.join(self.create_manifest().timestamp());

        // Create a new timestamp directory
        fs::create_dir(self.path())?;

        self.timestamped = false;

        Ok(())
    }

    // Always returns an error
    fn panic_protocol(&mut self) -> Result<(), io::Error> {
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Project directory already exists.",
        ))
    }

    // Creates the project directories
    fn initialize_directory_structure(&mut self) -> Result<(), io::Error> {
        // Create a manifest file
        let manifest = self.create_manifest();

        // Depending on the option, create a timestamped directory
        if self.timestamped {
            self.path = self.path.join(manifest.timestamp());

            fs::create_dir_all(self.path())?;
        }

        // Save the manifest file
        manifest.write(self.path())?;

        // Create directory for error and debug messages
        fs::create_dir_all(self.path().join("logs"))?;

        // Create directory for simulation checkpoints
        fs::create_dir_all(self.path().join("checkpoints"))?;

        // Create directory for simulation output
        fs::create_dir_all(self.path().join("data"))?;

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
