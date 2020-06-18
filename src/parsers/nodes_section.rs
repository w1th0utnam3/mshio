use std::collections::HashMap;

use nom::error::ParseError;
use nom::multi::count;
use nom::IResult;

use crate::error::MshParserError;
use crate::mshfile::{MshFloatT, MshHeader, MshIntT, MshUsizeT, Node, NodeBlock, Nodes};
use crate::parsers::num_parsers;

pub(crate) fn parse_node_section<'a, 'b: 'a>(
    header: &'a MshHeader,
) -> impl Fn(&'b [u8]) -> IResult<&'b [u8], Nodes<usize, i32, f64>, MshParserError<&'b [u8]>>
{
    let header = header.clone();
    move |input| {
        let size_t_parser =
            num_parsers::uint_parser::<usize>(header.size_t_size, header.endianness);

        let (input, num_entity_blocks) = size_t_parser(input)?;
        let (input, num_nodes) = size_t_parser(input)?;
        let (input, min_node_tag) = size_t_parser(input)?;
        let (input, max_node_tag) = size_t_parser(input)?;

        let int_parser = num_parsers::int_parser::<i32>(header.int_size, header.endianness);
        let double_parser = num_parsers::float_parser::<f64>(header.float_size, header.endianness);

        let sparse_tags = if min_node_tag == 0 {
            panic!("Node tag 0 is reserved for internal use");
        } else if max_node_tag - min_node_tag > num_nodes - 1 {
            true
        } else {
            false
        };

        let (input, node_entity_blocks) = count(
            |i| parse_node_entity(size_t_parser, int_parser, double_parser, sparse_tags, i),
            num_entity_blocks,
        )(input)?;

        Ok((
            input,
            Nodes {
                num_nodes,
                min_node_tag,
                max_node_tag,
                node_blocks: node_entity_blocks,
            },
        ))
    }
}

fn parse_node_entity<
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
    sparse_tags: bool,
    input: &'a [u8],
) -> IResult<&'a [u8], NodeBlock<U, I, F>, E>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, E>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, E>,
    FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, E>,
{
    let (input, entity_dim) = int_parser(input)?;
    let (input, entity_tag) = int_parser(input)?;
    let (input, parametric) = int_parser(input)?;
    let (input, num_nodes_in_block) = size_t_parser(input)?;
    let num_nodes_in_block = num_nodes_in_block.to_usize().unwrap();

    let parametric = if parametric == I::zero() {
        false
    } else if parametric == I::one() {
        true
    } else {
        panic!("Unsupported value for node block attribute 'parametric' (only 0 and 1 supported)")
    };

    if parametric {
        unimplemented!("Parametric nodes are not supported yet");
    }

    let parse_node_tag = |input| {
        let (input, node_tag) = size_t_parser(input)?;
        Ok((input, node_tag))
    };

    let (input, node_tags) = if sparse_tags {
        let (input, node_tags) = count(parse_node_tag, num_nodes_in_block)(input)?;
        (
            input,
            Some(
                node_tags
                    .into_iter()
                    .enumerate()
                    .map(|(i, tag)| (tag, i))
                    .collect::<HashMap<_, _>>(),
            ),
        )
    } else {
        let (input, _) = count(parse_node_tag, num_nodes_in_block)(input)?;
        (input, None)
    };

    let parse_node = |input| {
        let (input, x) = double_parser(input)?;
        let (input, y) = double_parser(input)?;
        let (input, z) = double_parser(input)?;

        Ok((input, Node { x, y, z }))
    };

    let (input, nodes) = count(parse_node, num_nodes_in_block as usize)(input)?;

    Ok((
        input,
        NodeBlock {
            entity_dim,
            entity_tag,
            parametric,
            node_tags,
            nodes,
            parametric_nodes: None,
        },
    ))
}
