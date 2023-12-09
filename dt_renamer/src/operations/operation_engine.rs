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

    // fn execute_dir_rule(&mut self, rule: &DirRule, input: &mut Vec<File>) -> Result<(), Error> {
    //     match rule {
    //         DirRule::Sort(d) => Self::sort(*d, input),
    //         DirRule::Remove(rule) => {
    //             let mut res = Vec::new();

    //             for f in input.drain(0..) {
    //                 if !rule.resolve(
    //                     &f.destination
    //                         .file_name()
    //                         .ok_or(Error::CannotIdentifyFileName)?
    //                         .to_str()
    //                         .ok_or(Error::CannotIdentifyFileName)?
    //                         .to_string(),
    //                 ) {
    //                     res.push(f);
    //                 }
    //             }

    //             let _ = std::mem::replace(input, res);
    //         }
    //         DirRule::IncludeOnly(rule) => {
    //             let mut res = Vec::new();

    //             for f in input.drain(0..) {
    //                 if rule.resolve(
    //                     &f.destination
    //                         .file_name()
    //                         .ok_or(Error::CannotIdentifyFileName)?
    //                         .to_str()
    //                         .ok_or(Error::CannotIdentifyFileName)?
    //                         .to_string(),
    //                 ) {
    //                     res.push(f);
    //                 }
    //             }

    //             let _ = std::mem::replace(input, res);
    //         }
    //         DirRule::OffsetLocalIndex(i) => self.local_index = *i,
    //     }

    //     return Ok(());
    // }

    // fn sort(direction: SortDirection, input: &mut Vec<File>) {
    //     match direction {
    //         SortDirection::Ascending => input.sort_by(|a, b| a.destination.cmp(&b.destination)),
    //         SortDirection::Descending => input.sort_by(|a, b| b.destination.cmp(&a.destination)),
    //     }
    // }

    // fn execute_file_rule(&self, rule: &FileRule, input: &mut PathBuf) -> Result<(), Error> {
    //     match rule {
    //         #[cfg(feature = "regex_match")]
    //         FileRule::RegexReplace(selection, find, replace) => {
    //             let new_f_name = match input
    //                 .file_name()
    //                 .map(|f_name| f_name.to_os_string().into_string())
    //             {
    //                 Some(Ok(f_name)) => Self::regex_replace(f_name, *selection, find, replace),
    //                 _ => return Err(Error::CannotIdentifyFileName),
    //             };

    //             input.set_file_name(new_f_name);
    //         }
    //         FileRule::Replace(selection, find, replace) => {
    //             let new_f_name = match input
    //                 .file_name()
    //                 .map(|f_name| f_name.to_os_string().into_string())
    //             {
    //                 Some(Ok(f_name)) => Self::replace(f_name, *selection, find, replace),
    //                 _ => return Err(Error::CannotIdentifyFileName),
    //             };

    //             input.set_file_name(new_f_name);
    //         }
    //         FileRule::Insert(pos, tp) => {
    //             let content = match tp {
    //                 InsertionType::LocalIndex => self.local_index.to_string(),
    //                 InsertionType::OverallIndex => self.global_index.to_string(),
    //                 InsertionType::Static(s) => s.clone(),
    //                 InsertionType::Variable(_) => todo!(),
    //             };

    //             let mut old_f_name = input
    //                 .file_name()
    //                 .ok_or(Error::CannotIdentifyFileName)
    //                 .map(|f_name| {
    //                     f_name
    //                         .to_os_string()
    //                         .into_string()
    //                         .map_err(|_| Error::CannotIdentifyFileName)
    //                 })??;

    //             let new_f_name = match pos {
    //                 Position::Index(i) => {
    //                     if *i > old_f_name.len() {
    //                         return Err(Error::InsertIndexTooLarge);
    //                     }

    //                     old_f_name.insert_str(*i, &content);

    //                     old_f_name
    //                 }
    //                 Position::After(f) => {
    //                     if let Some(i) = old_f_name.find(f) {
    //                         if i + f.len() > old_f_name.len() {
    //                             old_f_name.push_str(&content);
    //                         } else {
    //                             old_f_name.insert_str(i + f.len(), &content);
    //                         }
    //                     }

    //                     old_f_name
    //                 }
    //                 Position::AfterRegex(r) => {
    //                     if let Some(m) = r.find(&old_f_name) {
    //                         if m.end() >= old_f_name.len() {
    //                             old_f_name.push_str(&content);
    //                         } else {
    //                             old_f_name.insert_str(m.end(), &content);
    //                         }
    //                     }

    //                     old_f_name
    //                 }
    //                 Position::Before(f) => {
    //                     if let Some(i) = old_f_name.find(f) {
    //                         old_f_name.insert_str(i, &content);
    //                     }

    //                     old_f_name
    //                 }
    //                 Position::BeforeRegex(r) => {
    //                     if let Some(m) = r.find(&old_f_name) {
    //                         old_f_name.insert_str(m.start(), &content);
    //                     }

    //                     old_f_name
    //                 }
    //                 Position::Start => {
    //                     let mut c = content;
    //                     c.push_str(&old_f_name);

    //                     c
    //                 }
    //                 Position::End => {
    //                     old_f_name.push_str(&content);

    //                     old_f_name
    //                 }
    //             };

    //             input.set_file_name(new_f_name);
    //         }
    //         FileRule::Set(s) => input.set_file_name(s),
    //         FileRule::Left(m, inclusive) => {
    //             input.set_file_name(Self::left(Self::get_file_name(input)?, m, *inclusive));
    //         }
    //         FileRule::Right(m, inclusive) => {
    //             input.set_file_name(Self::right(Self::get_file_name(input)?, m, *inclusive));
    //         }
    //         #[cfg(feature = "regex_match")]
    //         FileRule::RegexLeft(reg, inclusive) => {
    //             input.set_file_name(Self::regex_left(
    //                 Self::get_file_name(input)?,
    //                 reg,
    //                 *inclusive,
    //             ));
    //         }
    //         #[cfg(feature = "regex_match")]
    //         FileRule::RegexRight(reg, inclusive) => {
    //             input.set_file_name(Self::regex_right(
    //                 Self::get_file_name(input)?,
    //                 reg,
    //                 *inclusive,
    //             ));
    //         }
    //         FileRule::SetCase(case) => {
    //             input.set_file_name(Self::get_file_name(input)?.to_case(*case))
    //         }
    //         FileRule::SetLowerCase => {
    //             input.set_file_name(Self::get_file_name(input)?.to_lowercase())
    //         }
    //         FileRule::SetUpperCase => {
    //             input.set_file_name(Self::get_file_name(input)?.to_uppercase())
    //         }
    //         #[cfg(feature = "regex_match")]
    //         FileRule::RegexOnly(reg) => {
    //             let fname = Self::get_file_name(&input)?;

    //             if let Some(m) = Self::regex_only(&fname, reg) {
    //                 input.set_file_name(m);
    //             }
    //         }
    //         FileRule::If(condition, then_rule, else_rule) => {
    //             if condition.resolve(&input.display().to_string()) {
    //                 return self.execute_file_rule(then_rule, input);
    //             } else if let Some(else_rule) = else_rule {
    //                 return self.execute_file_rule(else_rule, input);
    //             }
    //         }
    //         FileRule::Array(rules) => {
    //             for r in rules {
    //                 self.execute_file_rule(r, input)?;
    //             }
    //         }
    //         FileRule::DefineVariable(_, _) => todo!(),
    //     };

    //     return Ok(());
    // }

    // #[cfg(feature = "regex_match")]
    // fn regex_only<'a>(input: &'a str, reg: &Regex) -> Option<&'a str> {
    //     let m = reg.find(input);

    //     return m.map(|m| m.as_str());
    // }

    // #[cfg(feature = "regex_match")]
    // fn regex_left(mut input: String, reg: &Regex, inclusive: bool) -> String {
    //     if let Some(m) = reg.find(&input) {
    //         if inclusive {
    //             input = input[..m.end()].to_string();
    //         } else {
    //             input = input[..m.start()].to_string();
    //         }
    //     }

    //     return input;
    // }

    // #[cfg(feature = "regex_match")]
    // fn regex_right(mut input: String, reg: &Regex, inclusive: bool) -> String {
    //     if let Some(m) = reg.find(&input) {
    //         if inclusive {
    //             input = input[m.start()..].to_string();
    //         } else {
    //             input = input[m.end()..].to_string();
    //         }
    //     }

    //     return input;
    // }

    // fn left(mut input: String, match_str: &str, inclusive: bool) -> String {
    //     if let Some(mut slice) = input.find(match_str) {
    //         if inclusive {
    //             slice += match_str.len();
    //         }

    //         input = input[..slice].to_string()
    //     }

    //     return input;
    // }

    // fn right(mut input: String, match_str: &str, inclusive: bool) -> String {
    //     if let Some(mut slice) = input.find(match_str) {
    //         if !inclusive {
    //             slice += match_str.len();
    //         }

    //         input = input[slice..].to_string()
    //     }

    //     return input;
    // }

    // fn replace(input: String, selection: Selection, find: &String, replace: &String) -> String {
    //     return match selection {
    //         Selection::First => {
    //             // Could be better optimized

    //             if let Some(slice) = input.find(find) {
    //                 return [
    //                     &input[0..slice],
    //                     replace.as_str(),
    //                     &input[slice + find.len()..],
    //                 ]
    //                 .join("");
    //             } else {
    //                 return input;
    //             }
    //         }
    //         Selection::Last => {
    //             // Could be better optimized

    //             if let Some(slice) = input.rfind(find) {
    //                 return [
    //                     &input[0..slice],
    //                     replace.as_str(),
    //                     &input[slice + find.len()..],
    //                 ]
    //                 .join("");
    //             } else {
    //                 return input;
    //             }
    //         }
    //         Selection::All => input.replace(find, replace),
    //     };
    // }

    // #[cfg(feature = "regex_match")]
    // fn regex_replace(input: String, selection: Selection, find: &Regex, replace: &str) -> String {
    //     return match selection {
    //         Selection::First => find.replace(&input, replace).to_string(),
    //         Selection::Last => {
    //             let i = find.find_iter(&input);

    //             if let Some(m) = i.last() {
    //                 format!("{}{}{}", &input[0..m.start()], replace, &input[m.end()..])
    //             } else {
    //                 input
    //             }
    //         }
    //         Selection::All => find.replace_all(&input, replace).to_string(),
    //     };
    // }

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

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_replace_first_1() {
    //     assert_eq!(
    //         OperationEngine::replace(
    //             "test message hello".to_string(),
    //             Selection::First,
    //             &"message".to_string(),
    //             &"yo".to_string()
    //         ),
    //         "test yo hello"
    //     );
    // }

    // #[test]
    // fn test_replace_first_2() {
    //     assert_eq!(
    //         OperationEngine::replace(
    //             "test message message hello".to_string(),
    //             Selection::First,
    //             &"message".to_string(),
    //             &"yo".to_string()
    //         ),
    //         "test yo message hello"
    //     );
    // }

    // #[test]
    // fn test_replace_last_1() {
    //     assert_eq!(
    //         OperationEngine::replace(
    //             "test message hello".to_string(),
    //             Selection::Last,
    //             &"message".to_string(),
    //             &"yo".to_string()
    //         ),
    //         "test yo hello"
    //     );
    // }

    // #[test]
    // fn test_replace_last_2() {
    //     assert_eq!(
    //         OperationEngine::replace(
    //             "test message message hello".to_string(),
    //             Selection::Last,
    //             &"message".to_string(),
    //             &"yo".to_string()
    //         ),
    //         "test message yo hello"
    //     );
    // }

    // #[test]
    // fn test_left_1() {
    //     assert_eq!(
    //         OperationEngine::left(
    //             "test message message hello".to_string(),
    //             &"message".to_string(),
    //             true
    //         ),
    //         "test message"
    //     );
    // }

    // #[test]
    // fn test_left_2() {
    //     assert_eq!(
    //         OperationEngine::left(
    //             "test message message hello".to_string(),
    //             &"message".to_string(),
    //             false
    //         ),
    //         "test "
    //     );
    // }

    // #[test]
    // fn test_right_1() {
    //     assert_eq!(
    //         OperationEngine::right(
    //             "test message message hello".to_string(),
    //             &"message".to_string(),
    //             true
    //         ),
    //         "message message hello"
    //     );
    // }

    // #[test]
    // fn test_right_2() {
    //     assert_eq!(
    //         OperationEngine::right(
    //             "test message message hello".to_string(),
    //             &"message".to_string(),
    //             false
    //         ),
    //         " message hello"
    //     );
    // }

    // #[cfg(feature = "regex_match")]
    // mod regex {
    //     use super::*;

    //     #[test]
    //     fn test_regex_replace_first() {
    //         let r = Regex::new("test").unwrap();
    //         let input = "test cow test".to_string();

    //         let output = OperationEngine::regex_replace(input, Selection::First, &r, "cow");

    //         assert_eq!(output, "cow cow test");
    //     }

    //     #[test]
    //     fn test_regex_replace_last() {
    //         let r = Regex::new("test").unwrap();
    //         let input = "test cow test".to_string();

    //         let output = OperationEngine::regex_replace(input, Selection::Last, &r, "cow");

    //         assert_eq!(output, "test cow cow");
    //     }

    //     #[test]
    //     fn test_regex_replace_all() {
    //         let r = Regex::new("test").unwrap();
    //         let input = "test cow test".to_string();

    //         let output = OperationEngine::regex_replace(input, Selection::All, &r, "cow");

    //         assert_eq!(output, "cow cow cow");
    //     }

    //     #[test]
    //     fn test_regex_left_1() {
    //         assert_eq!(
    //             OperationEngine::regex_left(
    //                 "test message message hello".to_string(),
    //                 &Regex::new("message").unwrap(),
    //                 true
    //             ),
    //             "test message"
    //         );
    //     }

    //     #[test]
    //     fn test_regex_left_2() {
    //         assert_eq!(
    //             OperationEngine::regex_left(
    //                 "test message message hello".to_string(),
    //                 &Regex::new("message").unwrap(),
    //                 false
    //             ),
    //             "test "
    //         );
    //     }

    //     #[test]
    //     fn test_regex_right_1() {
    //         assert_eq!(
    //             OperationEngine::regex_right(
    //                 "test message message hello".to_string(),
    //                 &Regex::new("message").unwrap(),
    //                 true
    //             ),
    //             "message message hello"
    //         );
    //     }

    //     #[test]
    //     fn test_regex_right_2() {
    //         assert_eq!(
    //             OperationEngine::regex_right(
    //                 "test message message hello".to_string(),
    //                 &Regex::new("message").unwrap(),
    //                 false
    //             ),
    //             " message hello"
    //         );
    //     }

    //     #[test]
    //     fn test_regex_only_1() {
    //         assert_eq!(
    //             OperationEngine::regex_only(
    //                 "test message message hello",
    //                 &Regex::new("message").unwrap(),
    //             ),
    //             Some("message")
    //         );
    //     }
    // }
}
