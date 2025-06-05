use byteorder::{LittleEndian, ReadBytesExt};
use ndarray::{Array1, Array2, s};
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Instant;

use crate::types::*;

// Constants used throughout the reader
const RHS_MAGIC_NUMBER: u32 = 0xd69127ac;
const SAMPLES_PER_DATA_BLOCK: usize = 128;
const PRINT_PROGRESS_STEP: usize = 10;

// Scaling constants (from Intan RHS data format specification)
const AMPLIFIER_SCALE_FACTOR: f64 = 0.195; // μV per bit
const DC_AMPLIFIER_SCALE_FACTOR: f64 = 19.23; // mV per bit (note: positive, not negative)
const ADC_DAC_SCALE_FACTOR: f64 = 0.0003125; // V per bit (312.5 μV = 0.0003125 V)
const DC_AMPLIFIER_OFFSET: f64 = 512.0;
const ADC_DAC_OFFSET: f64 = 32768.0;

/// Loads an RHS file and returns a strongly-typed struct representation.
///
/// This function reads and parses an Intan RHS file, extracting both the header
/// information and the actual recorded data. For large files, this can consume
/// significant memory.
///
/// # Arguments
///
/// * `file_path` - Path to the RHS file to load
///
/// # Returns
///
/// A `Result` containing either the loaded `RhsFile` or an error.
///
/// # Performance
///
/// This function uses buffered I/O for improved reading performance. The parsing
/// process will report progress for large files.
pub fn load_file<P: AsRef<Path>>(file_path: P) -> Result<RhsFile, Box<dyn std::error::Error>> {
    // Start timing
    let tic = Instant::now();

    // Open file with buffered reader for better I/O performance
    let file = File::open(file_path.as_ref())?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::with_capacity(65536, file); // 64KB buffer

    // Read header
    let header = read_header(&mut reader)?;

    // Calculate how much data is present
    let (data_present, num_blocks, num_samples) =
        calculate_data_size(&header, file_size, &mut reader)?;

    // Read data if present
    let data = if data_present {
        let data = read_all_data_blocks(&header, num_samples, num_blocks, &mut reader)?;
        check_end_of_file(file_size, &mut reader)?;

        // Apply processing to the data
        let data = process_data(&header, data)?;
        Some(data)
    } else {
        None
    };

    // Report how long read took
    println!(
        "Done! Elapsed time: {:.1} seconds",
        tic.elapsed().as_secs_f64()
    );

    // Return the complete RHS file
    Ok(RhsFile {
        header,
        data,
        data_present,
        source_files: None,  // Add this line
    })
}

/// Reads the header from an RHS file
fn read_header<R: Read + Seek>(reader: &mut R) -> Result<RhsHeader, Box<dyn std::error::Error>> {
    // Create header with default values for RHS format
    let mut header = RhsHeader {
        version: Version { major: 0, minor: 0 },
        sample_rate: 0.0,
        num_samples_per_data_block: SAMPLES_PER_DATA_BLOCK as i32,
        dsp_enabled: 0,
        actual_dsp_cutoff_frequency: 0.0,
        actual_lower_bandwidth: 0.0,
        actual_lower_settle_bandwidth: 0.0,
        actual_upper_bandwidth: 0.0,
        desired_dsp_cutoff_frequency: 0.0,
        desired_lower_bandwidth: 0.0,
        desired_lower_settle_bandwidth: 0.0,
        desired_upper_bandwidth: 0.0,
        notch_filter_frequency: None,
        desired_impedance_test_frequency: 0.0,
        actual_impedance_test_frequency: 0.0,
        amp_settle_mode: 0,
        charge_recovery_mode: 0,
        stim_step_size: 0.0,
        recovery_current_limit: 0.0,
        recovery_target_voltage: 0.0,
        notes: Notes {
            note1: String::new(),
            note2: String::new(),
            note3: String::new(),
        },
        dc_amplifier_data_saved: false,
        eval_board_mode: 0,
        reference_channel: String::new(),
        amplifier_channels: Vec::new(),
        spike_triggers: Vec::new(),
        board_adc_channels: Vec::new(),
        board_dac_channels: Vec::new(),
        board_dig_in_channels: Vec::new(),
        board_dig_out_channels: Vec::new(),
        frequency_parameters: FrequencyParameters {
            amplifier_sample_rate: 0.0,
            board_adc_sample_rate: 0.0,
            board_dig_in_sample_rate: 0.0,
            desired_dsp_cutoff_frequency: 0.0,
            actual_dsp_cutoff_frequency: 0.0,
            dsp_enabled: 0,
            desired_lower_bandwidth: 0.0,
            desired_lower_settle_bandwidth: 0.0,
            actual_lower_bandwidth: 0.0,
            actual_lower_settle_bandwidth: 0.0,
            desired_upper_bandwidth: 0.0,
            actual_upper_bandwidth: 0.0,
            notch_filter_frequency: None,
            desired_impedance_test_frequency: 0.0,
            actual_impedance_test_frequency: 0.0,
        },
        stim_parameters: StimParameters {
            stim_step_size: 0.0,
            charge_recovery_current_limit: 0.0,
            charge_recovery_target_voltage: 0.0,
            amp_settle_mode: 0,
            charge_recovery_mode: 0,
        },
    };

    // Check magic number
    check_magic_number(reader)?;

    // Read version number
    read_version_number(reader, &mut header)?;

    // Read sample rate
    header.sample_rate = reader.read_f32::<LittleEndian>()?;
    header.frequency_parameters.amplifier_sample_rate = header.sample_rate;
    header.frequency_parameters.board_adc_sample_rate = header.sample_rate;
    header.frequency_parameters.board_dig_in_sample_rate = header.sample_rate;

    // Read frequency settings
    read_freq_settings(reader, &mut header)?;

    // Read notch filter
    read_notch_filter_frequency(reader, &mut header)?;

    // Read impedance test frequencies
    read_impedance_test_frequencies(reader, &mut header)?;

    // Read amp settle mode
    header.amp_settle_mode = reader.read_i16::<LittleEndian>()? as i32;
    header.stim_parameters.amp_settle_mode = header.amp_settle_mode;

    // Read charge recovery mode
    header.charge_recovery_mode = reader.read_i16::<LittleEndian>()? as i32;
    header.stim_parameters.charge_recovery_mode = header.charge_recovery_mode;

    // Read stim step size
    header.stim_step_size = reader.read_f32::<LittleEndian>()?;
    header.stim_parameters.stim_step_size = header.stim_step_size;

    // Read recovery current limit
    header.recovery_current_limit = reader.read_f32::<LittleEndian>()?;
    header.stim_parameters.charge_recovery_current_limit = header.recovery_current_limit;

    // Read recovery target voltage
    header.recovery_target_voltage = reader.read_f32::<LittleEndian>()?;
    header.stim_parameters.charge_recovery_target_voltage = header.recovery_target_voltage;

    // Read notes
    read_notes(reader, &mut header)?;

    // Read DC amp saved flag
    header.dc_amplifier_data_saved = reader.read_i16::<LittleEndian>()? != 0;

    // Read eval board mode
    header.eval_board_mode = reader.read_i16::<LittleEndian>()? as i32;

    // Read reference channel
    header.reference_channel = read_qstring(reader)?;

    // Read signal summary
    read_signal_summary(reader, &mut header)?;

    // Print header summary
    print_header_summary(&header);

    Ok(header)
}

/// Helper function to check the magic number that identifies RHS files
fn check_magic_number<R: Read>(reader: &mut R) -> Result<(), IntanError> {
    let magic_number = reader.read_u32::<LittleEndian>()?;
    if magic_number != RHS_MAGIC_NUMBER {
        return Err(IntanError::UnrecognizedFileFormat);
    }
    Ok(())
}

/// Helper function to read the version number
fn read_version_number<R: Read>(reader: &mut R, header: &mut RhsHeader) -> Result<(), IntanError> {
    let mut version_bytes = [0u8; 4];
    reader.read_exact(&mut version_bytes)?;

    header.version.major = i16::from_le_bytes([version_bytes[0], version_bytes[1]]) as i32;
    header.version.minor = i16::from_le_bytes([version_bytes[2], version_bytes[3]]) as i32;

    println!(
        "\nReading Intan Technologies RHS Data File, Version {}.{}\n",
        header.version.major, header.version.minor
    );

    Ok(())
}

/// Helper function to read frequency settings
fn read_freq_settings<R: Read>(reader: &mut R, header: &mut RhsHeader) -> Result<(), IntanError> {
    // Read DSP enabled flag
    header.dsp_enabled = reader.read_i16::<LittleEndian>()? as i32;
    header.frequency_parameters.dsp_enabled = header.dsp_enabled;

    // Read actual DSP cutoff frequency
    header.actual_dsp_cutoff_frequency = reader.read_f32::<LittleEndian>()?;
    header.frequency_parameters.actual_dsp_cutoff_frequency = header.actual_dsp_cutoff_frequency;

    // Read actual lower bandwidth
    header.actual_lower_bandwidth = reader.read_f32::<LittleEndian>()?;
    header.frequency_parameters.actual_lower_bandwidth = header.actual_lower_bandwidth;

    // Read actual lower settle bandwidth
    header.actual_lower_settle_bandwidth = reader.read_f32::<LittleEndian>()?;
    header.frequency_parameters.actual_lower_settle_bandwidth =
        header.actual_lower_settle_bandwidth;

    // Read actual upper bandwidth
    header.actual_upper_bandwidth = reader.read_f32::<LittleEndian>()?;
    header.frequency_parameters.actual_upper_bandwidth = header.actual_upper_bandwidth;

    // Read desired DSP cutoff frequency
    header.desired_dsp_cutoff_frequency = reader.read_f32::<LittleEndian>()?;
    header.frequency_parameters.desired_dsp_cutoff_frequency = header.desired_dsp_cutoff_frequency;

    // Read desired lower bandwidth
    header.desired_lower_bandwidth = reader.read_f32::<LittleEndian>()?;
    header.frequency_parameters.desired_lower_bandwidth = header.desired_lower_bandwidth;

    // Read desired lower settle bandwidth
    header.desired_lower_settle_bandwidth = reader.read_f32::<LittleEndian>()?;
    header.frequency_parameters.desired_lower_settle_bandwidth =
        header.desired_lower_settle_bandwidth;

    // Read desired upper bandwidth
    header.desired_upper_bandwidth = reader.read_f32::<LittleEndian>()?;
    header.frequency_parameters.desired_upper_bandwidth = header.desired_upper_bandwidth;

    Ok(())
}

/// Helper function to read notch filter frequency
fn read_notch_filter_frequency<R: Read>(reader: &mut R, header: &mut RhsHeader) -> Result<(), IntanError> {
    let notch_filter_mode = reader.read_i16::<LittleEndian>()? as i32;

    header.notch_filter_frequency = match notch_filter_mode {
        1 => Some(50),
        2 => Some(60),
        _ => None,
    };

    header.frequency_parameters.notch_filter_frequency = header.notch_filter_frequency;

    Ok(())
}

/// Helper function to read impedance test frequencies
fn read_impedance_test_frequencies<R: Read>(
    reader: &mut R,
    header: &mut RhsHeader,
) -> Result<(), IntanError> {
    header.desired_impedance_test_frequency = reader.read_f32::<LittleEndian>()?;
    header.actual_impedance_test_frequency = reader.read_f32::<LittleEndian>()?;

    header.frequency_parameters.desired_impedance_test_frequency =
        header.desired_impedance_test_frequency;
    header.frequency_parameters.actual_impedance_test_frequency =
        header.actual_impedance_test_frequency;

    Ok(())
}

/// Helper function to read notes
fn read_notes<R: Read + Seek>(reader: &mut R, header: &mut RhsHeader) -> Result<(), IntanError> {
    header.notes.note1 = read_qstring(reader)?;
    header.notes.note2 = read_qstring(reader)?;
    header.notes.note3 = read_qstring(reader)?;

    Ok(())
}

/// Helper function to read signal summary
fn read_signal_summary<R: Read + Seek>(reader: &mut R, header: &mut RhsHeader) -> Result<(), IntanError> {
    let number_of_signal_groups = reader.read_i16::<LittleEndian>()?;

    for _ in 1..=number_of_signal_groups {
        add_signal_group_information(header, reader)?;
    }

    Ok(())
}

/// Helper function to add signal group information
fn add_signal_group_information<R: Read + Seek>(header: &mut RhsHeader, reader: &mut R) -> Result<(), IntanError> {
    let signal_group_name = read_qstring(reader)?;
    let signal_group_prefix = read_qstring(reader)?;

    let signal_group_enabled = reader.read_i16::<LittleEndian>()?;
    let signal_group_num_channels = reader.read_i16::<LittleEndian>()?;
    let _ = reader.read_i16::<LittleEndian>()?; // signal_group_num_channels (unused)

    if signal_group_num_channels > 0 && signal_group_enabled > 0 {
        for _ in 0..signal_group_num_channels {
            add_channel_information(header, reader, &signal_group_name, &signal_group_prefix)?;
        }
    }

    Ok(())
}

/// Helper function to add channel information
fn add_channel_information<R: Read + Seek>(
    header: &mut RhsHeader,
    reader: &mut R,
    signal_group_name: &str,
    signal_group_prefix: &str,
) -> Result<(), IntanError> {
    // Create new channel
    let mut new_channel = ChannelInfo {
        port_name: signal_group_name.to_string(),
        port_prefix: signal_group_prefix.to_string(),
        port_number: 0,
        native_channel_name: String::new(),
        custom_channel_name: String::new(),
        native_order: 0,
        custom_order: 0,
        chip_channel: 0,
        board_stream: 0,
        electrode_impedance_magnitude: 0.0,
        electrode_impedance_phase: 0.0,
    };

    // Create new trigger channel
    let mut new_trigger = SpikeTrigger {
        voltage_trigger_mode: 0,
        voltage_threshold: 0,
        digital_trigger_channel: 0,
        digital_edge_polarity: 0,
    };

    // Read channel information
    new_channel.native_channel_name = read_qstring(reader)?;
    new_channel.custom_channel_name = read_qstring(reader)?;

    new_channel.native_order = reader.read_i16::<LittleEndian>()? as i32;
    new_channel.custom_order = reader.read_i16::<LittleEndian>()? as i32;

    let signal_type = reader.read_i16::<LittleEndian>()? as i32;
    let channel_enabled = reader.read_i16::<LittleEndian>()? as i32;

    new_channel.chip_channel = reader.read_i16::<LittleEndian>()? as i32;
    let _ = reader.read_i16::<LittleEndian>()?; // reserved
    new_channel.board_stream = reader.read_i16::<LittleEndian>()? as i32;

    // Read trigger information
    new_trigger.voltage_trigger_mode = reader.read_i16::<LittleEndian>()? as i32;
    new_trigger.voltage_threshold = reader.read_i16::<LittleEndian>()? as i32;
    new_trigger.digital_trigger_channel = reader.read_i16::<LittleEndian>()? as i32;
    new_trigger.digital_edge_polarity = reader.read_i16::<LittleEndian>()? as i32;

    // Read impedance information
    new_channel.electrode_impedance_magnitude = reader.read_f32::<LittleEndian>()?;
    new_channel.electrode_impedance_phase = reader.read_f32::<LittleEndian>()?;

    // If channel is enabled, add it to the appropriate list
    if channel_enabled == 0 {
        return Ok(());
    }

    match signal_type {
        0 => {
            header.amplifier_channels.push(new_channel);
            header.spike_triggers.push(new_trigger);
        }
        1 => return Err(IntanError::InvalidChannelType), // AuxInputSignals
        2 => return Err(IntanError::InvalidChannelType), // VddSignals
        3 => header.board_adc_channels.push(new_channel),
        4 => header.board_dac_channels.push(new_channel),
        5 => header.board_dig_in_channels.push(new_channel),
        6 => header.board_dig_out_channels.push(new_channel),
        _ => return Err(IntanError::InvalidChannelType),
    }

    Ok(())
}

// Helper function to print header summary
fn print_header_summary(header: &RhsHeader) {
    println!(
        "Found {} amplifier channel{}.",
        header.amplifier_channels.len(),
        if header.amplifier_channels.len() != 1 {
            "s"
        } else {
            ""
        }
    );

    if header.dc_amplifier_data_saved {
        println!(
            "Found {} DC amplifier channel{}.",
            header.amplifier_channels.len(),
            if header.amplifier_channels.len() != 1 {
                "s"
            } else {
                ""
            }
        );
    }

    println!(
        "Found {} board ADC channel{}.",
        header.board_adc_channels.len(),
        if header.board_adc_channels.len() != 1 {
            "s"
        } else {
            ""
        }
    );

    println!(
        "Found {} board DAC channel{}.",
        header.board_dac_channels.len(),
        if header.board_dac_channels.len() != 1 {
            "s"
        } else {
            ""
        }
    );

    println!(
        "Found {} board digital input channel{}.",
        header.board_dig_in_channels.len(),
        if header.board_dig_in_channels.len() != 1 {
            "s"
        } else {
            ""
        }
    );

    println!(
        "Found {} board digital output channel{}.",
        header.board_dig_out_channels.len(),
        if header.board_dig_out_channels.len() != 1 {
            "s"
        } else {
            ""
        }
    );

    println!();
}

/// Helper function to read a QString (UTF-16 encoded string)
///
/// QtStrings in RHS files are stored as UTF-16 with a 4-byte length prefix.
/// A special value of 0xFFFFFFFF indicates an empty string.
fn read_qstring<R: Read + Seek>(reader: &mut R) -> Result<String, IntanError> {
    let length = reader.read_u32::<LittleEndian>()?;

    // If length set to 0xFFFFFFFF, return empty string
    if length == 0xFFFFFFFF {
        return Ok(String::new());
    }

    // Verify that the string length is reasonable given remaining file size
    let current_position = reader.stream_position()?;
    let file_length = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(current_position))?;

    if length as u64 > file_length - current_position + 1 {
        return Err(IntanError::StringReadError);
    }

    // Convert length from bytes to 16-bit Unicode words
    let length = (length as usize) / 2;

    // Preallocate for performance
    let mut data = Vec::with_capacity(length);
    for _ in 0..length {
        let c = reader.read_u16::<LittleEndian>()?;
        data.push(c);
    }

    // Create string from UTF-16 characters
    let mut result = String::with_capacity(length);
    for &c in &data {
        match char::from_u32(c as u32) {
            Some(ch) => result.push(ch),
            None => return Err(IntanError::StringReadError),
        }
    }

    Ok(result)
}

/// Calculates how much data is present in the file and returns relevant metrics
///
/// # Arguments
///
/// * `header` - The parsed header information
/// * `file_size` - The total size of the file in bytes
/// * `reader` - The file reader, positioned after the header
///
/// # Returns
///
/// Tuple containing:
/// * `data_present` - Boolean indicating if any data blocks are present
/// * `num_blocks` - Number of data blocks in the file
/// * `num_samples` - Total number of samples in the file
fn calculate_data_size<R: Read + Seek>(
    header: &RhsHeader,
    file_size: u64,
    reader: &mut R,
) -> Result<(bool, u64, u64), Box<dyn std::error::Error>> {
    let bytes_per_block = get_bytes_per_data_block(header)?;

    // Calculate how many bytes remain in the file after the header
    let current_position = reader.stream_position()?;
    let bytes_remaining = file_size - current_position;

    let data_present = bytes_remaining > 0;

    // If the file size is somehow different than expected, raise an error
    if bytes_remaining % bytes_per_block as u64 != 0 {
        return Err(Box::new(IntanError::FileSizeError));
    }

    // Calculate how many data blocks are present
    let num_blocks = bytes_remaining / bytes_per_block as u64;

    let num_samples = num_blocks * header.num_samples_per_data_block as u64;

    print_record_time_summary(num_samples, header.sample_rate, data_present);

    Ok((data_present, num_blocks, num_samples))
}

// Helper function to print record time summary
fn print_record_time_summary(num_amp_samples: u64, sample_rate: f32, data_present: bool) {
    let record_time = num_amp_samples as f32 / sample_rate;

    if data_present {
        println!(
            "File contains {:.3} seconds of data. Amplifiers were sampled at {:.2} kS/s.",
            record_time,
            sample_rate / 1000.0
        );
    } else {
        println!(
            "Header file contains no data. Amplifiers were sampled at {:.2} kS/s.",
            sample_rate / 1000.0
        );
    }
}

// Helper function to get bytes per data block
fn get_bytes_per_data_block(header: &RhsHeader) -> Result<usize, Box<dyn std::error::Error>> {
    // RHS files always have 128 samples per data block
    let num_samples_per_data_block = 128;

    // Timestamps (one channel always present): start with 4 bytes per sample
    let mut bytes_per_block = bytes_per_signal_type(num_samples_per_data_block, 1, 4);

    // Amplifier data: Add 2 bytes per sample per enabled amplifier channel
    bytes_per_block += bytes_per_signal_type(
        num_samples_per_data_block,
        header.amplifier_channels.len(),
        2,
    );

    // DC Amplifier data (absent if flag was off)
    if header.dc_amplifier_data_saved {
        bytes_per_block += bytes_per_signal_type(
            num_samples_per_data_block,
            header.amplifier_channels.len(),
            2,
        );
    }

    // Stimulation data: Add 2 bytes per sample per enabled amplifier channel
    bytes_per_block += bytes_per_signal_type(
        num_samples_per_data_block,
        header.amplifier_channels.len(),
        2,
    );

    // Analog inputs: Add 2 bytes per sample per enabled analog input channel
    bytes_per_block += bytes_per_signal_type(
        num_samples_per_data_block,
        header.board_adc_channels.len(),
        2,
    );

    // Analog outputs: Add 2 bytes per sample per enabled analog output channel
    bytes_per_block += bytes_per_signal_type(
        num_samples_per_data_block,
        header.board_dac_channels.len(),
        2,
    );

    // Digital inputs: Add 2 bytes per sample
    if !header.board_dig_in_channels.is_empty() {
        bytes_per_block += bytes_per_signal_type(num_samples_per_data_block, 1, 2);
    }

    // Digital outputs: Add 2 bytes per sample
    if !header.board_dig_out_channels.is_empty() {
        bytes_per_block += bytes_per_signal_type(num_samples_per_data_block, 1, 2);
    }

    Ok(bytes_per_block)
}

// Helper function to calculate bytes per signal type
fn bytes_per_signal_type(
    num_samples: usize,
    num_channels: usize,
    bytes_per_sample: usize,
) -> usize {
    num_samples * num_channels * bytes_per_sample
}

// Helper struct to store raw data during reading
struct RawData {
    timestamps: Array1<i32>,
    amplifier_data_raw: Option<Array2<i32>>,
    dc_amplifier_data_raw: Option<Array2<i32>>,
    stim_data_raw: Option<Array2<i32>>,
    board_adc_data_raw: Option<Array2<i32>>,
    board_dac_data_raw: Option<Array2<i32>>,
    board_dig_in_raw: Option<Array2<i32>>,
    board_dig_out_raw: Option<Array2<i32>>,
}

/// Helper function to read all data blocks
///
/// This function reads all data blocks from the file into memory, organized by channel type.
fn read_all_data_blocks<R: Read + Seek>(
    header: &RhsHeader,
    num_samples: u64,
    num_blocks: u64,
    reader: &mut R,
) -> Result<RawData, Box<dyn std::error::Error>> {
    println!("Reading data from file...");

    // Initialize memory for raw data
    let mut raw_data = RawData {
        timestamps: Array1::zeros(num_samples as usize),
        amplifier_data_raw: if !header.amplifier_channels.is_empty() {
            Some(Array2::zeros((
                header.amplifier_channels.len(),
                num_samples as usize,
            )))
        } else {
            None
        },
        dc_amplifier_data_raw: if !header.amplifier_channels.is_empty()
            && header.dc_amplifier_data_saved
        {
            Some(Array2::zeros((
                header.amplifier_channels.len(),
                num_samples as usize,
            )))
        } else {
            None
        },
        stim_data_raw: if !header.amplifier_channels.is_empty() {
            Some(Array2::zeros((
                header.amplifier_channels.len(),
                num_samples as usize,
            )))
        } else {
            None
        },
        board_adc_data_raw: if !header.board_adc_channels.is_empty() {
            Some(Array2::zeros((
                header.board_adc_channels.len(),
                num_samples as usize,
            )))
        } else {
            None
        },
        board_dac_data_raw: if !header.board_dac_channels.is_empty() {
            Some(Array2::zeros((
                header.board_dac_channels.len(),
                num_samples as usize,
            )))
        } else {
            None
        },
        board_dig_in_raw: if !header.board_dig_in_channels.is_empty() {
            Some(Array2::zeros((
                header.board_dig_in_channels.len(),
                num_samples as usize,
            )))
        } else {
            None
        },
        board_dig_out_raw: if !header.board_dig_out_channels.is_empty() {
            Some(Array2::zeros((
                header.board_dig_out_channels.len(),
                num_samples as usize,
            )))
        } else {
            None
        },
    };

    // Read each data block
    let print_step = PRINT_PROGRESS_STEP;
    let mut percent_done = print_step;
    let num_blocks = num_blocks as usize;

    for i in 0..num_blocks {
        let index = i * SAMPLES_PER_DATA_BLOCK;
        read_one_data_block(&mut raw_data, header, index, reader)?;

        // Print progress
        let progress = (i as f64 / num_blocks as f64) * 100.0;
        if progress >= percent_done as f64 {
            println!("{}% done...", percent_done);
            percent_done += print_step;
        }
    }

    Ok(raw_data)
}

/// Helper function to read one data block
///
/// Reads a single block of data from the file, including timestamps, 
/// analog signals, and digital signals.
fn read_one_data_block<R: Read + Seek>(
    data: &mut RawData,
    header: &RhsHeader,
    index: usize,
    reader: &mut R,
) -> Result<(), Box<dyn std::error::Error>> {
    let samples_per_block = SAMPLES_PER_DATA_BLOCK;

    // Read timestamps
    read_timestamps(reader, &mut data.timestamps, index, samples_per_block)?;

    // Read analog signals
    read_analog_signals(reader, data, header, index, samples_per_block)?;

    // Read digital signals
    read_digital_signals(reader, data, header, index, samples_per_block)?;

    Ok(())
}

/// Helper function to read timestamps
/// 
/// Reads a block of timestamp values from the file into the timestamps array.
fn read_timestamps<R: Read>(
    reader: &mut R,
    timestamps: &mut Array1<i32>,
    index: usize,
    num_samples: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = index;
    let end = start + num_samples;

    // Read all timestamp bytes in one operation for better performance
    let mut buffer = vec![0u8; num_samples * 4];
    reader.read_exact(&mut buffer)?;

    let mut timestamps_slice = timestamps.slice_mut(s![start..end]);

    // Parse bytes into i32 values
    for i in 0..num_samples {
        let ts = i32::from_le_bytes([
            buffer[i * 4],
            buffer[i * 4 + 1],
            buffer[i * 4 + 2],
            buffer[i * 4 + 3],
        ]);
        timestamps_slice[i] = ts;
    }

    Ok(())
}

/// Helper function to read analog signals
/// 
/// Reads all analog signal types (amplifier, DC amplifier, stim, ADC, DAC) from a data block.
fn read_analog_signals<R: Read>(
    reader: &mut R,
    data: &mut RawData,
    header: &RhsHeader,
    index: usize,
    samples_per_block: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let num_amplifier_channels = header.amplifier_channels.len();

    // Read amplifier data
    if num_amplifier_channels > 0 {
        if let Some(ref mut amp_data) = data.amplifier_data_raw {
            read_analog_signal_type(
                reader,
                amp_data,
                index,
                samples_per_block,
                num_amplifier_channels,
            )?;
        }
    }

    // Read DC amplifier data
    if num_amplifier_channels > 0 && header.dc_amplifier_data_saved {
        if let Some(ref mut dc_amp_data) = data.dc_amplifier_data_raw {
            read_analog_signal_type(
                reader,
                dc_amp_data,
                index,
                samples_per_block,
                num_amplifier_channels,
            )?;
        }
    }

    // Read stim data
    if num_amplifier_channels > 0 {
        if let Some(ref mut stim_data) = data.stim_data_raw {
            read_analog_signal_type(
                reader,
                stim_data,
                index,
                samples_per_block,
                num_amplifier_channels,
            )?;
        }
    }

    // Read board ADC data
    let num_board_adc_channels = header.board_adc_channels.len();
    if num_board_adc_channels > 0 {
        if let Some(ref mut adc_data) = data.board_adc_data_raw {
            read_analog_signal_type(
                reader,
                adc_data,
                index,
                samples_per_block,
                num_board_adc_channels,
            )?;
        }
    }

    // Read board DAC data
    let num_board_dac_channels = header.board_dac_channels.len();
    if num_board_dac_channels > 0 {
        if let Some(ref mut dac_data) = data.board_dac_data_raw {
            read_analog_signal_type(
                reader,
                dac_data,
                index,
                samples_per_block,
                num_board_dac_channels,
            )?;
        }
    }

    Ok(())
}

/// Helper function to read an analog signal type
///
/// Reads a block of analog samples for multiple channels and stores them in the destination array.
fn read_analog_signal_type<R: Read>(
    reader: &mut R,
    dest: &mut Array2<i32>,
    start: usize,
    num_samples: usize,
    num_channels: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if num_channels < 1 {
        return Ok(());
    }

    let end = start + num_samples;

    // Read all channel data in one operation
    let mut buffer = vec![0u8; num_samples * num_channels * 2];
    reader.read_exact(&mut buffer)?;

    let mut t_slice = dest.slice_mut(s![.., start..end]);

    // Parse bytes into i16 values and store in the appropriate channel/sample position
    for ch in 0..num_channels {
        for s in 0..num_samples {
            let idx = 2 * (s * num_channels + ch);
            let sample = i16::from_le_bytes([buffer[idx], buffer[idx + 1]]) as i32;
            t_slice[[ch, s]] = sample;
        }
    }

    Ok(())
}

/// Helper function to read digital signals
///
/// Reads both digital input and output signals from a data block.
fn read_digital_signals<R: Read>(
    reader: &mut R,
    data: &mut RawData,
    header: &RhsHeader,
    index: usize,
    samples_per_block: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read digital input data
    let num_board_dig_in_channels = header.board_dig_in_channels.len();
    if num_board_dig_in_channels > 0 {
        read_digital_signal_type(reader, &mut data.board_dig_in_raw, index, samples_per_block)?;
    }

    // Read digital output data
    let num_board_dig_out_channels = header.board_dig_out_channels.len();
    if num_board_dig_out_channels > 0 {
        read_digital_signal_type(reader, &mut data.board_dig_out_raw, index, samples_per_block)?;
    }

    Ok(())
}

/// Helper function to read a digital signal type
///
/// Reads a block of digital samples for multiple channels and stores them in the destination array.
/// For digital signals, the same value is copied to all channels since they share the same data word.
fn read_digital_signal_type<R: Read>(
    reader: &mut R,
    dest: &mut Option<Array2<i32>>,
    start: usize,
    num_samples: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(dest_array) = dest.as_mut() {
        let num_channels = dest_array.shape()[0];
        if num_channels < 1 {
            return Ok(());
        }

        let end = start + num_samples;

        // Read all digital data in one operation
        let mut buffer = vec![0u8; num_samples * 2];
        reader.read_exact(&mut buffer)?;

        let mut t_slice = dest_array.slice_mut(s![.., start..end]);

        // For each sample, duplicate the value across all channels
        for s in 0..num_samples {
            let value = u16::from_le_bytes([buffer[s * 2], buffer[s * 2 + 1]]) as i32;

            for ch in 0..num_channels {
                t_slice[[ch, s]] = value;
            }
        }
    }

    Ok(())
}

/// Helper function to check end of file
///
/// Verifies that we've reached the end of the file after reading all data.
/// If there are bytes remaining, there's a problem with our understanding of the file format.
fn check_end_of_file<R: Read + Seek>(filesize: u64, reader: &mut R) -> Result<(), Box<dyn std::error::Error>> {
    let current_position = reader.stream_position()?;
    let bytes_remaining = filesize - current_position;

    if bytes_remaining != 0 {
        return Err(Box::new(IntanError::FileSizeError));
    }

    Ok(())
}

// Helper function to process raw data into final form
fn process_data(
    header: &RhsHeader,
    raw_data: RawData,
) -> Result<RhsData, Box<dyn std::error::Error>> {
    println!("Processing data...");

    // Create RhsData struct to hold processed data
    let mut data = RhsData {
        timestamps: raw_data.timestamps.clone(),
        amplifier_data: None,
        dc_amplifier_data: None,
        stim_data: None,
        compliance_limit_data: None,
        charge_recovery_data: None,
        amp_settle_data: None,
        board_adc_data: None,
        board_dac_data: None,
        board_dig_in_data: None,
        board_dig_out_data: None,
    };

    // Scale timestamps
    check_timestamps(&data.timestamps);

    // Process amplifier data
    if let Some(amp_data_raw) = raw_data.amplifier_data_raw {
        let mut amp_data = scale_amplifier_data(&amp_data_raw);

        // Apply notch filter if necessary
        apply_notch_filter(header, &mut amp_data);

        data.amplifier_data = Some(amp_data);
    }

    // Process DC amplifier data
    if let Some(dc_amp_data_raw) = raw_data.dc_amplifier_data_raw {
        let dc_amp_data = scale_dc_amplifier_data(&dc_amp_data_raw);
        data.dc_amplifier_data = Some(dc_amp_data);
    }

    // Process stim data
    if let Some(stim_data_raw) = raw_data.stim_data_raw {
        let (stim_data, compliance_limit_data, charge_recovery_data, amp_settle_data) =
            extract_stim_data(&stim_data_raw, header.stim_step_size);

        data.stim_data = Some(stim_data);
        data.compliance_limit_data = Some(compliance_limit_data);
        data.charge_recovery_data = Some(charge_recovery_data);
        data.amp_settle_data = Some(amp_settle_data);
    }

    // Process board ADC data
    if let Some(adc_data_raw) = raw_data.board_adc_data_raw {
        let adc_data = scale_adc_data(&adc_data_raw);
        data.board_adc_data = Some(adc_data);
    }

    // Process board DAC data
    if let Some(dac_data_raw) = raw_data.board_dac_data_raw {
        let dac_data = scale_dac_data(&dac_data_raw);
        data.board_dac_data = Some(dac_data);
    }

    // Process digital input data
    if let Some(dig_in_raw) = raw_data.board_dig_in_raw {
        data.board_dig_in_data = Some(extract_digital_data(
            &dig_in_raw,
            &header.board_dig_in_channels,
        )?);
    }

    // Process digital output data
    if let Some(dig_out_raw) = raw_data.board_dig_out_raw {
        data.board_dig_out_data = Some(extract_digital_data(
            &dig_out_raw,
            &header.board_dig_out_channels,
        )?);
    }

    Ok(data)
}

// Helper function to scale timestamps
fn check_timestamps(timestamps: &Array1<i32>) {
    // Check for gaps in timestamps
    let num_gaps = timestamps
        .windows(2)
        .into_iter()
        .filter(|window| window[1] - window[0] != 1)
        .count();

    if num_gaps == 0 {
        println!("No missing timestamps in data.");
    } else {
        println!(
            "Warning: {} gaps in timestamp data found. Time scale will not be uniform!",
            num_gaps
        );
    }
}

/// Scales amplifier data from raw ADC values to microvolts
///
/// Uses the scaling factor of 0.195 μV/bit with an offset of 32768
/// Raw values are treated as unsigned 16-bit integers
fn scale_amplifier_data(data_raw: &Array2<i32>) -> Array2<f64> {
    // Convert from signed to unsigned representation, then scale to microvolts
    data_raw.mapv(|x| {
        // Data was read as signed int16 but represents unsigned uint16 values
        let unsigned_val = if x < 0 { 
            (x + 65536) as f64 
        } else { 
            x as f64 
        };
        (unsigned_val - ADC_DAC_OFFSET) * AMPLIFIER_SCALE_FACTOR
    })
}

/// Scales DC amplifier data from raw ADC values to volts
///
/// Uses the scaling factor of 19.23 mV/bit with an offset of 512
/// Returns values in volts (not millivolts) for consistency
fn scale_dc_amplifier_data(data_raw: &Array2<i32>) -> Array2<f64> {
    // Convert from signed to unsigned, then scale to millivolts and convert to volts
    data_raw.mapv(|x| {
        let unsigned_val = if x < 0 { 
            (x + 65536) as f64 
        } else { 
            x as f64 
        };
        // Scale to millivolts then convert to volts
        ((unsigned_val - DC_AMPLIFIER_OFFSET) * DC_AMPLIFIER_SCALE_FACTOR) / 1000.0
    })
}

/// Scales ADC data from raw ADC values to volts
///
/// Uses the scaling factor of 0.0003125 V/bit with an offset of 32768
/// Raw values are treated as unsigned 16-bit integers
fn scale_adc_data(data_raw: &Array2<i32>) -> Array2<f64> {
    // Convert from signed to unsigned representation, then scale to volts
    data_raw.mapv(|x| {
        let unsigned_val = if x < 0 { 
            (x + 65536) as f64 
        } else { 
            x as f64 
        };
        (unsigned_val - ADC_DAC_OFFSET) * ADC_DAC_SCALE_FACTOR
    })
}

/// Scales DAC data from raw DAC values to volts
///
/// Uses the scaling factor of 0.0003125 V/bit with an offset of 32768
/// Raw values are treated as unsigned 16-bit integers
fn scale_dac_data(data_raw: &Array2<i32>) -> Array2<f64> {
    // Convert from signed to unsigned representation, then scale to volts
    data_raw.mapv(|x| {
        let unsigned_val = if x < 0 { 
            (x + 65536) as f64 
        } else { 
            x as f64 
        };
        (unsigned_val - ADC_DAC_OFFSET) * ADC_DAC_SCALE_FACTOR
    })
}

// Helper function to extract stim data
fn extract_stim_data(
    stim_data_raw: &Array2<i32>,
    stim_step_size: f32,
) -> (Array2<i32>, Array2<bool>, Array2<bool>, Array2<bool>) {
    let shape = stim_data_raw.shape();
    let num_channels = shape[0];
    let num_samples = shape[1];

    let mut stim_data = Array2::<i32>::zeros((num_channels, num_samples));
    let mut compliance_limit_data = Array2::<bool>::from_elem((num_channels, num_samples), false);
    let mut charge_recovery_data = Array2::<bool>::from_elem((num_channels, num_samples), false);
    let mut amp_settle_data = Array2::<bool>::from_elem((num_channels, num_samples), false);

    for i in 0..num_channels {
        for j in 0..num_samples {
            let value = stim_data_raw[[i, j]];

            // Interpret 2^15 bit (compliance limit) as true or false
            compliance_limit_data[[i, j]] = (value & 32768) != 0;

            // Interpret 2^14 bit (charge recovery) as true or false
            charge_recovery_data[[i, j]] = (value & 16384) != 0;

            // Interpret 2^13 bit (amp settle) as true or false
            amp_settle_data[[i, j]] = (value & 8192) != 0;

            // Interpret 2^8 bit (stim polarity) as +1 for 0_bit or -1 for 1_bit
            let stim_polarity = 1 - 2 * ((value & 256) >> 8);

            // Get least-significant 8 bits corresponding to the current amplitude
            let curr_amp = value & 255;

            // Multiply current amplitude by the correct sign and scaling factor
            stim_data[[i, j]] = ((curr_amp * stim_polarity) as f32 * stim_step_size) as i32;
        }
    }

    (
        stim_data,
        compliance_limit_data,
        charge_recovery_data,
        amp_settle_data,
    )
}

// Helper function to extract digital data
fn extract_digital_data(
    digital_data_raw: &Array2<i32>,
    channels: &[ChannelInfo],
) -> Result<Array2<i32>, Box<dyn std::error::Error>> {
    let shape = digital_data_raw.shape();
    let num_channels = channels.len();
    let num_samples = shape[1];

    let mut digital_data = Array2::<i32>::zeros((num_channels, num_samples));

    for (i, channel) in channels.iter().enumerate() {
        let mask = 1 << channel.native_order;

        for j in 0..num_samples {
            digital_data[[i, j]] = if (digital_data_raw[[0, j]] & mask) != 0 {
                1
            } else {
                0
            };
        }
    }

    Ok(digital_data)
}

// Helper function to apply notch filter
fn apply_notch_filter(header: &RhsHeader, data: &mut Array2<f64>) {
    // If data was not recorded with notch filter turned on, return without applying notch filter
    if header.notch_filter_frequency.is_none() {
        return;
    }

    // Similarly, if data was recorded from Intan RHX software version 3.0 or later,
    // any active notch filter was already applied to the saved data, so it should not be re-applied
    if header.version.major >= 3 {
        return;
    }

    let notch_freq = header.notch_filter_frequency.unwrap() as f32;

    // Apply notch filter individually to each channel
    println!("Applying notch filter...");
    let print_step = 10;
    let mut percent_done = print_step;
    let num_channels = data.shape()[0];

    for i in 0..num_channels {
        // Get channel data
        let channel_data: Vec<f64> = data.slice(s![i, ..]).to_vec();

        // Apply notch filter
        let filtered_data = notch_filter(&channel_data, header.sample_rate, notch_freq, 10);

        // Update the array
        let mut slice = data.slice_mut(s![i, ..]);
        for (j, &value) in filtered_data.iter().enumerate() {
            slice[j] = value;
        }

        // Print progress
        let progress = (i as f64 / num_channels as f64) * 100.0;
        if progress >= percent_done as f64 {
            println!("{}% done...", percent_done);
            percent_done += print_step;
        }
    }
}

// Helper function to apply notch filter to a single channel
fn notch_filter(signal_in: &[f64], f_sample: f32, f_notch: f32, bandwidth: i32) -> Vec<f64> {
    let t_step = 1.0 / f_sample as f64;
    let f_c = f_notch as f64 * t_step;
    let signal_length = signal_in.len();

    // Calculate filter parameters
    let d = (-2.0 * PI * (bandwidth as f64 / 2.0) * t_step).exp();
    let b = (1.0 + d * d) * (2.0 * PI * f_c).cos();
    let a0 = 1.0;
    let a1 = -b;
    let a2 = d * d;
    let a = (1.0 + d * d) / 2.0;
    let b0 = 1.0;
    let b1 = -2.0 * (2.0 * PI * f_c).cos();
    let b2 = 1.0;

    let mut signal_out = vec![0.0; signal_length];

    // Initialize first two samples
    signal_out[0] = signal_in[0];
    signal_out[1] = signal_in[1];

    // Apply filter to the rest of the samples
    for i in 2..signal_length {
        signal_out[i] =
            (a * b0 * signal_in[i] + a * b1 * signal_in[i - 1] + a * b2 * signal_in[i - 2]
                - a2 * signal_out[i - 2]
                - a1 * signal_out[i - 1])
                / a0;
    }

    signal_out
}


// Add these functions to the end of reader.rs

/// Loads and combines multiple RHS files into a single dataset
pub fn load_and_combine_files(file_paths: &[std::path::PathBuf]) -> Result<RhsFile, Box<dyn std::error::Error>> {
    
    if file_paths.is_empty() {
        return Err(Box::new(IntanError::Other("No files to load".to_string())));
    }
    
    // Load the first file
    println!("\nLoading file 1/{}: {}", file_paths.len(), file_paths[0].display());
    let mut combined_file = load_file(&file_paths[0])?;
    
    if file_paths.len() == 1 {
        return Ok(combined_file);
    }
    
    // Track source files
    combined_file.source_files = Some(vec![file_paths[0].to_string_lossy().to_string()]);
    
    // Load and combine remaining files
    for (i, file_path) in file_paths[1..].iter().enumerate() {
        println!("\nLoading file {}/{}: {}", i + 2, file_paths.len(), file_path.display());
        let next_file = load_file(file_path)?;

        
        // Verify headers are compatible
        verify_header_compatibility(&combined_file.header, &next_file.header)?;
        
        // Combine the data
        if combined_file.data_present && next_file.data_present {
            combine_data(&mut combined_file, next_file)?;
        }
        
        // Add to source files list
        if let Some(ref mut sources) = combined_file.source_files {
            sources.push(file_path.to_string_lossy().to_string());
        }
    }
    
    println!("\nSuccessfully combined {} files", file_paths.len());
    println!("Total duration: {:.2} seconds", combined_file.duration());
    
    Ok(combined_file)
}
/// Verifies that two headers are compatible for combining data
fn verify_header_compatibility(header1: &RhsHeader, header2: &RhsHeader) -> Result<(), Box<dyn std::error::Error>> {
    // Check sample rate
    if (header1.sample_rate - header2.sample_rate).abs() > 0.01 {
        return Err(Box::new(IntanError::Other(format!(
            "Sample rates don't match: {} Hz vs {} Hz",
            header1.sample_rate, header2.sample_rate
        ))));
    }
    
    // Check number of channels
    if header1.amplifier_channels.len() != header2.amplifier_channels.len() {
        return Err(Box::new(IntanError::Other(format!(
            "Number of amplifier channels don't match: {} vs {}",
            header1.amplifier_channels.len(), header2.amplifier_channels.len()
        ))));
    }
    
    if header1.board_adc_channels.len() != header2.board_adc_channels.len() {
        return Err(Box::new(IntanError::Other(format!(
            "Number of board ADC channels don't match: {} vs {}",
            header1.board_adc_channels.len(), header2.board_adc_channels.len()
        ))));
    }
    
    if header1.board_dig_in_channels.len() != header2.board_dig_in_channels.len() {
        return Err(Box::new(IntanError::Other(format!(
            "Number of digital input channels don't match: {} vs {}",
            header1.board_dig_in_channels.len(), header2.board_dig_in_channels.len()
        ))));
    }
    
    // Verify channel names match
    for (i, (ch1, ch2)) in header1.amplifier_channels.iter().zip(&header2.amplifier_channels).enumerate() {
        if ch1.native_channel_name != ch2.native_channel_name {
            return Err(Box::new(IntanError::Other(format!(
                "Amplifier channel {} names don't match: '{}' vs '{}'",
                i, ch1.native_channel_name, ch2.native_channel_name
            ))));
        }
    }
    
    Ok(())
}

/// Combines data from two RHS files
fn combine_data(combined: &mut RhsFile, next: RhsFile) -> Result<(), Box<dyn std::error::Error>> {
    use ndarray::{Axis, concatenate};
    
    if let (Some(combined_data), Some(next_data)) = (combined.data.as_mut(), next.data) {
 
        // Concatenate timestamps without adjustment, already saved with correct number between files
        combined_data.timestamps = concatenate![Axis(0), combined_data.timestamps.view(), next_data.timestamps.view()];
        
        // Concatenate amplifier data
        if let (Some(combined_amp), Some(next_amp)) = 
            (&mut combined_data.amplifier_data, next_data.amplifier_data) {
            *combined_amp = concatenate![Axis(1), combined_amp.view(), next_amp.view()];
        }
        
        // Concatenate DC amplifier data
        if let (Some(combined_dc), Some(next_dc)) = 
            (&mut combined_data.dc_amplifier_data, next_data.dc_amplifier_data) {
            *combined_dc = concatenate![Axis(1), combined_dc.view(), next_dc.view()];
        }
        
        // Concatenate stim data
        if let (Some(combined_stim), Some(next_stim)) = 
            (&mut combined_data.stim_data, next_data.stim_data) {
            *combined_stim = concatenate![Axis(1), combined_stim.view(), next_stim.view()];
        }
        
        // Concatenate compliance limit data
        if let (Some(combined_comp), Some(next_comp)) = 
            (&mut combined_data.compliance_limit_data, next_data.compliance_limit_data) {
            *combined_comp = concatenate![Axis(1), combined_comp.view(), next_comp.view()];
        }
        
        // Concatenate charge recovery data
        if let (Some(combined_charge), Some(next_charge)) = 
            (&mut combined_data.charge_recovery_data, next_data.charge_recovery_data) {
            *combined_charge = concatenate![Axis(1), combined_charge.view(), next_charge.view()];
        }
        
        // Concatenate amp settle data
        if let (Some(combined_settle), Some(next_settle)) = 
            (&mut combined_data.amp_settle_data, next_data.amp_settle_data) {
            *combined_settle = concatenate![Axis(1), combined_settle.view(), next_settle.view()];
        }
        
        // Concatenate board ADC data
        if let (Some(combined_adc), Some(next_adc)) = 
            (&mut combined_data.board_adc_data, next_data.board_adc_data) {
            *combined_adc = concatenate![Axis(1), combined_adc.view(), next_adc.view()];
        }
        
        // Concatenate board DAC data
        if let (Some(combined_dac), Some(next_dac)) = 
            (&mut combined_data.board_dac_data, next_data.board_dac_data) {
            *combined_dac = concatenate![Axis(1), combined_dac.view(), next_dac.view()];
        }
        
        // Concatenate digital input data
        if let (Some(combined_din), Some(next_din)) = 
            (&mut combined_data.board_dig_in_data, next_data.board_dig_in_data) {
            *combined_din = concatenate![Axis(1), combined_din.view(), next_din.view()];
        }
        
        // Concatenate digital output data
        if let (Some(combined_dout), Some(next_dout)) = 
            (&mut combined_data.board_dig_out_data, next_data.board_dig_out_data) {
            *combined_dout = concatenate![Axis(1), combined_dout.view(), next_dout.view()];
        }
    }
    
    Ok(())
}