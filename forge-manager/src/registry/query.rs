// Copyright Andrey Zelenskiy, 2024-2026

use std::collections::HashMap;

use crate::run::{parameters::ParameterValue, Run, RunStatus};

/// Filters that can be applied to query the simulations
pub struct RunFilter {
    /// Completion status
    status: Option<RunStatus>,
    /// Parameter ranges
    parameter_range: Option<HashMap<String, RangeFilter>>,
}

/// Simple container for the parameter range
pub struct RangeFilter {
    min: Option<ParameterValue>,
    max: Option<ParameterValue>,
}

/// Method to check whether a run satisfies a filter condition
pub fn apply_filter(run: &Run, filter: &RunFilter) -> bool {
    let mut pass = true;

    if let Some(kind) = &filter.status {
        pass = run.status == *kind;
    }

    if let Some(range) = &filter.parameter_range {
        todo!()
    }

    pass
}
