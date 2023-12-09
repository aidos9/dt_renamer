use convert_case::{Case, Casing};
#[cfg(feature = "regex_match")]
use regex::Regex;

use crate::error::Error;
use crate::operations::operation::Expression;
use crate::operations::{MatchRule, OperationEngine, Selection};
use crate::{clone_dyn, define_opexp_skeleton};

define_opexp_skeleton!(insert_expr, base: Box<dyn Expression>, insertion_text: Box<dyn Expression>);
define_opexp_skeleton!(replace_expr, content: Box<dyn Expression>, selection: Selection, match_str: Box<dyn Expression>, replacement: String);
define_opexp_skeleton!(if_expr, condition: MatchRule, then_expr: Box<dyn Expression>, else_expr: Option<Box<dyn Expression>>);
#[cfg(feature = "regex_match")]
define_opexp_skeleton!(regex_match_expr, regex: Regex, input: Box<dyn Expression>);
define_opexp_skeleton!(convert_case_expr, case: Case, input: Box<dyn Expression>);
define_opexp_skeleton!(to_upper_case_expr, input: Box<dyn Expression>);
define_opexp_skeleton!(to_lower_case_expr, input: Box<dyn Expression>);
define_opexp_skeleton!(variable_expr, var: String);
define_opexp_skeleton!(assign_variable_expr, var: String, value: Box<dyn Expression>);
define_opexp_skeleton!(left_expr, input: Box<dyn Expression>, match_str: Box<dyn Expression>, inclusive: bool);
define_opexp_skeleton!(right_expr, input: Box<dyn Expression>, match_str: Box<dyn Expression>, inclusive: bool);
define_opexp_skeleton!(combine_expr, lhs: Box<dyn Expression>, rhs: Box<dyn Expression>);
define_opexp_skeleton!(constant_expr, value: String);
define_opexp_skeleton!(file_name_expr);
define_opexp_skeleton!(file_extension_expr);

impl Expression for IfExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        let cond = self
            .condition
            .resolve(&FileNameExpr::new().execute(engine)?.unwrap());

        if cond {
            return self.then_expr.execute(engine);
        } else if let Some(else_branch) = &self.else_expr {
            return else_branch.execute(engine);
        }

        return Ok(None);
    }

    clone_dyn!(Expression);
}

#[cfg(feature = "regex_match")]
impl Expression for RegexMatchExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        return Ok(self
            .input
            .execute(engine)?
            .and_then(|input| self.regex.find(&input).map(|m| m.as_str().to_string())));
    }

    clone_dyn!(Expression);
}

impl Expression for ConvertCaseExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        return Ok(self.input.execute(engine)?.map(|v| v.to_case(self.case)));
    }

    clone_dyn!(Expression);
}

impl Expression for ToUpperCaseExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        return Ok(self.input.execute(engine)?.map(|v| v.to_uppercase()));
    }

    clone_dyn!(Expression);
}

impl Expression for ToLowerCaseExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        return Ok(self.input.execute(engine)?.map(|v| v.to_lowercase()));
    }

    clone_dyn!(Expression);
}

impl Expression for VariableExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        return engine
            .get_variable(&self.var)
            .map(|v| Some(v))
            .ok_or(Error::VariableNotDefined(self.var.clone()));
    }

    clone_dyn!(Expression);
}

impl Expression for AssignVariableExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        if let Some(value) = self.value.execute(engine)? {
            engine.set_variable(self.var.clone(), value.clone());

            return Ok(Some(value));
        } else {
            return Ok(None);
        }
    }

    clone_dyn!(Expression);
}

impl Expression for LeftExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        let mut input = match self.input.execute(engine)? {
            Some(i) => i,
            None => return Ok(None),
        };

        let match_str = match self.match_str.execute(engine)? {
            Some(i) => i,
            None => return Ok(None),
        };

        if let Some(mut slice) = input.find(&match_str) {
            if self.inclusive {
                slice += match_str.len();
            }

            input = input[..slice].to_string()
        }

        return Ok(Some(input));
    }

    clone_dyn!(Expression);
}

impl Expression for RightExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        let mut input = match self.input.execute(engine)? {
            Some(i) => i,
            None => return Ok(None),
        };

        let match_str = match self.match_str.execute(engine)? {
            Some(i) => i,
            None => return Ok(None),
        };

        if let Some(mut slice) = input.find(&match_str) {
            if !self.inclusive {
                slice += match_str.len();
            }

            input = input[slice..].to_string()
        }

        return Ok(Some(input));
    }

    clone_dyn!(Expression);
}

impl Expression for CombineExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        let Some(mut lhs) = self.lhs.execute(engine)? else {
            return self.rhs.execute(engine);
        };

        let Some(rhs) = self.rhs.execute(engine)? else {
            return Ok(Some(lhs));
        };

        lhs.push_str(&rhs);

        return Ok(Some(lhs));
    }

    clone_dyn!(Expression);
}

impl Expression for ConstantExpr {
    fn execute(&self, _engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        return Ok(Some(self.value.clone()));
    }

    clone_dyn!(Expression);
}

impl From<String> for ConstantExpr {
    fn from(value: String) -> Self {
        return Self::new(value);
    }
}

impl Expression for FileNameExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        return Ok(engine
            .current_file()
            .destination
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string()));
    }

    clone_dyn!(Expression);
}

impl Expression for FileExtensionExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        return Ok(engine
            .current_file()
            .destination
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string()));
    }

    clone_dyn!(Expression);
}

impl Expression for ReplaceExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        let Some(input) = self.content.execute(engine)? else {
            return Ok(None);
        };

        let Some(matches) = self.match_str.execute(engine)? else {
            return Ok(Some(input));
        };

        return match self.selection {
            Selection::First => {
                // Could be better optimized

                if let Some(slice) = input.find(&matches) {
                    return Ok(Some(
                        [
                            &input[0..slice],
                            self.replacement.as_str(),
                            &input[slice + matches.len()..],
                        ]
                        .join(""),
                    ));
                } else {
                    return Ok(Some(input));
                }
            }
            Selection::Last => {
                // Could be better optimized
                if let Some(slice) = input.rfind(&matches) {
                    return Ok(Some(
                        [
                            &input[0..slice],
                            self.replacement.as_str(),
                            &input[slice + matches.len()..],
                        ]
                        .join(""),
                    ));
                } else {
                    return Ok(Some(input));
                }
            }
            Selection::All => Ok(Some(input.replace(&matches, &self.replacement))),
        };
    }

    clone_dyn!(Expression);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_first_1() {
        assert_eq!(
            ReplaceExpr::new(
                ConstantExpr::from("test message hello".to_string()).into(),
                Selection::First,
                ConstantExpr::from("message".to_string()).into(),
                "yo".to_string()
            )
            .execute(&mut OperationEngine::new(Vec::new(), Vec::new()))
            .unwrap()
            .unwrap(),
            "test yo hello"
        );
    }

    #[test]
    fn test_replace_first_2() {
        assert_eq!(
            ReplaceExpr::new(
                ConstantExpr::from("test message message hello".to_string()).into(),
                Selection::First,
                ConstantExpr::from("message".to_string()).into(),
                "yo".to_string()
            )
            .execute(&mut OperationEngine::new(Vec::new(), Vec::new()))
            .unwrap()
            .unwrap(),
            "test yo message hello"
        );
    }

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
