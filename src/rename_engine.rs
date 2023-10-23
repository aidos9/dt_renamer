use std::path::PathBuf;

use crate::error::Error;
use crate::rules::rule::{DirRule, FileRule};
use crate::rules::RuleEngine;

#[derive(Debug)]
pub struct RenameEngine {
    builder: Builder,
    global_index: usize,
    rule_engine: RuleEngine,
}

#[derive(Debug, Default)]
pub struct Builder {
    directories: Vec<Dir>,
    files: Vec<File>,
    dir_rules: Vec<DirRule>,
    file_rules: Vec<FileRule>,
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
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

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct File {
    source: String,
    rules: Vec<FileRule>,
    destination: String,
    processed: bool,
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct DryRunResult {
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
        let mut engine: RenameEngine = self.into();

        engine.build_tree()?;

        return Ok(engine);
    }
}

impl RenameEngine {
    fn build_tree(&mut self) -> Result<(), Error> {
        for i in 0..self.builder.directories.len() {
            self.build_dir(&mut self.builder.directories[i])?;
        }

        todo!();
    }

    fn build_dir<'a>(&'a mut self, dir: &'a mut Dir) -> Result<(), Error> {
        // let
        todo!();
    }
}

impl From<Builder> for RenameEngine {
    fn from(value: Builder) -> Self {
        return Self {
            builder: value,
            global_index: 0,
            rule_engine: RuleEngine::new(),
        };
    }
}
