#[cfg(feature = "regex_match")]
use regex::Regex;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum DirRule {
    Sort(SortDirection),
    Remove(MatchRule),
    IncludeOnly(MatchRule),
    OffsetLocalIndex(usize),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum MatchRule {
    Equals(String),
    Contains(String),
    BeginsWith(String),
    EndsWith(String),
    Not(Box<MatchRule>),
    And(Box<MatchRule>, Box<MatchRule>),
    Or(Box<MatchRule>, Box<MatchRule>),
}

#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "regex_match"), derive(PartialEq, Eq, Hash))]
pub enum FileRule {
    #[cfg(feature = "regex_match")]
    RegexReplace(Selection, Regex, String),
    Replace(Selection, String, String),
    Insert(Position, InsertionType),
    Set(String),
    SkipIf(MatchRule),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Selection {
    First,
    Last,
    All,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Position {
    Index(usize),
    After(String),
    Before(String),
    Start,
    End,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum InsertionType {
    LocalIndex,
    OverallIndex,
    Static(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl MatchRule {
    pub fn resolve(&self, input: &String) -> bool {
        match self {
            MatchRule::Equals(s) => return input == s,
            MatchRule::Contains(s) => {
                if s.len() > input.len() {
                    return false;
                }

                return input.find(s).is_some();
            }
            MatchRule::BeginsWith(s) => {
                if s.len() > input.len() {
                    return false;
                }

                return &input[0..s.len()] == s;
            }
            MatchRule::EndsWith(s) => {
                if s.len() > input.len() {
                    return false;
                }

                return &input[input.len() - s.len()..] == s;
            }
            MatchRule::And(r1, r2) => return r1.resolve(input) && r2.resolve(input),
            MatchRule::Or(r1, r2) => return r1.resolve(input) || r2.resolve(input),
            MatchRule::Not(r) => return !r.resolve(input),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod match_rule {
        use super::*;

        #[test]
        fn test_equals_1() {
            return assert!(MatchRule::Equals("test".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_equals_2() {
            return assert!(!MatchRule::Equals("tes".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_equals_3() {
            return assert!(!MatchRule::Equals("testing".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_contains_1() {
            return assert!(MatchRule::Contains("test".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_contains_2() {
            return assert!(MatchRule::Contains("tes".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_contains_3() {
            return assert!(
                !MatchRule::Contains("testing".to_string()).resolve(&"test".to_string())
            );
        }

        #[test]
        fn test_contains_4() {
            return assert!(MatchRule::Contains("".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_begins_with_1() {
            return assert!(MatchRule::BeginsWith("test".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_begins_with_2() {
            return assert!(MatchRule::BeginsWith("te".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_begins_with_3() {
            return assert!(
                !MatchRule::BeginsWith("testing".to_string()).resolve(&"test".to_string())
            );
        }

        #[test]
        fn test_begins_with_4() {
            return assert!(!MatchRule::BeginsWith("car".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_begins_with_5() {
            return assert!(!MatchRule::BeginsWith("st".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_ends_with_1() {
            return assert!(MatchRule::EndsWith("test".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_ends_with_2() {
            return assert!(!MatchRule::EndsWith("te".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_ends_with_3() {
            return assert!(
                !MatchRule::EndsWith("testing".to_string()).resolve(&"test".to_string())
            );
        }

        #[test]
        fn test_ends_with_4() {
            return assert!(!MatchRule::EndsWith("car".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_ends_with_5() {
            return assert!(MatchRule::EndsWith("st".to_string()).resolve(&"test".to_string()));
        }

        #[test]
        fn test_and_1() {
            return assert!(MatchRule::And(
                MatchRule::Equals("test".to_string()).into(),
                MatchRule::Not(MatchRule::Equals("car".to_string()).into()).into()
            )
            .resolve(&"test".to_string()));
        }

        #[test]
        fn test_or_1() {
            return assert!(MatchRule::Or(
                MatchRule::Equals("test".to_string()).into(),
                MatchRule::Not(MatchRule::Equals("car".to_string()).into()).into()
            )
            .resolve(&"test".to_string()));
        }

        #[test]
        fn test_or_2() {
            return assert!(MatchRule::Or(
                MatchRule::Equals("test".to_string()).into(),
                MatchRule::Equals("car".to_string()).into()
            )
            .resolve(&"car".to_string()));
        }

        #[test]
        fn test_not_1() {
            return assert!(MatchRule::Not(MatchRule::Equals("st".to_string()).into())
                .resolve(&"test".to_string()));
        }
    }
}
