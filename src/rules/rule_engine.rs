use crate::{
    rename_engine::{Dir, File},
    rules::rule::{DirRule, FileRule, Selection, SortDirection},
};

#[derive(Debug, Default)]
pub struct RuleEngine {
    global_index: usize,
    local_index: usize,
}

impl RuleEngine {
    pub fn new() -> Self {
        return Self {
            global_index: 0,
            local_index: 0,
        };
    }

    pub fn process_dir(&mut self, dir: &mut Dir) {
        self.local_index = 0;

        // for f in dir {

        // }
    }

    pub fn process_file(&mut self, file: File) {
        self.local_index = 0;
        self.run_file(file);
    }

    fn run_file(&mut self, file: File) {
        // ...
        self.global_index += 1;
        self.local_index += 1;
    }

    fn execute_dir_rule(&self, rule: &DirRule, mut input: Vec<String>) -> Vec<String> {
        match rule {
            DirRule::Sort(d) => {
                Self::sort(*d, &mut input);

                return input;
            }
            DirRule::RemoveDuplicates => {
                use itertools::Itertools;

                return input.into_iter().unique().collect();
            }
            DirRule::Remove(_) => todo!(),
            DirRule::IncludeOnly(_) => todo!(),
        }
    }

    fn sort(direction: SortDirection, input: &mut Vec<String>) {
        match direction {
            SortDirection::Ascending => input.sort(),
            SortDirection::Descending => input.sort_by(|a, b| b.cmp(a)),
        }
    }

    fn execute_file_rule(&self, rule: &FileRule, input: String) -> String {
        return match rule {
            FileRule::Replace(selection, find, replace) => {
                Self::replace(input, *selection, find, replace)
            }
            FileRule::Insert(_, _) => todo!(),
            FileRule::Set(s) => s.clone(),
        };
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
