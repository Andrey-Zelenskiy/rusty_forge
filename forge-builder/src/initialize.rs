// Copyright Andrey Zelenskiy, 2024-2026

//! # Module for initializing scalar and array values
//!
//! This module provides four enums for safe initialization of scalar and array
//! values (float and integers).
//! The values are initialized either from common random distributions, or to
//! specified values.

use std::{error::Error, fmt, path::PathBuf};

use config::Config;
use rand::rng;

use rand_distr::{
    weighted::WeightedIndex, Binomial, Distribution, Normal, StandardNormal,
    Uniform,
};

use serde::{Deserialize, Serialize};

/// Error type returned when initialization fails
#[derive(Debug)]
pub enum InitializerError {
    /// Issue with random distribution parameters (returns message from one of
    /// the errors from [`rand_distr`]).
    DistributionParametersError { reason: String },
    /// Mismatch in array size
    ArraySizeMismatch {
        size_required: usize,
        size_actual: usize,
    },
    /// Config error
    ConfigError { reason: String },
}

impl fmt::Display for InitializerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DistributionParametersError { reason } => {
                write!(
                    f,
                    "Improper parameters in the random distribution: {reason}"
                )
            }
            Self::ArraySizeMismatch {
                size_required,
                size_actual,
            } => {
                write!(
                    f,
                    "Input array has different size ({size_actual}) than \
                    required for initialization ({size_required})."
                )
            }
            Self::ConfigError { reason } => {
                write!(
                    f,
                    "Array could not be loaded from the config file: {reason}"
                )
            }
        }
    }
}

impl Error for InitializerError {}

/// Initialization options for float values (f64).
///
/// ## Example
///
/// ```
/// use rusty_forge::initialize::FloatScalarInitializer;
///
/// // Returns a value uniformly sampled between -1 and 1
/// let value = FloatScalarInitializer::default()
///   .initialize()
///   .unwrap();
///
/// assert!(value >= -1.0);
///
/// assert!(value <= 1.0);
///
/// ```
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(tag = "type")]
pub enum FloatScalarInitializer {
    /// Random float value from uniform distribution in range (min, max)
    RandomUniform { min: f64, max: f64 },
    /// Random float value from normal distribution
    RandomNormal { mean: f64, std: f64 },
    /// Float value from input
    Value { value: f64 },
}

impl Default for FloatScalarInitializer {
    fn default() -> Self {
        Self::RandomUniform {
            min: -1.0,
            max: 1.0,
        }
    }
}

impl FloatScalarInitializer {
    /// Creates initializer from a uniform distribution with specified bounds.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::FloatScalarInitializer;
    ///
    /// // Returns a value uniformly sampled between 0 and 1
    /// let value = FloatScalarInitializer::new_uniform(0.0, 1.0)
    ///   .initialize()
    ///   .unwrap();
    /// ```
    pub fn new_uniform(min: f64, max: f64) -> Self {
        Self::RandomUniform { min, max }
    }

    /// Creates initializer from a normal distribution with specified mean and
    /// standard deviation.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::FloatScalarInitializer;
    ///
    /// // Returns a value sampled from standard normal distribution
    /// let value = FloatScalarInitializer::new_normal(0.0, 1.0)
    ///   .initialize()
    ///   .unwrap();
    /// ```
    pub fn new_normal(mean: f64, std: f64) -> Self {
        Self::RandomNormal { mean, std }
    }

    /// Creates initializer to a specified value.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::FloatScalarInitializer;
    ///
    /// // Returns value set to 1.0
    /// let value = FloatScalarInitializer::new_value(1.0)
    ///   .initialize()
    ///   .unwrap();
    /// ```
    pub fn new_value(value: f64) -> Self {
        Self::Value { value }
    }

    /// Initializes a float value based on the initialization variant
    pub fn initialize(&self) -> Result<f64, InitializerError> {
        match self {
            Self::RandomUniform { min, max } => {
                let distribution = Uniform::new(min, max).map_err(|e| {
                    InitializerError::DistributionParametersError {
                        reason: e.to_string(),
                    }
                })?;

                Ok(distribution.sample(&mut rng()))
            }
            Self::RandomNormal { mean, std } => {
                let distribution = Normal::new(*mean, *std).map_err(|e| {
                    InitializerError::DistributionParametersError {
                        reason: e.to_string(),
                    }
                })?;

                Ok(distribution.sample(&mut rng()))
            }
            Self::Value { value, .. } => Ok(*value),
        }
    }
}

/// Initialization options for integer values (u32).
///
/// ## Example
///
/// ```
/// use rusty_forge::initialize::IntegerScalarInitializer;
///
/// // Returns a value uniformly sampled integer between 0 and 10
/// let value = IntegerScalarInitializer::default()
///   .initialize()
///   .unwrap();
///
/// assert!(value >= 0);
///
/// assert!(value <= 10);
///
/// ```
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(tag = "type")]
pub enum IntegerScalarInitializer {
    /// Random integer value from uniform distribution in range (min, max)
    RandomUniform { min: u32, max: u32 },
    /// Random integer value from binomial distribution
    RandomBinomial { n: u64, p: f64 },
    /// Integer value from a configuration file
    Value { value: u32 },
}

impl Default for IntegerScalarInitializer {
    fn default() -> Self {
        Self::RandomUniform { min: 0, max: 10 }
    }
}

impl IntegerScalarInitializer {
    /// Creates initializer from a uniform distribution with specified bounds.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::IntegerScalarInitializer;
    ///
    /// // Returns a value uniformly sampled between 0 and 10
    /// let value = IntegerScalarInitializer::new_uniform(0, 10)
    ///   .initialize()
    ///   .unwrap();
    /// ```
    pub fn new_uniform(min: u32, max: u32) -> Self {
        Self::RandomUniform { min, max }
    }

    /// Creates initializer from a binomial distribution with specified number
    /// of trials n and trial probability p.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::IntegerScalarInitializer;
    ///
    /// // Returns a value sampled from binomial distribution
    /// let value = IntegerScalarInitializer::new_binomial(10, 0.5)
    ///   .initialize()
    ///   .unwrap();
    /// ```
    pub fn new_binomial(n: u64, p: f64) -> Self {
        Self::RandomBinomial { n, p }
    }

    /// Creates initializer to a specified value.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::IntegerScalarInitializer;
    ///
    /// // Returns value set to 1
    /// let value = IntegerScalarInitializer::new_value(1)
    ///   .initialize()
    ///   .unwrap();
    /// ```
    pub fn new_value(value: u32) -> Self {
        Self::Value { value }
    }

    /// Initialize an integer value based on the initialization variant
    pub fn initialize(&self) -> Result<u32, InitializerError> {
        match self {
            Self::RandomUniform { min, max } => {
                let distribution = Uniform::new(min, max).map_err(|e| {
                    InitializerError::DistributionParametersError {
                        reason: e.to_string(),
                    }
                })?;

                Ok(distribution.sample(&mut rng()))
            }
            Self::RandomBinomial { n, p } => {
                let distribution = Binomial::new(*n, *p).map_err(|e| {
                    InitializerError::DistributionParametersError {
                        reason: e.to_string(),
                    }
                })?;

                Ok(distribution.sample(&mut rng()) as u32)
            }
            Self::Value { value, .. } => Ok(*value),
        }
    }
}

/// Initialization options for float arrays (f64)
///
/// ## Example
///
/// ```
/// use rusty_forge::initialize::FloatArrayInitializer;
///
/// // Returns size 10 vector of floats uniformly sampled between -1 and 1
/// let array = FloatArrayInitializer::default()
///   .initialize(10)
///   .unwrap();
///
/// array.iter()
///   .for_each(|a| {
///     assert!(*a >= -1.0);
///     assert!(*a <= 1.0);
///   });
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum FloatArrayInitializer {
    /// Random array components from uniform distribution in range (min, max)
    RandomUniform { min: f64, max: f64 },
    /// Random array components from normal distribution
    RandomNormal { mean: f64, std: f64 },
    /// Random array as a point on a surface of a sphere
    RandomSpherical,
    /// Random array sampled from a specified set of values with relative
    /// weights
    RandomWeighted { values: Vec<f64>, weights: Vec<f64> },
    /// Array of identical values
    Value { value: f64 },
    /// Array from input
    Array { array: Vec<f64> },
    /// Array from a configuration file
    FromConfig { path: PathBuf, name: String },
}

impl Default for FloatArrayInitializer {
    fn default() -> Self {
        Self::RandomUniform {
            min: -1.0,
            max: 1.0,
        }
    }
}

impl FloatArrayInitializer {
    /// Creates initializer from a uniform distribution with specified bounds.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::FloatArrayInitializer;
    ///
    /// // Returns size 10 vector of floats uniformly sampled between -1 and 1
    /// let array = FloatArrayInitializer::new_uniform(-1.0, 1.0)
    ///   .initialize(10)
    ///   .unwrap();
    /// ```
    pub fn new_uniform(min: f64, max: f64) -> Self {
        Self::RandomUniform { min, max }
    }

    /// Creates initializer from a normal distribution with specified mean and
    /// standard deviation.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::FloatArrayInitializer;
    ///
    /// // Returns size 10 vector of floats sampled from standard normal
    /// // distribution
    /// let array = FloatArrayInitializer::new_normal(0.0, 1.0)
    ///   .initialize(10)
    ///   .unwrap();
    /// ```
    pub fn new_normal(mean: f64, std: f64) -> Self {
        Self::RandomNormal { mean, std }
    }

    /// Creates initializer from a uniform distribution on a surface of a
    /// high-dimensional sphere.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::FloatArrayInitializer;
    ///
    /// // Returns size 10 vector of floats sampled on a surface of a
    /// // 10-dimensional sphere
    /// let array = FloatArrayInitializer::new_spherical()
    ///   .initialize(10)
    ///   .unwrap();
    /// ```
    pub fn new_spherical() -> Self {
        Self::RandomSpherical
    }

    /// Creates initializer from a weighted distribution with a finite number
    /// of specified allowed values and the corresponding weights.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::FloatArrayInitializer;
    ///
    /// // Returns size 10 vector of floats values in [-1, 0, 1], with
    /// // corresponding weights equal to [0.1, 0.5, 0.4]
    /// let array = FloatArrayInitializer::new_weighted(
    ///   &[-1.0, 0.0 ,1.0],
    ///   &[0.1, 0.5, 0.4],
    /// )
    ///   .initialize(10)
    ///   .unwrap();
    /// ```
    pub fn new_weighted(values: &[f64], weights: &[f64]) -> Self {
        Self::RandomWeighted {
            values: values.into(),
            weights: weights.into(),
        }
    }

    /// Creates initializer to a single value.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::FloatArrayInitializer;
    ///
    /// // Returns size 10 vector of floats equal to 1.0
    /// let array = FloatArrayInitializer::new_value(1.0)
    ///   .initialize(10)
    ///   .unwrap();
    /// ```
    pub fn new_value(value: f64) -> Self {
        Self::Value { value }
    }

    /// Creates initializer to an input array.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::FloatArrayInitializer;
    ///
    /// // Returns size 10 vector of floats equal to [1, ..., 10]
    /// let array = FloatArrayInitializer::new_array(
    ///   &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
    ///   )
    ///   .initialize(10)
    ///   .unwrap();
    /// ```
    pub fn new_array(array: &[f64]) -> Self {
        Self::Array {
            array: array.into(),
        }
    }

    /// Creates initializer to an input array in a config at a given path.
    pub fn new_config<P: AsRef<PathBuf>>(path: P, name: String) -> Self {
        Self::FromConfig {
            path: path.as_ref().to_path_buf(),
            name,
        }
    }

    /// Initialize an array of floats based on the initialization variant
    pub fn initialize(
        &self,
        size: usize,
    ) -> Result<Vec<f64>, InitializerError> {
        match self {
            Self::RandomUniform { min, max } => {
                let distribution = Uniform::new(min, max).map_err(|e| {
                    InitializerError::DistributionParametersError {
                        reason: e.to_string(),
                    }
                })?;

                Ok(distribution.sample_iter(&mut rng()).take(size).collect())
            }
            Self::RandomNormal { mean, std } => {
                let distribution = Normal::new(*mean, *std).map_err(|e| {
                    InitializerError::DistributionParametersError {
                        reason: e.to_string(),
                    }
                })?;

                Ok(distribution.sample_iter(&mut rng()).take(size).collect())
            }
            Self::RandomSpherical => {
                let array_normal: Vec<f64> =
                    StandardNormal.sample_iter(&mut rng()).take(size).collect();

                let norm =
                    array_normal.iter().map(|x| x * x).sum::<f64>().sqrt();

                Ok(array_normal.iter().map(|x| x / norm).collect())
            }
            Self::RandomWeighted { values, weights } => {
                let distribution =
                    WeightedIndex::new(weights).map_err(|e| {
                        InitializerError::DistributionParametersError {
                            reason: e.to_string(),
                        }
                    })?;

                Ok(distribution
                    .sample_iter(&mut rng())
                    .take(size)
                    .map(|i| values[i])
                    .collect())
            }
            Self::Value { value } => Ok(vec![*value; size]),
            Self::Array { array } => {
                if size != array.len() {
                    Err(InitializerError::ArraySizeMismatch {
                        size_required: size,
                        size_actual: array.len(),
                    })
                } else {
                    Ok(array.clone())
                }
            }
            Self::FromConfig { path, name } => {
                let config = Config::builder()
                    .add_source(config::File::from(path.as_path()))
                    .build()
                    .map_err(|e| InitializerError::ConfigError {
                        reason: e.to_string(),
                    })?;

                let array = config.get::<Vec<f64>>(name).map_err(|e| {
                    InitializerError::ConfigError {
                        reason: e.to_string(),
                    }
                })?;

                if size != array.len() {
                    Err(InitializerError::ArraySizeMismatch {
                        size_required: size,
                        size_actual: array.len(),
                    })
                } else {
                    Ok(array)
                }
            }
        }
    }
}

/// Initialization options for integer arrays (u32)
///
/// ## Example
///
/// ```
/// use rusty_forge::initialize::IntegerArrayInitializer;
///
/// // Returns size 10 vector of floats uniformly sampled between 0 and 10
/// let array = IntegerArrayInitializer::default()
///   .initialize(10)
///   .unwrap();
///
/// array.iter()
///   .for_each(|a| {
///     assert!(*a >= 0);
///     assert!(*a <= 10);
///   });
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum IntegerArrayInitializer {
    /// Random array components from uniform distribution in range (min, max)
    RandomUniform { min: u32, max: u32 },
    /// Random array components from binomial distribution
    RandomBinomial { n: u64, p: f64 },
    /// Random array components sampled from a specified set of values with
    /// relative weights
    RandomWeighted { values: Vec<u32>, weights: Vec<f64> },
    /// Array of identical values
    Value { value: u32 },
    /// Array from input
    Array { array: Vec<u32> },
    /// Integer array from a configuration file
    FromConfig { path: PathBuf, name: String },
}

impl Default for IntegerArrayInitializer {
    fn default() -> Self {
        Self::RandomUniform { min: 0, max: 10 }
    }
}

impl IntegerArrayInitializer {
    /// Creates initializer from a uniform distribution with specified bounds.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::IntegerArrayInitializer;
    ///
    /// // Returns size 10 vector of ints uniformly sampled between 0 and 10
    /// let array = IntegerArrayInitializer::new_uniform(0, 10)
    ///   .initialize(10)
    ///   .unwrap();
    /// ```
    pub fn new_uniform(min: u32, max: u32) -> Self {
        Self::RandomUniform { min, max }
    }

    /// Creates initializer from a binomial distribution with specified number
    /// of trials and trial probability p.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::IntegerArrayInitializer;
    ///
    /// // Returns size 10 vector of ints sampled from binomial distribution
    /// let array = IntegerArrayInitializer::new_binomial(10, 0.5)
    ///   .initialize(10)
    ///   .unwrap();
    /// ```
    pub fn new_binomial(n: u64, p: f64) -> Self {
        Self::RandomBinomial { n, p }
    }

    /// Creates initializer from a weighted distribution with a finite number
    /// of specified allowed values and the corresponding weights.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::IntegerArrayInitializer;
    ///
    /// // Returns size 10 vector of ints values in [0, 1, 2], with
    /// // corresponding weights equal to [0.1, 0.5, 0.4]
    /// let array = IntegerArrayInitializer::new_weighted(
    ///   &[0, 1, 2],
    ///   &[0.1, 0.5, 0.4],
    /// )
    ///   .initialize(10)
    ///   .unwrap();
    /// ```
    pub fn new_weighted(values: &[u32], weights: &[f64]) -> Self {
        Self::RandomWeighted {
            values: values.into(),
            weights: weights.into(),
        }
    }

    /// Creates initializer to a single value.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::IntegerArrayInitializer;
    ///
    /// // Returns size 10 vector of ints equal to 1
    /// let array = IntegerArrayInitializer::new_value(1)
    ///   .initialize(10)
    ///   .unwrap();
    /// ```
    pub fn new_value(value: u32) -> Self {
        Self::Value { value }
    }

    /// Creates initializer to an input array.
    ///
    /// ## Example
    ///
    /// ```
    /// use rusty_forge::initialize::IntegerArrayInitializer;
    ///
    /// // Returns size 10 vector of floats equal to [1, ..., 10]
    /// let array = IntegerArrayInitializer::new_array(
    ///   &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    ///   )
    ///   .initialize(10)
    ///   .unwrap();
    /// ```
    pub fn new_array(array: &[u32]) -> Self {
        Self::Array {
            array: array.into(),
        }
    }

    /// Creates initializer to an input array in a config at a given path.
    pub fn new_config<P: AsRef<PathBuf>>(path: P, name: String) -> Self {
        Self::FromConfig {
            path: path.as_ref().to_path_buf(),
            name,
        }
    }

    /// Initialize an array of ints based on the initialization variant
    pub fn initialize(
        &self,
        size: usize,
    ) -> Result<Vec<u32>, InitializerError> {
        match self {
            Self::RandomUniform { min, max } => {
                let distribution = Uniform::new(min, max).map_err(|e| {
                    InitializerError::DistributionParametersError {
                        reason: e.to_string(),
                    }
                })?;

                Ok(distribution.sample_iter(&mut rng()).take(size).collect())
            }
            Self::RandomBinomial { n, p } => {
                let distribution = Binomial::new(*n, *p).map_err(|e| {
                    InitializerError::DistributionParametersError {
                        reason: e.to_string(),
                    }
                })?;

                Ok(distribution
                    .sample_iter(&mut rng())
                    .take(size)
                    .map(|x| x as u32)
                    .collect())
            }
            Self::RandomWeighted { values, weights } => {
                let distribution =
                    WeightedIndex::new(weights).map_err(|e| {
                        InitializerError::DistributionParametersError {
                            reason: e.to_string(),
                        }
                    })?;

                Ok(distribution
                    .sample_iter(&mut rng())
                    .take(size)
                    .map(|i| values[i])
                    .collect())
            }
            Self::Value { value } => Ok(vec![*value; size]),
            Self::Array { array } => {
                if size != array.len() {
                    Err(InitializerError::ArraySizeMismatch {
                        size_required: size,
                        size_actual: array.len(),
                    })
                } else {
                    Ok(array.clone())
                }
            }
            Self::FromConfig { path, name } => {
                let config = Config::builder()
                    .add_source(config::File::from(path.as_path()))
                    .build()
                    .map_err(|e| InitializerError::ConfigError {
                        reason: e.to_string(),
                    })?;

                let array = config.get::<Vec<u32>>(name).map_err(|e| {
                    InitializerError::ConfigError {
                        reason: e.to_string(),
                    }
                })?;

                if size != array.len() {
                    Err(InitializerError::ArraySizeMismatch {
                        size_required: size,
                        size_actual: array.len(),
                    })
                } else {
                    Ok(array)
                }
            }
        }
    }
}
