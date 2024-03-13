// Define submodules
mod analyzed_api;
mod analyzed_project;
mod ast_helpers;
mod parsed_api;
mod parsed_project;
mod source_project;

// Export project types
pub use analyzed_project::AnalyzedProject;
pub use parsed_project::ParsedProject;
pub use source_project::SourceProject;

// For each stage create an inline module with all relevant types and functionality
pub mod ast {
    pub use super::ast_helpers::*;
}
pub mod source {
    pub use super::source_project::SourceProject as Project;
}

pub mod parsed {
    pub use super::parsed_api::*;
    pub use super::parsed_project::ParsedProject as Project;
}

pub mod analyzed {
    pub use super::analyzed_api::*;
    pub use super::analyzed_project::AnalyzeConfig as Config;
    pub use super::analyzed_project::AnalyzedProject as Project;
}
