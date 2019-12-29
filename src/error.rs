use crate::primitive::*;
use std::fmt::{Display, Formatter, Debug};

#[derive(Debug, PartialEq)]
pub enum Error {
    Missing {
        branch: String,
        id: PrimitiveId,
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        // TODO: Don't use the debug implementation
        Debug::fmt(self, f)
    }
}

impl std::error::Error for Error {

}