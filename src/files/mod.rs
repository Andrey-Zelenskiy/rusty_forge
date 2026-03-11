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

// Enum encoding serialization type of the Filemanager
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) enum FileSerialization {
    // Non-serialized data
    None,
    // Toml format
    Toml,
    // Json format
    Json,
}

/// Structure for managing output file information
///
/// Note that project_dir, name, and extension must be specified to
/// successfully build the structure.
///
/// # Example
///
/// ```rust
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
    /// Indicator for serialized data (only supports toml and json)
    serialization: FileSerialization,
}

impl FileManager {
    /// Path to the output file
    ///
    /// # Example
    ///
    /// ```rust
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

    /// Initializes the ouput directory to write data (new file every time)
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
    ///   .set_name("averages")
    ///   .set_extension("txt")
    ///   .build()
    ///   .unwrap();
    ///
    /// file_manager.initialize_open().unwrap();
    ///
    /// assert!(!file_manager.path().exists());
    ///
    /// assert!(file_manager.path().parent().exists());
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

    /// Save data that can be converted to a single string to file.
    ///
    /// This method should be used to save smaller data structures, such as
    /// debug summary generated by `.to_string()` method.
    /// For large arrays, use [`save_array`] method.
    ///
    /// __Note__: if the `FileManager` is set to be serializable (either
    /// `.toml` or `.json` format), this method will return error of the
    /// `io::ErrorKind::InvaldFilename`.
    ///
    /// # Example
    ///
    /// ```rust
    /// ```
    pub fn save_data<T: fmt::Display>(
        &mut self,
        data: T,
        buffer: bool,
    ) -> Result<(), io::Error> {
        // Return error if the file is set as serializable
        if !matches!(self.serialization, FileSerialization::None) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidFilename,
                format!(
                    "Attempting to save string data to a serializable file \
                    {:#?}",
                    self.path()
                ),
            ));
        }

        if buffer {
            self.save_data_to_buffer(data)?;
        } else {
            self.save_data_to_file(data)?;
        }

        // If dealing with series, update the index
        if let Some((_, index)) = &mut self.series {
            *index += 1;
        }

        Ok(())
    }

    /// Save serializable data.
    ///
    /// This method should be used to save serialization of types in
    /// human-readable formats (currently supports `.toml` and `.json`).
    ///
    /// __Note__: if the `FileManager` is not set to be serializable ineither
    /// `.toml` nor `.json` format), this method will return error of the
    /// `io::ErrorKind::InvaldFilename`.
    ///
    /// # Example
    ///
    /// ```rust
    /// ```
    pub fn serialize_data<T: Serialize>(
        &mut self,
        data: T,
    ) -> Result<(), io::Error> {
        match &self.serialization {
            FileSerialization::None => Err(io::Error::new(
                io::ErrorKind::InvalidFilename,
                format!(
                    "Attempting to serialize data to a non-serializable file \
                    {:#?}",
                    self.path()
                ),
            )),
            FileSerialization::Toml => {
                self.save_data_to_buffer(toml::to_string(&data).map_err(
                    |e| {
                        io::Error::other(format!(
                            "Could not serialize data for {:#?}: {}",
                            self.path(),
                            e
                        ))
                    },
                )?)?;

                if let Some((_, index)) = &mut self.series {
                    *index += 1;
                }

                Ok(())
            }
            FileSerialization::Json => {
                self.save_data_to_buffer(
                    serde_json::to_string(&data).map_err(|e| {
                        io::Error::other(format!(
                            "Could not serialize data for {:#?}: {}",
                            self.path(),
                            e
                        ))
                    })?,
                )?;

                if let Some((_, index)) = &mut self.series {
                    *index += 1;
                }

                Ok(())
            }
        }
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

    // Saves data to file directly
    fn save_data_to_file<T: fmt::Display>(
        &self,
        data: T,
    ) -> Result<(), io::Error> {
        // Initialize a buffer and save the data
        let mut file = self.open_file()?;

        write!(file, "{}", data)?;

        Ok(())
    }

    // Saves data to file through buffer
    fn save_data_to_buffer<T: fmt::Display>(
        &self,
        data: T,
    ) -> Result<(), io::Error> {
        // Initialize a buffer and save the data
        let mut file = self.open_buffer()?;

        write!(file, "{}", data)?;

        // Finalize, write, and empty the buffer
        file.flush()?;

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
        self.summary_manager.save_data(data, true)
    }

    /// Save serialized checkpoint state of the data
    pub fn save_state<T: Serialize>(
        &mut self,
        data: T,
    ) -> Result<(), io::Error> {
        self.state_manager.serialize_data(data)
    }

    /// Save serialized builder of the data
    pub fn save_builder<T: TargetFromBuilder>(
        &mut self,
        data: T,
    ) -> Result<(), io::Error> {
        self.state_manager
            .serialize_data(T::Builder::from_target(&data))
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
    fn build_file_manager_no_output_dir() {
        let file_manager = FileManager::builder()
            .set_project_dir("./project")
            .set_name("averages")
            .set_extension("txt")
            .build()
            .unwrap();

        assert!(!file_manager.status.writable());

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
