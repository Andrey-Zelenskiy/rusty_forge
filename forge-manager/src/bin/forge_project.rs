// Copyright Andrey Zelenskiy, 2024-2026

use std::{io, path::PathBuf};

use clap::Parser;

use config::Config;
use forge_manager::{project::ProjectManager, ManagerResult};
use serde::Deserialize;

/// Command line interface for loading data
#[derive(Debug, Default, Parser)]
#[command(
    name = "forge_project",
    about = "Initializes a new simulation project",
    long_about = None
)]
struct Cli {
    /// Path to config.toml
    config_file: Option<PathBuf>,
    // Options that override the config
    /// Project name
    name: Option<String>,
    /// Project path
    path: Option<PathBuf>,
    /// Author
    author: Option<String>,
    /// Project description
    description: Option<String>,
}

impl Cli {
    pub fn load_config(&self) -> ManagerResult<ProjectInitializer> {
        match &self.config_file {
            None => Ok(ProjectInitializer::default()),
            Some(path) => {
                if !path.exists() {
                    Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Config file not found: {}", path.display()),
                    )
                    .into())
                } else {
                    let config = Config::builder()
                        .add_source(config::File::from(path.as_path()))
                        .build()
                        .map_err(|e| {
                            io::Error::new(
                                io::ErrorKind::NotFound,
                                format!("Config file not found: {e}"),
                            )
                        })?;

                    config.get::<ProjectInitializer>("project").map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                            "Failed to deserialize project config file: {e}"
                        ),
                        )
                        .into()
                    })
                }
            }
        }
    }
}

/// Required data to initialize ProjectManager
#[derive(Debug, Deserialize)]
struct ProjectInitializer {
    name: String,
    path: PathBuf,
    author: Option<String>,
    description: Option<String>,
}

impl ProjectInitializer {
    /// New initializer
    pub fn from_cli(cli: &Cli) -> ManagerResult<Self> {
        // Load information from the config
        let mut initializer = cli.load_config()?;

        // Update data from command line arguments
        if let Some(name) = &cli.name {
            initializer.name = name.clone();
        }

        if let Some(path) = &cli.path {
            initializer.path = path.clone();
        }

        if cli.author.is_some() {
            initializer.author = cli.author.clone();
        }

        if cli.description.is_some() {
            initializer.description = cli.description.clone();
        }

        Ok(initializer)
    }

    /// Initialize the directory and the project manager
    pub fn initialize(&self) -> ManagerResult<ProjectManager> {
        ProjectManager::create(
            &self.name,
            &self.author,
            &self.description,
            &self.path,
        )
    }
}

impl Default for ProjectInitializer {
    fn default() -> Self {
        Self {
            name: "new_project".to_string(),
            path: PathBuf::from("./"),
            author: None,
            description: None,
        }
    }
}

fn main() {
    match ProjectInitializer::from_cli(&Cli::parse()) {
        Err(e) => {
            eprintln!("\n Config file parsing failed: {e}\n");
            std::process::exit(1);
        }
        Ok(initializer) => match initializer.initialize() {
            Err(e) => {
                eprintln!("\n Project directory initialization failed: {e}\n");
                std::process::exit(1);
            }
            Ok(manager) => {
                println!();

                println!("Project {} created.", manager.name());

                println!();

                println!("path:        {}", manager.path().display());

                if let Some(author) = manager.author() {
                    println!("author:      {author}");
                }

                if let Some(description) = manager.description() {
                    println!("description: {description}")
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::tempdir;

    use super::*;

    fn make_config_file(dir: &Path, content: &str) -> PathBuf {
        let path = dir.join("config.toml");
        fs::write(&path, content).expect("Failed to write config to file");

        path
    }

    // Tests loading of the information from config file
    #[test]
    fn test_load_project_from_config() {
        let dir =
            tempdir().expect("Failed to initialize a temporary directory");

        let cli = Cli {
            config_file: Some(make_config_file(
                dir.path(),
                r#"
                    [project]
                    name = "test_project"
                    path = "/tmp/test"
                    author = "Master Yoda"
                    description = "Long time ago, in a galaxy far far away"
                "#,
            )),
            name: None,
            path: None,
            author: None,
            description: None,
        };

        let initializer = ProjectInitializer::from_cli(&cli)
            .expect("Failed to read the config file");

        assert_eq!("test_project", initializer.name);
        assert_eq!(PathBuf::from("/tmp/test"), initializer.path);
        assert_eq!(
            "Master Yoda",
            &initializer
                .author
                .expect("Could not read author from config")
        );
        assert_eq!(
            "Long time ago, in a galaxy far far away",
            &initializer
                .description
                .expect("Could not read description from config")
        );
    }

    #[test]
    #[should_panic]
    fn test_wrong_config_path() {
        let dir =
            tempdir().expect("Failed to initialize a temporary directory");

        let cli = Cli {
            config_file: Some(make_config_file(
                dir.path(),
                r#"
                    [model]
                    parameter = 42
                "#,
            )),
            name: None,
            path: None,
            author: None,
            description: None,
        };

        let _ = ProjectInitializer::from_cli(&cli)
            .expect("Failed to read the config file");
    }

    #[test]
    fn test_default_for_empty() {
        let cli = Cli::default();

        let initializer = ProjectInitializer::from_cli(&cli)
            .expect("Failed to read the config file");

        assert_eq!("new_project", initializer.name);
        assert_eq!(PathBuf::from("./"), initializer.path);
        assert_eq!(None, initializer.author);
        assert_eq!(None, initializer.description);
    }

    #[test]
    fn test_project_initialization() {
        let dir =
            tempdir().expect("Failed to initialize a temporary directory");

        let path = dir.path().join("test_project");

        let cli = Cli {
            config_file: Some(make_config_file(
                dir.path(),
                r#"
                    [project]
                    name = "test_project"
                    path = ""
                    author = "Master Yoda"
                    description = "Long time ago, in a galaxy far far away"
                "#,
            )),
            name: None,
            path: Some(path.clone()),
            author: None,
            description: None,
        };

        let initializer = ProjectInitializer::from_cli(&cli)
            .expect("Failed to read the config file");

        initializer
            .initialize()
            .expect("Failed to initialize the project");

        assert!(path.with_file_name("manifest.toml").exists());
        assert!(path.join("analysis").exists());
        assert!(path.join("runs").exists());
    }
}
