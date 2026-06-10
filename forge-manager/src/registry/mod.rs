// Copyright Andrey Zelenskiy, 2024-2026

use std::sync::Arc;

pub mod query;

use crate::{
    project::layout::ProjectLayout,
    registry::query::RunFilter,
    run::{Run, RunId, RunStatus},
    ManagerResult,
};

/// Structure that records simulation runs
pub struct Registry {
    layout: Arc<ProjectLayout>,
}

impl Registry {
    /// Initialize registry from project layout
    pub fn new(layout: Arc<ProjectLayout>) -> Self {
        Registry { layout }
    }

    /// Registers a new simulation run
    pub fn register(&self) -> ManagerResult<RunId> {
        todo!()
    }

    /// Returns the manager of a single simulation run
    pub fn get(&self, id: &RunId) -> ManagerResult<Run> {
        todo!()
    }

    /// Changes the state of the simulation run
    pub fn update_status(
        &self,
        id: &RunId,
        new_status: RunStatus,
    ) -> ManagerResult<()> {
        todo!()
    }

    /// List all registered runs
    pub fn list(&self) -> ManagerResult<Vec<Run>> {
        todo!()
    }

    /// List registered runs that fit some filtering criteria
    pub fn query(&self, filter: &RunFilter) -> ManagerResult<Vec<Run>> {
        todo!()
    }
}
