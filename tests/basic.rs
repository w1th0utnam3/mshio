fn msh_parses(msh: &[u8]) -> bool {
    match mshio::parse_msh_bytes::<nom::error::VerboseError<_>>(msh) {
        Ok((_, _)) => true,
        Err(err) => {
            println!("Error occured during parsing: {:?}", err);
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
    let circle_2d_bin = include_bytes!("circle_2d_bin.msh");
    assert!(msh_parses(circle_2d_bin));

    let (_, msh) = mshio::parse_msh_bytes::<()>(circle_2d_bin).unwrap();
    assert_eq!(msh.total_node_count(), 25);
    assert_eq!(msh.total_element_count(), 20);
}

#[test]
fn simple_ascii_file() {
    let circle_2d = include_str!("circle_2d.msh");
    assert!(msh_parses(circle_2d.as_bytes()));

    let (_, msh) = mshio::parse_msh_bytes::<()>(circle_2d.as_bytes()).unwrap();
    assert_eq!(msh.total_node_count(), 25);
    assert_eq!(msh.total_element_count(), 20);
}

#[test]
fn compare_simple_ascii_bin() {
    let circle_2d_bin_raw = include_bytes!("circle_2d_bin.msh");
    let circle_2d_raw = include_str!("circle_2d.msh");

    let (_, msh_bin) = mshio::parse_msh_bytes::<()>(circle_2d_bin_raw).unwrap();
    let (_, msh_ascii) = mshio::parse_msh_bytes::<()>(circle_2d_raw.as_bytes()).unwrap();

    // Headers differ, but data should be the same
    assert_eq!(msh_bin.data, msh_ascii.data);
}

#[test]
fn fine_bin_file() {
    let circle_2d_bin = include_bytes!("circle_2d_fine_bin.msh");
    assert!(msh_parses(circle_2d_bin));

    let (_, msh) = mshio::parse_msh_bytes::<()>(circle_2d_bin).unwrap();
    assert_eq!(msh.total_node_count(), 1313);
    assert_eq!(msh.total_element_count(), 1280);
}

#[test]
fn t13_bin_file() {
    let msh_bin = include_bytes!("t13_data.msh");
    assert!(msh_parses(msh_bin));

    let (_, msh) = mshio::parse_msh_bytes::<()>(msh_bin).unwrap();
    assert_eq!(msh.total_node_count(), 788);
    assert_eq!(msh.total_element_count(), 1864);
}

#[test]
fn cylinder_bin_file() {
    let msh_bin = include_bytes!("cylinder_3d.msh");
    assert!(msh_parses(msh_bin));

    let (_, msh) = mshio::parse_msh_bytes::<()>(msh_bin).unwrap();
    assert_eq!(msh.total_node_count(), 49602);
    assert_eq!(msh.total_element_count(), 9792);
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
