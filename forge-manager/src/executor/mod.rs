// Copyright Andrey Zelenskiy, 2024-2026

use std::{
    fs, io,
    path::{self, Path},
    process::Command,
};

use config::{Config, File, FileFormat};
use forge_builder::prelude::BuilderMethods;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    project::{build_methods::ProjectBuilder, ProjectManager},
    run::{parameters::ParameterMap, RunId, RunIdBuilder},
    ManagerError, ManagerResult, Simulation,
};

mod slurm;
use slurm::SlurmConfig;

/// Strategy for executing multiple simulations
#[derive(Serialize, Deserialize, Clone)]
pub enum ExecutionMethod {
    /// Serial run of scheduled simulations
    LocalSequential,
    /// Parallel run of scheduled simulations
    LocalParallel,
    /// Initialization of a slurm job
    Slurm(SlurmConfig),
}

impl ExecutionMethod {
    /// Run multiple simulations according to the specified option
    pub fn run<F1, F2, I>(
        &self,
        single_run: F1,
        slurm_submit: F2,
        iterator: I,
    ) -> ManagerResult<()>
    where
        F1: Fn(u32) -> ManagerResult<()> + Send + Sync,
        F2: Fn(u32, &SlurmConfig) -> ManagerResult<()> + Send + Sync,
        I: IntoIterator<Item = u32> + IntoParallelIterator<Item = u32>,
    {
        let err_vec: Vec<String> = match self {
            Self::LocalSequential => iterator
                .into_iter()
                .filter_map(|index| single_run(index).err())
                .map(|error| error.to_string())
                .collect(),
            Self::LocalParallel => iterator
                .into_par_iter()
                .filter_map(|index| single_run(index).err())
                .map(|error| error.to_string())
                .collect(),
            Self::Slurm(slurm_config) => iterator
                .into_iter()
                .filter_map(|index| slurm_submit(index, slurm_config).err())
                .map(|error| error.to_string())
                .collect(),
        };

        match err_vec.is_empty() {
            true => Ok(()),
            false => Err(ManagerError::MultipleErrors(err_vec.join("\n\n"))),
        }
    }
}

/// Type of simulation
#[derive(Default, Serialize, Deserialize, Clone)]
pub enum ExecutionMode {
    /// Single simulation run
    #[default]
    Single,
    /// Generate and submit a slurm job
    SlurmJob {
        slurm_config: SlurmConfig,
        execute: Option<RunId>,
    },
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

/// Generic configuration for numerical simulations
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct SimulationConfig<B>
where
    B: BuilderMethods<Target: Simulation>,
{
    /// Project directory and simulation processes manager
    pub project: ProjectBuilder,
    /// How simulation(s) are initialized
    pub execution_mode: ExecutionMode,
    /// Builder of the simmulation to be executed
    #[serde(bound(deserialize = "B: BuilderMethods"))]
    pub builder: B,
}

impl<B> SimulationConfig<B>
where
    B: BuilderMethods<Target: Simulation>,
{
    /// Write simulation config to toml file
    pub fn write(&self, path: &Path) -> ManagerResult<()> {
        let config_path = path.with_file_name("config.toml");
        let config_toml = toml::to_string_pretty(self)?;

        fs::write(config_path, config_toml).map_err(Into::into)
    }

    /// Read config from file
    pub fn load(config_path: &Path) -> ManagerResult<Self> {
        let config_path_str = config_path.to_str().ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unable to convert config Path to str",
        ))?;

        Config::builder()
            .add_source(File::new(config_path_str, FileFormat::Toml))
            .build()?
            .try_deserialize::<Self>()
            .map_err(Into::into)
    }

    /// Run the simulation according to the config
    pub fn execute(&self) -> ManagerResult<()> {
        match &self.execution_mode {
            ExecutionMode::Single => self.execute_single(),
            ExecutionMode::SlurmJob {
                slurm_config,
                execute,
            } => self.execute_slurm_job(slurm_config, execute),
            ExecutionMode::Multiple { n_runs, method } => {
                self.execute_multiple(n_runs, method)
            }
            ExecutionMode::Ensemble { n_runs, method } => {
                self.execute_ensemble(n_runs, method)
            }
            ExecutionMode::Sweep { method } => self.execute_sweep(method),
        }
    }

    // Helper functions

    /// Executor for Self::Single variant
    fn execute_single(&self) -> ManagerResult<()> {
        // Build the project directory
        let project_manager = self.project.clone().build()?;

        // Initialize the simulation
        let mut simulation = self.builder.clone().build()?;

        // Create a ParameterMap
        let parameters = simulation
            .get_parameter_map()
            .map_err(B::Target::into_err)?;

        // Create simulation run directory and register the run
        let run_id = project_manager
            .register_run(RunIdBuilder::Timestamp, &parameters)?;

        // Save a copy of the config at the initialized directory
        self.write(&project_manager.get_run_dir(&run_id))?;

        Self::single_run(&project_manager, &run_id, &mut simulation)
    }

    /// Executor for Self::Single variant
    fn execute_slurm_job(
        &self,
        slurm_config: &SlurmConfig,
        execute: &Option<RunId>,
    ) -> ManagerResult<()> {
        // Build the project directory
        let project_manager = self.project.clone().build()?;

        // Initialize the simulation
        let mut simulation = self.builder.clone().build()?;

        // Create a ParameterMap
        let parameters = simulation
            .get_parameter_map()
            .map_err(B::Target::into_err)?;

        match execute {
            None => self.slurm_submit(
                &project_manager,
                RunIdBuilder::Timestamp,
                &parameters,
                slurm_config,
            ),
            Some(run_id) => {
                Self::single_run(&project_manager, run_id, &mut simulation)
            }
        }
    }

    /// Executor for Self::Multiple variant
    fn execute_multiple(
        &self,
        n_runs: &u32,
        method: &ExecutionMethod,
    ) -> ManagerResult<()> {
        // Build the project directory
        let project_manager = self.project.clone().build()?;

        // Initialize the simulation
        let simulation = self.builder.clone().build()?;

        // Create a ParameterMap
        let parameters = simulation
            .get_parameter_map()
            .map_err(B::Target::into_err)?;

        // Define runner functions
        let single_run = |_: u32| {
            // Create simulation run directory and register the run
            let run_id = project_manager
                .register_run(RunIdBuilder::Random, &parameters)?;

            // Save a copy of the config at the initialized directory
            self.write(&project_manager.get_run_dir(&run_id))?;

            Self::single_run(&project_manager, &run_id, &mut simulation.clone())
        };

        let slurm_submit = |_: u32, slurm: &SlurmConfig| {
            self.slurm_submit(
                &project_manager,
                RunIdBuilder::Random,
                &parameters,
                slurm,
            )
        };

        method.run(single_run, slurm_submit, 0..*n_runs)
    }

    /// Executor for Self::Ensemble variant
    fn execute_ensemble(
        &self,
        n_runs: &u32,
        method: &ExecutionMethod,
    ) -> ManagerResult<()> {
        // Build the project directory
        let project_manager = self.project.clone().build()?;

        // Define runner functions
        let single_run = |_: u32| {
            // Initialize the simulation
            let simulation = self.clone().builder.build()?;

            // Create a ParameterMap
            let parameters = simulation
                .get_parameter_map()
                .map_err(B::Target::into_err)?;

            // Create simulation run directory and register the run
            let run_id = project_manager.register_run(
                RunIdBuilder::Parameters(parameters.hash()),
                &parameters,
            )?;

            // Save a copy of the config at the initialized directory
            self.write(&project_manager.get_run_dir(&run_id))?;

            Self::single_run(&project_manager, &run_id, &mut simulation.clone())
        };

        let slurm_submit = |_: u32, slurm: &SlurmConfig| {
            // Initialize the simulation
            let simulation = self.clone().builder.build()?;

            // Create a ParameterMap
            let parameters = simulation
                .get_parameter_map()
                .map_err(B::Target::into_err)?;

            self.slurm_submit(
                &project_manager,
                RunIdBuilder::Random,
                &parameters,
                slurm,
            )
        };

        method.run(single_run, slurm_submit, 0..*n_runs)
    }

    /// Executor for Self::Sweep variant
    fn execute_sweep(&self, method: &ExecutionMethod) -> ManagerResult<()> {
        // Build the project directory
        let project_manager = self.project.clone().build()?;

        todo!()
    }

    /// Perform a single simulation run
    fn single_run(
        project_manager: &ProjectManager,
        run_id: &RunId,
        simulation: &mut B::Target,
    ) -> ManagerResult<()> {
        // Set the global path for simulation output
        simulation
            .set_path(&project_manager.get_run_dir(run_id))
            .map_err(B::Target::into_err)?;

        // Run the simulation
        project_manager.update_run_status(run_id, "Running")?;

        match simulation.run() {
            Ok(()) => {
                project_manager.update_run_status(run_id, "Completed")?;
            }
            Err(e) => {
                project_manager.update_run_status(run_id, "Failed")?;
                project_manager.log_error(e, Some(run_id))?;
            }
        }

        Ok(())
    }

    /// Submits the job to slurm
    fn slurm_submit(
        &self,
        project_manager: &ProjectManager,
        id_builder: RunIdBuilder,
        parameters: &ParameterMap,
        slurm: &SlurmConfig,
    ) -> ManagerResult<()> {
        // Create simulation run directory and register run
        let run_id = project_manager.register_run(id_builder, parameters)?;

        let run_dir = project_manager.get_run_dir(&run_id);

        // Create a slurm script
        let slurm_name = format!("{}.sbatch", &run_id);
        let slurm_path = run_dir.with_file_name(slurm_name);
        let config_path = run_dir.with_file_name("config.toml");
        slurm.write_script(
            &slurm_path,
            &config_path,
            project_manager.name(),
        )?;

        // Save a copy of the config at the initialized directory
        let slurm_config = slurm.clone();
        let execute = Some(run_id);

        let mut config_run = self.clone();

        // Modify project path to be absolute
        if config_run.project.path.is_absolute() {
            config_run.project.path = path::absolute(&config_run.project.path)?;
        }

        // Change execution mode
        config_run.execution_mode = ExecutionMode::SlurmJob {
            slurm_config,
            execute,
        };

        config_run.write(&run_dir)?;

        // Submit the job to slurm
        Command::new("sbatch").arg(slurm_path).output()?;

        Ok(())
    }
}
