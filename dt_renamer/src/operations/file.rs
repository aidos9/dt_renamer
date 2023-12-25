use crate::error::Error;
use crate::operations::operation::Expression;
use crate::operations::{FileOperation, MatchRule};
use crate::{clone_dyn, define_opexp_skeleton};

use crate::OperationEngine;

define_opexp_skeleton!(if_operation, condition: MatchRule, then_op: Box<dyn FileOperation>, else_op: Option<Box<dyn FileOperation>>);
define_opexp_skeleton!(set_name_operation, name: Box<dyn Expression>);
define_opexp_skeleton!(set_stem_operation, stem: Box<dyn Expression>);
define_opexp_skeleton!(set_extension_operation, extension: Box<dyn Expression>);
define_opexp_skeleton!(no_op_operation, expression: Box<dyn Expression>);

impl FileOperation for NoOpOperation {
    fn execute(&self, engine: &mut OperationEngine) -> Result<bool, Error> {
        self.expression.execute(engine)?;

        return Ok(false);
    }

    clone_dyn!(FileOperation);
}

impl FileOperation for IfOperation {
    fn execute(&self, engine: &mut OperationEngine) -> Result<bool, Error> {
        let cond = self
            .condition
            .resolve(&engine.current_file().destination_path_string());

        if cond {
            return self.then_op.execute(engine);
        } else if let Some(else_branch) = &self.else_op {
            return else_branch.execute(engine);
        }

        return Ok(false);
    }

    clone_dyn!(FileOperation);
}

impl FileOperation for SetNameOperation {
    fn execute(&self, engine: &mut OperationEngine) -> Result<bool, Error> {
        let res = self.name.execute(engine)?;

        let Some(name) = res else {
            return Ok(false);
        };

        engine.current_file().destination.set_file_name(name);

        return Ok(true);
    }

    clone_dyn!(FileOperation);
}

impl FileOperation for SetStemOperation {
    fn execute(&self, engine: &mut OperationEngine) -> Result<bool, Error> {
        let res = self.stem.execute(engine)?;

        let Some(name) = res else {
            return Ok(false);
        };

        if let Some(extension) = engine.current_file().destination.extension().map(|r| {
            r.to_str()
                .ok_or(Error::CannotIdentifyFileExtension)
                .map(|s| s.to_string())
        }) {
            engine
                .current_file()
                .destination
                .set_file_name(format!("{}.{}", name, extension?));
        } else {
            engine.current_file().destination.set_file_name(name);
        }

        return Ok(true);
    }

    clone_dyn!(FileOperation);
}

impl FileOperation for SetExtensionOperation {
    fn execute(&self, engine: &mut OperationEngine) -> Result<bool, Error> {
        let res = self.extension.execute(engine)?;

        let Some(extension) = res else {
            return Ok(false);
        };

        engine.current_file().destination.set_extension(extension);

        return Ok(true);
    }

    clone_dyn!(FileOperation);
}
