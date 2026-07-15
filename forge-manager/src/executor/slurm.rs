// Copyright Andrey Zelenskiy, 2024-2026

use std::{fmt::Write, fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::ManagerResult;

/// Configuration for slurm jobs
#[derive(Serialize, Deserialize, Clone)]
pub struct SlurmConfig {
    pub partition: String,
    pub time: String,
    pub ntasks: usize,
    pub extra_directives: Vec<String>,
    pub command: String,
}

impl SlurmConfig {
    /// Write slurm file
    pub fn write_script(
        &self,
        slurm_path: &Path,
        config_path: &Path,
        job_name: &str,
    ) -> ManagerResult<()> {
        let contents = self.render_script(job_name, config_path)?;
        fs::write(slurm_path, contents).map_err(Into::into)
    }

    /// Build the slurm script with the executable command
    fn render_script(
        &self,
        job_name: &str,
        config_path: &Path,
    ) -> ManagerResult<String> {
        let mut script = String::new();

        // Build the #SBATCH header
        writeln!(script, "#!/bin/bash")?;
        writeln!(script, "#SBATCH --job-name={}", job_name)?;
        writeln!(script, "#SBATCH --partition={}", self.partition)?;
        writeln!(script, "#SBATCH --time={}", self.time)?;
        writeln!(script, "#SBATCH --ntasks={}", self.ntasks)?;

        for directive in self.extra_directives.iter() {
            if directive.starts_with("#SBATCH --") {
                writeln!(script, "{}", directive)?;
            } else {
                writeln!(script, "#SBATCH --{}", directive)?;
            }
        }

        // Add the call to the executable
        let config_path_str = config_path.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Config file path is invalid",
            )
        })?;

        writeln!(script)?;
        writeln!(script, "{} {}", self.command, config_path_str)?;

        Ok(script)
    }
}
