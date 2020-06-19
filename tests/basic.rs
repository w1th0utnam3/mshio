use mshio::mshfile::ElementType;

#[macro_use]
mod utils;

use crate::utils::*;

#[test]
fn test_simple_bin_file() {
    let circle_2d_bin = read_test_mesh("circle_2d_bin.msh");
    assert!(msh_parses(&circle_2d_bin));

    let msh = mshio::parse_msh_bytes(&circle_2d_bin).unwrap();
    assert_eq!(msh.total_node_count(), 25);
    assert_eq!(msh.total_element_count(), 20);

    let element_types = msh.count_element_types();
    assert_eq!(element_types.len(), 1);
    assert_eq!(element_types.get(&ElementType::Qua4), Some(&20));
}

#[test]
fn test_simple_ascii_file() {
    let circle_2d = read_test_mesh("circle_2d.msh");
    assert!(msh_parses(&circle_2d));

    let msh = mshio::parse_msh_bytes(&circle_2d).unwrap();
    assert_eq!(msh.total_node_count(), 25);
    assert_eq!(msh.total_element_count(), 20);

    let element_types = msh.count_element_types();
    assert_eq!(element_types.len(), 1);
    assert_eq!(element_types.get(&ElementType::Qua4), Some(&20));
}

#[test]
fn test_compare_simple_ascii_bin() {
    let circle_2d_bin_raw = read_test_mesh("circle_2d_bin.msh");
    let circle_2d_raw = read_test_mesh("circle_2d.msh");

    let msh_bin = mshio::parse_msh_bytes(&circle_2d_bin_raw).unwrap();
    let msh_ascii = mshio::parse_msh_bytes(&circle_2d_raw).unwrap();

    // Headers differ, but data should be the same
    assert_eq!(msh_bin.data, msh_ascii.data);
}

#[test]
fn test_fine_bin_file() {
    let circle_2d_bin = read_test_mesh("circle_2d_fine_bin.msh");
    assert!(msh_parses(&circle_2d_bin));

    let msh = mshio::parse_msh_bytes(&circle_2d_bin).unwrap();
    assert_eq!(msh.total_node_count(), 1313);
    assert_eq!(msh.total_element_count(), 1280);

    let element_types = msh.count_element_types();
    assert_eq!(element_types.len(), 1);
    assert_eq!(element_types.get(&ElementType::Qua4), Some(&1280));
}

#[test]
fn test_t13_bin_file() {
    let msh_bin = read_test_mesh("t13_data.msh");
    assert!(msh_parses(&msh_bin));

    let msh = mshio::parse_msh_bytes(&msh_bin).unwrap();
    assert_eq!(msh.total_node_count(), 788);
    assert_eq!(msh.total_element_count(), 1864);

    let element_types = msh.count_element_types();
    assert_eq!(element_types.len(), 2);
    assert_eq!(element_types.get(&ElementType::Lin2), Some(&284));
    assert_eq!(element_types.get(&ElementType::Tri3), Some(&1580));
}

#[test]
fn test_cylinder_bin_file() {
    let msh_bin = read_test_mesh("cylinder_3d.msh");
    assert!(msh_parses(&msh_bin));

    let msh = mshio::parse_msh_bytes(&msh_bin).unwrap();
    assert_eq!(msh.total_node_count(), 49602);
    assert_eq!(msh.total_element_count(), 9792);

    let element_types = msh.count_element_types();
    assert_eq!(element_types.len(), 1);
    assert_eq!(element_types.get(&ElementType::Tet20), Some(&9792));
}

#[test]
fn test_sphere_point_entities_file_ascii() {
    let msh_bin = read_test_mesh("sphere_coarse.msh");
    assert!(msh_parses(&msh_bin));

    let msh = mshio::parse_msh_bytes(&msh_bin).unwrap();
    assert_eq!(msh.total_node_count(), 183);
    assert_eq!(msh.total_element_count(), 891);

    assert_eq!(msh.point_count(), 2);
    assert_eq!(msh.curve_count(), 3);
    assert_eq!(msh.surface_count(), 1);
    assert_eq!(msh.volume_count(), 1);

    let element_types = msh.count_element_types();
    assert_eq!(element_types.len(), 4);
    assert_eq!(element_types.get(&ElementType::Pnt), Some(&2));
    assert_eq!(element_types.get(&ElementType::Lin2), Some(&10));
    assert_eq!(element_types.get(&ElementType::Tri3), Some(&286));
    assert_eq!(element_types.get(&ElementType::Tet4), Some(&593));
}

#[test]
fn test_sphere_point_entities_file_bin() {
    let msh_bin = read_test_mesh("sphere_coarse_bin.msh");
    assert!(msh_parses(&msh_bin));

    let msh = mshio::parse_msh_bytes(&msh_bin).unwrap();
    assert_eq!(msh.total_node_count(), 183);
    assert_eq!(msh.total_element_count(), 891);

    assert_eq!(msh.point_count(), 2);
    assert_eq!(msh.curve_count(), 3);
    assert_eq!(msh.surface_count(), 1);
    assert_eq!(msh.volume_count(), 1);

    let element_types = msh.count_element_types();
    assert_eq!(element_types.len(), 4);
    assert_eq!(element_types.get(&ElementType::Pnt), Some(&2));
    assert_eq!(element_types.get(&ElementType::Lin2), Some(&10));
    assert_eq!(element_types.get(&ElementType::Tri3), Some(&286));
    assert_eq!(element_types.get(&ElementType::Tet4), Some(&593));
}

#[test]
fn test_comment_section() {
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
fn test_coarse_bike_file() {
    let msh = read_test_mesh("bike_coarse.obj_linear.msh");
    assert!(msh_parses(&msh));

    let msh = mshio::parse_msh_bytes(&msh).unwrap();
    assert_eq!(msh.total_node_count(), 52);
    assert_eq!(msh.total_element_count(), 54);

    let element_types = msh.count_element_types();
    assert_eq!(element_types.len(), 1);
    assert_eq!(element_types.get(&ElementType::Tri3), Some(&54));
}

#[test]
fn test_fine_bike_file() {
    let msh = read_test_mesh("bike_original.obj_linear.msh");
    assert!(msh_parses(&msh));

    let msh = mshio::parse_msh_bytes(&msh).unwrap();
    assert_eq!(msh.total_node_count(), 850);
    assert_eq!(msh.total_element_count(), 1292);

    let element_types = msh.count_element_types();
    assert_eq!(element_types.len(), 1);
    assert_eq!(element_types.get(&ElementType::Tri3), Some(&1292));
}

#[test]
fn test_fine_bike_curved_file() {
    let msh = read_test_mesh("bike_original.obj_curved.msh");
    assert!(msh_parses(&msh));

    let msh = mshio::parse_msh_bytes(&msh).unwrap();
    assert_eq!(msh.total_node_count(), 2698);
    assert_eq!(msh.total_element_count(), 1292);

    let element_types = msh.count_element_types();
    assert_eq!(element_types.len(), 2);
    assert_eq!(element_types.get(&ElementType::Tri3), Some(&1028));
    assert_eq!(element_types.get(&ElementType::Tri10), Some(&264));
}
