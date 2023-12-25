use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

use crate::error::Error;
use crate::operations::{DirOperation, FileOperation};
use crate::OperationEngine;

use dt_walker::{DTWalker, DirProperties};
#[cfg(feature = "serializable")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RenameTree {
    file_set: BTreeSet<PathBuf>,
    files: Vec<File>,
}

#[derive(Debug, Default, Clone)]
pub struct RTBuilder {
    directories: Vec<Dir>,
    files: Vec<File>,
    dir_ops: Vec<Box<dyn DirOperation>>,
    file_ops: Vec<Box<dyn FileOperation>>,
}

#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "regex_match"), derive(PartialEq, Eq))]
pub struct Dir {
    pub(crate) path: PathBuf,
    pub(crate) recursive: bool,
    pub(crate) dir_ops: Vec<Box<dyn DirOperation>>,
    pub(crate) file_ops: Vec<Box<dyn FileOperation>>,
    pub(crate) contents: Vec<File>,
    pub(crate) processed: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "regex_match"), derive(PartialEq, Eq))]
pub struct File {
    pub(crate) source: PathBuf,
    pub(crate) ops: Vec<Box<dyn FileOperation>>,
    pub(crate) destination: PathBuf,
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
#[cfg_attr(feature = "serializable", derive(Serialize, Deserialize))]
pub struct RenameResult {
    source: PathBuf,
    destination: PathBuf,
}

impl RTBuilder {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn with_dir_op<O: DirOperation + 'static>(mut self, op: O) -> Self {
        self.dir_ops.push(Box::new(op));

        return self;
    }

    pub fn with_dir_ops(mut self, ops: &[Box<dyn DirOperation>]) -> Self {
        self.dir_ops.extend_from_slice(ops);

        return self;
    }

    pub fn with_file_op<O: FileOperation + 'static>(mut self, op: O) -> Self {
        self.file_ops.push(Box::new(op));

        return self;
    }

    pub fn with_file_ops(mut self, ops: &[Box<dyn FileOperation>]) -> Self {
        self.file_ops.extend_from_slice(ops);

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
    fn build_from_builder(builder: RTBuilder) -> Result<Self, Error> {
        let mut op_engine = OperationEngine::new(builder.dir_ops, builder.file_ops);

        for mut dir in builder.directories {
            dir.build()?;

            op_engine.process_dir(dir)?
        }

        for f in &builder.files {
            f.validate()?;
        }

        return Ok(op_engine.into());
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

impl From<OperationEngine> for RenameTree {
    fn from(value: OperationEngine) -> Self {
        return Self {
            files: value.into_files(),
            file_set: Default::default(),
        };
    }
}

impl Dir {
    pub fn new<P: Into<PathBuf>>(path: P, recursive: bool) -> Self {
        return Self::new_with_ops(path, recursive, Default::default(), Default::default());
    }

    pub fn new_with_ops<P: Into<PathBuf>>(
        path: P,
        recursive: bool,
        dir_ops: Vec<Box<dyn DirOperation>>,
        file_ops: Vec<Box<dyn FileOperation>>,
    ) -> Self {
        return Self {
            path: path.into(),
            recursive,
            dir_ops,
            file_ops,
            contents: Default::default(),
            processed: false,
        };
    }

    pub fn with_dir_op<O: DirOperation + 'static>(mut self, op: O) -> Self {
        self.dir_ops.push(Box::new(op));

        return self;
    }

    pub fn with_dir_ops(mut self, ops: &mut Vec<Box<dyn DirOperation>>) -> Self {
        self.dir_ops.append(ops);

        return self;
    }

    pub fn with_file_op<O: FileOperation + 'static>(mut self, op: O) -> Self {
        self.file_ops.push(Box::new(op));

        return self;
    }

    pub fn with_file_rules(mut self, ops: &mut Vec<Box<dyn FileOperation>>) -> Self {
        self.file_ops.append(ops);

        return self;
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
                let f = File::new_with_ops(f.display().to_string(), self.file_ops.clone());

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
                            res.push(File::new_with_ops(
                                entry
                                    .path()
                                    .canonicalize()
                                    .map_err(|e| Error::CanonicalizeError(e))?
                                    .display()
                                    .to_string(),
                                self.file_ops.clone(),
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
        return Self::new_with_ops(path, Default::default());
    }

    pub fn new_with_ops<P: Into<PathBuf>>(path: P, ops: Vec<Box<dyn FileOperation>>) -> Self {
        let source = path.into();
        let destination = source.clone();

        return Self {
            source,
            ops,
            destination,
        };
    }

    pub fn with_op<O: FileOperation + 'static>(mut self, op: O) -> Self {
        self.ops.push(Box::new(op));

        return self;
    }

    pub fn with_ops(mut self, ops: &mut Vec<Box<dyn FileOperation>>) -> Self {
        self.ops.append(ops);

        return self;
    }

    pub fn destination_path_string(&self) -> String {
        return self.destination.display().to_string();
    }

    fn validate(&self) -> Result<(), Error> {
        let path = Path::new(&self.source);

        if !path.is_file() {
            return Err(Error::NotFile(self.source.display().to_string()));
        }

        return Ok(());
    }
}

impl RenameResult {
    pub fn destination_path_string(&self) -> Option<String> {
        return self.destination.to_str().map(|s| s.to_string());
    }

    pub fn source_path_string(&self) -> Option<String> {
        return self.source.to_str().map(|s| s.to_string());
    }
}

impl fmt::Display for RenameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            f,
            "{} -> {}",
            self.source.display(),
            self.destination.display()
        );
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // const ROOT_DIR_FILES: [&str; 2] = ["Cargo.toml", "README.md"];
    // const ALL_SRC_DIR_FILES: [&str; 9] = [
    //     "Cargo.toml",
    //     "README.md",
    //     "src/error.rs",
    //     "src/lib.rs",
    //     "src/rename_tree.rs",
    //     "src/rules/mod.rs",
    //     "src/rules/rule_engine.rs",
    //     "src/rules/file_rule.rs",
    //     "src/rules/match_rule.rs",
    // ];

    // fn dt_files_from_paths<const N: usize>(paths: [&str; N]) -> [File; N] {
    //     return make_full_paths_from_arr(paths).map(|p| File::new(p));
    // }

    // fn make_full_paths_from_arr<const N: usize>(paths: [&str; N]) -> [String; N] {
    //     return paths.map(|s| {
    //         let mut p = std::env::current_dir().unwrap();
    //         p.push(s);

    //         return p.canonicalize().unwrap().display().to_string();
    //     });
    // }

    // mod build {
    //     use itertools::Itertools;

    //     use super::*;

    //     #[test]
    //     fn test_build_flat_tree() {
    //         let mut structure = RTBuilder::new()
    //             .with_directory(Dir::new_with_ops(
    //                 std::env::current_dir().unwrap(),
    //                 false,
    //                 Vec::new(),
    //                 Vec::new(),
    //             ))
    //             .build_tree()
    //             .unwrap();

    //         structure.files.sort_by(|a, b| a.source.cmp(&b.source));

    //         let mut cmp = dt_files_from_paths(ROOT_DIR_FILES);

    //         cmp.sort_by(|a, b| a.source.cmp(&b.source));

    //         assert_eq!(
    //             structure.files.into_iter().map(|f| f.source).collect_vec(),
    //             cmp.map(|f| f.source)
    //         );
    //     }

    //     #[test]
    //     fn test_build_recursive_tree() {
    //         let mut structure = RTBuilder::new()
    //             .with_directory(Dir::new_with_ops(
    //                 std::env::current_dir().unwrap(),
    //                 true,
    //                 Vec::new(),
    //                 Vec::new(),
    //             ))
    //             .build_tree()
    //             .unwrap();

    //         structure.files.sort_by(|a, b| a.source.cmp(&b.source));

    //         let mut cmp = dt_files_from_paths(ALL_SRC_DIR_FILES);

    //         cmp.sort_by(|a, b| a.source.cmp(&b.source));

    //         assert_eq!(
    //             structure.files.into_iter().map(|f| f.source).collect_vec(),
    //             cmp.map(|f| f.source)
    //         );
    //     }
    // }

    // mod run {
    //     use itertools::Itertools;

    //     use super::*;

    //     #[test]
    //     fn test_skip_toml() {
    //         let result = RTBuilder::new()
    //             .with_directory(Dir::new_with_ops(
    //                 std::env::current_dir().unwrap(),
    //                 true,
    //                 vec![DirRule::IncludeOnly(MatchRule::Not(
    //                     MatchRule::EndsWith(".toml".to_string()).into(),
    //                 ))],
    //                 Vec::new(),
    //             ))
    //             .build_tree()
    //             .unwrap()
    //             .dry_run()
    //             .unwrap();

    //         let mut result = result
    //             .into_iter()
    //             .map(|r| r.destination.display().to_string())
    //             .collect_vec();

    //         result.sort();

    //         let mut cmp = make_full_paths_from_arr(ALL_SRC_DIR_FILES)
    //             .into_iter()
    //             .filter(|p| !p.ends_with(".toml"))
    //             .collect_vec();

    //         cmp.sort();

    //         assert_eq!(result, cmp);
    //     }

    //     #[test]
    //     fn test_skip_toml_append2() {
    //         let result = RTBuilder::new()
    //             .with_directory(Dir::new_with_rules(
    //                 std::env::current_dir().unwrap(),
    //                 true,
    //                 vec![DirRule::IncludeOnly(MatchRule::Not(
    //                     MatchRule::EndsWith(".toml".to_string()).into(),
    //                 ))],
    //                 vec![FileRule::Insert(
    //                     Position::End,
    //                     InsertionType::Static("2".to_string()),
    //                 )],
    //             ))
    //             .build_tree()
    //             .unwrap()
    //             .dry_run()
    //             .unwrap();

    //         let mut result = result
    //             .into_iter()
    //             .map(|r| r.destination.display().to_string())
    //             .collect_vec();

    //         result.sort();

    //         let mut cmp = make_full_paths_from_arr(ALL_SRC_DIR_FILES)
    //             .into_iter()
    //             .filter(|p| !p.ends_with(".toml"))
    //             .map(|mut s| {
    //                 s.push_str("2");
    //                 s
    //             })
    //             .collect_vec();

    //         cmp.sort();

    //         assert_eq!(result, cmp);
    //     }

    //     #[test]
    //     fn test_only_toml() {
    //         let result = RTBuilder::new()
    //             .with_directory(Dir::new_with_rules(
    //                 std::env::current_dir().unwrap(),
    //                 true,
    //                 vec![DirRule::IncludeOnly(MatchRule::EndsWith(
    //                     ".toml".to_string(),
    //                 ))],
    //                 Vec::new(),
    //             ))
    //             .build_tree()
    //             .unwrap()
    //             .dry_run()
    //             .unwrap();

    //         let mut result = result
    //             .into_iter()
    //             .map(|r| r.destination.display().to_string())
    //             .collect_vec();

    //         result.sort();

    //         let mut cmp = make_full_paths_from_arr(ALL_SRC_DIR_FILES)
    //             .into_iter()
    //             .filter(|p| p.ends_with(".toml"))
    //             .collect_vec();

    //         cmp.sort();

    //         assert_eq!(result, cmp);
    //     }

    //     #[test]
    //     fn test_only_toml_replace_toml() {
    //         let result = RTBuilder::new()
    //             .with_directory(Dir::new_with_rules(
    //                 std::env::current_dir().unwrap(),
    //                 true,
    //                 vec![DirRule::IncludeOnly(MatchRule::EndsWith(
    //                     ".toml".to_string(),
    //                 ))],
    //                 vec![FileRule::Replace(
    //                     Selection::All,
    //                     ".toml".to_string(),
    //                     ".no".to_string(),
    //                 )],
    //             ))
    //             .build_tree()
    //             .unwrap()
    //             .dry_run()
    //             .unwrap();

    //         let mut result = result
    //             .into_iter()
    //             .map(|r| r.destination.display().to_string())
    //             .collect_vec();

    //         result.sort();

    //         let mut cmp = make_full_paths_from_arr(ALL_SRC_DIR_FILES)
    //             .into_iter()
    //             .filter(|p| p.ends_with(".toml"))
    //             .map(|s| s.replace(".toml", ".no"))
    //             .collect_vec();

    //         cmp.sort();

    //         assert_eq!(result, cmp);
    //     }

    //     #[test]
    //     fn test_only_md_set_test_rs() {
    //         let result = RTBuilder::new()
    //             .with_directory(Dir::new_with_rules(
    //                 std::env::current_dir().unwrap(),
    //                 true,
    //                 vec![DirRule::IncludeOnly(MatchRule::EndsWith(".md".to_string()))],
    //                 vec![FileRule::Set("test.md".to_string())],
    //             ))
    //             .build_tree()
    //             .unwrap()
    //             .dry_run()
    //             .unwrap();

    //         let mut result = result.into_iter().map(|r| r.destination).collect_vec();

    //         result.sort();

    //         let mut pth = std::env::current_dir().unwrap().canonicalize().unwrap();

    //         pth.push("test.md");

    //         let mut cmp = vec![pth];

    //         cmp.sort();

    //         assert_eq!(result, cmp);
    //     }
    // }
}
