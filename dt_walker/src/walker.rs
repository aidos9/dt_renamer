use std::fs::read_dir;
use std::path::{Path, PathBuf};

use crate::error::Error;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum DirProperties {
    Skip,
    First,
    Last,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DTWalker {
    root: PathBuf,
    directory_inclusions: DirProperties,
    max_depth: usize,
    fail_on_depth: bool,
}

impl DTWalker {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        return Self {
            root: root.as_ref().into(),
            directory_inclusions: DirProperties::First,
            max_depth: usize::MAX,
            fail_on_depth: true,
        };
    }

    pub fn with_dir_inclusions(mut self, directory_inclusions: DirProperties) -> Self {
        self.directory_inclusions = directory_inclusions;

        return self;
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;

        return self;
    }

    pub fn without_fail_on_depth(mut self) -> Self {
        self.fail_on_depth = false;

        return self;
    }

    pub fn run(self) -> Result<Vec<PathBuf>, Error> {
        return self.visit_directory(self.root.clone(), 0);
    }

    fn visit_directory(&self, dir: PathBuf, depth: usize) -> Result<Vec<PathBuf>, Error> {
        if depth >= self.max_depth {
            if self.fail_on_depth {
                return Err(Error::MaxDepthReached);
            } else {
                return Ok(match self.directory_inclusions {
                    DirProperties::First | DirProperties::Last => vec![dir],
                    DirProperties::Skip => Vec::new(),
                });
            }
        }

        let mut results = match self.directory_inclusions {
            DirProperties::Skip | DirProperties::Last => Vec::new(),
            DirProperties::First => vec![dir.clone()],
        };

        let contents = read_dir(dir.clone()).map_err(|e| Error::ReadDirError(e))?;

        for entry in contents {
            match entry {
                Ok(d) => {
                    let p = d.path();

                    if p.is_dir() {
                        results.extend(self.visit_directory(p, depth + 1)?);
                    } else if p.is_file() {
                        results.push(p.canonicalize().map_err(|e| Error::CanonicalizeError(e))?);
                    }
                }
                Err(e) => return Err(Error::ReadDirError(e)),
            }
        }

        if self.directory_inclusions == DirProperties::Last {
            results.push(dir);
        }

        return Ok(results);
    }
}
