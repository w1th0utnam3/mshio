use nom::IResult;

use crate::error::MshParserError;
use crate::mshfile::{MshFloatT, MshIntT, MshUsizeT};
use crate::parsers::num_parsers::{construct_usize_parser, NumParsers};

pub(crate) trait ParsesSizeT<U: MshUsizeT> {
    fn parse_size_t<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>>;
    fn parse_to_usize<'a>(&self, i: &'a [u8])
        -> IResult<&'a [u8], usize, MshParserError<&'a [u8]>>;
}

pub(crate) trait ParsesInt<I: MshIntT> {
    fn parse_int<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>>;
}

pub(crate) trait ParsesFloat<F: MshFloatT> {
    fn parse_float<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], F, MshParserError<&'a [u8]>>;
}

impl<U: MshUsizeT, T> ParsesSizeT<U> for &T
where
    T: ParsesSizeT<U>,
{
    #[inline(always)]
    fn parse_size_t<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>> {
        (*self).parse_size_t(i)
    }

    #[inline(always)]
    fn parse_to_usize<'a>(
        &self,
        i: &'a [u8],
    ) -> IResult<&'a [u8], usize, MshParserError<&'a [u8]>> {
        (*self).parse_to_usize(i)
    }
}

impl<I: MshIntT, T> ParsesInt<I> for &T
where
    T: ParsesInt<I>,
{
    #[inline(always)]
    fn parse_int<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>> {
        (*self).parse_int(i)
    }
}

impl<F: MshFloatT, T> ParsesFloat<F> for &T
where
    T: ParsesFloat<F>,
{
    #[inline(always)]
    fn parse_float<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], F, MshParserError<&'a [u8]>> {
        (*self).parse_float(i)
    }
}

impl<U: MshUsizeT, I: MshIntT, F: MshFloatT, SizeTParser, IntParser, FloatParser> ParsesSizeT<U>
    for NumParsers<U, I, F, SizeTParser, IntParser, FloatParser>
where
    for<'a> SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>>,
    for<'a> IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>>,
    for<'a> FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, MshParserError<&'a [u8]>>,
{
    fn parse_size_t<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>> {
        (self.size_t_parser)(input)
    }

    fn parse_to_usize<'a>(
        &self,
        input: &'a [u8],
    ) -> IResult<&'a [u8], usize, MshParserError<&'a [u8]>> {
        construct_usize_parser(&self.size_t_parser)(input)
    }
}

impl<U: MshUsizeT, I: MshIntT, F: MshFloatT, SizeTParser, IntParser, FloatParser> ParsesInt<I>
    for NumParsers<U, I, F, SizeTParser, IntParser, FloatParser>
where
    for<'a> SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>>,
    for<'a> IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>>,
    for<'a> FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, MshParserError<&'a [u8]>>,
{
    fn parse_int<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>> {
        (self.int_parser)(input)
    }
}

impl<U: MshUsizeT, I: MshIntT, F: MshFloatT, SizeTParser, IntParser, FloatParser> ParsesFloat<F>
    for NumParsers<U, I, F, SizeTParser, IntParser, FloatParser>
where
    for<'a> SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>>,
    for<'a> IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>>,
    for<'a> FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, MshParserError<&'a [u8]>>,
{
    fn parse_float<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], F, MshParserError<&'a [u8]>> {
        (self.float_parser)(input)
    }
}

#[inline(always)]
pub(crate) fn size_t_parser<'a, U: MshUsizeT, P: ParsesSizeT<U> + 'a>(
    parser: P,
) -> impl for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], U, MshParserError<&'b [u8]>> + 'a {
    move |i| parser.parse_size_t(i)
}

#[inline(always)]
pub(crate) fn usize_parser<'a, U: MshUsizeT, P: ParsesSizeT<U> + 'a>(
    parser: P,
) -> impl for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], usize, MshParserError<&'b [u8]>> + 'a {
    move |i| parser.parse_to_usize(i)
}

#[inline(always)]
pub(crate) fn int_parser<'a, I: MshIntT, P: ParsesInt<I> + 'a>(
    parser: P,
) -> impl for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], I, MshParserError<&'b [u8]>> + 'a {
    move |i| parser.parse_int(i)
}

#[inline(always)]
pub(crate) fn float_parser<'a, F: MshFloatT, P: ParsesFloat<F> + 'a>(
    parser: P,
) -> impl for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], F, MshParserError<&'b [u8]>> + 'a {
    move |i| parser.parse_float(i)
}
