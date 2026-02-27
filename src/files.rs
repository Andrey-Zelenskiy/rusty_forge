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
    fs::read_dir,
    io::{Error, ErrorKind},
    path::PathBuf,
};

use regex::Regex;

use serde::Deserialize;

/// Project directory initializer
#[derive(Deserialize, Debug)]
pub struct ProjectManager {
    /// Path to the project directory
    path: PathBuf,
    /// Type of behaviour if project files already exist
    data_protocol: DataProtocol,
}

/// Protocol for dealing with files that already exist
#[derive(Deserialize, Debug)]
pub enum DataProtocol {
    /// Unschedules writing of the files that already exist in the directory
    Ignore,
    /// Interrupts the program if data with duplicate names already exist
    Panic,
    /// In recovery mode, appends data to existing files
    /// Note: if recovery is set to False, the protocol will be automatically
    /// switched to Ignore
    Recovery,
    /// Copies duplicate data to a timestamped folder
    Timestamped,
}

impl ProjectManager {
    /// Initializes simulation directory
    ///
    ///
    pub fn initialize_project(&mut self) -> Result<(), Error> {
        // Check if the directory already exists, in which case execute data
        // protocol
        if self.exists() {
            // Check if the program is allowed to proceed
            if self.data_protocol = DataProtocol::Panic {
                Err(Error::new(
                    ErrorKind::AlreadyExists,
                    "Project directory already exists.",
                ))
            } else if let Ok(entries) = read_dir(self.path) {
                // Check if the existing directory is empty
                if entries.next().is_none() {
                    Ok(())
                }
                // Check for timestamp protocols
                else if self.data_protocol = DataProtocol::Timestamped {
                    self.is_timestamped()
                    // let mut manifest_path = PathBuf::from(self.path);
                    // manifest_path.push("manifest.toml")
                    // self.path
                }
            }

            // Check the manifest file for date, complition status
            // If no manifest, check that the directory is empty
            // If empty, Ok(()), otherwise ErrorKind::DirectoryNotEmpty

            // Timestamped: check if timestamps exist. If yes, create a new one
            // and return Ok(()). Otherwise, first copy the old data into a
            // timestamped directory.

            // Recovery: if manifest status is Completed, return
            // ErrorKind::Other, "Simulation "
        }

        // Check if recovery
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
            if let Ok(entries) = read_dir(&self.path) {
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
}
