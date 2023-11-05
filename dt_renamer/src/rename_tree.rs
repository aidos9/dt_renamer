use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::rules::rule::{DirRule, FileRule};
use crate::rules::RuleEngine;

use dt_walker::{DTWalker, DirProperties};

#[derive(Debug)]
pub struct RenameTree {
    rule_engine: RuleEngine,
    file_set: BTreeSet<PathBuf>,
    files: Vec<File>,
}

#[derive(Debug, Default)]
pub struct Builder {
    directories: Vec<Dir>,
    files: Vec<File>,
    dir_rules: Vec<DirRule>,
    file_rules: Vec<FileRule>,
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Dir {
    pub(crate) path: PathBuf,
    pub(crate) recursive: bool,
    pub(crate) dir_rules: Vec<DirRule>,
    pub(crate) file_rules: Vec<FileRule>,
    pub(crate) nested_dirs: Vec<Dir>,
    pub(crate) contents: Vec<File>,
    pub(crate) local_index: usize,
    pub(crate) processed: bool,
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct File {
    pub(crate) source: PathBuf,
    pub(crate) rules: Vec<FileRule>,
    pub(crate) destination: PathBuf,
    pub(crate) processed: bool,
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct RenameResult {
    source: PathBuf,
    destination: PathBuf,
}

impl Builder {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn with_dir_rule(mut self, rule: DirRule) -> Self {
        self.dir_rules.push(rule);

        return self;
    }

    pub fn with_dir_rules(mut self, rules: &[DirRule]) -> Self {
        self.dir_rules.extend_from_slice(rules);

        return self;
    }

    pub fn with_file_rule(mut self, rule: FileRule) -> Self {
        self.file_rules.push(rule);

        return self;
    }

    pub fn with_file_rules(mut self, rules: &[FileRule]) -> Self {
        self.file_rules.extend_from_slice(rules);

        return self;
    }

    pub fn with_directory(mut self, dir: Dir) -> Self {
        self.directories.push(dir);

        return self;
    }

    pub fn with_directories(mut self, dirs: &[Dir]) -> Self {
        self.directories.extend_from_slice(dirs);

        return self;
    }

    pub fn build_tree(self) -> Result<RenameTree, Error> {
        return RenameTree::build_from_builder(self);
    }
}

impl RenameTree {
    fn build_from_builder(mut builder: Builder) -> Result<Self, Error> {
        let mut engine = Self {
            rule_engine: RuleEngine::new(builder.dir_rules, builder.file_rules),
            file_set: BTreeSet::new(),
            files: Vec::new(),
        };

        for mut dir in builder.directories {
            dir.build()?;

            engine
                .files
                .append(&mut engine.rule_engine.process_dir(dir));
        }

        for f in &builder.files {
            f.validate()?;
        }

        engine.files.append(&mut builder.files);

        return Ok(engine);
    }

    pub fn run(self) -> Result<Vec<RenameResult>, Error> {
        return self.run_with_fn(Self::rename_file);
    }

    pub fn dry_run(self) -> Result<Vec<RenameResult>, Error> {
        return self.run_with_fn(Self::dry_rename_file);
    }

    fn run_with_fn(
        mut self,
        rename: fn(PathBuf, PathBuf) -> Result<RenameResult, Error>,
    ) -> Result<Vec<RenameResult>, Error> {
        let mut results = Vec::with_capacity(self.files.len());

        for file in self.files {
            if self.file_set.insert(file.source.clone()) {
                results.push(rename(file.source, file.destination)?);
            } else {
                return Err(Error::DuplicateFileError(file.source.display().to_string()));
            }
        }

        return Ok(results);
    }

    fn dry_rename_file(source: PathBuf, destination: PathBuf) -> Result<RenameResult, Error> {
        return Ok(RenameResult {
            source,
            destination,
        });
    }

    fn rename_file(source: PathBuf, destination: PathBuf) -> Result<RenameResult, Error> {
        return fs::rename(&source, &destination)
            .map_err(|e| Error::RenameError(e))
            .map(|_| RenameResult {
                source,
                destination,
            });
    }
}

impl Dir {
    pub fn new(
        path: PathBuf,
        recursive: bool,
        dir_rules: Vec<DirRule>,
        file_rules: Vec<FileRule>,
    ) -> Self {
        return Self {
            path,
            recursive,
            dir_rules,
            file_rules,
            nested_dirs: Vec::new(),
            contents: Vec::new(),
            local_index: 0,
            processed: false,
        };
    }

    fn build(&mut self) -> Result<(), Error> {
        let dir_path = Path::new(&self.path);

        if !dir_path.is_dir() {
            return Err(Error::NotDirectory(self.path.display().to_string()));
        }

        self.contents = if self.recursive {
            let mut res = Vec::new();

            for f in DTWalker::new(dir_path)
                .with_canonicalize()
                .with_dir_inclusions(DirProperties::Skip)
                .run()
                .map_err(|e| Error::WalkerError(e))?
                .into_iter()
            {
                let f = File::new_with_rules(f.display().to_string(), self.file_rules.clone());

                f.validate()?;

                res.push(f);
            }

            res
        } else {
            let contents = fs::read_dir(dir_path).map_err(|e| Error::ReadDirError(e))?;

            let mut res = Vec::new();

            for entry in contents {
                match entry {
                    Ok(entry) => {
                        let entry_path = entry.path();

                        if entry_path.is_file() {
                            res.push(File::new_with_rules(
                                entry
                                    .path()
                                    .canonicalize()
                                    .map_err(|e| Error::CanonicalizeError(e))?
                                    .display()
                                    .to_string(),
                                self.file_rules.clone(),
                            ));
                        }
                    }
                    Err(e) => return Err(Error::ReadDirEntryError(e)),
                }
            }

            res
        };

        self.processed = true;

        return Ok(());
    }
}

impl File {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        return Self {
            source: path.into(),
            rules: Vec::new(),
            destination: PathBuf::new(),
            processed: false,
        };
    }

    fn new_with_rules<P: Into<PathBuf>>(path: P, rules: Vec<FileRule>) -> Self {
        return Self {
            source: path.into(),
            rules,
            destination: PathBuf::new(),
            processed: false,
        };
    }

    pub fn with_rules(mut self, rules: Vec<FileRule>) -> Self {
        self.rules = rules;

        return self;
    }

    fn validate(&self) -> Result<(), Error> {
        let path = Path::new(&self.source);

        if !path.is_file() {
            return Err(Error::NotFile(self.source.display().to_string()));
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    mod build {
        use super::*;

        #[test]
        fn test_build_flat_tree() {
            let mut structure = Builder::new()
                .with_directory(Dir::new(
                    std::env::current_dir().unwrap(),
                    false,
                    Vec::new(),
                    Vec::new(),
                ))
                .build_tree()
                .unwrap();

            structure.files.sort_by(|a, b| a.source.cmp(&b.source));

            let mut cmp = ["/Cargo.toml", "/README.md"].map(|s| {
                File::new(
                    PathBuf::from_str(&format!(
                        "{}{}",
                        std::env::current_dir().unwrap().display(),
                        s
                    ))
                    .unwrap()
                    .canonicalize()
                    .unwrap()
                    .display()
                    .to_string(),
                )
            });

            cmp.sort_by(|a, b| a.source.cmp(&b.source));

            assert_eq!(structure.files, cmp);
        }

        #[test]
        fn test_build_recursive_tree() {
            let mut structure = Builder::new()
                .with_directory(Dir::new(
                    std::env::current_dir().unwrap(),
                    true,
                    Vec::new(),
                    Vec::new(),
                ))
                .build_tree()
                .unwrap();

            structure.files.sort_by(|a, b| a.source.cmp(&b.source));

            let mut cmp = [
                "/Cargo.toml",
                "/README.md",
                "/src/error.rs",
                "/src/lib.rs",
                "/src/rename_tree.rs",
                "/src/rules/mod.rs",
                "/src/rules/rule_engine.rs",
                "/src/rules/rule.rs",
            ]
            .map(|s| {
                File::new(
                    PathBuf::from_str(&format!(
                        "{}{}",
                        std::env::current_dir().unwrap().display(),
                        s
                    ))
                    .unwrap()
                    .canonicalize()
                    .unwrap()
                    .display()
                    .to_string(),
                )
            });

            cmp.sort_by(|a, b| a.source.cmp(&b.source));

            assert_eq!(structure.files, cmp);
        }
    }

    mod run {
        use super::*;

        // #[test]
        // fn test_
    }
}
