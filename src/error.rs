use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};

use nom::error::{context as nom_context, ErrorKind, ParseError, VerboseError, VerboseErrorKind};
use nom::IResult;

/// Contains error message strings used in the library
pub(crate) mod error_strings {
    pub(crate) static MSH_VERSION_UNSUPPORTED: &'static str =
        "MSH file of unsupported version loaded. Only the MSH file format specification of version 4.1 is supported.";
    pub(crate) static SECTION_HEADER_INVALID: &'static str =
        "Unexpected tokens found after file header. Expected a section according to the MSH file format specification.";
    pub(crate) static ELEMENT_UNKNOWN: &'static str =
        "An unknown element type was encountered in the MSH file.";
    pub(crate) static ELEMENT_NUM_NODES_UNKNOWN: &'static str =
        "Unimplemented: The number of nodes for an element encountered in the MSH file does not belong to a known element type.";
    pub(crate) static UINT_PARSING_ERROR: &'static str =
        "Parsing of an unsigned integer failed. The target data type may be too small to hold a value encountered in the MSH file.";
    pub(crate) static INT_PARSING_ERROR: &'static str =
        "Parsing of an integer failed. The target data type may be too small to hold a value encountered in the MSH file.";
    pub(crate) static FLOAT_PARSING_ERROR: &'static str =
        "Parsing of a float failed. The target data type may be too small to hold a value encountered in the MSH file.";
}

/// Returns a combinator that returns a nom ParseError with a context message
pub(crate) fn create_nom_error<I: Clone, E: ParseError<I>, O>(
    context_msg: &'static str,
    kind: ErrorKind,
) -> impl Fn(I) -> IResult<I, O, E> {
    nom_context(context_msg, move |i| {
        Err(nom::Err::Error(ParseError::from_error_kind(i, kind)))
    })
}

/// Returns a combinator that returns an error of the specified kind
pub(crate) fn create_error<I: Clone, O>(
    kind: MshParserErrorKind,
) -> impl Fn(I) -> IResult<I, O, MshParserError<I>> {
    move |i: I| {
        Err(nom::Err::Error(MshParserError::from_error_kind(
            i,
            kind.clone(),
        )))
    }
}

/// Returns a combinator that appends a context message if the callable returns an error
pub(crate) fn context<I: Clone, F, O, S: AsRef<str>>(
    context: S,
    f: F,
) -> impl Fn(I) -> IResult<I, O, MshParserError<I>>
where
    F: Fn(I) -> IResult<I, O, MshParserError<I>>,
{
    let context = context.as_ref().to_string();
    move |i: I| match f(i.clone()) {
        Ok(o) => Ok(o),
        Err(nom::Err::Incomplete(i)) => Err(nom::Err::Incomplete(i)),
        Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.with_context(i, context.clone()))),
        Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.with_context(i, context.clone()))),
    }
}

#[derive(Clone, Debug)]
pub enum MshParserErrorKind {
    MshVersionUnsupported,
    SectionHeaderInvalid,
    ElementUnknown,
    ElementNumNodesUnknown,
    Context(String),
    NomVerbose(VerboseErrorKind),
}

impl From<VerboseErrorKind> for MshParserErrorKind {
    fn from(ek: VerboseErrorKind) -> Self {
        MshParserErrorKind::NomVerbose(ek)
    }
}

/// Error type returned by the MSH parser if parsing fails without panic
pub struct MshParserError<I> {
    /// The error backtrace
    pub errors: Vec<(I, MshParserErrorKind)>,
}

impl<I> MshParserError<I> {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Construct a new error with the given input and error kind
    fn from_error_kind(input: I, kind: MshParserErrorKind) -> Self {
        Self {
            errors: vec![(input, kind)],
        }
    }

    /// Append an error to the backtrace with the given input and error kind
    fn append(&mut self, input: I, kind: MshParserErrorKind) {
        self.errors.push((input, kind));
    }

    /// Append a context message to the backtrace
    fn add_context(&mut self, input: I, context_msg: String) {
        self.append(input, MshParserErrorKind::Context(context_msg));
    }

    /// Append a context message to the backtrace, consuming
    fn with_context(mut self, input: I, context_msg: String) -> Self {
        self.add_context(input, context_msg);
        self
    }
}

impl<I: Debug> Debug for MshParserError<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MshParserError({:?})", self.errors)
    }
}

impl<I: Debug> Display for MshParserError<I> {
    // TODO: Adapt this implementation for the new error type
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /*
        if self.errors.len() > 2 {
            write!(f, "During parsing...\n")?;
            for (_, ek) in self.errors[2..].iter().rev() {
                if let VerboseErrorKind::Context(c) = ek {
                    write!(f, "\tin {},\n", c)?;
                }
            }
            write!(f, "an error occured: ")?;
            if let VerboseErrorKind::Context(c) = self.errors[1].1 {
                write!(f, "{}", c)?;
            } else {
                write!(f, "Unknown error ({:?}).", self.errors[1].1)?;
            }
            Ok(())
        } else if self.errors.len() == 2 {
            write!(f, "During parsing an error occured: ")?;
            if let VerboseErrorKind::Context(c) = self.errors[1].1 {
                write!(f, "{}", c)
            } else {
                write!(f, "Unknown error ({:?})", self.errors[1].1)
            }
        } else {
            write!(f, "Unknown error")
        }
        */
        write!(f, "Unknown error")
    }
}

impl<I> ParseError<I> for MshParserError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            errors: vec![(
                input,
                MshParserErrorKind::NomVerbose(VerboseErrorKind::Nom(kind)),
            )],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other.errors.push((
            input,
            MshParserErrorKind::NomVerbose(VerboseErrorKind::Nom(kind)),
        ));
        other
    }

    fn from_char(input: I, c: char) -> Self {
        Self {
            errors: vec![(
                input,
                MshParserErrorKind::NomVerbose(VerboseErrorKind::Char(c)),
            )],
        }
    }

    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((
            input,
            MshParserErrorKind::NomVerbose(VerboseErrorKind::Context(ctx)),
        ));
        other
    }
}

impl<I: Debug> Error for MshParserError<I> {}

impl<I: Debug> From<VerboseError<I>> for MshParserError<I> {
    fn from(e: VerboseError<I>) -> Self {
        MshParserError {
            errors: e.errors.into_iter().map(|(i, ek)| (i, ek.into())).collect(),
        }
    }
}

/// Convert a nom VerboseError to MshParserError
impl<I: Debug, E: Into<MshParserError<I>>> From<nom::Err<E>> for MshParserError<I> {
    fn from(error: nom::Err<E>) -> Self {
        match error {
            nom::Err::Error(ve) | nom::Err::Failure(ve) => ve.into(),
            _ => Self::new(),
        }
    }
}
