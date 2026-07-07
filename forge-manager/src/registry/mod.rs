// Copyright Andrey Zelenskiy, 2024-2026

use std::fs;

pub mod query;

use chrono::Utc;

use crate::{
    project::layout::ProjectLayout,
    registry::query::RunFilter,
    run::{
        parameters::ParameterMap, Run, RunId, RunIdBuilder, RunState, RunStatus,
    },
    ManagerError, ManagerResult,
};

/// Structure that records simulation runs
#[derive(Default)]
pub struct Registry;

impl Registry {
    /// Registers a new simulation run
    pub fn register(
        &self,
        layout: &ProjectLayout,
        builder: RunIdBuilder,
        parameters: &ParameterMap,
    ) -> ManagerResult<RunId> {
        let run_id = builder.build();
        let run_dir = layout.run_dir(&run_id);

        // Create run directory
        fs::create_dir_all(&run_dir)?;

        // Write parameters to file
        parameters.write(&run_dir)?;

        // Write initial state
        let state = RunState {
            id: run_id.clone(),
            status: RunStatus::Pending,
            initialization_time: Utc::now(),
        };

        state.write(&run_dir)?;

        Ok(run_id)
    }

    /// Returns the manager of a single simulation run
    pub fn get(&self,layout: &ProjectLayout, id: &RunId) -> ManagerResult<Run> {
        let run_dir = layout.run_dir(id);

        if !run_dir.exists() {
            return Err(ManagerError::RunNotFound(id.to_string()));
        }

        let state = RunState::load(&run_dir)?;
        let parameters = ParameterMap::load(&run_dir)?;

        Ok(Run {
            id: id.clone(),
            parameters,
            status: state.status,
            initialization_time: state.initialization_time,
            run_dir,
        })
    }

    /// List all registered runs
    pub fn list(&self, layout: &ProjectLayout) -> ManagerResult<Vec<Run>> {
        let runs_dir = layout.runs_dir();

        // Return empty vector if the directory doesn't exist
        if !runs_dir.exists() {
            return Ok(vec![]);
        }

        let mut runs = Vec::new();

        for entry in fs::read_dir(&runs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() && let Ok(state) = RunState::load(&path) {
                    runs.push(self.get(layout, &state.id)?);
            }
        }

        Ok(runs)
    }

    /// List registered runs that fit some filtering criteria
    pub fn query(&self, layout: &ProjectLayout, filter: &RunFilter) -> ManagerResult<Vec<Run>> {
        let all_runs = self.list(layout)?;
        Ok(all_runs.into_iter().filter(|run| filter.matches(run)).collect())
    }

    /// Changes the state of the simulation run
    pub fn update_status(
        &self,
        layout: &ProjectLayout,
        id: &RunId,
        new_status_str: &str,
    ) -> ManagerResult<()> {
        let run = self.get(layout, id)?;

        // Validate state transition
        if !run.status.can_transition_to(new_status_str) {
            return Err(ManagerError::InvalidStateTransition{
                from:run.status.to_string(),
                to: new_status_str.to_string(),
            });
        }

        let new_status = match new_status_str {
            "Pending" => RunStatus::Pending,
            "Running" => RunStatus::Running { start_time: Utc::now() },
            "Completed" => RunStatus::Completed {
                start_time: run.status.start_time()
                    .ok_or(ManagerError::InvalidRunStatus(new_status_str.to_string()))?,
                end_time: Utc::now() },
            "Failed" => RunStatus::Failed {
                start_time: run.status.start_time()
                    .ok_or(ManagerError::InvalidRunStatus(new_status_str.to_string()))?,
                end_time: Utc::now() },
            _ => return Err(ManagerError::InvalidRunStatus(new_status_str.to_string()) )
        };

        let state = RunState {
            id: id.clone(),
            status: new_status,
            initialization_time: run.initialization_time,
        };

        state.write(&run.run_dir)
    }

}

#[cfg(test)]
mod tests {

    use tempfile::tempdir;

    use crate::run::parameters::ParameterValue;

    use super::*;

    #[test]
    fn test_register_and_get() {
        let temp = tempdir()
            .expect("Failed to initialize a temporary directory");
        let layout = ProjectLayout::new(temp.path().to_path_buf());
        layout.create_layout().expect("Failed to initialize project");

        let registry = Registry;

        let mut params = ParameterMap::new();
        params.insert("x".to_string(), ParameterValue::Float(1.0));

        let run_id = registry.register(&layout,RunIdBuilder::Timestamp, &params)
            .expect("Failed to register a simulation run");
        let run = registry.get(&layout, &run_id)
            .expect("Failed to get a simulation run from its id.");

        assert_eq!(run.status, RunStatus::Pending);
    }

    #[test]
    fn test_status_transitions() {
        let temp = tempdir()
            .expect("Failed to initialize a temporary directory");
        let layout = ProjectLayout::new(temp.path().to_path_buf());
        layout.create_layout().expect("Failed to initialize project");
        
        let registry = Registry;

        let params = ParameterMap::new();
        let run_id = registry.register(&layout, RunIdBuilder::Timestamp, &params)
            .expect("Failed to register a simulation run");

        // Pending -> Running
        registry.update_status(&layout, 
            &run_id,
            "Running"
        ).expect("Failed to update status from Pending to Running");
        
        // Running -> Completed
        registry.update_status(&layout, 
            &run_id,
            "Completed"
        ).expect("Failed to update status from Pending to Running");

        let run = registry.get(&layout, &run_id)
            .expect("Failed to recover simulation run.");

        assert!(matches!(run.status, RunStatus::Completed {..}));
    }

    #[test]
    fn test_invalid_transition() {
        let temp = tempdir()
            .expect("Failed to initialize a temporary directory");
        let layout = ProjectLayout::new(temp.path().to_path_buf());
        layout.create_layout().expect("Failed to initialize project");
        
        let registry = Registry;

        let params = ParameterMap::new();
        let run_id = registry.register(&layout, RunIdBuilder::Timestamp, &params)
            .expect("Failed to register a simulation run");

        // Skip to Completed directly (invalid)
        let result = registry.update_status(&layout, &run_id, "Completed");

        assert!(result.is_err());        
    }

    #[test]
    fn test_list_runs() {
        let temp = tempdir()
            .expect("Failed to initialize a temporary directory");
        let layout = ProjectLayout::new(temp.path().to_path_buf());
        layout.create_layout().expect("Failed to initialize project");
        
        let registry = Registry;

        let params1 = ParameterMap::new();
        let params2 = ParameterMap::new();
        
        registry.register(&layout, RunIdBuilder::Timestamp, &params1)
            .expect("Failed to register a simulation run");

        registry.register(&layout, RunIdBuilder::Random, &params2)
            .expect("Failed to register a simulation run");

        let runs = registry.list(&layout, )
            .expect("Failed to return a list of all simulation runs");

        assert_eq!(runs.len(), 2);
                
    }
}
