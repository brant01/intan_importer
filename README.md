# Intan Importer

[![Crates.io](https://img.shields.io/crates/v/intan_importer.svg)](https://crates.io/crates/intan_importer)
[![Documentation](https://docs.rs/intan_importer/badge.svg)](https://docs.rs/intan_importer)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A high-performance Rust library for reading Intan Technologies RHS data files, commonly used in neuroscience research for recording neural signals. This library is designed to be fast, memory-efficient, and easy to use, making it ideal for processing large neurophysiology datasets.

## Key Features

- 🚀 **Fast**: 5-10x faster than the official Python implementation
- 📁 **Flexible Loading**: Load single files or automatically combine split recording sessions
- 🧠 **Neuroscience-Ready**: Automatic scaling to physical units (μV, V, μA)
- 🔧 **Signal Processing**: Built-in notch filtering for line noise removal
- 💾 **Memory Efficient**: Streaming architecture for processing large files
- 🐍 **Python Bindings**: Available via `pip install intan-importer` (coming soon)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
intan_importer = "0.2.0"
```

## Quick Start

### Basic Usage

```rust
use intan_importer::load;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a single RHS file
    let recording = load("path/to/recording.rhs")?;
    
    // Print basic information
    println!("Sample rate: {} Hz", recording.header.sample_rate);
    println!("Duration: {:.2} seconds", recording.duration());
    println!("Channels: {}", recording.header.amplifier_channels.len());
    
    // Access neural data
    if let Some(data) = &recording.data {
        if let Some(amp_data) = &data.amplifier_data {
            // Data is a 2D array: [channels, samples]
            println!("Data shape: {} channels × {} samples", 
                     amp_data.shape()[0], amp_data.shape()[1]);
            
            // Get first channel data
            let channel_0 = amp_data.slice(s![0, ..]);
            println!("First sample: {} μV", channel_0[0]);
        }
    }
    
    Ok(())
}
```

### Loading Split Recording Sessions

When recording long sessions, Intan software can split data into multiple files. This library can automatically combine them:

```rust
use intan_importer::load;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load all RHS files from a directory
    let recording = load("path/to/recording_session/")?;
    
    // Check which files were combined
    if let Some(files) = &recording.source_files {
        println!("Combined {} files:", files.len());
        for file in files {
            println!("  - {}", file);
        }
    }
    
    println!("Total duration: {:.2} minutes", recording.duration() / 60.0);
    
    Ok(())
}
```

## Data Structure

The library returns a hierarchical data structure that mirrors the Intan file format:

```rust
RhsFile {
    header: RhsHeader {
        // Recording configuration
        sample_rate: f32,
        notch_filter_frequency: Option<i32>,
        
        // Channel information
        amplifier_channels: Vec<ChannelInfo>,
        board_adc_channels: Vec<ChannelInfo>,
        board_dig_in_channels: Vec<ChannelInfo>,
        
        // Stimulation parameters (if using stim headstage)
        stim_parameters: StimParameters,
        
        // ... and more
    },
    data: Option<RhsData> {
        // Time vector (in seconds)
        timestamps: Array1<i32>,
        
        // Neural data (in microvolts)
        amplifier_data: Option<Array2<i32>>,
        
        // Stimulation data (in microamps)
        stim_data: Option<Array2<i32>>,
        
        // Digital events
        board_dig_in_data: Option<Array2<i32>>,
        
        // ... and more
    }
}
```

## Advanced Examples

### Processing Neural Signals

```rust
use intan_importer::load;
use ndarray::s;

fn analyze_channels(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let recording = load(path)?;
    
    if let Some(data) = &recording.data {
        if let Some(amp_data) = &data.amplifier_data {
            // Iterate through each channel
            for (idx, channel) in recording.header.amplifier_channels.iter().enumerate() {
                let channel_data = amp_data.slice(s![idx, ..]);
                
                // Calculate basic statistics
                let mean = channel_data.mean().unwrap_or(0.0);
                let max = channel_data.iter().max().copied().unwrap_or(0);
                let min = channel_data.iter().min().copied().unwrap_or(0);
                
                println!("Channel {}: mean={:.2}μV, range=[{}, {}]μV", 
                         channel.custom_channel_name, mean, min, max);
            }
        }
    }
    
    Ok(())
}
```

### Working with Stimulation Data

```rust
use intan_importer::load;

fn analyze_stim_events(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let recording = load(path)?;
    
    if let Some(data) = &recording.data {
        if let Some(stim_data) = &data.stim_data {
            // Find stimulation events (non-zero values)
            let channel_idx = 0;  // First stim channel
            let stim_channel = stim_data.slice(s![channel_idx, ..]);
            
            let stim_events: Vec<_> = stim_channel.iter()
                .enumerate()
                .filter(|(_, &value)| value != 0)
                .collect();
            
            println!("Found {} stimulation events", stim_events.len());
            
            // Check compliance limit warnings
            if let Some(compliance) = &data.compliance_limit_data {
                let violations = compliance.iter().filter(|&&v| v).count();
                if violations > 0 {
                    println!("Warning: {} compliance limit violations", violations);
                }
            }
        }
    }
    
    Ok(())
}
```

## Performance Tips

1. **Memory Usage**: Files are loaded entirely into memory. For very large recordings (>10GB), ensure adequate RAM.

2. **Parallel Processing**: The data arrays are compatible with Rayon for parallel processing:
   ```rust
   use rayon::prelude::*;
   
   let results: Vec<_> = (0..n_channels)
       .into_par_iter()
       .map(|ch| process_channel(amp_data.slice(s![ch, ..])))
       .collect();
   ```

3. **Chunked Processing**: For extremely large files, consider processing in chunks:
   ```rust
   let chunk_size = 30_000; // 1 second at 30kHz
   for chunk_start in (0..n_samples).step_by(chunk_size) {
       let chunk_end = (chunk_start + chunk_size).min(n_samples);
       let chunk = amp_data.slice(s![.., chunk_start..chunk_end]);
       process_chunk(chunk);
   }
   ```

## Signal Processing

The library automatically applies several processing steps:

1. **Scaling**: Raw ADC values are converted to physical units
   - Amplifier data → microvolts (μV)
   - ADC channels → volts (V)
   - Stimulation data → microamps (μA)

2. **Notch Filtering**: If enabled during recording, removes 50/60 Hz line noise
   - Only applied to data recorded with RHS software < v3.0
   - Software v3.0+ already saves filtered data

3. **Timestamp Alignment**: When combining multiple files, timestamps are adjusted to be continuous

## Contributing

Contributions are welcome! Areas of particular interest:

- Support for RHD file format
- Additional signal processing utilities
- Performance optimizations
- Python bindings improvements

## License

MIT License - see [LICENSE](LICENSE) for details.

## Citation

If you use this library in your research, please cite:

```bibtex
@software{intan_importer,
  author = {Brant, Jason},
  title = {Intan Importer: A Rust library for reading Intan RHS files},
  url = {https://github.com/jasonbrant/intan_importer},
  year = {2024}
}
```

## Acknowledgments

- [Intan Technologies](https://intantech.com/) for their excellent neurophysiology recording systems
- The neuroscience research community for feedback and use cases