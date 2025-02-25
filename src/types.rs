use ndarray::{Array1, Array2};
use std::error::Error;
use std::fmt;
use std::io;

/// Version information for the RHS file
#[derive(Debug, Clone)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
}

/// Notes stored in the RHS file
#[derive(Debug, Clone)]
pub struct Notes {
    pub note1: String,
    pub note2: String,
    pub note3: String,
}

/// Frequency parameters for the recording
#[derive(Debug, Clone)]
pub struct FrequencyParameters {
    pub amplifier_sample_rate: f32,
    pub board_adc_sample_rate: f32,
    pub board_dig_in_sample_rate: f32,
    pub desired_dsp_cutoff_frequency: f32,
    pub actual_dsp_cutoff_frequency: f32,
    pub dsp_enabled: i32,
    pub desired_lower_bandwidth: f32,
    pub desired_lower_settle_bandwidth: f32,
    pub actual_lower_bandwidth: f32,
    pub actual_lower_settle_bandwidth: f32,
    pub desired_upper_bandwidth: f32,
    pub actual_upper_bandwidth: f32,
    pub notch_filter_frequency: Option<i32>,
    pub desired_impedance_test_frequency: f32,
    pub actual_impedance_test_frequency: f32,
}

/// Stimulation parameters for the recording
#[derive(Debug, Clone)]
pub struct StimParameters {
    pub stim_step_size: f32,
    pub charge_recovery_current_limit: f32,
    pub charge_recovery_target_voltage: f32,
    pub amp_settle_mode: i32,
    pub charge_recovery_mode: i32,
}

/// Information about an individual channel
#[derive(Debug, Clone)]
pub struct ChannelInfo {
    pub port_name: String,
    pub port_prefix: String,
    pub port_number: i32,
    pub native_channel_name: String,
    pub custom_channel_name: String,
    pub native_order: i32,
    pub custom_order: i32,
    pub chip_channel: i32,
    pub board_stream: i32,
    pub electrode_impedance_magnitude: f32,
    pub electrode_impedance_phase: f32,
}

/// Spike trigger configuration
#[derive(Debug, Clone)]
pub struct SpikeTrigger {
    pub voltage_trigger_mode: i32,
    pub voltage_threshold: i32,
    pub digital_trigger_channel: i32,
    pub digital_edge_polarity: i32,
}

/// Header information from the RHS file
#[derive(Debug, Clone)]
pub struct RhsHeader {
    pub version: Version,
    pub sample_rate: f32,
    pub num_samples_per_data_block: i32,

    // DSP and bandwidth settings
    pub dsp_enabled: i32,
    pub actual_dsp_cutoff_frequency: f32,
    pub actual_lower_bandwidth: f32,
    pub actual_lower_settle_bandwidth: f32,
    pub actual_upper_bandwidth: f32,
    pub desired_dsp_cutoff_frequency: f32,
    pub desired_lower_bandwidth: f32,
    pub desired_lower_settle_bandwidth: f32,
    pub desired_upper_bandwidth: f32,

    // Filter settings
    pub notch_filter_frequency: Option<i32>,

    // Impedance test settings
    pub desired_impedance_test_frequency: f32,
    pub actual_impedance_test_frequency: f32,

    // Recovery and settle modes
    pub amp_settle_mode: i32,
    pub charge_recovery_mode: i32,

    // Stim settings
    pub stim_step_size: f32,
    pub recovery_current_limit: f32,
    pub recovery_target_voltage: f32,

    // Notes and modes
    pub notes: Notes,
    pub dc_amplifier_data_saved: bool,
    pub eval_board_mode: i32,
    pub reference_channel: String,

    // Channel information
    pub amplifier_channels: Vec<ChannelInfo>,
    pub spike_triggers: Vec<SpikeTrigger>,
    pub board_adc_channels: Vec<ChannelInfo>,
    pub board_dac_channels: Vec<ChannelInfo>,
    pub board_dig_in_channels: Vec<ChannelInfo>,
    pub board_dig_out_channels: Vec<ChannelInfo>,

    // Computed values
    pub frequency_parameters: FrequencyParameters,
    pub stim_parameters: StimParameters,
}

/// Data contained in the RHS file
#[derive(Debug, Clone)]
pub struct RhsData {
    pub timestamps: Array1<i32>,
    pub amplifier_data: Option<Array2<i32>>,
    pub dc_amplifier_data: Option<Array2<i32>>,
    pub stim_data: Option<Array2<i32>>,
    pub compliance_limit_data: Option<Array2<bool>>,
    pub charge_recovery_data: Option<Array2<bool>>,
    pub amp_settle_data: Option<Array2<bool>>,
    pub board_adc_data: Option<Array2<i32>>,
    pub board_dac_data: Option<Array2<i32>>,
    pub board_dig_in_data: Option<Array2<i32>>,
    pub board_dig_out_data: Option<Array2<i32>>,
}

/// Complete representation of an RHS file, including header and data
#[derive(Debug, Clone)]
pub struct RhsFile {
    pub header: RhsHeader,
    pub data: Option<RhsData>,
    pub data_present: bool,
}

/// Custom error types for the Intan importer
#[derive(Debug)]
pub enum IntanError {
    UnrecognizedFileFormat,
    InvalidChannelType,
    FileSizeError,
    StringReadError,
    ChannelNotFound,
    IoError(io::Error),
    Other(String),
}

impl fmt::Display for IntanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IntanError::UnrecognizedFileFormat => write!(f, "Unrecognized file format"),
            IntanError::InvalidChannelType => write!(f, "Invalid channel type"),
            IntanError::FileSizeError => write!(f, "File size error"),
            IntanError::StringReadError => write!(f, "Error reading string from file"),
            IntanError::ChannelNotFound => write!(f, "Channel not found"),
            IntanError::IoError(e) => write!(f, "IO error: {}", e),
            IntanError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for IntanError {}

impl From<io::Error> for IntanError {
    fn from(error: io::Error) -> Self {
        IntanError::IoError(error)
    }
}
