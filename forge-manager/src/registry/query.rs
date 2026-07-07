// Copyright Andrey Zelenskiy, 2024-2026

use std::collections::HashMap;

use crate::run::{parameters::ParameterValue, Run, RunStatus};

/// Filters that can be applied to query the simulations
#[derive(Default)]
pub struct RunFilter {
    /// Completion status
    status: Option<RunStatus>,
    /// Parameter ranges
    parameter_ranges: Option<HashMap<String, RangeFilter>>,
}

impl RunFilter {
    /// Initializes new RunFilter structure
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_status(mut self, status: RunStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_parameter_ranges(
        mut self,
        key:String,
        range: RangeFilter,
    ) -> Self {
        match &mut self.parameter_ranges {
            None => {
                let mut ranges = HashMap::new();
                ranges.insert(key, range);
                self.parameter_ranges = Some(ranges);
            }
            Some(ranges) => {
                ranges.insert(key, range);
                }
        }
        self
    }

    pub fn matches(&self, run: &Run) -> bool {
        // Check status
        if let Some(status) = &self.status && run.status != *status {
                return false;
            
        }

        // Check parameter range
        if let Some(ranges) = &self.parameter_ranges {
            for (key, range) in ranges {
                match run.parameters.get(key) {
                    None => return false,
                    Some(value) => {
                        if !range.contains(value){
                            return false;
                        }
                    }
                }
            }
        }

        true
    
    }
}

/// Simple container for the parameter range
#[derive(Default)]
pub struct RangeFilter {
    min: Option<f64>,
    max: Option<f64>,
}

impl RangeFilter {
    /// initializes new RangeFilter structure
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn with_max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn contains(&self, value: &ParameterValue) -> bool {
        let float_value = match value {
            ParameterValue::Float(val) => *val,
            ParameterValue::Int(val) => *val as f64,
            ParameterValue::Uint(val) => *val as f64,
            _ => return false,
        };

        let passes_min = self.min.is_none_or(|m| m <= float_value);
        let passes_max = self.max.is_none_or(|m| m >= float_value);

        passes_min && passes_max
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use tempfile::{ tempdir};

    use crate::run::{RunId, parameters::ParameterMap};

    use super::*;

    #[test]
    fn test_range_float() {
        let range = RangeFilter::new().with_min(-100.0).with_max(100.0);

        assert!(range.contains(&ParameterValue::Float(-50.0)));
        assert!(range.contains(&ParameterValue::Float(50.0)));
        assert!(!range.contains(&ParameterValue::Float(-2000.0)));
        assert!(!range.contains(&ParameterValue::Float(2000.0)));
    }

    #[test]
    fn test_range_int(){
        let range = RangeFilter::new().with_min(-100.0).with_max(100.0);

        assert!(range.contains(&ParameterValue::Int(-50)));
        assert!(range.contains(&ParameterValue::Int(50)));
        assert!(!range.contains(&ParameterValue::Int(-2000)));
        assert!(!range.contains(&ParameterValue::Int(2000)));
    }

    #[test]
    fn test_range_uint(){
        let range = RangeFilter::new().with_min(100.0).with_max(1000.0);

        assert!(range.contains(&ParameterValue::Uint(500)));
        assert!(!range.contains(&ParameterValue::Uint(50)));
        assert!(!range.contains(&ParameterValue::Uint(2000)));
    }

    #[test]
    fn test_run_filter_by_status(){
        let mut params = ParameterMap::new();
        params.insert("x".to_string(), ParameterValue::Float(1.0));
        
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        let run = Run::new(RunId::from_timestamp(), params, path);

        let filter_pending = RunFilter::new().with_status(RunStatus::Pending);
        assert!(filter_pending.matches(&run));

        let filter_done = RunFilter::new().with_status(RunStatus::Completed { start_time: Utc::now(), end_time: Utc::now() });
        assert!(!filter_done.matches(&run));
    }

    #[test]
    fn test_run_filter_by_range(){
        let mut params = ParameterMap::new();
        params.insert("x".to_string(), ParameterValue::Float(1.0));
        
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        let run = Run::new(RunId::from_timestamp(), params, path);

        let range = RangeFilter::new().with_min(0.0).with_max(2.0);

        let filter = RunFilter::new()
            .with_parameter_ranges("x".to_string(), range);

        assert!(filter.matches(&run));
    }
    

    #[test]
    fn test_missing_parameter(){
        let params = ParameterMap::new();
        
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        let run = Run::new(RunId::from_timestamp(), params, path);

        let range = RangeFilter::new().with_min(0.0).with_max(2.0);

        let filter = RunFilter::new()
            .with_parameter_ranges("x".to_string(), range);

        assert!(!filter.matches(&run));
        
    }
}
