use crate::{error::Error, RenameResult, RenameTree};

#[derive(Debug, Default)]
pub struct Script {
    trees: Vec<RenameTree>,
}

impl Script {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn with_tree(mut self, tree: RenameTree) -> Self {
        self.push(tree);

        return self;
    }

    pub fn push(&mut self, tree: RenameTree) {
        self.trees.push(tree);
    }

    pub fn run(self) -> Result<Vec<RenameResult>, Error> {
        let mut output = Vec::new();

        for res in self.trees.into_iter().map(|m| m.run()) {
            output.append(&mut res?);
        }

        return Ok(output);
    }

    pub fn dry_run(self) -> Result<Vec<RenameResult>, Error> {
        let mut output = Vec::new();

        for res in self.trees.into_iter().map(|m| m.dry_run()) {
            output.append(&mut res?);
        }

        return Ok(output);
    }
}

impl From<Vec<RenameTree>> for Script {
    fn from(value: Vec<RenameTree>) -> Self {
        return Self { trees: value };
    }
}

impl<const N: usize> From<[RenameTree; N]> for Script {
    fn from(value: [RenameTree; N]) -> Self {
        return Self {
            trees: value.into(),
        };
    }
}

impl From<&[RenameTree]> for Script {
    fn from(value: &[RenameTree]) -> Self {
        return Self {
            trees: value.to_vec(),
        };
    }
}

impl From<RenameTree> for Script {
    fn from(value: RenameTree) -> Self {
        return Self { trees: vec![value] };
    }
}
