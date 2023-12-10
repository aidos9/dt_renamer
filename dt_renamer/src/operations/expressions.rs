use convert_case::{Case, Casing};
#[cfg(feature = "regex_match")]
use regex::Regex;

use crate::error::Error;
use crate::operations::supporting_objects::{Position, Selection};
use crate::operations::{Expression, MatchRule};
use crate::OperationEngine;
use crate::{clone_dyn, define_opexp_skeleton};

#[cfg(feature = "regex_match")]
define_opexp_skeleton!(regex_match_expr, regex: Regex, input: Box<dyn Expression>);

define_opexp_skeleton!(insert_expr, position: Position, base: Box<dyn Expression>, insertion_text: Box<dyn Expression>);
define_opexp_skeleton!(replace_expr, content: Box<dyn Expression>, selection: Selection, match_str: Box<dyn Expression>, replacement: Box<dyn Expression>);
define_opexp_skeleton!(if_expr, condition: MatchRule, then_expr: Box<dyn Expression>, else_expr: Option<Box<dyn Expression>>);
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

macro_rules! unwrap_res_op {
    ($e:expr) => {{
        let Some(r) = $e? else {
            return Ok(None);
        };

        r
    }};
}

impl Expression for InsertExpr {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error> {
        let mut base = unwrap_res_op!(self.base.execute(engine));
        let insertion_text = unwrap_res_op!(self.insertion_text.execute(engine));

        return Ok(Some(match &self.position {
            Position::Index(i) => {
                base.insert_str(*i.min(&base.len()), &insertion_text);

                base
            }
            Position::After(f) => {
                let Some(insert_pos) = base.find(f) else {
                    return Ok(None);
                };

                base.insert_str(insert_pos + f.len(), &insertion_text);

                base
            }
            #[cfg(feature = "regex_match")]
            Position::AfterRegex(r) => {
                let Some(insert_pos) = r.find(&base) else {
                    return Ok(None);
                };

                base.insert_str(insert_pos.end(), &insertion_text);

                base
            }
            Position::Before(f) => {
                let Some(insert_pos) = base.find(f) else {
                    return Ok(None);
                };

                base.insert_str(insert_pos, &insertion_text);

                base
            }
            #[cfg(feature = "regex_match")]
            Position::BeforeRegex(r) => {
                let Some(insert_pos) = r.find(&base) else {
                    return Ok(None);
                };

                base.insert_str(insert_pos.start(), &insertion_text);

                base
            }
            Position::Start => {
                format!("{}{}", insertion_text, base)
            }
            Position::End => format!("{}{}", base, insertion_text),
        }));
    }

    clone_dyn!(Expression);
}

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
        let value = unwrap_res_op!(self.value.execute(engine));

        engine.set_variable(self.var.clone(), value.clone());

        return Ok(Some(value));
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

impl<'a> From<&'a str> for ConstantExpr {
    fn from(value: &'a str) -> Self {
        return Self::new(value.to_string());
    }
}

impl From<String> for ConstantExpr {
    fn from(value: String) -> Self {
        return Self::new(value);
    }
}

impl From<String> for Box<dyn Expression> {
    fn from(value: String) -> Self {
        return ConstantExpr::from(value).into();
    }
}

impl<'a> From<&'a str> for Box<dyn Expression> {
    fn from(value: &'a str) -> Self {
        return ConstantExpr::from(value).into();
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
        let input = unwrap_res_op!(self.content.execute(engine));
        let matches = unwrap_res_op!(self.match_str.execute(engine));
        let replacement = unwrap_res_op!(self.replacement.execute(engine));

        return match self.selection {
            Selection::First => {
                // Could be better optimized

                if let Some(slice) = input.find(&matches) {
                    return Ok(Some(
                        [
                            &input[0..slice],
                            replacement.as_str(),
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
                            replacement.as_str(),
                            &input[slice + matches.len()..],
                        ]
                        .join(""),
                    ));
                } else {
                    return Ok(Some(input));
                }
            }
            Selection::All => Ok(Some(input.replace(&matches, &replacement))),
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
                "test message hello".into(),
                Selection::First,
                "message".into(),
                "yo".into()
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
                "test message message hello".into(),
                Selection::First,
                "message".into(),
                "yo".into()
            )
            .execute(&mut OperationEngine::new(Vec::new(), Vec::new()))
            .unwrap()
            .unwrap(),
            "test yo message hello"
        );
    }

    #[test]
    fn test_replace_last_1() {
        assert_eq!(
            ReplaceExpr::new(
                "test message hello".into(),
                Selection::Last,
                "message".into(),
                "yo".into()
            )
            .execute(&mut OperationEngine::new(Vec::new(), Vec::new()))
            .unwrap()
            .unwrap(),
            "test yo hello"
        );
    }

    #[test]
    fn test_replace_last_2() {
        assert_eq!(
            ReplaceExpr::new(
                "test message message hello".into(),
                Selection::Last,
                "message".into(),
                "yo".into()
            )
            .execute(&mut OperationEngine::new(Vec::new(), Vec::new()))
            .unwrap()
            .unwrap(),
            "test message yo hello"
        );
    }

    #[test]
    fn test_left_1() {
        assert_eq!(
            LeftExpr::new("test message message hello".into(), "message".into(), true)
                .execute(&mut OperationEngine::new(Vec::new(), Vec::new()))
                .unwrap()
                .unwrap(),
            "test message"
        );
    }

    #[test]
    fn test_left_2() {
        assert_eq!(
            LeftExpr::new("test message message hello".into(), "message".into(), false)
                .execute(&mut OperationEngine::new(Vec::new(), Vec::new()))
                .unwrap()
                .unwrap(),
            "test "
        );
    }

    #[test]
    fn test_right_1() {
        assert_eq!(
            RightExpr::new("test message message hello".into(), "message".into(), true)
                .execute(&mut OperationEngine::new(Vec::new(), Vec::new()))
                .unwrap()
                .unwrap(),
            "message message hello"
        );
    }

    #[test]
    fn test_right_2() {
        assert_eq!(
            RightExpr::new("test message message hello".into(), "message".into(), false)
                .execute(&mut OperationEngine::new(Vec::new(), Vec::new()))
                .unwrap()
                .unwrap(),
            " message hello"
        );
    }

    #[cfg(feature = "regex_match")]
    mod regex {
        use super::*;

        #[test]
        fn test_insert_before_1() {
            let r = Regex::new("test").unwrap();

            assert_eq!(
                InsertExpr::new(
                    Position::BeforeRegex(r),
                    "test message hello".into(),
                    "yo ".into()
                )
                .execute(&mut OperationEngine::new(Vec::new(), Vec::new()))
                .unwrap()
                .unwrap(),
                "yo test message hello"
            );
        }

        #[test]
        fn test_insert_after_1() {
            let r = Regex::new("test ").unwrap();

            assert_eq!(
                InsertExpr::new(
                    Position::AfterRegex(r),
                    "test message hello".into(),
                    "yo ".into()
                )
                .execute(&mut OperationEngine::new(Vec::new(), Vec::new()))
                .unwrap()
                .unwrap(),
                "test yo message hello"
            );
        }

        #[test]
        fn test_match_1() {
            let r = Regex::new(r"\[.*\]").unwrap();

            assert_eq!(
                RegexMatchExpr::new(r, "Cow boy [boss] test".into())
                    .execute(&mut OperationEngine::new(Vec::new(), Vec::new()))
                    .unwrap()
                    .unwrap(),
                "[boss]"
            );
        }
    }
}
