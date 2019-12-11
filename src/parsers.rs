pub(crate) mod general_parsers;
pub(crate) mod num_parsers;

mod elements_section;
mod entities_section;
mod header_section;
mod nodes_section;

pub(crate) use elements_section::parse_element_section;
pub(crate) use entities_section::parse_entity_section;
pub(crate) use header_section::parse_header_section;
pub(crate) use nodes_section::parse_node_section;

pub use general_parsers::*;
