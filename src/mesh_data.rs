use std::collections::HashMap;
use std::hash::Hash;

use nom::number::Endianness;
use num::{Float, Integer, Signed, Unsigned};

#[derive(PartialEq, Debug)]
pub struct MshFile<UsizeT: Unsigned + Integer + Hash, IntT: Signed + Integer, FloatT: Float> {
    pub header: MshHeader,
    pub data: MshData<UsizeT, IntT, FloatT>,
}

impl<UsizeT: Unsigned + Integer + Hash, IntT: Signed + Integer, FloatT: Float>
    MshFile<UsizeT, IntT, FloatT>
{
    pub fn total_node_count(&self) -> usize {
        if let Some(nodes) = self.data.nodes.as_ref() {
            let mut node_count = 0;
            for node_entity in &nodes.node_entities {
                node_count += node_entity.nodes.len();
            }
            node_count
        } else {
            0
        }
    }

    pub fn total_element_count(&self) -> usize {
        if let Some(elements) = self.data.elements.as_ref() {
            let mut element_count = 0;
            for element_entity in &elements.element_entities {
                element_count += element_entity.elements.len();
            }
            element_count
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
pub struct MshData<UsizeT: Unsigned + Integer + Hash, IntT: Signed + Integer, FloatT: Float> {
    pub entities: Option<Entities<IntT, FloatT>>,
    pub nodes: Option<Nodes<UsizeT, IntT, FloatT>>,
    pub elements: Option<Elements<UsizeT, IntT>>,
}

#[derive(PartialEq, Debug)]
pub struct Entities<IntT: Signed + Integer, FloatT: Float> {
    pub points: Vec<Point<IntT, FloatT>>,
    pub curves: Vec<Curve<IntT, FloatT>>,
    pub surfaces: Vec<Surface<IntT, FloatT>>,
    pub volumes: Vec<Volume<IntT, FloatT>>,
}

#[derive(PartialEq, Debug)]
pub struct Point<IntT: Signed + Integer, FloatT: Float> {
    pub tag: IntT,
    pub x: FloatT,
    pub y: FloatT,
    pub z: FloatT,
    pub physical_tags: Vec<IntT>,
}

#[derive(PartialEq, Debug)]
pub struct Curve<IntT: Signed + Integer, FloatT: Float> {
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
pub struct Surface<IntT: Signed + Integer, FloatT: Float> {
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
pub struct Volume<IntT: Signed + Integer, FloatT: Float> {
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
pub struct Nodes<UsizeT: Unsigned + Integer + Hash, IntT: Signed + Integer, FloatT: Float> {
    pub min_node_tag: UsizeT,
    pub max_node_tag: UsizeT,
    pub node_entities: Vec<NodeEntity<UsizeT, IntT, FloatT>>,
}

#[derive(PartialEq, Debug)]
pub struct NodeEntity<UsizeT: Unsigned + Integer + Hash, IntT: Signed + Integer, FloatT: Float> {
    pub entity_dim: IntT,
    pub entity_tag: IntT,
    pub parametric: bool,
    pub node_tags: Option<HashMap<UsizeT, usize>>,
    pub nodes: Vec<Node<FloatT>>,
    pub parametric_nodes: Option<Vec<Node<FloatT>>>,
}

#[derive(PartialEq, Debug)]
pub struct Node<FloatT: Float> {
    pub x: FloatT,
    pub y: FloatT,
    pub z: FloatT,
}

#[derive(PartialEq, Debug)]
pub struct Elements<UsizeT: Unsigned + Integer + Hash, IntT: Signed + Integer> {
    pub min_node_tag: UsizeT,
    pub max_node_tag: UsizeT,
    pub element_entities: Vec<ElementEntity<UsizeT, IntT>>,
}

#[derive(PartialEq, Debug)]
pub struct ElementEntity<UsizeT: Unsigned + Integer + Hash, IntT: Signed + Integer> {
    pub entity_dim: IntT,
    pub entity_tag: IntT,
    pub element_type: IntT,
    pub element_tags: Option<HashMap<UsizeT, usize>>,
    pub elements: Vec<Element<UsizeT>>,
}

#[derive(PartialEq, Debug)]
pub struct Element<UsizeT: Unsigned + Integer> {
    pub element_tag: UsizeT,
    pub nodes: Vec<UsizeT>,
}
