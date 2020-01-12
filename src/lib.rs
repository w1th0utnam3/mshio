//! Parser for Gmsh mesh data of the MSH file format version 4.1

use std::convert::{TryFrom, TryInto};
use std::str;

use nom::bytes::complete::tag;
use nom::character::complete::{alpha0, char};
use nom::combinator::peek;
use nom::error::{context, ErrorKind, ParseError, VerboseError};
use nom::sequence::{delimited, preceded, terminated};
use nom::IResult;

/// Contains all types that are used to represent parsed MSH files
pub mod mshfile;
/// Parser utility functions used by this MSH parser (may be private in the future)
pub mod parsers;

pub use mshfile::*;
use parsers::{br, take_sp};
use parsers::{
    parse_element_section, parse_entity_section, parse_header_section, parse_node_section,
};

// TODO: Implement parser for point entities
// TODO: Implement parser for physical groups
// TODO: Replace panics and unimplemented! calls with Err
// TODO: Check imports of num vs num_traits
// TODO: Review the passing of primitive parser functions as generic parameters (don't support Copy)

// TODO: Add proper enum variants for custom error
// TODO: Global static strings for error context
// TODO: Map static string error context back to error enum variants

/// Debug helper to view u8 slice as utf8 str and print it
#[allow(dead_code)]
fn print_u8(text: &str, input: &[u8]) {
    println!("{}: '{}'", text, String::from_utf8_lossy(input));
}

/// Try to parse a MshFile from a slice of bytes
///
/// The input  can be the content of an ASCII or binary encoded MSH file of file format version 4.1.
impl<'a> TryFrom<&'a [u8]> for MshFile<usize, i32, f64> {
    type Error = MshParserError<&'a [u8]>;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match private_parse_msh_bytes::<VerboseError<_>>(value) {
            Ok((_, file)) => Ok(file),
            Err(e) => Err(e.into()),
        }
    }
}

/// Try to parse a MshFile from a slice of bytes
///
/// The input  can be the content of an ASCII or binary encoded MSH file of file format version 4.1.
pub fn parse_msh_bytes<'a>(
    input: &'a [u8],
) -> Result<MshFile<usize, i32, f64>, MshParserError<&'a [u8]>> {
    input.try_into()
}

fn private_parse_msh_bytes<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], MshFile<usize, i32, f64>, E> {
    let (input, header) = parsers::parse_delimited_block(
        terminated(tag("$MeshFormat"), br),
        terminated(tag("$EndMeshFormat"), br),
        parse_header_section,
    )(input)?;

    // Closure to detect a line with a section start tag
    let section_detected = |start_tag, input| {
        peek::<_, _, (), _>(delimited(take_sp, tag(start_tag), br))(input).is_ok()
    };

    // Macro to apply a parser to a section delimited by start and end tags
    macro_rules! parse_section {
        ($start_tag:expr, $end_tag:expr, $parser:expr, $input:expr) => {
            delimited(
                delimited(take_sp, tag($start_tag), br),
                $parser,
                delimited(take_sp, tag($end_tag), take_sp),
            )($input)
        };
    }

    let mut entity_sections = Vec::new();
    let mut node_sections = Vec::new();
    let mut element_sections = Vec::new();

    let mut input = input;

    // Loop over all sections of the mesh file
    while !parsers::eof::<_, ()>(input).is_ok() {
        // Check for entity section
        if section_detected("$Entities", input) {
            let (input_, entities) = parse_section!(
                "$Entities",
                "$EndEntities",
                |i| parse_entity_section(&header, i),
                input
            )?;

            entity_sections.push(entities);
            input = input_;
        }
        // Check for node section
        else if section_detected("$Nodes", input) {
            let (input_, nodes) = parse_section!(
                "$Nodes",
                "$EndNodes",
                |i| parse_node_section(&header, i),
                input
            )?;

            node_sections.push(nodes);
            input = input_;
        }
        // Check for elements section
        else if section_detected("$Elements", input) {
            let (input_, elements) = parse_section!(
                "$Elements",
                "$EndElements",
                |i| parse_element_section(&header, i),
                input
            )?;

            element_sections.push(elements);
            input = input_;
        }
        // Check for unknown section (gets ignored)
        else if let Ok((input_, section_header)) =
            peek::<_, _, (), _>(preceded(take_sp, delimited(char('$'), alpha0, br)))(input)
        {
            let section_header = String::from_utf8_lossy(section_header);
            let section_start_tag = format!("${}", section_header);
            let section_end_tag = format!("$End{}", section_header);

            let (input_, _) = parsers::delimited_block(
                delimited(take_sp, tag(&section_start_tag[..]), br),
                delimited(take_sp, tag(&section_end_tag[..]), take_sp),
            )(input_)?;

            input = input_;
        }
        // Check for invalid lines
        else {
            return context("Expected a section header", |i| {
                Err(nom::Err::Error(ParseError::from_error_kind(
                    i,
                    ErrorKind::Tag,
                )))
            })(input);
        }
    }

    let entities = match entity_sections.len() {
        1 => Some(entity_sections.remove(0)),
        0 => None,
        _ => unimplemented!("More than one entity section found"),
    };

    let nodes = match node_sections.len() {
        1 => Some(node_sections.remove(0)),
        0 => None,
        _ => unimplemented!("More than one node section found"),
    };

    let elements = match element_sections.len() {
        1 => Some(element_sections.remove(0)),
        0 => None,
        _ => unimplemented!("More than one element section found"),
    };

    Ok((
        input,
        MshFile {
            header,
            data: MshData {
                entities,
                nodes,
                elements,
            },
        },
    ))
}
