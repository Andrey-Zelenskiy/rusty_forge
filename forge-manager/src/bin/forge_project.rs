// Copyright Andrey Zelenskiy, 2024-2026
use clap::Parser;

use forge_manager::prelude::*;

use forge_builder::prelude::*;

fn main() {
    match <ProjectManager as TargetFromBuilder>::Builder::from_cli(
        &ProjectCli::parse(),
    ) {
        Err(e) => {
            eprintln!("\n Config file parsing failed: {e}\n");
            std::process::exit(1);
        }
        Ok(mut builder) => match builder.build() {
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
    use std::{
        fs,
        path::{Path, PathBuf},
    };

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

        let cli = ProjectCli::new(
            Some(make_config_file(
                dir.path(),
                r#"
                    [project]
                    name = "test_project"
                    path = "/tmp/test"
                    author = "Master Yoda"
                    description = "Long time ago, in a galaxy far far away"
                "#,
            )),
            None,
            None,
            None,
            None,
        );

        let initializer =
            <ProjectManager as TargetFromBuilder>::Builder::from_cli(&cli)
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

        let cli = ProjectCli::new(
            Some(make_config_file(
                dir.path(),
                r#"
                    [model]
                    parameter = 42
                "#,
            )),
            None,
            None,
            None,
            None,
        );

        let _ = <ProjectManager as TargetFromBuilder>::Builder::from_cli(&cli)
            .expect("Failed to read the config file");
    }

    #[test]
    fn test_default_for_empty() {
        let cli = ProjectCli::default();

        let initializer =
            <ProjectManager as TargetFromBuilder>::Builder::from_cli(&cli)
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

        let cli = ProjectCli::new(
            Some(make_config_file(
                dir.path(),
                r#"
                    [project]
                    name = "test_project"
                    path = ""
                    author = "Master Yoda"
                    description = "Long time ago, in a galaxy far far away"
                "#,
            )),
            None,
            Some(path.clone()),
            None,
            None,
        );

        let mut initializer =
            <ProjectManager as TargetFromBuilder>::Builder::from_cli(&cli)
                .expect("Failed to read the config file");

        initializer
            .build()
            .expect("Failed to initialize the project");

        assert!(path.with_file_name("manifest.toml").exists());
        assert!(path.join("analysis").exists());
        assert!(path.join("runs").exists());
    }
}
