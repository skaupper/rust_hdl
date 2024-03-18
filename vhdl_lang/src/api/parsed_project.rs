use fnv::{FnvHashMap, FnvHashSet};

use crate::data::{FilePath, Symbol};
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
        let mut analyzed_project = AnalyzedProject::empty(
            self.parser,
            self.config,
            self.empty_libraries,
            analyze_config.unused_declarations_linter,
        );
        analyzed_project.add_source_files(self.parsed_files.into_values());
        analyzed_project
    }

    pub fn analyze(self) -> AnalyzedProject {
        self.analyze_with_config(AnalyzeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use itertools::Itertools;
    use tempfile::TempDir;

    use crate::{syntax::test::check_no_diagnostics, SourceProject};

    use super::*;

    fn get_symbol(project: &ParsedProject, name: &str) -> Symbol {
        project.parser.symbols.symtab().insert_utf8(name)
    }

    fn create_file(root: &TempDir, file_name: &str, contents: &str) -> FilePath {
        let path = root.path().join(file_name);
        std::fs::write(&path, contents).unwrap();
        FilePath::new(&path)
    }

    fn get_source(file_path: &FilePath) -> Source {
        Source::from_latin1_file(file_path).unwrap()
    }

    fn get_std_config_path() -> PathBuf {
        std::env::current_dir()
            .unwrap()
            .join("../vhdl_libraries/vhdl_ls.toml")
    }

    fn get_non_std_files<'a>(
        files: impl Iterator<Item = &'a SourceFile>,
    ) -> impl Iterator<Item = &'a SourceFile> {
        files.filter(|f| {
            f.library_names
                .iter()
                .filter(|n| {
                    let name = n.name_utf8();
                    name == "std" || name == "ieee"
                })
                .count()
                == 0
        })
    }

    #[test]
    fn test_empty_project() {
        let mut messages = Vec::new();
        let project = SourceProject::default().parse(&mut messages);

        let analyzed_project = project.analyze_with_config(AnalyzeConfig::default());

        assert_eq!(analyzed_project.diagnostics().len(), 0);
        assert_eq!(analyzed_project.files().try_len().unwrap(), 0);
    }

    #[test]
    fn test_add_source() {
        let temp_root = tempfile::tempdir().unwrap();
        let file_path = create_file(&temp_root, "file.vhd", "entity ent is end entity;");

        let mut messages = Vec::new();
        let mut project = SourceProject::default().parse(&mut messages);

        // Add a source manually and analyze the project
        project.add_source(
            FnvHashSet::from_iter([get_symbol(&project, "lib")]),
            get_source(&file_path),
        );
        let analyzed_project = project.analyze();

        let files = analyzed_project.files().collect::<Vec<_>>();
        assert_eq!(files.len(), 1);
        check_no_diagnostics(analyzed_project.diagnostics());
    }

    #[test]
    fn test_add_source_file() {
        let temp_root = tempfile::tempdir().unwrap();
        let file_path = create_file(&temp_root, "file.vhd", "entity ent is end entity;");
        let config_path = create_file(
            &temp_root,
            "config.toml",
            "[libraries]\nlib.files = ['file.vhd']",
        );

        // Prepare a project to extract a `SourceFile` from
        let mut messages = Vec::new();
        let mut project = SourceProject::from_config_file(config_path.to_path_buf())
            .unwrap()
            .parse(&mut messages);
        let source_file = project.parsed_files.remove(&file_path).unwrap();

        // Create a new project with the std and ieee libraries in it
        let mut project = SourceProject::from_config_file(get_std_config_path())
            .unwrap()
            .parse(&mut messages);
        // Add the extracted `SourceFile` from above manually
        project.add_source_file(source_file);
        // Analyze the project
        let analyzed_project = project.analyze();

        // After removing all the std and ieee sources, only the manually added source file should remain
        let files = get_non_std_files(analyzed_project.files()).collect::<Vec<_>>();
        assert_eq!(files.len(), 1);
        check_no_diagnostics(analyzed_project.diagnostics());
    }
}
