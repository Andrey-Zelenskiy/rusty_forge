// Copyright Andrey Zelenskiy, 2024-2026

use std::{collections::HashMap, vec};

use serde::{Deserialize, Serialize};

use crate::{
    run::parameters::{ParameterMap, ParameterValue},
    ManagerError, ManagerResult,
};

/// Options for defining the sweep range
// TODO: Generic types don't work for multiple ranges - need correct types right away
#[derive(Serialize, Deserialize, Clone)]
pub enum SweepRange
{
    /// Exhaustive bool range: true and false
    Bool,
    /// Linear int range
    LinearInt { min: i64, max: i64},
    /// Explicit int values
    ExplicitInt { values: Vec<i64> },
    /// Linear unsigned int range
    LinearUint { min: u64, max: u64},
    /// Explicit unsigned int values
    ExplicitUint { values: Vec<u64> },
    /// Linear float range
    LinearFloat { min: f64, max: f64, n_values: u32 },
    /// Explicit float values
    ExplicitFloat { values: Vec<f64> },
    /// Logarithmically scalled float range
    Logarithmic { min: f64, max: f64, n_values: u32 },
    /// Logarithmically (base 10) scalled float range
    Logarithmic10 { min: f64, max: f64, n_values: u32 },
}

impl SweepRange {
    /// Returns number of values in the range
    pub fn n_values(&self) -> u32 {
        match self {
            Self::Bool => 2,
            Self::LinearInt { min, max } => (max - min) as u32,
            Self::LinearUint { min, max} => (max - min) as u32,
            Self::LinearFloat { n_values, .. }
            | Self::Logarithmic { n_values, .. }
            | Self::Logarithmic10 { n_values, .. } => *n_values,
            Self::ExplicitInt { values } => values.len() as u32,
            Self::ExplicitUint { values } => values.len() as u32,
            Self::ExplicitFloat { values } => values.len() as u32,
        }
    }

    /// Returns the type of the sweep as a string
    fn has_type(&self) -> String {
        match self {
            Self::Bool => String::from("Bool"),
            Self::LinearInt { .. } => String::from("LinearInt"),
            Self::ExplicitInt { .. } => String::from("ExplicitInt"),
            Self::LinearUint { .. } => String::from("LinearUint"),
            Self::ExplicitUint { .. } => String::from("ExplicitUint"),
            Self::LinearFloat { .. } => String::from("LinearFloat"),
            Self::ExplicitFloat { .. } => String::from("ExplicitFloat"),
            Self::Logarithmic { .. } => String::from("Logarithmic"),
            Self::Logarithmic10 { .. } => String::from("Logarithmic10"),
        }
    }

    /// Verify that the sweep is defined correctly
    fn validate_range(&self) -> ManagerResult<()> {
        if self.n_values() == 0 {
            return Err(ManagerError::SweepErrorNoValues);
        }

        if let Self::LinearInt { min, max } = &self && min >= max {
            return Err(ManagerError::SweepErrorInvalidLimits {
                min: min.to_string(),
                max: max.to_string(),
            });
        }
        
        if let Self::LinearUint { min, max } = &self && min >= max {
            return Err(ManagerError::SweepErrorInvalidLimits {
                min: min.to_string(),
                max: max.to_string(),
            });
        }
        
        if let Self::LinearFloat { min, max, .. } = &self && min >= max {
            return Err(ManagerError::SweepErrorInvalidLimits {
                min: min.to_string(),
                max: max.to_string(),
            });
        }
        
        if let Self::Logarithmic { min, max, .. } = &self && min >= max {
            return Err(ManagerError::SweepErrorInvalidLimits {
                min: min.to_string(),
                max: max.to_string(),
            });
        }
        
        if let Self::Logarithmic10 { min, max, .. } = &self && min >= max {
            return Err(ManagerError::SweepErrorInvalidLimits {
                min: min.to_string(),
                max: max.to_string(),
            });
        }

        Ok(())
    }
    
    /// Verify that the sweep type matches value type
    fn validate_type(&self, value: &ParameterValue) -> ManagerResult<()> {
        if !matches!(
            (self, value),
            (Self::Bool, ParameterValue::Bool(_))
            | (Self::LinearInt{..}, ParameterValue::Int(_))
            | (Self::ExplicitInt{..}, ParameterValue::Int(_))
            | (Self::LinearUint{..}, ParameterValue::Uint(_))
            | (Self::ExplicitUint{..}, ParameterValue::Uint(_))
            | (Self::LinearFloat{..}, ParameterValue::Float(_))
            | (Self::ExplicitFloat{..}, ParameterValue::Float(_))
            | (Self::Logarithmic{..}, ParameterValue::Float(_))
            | (Self::Logarithmic10{..}, ParameterValue::Float(_))
        ) {
            return Err(ManagerError::SweepErrorInvalidType {
                sweep: self.has_type(),
                value: value.has_type()
            });
        }
        Ok(())
    }

    /// Returns the value corresponding to the sweep index
    pub fn value(&self, index:u32) -> ManagerResult<ParameterValue> {
        #[cfg(debug_assertions)]
        assert!(index < self.n_values());
        
        match self {
            Self::Bool => {
                if index == 0 {
                    Ok(false.into())
                } else {
                    Ok(true.into())
                }
            }
            Self::ExplicitInt { values } => Ok(values[index as usize].into()),
            Self::ExplicitUint { values } => Ok(values[index as usize].into()),
            Self::ExplicitFloat { values } => Ok(values[index as usize].into()),
            Self::LinearInt { min, .. } => Ok((min + index as i64).into()),
            Self::LinearUint { min, .. } => Ok((min + index as u64).into()),
            Self::LinearFloat { min, max, n_values } => {
                let dv = (max - min) / *n_values as f64;
                let value = min + (index as f64) * dv;

                Ok(value.into())
            }
            Self::Logarithmic { min, max, n_values } => {
                let dv = (max.ln() - min.ln()) / *n_values as f64;
                let value = (min.ln() + (index as f64) * dv).exp();
                
                Ok(value.into())
            }
            Self::Logarithmic10 { min, max, n_values } => {
                let dv = (max.log10() - min.log10()) / *n_values as f64;
                let value = 10.0_f64.powf(min.log10() + (index as f64) * dv);
                
                Ok(value.into())
            }
            
        }
    }
}

impl Default for SweepRange {
    fn default() -> Self {
        Self::LinearFloat {
            min: 0.0,
            max: 1.0,
            n_values: 10,
        }
    }
}

/// Configuration for sweep runs
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct SweepConfig {
    sweep_parameters: HashMap<String, SweepRange>,
}

impl SweepConfig {
    /// Number of required simulations to cover the defined sweep range(s)
    pub fn n_runs(&self) -> u32 {
        self.n_values_vec().iter().product()
    }

    /// Verifies that the defined sweep ranges are valid for the parameter map
    pub fn validate_parameters(&self, parameters: &ParameterMap) -> ManagerResult<()>{
        for (key, range) in &self.sweep_parameters {
            match parameters.get(key) {
                Some(value) => range.validate_type(value)?,
                None => return Err(ManagerError::SweepErrorMissingKey {
                    key: key.to_string()
                }),
            }
            range.validate_range()?;
        }

        Ok(())
    }

    /// Returns parameter map correponding to the current set of sweep indices
    pub fn sweep_parameters(&self, index:u32, reference_parameters: &ParameterMap) -> ManagerResult<ParameterMap> {       
        let mut parameters = reference_parameters.clone();

        for ((key, range),index) in self.sweep_parameters.iter().zip(self.indices(index).iter()) {
            let value = range.value(*index)?;
            
            parameters.insert(key.to_string(), value).ok_or(ManagerError::SweepErrorMissingKey { key: key.to_string() })?;
        }

        Ok(parameters)
    }
    
    /// De-hash the index
    /// If the range indices are given by index[n], and the number of values
    /// is len[n], then the hash is computed as
    /// index_hash = index[0] + index[1] * len[0] + index[2] * len[0] * len[1]
    /// and so on
    pub fn indices(&self, index:u32) -> Vec<u32> {
        if self.size() == 1 {
            return vec![index];
        }

        // De-hash the index
        let lens = self.n_values_vec();
        let mut indices = vec![0; self.size()];
        let mut residual = 0;

        for i in 0..self.size() - 1 {
            let divisor = lens[..self.size() - 1 - i].iter().product::<u32>();
            indices[i] = (index - residual) / divisor;
            residual += indices[i] * divisor
        }

        indices.reverse();
        indices[0] = index - residual;

        indices
        
    }

    /// Returns number of values for each sweep range defined
    fn n_values_vec(&self) -> Vec<u32> {
        self.sweep_parameters
            .values()
            .map(|range| range.n_values())
            .collect()
    }

    /// Returns the number of defined ranges
    fn size(&self) -> usize {
        self.sweep_parameters.len()
    }
}
