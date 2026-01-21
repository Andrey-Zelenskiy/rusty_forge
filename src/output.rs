// Copyright Andrey Zelenskiy, 2024-2025
use std::{fmt, io::Write};

use serde::Serialize;

use crate::{
    files::FileManager,
    initialize::{BuilderMethods, TargetFromBuilder},
};

/// OutputMethods - initialization and output setup
/// There are three types of output:
/// (1) Summary - detailed summary of the parameters and state properties,
/// implemented by the fmt::Display trait;
/// (2) State snapshot - state properties that characterize the object,
/// implemented by the Serialize trait;
/// (3) Builder parameters - properties that can be used to initialize a
/// builder of the structure in the future, implemented (indirectly) through
/// TargetFromBuilder and BuilderMethods;
pub trait OutputMethods:
    fmt::Display + Serialize + TargetFromBuilder<Builder: BuilderMethods>
{
    /// Name of the object
    fn name(&self) -> String;

    /// Return a reference to the summary file manager
    fn get_summary_file_manager(&self) -> &FileManager;

    /// Return a reference to the snapshot file manager
    fn get_state_file_manager(&self) -> &FileManager;

    /// Return a reference to the builder file manager
    fn get_builder_file_manager(&self) -> &FileManager;

    /// Return a mutable reference to the object's output file managers
    fn get_file_managers(&mut self) -> Vec<&mut FileManager>;

    /// Remove output that was rejected by the project manager
    fn resolve_output(&mut self);

    /// Save summary of the current state of the component
    fn save_summary(&self) {
        let file_manager = self.get_summary_file_manager();

        if file_manager.writable() {
            let mut file = file_manager.open_file();

            if let Err(reason) = writeln!(file, "{self}") {
                panic!(
                    "Couldn't write summary of {} to file: {reason}",
                    self.name()
                );
            }
        }
    }

    /// Save the state of the structure
    fn save_state(&self) {
        let file_manager = self.get_state_file_manager();

        if file_manager.writable() {
            let mut file = file_manager.open_file();

            match toml::to_string(&self) {
                Ok(data) => {
                    if let Err(reason) = write!(file, "{data}") {
                        panic!(
                            "Could not save the state of {} to file: {reason}",
                            self.name()
                        )
                    }
                }
                Err(reason) => panic!(
                    "Could not convert the {} structure to toml string: \
                    {reason}",
                    self.name()
                ),
            }
        }
    }

    // Convert the state to builder and save it to file
    fn save_builder(&self) {
        let file_manager = self.get_builder_file_manager();

        if file_manager.writable() {
            let builder = Self::Builder::from_target(self);

            let mut file = file_manager.open_file();

            match toml::to_string(&builder) {
                Ok(data) => {
                    if let Err(reason) = write!(file, "{data}") {
                        panic!(
                            "Could not save the builder of {} to file: {reason}",
                            self.name()
                        )
                    }
                }
                Err(reason) => panic!(
                    "Could not convert the {} builder to toml string: {reason}",
                    self.name()
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;

    use crate::initialize::BuilderMethods;

    use super::*;
    use serde::Deserialize;

    #[derive(Serialize)]
    pub struct TargetStruct {
        x2: u32,
        xy: u32,
        y2: u32,
        summary_file_manager: FileManager,
        state_file_manager: FileManager,
        builder_file_manager: FileManager,
    }

    impl Display for TargetStruct {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let summary = format!(
                "Test structure with \nx^2 = {},\nxy = {}\n y^2 = {}\n",
                self.x2, self.xy, self.y2
            );
            write!(f, "{summary}")
        }
    }

    #[derive(Deserialize, Default, Serialize)]
    pub struct Builder {
        x: u32,
        y: u32,
    }

    // Add methods for setting values
    impl Builder {
        pub fn set_x(&mut self, x: u32) -> &mut Self {
            self.x = x;
            self
        }

        pub fn set_y(&mut self, y: u32) -> &mut Self {
            self.y = y;
            self
        }
    }

    impl BuilderMethods for Builder {
        type Target = TargetStruct;

        fn build(&mut self) -> Self::Target {
            Self::Target {
                x2: self.x * self.x,
                xy: self.x * self.y,
                y2: self.y * self.y,
                summary_file_manager: FileManager::default(),
                state_file_manager: FileManager::default(),
                builder_file_manager: FileManager::default(),
            }
        }

        fn from_target(target: &TargetStruct) -> Self {
            Self {
                x: target.x2.isqrt(),
                y: target.y2.isqrt(),
            }
        }
    }

    impl TargetFromBuilder for TargetStruct {
        type Builder = Builder;
    }

    impl OutputMethods for TargetStruct {
        fn name(&self) -> String {
            String::from("test structure")
        }

        fn get_summary_file_manager(&self) -> &FileManager {
            &self.summary_file_manager
        }

        fn get_state_file_manager(&self) -> &FileManager {
            &self.state_file_manager
        }

        fn get_builder_file_manager(&self) -> &FileManager {
            &self.builder_file_manager
        }

        fn get_file_managers(&mut self) -> Vec<&mut FileManager> {
            vec![
                &mut self.summary_file_manager,
                &mut self.state_file_manager,
                &mut self.builder_file_manager,
            ]
        }

        fn resolve_output(&mut self) {}
    }

    #[test]
    fn output() {
        let target = TargetStruct::builder().set_x(1).set_y(2).build();

        target.save_summary();
        target.save_state();
        target.save_builder();
    }
}
