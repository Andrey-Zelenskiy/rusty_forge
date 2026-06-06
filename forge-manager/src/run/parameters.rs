// Copyright Andrey Zelenskiy, 2024-2026

use std::{
    collections::HashMap,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use crate::ParameterResult;

/// Model parameters with stringified keys
#[derive(Serialize, Deserialize, Debug)]
pub struct ParameterMap(HashMap<String, ParameterValue>);

impl ParameterMap {
    /// Initialize a new set of parameters
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    // Methods that interface with the HashMap

    /// Insert a new parameter
    pub fn insert(
        &mut self,
        key: String,
        value: ParameterValue,
    ) -> Option<ParameterValue> {
        self.0.insert(key, value)
    }

    /// Get a parameter from key value
    pub fn get(&self, key: &str) -> Option<&ParameterValue> {
        self.0.get(key)
    }

    /// Return iterator with all keys of currently contained parameters
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    /// Return HashMap as an iterator
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ParameterValue)> {
        self.0.iter()
    }

    /// Check whether the Hashmap is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return the number of parameters
    pub fn len(&self) -> usize {
        self.0.len()
    }

    // Hashing methods

    /// Create a hash of the parameter set
    pub fn hash(&self) -> u64 {
        let pairs: Vec<_> = self.iter().collect();

        let mut hasher = DefaultHasher::new();
        for (key, value) in pairs {
            key.hash(&mut hasher);
            value.to_string().hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Compare two sets of parameters
    pub fn diff(&self, other: &Self) -> ParameterDiff {
        let mut only_in_left = HashMap::new();
        let mut only_in_right = HashMap::new();
        let mut changed = HashMap::new();

        // Check if keys are in self
        for (key, value) in &self.0 {
            match other.get(key) {
                Some(other_value) => {
                    if other_value != value {
                        changed.insert(key.clone(), (*value, *other_value));
                    }
                }
                None => {
                    only_in_left.insert(key.clone(), *value);
                }
            }
        }

        // Check if keys are in other
        for (key, value) in &other.0 {
            if !self.0.contains_key(key) {
                only_in_right.insert(key.clone(), *value);
            }
        }

        ParameterDiff {
            only_in_left,
            only_in_right,
            changed,
        }
    }
}

/// Types of model parameters accessed by ProjectManager
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ParameterValue {
    Bool(bool),
    Int(i64),
    Uint(u64),
    Float(f64),
}

impl ParameterValue {
    /// Returns the type of the parameter value as a string
    pub fn has_type(&self) -> String {
        match self {
            Self::Float(_) => String::from("f64"),
            Self::Int(_) => String::from("i64"),
            Self::Uint(_) => String::from("u64"),
            Self::Bool(_) => String::from("bool"),
        }
    }
}

impl fmt::Display for ParameterValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => write!(f, "{value}"),
            Self::Int(value) => write!(f, "{value}"),
            Self::Uint(value) => write!(f, "{value}"),
            Self::Float(value) => write!(f, "{value:17}"),
        }
    }
}

// Blanket From<T> implementation for simple types
macro_rules! impl_from_for_simple_types {
    ($variant: expr, $($t:ty),*) => {
        $(
            impl From<$t> for ParameterValue {
                fn from(value: $t) -> Self {
                    $variant(value.into())
                }
            }
        )*
    };
}

impl_from_for_simple_types!(ParameterValue::Bool, bool);
impl_from_for_simple_types!(ParameterValue::Int, i8, i16, i32, i64);
impl_from_for_simple_types!(ParameterValue::Uint, u8, u16, u32, u64);
impl_from_for_simple_types!(ParameterValue::Float, f32, f64);

/// Report structure to asses differences between two parameter sets
#[derive(Debug, Serialize, Deserialize)]
pub struct ParameterDiff {
    /// Parameters only included in the first set
    pub only_in_left: HashMap<String, ParameterValue>,
    /// Parameters only included in the second set
    pub only_in_right: HashMap<String, ParameterValue>,
    /// Parameter values that are change from one set to the second
    pub changed: HashMap<String, (ParameterValue, ParameterValue)>,
}

impl ParameterDiff {
    /// Checks if any differences between parameters have been logged
    pub fn is_empty(&self) -> bool {
        self.only_in_left.is_empty()
            && self.only_in_right.is_empty()
            && self.changed.is_empty()
    }
}

/// Trait for manipulating model parameters
pub trait ModelParameters: Sized {
    /// Generates ParameterMap with all necessary parameters
    fn to_map(&self) -> ParameterMap;

    /// Initializes model parameters from ParameterMap
    fn from_map(map: &ParameterMap) -> ParameterResult<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test to make sure that the order of parameters does not change the hash
    #[test]
    fn test_parameter_map_hash_order_independent() {
        let mut map1 = ParameterMap::new();

        map1.insert("a".to_string(), 1.0.into());
        map1.insert("b".to_string(), 2.0.into());

        let mut map2 = ParameterMap::new();

        map2.insert("b".to_string(), 2.0.into());
        map2.insert("a".to_string(), 1.0.into());

        assert_eq!(map1.hash(), map2.hash());
    }

    // Test difference detection for two sets of parameters
    #[test]
    fn test_parameter_diff() {
        let mut map1 = ParameterMap::new();

        map1.insert("a".to_string(), 1.0.into());
        map1.insert("b".to_string(), 2.0.into());

        let mut map2 = ParameterMap::new();

        map2.insert("c".to_string(), 3.0.into());
        map2.insert("a".to_string(), 1.5.into());

        let diff = map1.diff(&map2);
        assert!(diff.changed.contains_key("a"));
        assert!(diff.only_in_left.contains_key("b"));
        assert!(diff.only_in_right.contains_key("c"));
    }
}
