// Copyright Andrey Zelenskiy, 2024-2026

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ParameterResult;

/// Model parameters with stringified keys
#[derive(Serialize, Deserialize, Debug)]
pub struct ParamterMap {
    inner: HashMap<String, ParameterValue>,
}

/// Types of model parameters accessed by ProjectManager
pub enum ParameterValue {
    Float(f64),
    Int(i64),
    Bool(bool),
}

impl ParameterValue {
    /// Returns the type of the parameter value as a string
    pub fn has_type(&self) -> String {
        match self {
            Self::Float(_) => String::from("Float"),
            Self::Int(_) => String::from("Int"),
            Self::Bool(_) => String::from("Bool"),
        }
    }
}

/// Trait for manipulating model parameters
pub trait ModelParameters {
    /// Generates ParameterMap with all necessary parameters
    pub fn to_map(&self) -> ParamterMap;

    /// Initializes model parameters from ParameterMap
    pub fn from_map(map: &ParamterMap) -> ParameterResult<Self>;
}
