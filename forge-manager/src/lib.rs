// Copyright Andrey Zelenskiy, 2024-2026

mod errors;
pub use crate::errors::{ManagerError, ParameterError};

/// Errors occuring during management of the simulation project
pub type ManagerResult<T> = Result<T, ManagerError>;

/// Errors occuring during parameter initialization/manipulation
pub type ParameterResult<T> = Result<T, ParameterError>;

pub mod project;

pub mod run;

pub mod registry;
