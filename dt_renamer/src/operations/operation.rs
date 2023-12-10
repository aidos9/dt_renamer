use std::{fmt::Debug, path::PathBuf};

use crate::OperationEngine;
use crate::{error::Error, File};

#[macro_export]
macro_rules! define_opexp_skeleton {
    ($name:ident $(, $n:ident : $t:ty)*) => {
        paste::paste! {
            #[derive(Debug, Clone)]
            #[cfg_attr(not(feature = "regex_match"), derive(PartialEq, Eq, Hash))]
            pub struct [< $name:camel >] {
                $(
                    [< $n:snake >] : $t,
                )*
            }

            impl [< $name:camel >] {
                pub fn new($(
                    [< $n:snake >] : $t,
                )*) -> Self {
                    return Self {
                        $([< $n:snake >]),*
                    };
                }
            }
        }
    };
}

#[macro_export]
macro_rules! clone_dyn {
    ($t:ident) => {
        fn clone_dyn(&self) -> Box<dyn $t> {
            return Box::new(self.clone());
        }
    };
}

pub trait Expression: Debug {
    fn execute(&self, engine: &mut OperationEngine) -> Result<Option<String>, Error>;

    fn clone_dyn(&self) -> Box<dyn Expression>;
}

pub trait FileOperation: Debug {
    fn execute(&self, engine: &mut OperationEngine, input: &mut PathBuf) -> Result<bool, Error>;

    fn clone_dyn(&self) -> Box<dyn FileOperation>;
}

pub trait DirOperation: Debug {
    fn execute(&self, engine: &mut OperationEngine, input: &mut Vec<File>) -> Result<(), Error>;

    fn clone_dyn(&self) -> Box<dyn DirOperation>;
}

impl<T> From<T> for Box<dyn Expression>
where
    T: Expression + 'static,
{
    fn from(value: T) -> Self {
        return Box::new(value);
    }
}

impl Clone for Box<dyn Expression> {
    fn clone(&self) -> Self {
        return self.clone_dyn();
    }
}

impl<T> From<T> for Box<dyn FileOperation>
where
    T: FileOperation + 'static,
{
    fn from(value: T) -> Self {
        return Box::new(value);
    }
}

impl Clone for Box<dyn FileOperation> {
    fn clone(&self) -> Self {
        return self.clone_dyn();
    }
}

impl<T> From<T> for Box<dyn DirOperation>
where
    T: DirOperation + 'static,
{
    fn from(value: T) -> Self {
        return Box::new(value);
    }
}

impl Clone for Box<dyn DirOperation> {
    fn clone(&self) -> Self {
        return self.clone_dyn();
    }
}
