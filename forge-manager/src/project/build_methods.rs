// Copyright Andrey Zelenskiy, 2024-2026
use std::{io, path::PathBuf};

use clap::Parser;

use config::Config;

use serde::{Deserialize, Serialize};

use forge_builder::prelude::*;

use crate::{ManagerError, ManagerResult, ProjectManager};

/// Command line interface for loading data
#[derive(Debug, Default, Parser)]
#[command(
    name = "forge_project",
    about = "Initializes a new simulation project",
    long_about = None
)]
pub struct Cli {
    /// Path to config.toml
    config_file: Option<PathBuf>,
    // Options that override the config
    /// Project name
    name: Option<String>,
    /// Project path
    path: Option<PathBuf>,
    /// Author
    author: Option<String>,
    /// Project description
    description: Option<String>,
}

impl Cli {
    /// Initializes a new structure
    pub fn new(
        config_file: Option<PathBuf>,
        name: Option<String>,
        path: Option<PathBuf>,
        author: Option<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            config_file,
            name,
            path,
            author,
            description,
        }
    }

    /// Collects project information from the config file
    fn load_config(&self) -> ManagerResult<ProjectBuilder> {
        match &self.config_file {
            None => Ok(ProjectBuilder::default()),
            Some(path) => {
                if !path.exists() {
                    Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Config file not found: {}", path.display()),
                    )
                    .into())
                } else {
                    let config = Config::builder()
                        .add_source(config::File::from(path.as_path()))
                        .build()
                        .map_err(|e| {
                            io::Error::new(
                                io::ErrorKind::NotFound,
                                format!("Config file not found: {e}"),
                            )
                        })?;

                    config.get::<ProjectBuilder>("project").map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                            "Failed to deserialize project config file: {e}"
                        ),
                        )
                        .into()
                    })
                }
            }
        }
    }
}

/// Required data to initialize ProjectManager
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectBuilder {
    pub name: String,
    pub path: PathBuf,
    pub author: Option<String>,
    pub description: Option<String>,
}

impl ProjectBuilder {
    /// New initializer
    pub fn from_cli(cli: &Cli) -> ManagerResult<Self> {
        // Load information from the config
        let mut initializer = cli.load_config()?;

        // Update data from command line arguments
        if let Some(name) = &cli.name {
            initializer.name = name.clone();
        }

        if let Some(path) = &cli.path {
            initializer.path = path.clone();
        }

        if cli.author.is_some() {
            initializer.author = cli.author.clone();
        }

        if cli.description.is_some() {
            initializer.description = cli.description.clone();
        }

        Ok(initializer)
    }
}

impl Default for ProjectBuilder {
    fn default() -> Self {
        Self {
            name: "new_project".to_string(),
            path: PathBuf::from("./"),
            author: None,
            description: None,
        }
    }
}

impl BuilderMethods for ProjectBuilder {
    type Target = ProjectManager;

    fn build(&mut self) -> Result<Self::Target, BuildError> {
        match Self::Target::create(
            &self.name,
            &self.author,
            &self.description,
            &self.path,
        ) {
            Ok(manager) => Ok(manager),
            Err(ManagerError::ProjectAlreadyExists(path)) => {
                Self::Target::load(&path).map_err(|e| {
                    BuildError::IncompleteBuilderData {
                        reason: format!("{e}"),
                    }
                })
            }
            Err(e) => Err(BuildError::ConfigError {
                reason: format!("{e}"),
            }),
        }
    }

    fn from_target(target: &Self::Target) -> Self {
        Self {
            name: target.name().to_string(),
            path: target.path().to_path_buf(),
            author: target.author().clone(),
            description: target.description().clone(),
        }
    }
}

impl TargetFromBuilder for ProjectManager {
    type Builder = ProjectBuilder;
}
