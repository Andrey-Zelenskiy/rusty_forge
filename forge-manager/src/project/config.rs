// Copyright Andrey Zelenskiy, 2024-2026

use std::{
    fs::write,
    io,
    path::{Path, PathBuf},
};

use chrono::{DateTime, TimeDelta, Utc};

use config::Config;
use rayon::current_num_threads;

use serde::{Deserialize, Serialize};

use sysinfo::System;

use crate::{errors::ManagerError, ManagerResult};

pub const CURRENT_SCHEMA_VERSION: u32 = 0;

// Simulation project manifest
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ProjectManifest {
    // Project-specific metadata
    pub(super) metadata: ProjectMeta,
    // Environment-specific metadata
    environment: EnvironmentMeta,
    // Schema/version of project manifest
    // (current version is stored in CURRENT_SCHEMA_VERSION)
    pub(super) schema_version: u32,
}

impl ProjectManifest {
    /// Initializes a manifest data for a new project
    pub fn new(
        name: &str,
        author: &Option<String>,
        description: &Option<String>,
    ) -> Self {
        let mut sys = System::new_all();

        sys.refresh_all();

        Self {
            metadata: ProjectMeta {
                name: String::from(name),
                author: author.clone(),
                description: description.clone(),
                start_time: Utc::now(),
                end_time: None,
                duration: None,
                modification_times: None,
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
            schema_version: CURRENT_SCHEMA_VERSION,
        }
    }

    /// Write manifest data to manifest.toml file
    pub fn write<P: AsRef<Path>>(&self, path: P) -> ManagerResult<()> {
        let toml_string = toml::to_string_pretty(self)?;
        let manifest_path = path.as_ref().with_file_name("manifest.toml");

        write(&manifest_path, toml_string).map_err(|e| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "Can't initialize manifest.toml at {:?}: {e}",
                    &manifest_path
                ),
            )
            .into()
        })
    }

    /// Extracts manifest data from file if it matches the current schema
    pub fn load<P: AsRef<Path>>(path: P) -> ManagerResult<Self> {
        // Try to load manifest.toml
        let manifest_path = path.as_ref().with_file_name("manifest.toml");

        match Config::builder()
            .add_source(config::File::from(manifest_path.as_path()))
            .build()
        {
            Ok(config) => {
                // Before deserializing, check that the schema_version matches
                // CURRENT_SCHEMA_VERSION value
                let schema = config.get("schema_version").map_err(|_| {
                    ManagerError::SchemaNotFound(PathBuf::from(&manifest_path))
                })?;

                if schema != CURRENT_SCHEMA_VERSION {
                    Err(ManagerError::SchemaMismatch {
                        path: PathBuf::from(&manifest_path),
                        manifest_schema: schema,
                        current_schema: CURRENT_SCHEMA_VERSION,
                    })
                } else {
                    config.try_deserialize::<ProjectManifest>().map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Failed to deserialize manifest file: {e}"),
                        )
                        .into()
                    })
                }
            }
            Err(e) => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "manifest.toml not found in the project directory: {e}"
                ),
            )
            .into()),
        }
    }
}

/// State of the program at the start of the run
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub(super) struct ProjectMeta {
    // Project name
    pub(super) name: String,
    // Project author
    pub(super) author: Option<String>,
    // Project description
    pub(super) description: Option<String>,
    // Initialization time of the simulation project
    pub(super) start_time: DateTime<Utc>,
    // Completion time of the simulation project
    end_time: Option<DateTime<Utc>>,
    // Duration of the program from initialization to completion
    duration: Option<TimeDelta>,
    // Record of modifications
    modification_times: Option<Vec<DateTime<Utc>>>,
    // Hash of the git commit
    git_hash: Option<String>,
}

/// State of the program at the start of the run
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
struct EnvironmentMeta {
    // OS info
    os: String,
    // CPU type
    cpu: String,
    // Number of threads
    threads: usize,
}
