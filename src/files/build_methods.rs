// Copyright Andrey Zelenskiy, 2024-2026
//
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::builder::{BuilderMethods, TargetFromBuilder};

use super::FileManager;

/// Builder of FileManager
#[derive(Default, Deserialize, Serialize)]
pub struct FileManagerBuilder {
    // Project root directory
    project_dir: Option<PathBuf>,
    // Data subdirectory
    output_dir: Option<PathBuf>,
    // File name
    name: Option<String>,
    // File extension
    extension: Option<String>,
    // Option for file series
    series: Option<usize>,
}

impl FileManagerBuilder {
    // Setter methods

    /// Sets project root directory
    pub fn set_project_dir<T>(&mut self, project_dir: T) -> &mut Self
    where
        PathBuf: From<T>,
    {
        self.project_dir = Some(PathBuf::from(project_dir));
        self
    }

    /// Sets subdirectory of the data
    pub fn set_output_dir<T>(&mut self, output_dir: T) -> &mut Self
    where
        PathBuf: From<T>,
    {
        self.output_dir = Some(PathBuf::from(output_dir));
        self
    }

    /// Sets file name
    pub fn set_name<T: ToString>(&mut self, name: T) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets file extension
    pub fn set_extension<T: ToString>(&mut self, extension: T) -> &mut Self {
        self.extension = Some(extension.to_string());
        self
    }

    /// Specifies manager for a series of n_files outputs
    pub fn set_series(&mut self, n_files: usize) -> &mut Self {
        self.series = Some(n_files);
        self
    }
}

impl BuilderMethods for FileManagerBuilder {
    type Target = FileManager;

    fn try_build(&mut self) -> Result<Self::Target, String> {
        // Ensure that the required components are specified
        match (&self.project_dir, &self.name, &self.extension) {
            (Some(project_dir), Some(name), Some(extension)) => {
                // Construct the path to output
                let mut path = PathBuf::new();

                // Project root
                path.push(project_dir);

                // Output subdirectory
                let output_dir = match &self.output_dir {
                    Some(output_path) => output_path,
                    None => &PathBuf::new(),
                };

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
                    project_dir: PathBuf::from(project_dir),
                    output_dir: self.output_dir.clone(),
                    name: name.to_string(),
                    extension: extension.to_string(),
                    series: self.series.map(|n| (n, 0)),
                    path,
                    writable: false,
                })
            }
            _ => {
                let values = [
                    {
                        match &self.project_dir {
                            Some(s) => String::from(
                                s.to_str().expect("Missing project root path"),
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

                Err(format!(
                    "FileManager requires project_dir ({}), name ({}), \
                        and extension ({}).",
                    values[0], values[1], values[2]
                ))
            }
        }
    }

    fn from_target(target: &Self::Target) -> Self {
        Self {
            project_dir: Some(PathBuf::from(&target.project_dir)),
            output_dir: target.output_dir.clone(),
            name: Some(target.name.to_string()),
            extension: Some(target.extension.to_string()),
            series: target.series.map(|(n, _)| n),
        }
    }
}

impl TargetFromBuilder for FileManager {
    type Builder = FileManagerBuilder;
}
