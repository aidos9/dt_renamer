use crate::error::Error;
use crate::operations::supporting_objects::SortDirection;
use crate::operations::{DirOperation, MatchRule};
use crate::{clone_dyn, define_opexp_skeleton, File, OperationEngine};

define_opexp_skeleton!(sort_operation, direction: SortDirection);
define_opexp_skeleton!(remove_operation, rule: MatchRule);
define_opexp_skeleton!(include_only_operation, rule: MatchRule);
define_opexp_skeleton!(offset_local_index_operation, offset: usize);

impl DirOperation for SortOperation {
    fn execute(&self, _engine: &mut OperationEngine, input: &mut Vec<File>) -> Result<(), Error> {
        match self.direction {
            SortDirection::Ascending => {
                input.sort_unstable_by(|a, b| a.destination.cmp(&b.destination))
            }
            SortDirection::Descending => {
                input.sort_unstable_by(|a, b| b.destination.cmp(&a.destination))
            }
        }

        return Ok(());
    }

    clone_dyn!(DirOperation);
}

impl DirOperation for RemoveOperation {
    fn execute(&self, _engine: &mut OperationEngine, input: &mut Vec<File>) -> Result<(), Error> {
        let mut res = Vec::new();

        for f in input.drain(0..) {
            if !self.rule.resolve(
                &f.destination
                    .file_name()
                    .ok_or(Error::CannotIdentifyFileName)?
                    .to_str()
                    .ok_or(Error::CannotIdentifyFileName)?
                    .to_string(),
            ) {
                res.push(f);
            }
        }

        let _ = std::mem::replace(input, res);

        return Ok(());
    }

    clone_dyn!(DirOperation);
}

impl DirOperation for IncludeOnlyOperation {
    fn execute(&self, _engine: &mut OperationEngine, input: &mut Vec<File>) -> Result<(), Error> {
        let mut res = Vec::new();

        for f in input.drain(0..) {
            if self.rule.resolve(
                &f.destination
                    .file_name()
                    .ok_or(Error::CannotIdentifyFileName)?
                    .to_str()
                    .ok_or(Error::CannotIdentifyFileName)?
                    .to_string(),
            ) {
                res.push(f);
            }
        }

        let _ = std::mem::replace(input, res);

        return Ok(());
    }

    clone_dyn!(DirOperation);
}

impl DirOperation for OffsetLocalIndexOperation {
    fn execute(&self, engine: &mut OperationEngine, _input: &mut Vec<File>) -> Result<(), Error> {
        engine.set_local_index(self.offset);

        return Ok(());
    }

    clone_dyn!(DirOperation);
}
