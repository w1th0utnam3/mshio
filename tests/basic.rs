use std::fs::OpenOptions;
use std::io::{BufReader, Read};

use mshio::mshfile::ElementType;

fn read_bytes(path: &str) -> Vec<u8> {
    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(path)
        .unwrap();
    let mut buf_reader = BufReader::new(file);

    let mut data = Vec::new();
    buf_reader.read_to_end(&mut data).unwrap();
    data
}

/// Returns whether the supplied data can be parsed successfully as a MSH file
fn msh_parses(msh: &[u8]) -> bool {
    match mshio::parse_msh_bytes(msh) {
        Ok(_) => true,
        Err(err) => {
            println!("Error: {}", err);
            false
        }
    }
}

#[test]
fn does_nothing() {
    assert!(true);
}

#[test]
fn simple_bin_file() {
    let circle_2d_bin = read_bytes("tests/circle_2d_bin.msh");
    assert!(msh_parses(&circle_2d_bin));

    let msh = mshio::parse_msh_bytes(&circle_2d_bin).unwrap();
    assert_eq!(msh.total_node_count(), 25);
    assert_eq!(msh.total_element_count(), 20);
    assert!(msh
        .data
        .elements
        .unwrap()
        .element_entities
        .iter()
        .all(|elem_entity| elem_entity.element_type == ElementType::Qua4));
}

#[test]
fn simple_ascii_file() {
    let circle_2d = read_bytes("tests/circle_2d.msh");
    assert!(msh_parses(&circle_2d));

    let msh = mshio::parse_msh_bytes(&circle_2d).unwrap();
    assert_eq!(msh.total_node_count(), 25);
    assert_eq!(msh.total_element_count(), 20);
    assert!(msh
        .data
        .elements
        .unwrap()
        .element_entities
        .iter()
        .all(|elem_entity| elem_entity.element_type == ElementType::Qua4));
}

#[test]
fn compare_simple_ascii_bin() {
    let circle_2d_bin_raw = read_bytes("tests/circle_2d_bin.msh");
    let circle_2d_raw = read_bytes("tests/circle_2d.msh");

    let msh_bin = mshio::parse_msh_bytes(&circle_2d_bin_raw).unwrap();
    let msh_ascii = mshio::parse_msh_bytes(&circle_2d_raw).unwrap();

    // Headers differ, but data should be the same
    assert_eq!(msh_bin.data, msh_ascii.data);
}

#[test]
fn fine_bin_file() {
    let circle_2d_bin = read_bytes("tests/circle_2d_fine_bin.msh");
    assert!(msh_parses(&circle_2d_bin));

    let msh = mshio::parse_msh_bytes(&circle_2d_bin).unwrap();
    assert_eq!(msh.total_node_count(), 1313);
    assert_eq!(msh.total_element_count(), 1280);
    assert!(msh
        .data
        .elements
        .unwrap()
        .element_entities
        .iter()
        .all(|elem_entity| elem_entity.element_type == ElementType::Qua4));
}

#[test]
fn t13_bin_file() {
    let msh_bin = read_bytes("tests/t13_data.msh");
    assert!(msh_parses(&msh_bin));

    let msh = mshio::parse_msh_bytes(&msh_bin).unwrap();
    assert_eq!(msh.total_node_count(), 788);
    assert_eq!(msh.total_element_count(), 1864);
    assert!(msh
        .data
        .elements
        .unwrap()
        .element_entities
        .iter()
        .all(|elem_entity| {
            elem_entity.element_type == ElementType::Lin2
                || elem_entity.element_type == ElementType::Tri3
        }));
}

#[test]
fn cylinder_bin_file() {
    let msh_bin = read_bytes("tests/cylinder_3d.msh");
    assert!(msh_parses(&msh_bin));

    let msh = mshio::parse_msh_bytes(&msh_bin).unwrap();
    assert_eq!(msh.total_node_count(), 49602);
    assert_eq!(msh.total_element_count(), 9792);
    assert!(msh
        .data
        .elements
        .unwrap()
        .element_entities
        .iter()
        .all(|elem_entity| elem_entity.element_type == ElementType::Tet20));
}

#[test]
fn comment_section_test() {
    let msh = "\
$MeshFormat
4.1 0 8
$EndMeshFormat
$Comment
Hello


$EndComment

";
    assert!(msh_parses(msh.as_bytes()));
}

#[test]
fn invalid_test() {
    let msh = "\
$MeshFormat
4.1 0 8
$EndMeshFormat
$Comment
$EndComment
Hello

";
    assert!(!msh_parses(msh.as_bytes()));
}

#[test]
fn old_msh_version() {
    let msh = read_bytes("tests/old_msh_version.msh");
    assert!(!msh_parses(&msh));
}

#[test]
fn coarse_bike_file() {
    let msh = read_bytes("tests/bike_coarse.obj_linear.msh");
    assert!(msh_parses(&msh));

    let msh = mshio::parse_msh_bytes(&msh).unwrap();
    assert_eq!(msh.total_node_count(), 52);
    assert_eq!(msh.total_element_count(), 54);
}

#[test]
fn fine_bike_file() {
    let msh = read_bytes("tests/bike_original.obj_linear.msh");
    assert!(msh_parses(&msh));

    let msh = mshio::parse_msh_bytes(&msh).unwrap();
    assert_eq!(msh.total_node_count(), 850);
    assert_eq!(msh.total_element_count(), 1292);
}

#[test]
fn fine_bike_curved_file() {
    let msh = read_bytes("tests/bike_original.obj_curved.msh");
    assert!(msh_parses(&msh));

    let msh = mshio::parse_msh_bytes(&msh).unwrap();
    assert_eq!(msh.total_node_count(), 2698);
    assert_eq!(msh.total_element_count(), 1292);
}
