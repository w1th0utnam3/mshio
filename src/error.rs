use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};

use nom::error::{context, ErrorKind, ParseError, VerboseError};
use nom::IResult;

/// Contains error message strings used in the library
pub(crate) mod error_strings {
    pub(crate) static MSH_VERSION_UNSUPPORTED: &'static str = "MSH file of unsupported version loaded. Only the MSH file format specification of version 4.1 is supported.";
    pub(crate) static SECTION_HEADER_INVALID: &'static str = "Unexpected tokens found after file header. Expected a section according to the MSH file format specification.";
    pub(crate) static ELEMENT_UNKNOWN: &'static str =
        "An unknown element type was encountered in the MSH file.";
    pub(crate) static ELEMENT_NUM_NODES_UNKNOWN: &'static str =
        "Unimplemented: The number of nodes for an element encountered in the MSH file is unknown.";
}

/// Creates a nom ParseError with a context message
pub(crate) fn with_context<I: Clone, E: ParseError<I>, O>(
    context_msg: &'static str,
    kind: ErrorKind,
) -> impl Fn(I) -> IResult<I, O, E> {
    context(context_msg, move |i| {
        Err(nom::Err::Error(ParseError::from_error_kind(i, kind)))
    })
}

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
