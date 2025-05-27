// examples/test_loader.rs
use intan_importer::load;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_rhs_file_or_directory>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    println!("Loading from: {}", path);

    match load(path) {
        Ok(file) => {
            println!("\n✓ Successfully loaded!");
            println!("  Sample rate: {} Hz", file.header.sample_rate);
            println!("  Channels: {}", file.header.amplifier_channels.len());
            println!("  Duration: {:.2} seconds", file.duration());
            
            if let Some(sources) = &file.source_files {
                println!("  Source files: {}", sources.len());
                for (i, source) in sources.iter().enumerate() {
                    println!("    {}: {}", i + 1, source);
                }
            }
            
            if let Some(data) = &file.data {
                if let Some(amp_data) = &data.amplifier_data {
                    println!("  Data shape: {} channels × {} samples", 
                             amp_data.shape()[0], amp_data.shape()[1]);
                }
            }
        },
        Err(e) => {
            eprintln!("\n✗ Error loading file: {}", e);
            std::process::exit(1);
        }
    }
}