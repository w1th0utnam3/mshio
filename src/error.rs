use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};

use nom::error::{ErrorKind, ParseError, VerboseError, VerboseErrorKind};
use nom::{HexDisplay, IResult};

// TODO: Think about a better solution than a static_context and owned_context combinator

/*
/// Returns a combinator that returns a nom ParseError with a context message
pub(crate) fn nom_error<I: Clone, E: ParseError<I>, O>(
    context_msg: &'static str,
    kind: ErrorKind,
) -> impl Fn(I) -> IResult<I, O, E> {
    nom_context(context_msg, move |i| {
        Err(nom::Err::Error(ParseError::from_error_kind(i, kind)))
    })
}
*/

/// Returns a combinator that returns an error of the specified kind
pub(crate) fn error<I, O>(
    kind: MshParserErrorKind,
) -> impl Fn(I) -> IResult<I, O, MshParserError<I>> {
    move |i: I| Err(MshParserError::from_error_kind(i, kind.clone()).into_nom_err())
}

/// Returns a combinator that appends a static context message if the callable returns an error
pub(crate) fn static_context<I: Clone, F, O>(
    context: &'static str,
    f: F,
) -> impl Fn(I) -> IResult<I, O, MshParserError<I>>
where
    F: Fn(I) -> IResult<I, O, MshParserError<I>>,
{
    move |i: I| match f(i.clone()) {
        Ok(o) => Ok(o),
        Err(nom::Err::Incomplete(i)) => Err(nom::Err::Incomplete(i)),
        Err(nom::Err::Error(e)) => Err(nom::Err::Error(
            e.append(i, MshParserErrorKind::StaticContext(context)),
        )),
        Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(
            e.append(i, MshParserErrorKind::StaticContext(context)),
        )),
    }
}

/// Returns a combinator that appends an owned context message (String) if the callable returns an error
pub(crate) fn owned_context<I: Clone, F, O, S: AsRef<str>>(
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

#[derive(Clone, Debug, thiserror::Error)]
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
    #[error("An unsigned integer value could not be parsed because it was out of range of the target data type.")]
    UnsignedIntegerOutOfRange,
    #[error(
        "An integer value could not be parsed because it was out of range of the target data type."
    )]
    IntegerOutOfRange,
    #[error("A floating point value could not be parsed because it was out of range of the target data type.")]
    FloatOutOfRange,
    #[error("{0}")]
    OwnedContext(String),
    #[error("{0}")]
    StaticContext(&'static str),
    #[error("{0:?}")]
    NomVerbose(VerboseErrorKind),
}

impl MshParserErrorKind {
    /// Returns whether the variant is an internal nom error
    pub fn is_nom_error(&self) -> bool {
        match self {
            MshParserErrorKind::NomVerbose(_) => true,
            _ => false,
        }
    }

    /// Returns a reference to the context message of this error contains one
    pub fn context(&self) -> Option<&str> {
        match self {
            MshParserErrorKind::OwnedContext(c) => Some(c.as_str()),
            MshParserErrorKind::StaticContext(c) => Some(*c),
            MshParserErrorKind::NomVerbose(VerboseErrorKind::Context(c)) => Some(*c),
            _ => None,
        }
    }
}

impl From<VerboseErrorKind> for MshParserErrorKind {
    fn from(ek: VerboseErrorKind) -> Self {
        MshParserErrorKind::NomVerbose(ek)
    }
}

/// Error type returned by the MSH parser if parsing fails without panic
pub struct MshParserError<I> {
    /// The error backtrace
    pub backtrace: Vec<(I, MshParserErrorKind)>,
}

impl<I> MshParserError<I> {
    fn new() -> Self {
        Self {
            backtrace: Vec::new(),
        }
    }

    pub(crate) fn into_nom_err(self) -> nom::Err<Self> {
        nom::Err::Error(self)
    }

    /// Construct a new error with the given input and error kind
    pub(crate) fn from_error_kind(input: I, kind: MshParserErrorKind) -> Self {
        Self {
            backtrace: vec![(input, kind)],
        }
    }

    /// Append an error to the backtrace with the given input and error kind
    pub(crate) fn append(mut self, input: I, kind: MshParserErrorKind) -> Self {
        self.backtrace.push((input, kind));
        self
    }

    /// Append a context message to the backtrace, consuming
    pub(crate) fn with_context(self, input: I, context_msg: String) -> Self {
        self.append(input, MshParserErrorKind::OwnedContext(context_msg))
    }

    /// Iterator to the first error in the backtrace that is actually a MSH error
    pub fn begin_msh_errors(&self) -> impl Iterator<Item = &(I, MshParserErrorKind)> {
        self.backtrace.iter().skip_while(|(_, e)| e.is_nom_error())
    }

    /// Returns the first error in the backtrace that is not an internal nom error
    pub fn first_msh_error(&self) -> Option<MshParserErrorKind> {
        self.begin_msh_errors().next().map(|(_, ek)| ek).cloned()
    }
}

impl<I: Clone> MshParserError<I> {
    /// Returns a backtrace of all errors, excluding the deepest internal nom errors
    pub fn trimmed_backtrace(&self) -> Vec<(I, MshParserErrorKind)> {
        self.begin_msh_errors().cloned().collect()
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
        let backtrace = self.trimmed_backtrace();
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
            backtrace: vec![(
                input,
                MshParserErrorKind::NomVerbose(VerboseErrorKind::Nom(kind)),
            )],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other.backtrace.push((
            input,
            MshParserErrorKind::NomVerbose(VerboseErrorKind::Nom(kind)),
        ));
        other
    }

    fn from_char(input: I, c: char) -> Self {
        Self {
            backtrace: vec![(
                input,
                MshParserErrorKind::NomVerbose(VerboseErrorKind::Char(c)),
            )],
        }
    }

    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.backtrace.push((
            input,
            MshParserErrorKind::NomVerbose(VerboseErrorKind::Context(ctx)),
        ));
        other
    }
}

impl<I: Debug + HexDisplay + ?Sized> Error for MshParserError<&I> {}

impl<I: Debug> From<VerboseError<I>> for MshParserError<I> {
    fn from(e: VerboseError<I>) -> Self {
        MshParserError {
            backtrace: e.errors.into_iter().map(|(i, ek)| (i, ek.into())).collect(),
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

pub(crate) trait MshParserCompatibleError<I>
where
    Self: ParseError<I>,
    nom::Err<MshParserError<I>>: From<nom::Err<Self>>,
{
}

impl<I, T> MshParserCompatibleError<I> for T
where
    T: ParseError<I>,
    nom::Err<MshParserError<I>>: From<nom::Err<T>>,
{
}
