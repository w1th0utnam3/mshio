use std::fs::OpenOptions;
use std::io::{BufReader, Read};
use std::path::Path;

use mshio::MshParserError;

/// Relative path to the directory containing the test mesh data
static TEST_DATA_DIR: &'static str = "tests/data";

/// Reads a whole test mesh file from the data directory as a vector of bytes
pub fn read_test_mesh<P: AsRef<Path>>(filename: P) -> Vec<u8> {
    read_bytes(Path::join(TEST_DATA_DIR.as_ref(), filename.as_ref()))
}

/// Reads a whole file as a vector of bytes
pub fn read_bytes<P: AsRef<Path>>(filepath: P) -> Vec<u8> {
    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(filepath.as_ref())
        .unwrap();
    let mut buf_reader = BufReader::new(file);

    let mut data = Vec::new();
    buf_reader.read_to_end(&mut data).unwrap();
    data
}

/// Returns whether the supplied data can be parsed successfully as a MSH file
pub fn msh_parses(msh: &[u8]) -> bool {
    match mshio::parse_msh_bytes(msh) {
        Ok(_) => true,
        Err(err) => {
            print_error_report(&err);
            false
        }
    }
}

pub fn print_error_report(error: &MshParserError<&[u8]>) {
    eprintln!("Test error:\n{}", error);
    eprintln!("Error debug: {:?}", error);
}

/// Wraps print statements around an expression that inform the user that error output is intended
#[macro_export]
macro_rules! intended_error_output {
    ($test_name:ident, $error_expr:expr) => {
        eprintln!("--- Start of intentionally provoked error output ({}) ---", stringify!($test_name));
        {$error_expr};
        eprintln!("--- End of intentionally provoked error output ({}) ---", stringify!($test_name));
        println!("")
    };
}
