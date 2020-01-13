use nom::error::ParseError;
use nom::multi::count;
use nom::IResult;

use crate::mshfile::{Curve, Entities, MshFloatT, MshHeader, MshIntT, MshUsizeT, Surface, Volume};
use crate::parsers::num_parsers;

pub(crate) fn parse_entity_section<'a, 'b: 'a, E>(
    header: &'a MshHeader,
) -> impl Fn(&'b [u8]) -> IResult<&'b [u8], Entities<i32, f64>, E>
where
    E: ParseError<&'b [u8]>,
{
    let header = header.clone();
    move |input| {
        let size_t_parser =
            num_parsers::uint_parser::<usize, _>(header.size_t_size, header.endianness);
        let (input, num_points) = size_t_parser(input)?;
        let (input, num_curves) = size_t_parser(input)?;
        let (input, num_surfaces) = size_t_parser(input)?;
        let (input, num_volumes) = size_t_parser(input)?;

        let int_parser = num_parsers::int_parser::<i32, _>(header.int_size, header.endianness);
        let double_parser = num_parsers::float_parser::<f64, _>(8, header.endianness);

        for _ in 0..num_points {
            unimplemented!("Point entity reading not implemented")
        }

        let (input, curves) = count(
            |i| parse_curve(size_t_parser, int_parser, double_parser, i),
            num_curves,
        )(input)?;

        let (input, surfaces) = count(
            |i| parse_surface(size_t_parser, int_parser, double_parser, i),
            num_surfaces,
        )(input)?;

        let (input, volumes) = count(
            |i| parse_volume(size_t_parser, int_parser, double_parser, i),
            num_volumes,
        )(input)?;

        Ok((
            input,
            Entities {
                points: Vec::new(),
                curves,
                surfaces,
                volumes,
            },
        ))
    }
}

fn parse_curve<
    'a,
    U: MshUsizeT,
    I: MshIntT,
    F: MshFloatT,
    SizeTParser,
    IntParser,
    FloatParser,
    E: ParseError<&'a [u8]>,
>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    double_parser: FloatParser,
    input: &'a [u8],
) -> IResult<&'a [u8], Curve<I, F>, E>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, E>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, E>,
    FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, E>,
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

    let mut physical_tags = vec![I::zero(); num_physical_tags];
    for j in 0..num_physical_tags {
        physical_tags[j] = int_parser(input)?.1;
    }

    let (input, num_bounding_points) = size_t_parser(input)?;
    let num_bounding_points = num_bounding_points.to_usize().unwrap();

    let mut point_tags = vec![I::zero(); num_bounding_points];
    for j in 0..num_bounding_points {
        point_tags[j] = int_parser(input)?.1;
    }

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

fn parse_surface<
    'a,
    U: MshUsizeT,
    I: MshIntT,
    F: MshFloatT,
    SizeTParser,
    IntParser,
    FloatParser,
    E: ParseError<&'a [u8]>,
>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    double_parser: FloatParser,
    input: &'a [u8],
) -> IResult<&'a [u8], Surface<I, F>, E>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, E>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, E>,
    FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, E>,
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

    let mut physical_tags = vec![I::zero(); num_physical_tags];
    for j in 0..num_physical_tags {
        physical_tags[j] = int_parser(input)?.1;
    }

    let (input, num_bounding_curves) = size_t_parser(input)?;
    let num_bounding_curves = num_bounding_curves.to_usize().unwrap();

    let mut curve_tags = vec![I::zero(); num_bounding_curves];
    for j in 0..num_bounding_curves {
        curve_tags[j] = int_parser(input)?.1;
    }

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

fn parse_volume<
    'a,
    U: MshUsizeT,
    I: MshIntT,
    F: MshFloatT,
    SizeTParser,
    IntParser,
    FloatParser,
    E: ParseError<&'a [u8]>,
>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    double_parser: FloatParser,
    input: &'a [u8],
) -> IResult<&'a [u8], Volume<I, F>, E>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, E>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, E>,
    FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, E>,
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

    let mut physical_tags = vec![I::zero(); num_physical_tags];
    for j in 0..num_physical_tags {
        physical_tags[j] = int_parser(input)?.1;
    }

    let (input, num_bounding_surfaces) = size_t_parser(input)?;
    let num_bounding_surfaces = num_bounding_surfaces.to_usize().unwrap();

    let mut surface_tags = vec![I::zero(); num_bounding_surfaces];
    for j in 0..num_bounding_surfaces {
        surface_tags[j] = int_parser(input)?.1;
    }

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
