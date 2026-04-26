// Copyright Andrey Zelenskiy, 2024-2026

use std::fmt;

use chrono::{DateTime, Utc};

use rand::random;

pub mod parameters;

pub struct Run {}

/// ID of a simulation run
pub struct RunID {
    /// Timestamp
    timestamp: String,
    /// Additional suffix (in case of multiple consequtive runs)
    suffix: Option<String>,
}

impl RunID {
    /// Generate new RunID with no suffix
    pub fn from_timestamp() -> Self {
        Self {
            timestamp: Utc::now().format("%Y-%m-%d").to_string(),
            suffix: None,
        }
    }

    /// Generate new RunID with random suffix
    pub fn from_random() -> Self {
        Self {
            timestamp: Utc::now().format("%Y-%m-%d").to_string(),
            suffix: Some(format!("{:08x}", random::<u32>())),
        }
    }

    /// Generate new RunID with a single numerical index
    pub fn from_index(index: u32) -> Self {
        Self {
            timestamp: Utc::now().format("%Y-%m-%d").to_string(),
            suffix: Some(format!("{:04}", index)),
        }
    }

    /// Generate new RunID with a pair of numerical indices
    pub fn from_index_pair(index1: u32, index2: u32) -> Self {
        Self {
            timestamp: Utc::now().format("%Y-%m-%d").to_string(),
            suffix: Some(format!("{:04}_{:04}", index1, index2)),
        }
    }
}

impl fmt::Display for RunID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.suffix {
            Some(s) => write!(f, "{}_{}", self.timestamp, s),
            None => write!(f, "{}", self.timestamp),
        }
    }
}

/// Status state of a simulation run
pub enum RunStatus {
    /// Run is scheduled, but has not been started by the scheduler yet
    Pending,
    /// Run has been started, and hasn't terminated yet
    Running {
        /// Initialization time of the simulation run
        start_time: DateTime<Utc>,
    },
    /// Run has been completed successfully
    Completed {
        /// Initialization time of the simulation run
        start_time: DateTime<Utc>,
        /// Completion time of the simulation run
        end_time: DateTime<Utc>,
    },
    /// Run was ended before the completion of the simulation
    /// (likely due to error)
    Failed {
        /// Initialization time of the simulation run
        start_time: DateTime<Utc>,
        /// Completion time of the simulation run
        end_time: DateTime<Utc>,
        /// Reason for preliminary termination of the run
        reason: String,
    },
}
