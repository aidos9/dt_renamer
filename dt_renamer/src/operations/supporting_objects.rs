#[cfg(feature = "regex_match")]
use regex::Regex;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Selection {
    First,
    Last,
    All,
}

#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "regex_match"), derive(PartialEq, Eq, Hash))]
pub enum Position {
    Index(usize),
    After(String),
    #[cfg(feature = "regex_match")]
    AfterRegex(Regex),
    Before(String),
    #[cfg(feature = "regex_match")]
    BeforeRegex(Regex),
    Start,
    End,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum InsertionType {
    LocalIndex,
    OverallIndex,
    Static(String),
    Variable(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Direction {
    LeftExclusive,
    LeftInclusive,
    RightExclusive,
    RightInclusive,
}

impl From<&str> for InsertionType {
    fn from(value: &str) -> Self {
        return value.to_string().into();
    }
}

impl From<String> for InsertionType {
    fn from(value: String) -> Self {
        return Self::Static(value);
    }
}
