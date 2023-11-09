mod file_rule;
mod match_rule;
mod rule_engine;

pub use match_rule::*;
pub use rule_engine::RuleEngine;

// pub trait RuleExecutor
// where
//     Self: Sized,
// {
//     fn local_index(&self) -> usize;

//     fn set_local_index(&mut self, index: usize);

//     fn global_index(&self) -> usize;

//     fn execute_file_rule<R: FileRule>(&self, rule: R, input: &mut PathBuf) -> Result<bool, Error> {
//         return rule.execute(self, input);
//     }

//     fn execute_dir_rule<R: DirRule>(&mut self, rule: &R, input: &mut Vec<File>) {
//         return rule.execute(self, input);
//     }
// }

// pub trait FileRule {
//     fn execute<R: RuleExecutor>(&self, engine: &R, input: &mut PathBuf) -> Result<bool, Error>;
// }

// pub trait DirRule {
//     fn execute<R: RuleExecutor>(&self, engine: &mut R, input: &mut Vec<File>);
// }
