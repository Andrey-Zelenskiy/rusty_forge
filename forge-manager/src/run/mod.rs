// Copyright Andrey Zelenskiy, 2024-2026

use std::{fmt, path::PathBuf};

use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

use crate::run::parameters::ParameterMap;

pub mod parameters;

/// Manager for a single simulation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    /// ID of the simulation run
    pub id: RunId,
    /// Config for simulation parameters
    pub parameters: ParameterMap,
    /// Current status of the simulation
    pub status: RunStatus,
    /// Timestamp of run initialization
    pub initialization_time: DateTime<Utc>,
    /// Output directory for the simulation
    pub run_dir: PathBuf,
}

impl Run {
    /// Initialize a new simulation run
    pub fn new(id: RunId, parameters: ParameterMap, run_dir: PathBuf) -> Self {
        Self {
            id,
            parameters,
            status: RunStatus::Pending,
            initialization_time: Utc::now(),
            run_dir,
        }
    }
}

/// Information that gets serialized by the Registry
pub struct RunState {
    /// ID of the simulation run
    pub id: RunId,
    /// Current status of the simulation
    pub status: RunStatus,
    /// Timestamp of run initialization
    pub initialization_time: DateTime<Utc>,
}

impl From<&Run> for RunState {
    fn from(value: &Run) -> Self {
        Self {
            id: value.id.clone(),
            status: value.status.clone(),
            initialization_time: value.initialization_time,
        }
    }
}

/// ID of a simulation run
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct RunId {
    /// Timestamp
    timestamp: DateTime<Utc>,
    /// Additional suffix (in case of multiple consequtive runs)
    suffix: Option<Suffix>,
}

impl RunId {
    /// Generate new RunId with random suffix
    pub fn from_random() -> Self {
        Self {
            timestamp: Utc::now(),
            suffix: Some(Suffix::Random(rand::random())),
        }
    }

    /// Generate new RunId with no suffix
    pub fn from_timestamp() -> Self {
        Self {
            timestamp: Utc::now(),
            suffix: None,
        }
    }

    /// Generate new RunId from timestamp with parameter hash suffix
    pub fn from_parameters(params: &ParameterMap) -> Self {
        Self {
            timestamp: Utc::now(),
            suffix: Some(Suffix::Hash(format!("{:08x}", params.hash()))),
        }
    }

    /// Generate new RunId with a single numerical index
    pub fn from_index(index: u32) -> Self {
        Self {
            timestamp: Utc::now(),
            suffix: Some(Suffix::Index(index)),
        }
    }

    /// Generate new RunId with a pair of numerical indices
    pub fn from_index_set(indices: Vec<u32>) -> Self {
        Self {
            timestamp: Utc::now(),
            suffix: Some(Suffix::IndexSet(indices)),
        }
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ts = self.timestamp.format("%Y-%m-%d").to_string();
        match &self.suffix {
            Some(s) => match s {
                Suffix::Random(r) => write!(f, "{}_{:08x}", ts, r),
                Suffix::Hash(h) => write!(f, "{}_param_{}", ts, h),
                Suffix::Index(i) => write!(f, "{}_{}", ts, i),
                Suffix::IndexSet(i_set) => {
                    write!(
                        f,
                        "{}_{}",
                        ts,
                        i_set
                            .iter()
                            .map(|i| i.to_string())
                            .collect::<Vec<String>>()
                            .join("_")
                    )
                }
            },
            None => write!(f, "{}", self.timestamp),
        }
    }
}

/// Options for a suffix
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(untagged)]
pub enum Suffix {
    /// Random suffix
    #[serde(rename = "random")]
    Random(u32),
    // First 8 hex chars of parameter hash
    #[serde(rename = "hash")]
    Hash(String),
    #[serde(rename = "index")]
    Index(u32),
    #[serde(rename = "index_set")]
    IndexSet(Vec<u32>),
}

/// Status state of a simulation run
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum RunStatus {
    /// Run is scheduled, but has not been started by the scheduler yet
    #[serde(rename = "pending")]
    Pending,
    /// Run has been started, and hasn't terminated yet
    #[serde(rename = "running")]
    Running {
        /// Initialization time of the simulation run
        start_time: DateTime<Utc>,
    },
    /// Run has been completed successfully
    #[serde(rename = "completed")]
    Completed {
        /// Initialization time of the simulation run
        start_time: DateTime<Utc>,
        /// Completion time of the simulation run
        end_time: DateTime<Utc>,
    },
    /// Run was ended before the completion of the simulation
    /// (likely due to error)
    #[serde(rename = "failed")]
    Failed {
        /// Initialization time of the simulation run
        start_time: DateTime<Utc>,
        /// Completion time of the simulation run
        end_time: DateTime<Utc>,
        /// Reason for preliminary termination of the run
        reason: String,
    },
}

impl RunStatus {
    /// Checks if the run is terminated
    pub fn is_terminated(&self) -> bool {
        matches!(self, Self::Completed { .. } | Self::Failed { .. })
    }

    /// Checks if the simulation is running
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    /// Validates state transition
    pub fn can_transition_to(&self, next: &RunStatus) -> bool {
        matches!(
            (self, next),
            (Self::Pending, Self::Running { .. })
                | (Self::Running { .. }, Self::Completed { .. })
                | (Self::Running { .. }, Self::Failed { .. })
                | (Self::Failed { .. }, Self::Pending)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_status_transitions() {
        let pending = RunStatus::Pending;
        let running = RunStatus::Running {
            start_time: Utc::now(),
        };
        let completed = RunStatus::Completed {
            start_time: Utc::now(),
            end_time: Utc::now(),
        };

        assert!(!pending.can_transition_to(&pending));
        assert!(pending.can_transition_to(&running));
        assert!(!pending.can_transition_to(&completed));

        assert!(!running.can_transition_to(&pending));
        assert!(!running.can_transition_to(&running));
        assert!(running.can_transition_to(&completed));

        assert!(!completed.can_transition_to(&pending));
        assert!(!completed.can_transition_to(&running));
        assert!(!completed.can_transition_to(&completed));
    }

    #[test]
    fn test_is_terminated() {
        assert!(!RunStatus::Pending.is_terminated());
        assert!(!RunStatus::Running {
            start_time: Utc::now()
        }
        .is_terminated());
        assert!(RunStatus::Completed {
            start_time: Utc::now(),
            end_time: Utc::now()
        }
        .is_terminated());
        assert!(RunStatus::Failed {
            start_time: Utc::now(),
            end_time: Utc::now(),
            reason: "none".to_string()
        }
        .is_terminated());
    }
}
