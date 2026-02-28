// Copyright Andrey Zelenskiy, 2024-2026

//! # Module for interfacing with data files
//!
//! This module provides two structures for setting up the output files:
//! [ `FileManager` ] and [ `OutputManager` ].
//! Both structures contain methods for quick and safe writing of output to
//! files in the project.
//!
//! [ `FileManager` ] holds the path to a file or series of files with the same
//! extension, _e.g._ `./project/averages.txt` or
//! `./project/averages_{0...n}.txt`.
//! This structure is meant to point to the data of a single output type.
//!
//! [ `OutputManager` ] holds three [ `FileManager` ] structures, respectively
//! for storing the debugging message, object's serialization, and the
//! serialization of the corresponding builder.
//!
//! ## Example
//!

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::Serialize;

mod build_methods;

/// Structure for managing output file information
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
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FileManager {
    // Project root directory
    project_dir: PathBuf,
    // Data subdirectory
    output_dir: Option<PathBuf>,
    // File name
    name: String,
    // File extension
    extension: String,
    // Option for file series
    series: Option<(usize, usize)>,
    /// Path to the data output
    path: PathBuf,
    /// Permission for writing to the file
    writable: bool,
}

impl FileManager {
    /// Path to the output file
    ///
    /// ## Example
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// use rusty_forge::{FileManager, BuilderMethods, TargetFromBuilder};
    ///
    /// let file_manager = FileManager::builder()
    ///   .set_project_dir("./project")
    ///   .set_name("averages")
    ///   .set_extension("txt")
    ///   .build()
    ///   .unwrap();
    ///
    /// assert_eq!(
    ///   PathBuf::from("./project/averages.txt"),
    ///   PathBuf::from(file_manager.path()),
    /// );
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Creates an empty file at the specified path
    ///
    /// ## Example
    ///
    /// ```
    /// use tempfile::tempdir;
    ///
    /// use rusty_forge::{FileManager, BuilderMethods, TargetFromBuilder};
    ///
    /// let path = tempdir()
    ///   .expect("Failed to initialize a temporary directory")
    ///   .path()
    ///   .join("project");
    ///
    /// let mut file_manager = FileManager::builder()
    ///   .set_project_dir(path)
    ///   .set_name("averages")
    ///   .set_extension("txt")
    ///   .build()
    ///   .unwrap();
    ///
    /// file_manager.initialize_file().unwrap();
    ///
    /// assert!(file_manager.path().exists());
    ///
    /// assert!(file_manager.path().is_file());
    /// ```
    pub fn initialize_file(&mut self) -> Result<(), io::Error> {
        // Check if the file(s) is already initialized
        if let Some((n_files, _)) = &self.series {
            self.change_file_index(n_files - 1);
            self.set_path();
        }

        if self.path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "FileManager is attempting to initialize an \
                    already existing file: {:?}",
                    self.path
                ),
            ));
        }

        // Create the directory tree
        match self.path.parent() {
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "FileManager is missing a parent directory: {:?}",
                        self.path
                    ),
                ))
            }
            Some(parent_path) => {
                // Parent directory
                fs::create_dir_all(parent_path)?;
            }
        };

        // Initialize file(s)
        match &self.series {
            None => {
                fs::OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(self.path())?;
            }
            Some((n_files, _)) => {
                for i in 0..*n_files {
                    self.change_file_index(i);
                    self.set_path();

                    fs::OpenOptions::new()
                        .create(true)
                        .truncate(true)
                        .write(true)
                        .open(self.path())?;
                }
            }
        };

        // Once the ouput is initialized, we can permit writing of data
        self.writable = true;

        Ok(())
    }

    /// Open a file to append data
    pub fn open_file(&self) -> Result<fs::File, io::Error> {
        if !self.writable {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "FileManager {:?} does not have write permissions",
                    self.path
                ),
            ));
        }

        fs::OpenOptions::new().append(true).open(self.path())
    }

    /// Open a file in a buffer to append the data (for large arrays)
    pub fn open_buffer(&self) -> Result<io::BufWriter<fs::File>, io::Error> {
        Ok(io::BufWriter::new(self.open_file()?))
    }

    /// Change writing permissions
    pub fn change_write_permissions(&mut self, writable: bool) {
        self.writable = writable;
    }

    // Re-calculate output path
    fn set_path(&mut self) {
        // Construct the path to output
        let mut path = PathBuf::new();

        // Project root
        path.push(&self.project_dir);

        // Output subdirectory
        let output_dir = match &self.output_dir {
            Some(output_path) => output_path,
            None => &PathBuf::new(),
        };

        path.push(output_dir);

        // File name
        // If working with a series of file, initialize the first file
        let filename = match &self.series {
            Some((_, index)) => &format!("{}_{index}", self.name),
            None => &self.name,
        };

        path.push(filename);

        // File extension
        path.set_extension(&self.extension);

        self.path = path;
    }

    // If working with series of files, change the index of the file
    fn change_file_index(&mut self, value: usize) {
        if let Some((_, index)) = &mut self.series {
            *index = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    use crate::initialize::{BuilderMethods, TargetFromBuilder};

    #[test]
    fn build_file_manager() {
        let file_manager = FileManager::builder()
            .set_project_dir("./project")
            .set_output_dir("data")
            .set_name("averages")
            .set_extension("txt")
            .build()
            .unwrap();

        assert!(!file_manager.writable);

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

        assert!(!file_manager.writable);

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

        assert!(!file_manager.writable);

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

    #[test]
    fn already_initialized() {}

    #[test]
    fn not_writable() {
        let file_manager = FileManager::builder()
            .set_project_dir("./project")
            .set_name("averages")
            .set_extension("txt")
            .build()
            .unwrap();

        assert!(file_manager
            .open_file()
            .is_err_and(|e| e.kind() == io::ErrorKind::PermissionDenied));
    }
}
