// Copyright Andrey Zelenskiy, 2024-2026

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};

use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};

use crate::{run::parameters::ParameterMap, ManagerResult};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    /// ID of the simulation run
    pub id: RunId,
    /// Current status of the simulation
    pub status: RunStatus,
    /// Timestamp of run initialization
    pub initialization_time: DateTime<Utc>,
}

impl RunState {
    /// Write state to run directory
    pub fn write(&self, path: &Path) -> ManagerResult<()> {
        let state_path = path.with_file_name("state.toml");
        let state_toml = toml::to_string_pretty(self)?;

        fs::write(state_path, state_toml).map_err(Into::into)
    }

    /// Read state from run directory
    pub fn load(path: &Path) -> ManagerResult<Self> {
        let state_path = path.with_file_name("state.toml");
        let state_path_str = state_path.to_str().ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unable to convert run Path to str",
        ))?;

        Config::builder()
            .add_source(File::new(state_path_str, FileFormat::Toml))
            .build()?
            .try_deserialize::<Self>()
            .map_err(Into::into)
    }
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
    pub fn from_parameters(params_hash: &u64) -> Self {
        Self {
            timestamp: Utc::now(),
            suffix: Some(Suffix::Hash(format!("{:08x}", params_hash))),
        }
    }

    /// Generate new RunId with a single numerical index
    pub fn from_index(index: u32) -> Self {
        Self {
            timestamp: Utc::now(),
            suffix: Some(Suffix::Index(index)),
        }
    }

    /// Generate new RunId with a set of numerical indices
    pub fn from_index_set(indices: &[u32]) -> Self {
        Self {
            timestamp: Utc::now(),
            suffix: Some(Suffix::IndexSet(Vec::from(indices))),
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
    #[serde(rename = "Pending")]
    Pending,
    /// Run has been started, and hasn't terminated yet
    #[serde(rename = "Running")]
    Running {
        /// Initialization time of the simulation run
        start_time: DateTime<Utc>,
    },
    /// Run has been completed successfully
    #[serde(rename = "Completed")]
    Completed {
        /// Initialization time of the simulation run
        start_time: DateTime<Utc>,
        /// Completion time of the simulation run
        end_time: DateTime<Utc>,
    },
    /// Run was ended before the completion of the simulation
    /// (likely due to error)
    #[serde(rename = "Failed")]
    Failed {
        /// Initialization time of the simulation run
        start_time: DateTime<Utc>,
        /// Completion time of the simulation run
        end_time: DateTime<Utc>,
    },
}

impl RunStatus {
    /// Return initialization time
    pub fn start_time(&self) -> Option<DateTime<Utc>> {
        match &self {
            Self::Pending => None,
            Self::Running { start_time }
            | Self::Completed { start_time, .. }
            | Self::Failed { start_time, .. } => Some(*start_time),
        }
    }

    /// Checks if the run is terminated
    pub fn is_terminated(&self) -> bool {
        matches!(self, Self::Completed { .. } | Self::Failed { .. })
    }

    /// Checks if the simulation is running
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    /// Validates state transition
    pub fn can_transition_to(&self, next: &str) -> bool {
        matches!(
            (self, next),
            (Self::Pending, "Running")
                | (Self::Running { .. }, "Completed")
                | (Self::Running { .. }, "Failed")
                | (Self::Failed { .. }, "Pending")
        )
    }
}

impl fmt::Display for RunStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Running { .. } => write!(f, "Running"),
            Self::Completed { .. } => write!(f, "Completed"),
            Self::Failed { .. } => write!(f, "Failed"),
        }
    }
}

/// Options for the initialization of the RunId
pub enum RunIdBuilder<'a> {
    /// Random suffix
    Random,
    /// No suffix
    Timestamp,
    /// Parameters hash
    Parameters(u64),
    /// Numerical index
    Index(u32),
    /// Set of numerical indices
    IndexSet(&'a [u32]),
}

impl<'a> RunIdBuilder<'a> {
    /// Initialize RunId
    pub fn build(&self) -> RunId {
        match self {
            Self::Random => RunId::from_random(),
            Self::Timestamp => RunId::from_timestamp(),
            Self::Parameters(params_hash) => {
                RunId::from_parameters(params_hash)
            }
            Self::Index(index) => RunId::from_index(*index),
            Self::IndexSet(indices) => RunId::from_index_set(indices),
        }
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

        assert!(!pending.can_transition_to("Pending"));
        assert!(pending.can_transition_to("Running"));
        assert!(!pending.can_transition_to("Completed"));

        assert!(!running.can_transition_to("Pending"));
        assert!(!running.can_transition_to("Running"));
        assert!(running.can_transition_to("Completed"));

        assert!(!completed.can_transition_to("Pending"));
        assert!(!completed.can_transition_to("Running"));
        assert!(!completed.can_transition_to("Completed"));
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
        }
        .is_terminated());
    }
}
