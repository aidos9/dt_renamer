use std::collections::HashMap;
use std::path::PathBuf;

use super::{InsertionType, Position, Selection, SortDirection};
use crate::error::Error;
use crate::operations::{DirOperation, FileOperation};
use crate::rename_tree::{Dir, File};

use convert_case::Casing;
#[cfg(feature = "regex_match")]
use regex::Regex;

#[derive(Debug, Default, Clone)]
pub struct OperationEngine {
    global_index: usize,
    local_index: usize,
    variables: HashMap<String, String>,
    dir_operations: Vec<Box<dyn DirOperation>>,
    file_operations: Vec<Box<dyn FileOperation>>,
    current_file: usize,
    files: Vec<File>,
}

impl OperationEngine {
    pub fn new(
        dir_operations: Vec<Box<dyn DirOperation>>,
        file_operations: Vec<Box<dyn FileOperation>>,
    ) -> Self {
        return Self {
            global_index: 0,
            local_index: 0,
            variables: Default::default(),
            dir_operations,
            file_operations,
            current_file: 0,
            files: Default::default(),
        };
    }

    pub fn process_dir(&mut self, mut dir: Dir) -> Result<(), Error> {
        self.local_index = 0;

        let mut files = std::mem::take(&mut dir.contents);

        for op in std::mem::take(&mut self.dir_operations) {
            op.execute(self, &mut files)?;
        }

        for op in dir.dir_ops {
            op.execute(self, &mut files)?;
        }

        self.files = files;

        return Ok(());
    }

    fn run_files(&mut self) {}

    pub fn process_file(&mut self, file: File) -> Result<(), Error> {
        self.local_index = 0;

        return self.run_file(file);
    }

    fn run_file(&mut self, mut file: File) -> Result<(), Error> {
        let ops = self.file_operations.clone();

        for op in ops {
            op.execute(self, &mut file.destination)?;
        }

        for op in &file.ops {
            op.execute(self, &mut file.destination)?;
        }

        self.global_index += 1;
        self.local_index += 1;

        return Ok(());
    }

    fn get_file_name(path: &PathBuf) -> Result<String, Error> {
        return path
            .file_name()
            .map(|f_name| {
                f_name
                    .to_os_string()
                    .into_string()
                    .map_err(|_| Error::CannotIdentifyFileName)
            })
            .ok_or(Error::CannotIdentifyFileName)?;
    }

    pub(crate) fn local_index(&self) -> usize {
        return self.local_index;
    }

    pub(crate) fn set_local_index(&mut self, index: usize) {
        self.local_index = index;
    }

    pub(crate) fn global_index(&self) -> usize {
        return self.global_index;
    }

    pub(crate) fn set_variable(&mut self, var_name: String, value: String) {
        self.variables.insert(var_name, value);
    }

    pub(crate) fn get_variable(&self, var_name: &str) -> Option<String> {
        return self.variables.get(var_name).map(|s| s.clone());
    }

    pub(crate) fn current_file(&self) -> &File {
        return &self.files[self.current_file];
    }
}
