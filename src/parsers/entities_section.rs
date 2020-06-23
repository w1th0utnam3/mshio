use nom::multi::count;
use nom::IResult;

use crate::error::{context, context_from, error, MapMshError, MshParserError, MshParserErrorKind};
use crate::mshfile::{
    Curve, Entities, MshFloatT, MshHeader, MshIntT, MshUsizeT, Point, Surface, Volume,
};
use crate::parsers::count_indexed;
use crate::parsers::num_parsers;

// TODO: Additional errors are required when parsing the bounding box values of the entities

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
        macro_rules! parse_entities_of_kind {
            ($entity_type:ident, $num_entities:ident, $entity_parser_fun:ident) => {
                context(
                    concat!("entity section: ", stringify!($entity_type), "s"),
                    count_indexed(
                        |index, input| {
                            $entity_parser_fun(&size_t_parser, &int_parser, &double_parser, input)
                                .with_context_from(input, || {
                                    format!(
                                        concat!(stringify!($entity_type), " entity ({} of {})"),
                                        index + 1,
                                        $num_entities
                                    )
                                })
                        },
                        $num_entities,
                    ),
                )
            };
        }

        // Parse all individual entities
        let (input, points) = parse_entities_of_kind!(point, num_points, parse_point)(input)?;
        let (input, curves) = parse_entities_of_kind!(curve, num_curves, parse_curve)(input)?;
        let (input, surfaces) =
            parse_entities_of_kind!(surface, num_surfaces, parse_surface)(input)?;
        let (input, volumes) = parse_entities_of_kind!(volume, num_volumes, parse_volume)(input)?;

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
    SizeTParser: for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], u64, MshParserError<&'b [u8]>>,
{
    let to_usize_parser = num_parsers::construct_usize_parser(&size_t_parser);

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
    SizeTParser: for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], U, MshParserError<&'b [u8]>>,
    IntParser: for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], I, MshParserError<&'b [u8]>>,
    FloatParser: for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], F, MshParserError<&'b [u8]>>,
{
    let to_usize_parser = num_parsers::construct_usize_parser(&size_t_parser);

    let (input, point_tag) = context(
        "entity tag",
        error(MshParserErrorKind::InvalidTag, &int_parser),
    )(input)?;

    let (input, x) = context("x-coordinate", &double_parser)(input)?;
    let (input, y) = context("y-coordinate", &double_parser)(input)?;
    let (input, z) = context("z-coordinate", &double_parser)(input)?;

    let (input, num_physical_tags) = context("number of physical tags", &to_usize_parser)(input)?;

    let (input, physical_tags) = context(
        "point entity physical tags",
        count(
            context_from(
                || format!("Expected {} valid physical tags", num_physical_tags),
                error(MshParserErrorKind::InvalidTag, &int_parser),
            ),
            num_physical_tags,
        ),
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

macro_rules! single_entity_parser {
    ($parser_name:ident, $entity_type:ident, $entity_name:ident, $bounding_entity_name:ident, $bounding_entity_field:ident) => {
        fn $parser_name<
            'a,
            U: MshUsizeT,
            I: MshIntT,
            F: MshFloatT,
            SizeTParser,
            IntParser,
            FloatParser,
        >(
            size_t_parser: SizeTParser,
            int_parser: IntParser,
            double_parser: FloatParser,
            input: &'a [u8],
        ) -> IResult<&'a [u8], $entity_type<I, F>, MshParserError<&'a [u8]>>
        where
            SizeTParser: for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], U, MshParserError<&'b [u8]>>,
            IntParser: for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], I, MshParserError<&'b [u8]>>,
            FloatParser: for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], F, MshParserError<&'b [u8]>>,
        {
            let to_usize_parser = num_parsers::construct_usize_parser(&size_t_parser);

            let (input, entity_tag) = context(
                "entity tag",
                error(MshParserErrorKind::InvalidTag, &int_parser),
            )(input)?;

            let (input, min_x) = context("min x-coordinate", &double_parser)(input)?;
            let (input, min_y) = context("min y-coordinate", &double_parser)(input)?;
            let (input, min_z) = context("min z-coordinate", &double_parser)(input)?;
            let (input, max_x) = context("max x-coordinate", &double_parser)(input)?;
            let (input, max_y) = context("max x-coordinate", &double_parser)(input)?;
            let (input, max_z) = context("max x-coordinate", &double_parser)(input)?;

            let (input, num_physical_tags) =
                context("number of physical tags", &to_usize_parser)(input)?;

            let (input, physical_tags) = context(
                concat!(stringify!($entity_name), " entity physical tags"),
                count(
                    context_from(
                        || format!("Expected {} valid physical tags", num_physical_tags),
                        error(MshParserErrorKind::InvalidTag, &int_parser),
                    ),
                    num_physical_tags,
                ),
            )(input)?;

            let (input, num_bounding_entities) = context(
                concat!(
                    "number of bounding ",
                    stringify!($bounding_entity_name),
                    "s"
                ),
                &to_usize_parser,
            )(input)?;

            let (input, $bounding_entity_field) = context(
                concat!(
                    stringify!($entity_name),
                    " entity bounding ",
                    stringify!($bounding_entity_name),
                    " tags"
                ),
                count(
                    context_from(
                        || {
                            format!(
                                "Expected {} valid bounding entity tags",
                                num_bounding_entities
                            )
                        },
                        error(MshParserErrorKind::InvalidTag, &int_parser),
                    ),
                    num_bounding_entities,
                ),
            )(input)?;

            Ok((
                input,
                $entity_type {
                    tag: entity_tag,
                    min_x,
                    min_y,
                    min_z,
                    max_x,
                    max_y,
                    max_z,
                    physical_tags,
                    $bounding_entity_field,
                },
            ))
        }
    };
}

single_entity_parser!(parse_curve, Curve, curve, point, point_tags);
single_entity_parser!(parse_surface, Surface, surface, curve, curve_tags);
single_entity_parser!(parse_volume, Volume, volume, surface, surface_tags);
