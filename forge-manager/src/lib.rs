// Copyright Andrey Zelenskiy, 2024-2026

mod errors;

pub type ManagerResult<T> = Result<T, errors::SimulationError>;

pub mod project;
