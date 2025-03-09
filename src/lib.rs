/*!
# Intan Importer

A Rust library for importing and processing Intan Technologies RHS data files. This crate provides
strongly-typed interfaces to Intan's neural recording file formats, used in electrophysiology research.

## Overview

Intan Technologies manufactures hardware for neural recording, including systems that record electrical signals
from the brain and nervous system. These systems generate data files in proprietary formats (RHS/RHD).
This library provides tools to read and process these files, making the data accessible for analysis.

## Main Features

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

// Load RHS file
let result = load("path/to/your/file.rhs");

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

// Re-export types
pub use types::*;

/// Loads an RHS file and returns a struct representation with header information and data.
///
/// This function reads the entire file, including both header information and recorded data.
/// For large files, this can require significant memory. The returned `RhsFile` struct contains
/// all information from the file in a strongly-typed representation.
///
/// # Parameters
///
/// * `file_path` - Path to the RHS file to load
///
/// # Returns
///
/// * `Result<RhsFile, Box<dyn Error>>` - Either the loaded file data or an error
///
/// # Examples
///
/// ```no_run
/// use intan_importer::load;
///
/// // Load an RHS file
/// let result = load("path/to/your/file.rhs");
/// 
/// match result {
///     Ok(rhs_file) => {
///         // Print basic information
///         println!("Sample rate: {} Hz", rhs_file.header.sample_rate);
///         println!("Channels: {}", rhs_file.header.amplifier_channels.len());
///         
///         // Access recording data if present
///         if rhs_file.data_present {
///             if let Some(data) = &rhs_file.data {
///                 // Example: Calculate recording duration
///                 let num_samples = data.timestamps.len();
///                 let duration = num_samples as f32 / rhs_file.header.sample_rate;
///                 println!("Recording duration: {:.2} seconds", duration);
///             }
///         }
///     },
///     Err(e) => println!("Error loading file: {}", e),
/// }
/// ```
///
/// # Performance Considerations
///
/// For large files, the entire dataset is loaded into memory. Be aware of memory usage
/// when dealing with lengthy recordings.
pub fn load<P: AsRef<Path>>(file_path: P) -> Result<RhsFile, Box<dyn Error>> {
    reader::load_file(file_path)
}
