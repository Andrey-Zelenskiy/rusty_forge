// Copyright Andrey Zelenskiy, 2024-2026

use std::{error::Error, fmt, path::PathBuf};

use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

/// Trait for argument structure with required initialization function
pub trait BuilderMethods:
    Default + for<'de> Deserialize<'de> + Serialize
{
    type Target;

    /// Initialize target structure from the parameters
    fn build(&mut self) -> Result<Self::Target, BuildError>;

    /// Create a builder from the existing target
    fn from_target(target: &Self::Target) -> Self;
}

/// Trait for initializing a structure from an argument structure
pub trait TargetFromBuilder {
    type Builder: BuilderMethods<Target = Self>;
    /// Initialize new Target from input parameters
    fn builder() -> Self::Builder {
        Self::Builder::default()
    }

    /// Initialize Target from a config file
    fn from_config(
        config: &Config,
        config_name: &str,
    ) -> Result<Self, BuildError>
    where
        Self: Sized,
    {
        //Populate the parameters from the config
        let mut builder = config
            .get::<Self::Builder>(config_name)
            .map_err(BuildError::from)?;

        builder.build()
    }
}

/// Error handling for the builder pattern
#[derive(Debug)]
pub enum BuildError {
    /// Issue with the config file
    ConfigError { reason: String },
    /// Some required items have not been specified in the builder
    IncompleteBuilderData { reason: String },
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigError { reason } => {
                write!(f, "Could not build target from config: {reason}")
            }
            Self::IncompleteBuilderData { reason } => {
                write!(f, "Builder missing required fields: {reason}")
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

// Blanket implementation for simple targets
macro_rules! impl_builder_for_simple_target {
    ($($t:ty),*) => {
        $(
            impl BuilderMethods for $t {
                type Target = $t;

                fn build(&mut self) -> Result<Self::Target, BuildError> {
                    // For simple types, building is just returning the value
                    Ok(self.clone())
                }

                fn from_target(target: &Self::Target) -> Self {
                    target.clone()
                }
            }

            impl TargetFromBuilder for $t {
                type Builder = $t;
            }
        )*
    };
}

impl_builder_for_simple_target!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64,
    bool, String, PathBuf
);

// Blanket implementation for Option<T>
impl<T> BuilderMethods for Option<T>
where
    T: BuilderMethods<Target = T>,
{
    type Target = Option<T>;

    fn build(&mut self) -> Result<Self::Target, BuildError> {
        match self {
            Some(value) => Ok(Some(value.build()?)),
            None => Ok(None),
        }
    }

    fn from_target(target: &Self::Target) -> Self {
        target.as_ref().map(|t| T::from_target(t))
    }
}

impl<T> TargetFromBuilder for Option<T>
where
    T: TargetFromBuilder<Builder = T> + BuilderMethods<Target = T>,
{
    type Builder = Option<T::Builder>;
}

// Blanket implementation for Vec<T>
impl<T> BuilderMethods for Vec<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Default,
{
    type Target = Vec<T>;

    fn build(&mut self) -> Result<Self::Target, BuildError> {
        Ok(self.clone())
    }

    fn from_target(target: &Self::Target) -> Self {
        target.clone()
    }
}

impl<T> TargetFromBuilder for Vec<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Default,
{
    type Builder = Vec<T>;
}

// Blanket implementation for [T;N]
// Note: Unfortunately, Default, Serialize, and Deserialize are only
// implemented for N <=32 (even though generic const exists).
// Until this is resolved in the future versions of std and serde,
// this library will mimic this implementation
macro_rules! impl_builder_for_sized_arrays {
    ($($N:expr),*) => {
        $(
            impl<T> BuilderMethods for [T; $N]
            where
                T: Clone + Serialize + for<'de> Deserialize<'de> + Default + Copy,
            {
                type Target = [T; $N];

                fn build(&mut self) -> Result<Self::Target, BuildError> {
                    Ok(*self)
                }

                fn from_target(target: &Self::Target) -> Self {
                    *target
                }
            }

            impl<T> TargetFromBuilder for [T; $N]
            where
                T: Clone + Serialize + for<'de> Deserialize<'de> + Default + Copy,
            {
                type Builder = [T; $N];
            }
        )*
    };
}

impl_builder_for_sized_arrays!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
);

#[cfg(test)]
mod tests {
    use super::*;
    use builder_derive::{BuilderFromTargets, BuilderSetters};
    use serde::Deserialize;

    pub struct TargetExplicit {
        x2: u32,
        xy: u32,
        y2: u32,
    }

    #[derive(BuilderSetters, Deserialize, Default, Serialize)]
    pub struct BuilderExplicit {
        x: u32,
        y: u32,
    }

    // Add methods for setting values
    // impl BuilderExplicit {
    //     pub fn set_x(&mut self, x: u32) -> &mut Self {
    //         self.x = x;
    //         self
    //     }

    //     pub fn set_y(&mut self, y: u32) -> &mut Self {
    //         self.y = y;
    //         self
    //     }
    // }

    impl BuilderMethods for BuilderExplicit {
        type Target = TargetExplicit;

        fn build(&mut self) -> Result<Self::Target, BuildError> {
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

    #[derive(BuilderSetters, Default, Deserialize, Serialize)]
    pub struct BuilderOption {
        x: Option<i32>,
        y: Option<i32>,
    }

    // impl BuilderOption {
    //     pub fn set_x(&mut self, value: i32) -> &mut Self {
    //         self.x = Some(value);
    //         self
    //     }

    //     pub fn set_y(&mut self, value: i32) -> &mut Self {
    //         self.y = Some(value);
    //         self
    //     }
    // }

    impl BuilderMethods for BuilderOption {
        type Target = TargetOption;

        fn build(&mut self) -> Result<Self::Target, BuildError> {
            match (self.x, self.y) {
                (Some(x_value), Some(y_value)) => Ok(TargetOption {
                    sum: x_value + y_value,
                    diff: x_value - y_value,
                }),
                _ => Err(BuildError::IncompleteBuilderData {
                    reason: "Both x and y must be specified to \
                        initialize TargetOption. "
                        .to_string(),
                }),
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

    #[derive(BuilderFromTargets)]
    pub struct TargetComposite {
        t1: TargetExplicit,
        t2: TargetExplicit,
        t3: TargetOption,
    }

    #[test]
    fn build() {
        let target = TargetExplicit::builder()
            .set_x(1_u32)
            .set_y(2_u32)
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

    #[test]
    fn builder_macro() {
        let target = TargetComposite::builder()
            .set_t1(|s| s.set_x(1_u32).set_y(2_u32))
            .set_t2(|s| s.set_x(2_u32).set_y(3_u32))
            .set_t3(|s| s.set_x(1).set_y(2))
            .build()
            .expect("Failed to build a composite target");

        assert_eq!(1, target.t1.x2);
        assert_eq!(2, target.t1.xy);
        assert_eq!(4, target.t1.y2);

        assert_eq!(4, target.t2.x2);
        assert_eq!(6, target.t2.xy);
        assert_eq!(9, target.t2.y2);

        assert_eq!(3, target.t3.sum);
        assert_eq!(-1, target.t3.diff);
    }
}
