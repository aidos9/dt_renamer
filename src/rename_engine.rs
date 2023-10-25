use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::rules::rule::{DirRule, FileRule};
use crate::rules::RuleEngine;

use dt_walker::{DTWalker, DirProperties};

#[derive(Debug)]
pub struct RenameEngine {
    global_index: usize,
    rule_engine: RuleEngine,
    files: Vec<File>,
    dir_rules: Vec<DirRule>,
    file_rules: Vec<FileRule>,
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
    pub(crate) path: String,
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
    source: String,
    rules: Vec<FileRule>,
    destination: String,
    processed: bool,
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct RenameResult {
    source: String,
    destination: String,
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

    pub fn build(self) -> Result<RenameEngine, Error> {
        let mut engine = RenameEngine::default();

        engine.build_tree(self)?;

        return Ok(engine);
    }
}

impl RenameEngine {
    fn build_tree(&mut self, mut builder: Builder) -> Result<(), Error> {
        for mut dir in builder.directories {
            dir.build()?;

            self.files.append(&mut dir.contents);
        }

        for f in &builder.files {
            f.validate()?;
        }

        self.files.append(&mut builder.files);

        return Ok(());
    }

    pub fn run(self) -> Result<Vec<RenameResult>, Error> {
        return self.run_with_fn(Self::rename_file);
    }

    pub fn dry_run(self) -> Result<Vec<RenameResult>, Error> {
        return self.run_with_fn(Self::dry_rename_file);
    }

    fn run_with_fn(
        mut self,
        rename_func: fn(PathBuf, String) -> Result<RenameResult, Error>,
    ) -> Result<Vec<RenameResult>, Error> {
        todo!();
    }

    fn dry_rename_file(source: PathBuf, dest: String) -> Result<RenameResult, Error> {
        return Ok(RenameResult {
            source: source.display().to_string(),
            destination: dest,
        });
    }

    fn rename_file(source: PathBuf, dest: String) -> Result<RenameResult, Error> {
        let result = RenameResult {
            source: source.display().to_string(),
            destination: dest.clone(),
        };

        return fs::rename(source, dest)
            .map_err(|e| Error::RenameError(e))
            .map(|_| result);
    }
}

impl Default for RenameEngine {
    fn default() -> Self {
        return Self {
            global_index: 0,
            rule_engine: Default::default(),
            files: Default::default(),
            dir_rules: Default::default(),
            file_rules: Default::default(),
        };
    }
}

impl Dir {
    fn new(
        path: String,
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
            return Err(Error::NotDirectory(self.path.clone()));
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
    pub fn new(path: String) -> Self {
        return Self {
            source: path,
            rules: Vec::new(),
            destination: String::new(),
            processed: false,
        };
    }

    fn new_with_rules(path: String, rules: Vec<FileRule>) -> Self {
        return Self {
            source: path,
            rules,
            destination: String::new(),
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
            return Err(Error::NotFile(self.source.clone()));
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_build_flat_tree() {
        let structure = Builder::new()
            .with_directory(Dir::new(
                "dt_walker".to_string(),
                false,
                Vec::new(),
                Vec::new(),
            ))
            .build()
            .unwrap();

        assert_eq!(
            structure.files,
            vec![File::new(
                PathBuf::from_str("dt_walker/Cargo.toml")
                    .unwrap()
                    .canonicalize()
                    .unwrap()
                    .display()
                    .to_string()
            )]
        );
    }

    #[test]
    fn test_build_recursive_tree() {
        let structure = Builder::new()
            .with_directory(Dir::new(
                "dt_walker".to_string(),
                true,
                Vec::new(),
                Vec::new(),
            ))
            .build()
            .unwrap();

        assert_eq!(
            structure.files,
            vec![
                File::new(
                    PathBuf::from_str("dt_walker/Cargo.toml")
                        .unwrap()
                        .canonicalize()
                        .unwrap()
                        .display()
                        .to_string()
                ),
                File::new(
                    PathBuf::from_str("dt_walker/src/error.rs")
                        .unwrap()
                        .canonicalize()
                        .unwrap()
                        .display()
                        .to_string()
                ),
                File::new(
                    PathBuf::from_str("dt_walker/src/lib.rs")
                        .unwrap()
                        .canonicalize()
                        .unwrap()
                        .display()
                        .to_string()
                ),
                File::new(
                    PathBuf::from_str("dt_walker/src/walker.rs")
                        .unwrap()
                        .canonicalize()
                        .unwrap()
                        .display()
                        .to_string()
                )
            ]
        );
    }
}
