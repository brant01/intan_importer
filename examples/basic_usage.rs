use intan_importer::load;
use ndarray::s;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Load RHS file
    let rhs_file = load("data/20231220_IC_TEST_B_231220_160314.rhs")?;

    // Print basic file information
    println!(
        "File version: {}.{}",
        rhs_file.header.version.major, rhs_file.header.version.minor
    );
    println!("Sample rate: {} Hz", rhs_file.header.sample_rate);

    // Print notes if any
    if !rhs_file.header.notes.note1.is_empty() {
        println!("Note 1: {}", rhs_file.header.notes.note1);
    }

    if !rhs_file.header.notes.note2.is_empty() {
        println!("Note 2: {}", rhs_file.header.notes.note2);
    }

    if !rhs_file.header.notes.note3.is_empty() {
        println!("Note 3: {}", rhs_file.header.notes.note3);
    }

    // Print channel information
    println!(
        "Number of amplifier channels: {}",
        rhs_file.header.amplifier_channels.len()
    );
    println!(
        "Number of ADC channels: {}",
        rhs_file.header.board_adc_channels.len()
    );
    println!(
        "Number of DAC channels: {}",
        rhs_file.header.board_dac_channels.len()
    );
    println!(
        "Number of digital input channels: {}",
        rhs_file.header.board_dig_in_channels.len()
    );
    println!(
        "Number of digital output channels: {}",
        rhs_file.header.board_dig_out_channels.len()
    );

    // List first few amplifier channels
    if !rhs_file.header.amplifier_channels.is_empty() {
        println!("\nAmplifier channels:");
        for (i, channel) in rhs_file
            .header
            .amplifier_channels
            .iter()
            .enumerate()
            .take(5)
        {
            println!(
                "  {}: {} ({})",
                i, channel.custom_channel_name, channel.native_channel_name
            );
        }

        if rhs_file.header.amplifier_channels.len() > 5 {
            println!(
                "  ... and {} more",
                rhs_file.header.amplifier_channels.len() - 5
            );
        }
    }

    // Check if data is present and print summary
    if rhs_file.data_present {
        if let Some(data) = &rhs_file.data {
            println!("\nData summary:");
            println!("  Number of time samples: {}", data.timestamps.len());

            // Print time range
            if data.timestamps.len() > 1 {
                let start_time = data.timestamps[0] as f32 / rhs_file.header.sample_rate;
                let end_time =
                    data.timestamps[data.timestamps.len() - 1] as f32 / rhs_file.header.sample_rate;
                println!("  Time range: {:.3} to {:.3} seconds", start_time, end_time);
                println!("  Duration: {:.3} seconds", end_time - start_time);
            }

            // Summarize amplifier data if present
            if let Some(amp_data) = &data.amplifier_data {
                let shape = amp_data.shape();
                println!(
                    "  Amplifier data: {} channels x {} samples",
                    shape[0], shape[1]
                );

                // Show first few samples of first channel
                if shape[0] > 0 && shape[1] > 0 {
                    let channel_data = amp_data.slice(s![0, ..]);
                    let num_samples = std::cmp::min(5, shape[1]);

                    println!("  First channel data (first {} samples):", num_samples);
                    for i in 0..num_samples {
                        println!("    {}: {} μV", i, channel_data[i]);
                    }
                }
            }
        }
    } else {
        println!("\nNo data present in file (header only).");
    }

    Ok(())
}
