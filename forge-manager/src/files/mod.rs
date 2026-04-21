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
//! # Example
//!

use std::{
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::builder::prelude::*;

mod build_methods;

// Enum encoding the permissions of the Filemanager
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) enum FileStatus {
    // FileManager is built, no project path, no initialized files
    NotInitialized,
    // Project path is assigned, file(s) initialized, has write permissions,
    // appends data to a single file
    InitializedWritableAppend,
    // Project path is assigned, file(s) initialized, has write permissions,
    // creates a new file each time single file
    InitializedWritableOpen,
    // Project path is assigned, file(s) initialized, no write permissions
    InitializedForbidden,
}

impl FileStatus {
    pub(super) fn initialized(&self) -> bool {
        !matches!(self, Self::NotInitialized)
    }

    pub(super) fn writable(&self) -> bool {
        matches!(
            self,
            Self::InitializedWritableAppend | Self::InitializedWritableOpen
        )
    }
}

impl From<FileStatus> for Result<(), io::ErrorKind> {
    fn from(value: FileStatus) -> Self {
        match value {
            FileStatus::NotInitialized => Err(io::ErrorKind::NotFound),
            FileStatus::InitializedForbidden => {
                Err(io::ErrorKind::PermissionDenied)
            }
            _ => Ok(()),
        }
    }
}

/// Structure for managing output file information
///
/// Note that project_dir, name, and extension must be specified to
/// successfully build the structure.
///
/// # Example
///
/// ```rust
/// use rusty_forge::FileManager;
/// use rusty_forge::builder::prelude::*;
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
    project_dir: Option<PathBuf>,
    // Data subdirectory
    output_dir: PathBuf,
    // File name
    name: String,
    // File extension
    extension: String,
    // Option for file series (optinal number of files, current file index)
    series: Option<(Option<usize>, usize)>,
    /// Path to the data output
    path: PathBuf,
    /// File initialization status
    status: FileStatus,
}

impl FileManager {
    /// Path to the output file
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::path::PathBuf;
    ///
    /// use rusty_forge::FileManager;
    /// use rusty_forge::builder::prelude::*;
    ///
    /// let file_manager = FileManager::builder()
    ///   .set_project_dir("./project")
    ///   .set_output_dir("data")
    ///   .set_name("averages")
    ///   .set_extension("txt")
    ///   .build()
    ///   .unwrap();
    ///
    /// assert_eq!(
    ///   PathBuf::from("./project/data/averages.txt"),
    ///   PathBuf::from(file_manager.path()),
    /// );
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Creates an empty file at the specified path for appending data
    ///
    /// # Example
    ///
    /// ```rust
    /// use tempfile::tempdir;
    ///
    /// use rusty_forge::builder::prelude::*;
    /// use rusty_forge::FileManager;
    ///
    /// let path = tempdir()
    ///   .expect("Failed to initialize a temporary directory")
    ///   .path()
    ///   .join("project");
    ///
    /// let mut file_manager = FileManager::builder()
    ///   .set_project_dir(path)
    ///   .set_output_dir("data")
    ///   .set_name("averages")
    ///   .set_extension("txt")
    ///   .build()
    ///   .unwrap();
    ///
    /// file_manager.initialize_append().unwrap();
    ///
    /// assert!(file_manager.path().exists());
    ///
    /// assert!(file_manager.path().is_file());
    /// ```
    pub fn initialize_append(&mut self) -> Result<(), io::Error> {
        // Check that the file hasn't been created yet
        self.check_file_exists()?;

        // Initialize output directory
        self.create_output_dir()?;

        // Initialize a file to which the data will be appended
        self.create_file()?;

        // Change the permission status
        self.status = FileStatus::InitializedWritableAppend;

        Ok(())
    }

    /// Initializes the output directory to write data (new file every time)
    ///
    /// # Example
    ///
    /// ```rust
    /// use tempfile::tempdir;
    ///
    /// use rusty_forge::builder::prelude::*;
    /// use rusty_forge::FileManager;
    ///
    /// let path = tempdir()
    ///   .expect("Failed to initialize a temporary directory")
    ///   .path()
    ///   .join("project");
    ///
    /// let mut file_manager = FileManager::builder()
    ///   .set_project_dir(path)
    ///   .set_output_dir("data")
    ///   .set_name("averages")
    ///   .set_extension("txt")
    ///   .build()
    ///   .unwrap();
    ///
    /// file_manager.initialize_open().unwrap();
    ///
    /// assert!(!file_manager.path().exists());
    ///
    /// assert!(file_manager.path().parent().is_some());
    /// ```
    pub fn initialize_open(&mut self) -> Result<(), io::Error> {
        // Check that the file hasn't been created yet
        self.check_file_exists()?;

        // Initialize output directory
        self.create_output_dir()?;

        // Change the permission status
        self.status = FileStatus::InitializedWritableOpen;

        Ok(())
    }

    /// Sets the project directory to the file manager
    pub fn set_project_dir<T: Into<PathBuf>>(&mut self, path: T) {
        self.project_dir = Some(path.into());
        self.set_path();
    }

    /// Open a file to write data
    pub fn open_file(&self) -> Result<fs::File, io::Error> {
        match self.status {
            FileStatus::InitializedWritableAppend => {
                fs::OpenOptions::new().append(true).open(self.path())
            }
            FileStatus::InitializedWritableOpen => fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.path()),
            _ => Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "FileManager {:?} does not have write permissions",
                    self.path
                ),
            )),
        }
    }

    /// Open a file in a buffer to append the data (for large arrays)
    pub fn open_buffer(&self) -> Result<io::BufWriter<fs::File>, io::Error> {
        Ok(io::BufWriter::new(self.open_file()?))
    }

    /// Write permissions of the FileManager
    pub fn writable(&self) -> bool {
        self.status.writable()
    }

    /// Change writing permissions
    pub fn remove_permissions(&mut self) {
        if self.status.initialized() {
            self.status = FileStatus::InitializedForbidden;
        }
    }

    /// If working with series of files, change the index of the file
    pub fn change_file_index(&mut self, value: usize) {
        if let Some((_, index)) = &mut self.series {
            *index = value;
            self.set_path();
        }
    }

    // Re-calculate output path
    fn set_path(&mut self) {
        // Construct the path to output
        let mut path = PathBuf::new();

        // Project root
        let project_dir = match &self.project_dir {
            Some(project_path) => project_path,
            None => &PathBuf::new(),
        };

        path.push(project_dir);

        // Output subdirectory
        path.push(&self.output_dir);

        // File name
        // If working with a series of files, modify filename
        let filename = match &self.series {
            Some((_, index)) => &format!("{}_{index}", self.name),
            None => &self.name,
        };

        path.push(filename);

        // File extension
        path.set_extension(&self.extension);

        self.path = path;
    }

    // Check if the file(s) set to be initialized have already been created
    fn check_file_exists(&self) -> Result<(), io::Error> {
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
        Ok(())
    }

    // Initialize file output directory
    fn create_output_dir(&self) -> Result<(), io::Error> {
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
        Ok(())
    }

    // Initialize file(s)
    fn create_file(&mut self) -> Result<(), io::Error> {
        match &self.series {
            None => {
                fs::OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(self.path())?;
            }
            Some((None, _)) => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Attempting to create an unsized series file without \
                        writing data.",
                ))
            }
            Some((Some(n_files), _)) => {
                for i in 0..*n_files {
                    self.change_file_index(i);

                    fs::OpenOptions::new()
                        .create(true)
                        .truncate(true)
                        .write(true)
                        .open(self.path())?;
                }
            }
        };

        Ok(())
    }
}

/// Structure for managing output of debug summary, serialized state,
/// and the builder of a structure.
///
/// # Example
///
/// ```rust
///
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OutputManager {
    /// Debug summary file manager
    summary_manager: FileManager,
    /// Serialized state file manager
    state_manager: FileManager,
    /// Serialized builder state file manager
    builder_manager: FileManager,
}

impl OutputManager {
    /// New OutputManager given the name of the structure
    ///
    /// # Panics
    ///
    /// This function panics if one of the three FileManagers fails to
    /// initialize.
    pub fn new(name: &str) -> Self {
        Self {
            summary_manager: FileManager::builder()
                .set_output_dir("summary")
                .set_name(name)
                .set_extension("out")
                .build()
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed to build debug summary FileManager for {} \
                        OutputManager",
                        name
                    )
                }),
            state_manager: FileManager::builder()
                .set_output_dir("checkpoint")
                .set_name(name)
                .set_extension("toml")
                .set_series(None)
                .build()
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed to build state checkpoint FileManager for {} \
                        OutputManager",
                        name
                    )
                }),
            builder_manager: FileManager::builder()
                .set_output_dir("builder")
                .set_name(name)
                .set_extension("toml")
                .set_series(None)
                .build()
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed to build builder checkpoint FileManager for \
                        {} OutputManager",
                        name
                    )
                }),
        }
    }

    /// Returns mutable references to the file managers
    pub fn get_file_managers(&mut self) -> Vec<&mut FileManager> {
        vec![
            &mut self.summary_manager,
            &mut self.state_manager,
            &mut self.builder_manager,
        ]
    }

    /// Save debug summary of the data
    pub fn save_summary<T: fmt::Display>(
        &mut self,
        data: T,
    ) -> Result<(), io::Error> {
        let mut file = self.summary_manager.open_buffer()?;

        write!(file, "{}", data)?;

        file.flush()?;

        Ok(())
    }

    /// Save serialized checkpoint state of the data
    pub fn save_state<T: Serialize>(
        &mut self,
        data: T,
    ) -> Result<(), io::Error> {
        let data_str = toml::to_string(&data).map_err(|e| {
            io::Error::other(format!(
                "Could not serialize state data for {:#?}: {}",
                self.state_manager.path(),
                e
            ))
        })?;

        let mut file = self.state_manager.open_buffer()?;

        write!(file, "{}", data_str)?;

        file.flush()?;

        self.state_manager.series.map(|(n_series, mut index)| {
            index += 1;
            (n_series, index)
        });

        Ok(())
    }

    /// Save serialized builder of the data
    pub fn save_builder<T: TargetFromBuilder>(
        &mut self,
        data: T,
    ) -> Result<(), io::Error> {
        let data_str = toml::to_string(&T::Builder::from_target(&data))
            .map_err(|e| {
                io::Error::other(format!(
                    "Could not serialize builder data for {:#?}: {}",
                    self.builder_manager.path(),
                    e
                ))
            })?;

        let mut file = self.builder_manager.open_buffer()?;

        write!(file, "{}", data_str)?;

        file.flush()?;

        self.builder_manager.series.map(|(n_series, mut index)| {
            index += 1;
            (n_series, index)
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

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

        assert!(!file_manager.status.writable());

        assert_eq!(
            PathBuf::from_str("./project/data/averages.txt").unwrap(),
            file_manager.path
        );
    }

    #[test]
    fn build_file_manager_no_project_dir() {
        let file_manager = FileManager::builder()
            .set_output_dir("data")
            .set_name("averages")
            .set_extension("txt")
            .build()
            .unwrap();

        assert!(!file_manager.status.writable());

        assert_eq!(
            PathBuf::from_str("data/averages.txt").unwrap(),
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
            .set_series(5_usize)
            .build()
            .unwrap();

        assert!(!file_manager.status.writable());

        assert_eq!(
            PathBuf::from_str("./project/data/averages_0.txt").unwrap(),
            file_manager.path
        );
    }

    // Tests for incomplete data
    #[test]
    #[should_panic]
    fn build_file_manager_no_output_dir() {
        let _ = FileManager::builder()
            .set_project_dir("./project")
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
            .set_output_dir("data")
            .set_name("averages")
            .set_extension("txt")
            .build()
            .unwrap();

        assert!(file_manager
            .open_file()
            .is_err_and(|e| e.kind() == io::ErrorKind::PermissionDenied));
    }
}
