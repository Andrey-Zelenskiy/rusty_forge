// Copyright Andrey Zelenskiy, 2024-2026
pub mod files;
pub use files::FileManager;

pub mod builder;
pub use crate::builder::{BuildError, BuilderMethods, TargetFromBuilder};

//pub mod output;
pub mod project_setup;
pub use crate::project_setup::ProjectManager;

pub mod initialize;
