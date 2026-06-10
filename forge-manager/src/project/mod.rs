// Copyright Andrey Zelenskiy, 2024-2026

//! # Module for setting up simulation project
//!
//! This module provides methods for setting up the files and directory tree
//! of a numerical simulation project.
//! The methods emphasize data safety, providing several protocols for dealing
//! with existing data.
//!
//! ## Example

mod config;
use std::path::{Path, PathBuf};

use config::ProjectManifest;

pub mod layout;
use layout::ProjectLayout;

use crate::{errors::ManagerError, ManagerResult};

/// Structure that manages the simulation project
pub struct ProjectManager {
    /// Project configuration data
    manifest: ProjectManifest,
    /// Project path dependencies
    layout: ProjectLayout,
}

impl ProjectManager {
    /// Creates a new simulation project
    ///
    /// The resulting layout for project with path `project_root` is
    ///
    /// ``` ignore
    /// project_root/
    /// |     index.toml       # Index of the simulation runs
    /// |     manifest.toml    # Project manifest file
    /// |     registry.toml    # Simulation run registry
    /// |
    /// +---- analysis/        # Directory for simulation data analysis
    /// |
    /// +---- runs/            # Directory with all simulation runs
    /// ```
    pub fn create(
        name: &str,
        author: &Option<String>,
        description: &Option<String>,
        path: &Path,
    ) -> ManagerResult<Self> {
        if Self::exists(path) {
            Err(ManagerError::ProjectAlreadyExists(PathBuf::from(path)))
        } else {
            // Create a new project manager
            let manager = Self {
                manifest: ProjectManifest::new(name, author, description),
                layout: ProjectLayout::new(PathBuf::from(path)),
            };

            // Initialize project layout
            manager.layout.create_layout()?;

            // Write manifest.toml
            manager.manifest.write(path)?;

            Ok(manager)
        }
    }

    /// Loads a simulation project from file
    pub fn load(path: &Path) -> ManagerResult<Self> {
        if Self::exists(path) {
            Ok(Self {
                manifest: ProjectManifest::load(path)?,
                layout: ProjectLayout::new(PathBuf::from(path)),
            })
        } else {
            Err(ManagerError::ProjectNotFound(PathBuf::from(path)))
        }
    }

    /// Checks if a project is initialized at a given path
    pub fn exists(path: &Path) -> bool {
        path.with_file_name("manifest.toml").exists()
    }

    /// Project name
    pub fn name(&self) -> &str {
        &self.manifest.metadata.name
    }

    /// Project path
    pub fn path(&self) -> &Path {
        self.layout.root_dir()
    }

    /// Project author (if given)
    pub fn author(&self) -> &Option<String> {
        &self.manifest.metadata.author
    }

    /// Project description (if given)
    pub fn description(&self) -> &Option<String> {
        &self.manifest.metadata.description
    }
}

// /// Structure that manages the simulation project
// #[derive(Deserialize, Debug)]
// pub struct ProjectManager {
//     /// Path to the project directory
//     path: PathBuf,
//     /// Recovery mode option for restarting a simulation from the last
//     /// checkpoint
//     #[serde(default)]
//     recovery_mode: Option<usize>,
//     /// Option to timestamp the name of the project directory with /yyy-mm-dd/
//     #[serde(default)]
//     timestamped: bool,
//     /// Type of behaviour if project path already exists
//     data_protocol: DataProtocol,
// }

// use std::{
//     fs, io,
//     path::{Path, PathBuf},
// };

// use config::Config;

// use regex::Regex;

// use serde::Deserialize;

// mod manifest;
// use manifest::ProjectManifest;

// /// Protocol for dealing with files that already exist
// #[derive(Deserialize, Debug, PartialEq, Eq)]
// pub enum DataProtocol {
//     /// Copies duplicate data to a timestamped folder
//     Archive,
//     /// Interrupts the program if data with duplicate names already exist
//     Panic,
// }

// impl ProjectManager {
//     /// Simple project structure: new directory without timestamps,
//     /// panics if already exists
//     ///
//     /// ## Examples
//     ///
//     /// ```
//     /// use rusty_forge::project_setup::ProjectManager;
//     ///
//     /// let project_manager = ProjectManager::new_simple("/tmp/simulation_project");
//     ///
//     /// assert!(!project_manager.is_recovery());
//     /// ```
//     pub fn new_simple<P: AsRef<Path>>(path: P) -> Self {
//         Self {
//             path: PathBuf::from(path.as_ref()),
//             recovery_mode: None,
//             timestamped: false,
//             data_protocol: DataProtocol::Panic,
//         }
//     }

//     /// Initializes project, depending on the recovery option
//     ///
//     ///
//     pub fn initialize_project(&mut self) -> Result<(), io::Error> {
//         // Choose initialization based on the recovery option
//         if self.is_recovery() {
//             self.initialize_recovery()
//         } else {
//             self.initialize_new()
//         }
//     }

//     /// Initializes log files in the project directory
//     pub fn initialize_logs(
//         &mut self,
//         file_managers: &mut Vec<&mut FileManager>,
//     ) -> Result<(), io::Error> {
//         file_managers.iter_mut().try_for_each(|f| {
//             // Add project directory to all files
//             f.set_project_dir(self.path().join("logs"));

//             f.initialize_open()
//         })?;

//         Ok(())
//     }

//     /// Initializes checkpoint files in the project directory
//     pub fn initialize_checkpoints(
//         &mut self,
//         file_managers: &mut Vec<&mut FileManager>,
//     ) -> Result<(), io::Error> {
//         file_managers.iter_mut().try_for_each(|f| {
//             // Add project directory to all files
//             f.set_project_dir(self.path().join("checkpoints"));

//             f.initialize_open()
//         })?;

//         Ok(())
//     }

//     /// Initializes output files in the project directory
//     pub fn initialize_data(
//         &mut self,
//         file_managers: &mut Vec<&mut FileManager>,
//     ) -> Result<(), io::Error> {
//         file_managers.iter_mut().try_for_each(|f| {
//             // Add project directory to all files
//             f.set_project_dir(self.path().join("data"));

//             f.initialize_append()
//         })?;

//         Ok(())
//     }

//     /// Returns reference to project's path
//     pub fn path(&self) -> &Path {
//         &self.path
//     }

//     /// Returns true if the project manager is in the recovery mode
//     pub fn is_recovery(&self) -> bool {
//         self.recovery_mode.is_some()
//     }

//     // Checks if the project directory already exists
//     fn exists(&self) -> bool {
//         self.path.is_dir()
//     }

//     // Checks if the project directory is timestamped
//     fn exists_and_timestamped(&self) -> bool {
//         let mut timestamped = false;

//         if self.exists() {
//             // Regex for the timestamps
//             let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();

//             // Get items of the directory
//             if let Ok(entries) = fs::read_dir(self.path()) {
//                 for entry in entries.flatten() {
//                     if let Ok(file_type) = entry.file_type() {
//                         // Only care about directories
//                         if file_type.is_dir() {
//                             if let Some(name) = entry.file_name().to_str() {
//                                 // Check if directory name matches timestamp
//                                 // format yyyy-mm-dd
//                                 if re.is_match(name) {
//                                     timestamped = true;
//                                     break;
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//         timestamped
//     }

//     // Creates a new manifest file
//     fn create_manifest(&self) -> ProjectManifest {
//         ProjectManifest::new()
//     }

//     // Loads a preexisting manifest file
//     fn load_manifest(&self) -> Result<ProjectManifest, io::Error> {
//         match Config::builder()
//             .add_source(config::File::from(self.path.join("manifest.toml")))
//             .build()
//         {
//             Ok(config) => {
//                 config.try_deserialize::<ProjectManifest>().map_err(|e| {
//                     io::Error::new(
//                         io::ErrorKind::InvalidData,
//                         format!("Failed to deserialize manifest file: {e}"),
//                     )
//                 })
//             }
//             Err(e) => Err(io::Error::new(
//                 io::ErrorKind::NotFound,
//                 format!(
//                     "manifest.toml not found in the project directory: {e}"
//                 ),
//             )),
//         }
//     }

//     // Prepare for a recovery of an existing project
//     fn initialize_recovery(&mut self) -> Result<(), io::Error> {
//         // Ensure that the directory exists
//         if !self.exists() {
//             Err(io::Error::new(
//                 io::ErrorKind::NotFound,
//                 format!(
//                     "Project directory scheduled for recovery does \
//                         not exist: {:#?}",
//                     self.path
//                 ),
//             ))?;
//         }

//         // Load manifest file
//         //let manifest = self.load_manifest()?;

//         // Determine the index of the last saved checkpoint
//         // Locate the checkpoint
//         // Create a new data directory
//         // Return Config
//         Ok(())
//     }

//     // Creates a new project, following data safety protocols
//     fn initialize_new(&mut self) -> Result<(), io::Error> {
//         // Check if the directory already exists, in which case execute data
//         // protocol
//         if self.exists() {
//             self.execute_data_protocol()?;
//         }

//         // Create project directory and manifest file
//         self.initialize_directory_structure()?;

//         Ok(())
//     }

//     // Method for safely dealing with the existing data
//     fn execute_data_protocol(&mut self) -> Result<(), io::Error> {
//         // Check if the existing directory is empty
//         // (if it is, no need for data protocols)
//         let mut entries = fs::read_dir(self.path())?;

//         if entries.next().is_some() {
//             match &self.data_protocol {
//                 DataProtocol::Archive => self.archive_protocol()?,
//                 DataProtocol::Panic => self.panic_protocol()?,
//             }
//         }

//         Ok(())
//     }

//     // Archives old data in a timestamped directory
//     fn archive_protocol(&mut self) -> Result<(), io::Error> {
//         if !self.exists_and_timestamped() {
//             // Copy data to a timestamped directory
//             let manifest = self.load_manifest()?;

//             let timestamp = manifest.timestamp();

//             // Create a temporary archive
//             let staging_path = self.path.join(".tmp_archive");

//             // Clean up any failed previous attempts
//             if staging_path.exists() {
//                 fs::remove_dir_all(&staging_path)?;
//             }

//             fs::create_dir(&staging_path)?;

//             // Snapshot existing files
//             let entries: Vec<PathBuf> = fs::read_dir(self.path())?
//                 .filter_map(|res| res.ok())
//                 .map(|e| e.path())
//                 .filter(|p| p != &staging_path)
//                 .collect();

//             // Atomic renames into staging
//             for path in entries {
//                 if let Some(name) = path.file_name() {
//                     let destination = staging_path.join(name);
//                     fs::rename(&path, &destination)?;
//                 }
//             }

//             // Atomic rename of the whole directory
//             let archive_path = self.path.join(timestamp);

//             fs::rename(&staging_path, &archive_path)?;
//         }
//         // Whether or not the data is archived, there will be timestamped
//         // directories. To deal with this, we create a new timestamped
//         // directory and append it to path

//         // Append the project path
//         self.path = self.path.join(self.create_manifest().timestamp());

//         // Create a new timestamp directory
//         fs::create_dir(self.path())?;

//         self.timestamped = false;

//         Ok(())
//     }

//     // Always returns an error
//     fn panic_protocol(&mut self) -> Result<(), io::Error> {
//         Err(io::Error::new(
//             io::ErrorKind::AlreadyExists,
//             "Project directory already exists.",
//         ))
//     }

//     // Creates the project directories
//     fn initialize_directory_structure(&mut self) -> Result<(), io::Error> {
//         // Create a manifest file
//         let manifest = self.create_manifest();

//         // Depending on the option, create a timestamped directory
//         if self.timestamped {
//             self.path = self.path.join(manifest.timestamp());

//             fs::create_dir_all(self.path())?;
//         }

//         // Save the manifest file
//         manifest.write(self.path())?;

//         // Create directory for error and debug messages
//         fs::create_dir_all(self.path().join("logs"))?;

//         // Create directory for simulation checkpoints
//         fs::create_dir_all(self.path().join("checkpoints"))?;

//         // Create directory for simulation output
//         fs::create_dir_all(self.path().join("data"))?;

//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {
    use crate::project::config::CURRENT_SCHEMA_VERSION;

    use super::*;
    use tempfile::tempdir;

    // Creates correct directory structure
    #[test]
    fn test_create_correct_layout() {
        // Get a path to a new temporary directory
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        assert!(
            ProjectManager::create("test", &None, &None, &path).is_ok(),
            "Failed to initialize a simple project directory"
        );

        assert!(&path.exists(), "Failed to initialize project directory");

        assert!(
            &path.with_file_name("manifest.toml").exists(),
            "Failed to create the manifest file"
        );

        assert!(
            &path.join("runs").exists(),
            "Failed to initialize root/runs/ directory"
        );

        assert!(
            &path.join("analysis").exists(),
            "Failed to initialize root/analysis/ directory"
        );
    }

    // Loading a created project returns identical config
    #[test]
    fn load_created_project() {
        // Get a path to a new temporary directory
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        let manager = ProjectManager::create("test", &None, &None, &path)
            .expect("Failed to initialize a simple project directory");

        let manager_load = ProjectManager::load(&path)
            .expect("Failed to load existing project");

        assert_eq!(
            manager.manifest, manager_load.manifest,
            "Loaded manifest differs from the created copy"
        );
    }

    // Creating project twice returns ProjectAlreadyExists
    #[test]
    fn test_create_twice() {
        // Get a path to a new temporary directory
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        assert!(
            ProjectManager::create("test", &None, &None, &path).is_ok(),
            "Failed to initialize a test project"
        );

        assert!(
            ProjectManager::create("test", &None, &None, &path).is_err_and(
                |e| {
                    if let ManagerError::ProjectAlreadyExists(p) = e {
                        p == path
                    } else {
                        false
                    }
                }
            ),
            "Unexpected second initialization of existing project"
        );
    }

    // Loading nonexistent path returns ProjectNotFound
    #[test]
    fn test_load_missing() {
        // Get a path to a new temporary directory
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        let result = ProjectManager::load(&path);

        assert!(
            result.is_err_and(|e| {
                if let ManagerError::ProjectNotFound(p) = e {
                    p == path
                } else {
                    false
                }
            }),
            "Unexpected load of project from non-existing source"
        );
    }

    // Old schema version returns SchemaMismatch
    #[test]
    fn load_different_schema() {
        // Get a path to a new temporary directory
        let path = tempdir()
            .expect("Failed to initialize a temporary directory.")
            .path()
            .join("project");

        let mut manager = ProjectManager::create("test", &None, &None, &path)
            .expect("Failed to initialize a simple project directory");

        manager.manifest.schema_version += 1;

        // This is an exceptional case for accessing schema version
        manager
            .manifest
            .write(&path)
            .expect("Failed to overwrite manifest.toml");

        assert!(ProjectManager::load(&path).is_err_and(|e| {
            if let ManagerError::SchemaMismatch {
                path,
                manifest_schema,
                current_schema,
            } = e
            {
                path == path
                    && manifest_schema == CURRENT_SCHEMA_VERSION + 1
                    && current_schema == CURRENT_SCHEMA_VERSION
            } else {
                false
            }
        }))
    }
}
