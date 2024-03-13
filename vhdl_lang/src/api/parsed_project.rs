use fnv::{FnvHashMap, FnvHashSet};

use crate::data::{FilePath, Symbol};
use crate::lint::dead_code::UnusedDeclarationsLinter;
use crate::{Config, Source, SourceFile, VHDLParser};

use super::analyzed_project::AnalyzeConfig;
use super::parsed_api::ParsedApiRoot;
use super::AnalyzedProject;

#[derive(Default)]
pub struct ParsedProject {
    pub(crate) config: Config,
    pub(crate) parser: VHDLParser,
    pub(crate) parsed_files: FnvHashMap<FilePath, SourceFile>,
    pub(crate) empty_libraries: FnvHashSet<Symbol>,
}

impl ParsedProject {
    //
    // Constructors
    //
    pub(super) fn new(
        config: Config,
        parser: VHDLParser,
        parsed_files: FnvHashMap<FilePath, SourceFile>,
        empty_libraries: FnvHashSet<Symbol>,
    ) -> Self {
        ParsedProject {
            config,
            parser,
            parsed_files,
            empty_libraries,
        }
    }

    //
    // Source file management
    //
    pub fn add_source(&mut self, library_symbols: FnvHashSet<Symbol>, source: Source) {
        let mut diagnostics = Vec::new();
        let design_file = self.parser.parse_design_source(&source, &mut diagnostics);
        self.add_source_file(SourceFile {
            library_names: library_symbols,
            source,
            design_file,
            parser_diagnostics: diagnostics,
        });
    }

    pub fn add_source_file(&mut self, source_file: SourceFile) {
        self.parsed_files
            .insert(source_file.source.file_path().to_owned(), source_file);
    }

    pub fn api(&self) -> ParsedApiRoot {
        ParsedApiRoot {
            parsed_files: &self.parsed_files,
        }
    }

    /// Process parsed source files and produce a `AnalyzedProject`
    pub fn analyze_with_config(self, analyze_config: AnalyzeConfig) -> AnalyzedProject {
        self.analyze_internal(analyze_config.unused_declarations_linter)
    }

    pub fn analyze(self, enable_unused_declarations_linter: bool) -> AnalyzedProject {
        let unused_declarations_linter = if enable_unused_declarations_linter {
            Some(UnusedDeclarationsLinter::default())
        } else {
            None
        };

        self.analyze_internal(unused_declarations_linter)
    }

    fn analyze_internal(
        self,
        unused_declarations_linter: Option<UnusedDeclarationsLinter>,
    ) -> AnalyzedProject {
        let mut analyzed_project = AnalyzedProject::empty(
            self.parser,
            self.config,
            self.empty_libraries,
            unused_declarations_linter,
        );
        analyzed_project.add_source_files(self.parsed_files.into_values());
        analyzed_project
    }
}
