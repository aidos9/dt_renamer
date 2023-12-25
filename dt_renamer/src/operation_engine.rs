use std::collections::HashMap;

use crate::error::Error;
use crate::operations::{DirOperation, FileOperation};
use crate::rename_tree::{Dir, File};

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

        return self.run_files(files);
    }

    fn run_files(&mut self, files: Vec<File>) -> Result<(), Error> {
        self.files = files;

        while self.current_file < self.files.len() {
            self.run_file()?;

            self.current_file += 1;
        }

        return Ok(());
    }

    pub fn process_file(&mut self, file: File) -> Result<(), Error> {
        self.local_index = 0;
        self.files = vec![file];
        self.current_file = 0;

        return self.run_file();
    }

    fn run_file(&mut self) -> Result<(), Error> {
        let ops = self.file_operations.clone();

        for op in ops {
            op.execute(self)?;
        }

        let ops = self.current_file().ops.clone();

        for op in ops {
            op.execute(self)?;
        }

        self.global_index += 1;
        self.local_index += 1;

        return Ok(());
    }

    pub(crate) fn set_local_index(&mut self, index: usize) {
        self.local_index = index;
    }

    pub(crate) fn set_variable(&mut self, var_name: String, value: String) {
        self.variables.insert(var_name, value);
    }

    pub(crate) fn get_variable(&self, var_name: &str) -> Option<String> {
        return match var_name {
            "global_index" => Some(self.global_index.to_string()),
            "local_index" => Some(self.local_index.to_string()),
            s => self.variables.get(s).map(|s| s.clone()),
        };
    }

    pub(crate) fn current_file(&mut self) -> &mut File {
        return &mut self.files[self.current_file];
    }

    pub fn into_files(self) -> Vec<File> {
        return self.files;
    }
}
