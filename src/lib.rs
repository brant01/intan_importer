mod reader;
pub mod types;

use std::error::Error;
use std::path::Path;

// Re-export types
pub use types::*;

/// Loads an RHS file and returns a struct representation
///
/// # Examples
///
/// ```no_run
/// use intan_importer::load;
///
/// let result = load("path/to/your/file.rhs");
/// match result {
///     Ok(rhs_file) => println!("Sample rate: {} Hz", rhs_file.header.sample_rate),
///     Err(e) => println!("Error loading file: {}", e),
/// }
/// ```
pub fn load<P: AsRef<Path>>(file_path: P) -> Result<RhsFile, Box<dyn Error>> {
    reader::load_file(file_path)
}
