use std::borrow::{Borrow, Cow};
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};

use nom::error::{ErrorKind, ParseError};
use nom::{HexDisplay, IResult};

pub(crate) fn make_error<I>(input: I, kind: MshParserErrorKind) -> nom::Err<MshParserError<I>> {
    MshParserError::from_error_kind(input, kind.clone()).into_nom_error()
}

/// Returns a combinator that always returns an error of the specified kind
pub(crate) fn always_error<I, O>(
    kind: MshParserErrorKind,
) -> impl Fn(I) -> IResult<I, O, MshParserError<I>> {
    move |i: I| Err(make_error(i, kind.clone()))
}

/// Returns a combinator that appends an if the callable returns an error
pub(crate) fn error<I: Clone, F, O>(
    kind: MshParserErrorKind,
    f: F,
) -> impl Fn(I) -> IResult<I, O, MshParserError<I>>
where
    F: Fn(I) -> IResult<I, O, MshParserError<I>>,
{
    move |i: I| f(i.clone()).with_error(i, kind.clone())
}

/// Returns a combinator that appends a context message if the callable returns an error
pub(crate) fn context<I: Clone, F, O>(
    ctx: &'static str,
    f: F,
) -> impl Fn(I) -> IResult<I, O, MshParserError<I>>
where
    F: Fn(I) -> IResult<I, O, MshParserError<I>>,
{
    move |i: I| f(i.clone()).with_context(i, ctx)
}

/// Returns a combinator that appends a context message obtained from the callable if the callable returns an error
pub(crate) fn context_from<I: Clone, C, F, O, S: Clone + Into<Cow<'static, str>>>(
    ctx: C,
    f: F,
) -> impl Fn(I) -> IResult<I, O, MshParserError<I>>
where
    C: Fn() -> S,
    F: Fn(I) -> IResult<I, O, MshParserError<I>>,
{
    move |i: I| f(i.clone()).with_context(i, ctx())
}

/// Enum for the categories of value types that MSH files contain
#[derive(Copy, Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ValueType {
    /// The unsigned integer or size_t type
    #[error("unsigned integer")]
    UnsignedInt,
    /// The integer or int type
    #[error("integer")]
    Int,
    /// The floating point or double type
    #[error("floating point")]
    Float,
}

/// Enum of all error kinds that may be part of a [`MshParserError`](struct.MshParserError.html) backtrace
#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum MshParserErrorKind {
    /// Error indicating that the MSH file header specifies an unsupported file format revision (only 4.1 is supported)
    #[error("MSH file of unsupported format version loaded. Only the MSH file format specification of revision 4.1 is supported.")]
    UnsupportedMshVersion,
    /// Error indicating that the MSH file header specifies a size for a value type which is not supported by this crate
    #[error("There is no parser available to parse binary {0} values with a size of {1}.")]
    UnsupportedTypeSize(ValueType, usize),
    /// Error indicating that the MSH file header does not conform to the file format specification
    #[error("The MSH file header is not valid.")]
    InvalidFileHeader,
    /// Error indicating that the MSH body contains tokens that do not form a valid section
    #[error("Unexpected tokens found after file header. Expected a section according to the MSH file format specification.")]
    InvalidSectionHeader,
    /// Error indicating that an element entity contains an [`ElementType`](../mshfile/enum.ElementType.html) that is not supported by this crate
    #[error("An unknown element type was encountered in the MSH file.")]
    UnknownElement,
    /// Error indicating that a section contains too many entities (e.g. nodes, elements, groups), i.e. they do not fit into a `Vec` because `usize::MAX` is too small
    #[error("There are too many entities to parse them into contiguous memory on the current system (usize type too small).")]
    TooManyEntities,
    /// Error indicating that a value was encountered in the MSH file that is out of range of the target value type
    #[error("A {0} value could not be parsed because it was out of range of the target data type.")]
    ValueOutOfRange(ValueType),
    /// Error indicating that an entity tag with an invalid value was encountered, e.g. the Gmsh internally reserved value of 0 or a `max_tag` that is smaller than a `min_tag`
    #[error("An invalid entity tag value was encountered, e.g. the internally reserved value of 0, or a max_tag that is smaller than a min_tag")]
    InvalidTag,
    /// Error indicating that an invalid parameter value was encountered
    #[error("An invalid parameter value was encountered.")]
    InvalidParameter,
    /// Error indicating that a single element definition could not be parsed, e.g. it did not provide the correct number of node indices corresponding to the [`ElementType`](../mshfile/enum.ElementType.html) of the element block
    #[error("An invalid element definition was encountered.")]
    InvalidElementDefinition,
    /// Error indicating that a single node definition could not be parsed, e.g. it does not contain three parsable floating point values for its coordinates
    #[error("An invalid node definition was encountered.")]
    InvalidNodeDefinition,
    /// Error indicating that the MSH file contains a MSH format feature that is not yet supported by this crate
    #[error("An unimplemented feature was detected.")]
    Unimplemented,
    /// Additional context information for pretty printing the backtrace for a user
    #[error("{0}")]
    Context(Cow<'static,str>),
    /// Internal nom parser error, such as an error when parsing a single digit
    #[error("{0:?}")]
    NomError(ErrorKind),
}

impl MshParserErrorKind {
    pub(crate) fn into_error<I>(self, input: I) -> MshParserError<I> {
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

/// Error type returned by the crate when parsing fails
pub struct MshParserError<I> {
    /// Error backtrace that contains per level a reference into the input where the error ocurred and the corresponding error kind
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
    pub(crate) fn from_error_kind(input: I, kind: MshParserErrorKind) -> Self {
        Self {
            backtrace: vec![(input, kind)],
        }
    }

    /// Wraps the error into a (recoverable) nom::Err::Error
    pub(crate) fn into_nom_error(self) -> nom::Err<Self> {
        nom::Err::Error(self)
    }

    /// Wraps the error into a (unrecoverable) nom::Err::Failure
    pub(crate) fn into_nom_failure(self) -> nom::Err<Self> {
        nom::Err::Failure(self)
    }

    /// Append an error to the backtrace with the given input and error kind
    pub(crate) fn with_append(mut self, input: I, kind: MshParserErrorKind) -> Self {
        self.backtrace.push((input, kind));
        self
    }

    /// Append a context message to the backtrace
    pub(crate) fn with_context<S: Into<Cow<'static, str>>>(self, input: I, ctx: S) -> Self {
        self.with_append(input, MshParserErrorKind::Context(ctx.into()))
    }

    /// Iterator that skips all errors in the beginning of the backtrace that are not actual MSH format errors (i.e. internal nom parser errors)
    pub fn begin_msh_errors(&self) -> impl Iterator<Item = &(I, MshParserErrorKind)> {
        self.backtrace.iter().skip_while(|(_, e)| e.is_nom_error())
    }

    /// Iterator over all errors in the backtrace that are actual MSH format errors (i.e. filters out all internal nom parser errors)
    pub fn filter_msh_errors(&self) -> impl Iterator<Item = &(I, MshParserErrorKind)> {
        self.backtrace.iter().filter(|(_, e)| !e.is_nom_error())
    }

    /// Returns the kind of the first error in the backtrace that is an actual MSH format error kind (i.e. skips internal nom parser errors)
    pub fn first_msh_error(&self) -> Option<MshParserErrorKind> {
        self.begin_msh_errors().next().map(|(_, ek)| ek).cloned()
    }
}

impl<I: Clone> MshParserError<I> {
    /// Returns a backtrace containing only the errors that are actual MSH format errors (i.e. without internal nom parser errors)
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

pub(crate) trait MapMshError<I> {
    /// Maps the MshParserError if self contains an error
    fn map_msh_err<F>(self, f: F) -> Self
    where
        F: FnOnce(MshParserError<I>) -> MshParserError<I>;

    /// Appends the specified error if self already contains an error
    fn with_error(self, input: I, kind: MshParserErrorKind) -> Self
    where
        Self: Sized,
    {
        self.map_msh_err(|e| e.with_append(input, kind))
    }

    /// Appends the given context if self already contains an error
    fn with_context<S: Into<Cow<'static, str>>>(self, input: I, ctx: S) -> Self
    where
        Self: Sized,
    {
        self.map_msh_err(|e| e.with_context(input, ctx))
    }

    /// Obtains a context from the given callable if self already contains an error
    fn with_context_from<S: Into<Cow<'static, str>>, C: Fn() -> S>(self, input: I, ctx: C) -> Self
    where
        Self: Sized,
    {
        self.map_msh_err(|e| e.with_context(input, ctx()))
    }
}

/// Implementation that allows to map a MshParserError inside of an nom::Err, if it contains one
impl<I> MapMshError<I> for nom::Err<MshParserError<I>> {
    fn map_msh_err<F>(self, f: F) -> Self
    where
        F: FnOnce(MshParserError<I>) -> MshParserError<I>,
    {
        self.map(f)
    }
}

/// Implementation that allows to map a MshParserError inside of an IResult, if it contains one
impl<I, O> MapMshError<I> for IResult<I, O, MshParserError<I>> {
    fn map_msh_err<F>(self, f: F) -> Self
    where
        F: FnOnce(MshParserError<I>) -> MshParserError<I>,
    {
        self.map_err(|err| err.map(f))
    }
}
