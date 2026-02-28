// Copyright Andrey Zelenskiy, 2024-2026

use std::{
    error::Error,
    fmt::{self, Display},
};

use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

/// Trait for argument structure with required initialization function
pub trait BuilderMethods:
    Default + for<'de> Deserialize<'de> + Serialize
{
    type Target;

    /// User implementation of the target build
    fn try_build(&mut self) -> Result<Self::Target, String>;

    /// Initialize target structure from the parameters
    fn build(&mut self) -> Result<Self::Target, BuildError> {
        self.try_build()
            .map_err(|s| BuildError::IncompleteBuilderData { reason: s })
    }

    /// Create a builder from the existing target
    fn from_target(target: &Self::Target) -> Self;
}

/// Trait for initializing a structure from an argument structure
pub trait TargetFromBuilder {
    type Builder: BuilderMethods<Target = Self>;
    // Initialize new Target from input parameters
    fn builder() -> Self::Builder {
        Self::Builder::default()
    }

    // Initialize Target from a config file
    fn from_config(
        config: &Config,
        config_name: &str,
    ) -> Result<Self, BuildError>
    where
        Self: Sized,
    {
        //Populate the parameters from the config
        match config.get::<Self::Builder>(config_name) {
            Ok(mut builder) => builder.build(),
            Err(reason) => panic!("Missing config for {config_name}: {reason}"),
        }
    }
}

/// Error handling for the builder pattern
#[derive(Debug)]
pub enum BuildError {
    // Issue with the config file
    ConfigError { reason: String },
    // Some required items have not been specified in the builder
    IncompleteBuilderData { reason: String },
}

impl Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigError { reason } => {
                writeln!(f, "Could not build target from config: {reason}")
            }
            Self::IncompleteBuilderData { reason } => {
                writeln!(f, "Builder missing required fields: {reason}")
            }
        }
    }
}

impl Error for BuildError {}

impl From<ConfigError> for BuildError {
    fn from(value: ConfigError) -> Self {
        Self::ConfigError {
            reason: value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    pub struct TargetExplicit {
        x2: u32,
        xy: u32,
        y2: u32,
    }

    #[derive(Deserialize, Default, Serialize)]
    pub struct BuilderExplicit {
        x: u32,
        y: u32,
    }

    // Add methods for setting values
    impl BuilderExplicit {
        pub fn set_x(&mut self, x: u32) -> &mut Self {
            self.x = x;
            self
        }

        pub fn set_y(&mut self, y: u32) -> &mut Self {
            self.y = y;
            self
        }
    }

    impl BuilderMethods for BuilderExplicit {
        type Target = TargetExplicit;

        fn try_build(&mut self) -> Result<Self::Target, String> {
            Ok(Self::Target {
                x2: self.x * self.x,
                xy: self.x * self.y,
                y2: self.y * self.y,
            })
        }

        fn from_target(target: &Self::Target) -> Self {
            Self {
                x: target.x2.isqrt(),
                y: target.y2.isqrt(),
            }
        }
    }

    impl TargetFromBuilder for TargetExplicit {
        type Builder = BuilderExplicit;
    }

    pub struct TargetOption {
        sum: i32,
        diff: i32,
    }

    #[derive(Default, Deserialize, Serialize)]
    pub struct BuilderOption {
        x: Option<i32>,
        y: Option<i32>,
    }

    impl BuilderOption {
        pub fn set_x(&mut self, value: i32) -> &mut Self {
            self.x = Some(value);
            self
        }

        pub fn set_y(&mut self, value: i32) -> &mut Self {
            self.y = Some(value);
            self
        }
    }

    impl BuilderMethods for BuilderOption {
        type Target = TargetOption;

        fn try_build(&mut self) -> Result<Self::Target, String> {
            match (self.x, self.y) {
                (Some(x_value), Some(y_value)) => Ok(TargetOption {
                    sum: x_value + y_value,
                    diff: x_value - y_value,
                }),
                _ => Err("Both x and y must be specified to \
                        initialize TargetOption. "
                    .to_string()),
            }
        }

        fn from_target(target: &Self::Target) -> Self {
            let x = (target.sum + target.diff) / 2;
            let y = (target.sum - target.diff) / 2;

            Self {
                x: Some(x),
                y: Some(y),
            }
        }
    }

    impl TargetFromBuilder for TargetOption {
        type Builder = BuilderOption;
    }

    #[test]
    fn build() {
        let target = TargetExplicit::builder()
            .set_x(1)
            .set_y(2)
            .build()
            .expect("Failed to build the test structure");

        assert_eq!(1, target.x2);
        assert_eq!(2, target.xy);
        assert_eq!(4, target.y2);
    }

    #[test]
    fn builder_from_target() {
        let target = TargetExplicit {
            x2: 1,
            xy: 2,
            y2: 4,
        };
        let builder = BuilderExplicit::from_target(&target);

        assert_eq!(builder.x, 1);
        assert_eq!(builder.y, 2);
    }

    #[test]
    fn incomplete_build() {
        assert!(TargetOption::builder().set_x(1).build().is_err());
        assert!(TargetOption::builder().set_y(1).build().is_err());
    }
}
