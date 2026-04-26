// Copyright Andrey Zelenskiy, 2024-2026

use std::path::PathBuf;

use thiserror::Error;

/// Errors that occur during initialization and execution of simulations
#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Error in the IO operation: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization failed: {0}")]
    Serialization(#[from] toml::ser::Error),
    #[error(
        "Unable to load project manifest: missing schema_version value in {0}"
    )]
    SchemaNotFound(PathBuf),
    #[error(
        "Unable to load project manifest: \
        schema_version ({manifest_schema}) in {path} is different from the \
        current version ({current_schema})"
    )]
    SchemaMismatch {
        path: PathBuf,
        manifest_schema: u32,
        current_schema: u32,
    },
    #[error("Project directory already exists at {0}")]
    ProjectAlreadyExists(PathBuf),
    #[error("Cannot find project directory at {0}")]
    ProjectNotFound(PathBuf),
}

/// Errors that occur during initialization of model parameters
#[derive(Error, Debug)]
pub enum ParameterError {
    #[error("Serialization failed: {0}")]
    Serialization(#[from] toml::ser::Error),
    #[error("Missing required key {0} during parameter initialization")]
    MissingKey(String),
    #[error(
        "Incorrect value type during initialization of {name}: \
        expected {expected}, got {current}"
    )]
    IncorrectValueType {
        name: String,
        expected: String,
        current: String,
    },
}
