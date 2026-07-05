// Copyright Andrey Zelenskiy, 2024-2026
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

use forge_builder::prelude::BuilderMethods;

mod errors;

pub use crate::errors::{ManagerError, ParameterError};

/// Errors occuring during management of the simulation project
pub type ManagerResult<T> = Result<T, ManagerError>;

/// Errors occuring during parameter initialization/manipulation
pub type ParameterResult<T> = Result<T, ParameterError>;

mod project;
pub use project::ProjectManager;

mod run;

mod registry;

pub mod prelude {
    pub use super::project::{build_methods::Cli, ProjectManager};
}

pub struct Executor<S> {
    /// Project directory and simulation processes manager
    pub project_manager: ProjectManager,
    /// Simmulation to be executed
    pub simulation: PhantomData<S>,
    /// How simulation(s) are initialized
    execution_mode: ExecutionMode,
}

#[derive(Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Single simulation run
    Single,
    /// Multiple simulation runs with the identical parameters
    Multiple {
        n_runs: u32,
        method: ExecutionMethod,
    },
    /// Multiple simulation runs with distinct random parameters (if present)
    /// Note: if all parameters are initialized deterministically,
    /// ExecutionMode::Ensemble is the same as ExecutionMode::Multiple
    Ensemble {
        n_runs: u32,
        method: ExecutionMethod,
    },
    /// Sweep over parameters
    Sweep { method: ExecutionMethod },
}

#[derive(Serialize, Deserialize)]
pub enum ExecutionMethod {
    /// Serial run of scheduled simulations
    LocalSequential,
    /// Parallel run of scheduled simulations
    LocalParallel,
    /// Initialization of a slurm job
    Slurm(SlurmConfig),
}

#[derive(Serialize, Deserialize)]
pub struct SlurmConfig {
    pub partition: String,
    pub time: String,
    pub ntasks: usize,
    pub extra_directives: Vec<String>,
}

pub trait Simulation: BuilderMethods {
    /// Run the simulation according to the specified parameters.
    /// This method assumes that the configuration file of the simulation is
    /// located in the current directory, i.e. at ./config.toml.
    fn execute() -> ManagerResult<()> {
        Ok(())
    }
}
