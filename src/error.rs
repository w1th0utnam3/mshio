use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};

use nom::error::{context, ErrorKind, ParseError, VerboseError, VerboseErrorKind};
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

/// Creates a nom ParseError with a context message without invoking another parser
pub(crate) fn create_error<I: Clone, E: ParseError<I>, O>(
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
        match &self.details {
            nom::Err::Error(ve) | nom::Err::Failure(ve) => {
                if ve.errors.len() > 2 {
                    write!(f, "During parsing...\n")?;
                    for (_, ek) in ve.errors[2..].iter().rev() {
                        if let VerboseErrorKind::Context(c) = ek {
                            write!(f, "\tin {},\n", c)?;
                        }
                    }
                    write!(f, "an error occured: ")?;
                    if let VerboseErrorKind::Context(c) = ve.errors[1].1 {
                        write!(f, "{}", c)?;
                    } else {
                        write!(f, "Unknown error ({:?}).", ve.errors[1].1)?;
                    }
                    Ok(())
                } else if ve.errors.len() == 2 {
                    write!(f, "During parsing an error occured: ")?;
                    if let VerboseErrorKind::Context(c) = ve.errors[1].1 {
                        write!(f, "{}", c)
                    } else {
                        write!(f, "Unknown error ({:?})", ve.errors[1].1)
                    }
                } else {
                    write!(f, "Unknown error")
                }
            }
            _ => write!(f, "Unknown error"),
        }
    }
}

impl<I: Debug> Error for MshParserError<I> {}

/// Convert a nom VerboseError to MshParserError
impl<I: Debug> From<nom::Err<VerboseError<I>>> for MshParserError<I> {
    fn from(error: nom::Err<VerboseError<I>>) -> Self {
        MshParserError { details: error }
    }
}
