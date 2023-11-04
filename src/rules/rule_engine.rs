use std::path::PathBuf;

use itertools::Itertools;

use crate::{
    rename_tree::{Dir, File},
    rules::rule::{DirRule, FileRule, Selection, SortDirection},
};

#[derive(Debug, Default)]
pub struct RuleEngine {
    global_index: usize,
    local_index: usize,
    dir_rules: Vec<DirRule>,
    file_rules: Vec<FileRule>,
}

impl RuleEngine {
    pub fn new(dir_rules: Vec<DirRule>, file_rules: Vec<FileRule>) -> Self {
        return Self {
            global_index: 0,
            local_index: 0,
            dir_rules,
            file_rules,
        };
    }

    pub fn process_dir(&mut self, mut dir: Dir) -> Vec<File> {
        self.local_index = 0;

        let mut files = std::mem::take(&mut dir.contents);

        for rule in self.dir_rules.clone() {
            self.execute_dir_rule(&rule, &mut files);
        }

        for rule in &dir.dir_rules {
            self.execute_dir_rule(&rule, &mut files);
        }

        for f in &mut files {
            self.run_file(f);
        }

        return files;
    }

    pub fn process_file(&mut self, file: &mut File) {
        self.local_index = 0;
        self.run_file(file);
    }

    fn run_file(&mut self, file: &mut File) {
        for rule in &self.file_rules {
            self.execute_file_rule(rule, &mut file.source);
        }

        for rule in &file.rules {
            self.execute_file_rule(rule, &mut file.source);
        }

        self.global_index += 1;
        self.local_index += 1;
    }

    fn execute_dir_rule(&mut self, rule: &DirRule, input: &mut Vec<File>) {
        match rule {
            DirRule::Sort(d) => Self::sort(*d, input),
            DirRule::Remove(rule) => {
                let filtered = input
                    .drain(0..)
                    .filter(|f| !rule.resolve(&f.source.display().to_string()))
                    .collect_vec();

                let _ = std::mem::replace(input, filtered);
            }
            DirRule::IncludeOnly(rule) => {
                let filtered = input
                    .drain(0..)
                    .filter(|f| rule.resolve(&f.source.display().to_string()))
                    .collect_vec();

                let _ = std::mem::replace(input, filtered);
            }
            DirRule::OffsetLocalIndex(i) => self.local_index = *i,
        }
    }

    fn sort(direction: SortDirection, input: &mut Vec<File>) {
        match direction {
            SortDirection::Ascending => input.sort_by(|a, b| a.source.cmp(&b.source)),
            SortDirection::Descending => input.sort_by(|a, b| b.source.cmp(&a.source)),
        }
    }

    fn execute_file_rule(&self, rule: &FileRule, input: &mut PathBuf) -> bool {
        match rule {
            FileRule::Replace(selection, find, replace) => {
                let _ = std::mem::replace(
                    input,
                    PathBuf::from(Self::replace(
                        input.display().to_string(),
                        *selection,
                        find,
                        replace,
                    )),
                );
            }
            FileRule::Insert(_, _) => todo!(),
            FileRule::Set(s) => input.set_file_name(s),
            FileRule::SkipIf(rule) => {
                if rule.resolve(&input.display().to_string()) {
                    return false;
                }
            }
        };

        return true;
    }

    fn replace(input: String, selection: Selection, find: &String, replace: &String) -> String {
        return match selection {
            Selection::First => {
                // Could be better optimized

                if let Some(slice) = input.find(find) {
                    return [
                        &input[0..slice],
                        replace.as_str(),
                        &input[slice + find.len()..],
                    ]
                    .join("");
                } else {
                    return input;
                }
            }
            Selection::Last => {
                // Could be better optimized

                if let Some(slice) = input.rfind(find) {
                    return [
                        &input[0..slice],
                        replace.as_str(),
                        &input[slice + find.len()..],
                    ]
                    .join("");
                } else {
                    return input;
                }
            }
            Selection::All => input.replace(find, replace),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_first_1() {
        assert_eq!(
            RuleEngine::replace(
                "test message hello".to_string(),
                Selection::First,
                &"message".to_string(),
                &"yo".to_string()
            ),
            "test yo hello"
        );
    }

    #[test]
    fn test_replace_first_2() {
        assert_eq!(
            RuleEngine::replace(
                "test message message hello".to_string(),
                Selection::First,
                &"message".to_string(),
                &"yo".to_string()
            ),
            "test yo message hello"
        );
    }

    #[test]
    fn test_replace_last_1() {
        assert_eq!(
            RuleEngine::replace(
                "test message hello".to_string(),
                Selection::Last,
                &"message".to_string(),
                &"yo".to_string()
            ),
            "test yo hello"
        );
    }

    #[test]
    fn test_replace_last_2() {
        assert_eq!(
            RuleEngine::replace(
                "test message message hello".to_string(),
                Selection::Last,
                &"message".to_string(),
                &"yo".to_string()
            ),
            "test message yo hello"
        );
    }
}
