use std::path::PathBuf;

use crate::error::Error;
use crate::operations::operation::Expression;
use crate::operations::{FileOperation, MatchRule};
use crate::{clone_dyn, define_opexp_skeleton};

use super::OperationEngine;

define_opexp_skeleton!(if_operation, condition: MatchRule, then_op: Box<dyn FileOperation>, else_op: Option<Box<dyn FileOperation>>);
define_opexp_skeleton!(set_name_operation, name: Box<dyn Expression>);
define_opexp_skeleton!(set_extension_operation, extension: Box<dyn Expression>);
define_opexp_skeleton!(array_operation, operations: Vec<Box<dyn FileOperation>>);

impl FileOperation for IfOperation {
    fn execute(&self, engine: &mut OperationEngine, input: &mut PathBuf) -> Result<bool, Error> {
        let cond = self.condition.resolve(&input.display().to_string());

        if cond {
            return self.then_op.execute(engine, input);
        } else if let Some(else_branch) = &self.else_op {
            return else_branch.execute(engine, input);
        }

        return Ok(false);
    }

    clone_dyn!(FileOperation);
}

impl FileOperation for SetNameOperation {
    fn execute(&self, engine: &mut OperationEngine, input: &mut PathBuf) -> Result<bool, Error> {
        let res = self.name.execute(engine)?;

        let Some(name) = res else {
            return Ok(false);
        };

        input.set_file_name(format!(
            "{}{}",
            name,
            input
                .extension()
                .ok_or(Error::CannotIdentifyFileExtension)?
                .to_str()
                .ok_or(Error::CannotIdentifyFileExtension)?
        ));

        return Ok(true);
    }

    clone_dyn!(FileOperation);
}

impl FileOperation for SetExtensionOperation {
    fn execute(&self, engine: &mut OperationEngine, input: &mut PathBuf) -> Result<bool, Error> {
        let res = self.extension.execute(engine)?;

        let Some(extension) = res else {
            return Ok(false);
        };

        input.set_extension(extension);

        return Ok(true);
    }

    clone_dyn!(FileOperation);
}

impl FileOperation for ArrayOperation {
    fn execute(&self, engine: &mut OperationEngine, input: &mut PathBuf) -> Result<bool, Error> {
        let mut res = false;

        for op in &self.operations {
            res = res || op.execute(engine, input)?;
        }

        return Ok(res);
    }

    clone_dyn!(FileOperation);
}
