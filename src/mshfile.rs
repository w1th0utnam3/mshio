use std::collections::HashMap;
use std::hash::Hash;

use nom::number::Endianness;
use num::{Float, Integer, Signed, ToPrimitive, Unsigned};
use num_derive::FromPrimitive;

/// A parsed MSH file containing mesh and header data
///
/// Models MSH files after revision 4.1 described at
/// [gmsh.info](http://gmsh.info/doc/texinfo/gmsh.html#MSH-file-format)
#[derive(PartialEq, Debug)]
pub struct MshFile<UsizeT, IntT, FloatT>
where
    UsizeT: Unsigned + Integer + ToPrimitive + Hash,
    IntT: Signed + Integer + ToPrimitive,
    FloatT: Float + ToPrimitive,
{
    /// Data extracted from the file format header
    pub header: MshHeader,
    /// Actual mesh data of the MSH file
    pub data: MshData<UsizeT, IntT, FloatT>,
}

impl<UsizeT, IntT, FloatT> MshFile<UsizeT, IntT, FloatT>
where
    UsizeT: Unsigned + Integer + ToPrimitive + Hash,
    IntT: Signed + Integer + ToPrimitive,
    FloatT: Float + ToPrimitive,
{
    /// Returns the total number of nodes in the MSH file
    pub fn total_node_count(&self) -> usize {
        if let Some(nodes) = self.data.nodes.as_ref() {
            nodes.num_nodes.to_usize().unwrap()
        } else {
            0
        }
    }

    /// Returns the total number of elements in the MSH file
    pub fn total_element_count(&self) -> usize {
        if let Some(elements) = self.data.elements.as_ref() {
            elements.num_elements.to_usize().unwrap()
        } else {
            0
        }
    }
}

/// The header of a MSH file (irrelevant for most users)
#[derive(PartialEq, Debug)]
pub struct MshHeader {
    /// File format version of the parsed MSH file
    pub version: f64,
    /// File type of the MSH file (0=ascii, 1=binary)
    pub file_type: i32,
    /// Size in bytes of the size_t data type in this MSH file
    pub size_t_size: usize,
    /// Size in bytes of the int data type in this MSH file
    pub int_size: usize,
    /// The detected endianness of this MSh file if it is binary
    pub endianness: Option<Endianness>,
}

/// Mesh data of a
#[derive(PartialEq, Debug)]
pub struct MshData<UsizeT, IntT, FloatT>
where
    UsizeT: Unsigned + Integer + Hash,
    IntT: Signed + Integer,
    FloatT: Float,
{
    /// Geometric entities of this mesh such as points, curves, etc. (if it contains entities)
    pub entities: Option<Entities<IntT, FloatT>>,
    /// Node data of this mesh (if it contains nodes)
    pub nodes: Option<Nodes<UsizeT, IntT, FloatT>>,
    /// Element data of this mesh (if it contains nodes)
    pub elements: Option<Elements<UsizeT, IntT>>,
}

/// Boundary representations of geometrical entities of the MSH file
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

/// A geometrical point entity
#[derive(PartialEq, Debug)]
pub struct Point<IntT, FloatT>
where
    IntT: Signed + Integer,
    FloatT: Float,
{
    /// The entity tag of this point
    pub tag: IntT,
    /// X-coordinate of this point
    pub x: FloatT,
    /// Y-coordinate of this point
    pub y: FloatT,
    /// Z-coordinate of this point
    pub z: FloatT,
    /// Tags of physical groups this point belongs to
    ///
    /// This is currently unimplemented.
    pub physical_tags: Vec<IntT>,
}

/// A geometrical curve entity and its boundary
#[derive(PartialEq, Debug)]
pub struct Curve<IntT, FloatT>
where
    IntT: Signed + Integer,
    FloatT: Float,
{
    /// The entity tag of this curve
    pub tag: IntT,
    /// Lower x-coordinate bound of this curve
    pub min_x: FloatT,
    /// Lower y-coordinate bound of this curve
    pub min_y: FloatT,
    /// Lower z-coordinate bound of this curve
    pub min_z: FloatT,
    /// Upper x-coordinate bound of this curve
    pub max_x: FloatT,
    /// Upper y-coordinate bound of this curve
    pub max_y: FloatT,
    /// Upper z-coordinate bound of this curve
    pub max_z: FloatT,
    /// Tags of physical groups this curve belongs to
    ///
    /// This is currently unimplemented.
    pub physical_tags: Vec<IntT>,
    /// Tags of the curves's bounding points
    pub point_tags: Vec<IntT>,
}

/// A geometrical surface entity and its boundary
#[derive(PartialEq, Debug)]
pub struct Surface<IntT, FloatT>
where
    IntT: Signed + Integer,
    FloatT: Float,
{
    /// The entity tag of this surface
    pub tag: IntT,
    /// Lower x-coordinate bound of this surface
    pub min_x: FloatT,
    /// Lower y-coordinate bound of this surface
    pub min_y: FloatT,
    /// Lower z-coordinate bound of this surface
    pub min_z: FloatT,
    /// Upper x-coordinate bound of this surface
    pub max_x: FloatT,
    /// Upper y-coordinate bound of this surface
    pub max_y: FloatT,
    /// Upper z-coordinate bound of this surface
    pub max_z: FloatT,
    /// Tags of physical groups this surface belongs to
    ///
    /// This is currently unimplemented.
    pub physical_tags: Vec<IntT>,
    /// Tags of the surface's bounding curves
    pub curve_tags: Vec<IntT>,
}

/// A geometrical volume entity and its boundary
#[derive(PartialEq, Debug)]
pub struct Volume<IntT, FloatT>
where
    IntT: Signed + Integer,
    FloatT: Float,
{
    /// The entity tag of this volume
    pub tag: IntT,
    /// Lower x-coordinate bound of this volume
    pub min_x: FloatT,
    /// Lower y-coordinate bound of this volume
    pub min_y: FloatT,
    /// Lower z-coordinate bound of this volume
    pub min_z: FloatT,
    /// Upper x-coordinate bound of this volume
    pub max_x: FloatT,
    /// Upper y-coordinate bound of this volume
    pub max_y: FloatT,
    /// Upper z-coordinate bound of this volume
    pub max_z: FloatT,
    /// Tags of physical groups this volume belongs to
    ///
    /// This is currently unimplemented.
    pub physical_tags: Vec<IntT>,
    /// Tags of the volumes's bounding surfaces
    pub surface_tags: Vec<IntT>,
}

/// All node data of a mesh
#[derive(PartialEq, Debug)]
pub struct Nodes<UsizeT, IntT, FloatT>
where
    UsizeT: Unsigned + Integer + Hash,
    IntT: Signed + Integer,
    FloatT: Float,
{
    /// Total number of nodes across all node blocks
    pub num_nodes: UsizeT,
    /// The smallest node tag assigned to a node
    pub min_node_tag: UsizeT,
    /// The largest node tag assigned to a node
    pub max_node_tag: UsizeT,
    /// Blocks of nodes with shared properties
    pub node_entities: Vec<NodeBlock<UsizeT, IntT, FloatT>>,
}

/// A block of nodes
#[derive(PartialEq, Debug)]
pub struct NodeBlock<UsizeT, IntT, FloatT>
where
    UsizeT: Unsigned + Integer + Hash,
    IntT: Signed + Integer,
    FloatT: Float,
{
    /// The number of dimensions of nodes in this block
    pub entity_dim: IntT,
    /// The tag of the geometric entity this block of elements is associated to
    pub entity_tag: IntT,
    /// Whether this node entity provides parametric coordinates for its nodes
    ///
    /// This is currently unimplemented.
    pub parametric: bool,
    /// Maps the tag of each node to its linear index in this block
    ///
    /// Node tags (used to reference nodes from entities) can be non-sequential (i.e. sparse).
    /// This map is only present if the node tags of this block are actually sparse.
    /// Otherwise it is None.
    pub node_tags: Option<HashMap<UsizeT, usize>>,
    /// The nodes of this block
    pub nodes: Vec<Node<FloatT>>,
    /// May contain parametric coordinates of the nodes
    ///
    /// This is currently unimplemented.
    pub parametric_nodes: Option<Vec<Node<FloatT>>>,
}

/// Coordinates of a single node
///
/// Note that only the components corresponding to the number of dimensions of the node's block
/// may contain meaningful values.
#[derive(PartialEq, Debug)]
pub struct Node<FloatT>
where
    FloatT: Float,
{
    /// X-coordinate of the node
    pub x: FloatT,
    /// Y-coordinate of the node (if entity_dim > 1)
    pub y: FloatT,
    /// Z-coordinate of the node (if entity_dim > 2)
    pub z: FloatT,
}

/// All element data of a mesh
#[derive(PartialEq, Debug)]
pub struct Elements<UsizeT, IntT>
where
    UsizeT: Unsigned + Integer + Hash,
    IntT: Signed + Integer,
{
    /// Total number of elements across all element blocks
    pub num_elements: UsizeT,
    /// The smallest element tag assigned to an element
    pub min_element_tag: UsizeT,
    /// The largest element tag assigned to an element
    pub max_element_tag: UsizeT,
    /// Blocks of elements with shared properties
    pub element_entities: Vec<ElementBlock<UsizeT, IntT>>,
}

/// A block of elements
#[derive(PartialEq, Debug)]
pub struct ElementBlock<UsizeT, IntT>
where
    UsizeT: Unsigned + Integer + Hash,
    IntT: Signed + Integer,
{
    /// The number of dimensions of elements in this block
    pub entity_dim: IntT,
    /// The tag of the geometric entity this block of elements is associated to
    pub entity_tag: IntT,
    /// The type of all elements in this block
    pub element_type: ElementType,
    /// Maps the tag of each element to its linear index in this block
    ///
    /// Element tags (used to reference elements from entities) can be non-sequential (i.e. sparse).
    /// This map is only present if the element tags of this block are actually sparse.
    /// Otherwise it is None.
    pub element_tags: Option<HashMap<UsizeT, usize>>,
    /// The elements of this block
    pub elements: Vec<Element<UsizeT>>,
}

/// Data of one mesh element
#[derive(PartialEq, Debug)]
pub struct Element<UsizeT>
where
    UsizeT: Unsigned + Integer,
{
    /// Tag of this element
    pub element_tag: UsizeT,
    /// The tags of nodes associated to this element
    pub nodes: Vec<UsizeT>,
}

/// Element types supported by the MSH file format
///
/// Based on Gmsh's [GmshDefines.h](https://gitlab.onelab.info/gmsh/gmsh/blob/master/Common/GmshDefines.h) header.
/// ```
/// use mshio::mshfile::ElementType;
/// use num_traits::FromPrimitive;
/// assert_eq!(ElementType::from_u8(4).unwrap(), ElementType::Tet4);
/// assert!(ElementType::from_u8(0).is_none());
/// assert!(ElementType::from_u8(141).is_none());
/// ```
#[derive(Copy, Clone, PartialEq, Debug, FromPrimitive)]
pub enum ElementType {
    Lin2 = 1,
    Tri3 = 2,
    Qua4 = 3,
    Tet4 = 4,
    Hex8 = 5,
    Pri6 = 6,
    Pyr5 = 7,
    Lin3 = 8,
    Tri6 = 9,
    Qua9 = 10,
    Tet10 = 11,
    Hex27 = 12,
    Pri18 = 13,
    Pyr14 = 14,
    Pnt = 15,
    Qua8 = 16,
    Hex20 = 17,
    Pri15 = 18,
    Pyr13 = 19,
    Tri9 = 20,
    Tri10 = 21,
    Tri12 = 22,
    Tri15 = 23,
    Tri15i = 24,
    Tri21 = 25,
    Lin4 = 26,
    Lin5 = 27,
    Lin6 = 28,
    Tet20 = 29,
    Tet35 = 30,
    Tet56 = 31,
    Tet22 = 32,
    Tet28 = 33,
    Polyg = 34,
    Polyh = 35,
    Qua16 = 36,
    Qua25 = 37,
    Qua36 = 38,
    Qua12 = 39,
    Qua16i = 40,
    Qua20 = 41,
    Tri28 = 42,
    Tri36 = 43,
    Tri45 = 44,
    Tri55 = 45,
    Tri66 = 46,
    Qua49 = 47,
    Qua64 = 48,
    Qua81 = 49,
    Qua100 = 50,
    Qua121 = 51,
    Tri18 = 52,
    Tri21i = 53,
    Tri24 = 54,
    Tri27 = 55,
    Tri30 = 56,
    Qua24 = 57,
    Qua28 = 58,
    Qua32 = 59,
    Qua36i = 60,
    Qua40 = 61,
    Lin7 = 62,
    Lin8 = 63,
    Lin9 = 64,
    Lin10 = 65,
    Lin11 = 66,
    LinB = 67,
    TriB = 68,
    PolygB = 69,
    LinC = 70,
    // TETS COMPLETE (6->10)
    Tet84 = 71,
    Tet120 = 72,
    Tet165 = 73,
    Tet220 = 74,
    Tet286 = 75,
    // TETS INCOMPLETE (6->10)
    Tet34 = 79,
    Tet40 = 80,
    Tet46 = 81,
    Tet52 = 82,
    Tet58 = 83,
    //
    Lin1 = 84,
    Tri1 = 85,
    Qua1 = 86,
    Tet1 = 87,
    Hex1 = 88,
    Pri1 = 89,
    Pri40 = 90,
    Pri75 = 91,
    // HEXES COMPLETE (3->9)
    Hex64 = 92,
    Hex125 = 93,
    Hex216 = 94,
    Hex343 = 95,
    Hex512 = 96,
    Hex729 = 97,
    Hex1000 = 98,
    // HEXES INCOMPLETE (3->9)
    Hex32 = 99,
    Hex44 = 100,
    Hex56 = 101,
    Hex68 = 102,
    Hex80 = 103,
    Hex92 = 104,
    Hex104 = 105,
    // PRISMS COMPLETE (5->9)
    Pri126 = 106,
    Pri196 = 107,
    Pri288 = 108,
    Pri405 = 109,
    Pri550 = 110,
    // PRISMS INCOMPLETE (3->9)
    Pri24 = 111,
    Pri33 = 112,
    Pri42 = 113,
    Pri51 = 114,
    Pri60 = 115,
    Pri69 = 116,
    Pri78 = 117,
    // PYRAMIDS COMPLETE (3->9)
    Pyr30 = 118,
    Pyr55 = 119,
    Pyr91 = 120,
    Pyr140 = 121,
    Pyr204 = 122,
    Pyr285 = 123,
    Pyr385 = 124,
    // PYRAMIDS INCOMPLETE (3->9)
    Pyr21 = 125,
    Pyr29 = 126,
    Pyr37 = 127,
    Pyr45 = 128,
    Pyr53 = 129,
    Pyr61 = 130,
    Pyr69 = 131,
    // Additional types
    Pyr1 = 132,
    PntSub = 133,
    LinSub = 134,
    TriSub = 135,
    TetSub = 136,
    Tet16 = 137,
    TriMini = 138,
    TetMini = 139,
    Trih4 = 140,
}

impl ElementType {
    /// Returns the number of nodes per element of an element type
    pub fn nodes(&self) -> Result<usize, ()> {
        Ok(match self {
            ElementType::Lin2 => 2,
            ElementType::Tri3 => 3,
            ElementType::Qua4 => 4,
            ElementType::Tet4 => 4,
            ElementType::Hex8 => 8,
            ElementType::Pri6 => 6,
            ElementType::Pyr5 => 5,
            ElementType::Lin3 => 3,
            ElementType::Tri6 => 6,
            ElementType::Qua9 => 9,
            ElementType::Tet10 => 10,
            ElementType::Hex27 => 27,
            ElementType::Pri18 => 28,
            ElementType::Pyr14 => 14,
            ElementType::Pnt => 1,
            ElementType::Qua8 => 8,
            ElementType::Hex20 => 20,
            ElementType::Pri15 => 15,
            ElementType::Pyr13 => 13,
            ElementType::Tri9 => 9,
            ElementType::Tri10 => 10,
            ElementType::Tri12 => 12,
            ElementType::Tri15 => 15,
            ElementType::Tri15i => 15,
            ElementType::Tri21 => 21,
            ElementType::Lin4 => 4,
            ElementType::Lin5 => 5,
            ElementType::Lin6 => 6,
            ElementType::Tet20 => 20,
            ElementType::Tet35 => 35,
            ElementType::Tet56 => 56,
            ElementType::Tet22 => 22,
            ElementType::Tet28 => 28,
            ElementType::Polyg => return Err(()),
            ElementType::Polyh => return Err(()),
            ElementType::Qua16 => 16,
            ElementType::Qua25 => 25,
            ElementType::Qua36 => 36,
            ElementType::Qua12 => 12,
            ElementType::Qua16i => 16,
            ElementType::Qua20 => 20,
            ElementType::Tri28 => 28,
            ElementType::Tri36 => 36,
            ElementType::Tri45 => 45,
            ElementType::Tri55 => 55,
            ElementType::Tri66 => 66,
            ElementType::Qua49 => 49,
            ElementType::Qua64 => 64,
            ElementType::Qua81 => 81,
            ElementType::Qua100 => 100,
            ElementType::Qua121 => 121,
            ElementType::Tri18 => 18,
            ElementType::Tri21i => 21,
            ElementType::Tri24 => 24,
            ElementType::Tri27 => 27,
            ElementType::Tri30 => 30,
            ElementType::Qua24 => 24,
            ElementType::Qua28 => 28,
            ElementType::Qua32 => 32,
            ElementType::Qua36i => 36,
            ElementType::Qua40 => 40,
            ElementType::Lin7 => 7,
            ElementType::Lin8 => 8,
            ElementType::Lin9 => 9,
            ElementType::Lin10 => 10,
            ElementType::Lin11 => 11,
            ElementType::LinB => return Err(()),
            ElementType::TriB => return Err(()),
            ElementType::PolygB => return Err(()),
            ElementType::LinC => return Err(()),
            ElementType::Tet84 => 84,
            ElementType::Tet120 => 120,
            ElementType::Tet165 => 165,
            ElementType::Tet220 => 220,
            ElementType::Tet286 => 286,
            ElementType::Tet34 => 34,
            ElementType::Tet40 => 40,
            ElementType::Tet46 => 46,
            ElementType::Tet52 => 52,
            ElementType::Tet58 => 58,
            ElementType::Lin1 => 1,
            ElementType::Tri1 => 1,
            ElementType::Qua1 => 1,
            ElementType::Tet1 => 1,
            ElementType::Hex1 => 1,
            ElementType::Pri1 => 1,
            ElementType::Pri40 => 40,
            ElementType::Pri75 => 75,
            ElementType::Hex64 => 64,
            ElementType::Hex125 => 125,
            ElementType::Hex216 => 216,
            ElementType::Hex343 => 343,
            ElementType::Hex512 => 512,
            ElementType::Hex729 => 729,
            ElementType::Hex1000 => 1000,
            ElementType::Hex32 => 32,
            ElementType::Hex44 => 44,
            ElementType::Hex56 => 56,
            ElementType::Hex68 => 68,
            ElementType::Hex80 => 80,
            ElementType::Hex92 => 92,
            ElementType::Hex104 => 104,
            ElementType::Pri126 => 126,
            ElementType::Pri196 => 196,
            ElementType::Pri288 => 288,
            ElementType::Pri405 => 405,
            ElementType::Pri550 => 550,
            ElementType::Pri24 => 24,
            ElementType::Pri33 => 33,
            ElementType::Pri42 => 42,
            ElementType::Pri51 => 51,
            ElementType::Pri60 => 60,
            ElementType::Pri69 => 69,
            ElementType::Pri78 => 78,
            ElementType::Pyr30 => 30,
            ElementType::Pyr55 => 55,
            ElementType::Pyr91 => 91,
            ElementType::Pyr140 => 140,
            ElementType::Pyr204 => 204,
            ElementType::Pyr285 => 285,
            ElementType::Pyr385 => 385,
            ElementType::Pyr21 => 21,
            ElementType::Pyr29 => 29,
            ElementType::Pyr37 => 37,
            ElementType::Pyr45 => 45,
            ElementType::Pyr53 => 53,
            ElementType::Pyr61 => 61,
            ElementType::Pyr69 => 69,
            ElementType::Pyr1 => 1,
            ElementType::PntSub => return Err(()),
            ElementType::LinSub => return Err(()),
            ElementType::TriSub => return Err(()),
            ElementType::TetSub => return Err(()),
            ElementType::Tet16 => 16,
            ElementType::TriMini => return Err(()),
            ElementType::TetMini => return Err(()),
            ElementType::Trih4 => return Err(()),
        })
    }
}
