// Copyright Andrey Zelenskiy, 2024-2026
mod builder;

pub mod prelude {
    pub use super::builder::{BuildError, BuilderMethods, TargetFromBuilder};
    pub use forge_derive::{BuilderFromTargets, BuilderSetters};
}

pub mod value;
