use nom::multi::count;
use nom::IResult;

use crate::error::{context, MapMshError, MshParserError};
use crate::mshfile::{
    Curve, Entities, MshFloatT, MshHeader, MshIntT, MshUsizeT, Point, Surface, Volume,
};
use crate::parsers::count_indexed;
use crate::parsers::num_parsers;

struct EntitySectionHeader {
    num_points: usize,
    num_curves: usize,
    num_surfaces: usize,
    num_volumes: usize,
}

pub(crate) fn parse_entity_section<'a, 'b: 'a>(
    header: &'a MshHeader,
) -> impl Fn(&'b [u8]) -> IResult<&'b [u8], Entities<i32, f64>, MshParserError<&'b [u8]>> {
    let header = header.clone();
    move |input| {
        let size_t_parser = num_parsers::uint_parser::<u64>(header.size_t_size, header.endianness);
        let int_parser = num_parsers::int_parser::<i32>(header.int_size, header.endianness);
        let double_parser = num_parsers::float_parser::<f64>(header.float_size, header.endianness);

        // Parse the section header
        let (input, entity_section_header) = context("entity section header", |input| {
            parse_entity_section_header(&size_t_parser, input)
        })(input)?;

        let EntitySectionHeader {
            num_points,
            num_curves,
            num_surfaces,
            num_volumes,
        } = entity_section_header;

        // Macro that returns a parser that runs an entity parser `$entity_parser_fun`
        // for `$num_entities` times and adds context messages
        macro_rules! entity_parser {
            ($entity_type:ident, $num_entities:ident, $entity_parser_fun:ident) => {
                context(
                    concat!("entity section: ", stringify!($entity_type), "s"),
                    count_indexed(
                        |index, input| {
                            $entity_parser_fun(&size_t_parser, &int_parser, &double_parser, input)
                                .with_context_from(input, || {
                                    format!(concat!(stringify!($entity_type), " entity ({} of {})"), index + 1, $num_entities)
                                })
                        },
                        $num_entities,
                    ),
                )
            }
        }

        let (input, points) = entity_parser!(point, num_points, parse_point)(input)?;
        let (input, curves) = entity_parser!(curve, num_curves, parse_curve)(input)?;
        let (input, surfaces) = entity_parser!(surface, num_surfaces, parse_surface)(input)?;
        let (input, volumes) = entity_parser!(volume, num_volumes, parse_volume)(input)?;

        Ok((
            input,
            Entities {
                points,
                curves,
                surfaces,
                volumes,
            },
        ))
    }
}

fn parse_entity_section_header<'a, SizeTParser>(
    size_t_parser: SizeTParser,
    input: &'a [u8],
) -> IResult<&'a [u8], EntitySectionHeader, MshParserError<&'a [u8]>>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], u64, MshParserError<&'a [u8]>>,
{
    let to_usize_parser = num_parsers::usize_parser(&size_t_parser);

    let (input, num_points) = context("number of point entities", &to_usize_parser)(input)?;
    let (input, num_curves) = context("number of curve entities", &to_usize_parser)(input)?;
    let (input, num_surfaces) = context("number of surface entities", &to_usize_parser)(input)?;
    let (input, num_volumes) = context("number of volume entities", &to_usize_parser)(input)?;

    Ok((
        input,
        EntitySectionHeader {
            num_points,
            num_curves,
            num_surfaces,
            num_volumes,
        },
    ))
}

fn parse_point<'a, U: MshUsizeT, I: MshIntT, F: MshFloatT, SizeTParser, IntParser, FloatParser>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    double_parser: FloatParser,
    input: &'a [u8],
) -> IResult<&'a [u8], Point<I, F>, MshParserError<&'a [u8]>>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>>,
    FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, MshParserError<&'a [u8]>>,
{
    let (input, point_tag) = int_parser(input)?;

    let (input, x) = double_parser(input)?;
    let (input, y) = double_parser(input)?;
    let (input, z) = double_parser(input)?;

    let (input, num_physical_tags) = size_t_parser(input)?;
    let num_physical_tags = num_physical_tags.to_usize().unwrap();

    let (input, physical_tags) = context(
        "Point entity physical tags",
        count(|i| int_parser(i), num_physical_tags),
    )(input)?;

    Ok((
        input,
        Point {
            tag: point_tag,
            x,
            y,
            z,
            physical_tags,
        },
    ))
}

fn parse_curve<'a, U: MshUsizeT, I: MshIntT, F: MshFloatT, SizeTParser, IntParser, FloatParser>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    double_parser: FloatParser,
    input: &'a [u8],
) -> IResult<&'a [u8], Curve<I, F>, MshParserError<&'a [u8]>>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>>,
    FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, MshParserError<&'a [u8]>>,
{
    let (input, curve_tag) = int_parser(input)?;

    let (input, min_x) = double_parser(input)?;
    let (input, min_y) = double_parser(input)?;
    let (input, min_z) = double_parser(input)?;
    let (input, max_x) = double_parser(input)?;
    let (input, max_y) = double_parser(input)?;
    let (input, max_z) = double_parser(input)?;

    let (input, num_physical_tags) = size_t_parser(input)?;
    let num_physical_tags = num_physical_tags.to_usize().unwrap();

    let (input, physical_tags) = context(
        "Curve entity physical tags",
        count(|i| int_parser(i), num_physical_tags),
    )(input)?;

    let (input, num_bounding_points) = size_t_parser(input)?;
    let num_bounding_points = num_bounding_points.to_usize().unwrap();

    let (input, point_tags) = context(
        "Curve entity bounding point tags",
        count(|i| int_parser(i), num_bounding_points),
    )(input)?;

    Ok((
        input,
        Curve {
            tag: curve_tag,
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
            physical_tags,
            point_tags,
        },
    ))
}

fn parse_surface<'a, U: MshUsizeT, I: MshIntT, F: MshFloatT, SizeTParser, IntParser, FloatParser>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    double_parser: FloatParser,
    input: &'a [u8],
) -> IResult<&'a [u8], Surface<I, F>, MshParserError<&'a [u8]>>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>>,
    FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, MshParserError<&'a [u8]>>,
{
    let (input, surface_tag) = int_parser(input)?;

    let (input, min_x) = double_parser(input)?;
    let (input, min_y) = double_parser(input)?;
    let (input, min_z) = double_parser(input)?;
    let (input, max_x) = double_parser(input)?;
    let (input, max_y) = double_parser(input)?;
    let (input, max_z) = double_parser(input)?;

    let (input, num_physical_tags) = size_t_parser(input)?;
    let num_physical_tags = num_physical_tags.to_usize().unwrap();

    let (input, physical_tags) = context(
        "Surface entity physical tags",
        count(|i| int_parser(i), num_physical_tags),
    )(input)?;

    let (input, num_bounding_curves) = size_t_parser(input)?;
    let num_bounding_curves = num_bounding_curves.to_usize().unwrap();

    let (input, curve_tags) = context(
        "Surface entity bounding curve tags",
        count(|i| int_parser(i), num_bounding_curves),
    )(input)?;

    Ok((
        input,
        Surface {
            tag: surface_tag,
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
            physical_tags,
            curve_tags,
        },
    ))
}

fn parse_volume<'a, U: MshUsizeT, I: MshIntT, F: MshFloatT, SizeTParser, IntParser, FloatParser>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    double_parser: FloatParser,
    input: &'a [u8],
) -> IResult<&'a [u8], Volume<I, F>, MshParserError<&'a [u8]>>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>>,
    FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, MshParserError<&'a [u8]>>,
{
    let (input, volume_tag) = int_parser(input)?;

    let (input, min_x) = double_parser(input)?;
    let (input, min_y) = double_parser(input)?;
    let (input, min_z) = double_parser(input)?;
    let (input, max_x) = double_parser(input)?;
    let (input, max_y) = double_parser(input)?;
    let (input, max_z) = double_parser(input)?;

    let (input, num_physical_tags) = size_t_parser(input)?;
    let num_physical_tags = num_physical_tags.to_usize().unwrap();

    let (input, physical_tags) = context(
        "Volume entity physical tags",
        count(|i| int_parser(i), num_physical_tags),
    )(input)?;

    let (input, num_bounding_surfaces) = size_t_parser(input)?;
    let num_bounding_surfaces = num_bounding_surfaces.to_usize().unwrap();

    let (input, surface_tags) = context(
        "Volume entity bounding point surfaces",
        count(|i| int_parser(i), num_bounding_surfaces),
    )(input)?;

    Ok((
        input,
        Volume {
            tag: volume_tag,
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
            physical_tags,
            surface_tags,
        },
    ))
}
