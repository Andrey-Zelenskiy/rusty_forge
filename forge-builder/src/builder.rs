// Copyright Andrey Zelenskiy, 2024-2026

//! This module defines methods for defining builders of custom types,
//! following the [Builder pattern](rust-unofficial.fithub.io/patterns/patterns/creational/builder.html).
//!
//! The key idea is to have two types: one with mutable data  (Builder) used
//! for the allocation, and the other (Target) with private data used for
//! performing functions:
//!
//! ```rust,ignore
//! let target = Target::builder() // Creates Builder::default()
//!   .set_field(value)            // Allocates data to the Builder,
//!   ...                          // returns &mut Builder
//!   .build();                    // Returns Result<Target, Error>
//! ```
//!
//! This can be implemented using a pair of traits, [BuilderMethods] and
//! [TargetFromBuilder], which respectively define the methods for
//! Builder and Target types.
//!
//! ```rust
//! use forge_builder::prelude::*;
//!
//! // Define the target structure
//! pub struct MyStruct {
//!     sum: i32,
//!     diff: i32,
//! }
//!
//! // Define the builder structure
//! #[derive(Default, serde::Deserialize, serde::Serialize)]
//! pub struct MyStructBuilder {
//!     x: Option<i32>,
//!     y: Option<i32>,
//! }
//!
//! // Define methods to allocate data to the builder
//! impl MyStructBuilder {
//!   pub fn set_x(&mut self, value: i32) -> &mut Self {
//!     self.x = Some(value);
//!     self
//!   }
//!
//!   pub fn set_y(&mut self, value: i32) -> &mut Self {
//!     self.y = Some(value);
//!     self
//!   }
//! }
//!
//! // Implement the required methods for building MyStruct
//! impl BuilderMethods for MyStructBuilder {
//!     // Which type are we constructing
//!     type Target = MyStruct;
//!
//!     // Logic to build the target from the data in the builder
//!     fn build(&mut self) -> Result<Self::Target, BuildError> {
//!         match (self.x, self.y) {
//!             (Some(x_value), Some(y_value)) => Ok(Self::Target {
//!                 sum: x_value + y_value,
//!                 diff: x_value - y_value,
//!             }),
//!             _ => Err(BuildError::IncompleteBuilderData {
//!                 reason: "Both x and y must be specified to \
//!                     initialize TargetOption. "
//!                     .to_string(),
//!             }),
//!         }
//!     }
//!
//!     // Logic to create the builder given the target data
//!     fn from_target(target: &Self::Target) -> Self {
//!         let x = (target.sum + target.diff) / 2;
//!         let y = (target.sum - target.diff) / 2;
//!         Self {
//!             x: Some(x),
//!             y: Some(y),
//!         }
//!     }
//!
//! }
//!
//! impl TargetFromBuilder for MyStruct {
//!     type Builder = MyStructBuilder;
//! }
//! ```
//! A couple of comments are worth making at this point.
//! Note that the logic defined in [BuilderMethods] and [TargetFromBuilder]
//! assumes one-to-one relationship, i.e. there can only be one builder for
//! a given target.
//! The [BuilderMethods::from_target()] method is required in order to easily
//! serialize and output the builder data, for future initializations.
//!
//! ## [BuilderSetter](builder_derive/derive.BuilderSetters.html) derive macro
//!
//! To eliminate boilerplate, [forge-derive] module implements
//! a derive macro for automatic definition of setters for both Structs and
//! Enums.
//! The `impl MyStructBuilder` block in the example above can be removed by
//! adding a new derive trait:
//!
//! ```rust,ignore
//! use forge_builder::prelude::*;
//!
//! // Define the target structure
//! pub struct MyStruct {
//!     sum: i32,
//!     diff: i32,
//! }
//!
//! // Define the builder structure
//! #[derive(BuilderSetters, Default, serde::Deserialize, serde::Serialize)]
//! pub struct MyStructBuilder {
//!     x: Option<i32>,
//!     y: Option<i32>,
//! }
//!
//! // Same implementations of BuilderMethods and TargetFromBuilder as above
//! ```
//! In this example, [BuilderSetters](builder_derive/derive.BuilderSetters.html)
//! automatically defines `.set_x()` and `.set_y()`, which take in [i32] and
//! set the values to [`Some<T>`].
//! For non-option values, this derive macro defines a setter to the respective
//! field type:
//!
//! ```rust
//! use forge_builder::prelude::*;
//!
//! #[derive(BuilderSetters)]
//! struct Builder {
//!   x: i32,
//!   y: Option<i32>
//! }
//!
//! // Same as
//! //
//! // impl Builder {
//! //   pub fn set_x(value: i32) -> &mut Self {
//! //     self.x = value;
//! //     self
//! //   }
//! //
//! //   pub fn set_y(value: i32) -> &mut Self {
//! //     self.y = Some(value);
//! //     self
//! //   }
//! // }
//!
//! ```
//!
//! If one of the fields of the structure already implements
//! [BuilderSetter](builder_derive/derive.BuilderSetters.html),
//! we can indicate it by a special attribute:
//!
//! ```rust
//! use forge_builder::prelude::*;
//!
//! #[derive(BuilderSetters, Default)]
//! struct Complex {
//!   real: f64,
//!   imag: f64
//! }
//!
//! #[derive(BuilderSetters, Default)]
//! struct Builder {
//!   x: f64,
//!   y: Option<f64>,
//!   #[setter(nested)]
//!   z: Complex,
//! }
//!
//! // This allows us to define all nested fields
//! let builder = Builder::default()
//!   .set_x(1.0)
//!   .set_y(0.0)
//!   .set_z(|s| s.set_real(1.0).set_imag(0.0));
//!```
//!
//! This trait works also on Enums:
//!
//! ```rust
//! use forge_builder::prelude::*;
//!
//! // Enum representing initialization options of a float scalar
//! #[derive(BuilderSetters, Default)]
//! enum ScalarBuilder {
//!   // Initializes to zero, no parameters needed
//!   #[default]
//!   Zero,
//!   // Initializes to the input value
//!   FromValue(f64),
//!   // Initializes from a uniform random distribution with specified bounds
//!   UniformDistribution {min: f64, max: f64},
//! }
//!
//! let builder_value = ScalarBuilder::default()
//!   .set_zero();
//!
//! let builder_value = ScalarBuilder::default()
//!   .set_from_value(1.0);
//!
//! let builder_value = ScalarBuilder::default()
//!   .set_uniform_distribution(-1.0, 1.0);
//! ```
//!
//! ## [BuilderFromTargets](builder_derive/derive.BuilderFromTargets.html) derive macro
//!
//! If we are dealing with a structure whose fields are types that implement
//! [TargetFromBuilder], then we can use
//! [BuilderFromTargets](builder_derive/derive.BuilderFromTargets.html) macro to
//! automatically define a corresponding builder structure:
//!
//! ```rust
//! use forge_builder::prelude::*;
//!
//! use std::path::PathBuf;
//!
//! pub struct MyStruct {
//!     sum: i32,
//!     diff: i32,
//! }
//!
//! #[derive(BuilderSetters, Default, serde::Deserialize, serde::Serialize)]
//! pub struct MyStructBuilder {
//!     x: Option<i32>,
//!     y: Option<i32>,
//! }
//!
//! // Implement the required methods for building MyStruct
//! impl BuilderMethods for MyStructBuilder {
//!     type Target = MyStruct;
//!
//!     fn build(&mut self) -> Result<Self::Target, BuildError> {
//!         match (self.x, self.y) {
//!             (Some(x_value), Some(y_value)) => Ok(Self::Target {
//!                 sum: x_value + y_value,
//!                 diff: x_value - y_value,
//!             }),
//!             _ => Err(BuildError::IncompleteBuilderData {
//!                 reason: "Both x and y must be specified to \
//!                     initialize MyStruct. "
//!                     .to_string(),
//!             }),
//!         }
//!     }
//!
//!     fn from_target(target: &Self::Target) -> Self {
//!         let x = (target.sum + target.diff) / 2;
//!         let y = (target.sum - target.diff) / 2;
//!         Self {
//!             x: Some(x),
//!             y: Some(y),
//!         }
//!     }
//!
//! }
//!
//! impl TargetFromBuilder for MyStruct {
//!     type Builder = MyStructBuilder;
//! }
//!
//! // Automatically create a builder struct with derived BuilderSetter,
//! // and implement TargetFromBuilder for Target
//! #[derive(BuilderFromTargets)]
//! pub struct Target {
//!   #[builder(nested)] // Flag the fields that will use nested setters
//!   t1: MyStruct,
//!   #[builder(nested)]
//!   t2: MyStruct,
//!   scalar: f64,       // Simple types implement BuilderMethods and
//!   vector: Vec<f64>,  // TargetFromBuilder as their own Builders and
//!   path: PathBuf,     // Targets
//! }
//!
//! let target = Target::builder()      // Creates TargetBuilder::default()
//!   .set_t1(|s| s.set_x(1).set_y(1))  // Allocates data to the two fields
//!   .set_t2(|s| s.set_x(1).set_y(2))  // with custom Builders
//!   .set_scalar(2.0)                  // Allocate fields that are their own
//!   .set_vector(vec![1.0, 1.0, 0.0])  // Builders
//!   .set_path("/tmp/")                //
//!   .build()
//!   .unwrap();                        // Creates Target structure
//!
//! ```
//!
//! Since [BuilderMethods] and [TargetFromBuilder] traits are implemented for
//! many common types, such as scalaras and arrays, one can further minimize
//! the number of manual implementations of these traits.

use std::{error::Error, fmt, path::PathBuf};

use config::{Config, ConfigError};

use serde::{Deserialize, Serialize};

/// Trait for argument structure with required initialization function
pub trait BuilderMethods:
    Default + for<'de> Deserialize<'de> + Serialize + Sync + Clone
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
    T: Clone + Serialize + for<'de> Deserialize<'de> + Default + Sync,
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
    T: Clone + Serialize + for<'de> Deserialize<'de> + Default + Sync,
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
                T: Clone + Serialize + for<'de> Deserialize<'de> + Default + Copy + Sync,
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
                T: Clone + Serialize + for<'de> Deserialize<'de> + Default + Copy + Sync,
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
    use forge_derive::{BuilderFromTargets, BuilderSetters};
    use serde::Deserialize;

    pub struct TargetExplicit {
        x2: u32,
        xy: u32,
        y2: u32,
    }

    #[derive(BuilderSetters, Deserialize, Default, Serialize, Clone)]
    pub struct BuilderExplicit {
        x: u32,
        y: u32,
    }

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

    #[derive(BuilderSetters, Default, Deserialize, Serialize, Clone)]
    pub struct BuilderOption {
        x: Option<i32>,
        y: Option<i32>,
    }

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
        #[builder(nested)]
        t1: TargetExplicit,
        #[builder(nested)]
        t2: TargetExplicit,
        #[builder(nested)]
        t3: TargetOption,
        scalar: f64,
        vector: Vec<f64>,
        string: String,
        path: PathBuf,
    }

    #[derive(Default, BuilderSetters)]
    pub enum Builder {
        #[allow(dead_code)]
        UniformDistribution { min: f64, max: f64 },
        #[default]
        Zero,
        #[allow(dead_code)]
        FromValue(f64),
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
    fn enum_setter() {
        let _ = Builder::default()
            .set_zero()
            .set_from_value(1.0)
            .set_uniform_distribution(-1.0, 1.0);
    }

    #[test]
    fn builder_macro() {
        let target = TargetComposite::builder()
            .set_t1(|s| s.set_x(1_u32).set_y(2_u32))
            .set_t2(|s| s.set_x(2_u32).set_y(3_u32))
            .set_t3(|s| s.set_x(1).set_y(2))
            .set_scalar(1.0)
            .set_vector(vec![1.0, 2.0, 3.0])
            .set_string("one")
            .set_path("/tmp/")
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

        assert_eq!(1.0, target.scalar);
        assert_eq!(vec![1.0, 2.0, 3.0], target.vector);
        assert_eq!("one", target.string);
        assert_eq!(PathBuf::from("/tmp/"), target.path);
    }
}
