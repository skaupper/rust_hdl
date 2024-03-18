use std::path::{Path, PathBuf};

use fnv::{FnvHashMap, FnvHashSet};

use super::parsed_project::ParsedProject;
use crate::data::{FilePath, Symbol};
use crate::{Config, Message, MessageHandler, Source, SourceFile, VHDLParser};

struct UnparsedSourceFile {
    library_names: Vec<String>,
    source: Source,
}

/// A completely unprocessed project, which is responsible for collecting configuration files and source files.
#[derive(Default)]
pub struct SourceProject {
    configs: Vec<Config>,
    files: FnvHashMap<FilePath, UnparsedSourceFile>,
    preloaded_sources: FnvHashMap<FilePath, Source>,
}

impl SourceProject {
    //
    // Constructors
    //
    pub fn from_config_file<T: AsRef<Path>>(path: T) -> std::io::Result<Self> {
        let mut inst = Self::default();
        inst.add_config_from_file(path)?;
        Ok(inst)
    }

    pub fn from_config(config: Config) -> Self {
        let mut inst = Self::default();
        inst.add_config(config);
        inst
    }

    //
    // Source file and config management
    //
    pub fn add_source_file<T: AsRef<Path>, U: Into<String>>(
        &mut self,
        path: T,
        library_name: U,
    ) -> std::io::Result<()> {
        let file_path = FilePath::new(path.as_ref());
        let source = Source::from_latin1_file(path.as_ref())?;
        self.files.insert(
            file_path,
            UnparsedSourceFile {
                library_names: vec![library_name.into()],
                source,
            },
        );
        Ok(())
    }

    pub fn add_config_from_file<T: AsRef<Path>>(&mut self, path: T) -> std::io::Result<()> {
        let config = Config::read_file_path(path.as_ref())?;
        self.add_config(config);
        Ok(())
    }

    pub fn add_config(&mut self, config: Config) {
        self.configs.push(config);
    }

    pub fn add_preloaded_source_files(&mut self, source_files: impl Iterator<Item = SourceFile>) {
        for source_file in source_files {
            self.preloaded_sources
                .insert(source_file.source.file_path().clone(), source_file.source);
        }
    }

    /// Process configs and source files and produce a `ParsedProject`
    pub fn parse(self, messages: &mut dyn MessageHandler) -> ParsedProject {
        use rayon::prelude::*;

        // Local intermediate type
        struct FileEntry {
            library_symbols: FnvHashSet<Symbol>,
            source: Source,
        }

        // Merge all given configs (if any)
        let config = self
            .configs
            .into_iter()
            .reduce(|mut lhs, rhs| {
                lhs.append(&rhs, messages);
                lhs
            })
            .unwrap_or(Config::default());

        let parser = VHDLParser::default();
        let mut preloaded_sources = self.preloaded_sources;
        let mut all_files = FnvHashMap::<PathBuf, FileEntry>::default();
        let mut empty_libraries = FnvHashSet::default();

        // Gather all source files referenced by configuration files
        for lib in config.iter_libraries() {
            let lib_symbol = parser.symbols.symtab().insert_utf8(lib.name());

            let files = lib.file_names(messages);
            if files.is_empty() {
                empty_libraries.insert(lib_symbol);
                continue;
            }
            // Iterate over files in library specification
            for file_path in files {
                if let Some(entry) = all_files.get_mut(&file_path) {
                    // File is already in list
                    entry.library_symbols.insert(lib_symbol.clone());
                    continue;
                }

                let source =
                    if let Some(source) = preloaded_sources.remove(&FilePath::new(&file_path)) {
                        // File is already loaded
                        source
                    } else {
                        // File is not loaded
                        let source_result = Source::from_latin1_file(&file_path);
                        match source_result {
                            Ok(result) => result,
                            Err(err) => {
                                messages.push(Message::file_error(err.to_string(), &file_path));
                                continue;
                            }
                        }
                    };

                all_files.insert(
                    file_path,
                    FileEntry {
                        library_symbols: FnvHashSet::from_iter([lib_symbol.clone()]),
                        source,
                    },
                );
            }
        }

        // Gather explicitely added source files
        for (file_path, unparsed_source_file) in self.files.into_iter() {
            if let Some(entry) = all_files.get_mut(&*file_path) {
                // File already exists
                for lib_name in unparsed_source_file.library_names {
                    entry
                        .library_symbols
                        .insert(parser.symbols.symtab().insert_utf8(&lib_name));
                }
            } else {
                // File needs an entry
                let library_symbols = unparsed_source_file
                    .library_names
                    .into_iter()
                    .map(|lib_name| parser.symbols.symtab().insert_utf8(&lib_name))
                    .collect();
                all_files.insert(
                    file_path.to_path_buf(),
                    FileEntry {
                        library_symbols,
                        source: unparsed_source_file.source,
                    },
                );
            }
        }

        // Parse all found files
        let mut parsed_project =
            ParsedProject::new(config, parser, FnvHashMap::default(), empty_libraries);
        let source_files = all_files
            .into_par_iter()
            .map_init(
                || &parsed_project.parser,
                |parser, (_, file_entry)| {
                    let mut diagnostics = Vec::new();
                    let design_file =
                        parser.parse_design_source(&file_entry.source, &mut diagnostics);
                    SourceFile {
                        library_names: file_entry.library_symbols,
                        source: file_entry.source,
                        design_file,
                        parser_diagnostics: diagnostics,
                    }
                },
            )
            .collect::<Vec<_>>();

        for f in source_files {
            parsed_project.add_source_file(f);
        }
        parsed_project
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::Diagnostic;

    use super::*;

    fn get_path(root: &Path, file_name: &str) -> FilePath {
        let mut buf = root.to_path_buf();
        buf.push(file_name);
        FilePath::new(&buf)
    }

    fn get_symbol(project: &ParsedProject, name: &str) -> Symbol {
        project.parser.symbols.symtab().insert_utf8(name)
    }

    fn create_file(root: &TempDir, file_name: &str, contents: &str) -> FilePath {
        let path = root.path().join(file_name);
        std::fs::write(&path, contents).unwrap();
        FilePath::new(&path)
    }

    fn gather_file_diagnostics(project: &ParsedProject) -> impl Iterator<Item = &Diagnostic> {
        project
            .parsed_files
            .values()
            .flat_map(|f| f.parser_diagnostics.iter())
    }

    #[test]
    fn test_empty_project() {
        let project = SourceProject::default();
        let mut messages = Vec::new();

        let parsed_project = project.parse(&mut messages);
        assert_eq!(messages.len(), 0);

        let diagnostics = gather_file_diagnostics(&parsed_project).collect::<Vec<_>>();
        assert_eq!(diagnostics.len(), 0);

        assert_eq!(parsed_project.parsed_files.len(), 0);
        assert_eq!(parsed_project.empty_libraries.len(), 0);
    }

    #[test]
    fn test_files_from_config() {
        let temp_root = tempfile::tempdir().unwrap();
        let file_path = create_file(
            &temp_root,
            "file.vhd",
            "library missing;\nentity ent is end entity;",
        );

        let config_str = "
[libraries]
missing.files = []
lib.files = ['file.vhd']
        ";

        let config = Config::from_str(config_str, temp_root.path()).unwrap();
        let project = SourceProject::from_config(config);
        let mut messages = Vec::new();

        let parsed_project = project.parse(&mut messages);
        assert_eq!(messages.len(), 0);

        assert_eq!(
            parsed_project.empty_libraries,
            FnvHashSet::from_iter([get_symbol(&parsed_project, "missing")])
        );
        assert_eq!(parsed_project.parsed_files.len(), 1);
        assert!(parsed_project.parsed_files.contains_key(&file_path));
    }

    #[test]
    fn test_files_added_manually() {
        let temp_root = tempfile::tempdir().unwrap();
        let file_path = create_file(
            &temp_root,
            "file_manual.vhd",
            "entity ent is end entity;",
        );

        let mut project = SourceProject::default();
        let mut messages = Vec::new();

        project
            .add_source_file(
                &get_path(temp_root.path(), "file_manual.vhd").to_path_buf(),
                "ent_lib",
            )
            .unwrap();

        let parsed_project = project.parse(&mut messages);
        assert_eq!(messages.len(), 0);

        assert_eq!(parsed_project.empty_libraries.len(), 0);
        assert_eq!(parsed_project.parsed_files.len(), 1);
        assert!(parsed_project.parsed_files.contains_key(&file_path));
    }

    #[test]
    fn test_mix_configs_and_manual_files() {
        let temp_root = tempfile::tempdir().unwrap();
        let file_path_1 = create_file(
            &temp_root,
            "file1.vhd",
            "entity ent1 is end entity;",
        );
        let file_path_2 = create_file(
            &temp_root,
            "file2.vhd",
            "architecture arch for ent1 is end architecture;",
        );
        let config1 = create_file(
            &temp_root,
            "config1.toml",
            "[libraries]\nlib1.files = ['file1.vhd', 'file2.vhd']",
        );

        let file_path_3 = create_file(
            &temp_root,
            "file3.vhd",
            "entity ent2 is end entity;",
        );
        let file_path_4 = create_file(
            &temp_root,
            "file4.vhd",
            "architecture arch for ent2 is end architecture;",
        );
        let config2 = create_file(
            &temp_root,
            "config2.toml",
            "[libraries]\nlib2.files = ['file3.vhd', 'file4.vhd']",
        );

        let file_path_5 = create_file(
            &temp_root,
            "lib1_arch2.vhd",
            "architecture alternative for ent1 is end architecture;",
        );

        let mut project = SourceProject::from_config_file(config1.to_path_buf()).unwrap();
        let mut messages = Vec::new();

        // This source file which contains only an architecture will create a new library.
        // During parsing it is not relevant if there is a corresponding entity defined!
        project
            .add_source_file(file_path_5.to_path_buf(), "lib_missing")
            .unwrap();

        project.add_config_from_file(config2.to_path_buf()).unwrap();

        let parsed_project = project.parse(&mut messages);
        assert_eq!(messages.len(), 0);

        assert_eq!(parsed_project.empty_libraries.len(), 0);
        assert_eq!(parsed_project.parsed_files.len(), 5);
        assert!(parsed_project.parsed_files.contains_key(&file_path_1));
        assert!(parsed_project.parsed_files.contains_key(&file_path_2));
        assert!(parsed_project.parsed_files.contains_key(&file_path_3));
        assert!(parsed_project.parsed_files.contains_key(&file_path_4));
        assert!(parsed_project.parsed_files.contains_key(&file_path_5));
    }

    #[test]
    fn test_preloaded_sources() {
        let mut messages = Vec::new();

        let temp_root = tempfile::tempdir().unwrap();
        let file_path = create_file(
            &temp_root,
            "file1.vhd",
            "entity ent1 is end entity;",
        );
        let config_path = create_file(
            &temp_root,
            "config.toml",
            "[libraries]\nlib.files = ['file1.vhd']",
        );

        // Create a project from config
        let mut parsed_project = SourceProject::from_config_file(config_path.to_path_buf())
            .unwrap()
            .parse(&mut messages);
        assert_eq!(messages.len(), 0);
        assert_eq!(parsed_project.parsed_files.len(), 1);
        assert!(parsed_project.parsed_files.contains_key(&file_path));

        // Create a new project and use the already loaded source file to preload "file1.vhd"
        let source_file = parsed_project.parsed_files.remove(&file_path).unwrap();
        let mut project = SourceProject::from_config_file(config_path.to_path_buf()).unwrap();

        project.add_preloaded_source_files([source_file].into_iter());

        // Write some non-VHDL content to 'file1.vhd' which would cause the parser to fail, if it reloaded the file's contents
        println!(
            "{}",
            std::fs::read_to_string(file_path.to_path_buf()).unwrap()
        );
        let new_file_content = "/* Definitely not VHDL */ !";
        create_file(
            &temp_root,
            file_path.file_name().unwrap().to_str().unwrap(),
            new_file_content,
        );
        assert_eq!(
            std::fs::read_to_string(file_path.to_path_buf()).unwrap(),
            new_file_content
        );

        // Try parsing the project
        let parsed_project = project.parse(&mut messages);
        assert_eq!(messages.len(), 0);

        let diagnostics = gather_file_diagnostics(&parsed_project).collect::<Vec<_>>();
        assert!(diagnostics.is_empty());
        assert_eq!(parsed_project.parsed_files.len(), 1);
        assert!(parsed_project.parsed_files.contains_key(&file_path));

        // Creating the same project without the preloaded source should result in a parser error
        let project = SourceProject::from_config_file(config_path.to_path_buf()).unwrap();
        let parsed_project = project.parse(&mut messages);
        assert_eq!(messages.len(), 0);

        let diagnostics = gather_file_diagnostics(&parsed_project)
            .cloned()
            .collect::<Vec<_>>();
        assert!(!diagnostics.is_empty());
        assert_eq!(parsed_project.parsed_files.len(), 1);
    }
}
