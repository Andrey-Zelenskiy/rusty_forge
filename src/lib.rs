// Copyright Andrey Zelenskiy, 2024-2026
pub mod files;
pub use files::FileManager;

pub mod builder;
pub use builder::{BuildError, BuilderMethods, TargetFromBuilder};

pub use builder_derive::{BuilderFromTargets, BuilderSetters};

//pub mod output
pub mod project_setup;
pub use project_setup::ProjectManager;

pub mod initialize;
