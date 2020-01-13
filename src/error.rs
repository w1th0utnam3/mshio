use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};

use nom::error::VerboseError;

/// Error type returned by the MSH parser if parsing fails without panic
pub struct MshParserError<I> {
    /// The internal error returned by nom
    pub details: nom::Err<VerboseError<I>>,
}

impl<I: Debug> Debug for MshParserError<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MshParserError({:?})", self.details)
    }
}

impl<I: Debug> Display for MshParserError<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MshParserError({:?})", self.details)
    }
}

impl<I: Debug> Error for MshParserError<I> {}

impl<I: Debug> From<nom::Err<VerboseError<I>>> for MshParserError<I> {
    fn from(error: nom::Err<VerboseError<I>>) -> Self {
        MshParserError { details: error }
    }
}