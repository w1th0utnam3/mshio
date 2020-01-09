use std::collections::HashMap;
use std::hash::Hash;

use nom::number::Endianness;
use num::{Float, Integer, Signed, ToPrimitive, Unsigned};

#[derive(PartialEq, Debug)]
pub struct MshFile<UsizeT, IntT, FloatT>
where
    UsizeT: Unsigned + Integer + ToPrimitive + Hash,
    IntT: Signed + Integer + ToPrimitive,
    FloatT: Float + ToPrimitive,
{
    pub header: MshHeader,
    pub data: MshData<UsizeT, IntT, FloatT>,
}

impl<UsizeT, IntT, FloatT> MshFile<UsizeT, IntT, FloatT>
where
    UsizeT: Unsigned + Integer + ToPrimitive + Hash,
    IntT: Signed + Integer + ToPrimitive,
    FloatT: Float + ToPrimitive,
{
    pub fn total_node_count(&self) -> usize {
        if let Some(nodes) = self.data.nodes.as_ref() {
            nodes.num_nodes.to_usize().unwrap()
        } else {
            0
        }
    }

    pub fn total_element_count(&self) -> usize {
        if let Some(elements) = self.data.elements.as_ref() {
            elements.num_elements.to_usize().unwrap()
        } else {
            0
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct MshHeader {
    pub version: f64,
    pub file_type: i32,
    pub size_t_size: usize,
    pub int_size: usize,
    pub endianness: Option<Endianness>,
}

#[derive(PartialEq, Debug)]
pub struct MshData<UsizeT, IntT, FloatT>
where
    UsizeT: Unsigned + Integer + Hash,
    IntT: Signed + Integer,
    FloatT: Float,
{
    pub entities: Option<Entities<IntT, FloatT>>,
    pub nodes: Option<Nodes<UsizeT, IntT, FloatT>>,
    pub elements: Option<Elements<UsizeT, IntT>>,
}

#[derive(PartialEq, Debug)]
pub struct Entities<IntT, FloatT>
where
    IntT: Signed + Integer,
    FloatT: Float,
{
    pub points: Vec<Point<IntT, FloatT>>,
    pub curves: Vec<Curve<IntT, FloatT>>,
    pub surfaces: Vec<Surface<IntT, FloatT>>,
    pub volumes: Vec<Volume<IntT, FloatT>>,
}

#[derive(PartialEq, Debug)]
pub struct Point<IntT, FloatT>
where
    IntT: Signed + Integer,
    FloatT: Float,
{
    pub tag: IntT,
    pub x: FloatT,
    pub y: FloatT,
    pub z: FloatT,
    pub physical_tags: Vec<IntT>,
}

#[derive(PartialEq, Debug)]
pub struct Curve<IntT, FloatT>
where
    IntT: Signed + Integer,
    FloatT: Float,
{
    pub tag: IntT,
    pub min_x: FloatT,
    pub min_y: FloatT,
    pub min_z: FloatT,
    pub max_x: FloatT,
    pub max_y: FloatT,
    pub max_z: FloatT,
    pub physical_tags: Vec<IntT>,
    pub point_tags: Vec<IntT>,
}

#[derive(PartialEq, Debug)]
pub struct Surface<IntT, FloatT>
where
    IntT: Signed + Integer,
    FloatT: Float,
{
    pub tag: IntT,
    pub min_x: FloatT,
    pub min_y: FloatT,
    pub min_z: FloatT,
    pub max_x: FloatT,
    pub max_y: FloatT,
    pub max_z: FloatT,
    pub physical_tags: Vec<IntT>,
    pub curve_tags: Vec<IntT>,
}

#[derive(PartialEq, Debug)]
pub struct Volume<IntT, FloatT>
where
    IntT: Signed + Integer,
    FloatT: Float,
{
    pub tag: IntT,
    pub min_x: FloatT,
    pub min_y: FloatT,
    pub min_z: FloatT,
    pub max_x: FloatT,
    pub max_y: FloatT,
    pub max_z: FloatT,
    pub physical_tags: Vec<IntT>,
    pub surface_tags: Vec<IntT>,
}

#[derive(PartialEq, Debug)]
pub struct Nodes<UsizeT, IntT, FloatT>
where
    UsizeT: Unsigned + Integer + Hash,
    IntT: Signed + Integer,
    FloatT: Float,
{
    /// Total number of nodes over all node entities
    pub num_nodes: UsizeT,
    /// The smallest node tag assigned to a node
    pub min_node_tag: UsizeT,
    /// The largest node tag assigned to a node
    pub max_node_tag: UsizeT,
    /// Entities (blocks) of nodes
    pub node_entities: Vec<NodeEntity<UsizeT, IntT, FloatT>>,
}

#[derive(PartialEq, Debug)]
pub struct NodeEntity<UsizeT, IntT, FloatT>
where
    UsizeT: Unsigned + Integer + Hash,
    IntT: Signed + Integer,
    FloatT: Float,
{
    pub entity_dim: IntT,
    pub entity_tag: IntT,
    pub parametric: bool,
    pub node_tags: Option<HashMap<UsizeT, usize>>,
    pub nodes: Vec<Node<FloatT>>,
    pub parametric_nodes: Option<Vec<Node<FloatT>>>,
}

#[derive(PartialEq, Debug)]
pub struct Node<FloatT>
where
    FloatT: Float,
{
    pub x: FloatT,
    pub y: FloatT,
    pub z: FloatT,
}

#[derive(PartialEq, Debug)]
pub struct Elements<UsizeT, IntT>
where
    UsizeT: Unsigned + Integer + Hash,
    IntT: Signed + Integer,
{
    /// Total number of elements over all element entities
    pub num_elements: UsizeT,
    /// The smallest element tag assigned to an element
    pub min_element_tag: UsizeT,
    /// The largest element tag assigned to an element
    pub max_element_tag: UsizeT,
    /// Entities (blocks) of elements
    pub element_entities: Vec<ElementEntity<UsizeT, IntT>>,
}

#[derive(PartialEq, Debug)]
pub struct ElementEntity<UsizeT, IntT>
where
    UsizeT: Unsigned + Integer + Hash,
    IntT: Signed + Integer,
{
    pub entity_dim: IntT,
    pub entity_tag: IntT,
    pub element_type: IntT,
    pub element_tags: Option<HashMap<UsizeT, usize>>,
    pub elements: Vec<Element<UsizeT>>,
}

#[derive(PartialEq, Debug)]
pub struct Element<UsizeT>
where
    UsizeT: Unsigned + Integer,
{
    pub element_tag: UsizeT,
    pub nodes: Vec<UsizeT>,
}
