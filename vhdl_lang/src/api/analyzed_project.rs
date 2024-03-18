use fnv::{FnvHashMap, FnvHashSet};

use crate::analysis::DesignRoot;
use crate::ast::search::Searcher;
use crate::ast::DesignFile;
use crate::config::Config;
use crate::lint::dead_code::UnusedDeclarationsLinter;
use crate::{
    data::*, list_completion_options, AnyEnt, CompletionItem, EntHierarchy, EntRef, EntityId, Token,
};
use crate::{SourceFile, VHDLParser};

use super::analyzed_api::AnalyzedApiRoot;
use super::SourceProject;

#[derive(Default)]
pub struct AnalyzeConfig {
    pub(super) unused_declarations_linter: Option<UnusedDeclarationsLinter>,
}

impl AnalyzeConfig {
    pub fn enable_unused_declaration_detection(&mut self) {
        self.unused_declarations_linter = Some(UnusedDeclarationsLinter::default());
    }
}

pub struct AnalyzedProject {
    parser: VHDLParser,
    config: Config,
    root: DesignRoot,
    files: FnvHashMap<FilePath, SourceFile>,
    empty_libraries: FnvHashSet<Symbol>,
    lint: Option<UnusedDeclarationsLinter>,
    diagnostics: Vec<Diagnostic>,
}

impl Default for AnalyzedProject {
    fn default() -> Self {
        AnalyzedProject::empty(
            VHDLParser::default(),
            Config::default(),
            FnvHashSet::default(),
            None,
        )
    }
}

impl AnalyzedProject {
    pub(super) fn empty(
        parser: VHDLParser,
        config: Config,
        empty_libraries: FnvHashSet<Symbol>,
        lint: Option<UnusedDeclarationsLinter>,
    ) -> Self {
        let symbols = parser.symbols.clone();
        AnalyzedProject {
            parser,
            config,
            root: DesignRoot::new(symbols),
            files: FnvHashMap::default(),
            empty_libraries,
            lint,
            diagnostics: Vec::new(),
        }
    }

    pub fn api(&self) -> AnalyzedApiRoot {
        AnalyzedApiRoot { root: &self.root }
    }

    //
    // Accessor functions
    //
    pub fn diagnostics(&self) -> &Vec<Diagnostic> {
        &self.diagnostics
    }

    pub fn files(&self) -> impl Iterator<Item = &SourceFile> {
        self.files.values()
    }

    //
    // Source file management
    //

    /// Add the source file and re-analyze the project
    pub fn add_source_file(&mut self, source_file: SourceFile) {
        self.add_source_files(std::iter::once(source_file));
    }

    /// Add all source files from the given iterators and re-analyze the project
    pub fn add_source_files(&mut self, source_files: impl Iterator<Item = SourceFile>) {
        for mut source_file in source_files {
            self.root.add_source_file(&mut source_file);
            self.files
                .insert(source_file.source.file_path().to_owned(), source_file);
        }
        self.analyze();
    }

    /// Parse the given source, add it and re-analyze the project.
    ///
    /// If the file the source is referring to already exists, it gets replaced.
    pub fn add_or_update_source(&mut self, source: &Source) {
        let mut source_file = {
            if let Some(mut source_file) = self.files.remove(source.file_path()) {
                // File is already part of the project
                for library_name in source_file.library_names.iter() {
                    self.root.remove_source(library_name.clone(), source);
                }
                source_file.source = source.clone();
                source_file.parser_diagnostics.clear();
                source_file
            } else {
                // File is not part of the project
                // @TODO use config wildcards to map to library

                // Add unmapped files to an anonymous library work
                // To still get some semantic analysis for unmapped files
                let mut library_names = FnvHashSet::default();
                library_names.insert(self.root.symbol_utf8("work"));

                SourceFile {
                    source: source.clone(),
                    library_names,
                    parser_diagnostics: vec![],
                    design_file: DesignFile::default(),
                }
            }
        };

        // Parse the source file
        source_file.design_file = self
            .parser
            .parse_design_source(source, &mut source_file.parser_diagnostics);

        // Add the parsed source file and re-analyze the project
        self.add_source_file(source_file);
    }

    /// Updates the project's configuration
    ///
    /// This step basically creates a new project instance but reuses already loaded file contents if possible.
    pub fn update_config(&mut self, config: Config, messages: &mut dyn MessageHandler) {
        let mut source_project = SourceProject::from_config(config);
        source_project.add_preloaded_source_files(self.files.drain().map(|kv| kv.1));

        let parsed_project = source_project.parse(messages);
        let analyzed_project = parsed_project.analyze_with_config(AnalyzeConfig {
            unused_declarations_linter: self.lint.take(),
        });

        *self = AnalyzedProject { ..analyzed_project };
    }

    //
    // Project functions
    //
    pub fn analyze(&mut self) {
        let mut diagnostics = Vec::new();

        // Gather diagnostics from source files
        for source_file in self.files.values() {
            for diagnostic in source_file.parser_diagnostics.iter() {
                diagnostics.push(diagnostic.clone());
            }
        }

        // Create empty libraries
        for library_name in self.empty_libraries.iter() {
            self.root.ensure_library(library_name.clone());
        }

        // Analyze the `DesignRoot` instance
        let analyzed_units = self.root.analyze(&mut diagnostics);

        // Linting
        if let Some(unused_decl_linter) = self.lint.as_mut() {
            unused_decl_linter.lint(&self.root, &self.config, &analyzed_units, &mut diagnostics);
        }

        self.diagnostics = diagnostics;
    }

    pub fn get_source(&self, file_name: &Path) -> Option<Source> {
        self.files
            .get(&FilePath::new(file_name))
            .map(|file| file.source.clone())
    }

    //
    // Query functions (TODO: refactor into API!)
    //
    pub fn find_declaration(&self, source: &Source, cursor: Position) -> Option<EntRef> {
        let ent = self.root.search_reference(source, cursor)?;
        Some(ent.declaration())
    }

    pub fn find_definition(&self, source: &Source, cursor: Position) -> Option<EntRef> {
        let ent = self.root.search_reference(source, cursor)?;
        self.root.find_definition_of(ent)
    }

    pub fn find_implementation(&self, source: &Source, cursor: Position) -> Vec<EntRef> {
        if let Some(ent) = self.find_declaration(source, cursor) {
            self.root.find_implementation(ent)
        } else {
            Vec::default()
        }
    }

    pub fn find_all_references(&self, ent: &AnyEnt) -> Vec<SrcPos> {
        self.root.find_all_references(ent)
    }

    pub fn find_all_unresolved(&self) -> (usize, Vec<SrcPos>) {
        self.root.find_all_unresolved()
    }

    pub fn item_at_cursor(&self, source: &Source, cursor: Position) -> Option<(SrcPos, EntRef)> {
        self.root.item_at_cursor(source, cursor)
    }

    pub fn search(&self, searcher: &mut impl Searcher) {
        let _ = self.root.search(searcher);
    }

    pub fn library_mapping_of(&self, source: &Source) -> Vec<Symbol> {
        let file = if let Some(file) = self.files.get(source.file_path()) {
            file
        } else {
            return Vec::new();
        };
        let mut libs: Vec<_> = file.library_names.iter().cloned().collect();
        libs.sort_by_key(|lib| lib.name_utf8());
        libs
    }

    pub fn public_symbols<'a>(&'a self) -> Box<dyn Iterator<Item = EntRef<'a>> + 'a> {
        self.root.public_symbols()
    }

    pub fn document_symbols<'a>(
        &'a self,
        library_name: &Symbol,
        source: &Source,
    ) -> Vec<(EntHierarchy<'a>, &Vec<Token>)> {
        self.root.document_symbols(library_name, source)
    }

    pub fn list_completion_options(
        &self,
        source: &Source,
        cursor: Position,
    ) -> Vec<CompletionItem> {
        list_completion_options(&self.root, source, cursor)
    }

    //
    // Format functions (TODO: refactor into API!)
    //
    pub fn format_declaration(&self, ent: &AnyEnt) -> Option<String> {
        self.root.format_declaration(ent)
    }

    pub fn format_entity(&self, id: EntityId) -> Option<String> {
        let ent = self.root.get_ent(id);
        self.format_declaration(ent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::test::check_no_diagnostics;

    // TODO: Check which files are in the project and which library they are associated with!

    /// Test that an empty library is created
    /// Thus test case was added when fixing a bug
    /// Where a library with no files was never added
    #[test]
    fn test_empty_library_is_defined() {
        let root = tempfile::tempdir().unwrap();
        let vhdl_file_path = root.path().join("file.vhd");
        std::fs::write(
            vhdl_file_path,
            "
library missing;

entity ent is
end entity;
        ",
        )
        .unwrap();

        let config_str = "
[libraries]
missing.files = []
lib.files = ['file.vhd']
        ";

        let config = Config::from_str(config_str, root.path()).unwrap();
        let mut messages = Vec::new();
        let project = SourceProject::from_config(config)
            .parse(&mut messages)
            .analyze();
        assert_eq!(messages, vec![]);
        check_no_diagnostics(project.diagnostics());
    }

    #[test]
    fn unmapped_libraries_are_analyzed() {
        let mut messages = Vec::new();
        let mut project = SourceProject::from_config(Config::default())
            .parse(&mut messages)
            .analyze();
        assert_eq!(messages, vec![]);
        check_no_diagnostics(project.diagnostics());

        let root = tempfile::tempdir().unwrap();
        let vhdl_file_path = root.path().join("file.vhd");
        std::fs::write(
            &vhdl_file_path,
            "
entity ent is
end ent;

architecture rtl of ent is
begin
end architecture;

architecture rtl of ent is
begin
end architecture;
",
        )
        .unwrap();
        let source = Source::from_latin1_file(&vhdl_file_path).unwrap();

        project.add_or_update_source(&source);
        let diagnostics = project.diagnostics();
        assert_eq!(diagnostics.len(), 1);
        let diag = diagnostics.first().unwrap();
        assert_eq!(diag.message, "Duplicate architecture 'rtl' of entity 'ent'")
    }

    /// Test that the same file can be added to several libraries
    #[test]
    fn test_same_file_in_multiple_libraries() {
        let root = tempfile::tempdir().unwrap();
        let vhdl_file_path1 = root.path().join("file.vhd");
        std::fs::write(
            vhdl_file_path1,
            "
package pkg is
end package;
        ",
        )
        .unwrap();

        let vhdl_file_path2 = root.path().join("use_file.vhd");
        std::fs::write(
            vhdl_file_path2,
            "
library lib1;
use lib1.pkg.all;

package use_pkg1 is
end package;

library lib2;
use lib2.pkg.all;

package use_pkg2 is
end package;
        ",
        )
        .unwrap();

        let config_str = "
[libraries]
lib1.files = ['file.vhd']
lib2.files = ['file.vhd']
use_lib.files = ['use_file.vhd']
        ";

        let config = Config::from_str(config_str, root.path()).unwrap();
        let mut messages = Vec::new();
        let project = SourceProject::from_config(config)
            .parse(&mut messages)
            .analyze();
        assert_eq!(messages, vec![]);
        check_no_diagnostics(project.diagnostics());
    }

    fn update(project: &mut AnalyzedProject, source: &mut Source, contents: &str) {
        std::fs::write(Path::new(source.file_name()), contents).unwrap();
        *source = Source::from_latin1_file(source.file_name()).unwrap();
        project.add_or_update_source(source);
    }

    /// Test that the same file can be added to several libraries
    #[test]
    fn test_re_analyze_after_update() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(tempdir.path()).unwrap();

        let path1 = root.join("file1.vhd");
        let path2 = root.join("file2.vhd");
        std::fs::write(
            &path1,
            "
package pkg is
end package;
        ",
        )
        .unwrap();
        let mut source1 = Source::from_latin1_file(&path1).unwrap();

        std::fs::write(
            &path2,
            "
library lib1;
use lib1.pkg.all;

package pkg is
end package;
        ",
        )
        .unwrap();
        let mut source2 = Source::from_latin1_file(&path2).unwrap();

        let config_str = "
[libraries]
lib1.files = ['file1.vhd']
lib2.files = ['file2.vhd']
        ";

        let config = Config::from_str(config_str, &root).unwrap();
        let mut messages = Vec::new();
        let mut project = SourceProject::from_config(config)
            .parse(&mut messages)
            .analyze();
        assert_eq!(messages, vec![]);
        check_no_diagnostics(project.diagnostics());

        // Add syntax error
        update(
            &mut project,
            &mut source1,
            "
package is
        ",
        );
        let diagnostics = project.diagnostics();
        assert_eq!(diagnostics.len(), 3);
        // Syntax error comes first
        assert_eq!(diagnostics[0].pos.source, source1);
        assert_eq!(diagnostics[1].pos.source, source1);
        assert_eq!(diagnostics[2].pos.source, source2);

        // Make it good again
        update(
            &mut project,
            &mut source1,
            "
package pkg is
end package;
        ",
        );
        check_no_diagnostics(project.diagnostics());

        // Add analysis error
        update(
            &mut project,
            &mut source2,
            "
package pkg is
end package;

package pkg is
end package;
        ",
        );
        let diagnostics = project.diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].pos.source, source2);

        // Make it good again
        update(
            &mut project,
            &mut source2,
            "
package pkg is
end package;
        ",
        );
        check_no_diagnostics(project.diagnostics());
    }

    /// Test that the configuration can be updated
    #[test]
    fn test_config_update() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(tempdir.path()).unwrap();

        let path1 = root.join("file1.vhd");
        let path2 = root.join("file2.vhd");
        std::fs::write(
            &path1,
            "
library unkown;
use unkown.pkg.all;

package pkg is
end package;
        ",
        )
        .unwrap();
        let source1 = Source::from_latin1_file(&path1).unwrap();

        std::fs::write(
            &path2,
            "
library unkown;
use unkown.pkg.all;

package pkg is
end package;
        ",
        )
        .unwrap();
        let source2 = Source::from_latin1_file(&path2).unwrap();

        let config_str1 = "
[libraries]
lib.files = ['file1.vhd']
        ";
        let config1 = Config::from_str(config_str1, &root).unwrap();

        let config_str2 = "
[libraries]
lib.files = ['file2.vhd']
        ";
        let config2 = Config::from_str(config_str2, &root).unwrap();

        let mut messages = Vec::new();
        let mut project = SourceProject::from_config(config1)
            .parse(&mut messages)
            .analyze();
        assert_eq!(messages, vec![]);

        // Invalid library should only be reported in source1
        let diagnostics = project.diagnostics();
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].pos.source, source1); // No such library
        assert_eq!(diagnostics[1].pos.source, source1); // No declaration

        // Change configuration file
        project.update_config(config2, &mut messages);
        assert_eq!(messages, vec![]);

        // Invalid library should only be reported in source2
        let diagnostics = project.diagnostics();
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].pos.source, source2); // No such library
        assert_eq!(diagnostics[1].pos.source, source2); // No declaration
    }
}
