// Copyright Andrey Zelenskiy, 2024-2026

//! # Module for creating a metadata of the project
//!
//! The metadata is used to store the instance of the system at the time of
//! the simulation for robust reproducibility of the data, as well as for
//! tracking the completion status of the simulation runs.
//!
//! ## Example
//!
//! ```rust
//! use rusty_forge::project_setup::ProjectManager;
//! ```

use std::{fs::write, io, path::Path};

use chrono::{DateTime, TimeDelta, Utc};

use rayon::current_num_threads;
use serde::{Deserialize, Serialize};
use sysinfo::System;

/// Holds the metadata of the simulation
#[derive(Deserialize, Serialize, Debug)]
pub struct ProjectManifest {
    /// State of the program at the start of the run
    metadata: SimulationMeta,
    /// Information about hardware architecture
    environment: EnvironmentMeta,
}

impl ProjectManifest {
    /// Initialize metadata
    pub fn new() -> Self {
        let mut sys = System::new_all();

        sys.refresh_all();

        Self {
            metadata: SimulationMeta {
                last_checkpoint: None,
                start_time: Utc::now(),
                end_time: None,
                duration: None,
                git_hash: option_env!("GIT_HASH").map(|s| s.to_string()),
            },
            environment: EnvironmentMeta {
                os: System::name().unwrap_or_else(|| "Unknown OS".to_string()),
                cpu: sys
                    .cpus()
                    .first()
                    .map(|c| c.brand().to_string())
                    .unwrap_or_else(|| "Unknown CPU".to_string()),
                threads: current_num_threads(),
            },
        }
    }

    /// Get timestamp of simulation start as yyyy-mm-dd string
    pub fn timestamp(&self) -> String {
        self.metadata.start_time.format("%Y-%m-%d").to_string()
    }

    /// Return the index of the last checkpoint
    #[allow(dead_code)]
    pub fn last_checkpoint(&self) -> &Option<usize> {
        &self.metadata.last_checkpoint
    }

    /// Update checpoint index
    #[allow(dead_code)]
    pub fn update_checkpoint(&mut self) {
        match &self.metadata.last_checkpoint {
            None => self.metadata.last_checkpoint = Some(0),
            Some(index) => {
                self.metadata.last_checkpoint = Some(*index + 1);
            }
        }
    }

    /// Check if the program was completed without errors
    #[allow(dead_code)]
    pub fn is_completed(&self) -> bool {
        self.metadata.end_time.is_some()
    }

    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        let toml_string = toml::to_string_pretty(self)
            .expect("Failed to convert manifest to toml string.");
        let manifest_path = path.as_ref().with_file_name("manifest.toml");

        write(&manifest_path, toml_string).map_err(|e| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "Can't initialize manifest.toml at {:?}: {e}",
                    &manifest_path
                ),
            )
        })
    }
}

/// State of the program at the start of the run
#[derive(Deserialize, Serialize, Debug)]
struct SimulationMeta {
    // Index of the last checkpoint
    last_checkpoint: Option<usize>,
    // Start time of the simulation
    start_time: DateTime<Utc>,
    // End time of the simulation
    end_time: Option<DateTime<Utc>>,
    // Duration of the program
    duration: Option<TimeDelta>,
    // Hash of the git commit
    git_hash: Option<String>,
}

/// State of the program at the start of the run
#[derive(Deserialize, Serialize, Debug)]
struct EnvironmentMeta {
    // OS info
    os: String,
    // CPU type
    cpu: String,
    // Number of threads
    threads: usize,
}
