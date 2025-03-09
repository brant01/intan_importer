# Intan Importer

[![Crates.io](https://img.shields.io/crates/v/intan_importer.svg)](https://crates.io/crates/intan_importer)
[![Documentation](https://docs.rs/intan_importer/badge.svg)](https://docs.rs/intan_importer)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust library for importing and processing Intan Technologies RHS data files. This crate provides a fast, strongly-typed interface for working with neural recording data from Intan's electrophysiology systems.

## Features

- Read and parse RHS file format (Intan Recording Controller)
- Extract complete header information including:
  - Version information
  - Sample rates
  - Bandwidth parameters
  - Channel configuration
  - Notes and experimental settings
- Load and process recorded data:
  - Amplifier signals (conversion to μV)
  - DC amplifier data
  - Stimulation data
  - ADC/DAC signals
  - Digital inputs/outputs
- Automatic processing:
  - Signal scaling to appropriate units
  - Notch filter application
  - Timestamp handling
- Strong Rust types for all data structures

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
intan_importer = "0.1.0"
```

## Basic Usage

```rust
use intan_importer::load;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Load an RHS file
    let rhs_file = load("path/to/your/file.rhs")?;
    
    // Access header information
    println!("Sample rate: {} Hz", rhs_file.header.sample_rate);
    println!("Found {} amplifier channels", rhs_file.header.amplifier_channels.len());
    
    // Access the data (if present)
    if rhs_file.data_present {
        if let Some(data) = &rhs_file.data {
            // Get the first channel's data
            if let Some(amp_data) = &data.amplifier_data {
                if amp_data.shape()[0] > 0 {
                    println!("First sample of first channel: {} μV", amp_data[[0, 0]]);
                }
            }
        }
    }
    
    Ok(())
}
```

See the `examples` directory for more comprehensive usage examples.

## Python Integration

This crate is designed to be easily wrapped for Python usage. A companion Python package is available that provides a Pythonic interface to this Rust implementation.

## Performance

The library is optimized for performance:
- Uses memory-efficient data structures
- Employs buffered reading for fast file I/O
- Provides direct access to data without unnecessary copies

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [Intan Technologies](https://intantech.com/) for their neurophysiology recording systems and file format documentation.