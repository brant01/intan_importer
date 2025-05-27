/*!
# Intan Importer

A Rust library for importing and processing Intan Technologies RHS data files. This crate provides
strongly-typed interfaces to Intan's neural recording file formats, used in electrophysiology research.

## Overview

Intan Technologies manufactures hardware for neural recording, including systems that record electrical signals
from the brain and nervous system. These systems generate data files in proprietary formats (RHS/RHD).
This library provides tools to read and process these files, making the data accessible for analysis.

## Main Features

- **Single File or Directory Loading**: Load individual RHS files or automatically combine multiple files from a directory
- **Comprehensive Parsing**: Read and parse Intan RHS files with full support for all sections
- **Strong Typing**: All data structures are strongly typed with appropriate Rust types
- **Automatic Processing**: 
  - Signal scaling to meaningful units (μV, V)
  - Notch filter application for line noise removal
  - Timestamp alignment and verification
- **Efficient Implementation**:
  - Fast binary parsing with minimal allocations
  - Proper error handling with descriptive messages
  - Memory-efficient data structures

## Quick Start

```rust
use intan_importer::load;

// Load single RHS file
let result = load("path/to/your/file.rhs");

// Or load all RHS files from a directory
let result = load("path/to/recording/directory");

match result {
    Ok(rhs_file) => {
        // Access header information
        println!("Sample rate: {} Hz", rhs_file.header.sample_rate);
        println!("Number of channels: {}", rhs_file.header.amplifier_channels.len());
        
        // Access recording data if present
        if rhs_file.data_present {
            if let Some(data) = &rhs_file.data {
                if let Some(amp_data) = &data.amplifier_data {
                    // Process amplifier data
                    println!("Data dimensions: {:?}", amp_data.shape());
                }
            }
        }
    },
    Err(e) => println!("Error loading file: {}", e),
}
```

## Data Structure

The crate organizes Intan data into a hierarchy of structs:

- `RhsFile`: Top-level container with header and data
  - `RhsHeader`: Configuration, channel info, and recording parameters
    - `Version`, `Notes`, `FrequencyParameters`, etc.
    - Lists of channels: `amplifier_channels`, `board_adc_channels`, etc.
  - `RhsData`: Actual recorded signals
    - `timestamps`, `amplifier_data`, `stim_data`, etc.

## Error Handling

The library provides descriptive errors for various failure scenarios (file format errors,
I/O failures, etc.) through the `IntanError` type.
*/

mod reader;
pub mod types;

use std::error::Error;
use std::path::Path;
use std::fs;

// Re-export types
pub use types::*;

/// Loads RHS data from a file or directory.
///
/// This function can handle two input types:
/// 1. **Single file**: Loads a single RHS file
/// 2. **Directory**: Loads and combines all RHS files in the directory
///
/// When loading from a directory, all RHS files are combined chronologically into
/// a single dataset. Files must have compatible headers (same channels, sample rates, etc.).
///
/// # Parameters
///
/// * `path` - Path to an RHS file or a directory containing RHS files
///
/// # Returns
///
/// * `Result<RhsFile, Box<dyn Error>>` - Either the loaded file data or an error
///
/// # Examples
///
/// ## Loading a single file
/// ```no_run
/// use intan_importer::load;
///
/// let result = load("recording.rhs");
/// match result {
///     Ok(rhs_file) => {
///         println!("Loaded {} seconds of data", rhs_file.duration());
///     },
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
///
/// ## Loading from a directory
/// ```no_run
/// use intan_importer::load;
///
/// // Load all RHS files from a recording session split across multiple files
/// let result = load("recording_session/");
/// match result {
///     Ok(rhs_file) => {
///         println!("Combined {} files", rhs_file.source_files.as_ref().unwrap().len());
///         println!("Total duration: {} seconds", rhs_file.duration());
///     },
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
///
/// # Performance Considerations
///
/// When loading multiple files, the entire combined dataset is loaded into memory.
/// Be aware of memory usage when dealing with lengthy recording sessions.
pub fn load<P: AsRef<Path>>(path: P) -> Result<RhsFile, Box<dyn Error>> {
    let path = path.as_ref();
    
    if path.is_file() {
        // Load single file
        reader::load_file(path)
    } else if path.is_dir() {
        // Load and combine all RHS files in directory
        load_directory(path)
    } else {
        Err(Box::new(IntanError::Other(format!(
            "Path '{}' is neither a file nor a directory",
            path.display()
        ))))
    }
}

/// Loads and combines all RHS files from a directory
fn load_directory<P: AsRef<Path>>(dir_path: P) -> Result<RhsFile, Box<dyn Error>> {
    let dir_path = dir_path.as_ref();
    
    // Find all .rhs files in the directory
    let mut rhs_files: Vec<_> = fs::read_dir(dir_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("rhs"))
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .collect();
    
    if rhs_files.is_empty() {
        return Err(Box::new(IntanError::Other(
            "No RHS files found in directory".to_string()
        )));
    }
    
    // Sort files by name to ensure consistent ordering
    rhs_files.sort();
    
    println!("Found {} RHS files to combine:", rhs_files.len());
    for file in &rhs_files {
        println!("  - {}", file.display());
    }
    
    // Load and combine the files
    reader::load_and_combine_files(&rhs_files)
}