// Copyright Andrey Zelenskiy, 2024-2026

//! # Module for interfacing with data files
//!
//! This module provides methods for setting up the output files.
//!
//! ## Example
//!

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::initialize::{BuilderMethods, TargetFromBuilder};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FileManager {
    // Project root directory
    project_dir: String,
    // Data subdirectory
    output_dir: Option<String>,
    // File name
    name: String,
    // File extension
    extension: String,
    // Option for file series
    series: Option<(u32, usize)>,
    /// Path to the data output
    path: PathBuf,
    /// Permission for writing to the file
    writable: bool,
}

/// Builder of FileManager
///
/// Note that project_dir, name, and extension must be specified to
/// successfully build the structure.
///
/// ## Example
///
/// ```
/// use rusty_forge::{FileManager, BuilderMethods, TargetFromBuilder};
///
/// let file_manager = FileManager::builder()
///   .set_project_dir("./project")
///   .set_output_dir("data")
///   .set_name("averages")
///   .set_extension("txt")
///   .build()
///   .expect("Failed to initialize a file manager");
/// ```
#[derive(Default, Deserialize, Serialize)]
pub struct FileManagerBuilder {
    // Project root directory
    project_dir: Option<String>,
    // Data subdirectory
    output_dir: Option<String>,
    // File name
    name: Option<String>,
    // File extension
    extension: Option<String>,
    // Option for file series
    series: Option<u32>,
}

impl FileManagerBuilder {
    // Setter methods

    /// Sets project root directory
    pub fn set_project_dir<V: ToString>(
        &mut self,
        project_dir: V,
    ) -> &mut Self {
        self.project_dir = Some(project_dir.to_string());
        self
    }

    /// Sets subdirectory of the data
    pub fn set_output_dir<V: ToString>(&mut self, output_dir: V) -> &mut Self {
        self.output_dir = Some(output_dir.to_string());
        self
    }

    /// Sets file name
    pub fn set_name<V: ToString>(&mut self, name: V) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets file extension
    pub fn set_extension<V: ToString>(&mut self, extension: V) -> &mut Self {
        self.extension = Some(extension.to_string());
        self
    }

    /// Specifies manager for a series of n_files outputs
    pub fn set_series(&mut self, n_files: u32) -> &mut Self {
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
                    Some(output_path) => output_path.to_string(),
                    None => "".to_string(),
                };

                path.push(output_dir);

                // File name
                // If working with a series of file, initialize the first file
                let filename = match &self.series {
                    Some(_) => format!("{name}_0"),
                    None => name.to_string(),
                };

                path.push(filename);

                // File extension
                path.set_extension(extension);

                Ok(Self::Target {
                    project_dir: project_dir.to_string(),
                    output_dir: self.output_dir.clone(),
                    name: name.to_string(),
                    extension: extension.to_string(),
                    series: self.series.map(|n| (n, 0)),
                    path,
                    writable: true,
                })
            }
            _ => {
                let values_opt =
                    [&self.project_dir, &self.name, &self.extension];
                let values = values_opt.map(|v| match v {
                    Some(s) => s.to_string(),
                    None => String::from("NOT_SPECIFIED"),
                });

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
            project_dir: Some(target.project_dir.to_string()),
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn build_file_manager() {
        let file_manager = FileManager::builder()
            .set_project_dir("./project")
            .set_output_dir("data")
            .set_name("averages")
            .set_extension("txt")
            .build()
            .unwrap();

        assert!(file_manager.writable);

        assert_eq!(
            PathBuf::from_str("./project/data/averages.txt").unwrap(),
            file_manager.path
        );
    }

    #[test]
    fn build_file_manager_no_output_dir() {
        let file_manager = FileManager::builder()
            .set_project_dir("./project")
            .set_name("averages")
            .set_extension("txt")
            .build()
            .unwrap();

        assert!(file_manager.writable);

        assert_eq!(
            PathBuf::from_str("./project/averages.txt").unwrap(),
            file_manager.path
        );
    }

    #[test]
    fn build_file_manager_series() {
        let file_manager = FileManager::builder()
            .set_project_dir("./project")
            .set_output_dir("data")
            .set_name("averages")
            .set_extension("txt")
            .set_series(5)
            .build()
            .unwrap();

        assert!(file_manager.writable);

        assert_eq!(
            PathBuf::from_str("./project/data/averages_0.txt").unwrap(),
            file_manager.path
        );
    }

    // Tests for incomplete data
    #[test]
    #[should_panic]
    fn build_file_manager_no_project_dir() {
        let _ = FileManager::builder()
            .set_output_dir("data")
            .set_name("averages")
            .set_extension("txt")
            .build()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn build_file_manager_no_name() {
        let _ = FileManager::builder()
            .set_project_dir("./project")
            .set_output_dir("data")
            .set_extension("txt")
            .build()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn build_file_manager_no_extension() {
        let _ = FileManager::builder()
            .set_project_dir("./project")
            .set_output_dir("data")
            .set_name("averages")
            .build()
            .unwrap();
    }
}
