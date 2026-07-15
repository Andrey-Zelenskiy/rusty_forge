// Copyright Andrey Zelenskiy, 2024-2026
use clap::Parser;

use std::{
    error,
    path::{Path, PathBuf},
};

use forge_builder::prelude::TargetFromBuilder;

mod errors;
pub use errors::{ManagerError, ParameterError};

/// Errors occuring during management of the simulation project
pub type ManagerResult<T> = Result<T, ManagerError>;

/// Errors occuring during parameter initialization/manipulation
pub type ParameterResult<T> = Result<T, ParameterError>;

mod project;

mod run;
use run::parameters::ParameterMap;

use crate::executor::SimulationConfig;

mod executor;

pub mod prelude {
    pub use super::project::{
        build_methods::{ProjectBuilder, ProjectCli},
        ProjectManager,
    };
    pub use super::run::{parameters::ParameterMap, RunId, RunIdBuilder};
    pub use super::Simulation;
}

/// Command line interface for loading config path
#[derive(Debug, Default, Parser)]
struct ConfigCli {
    config_path: Option<PathBuf>,
}

impl ConfigCli {
    fn load_path(&self) -> PathBuf {
        match &self.config_path {
            Some(path) => path.clone(),
            None => PathBuf::from("./config.toml"),
        }
    }
}

/// Interface for running simulations with specified execution parameters
pub trait Simulation: Sized + TargetFromBuilder + Clone + Sync {
    /// Simulation error type
    type Error: error::Error + Send + Sync + 'static;

    /// Set the output path of the simulation
    fn set_path(&mut self, path: &Path) -> Result<(), Self::Error>;

    /// Returns ParameterMap for model parameters
    fn get_parameter_map(&self) -> Result<ParameterMap, Self::Error>;

    /// Update model parameters from ParameterMap entry
    fn upadate_parameters(
        &mut self,
        parameters: &ParameterMap,
    ) -> Result<(), Self::Error>;

    /// Run the simulation with the current set of parameters
    fn run(&mut self) -> Result<(), Self::Error>;

    /// Convert Self::Error to ManagerError
    fn into_err(e: Self::Error) -> ManagerError {
        ManagerError::SimulationError(e.to_string())
    }

    /// Run the simulation according to the specified parameters.
    /// This method assumes that the configuration file of the simulation is
    /// located in the current directory, i.e. at ./config.toml.
    fn execute() -> ManagerResult<()> {
        // Load config path
        let config_path = ConfigCli::parse().load_path();

        // Check if the config exists, if not write a default one and exit
        if !config_path.exists() {
            return SimulationConfig::<Self::Builder>::default()
                .write(&config_path);
        }

        // Load the config
        let config = SimulationConfig::<Self::Builder>::load(&config_path)?;

        // Run the simulation
        config.execute()
    }
}
