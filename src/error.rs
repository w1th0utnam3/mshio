use std::borrow::{Borrow, Cow};
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};

use nom::error::{ErrorKind, ParseError};
use nom::{HexDisplay, IResult};

/// Returns a combinator that returns an error of the specified kind
pub(crate) fn error<I, O>(
    kind: MshParserErrorKind,
) -> impl Fn(I) -> IResult<I, O, MshParserError<I>> {
    move |i: I| Err(MshParserError::from_error_kind(i, kind.clone()).into_nom_error())
}

/// Returns a combinator that appends a context message if the callable returns an error
pub(crate) fn context<I: Clone, F, O, S: Clone + Into<Cow<'static, str>>>(
    ctx: S,
    f: F,
) -> impl Fn(I) -> IResult<I, O, MshParserError<I>>
where
    F: Fn(I) -> IResult<I, O, MshParserError<I>>,
{
    move |i: I| match f(i.clone()) {
        Ok(o) => Ok(o),
        Err(nom::Err::Incomplete(i)) => Err(nom::Err::Incomplete(i)),
        Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.with_context(i, ctx.clone()))),
        Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.with_context(i, ctx.clone()))),
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ValueType {
    #[error("unsigned integer")]
    UnsignedInt,
    #[error("integer")]
    Int,
    #[error("floating point")]
    Float,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum MshParserErrorKind {
    #[error("MSH file of unsupported version loaded. Only the MSH file format specification of version 4.1 is supported.")]
    MshVersionUnsupported,
    #[error("Unexpected tokens found after file header. Expected a section according to the MSH file format specification.")]
    SectionHeaderInvalid,
    #[error("An unknown element type was encountered in the MSH file.")]
    ElementUnknown,
    #[error("Unimplemented: The number of nodes for an element encountered in the MSH file does not belong to a known element type.")]
    ElementNumNodesUnknown,
    #[error("There are too many entities to parse them into contiguous memory on the current system (usize type too small).")]
    TooManyEntities,
    #[error("A {0} value could not be parsed because it was out of range of the target data type.")]
    ValueOutOfRange(ValueType),
    #[error("An invalid entity tag was detected.")]
    InvalidTag,
    #[error("{0}")]
    Context(Cow<'static,str>),
    #[error("{0:?}")]
    NomError(ErrorKind),
}

impl MshParserErrorKind {
    pub fn into_error<I>(self, input: I) -> MshParserError<I> {
        MshParserError::from_error_kind(input, self)
    }

    /// Returns whether the variant is an internal nom error
    pub fn is_nom_error(&self) -> bool {
        match self {
            MshParserErrorKind::NomError(_) => true,
            _ => false,
        }
    }

    /// Returns a reference to the context message of this error contains one
    pub fn context(&self) -> Option<&str> {
        match self {
            MshParserErrorKind::Context(ctx) => Some(ctx.borrow()),
            _ => None,
        }
    }
}

impl From<ErrorKind> for MshParserErrorKind {
    fn from(ek: ErrorKind) -> Self {
        MshParserErrorKind::NomError(ek)
    }
}

/// Error type returned by the MSH parser if parsing fails without panic
pub struct MshParserError<I> {
    /// The error backtrace
    pub backtrace: Vec<(I, MshParserErrorKind)>,
}

impl<I> MshParserError<I> {
    /// Creates a new empty error
    fn new() -> Self {
        Self {
            backtrace: Vec::new(),
        }
    }

    /// Construct a new error with the given input and error kind
    pub fn from_error_kind(input: I, kind: MshParserErrorKind) -> Self {
        Self {
            backtrace: vec![(input, kind)],
        }
    }

    /// Wraps the error into a (recoverable) nom::Err::Error
    pub fn into_nom_error(self) -> nom::Err<Self> {
        nom::Err::Error(self)
    }

    /// Wraps the error into a (unrecoverable) nom::Err::Failure
    pub fn into_nom_failure(self) -> nom::Err<Self> {
        nom::Err::Failure(self)
    }

    /// Append an error to the backtrace with the given input and error kind
    pub fn with_append(mut self, input: I, kind: MshParserErrorKind) -> Self {
        self.backtrace.push((input, kind));
        self
    }

    /// Append a context message to the backtrace
    pub fn with_context<S: Into<Cow<'static, str>>>(self, input: I, ctx: S) -> Self {
        self.with_append(input, MshParserErrorKind::Context(ctx.into()))
    }

    /// Iterator to the first error in the backtrace that is actually a MSH error
    pub fn begin_msh_errors(&self) -> impl Iterator<Item = &(I, MshParserErrorKind)> {
        self.backtrace.iter().skip_while(|(_, e)| e.is_nom_error())
    }

    /// Iterator to the first error in the backtrace that is actually a MSH error
    pub fn filter_msh_errors(&self) -> impl Iterator<Item = &(I, MshParserErrorKind)> {
        self.backtrace.iter().filter(|(_, e)| !e.is_nom_error())
    }

    /// Returns the kind of the first error in the backtrace that is not an internal nom error
    pub fn first_msh_error(&self) -> Option<MshParserErrorKind> {
        self.begin_msh_errors().next().map(|(_, ek)| ek).cloned()
    }
}

impl<I: Clone> MshParserError<I> {
    /// Returns a backtrace of all errors, excluding the first internal nom errors
    pub fn filtered_backtrace(&self) -> Vec<(I, MshParserErrorKind)> {
        self.filter_msh_errors().cloned().collect()
    }
}

impl<I: Debug> Debug for MshParserError<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MshParserError({:?})", self.backtrace)
    }
}

impl<I: Debug + HexDisplay + ?Sized> Display for MshParserError<&I> {
    // TODO: Move this to a "report" method of the error.
    // TODO: Instead, make Display implementation more simple.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Remove all internal nom errors
        let backtrace = self.filtered_backtrace();
        if backtrace.len() > 1 {
            write!(f, "During parsing...\n")?;
            for (_, ek) in backtrace[1..].iter().rev() {
                if let Some(c) = ek.context() {
                    write!(f, "\tin {},\n", c)?;
                } else {
                    write!(f, "\tin {},\n", ek)?;
                }
            }
            write!(f, "an error occurred: ")?;
            write!(f, "{}\n", backtrace[0].1)?;
            write!(
                f,
                "Hex dump of the file at the error location:\n{}",
                // TODO: Limit to a reasonable number of bytes
                backtrace[0].0.to_hex(16)
            )?;
            Ok(())
        } else if backtrace.len() == 1 {
            write!(f, "An error occurred during: ")?;
            write!(f, "{}", backtrace[0].1)?;
            Ok(())
        } else {
            write!(f, "Unknown error occurred\n")
        }
    }
}

impl<I> ParseError<I> for MshParserError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            backtrace: vec![(input, MshParserErrorKind::NomError(kind))],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other
            .backtrace
            .push((input, MshParserErrorKind::NomError(kind)));
        other
    }

    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other
            .backtrace
            .push((input, MshParserErrorKind::Context(Cow::Borrowed(ctx))));
        other
    }
}

impl<I: Debug + HexDisplay + ?Sized> Error for MshParserError<&I> {}

/// Convert a nom::Err to MshParserError
impl<I: Debug, E: Into<MshParserError<I>>> From<nom::Err<E>> for MshParserError<I> {
    fn from(error: nom::Err<E>) -> Self {
        match error {
            nom::Err::Error(ve) | nom::Err::Failure(ve) => ve.into(),
            _ => Self::new(),
        }
    }
}
