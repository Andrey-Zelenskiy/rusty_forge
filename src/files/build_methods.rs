// Copyright Andrey Zelenskiy, 2024-2026

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{builder::prelude::*, files::FileStatus};

use super::FileManager;

/// Builder of FileManager
#[derive(BuilderSetters, Default, Deserialize, Serialize)]
pub struct FileManagerBuilder {
    // Project root directory
    project_dir: Option<PathBuf>,
    // Data subdirectory
    output_dir: Option<PathBuf>,
    // File name
    name: Option<String>,
    // File extension
    extension: Option<String>,
    // Option for file series (optinal number of files)
    series: Option<Option<usize>>,
}

impl BuilderMethods for FileManagerBuilder {
    type Target = FileManager;

    fn build(&mut self) -> Result<Self::Target, BuildError> {
        // Ensure that the required components are specified
        match (&self.output_dir, &self.name, &self.extension) {
            (Some(output_dir), Some(name), Some(extension)) => {
                // Construct the path to output
                let mut path = PathBuf::new();

                // Project root
                let project_dir = match &self.project_dir {
                    Some(project_path) => project_path,
                    None => &PathBuf::new(),
                };

                path.push(project_dir);

                // Output subdirectory
                path.push(output_dir);

                // File name
                // If working with a series of file, initialize the first file
                let filename = match &self.series {
                    Some(_) => &format!("{name}_0"),
                    None => name,
                };

                path.push(filename);

                // File extension
                path.set_extension(extension);

                Ok(Self::Target {
                    project_dir: self.project_dir.clone(),
                    output_dir: PathBuf::from(output_dir),
                    name: name.to_string(),
                    extension: extension.to_string(),
                    series: self.series.map(|n| (n, 0)),
                    path,
                    status: FileStatus::NotInitialized,
                })
            }
            _ => {
                let values = [
                    {
                        match &self.output_dir {
                            Some(s) => String::from(
                                s.to_str().expect("Missing output path"),
                            ),
                            None => String::from("NOT_SPECIFIED"),
                        }
                    },
                    {
                        match &self.name {
                            Some(s) => s.to_string(),
                            None => String::from("NOT_SPECIFIED"),
                        }
                    },
                    {
                        match &self.extension {
                            Some(s) => s.to_string(),
                            None => String::from("NOT_SPECIFIED"),
                        }
                    },
                ];

                Err(BuildError::IncompleteBuilderData {
                    reason: format!(
                        "FileManager requires output_dir ({}), name ({}), \
                        and extension ({}).",
                        values[0], values[1], values[2]
                    ),
                })
            }
        }
    }

    fn from_target(target: &Self::Target) -> Self {
        Self {
            project_dir: target.project_dir.clone(),
            output_dir: Some(PathBuf::from(&target.output_dir)),
            name: Some(target.name.to_string()),
            extension: Some(target.extension.to_string()),
            series: target.series.map(|(n, _)| n),
        }
    }
}

impl TargetFromBuilder for FileManager {
    type Builder = FileManagerBuilder;
}
