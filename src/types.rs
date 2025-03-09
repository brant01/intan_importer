use ndarray::{Array1, Array2};
use std::error::Error;
use std::fmt;
use std::io;

/// Version information for the RHS file.
///
/// Contains major and minor version numbers for the file format.
#[derive(Debug, Clone)]
pub struct Version {
    /// Major version number
    pub major: i32,
    /// Minor version number
    pub minor: i32,
}

/// Notes stored in the RHS file.
///
/// Intan recording software allows up to three notes to be stored with each recording.
/// These are typically used to document experimental conditions or other metadata.
#[derive(Debug, Clone)]
pub struct Notes {
    /// First note text
    pub note1: String,
    /// Second note text
    pub note2: String,
    /// Third note text
    pub note3: String,
}

/// Frequency parameters for the recording.
///
/// Contains various sampling rates and filter settings for the recording.
/// Includes both the originally requested values ("desired_*") and the actual
/// values that were achieved by the hardware ("actual_*").
#[derive(Debug, Clone)]
pub struct FrequencyParameters {
    /// Sample rate for amplifier channels (Hz)
    pub amplifier_sample_rate: f32,
    /// Sample rate for board ADC channels (Hz)
    pub board_adc_sample_rate: f32,
    /// Sample rate for digital input channels (Hz)
    pub board_dig_in_sample_rate: f32,
    /// User-requested DSP cutoff frequency (Hz)
    pub desired_dsp_cutoff_frequency: f32,
    /// Actual DSP cutoff frequency achieved (Hz)
    pub actual_dsp_cutoff_frequency: f32,
    /// Whether DSP was enabled (1) or disabled (0)
    pub dsp_enabled: i32,
    /// User-requested lower bandwidth (Hz)
    pub desired_lower_bandwidth: f32,
    /// User-requested lower settle bandwidth (Hz)
    pub desired_lower_settle_bandwidth: f32,
    /// Actual lower bandwidth achieved (Hz)
    pub actual_lower_bandwidth: f32,
    /// Actual lower settle bandwidth achieved (Hz)
    pub actual_lower_settle_bandwidth: f32,
    /// User-requested upper bandwidth (Hz)
    pub desired_upper_bandwidth: f32,
    /// Actual upper bandwidth achieved (Hz)
    pub actual_upper_bandwidth: f32,
    /// Notch filter frequency (50Hz, 60Hz, or None)
    pub notch_filter_frequency: Option<i32>,
    /// User-requested impedance test frequency (Hz)
    pub desired_impedance_test_frequency: f32,
    /// Actual impedance test frequency achieved (Hz)
    pub actual_impedance_test_frequency: f32,
}

/// Stimulation parameters for the recording.
///
/// Contains settings related to electrical stimulation, which is a feature
/// of some Intan recording systems.
#[derive(Debug, Clone)]
pub struct StimParameters {
    /// Stimulation current step size (μA)
    pub stim_step_size: f32,
    /// Maximum current used in charge recovery (μA)
    pub charge_recovery_current_limit: f32,
    /// Target voltage for charge recovery (V)
    pub charge_recovery_target_voltage: f32,
    /// Amplifier settle mode setting
    /// - 0: Traditional (switch to ground)
    /// - 1: Limited switches
    pub amp_settle_mode: i32,
    /// Charge recovery mode setting
    /// - 0: Current-limited charge recovery circuit engaged during stimulation
    /// - 1: Circuit engaged all the time
    pub charge_recovery_mode: i32,
}

/// Information about an individual channel.
///
/// Contains naming, ordering, and hardware configuration for a single recording channel.
/// This includes amplifier channels, ADC channels, digital inputs, etc.
#[derive(Debug, Clone)]
pub struct ChannelInfo {
    /// Name of the port (e.g., "Port A")
    pub port_name: String,
    /// Prefix for the port (e.g., "A")
    pub port_prefix: String,
    /// Port number on the device
    pub port_number: i32,
    /// Default channel name assigned by the system
    pub native_channel_name: String,
    /// User-defined custom name for the channel
    pub custom_channel_name: String,
    /// Original order in the native system
    pub native_order: i32,
    /// Custom order (often used for display purposes)
    pub custom_order: i32,
    /// Channel on the chip
    pub chip_channel: i32,
    /// Hardware stream on the board
    pub board_stream: i32,
    /// Measured electrode impedance magnitude (Ω)
    pub electrode_impedance_magnitude: f32,
    /// Measured electrode impedance phase (radians)
    pub electrode_impedance_phase: f32,
}

/// Spike trigger configuration.
///
/// Contains settings for spike detection triggers.
#[derive(Debug, Clone)]
pub struct SpikeTrigger {
    /// Voltage trigger mode
    /// - 0: Trigger on digital input
    /// - 1: Trigger on voltage threshold
    pub voltage_trigger_mode: i32,
    /// Voltage threshold for triggering (μV)
    pub voltage_threshold: i32,
    /// Digital input channel to use for triggering
    pub digital_trigger_channel: i32,
    /// Digital edge polarity for trigger
    /// - 0: Trigger on falling edge
    /// - 1: Trigger on rising edge
    pub digital_edge_polarity: i32,
}

/// Header information from the RHS file.
///
/// Contains all metadata and configuration information from the recording file.
/// This includes version information, sampling rates, filter settings, channel
/// configurations, and more.
#[derive(Debug, Clone)]
pub struct RhsHeader {
    /// File format version
    pub version: Version,
    /// Primary sample rate of the recording (Hz)
    pub sample_rate: f32,
    /// Number of samples per data block (fixed at 128 for RHS files)
    pub num_samples_per_data_block: i32,

    // DSP and bandwidth settings
    /// Whether DSP was enabled (1) or disabled (0)
    pub dsp_enabled: i32,
    /// Actual DSP cutoff frequency achieved (Hz)
    pub actual_dsp_cutoff_frequency: f32,
    /// Actual lower bandwidth achieved (Hz)
    pub actual_lower_bandwidth: f32,
    /// Actual lower settle bandwidth achieved (Hz)
    pub actual_lower_settle_bandwidth: f32,
    /// Actual upper bandwidth achieved (Hz)
    pub actual_upper_bandwidth: f32,
    /// User-requested DSP cutoff frequency (Hz)
    pub desired_dsp_cutoff_frequency: f32,
    /// User-requested lower bandwidth (Hz)
    pub desired_lower_bandwidth: f32,
    /// User-requested lower settle bandwidth (Hz)
    pub desired_lower_settle_bandwidth: f32,
    /// User-requested upper bandwidth (Hz)
    pub desired_upper_bandwidth: f32,

    // Filter settings
    /// Notch filter frequency (50Hz, 60Hz, or None)
    pub notch_filter_frequency: Option<i32>,

    // Impedance test settings
    /// User-requested impedance test frequency (Hz)
    pub desired_impedance_test_frequency: f32,
    /// Actual impedance test frequency achieved (Hz)
    pub actual_impedance_test_frequency: f32,

    // Recovery and settle modes
    /// Amplifier settle mode setting
    /// - 0: Traditional (switch to ground)
    /// - 1: Limited switches
    pub amp_settle_mode: i32,
    /// Charge recovery mode setting
    /// - 0: Current-limited charge recovery circuit engaged during stimulation
    /// - 1: Circuit engaged all the time
    pub charge_recovery_mode: i32,

    // Stim settings
    /// Stimulation current step size (μA)
    pub stim_step_size: f32,
    /// Maximum current used in charge recovery (μA)
    pub recovery_current_limit: f32,
    /// Target voltage for charge recovery (V)
    pub recovery_target_voltage: f32,

    // Notes and modes
    /// User notes saved with the recording
    pub notes: Notes,
    /// Whether DC amplifier data was saved (true) or not (false)
    pub dc_amplifier_data_saved: bool,
    /// Evaluation board mode
    /// - 0: Recording Controller
    /// - 1: Recording Controller + Stim
    /// - 2: Recording System
    pub eval_board_mode: i32,
    /// Name of the reference channel used
    pub reference_channel: String,

    // Channel information
    /// List of amplifier channels in the recording
    pub amplifier_channels: Vec<ChannelInfo>,
    /// List of spike trigger configurations (one per amplifier channel)
    pub spike_triggers: Vec<SpikeTrigger>,
    /// List of board ADC (analog-to-digital converter) channels
    pub board_adc_channels: Vec<ChannelInfo>,
    /// List of board DAC (digital-to-analog converter) channels
    pub board_dac_channels: Vec<ChannelInfo>,
    /// List of board digital input channels
    pub board_dig_in_channels: Vec<ChannelInfo>,
    /// List of board digital output channels
    pub board_dig_out_channels: Vec<ChannelInfo>,

    // Computed values
    /// Consolidated frequency parameters from various header fields
    pub frequency_parameters: FrequencyParameters,
    /// Consolidated stimulation parameters from various header fields
    pub stim_parameters: StimParameters,
}

/// Data contained in the RHS file.
///
/// Contains the actual recorded signals from all enabled channels.
/// Each field is an ndarray where the first dimension is the channel
/// and the second dimension is the time sample.
#[derive(Debug, Clone)]
pub struct RhsData {
    /// Timestamps for each sample (in samples, convert to seconds by dividing by sample_rate)
    pub timestamps: Array1<i32>,
    /// Neural data from amplifier channels (μV)
    /// - Shape: [num_channels, num_samples]
    pub amplifier_data: Option<Array2<i32>>,
    /// DC amplifier data (V)
    /// - Shape: [num_channels, num_samples]
    pub dc_amplifier_data: Option<Array2<i32>>,
    /// Stimulation current data (μA)
    /// - Shape: [num_channels, num_samples]
    pub stim_data: Option<Array2<i32>>,
    /// Compliance limit status for each channel and sample
    /// - true: compliance limit was reached
    /// - false: compliance limit was not reached
    /// - Shape: [num_channels, num_samples]
    pub compliance_limit_data: Option<Array2<bool>>,
    /// Charge recovery status for each channel and sample
    /// - true: charge recovery was active
    /// - false: charge recovery was inactive
    /// - Shape: [num_channels, num_samples]
    pub charge_recovery_data: Option<Array2<bool>>,
    /// Amplifier settle status for each channel and sample
    /// - true: amplifier settle was active
    /// - false: amplifier settle was inactive
    /// - Shape: [num_channels, num_samples]
    pub amp_settle_data: Option<Array2<bool>>,
    /// Board ADC data (V)
    /// - Shape: [num_channels, num_samples]
    pub board_adc_data: Option<Array2<i32>>,
    /// Board DAC data (V)
    /// - Shape: [num_channels, num_samples]
    pub board_dac_data: Option<Array2<i32>>,
    /// Board digital input data (0 or 1)
    /// - Shape: [num_channels, num_samples]
    pub board_dig_in_data: Option<Array2<i32>>,
    /// Board digital output data (0 or 1)
    /// - Shape: [num_channels, num_samples]
    pub board_dig_out_data: Option<Array2<i32>>,
}

/// Complete representation of an RHS file, including header and data.
///
/// This is the top-level struct returned by the `load` function. It contains
/// both the header information (metadata, configuration) and the actual recorded
/// data (if present in the file).
///
/// # Examples
///
/// ```no_run
/// use intan_importer::load;
///
/// let rhs_file = load("path/to/your/file.rhs").unwrap();
///
/// // Access header information
/// println!("Sample rate: {} Hz", rhs_file.header.sample_rate);
///
/// // Check if data is present
/// if rhs_file.data_present {
///     // Access the first amplifier channel if available
///     if let Some(data) = &rhs_file.data {
///         if let Some(amp_data) = &data.amplifier_data {
///             if amp_data.shape()[0] > 0 {
///                 println!("First sample: {} μV", amp_data[[0, 0]]);
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RhsFile {
    /// Header information containing metadata and configuration
    pub header: RhsHeader,
    /// Recorded data (if present in the file)
    pub data: Option<RhsData>,
    /// Flag indicating whether data is present in the file
    pub data_present: bool,
}

impl RhsFile {
    /// Returns the duration of the recording in seconds.
    ///
    /// If no data is present, returns 0.0.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use intan_importer::load;
    ///
    /// let rhs_file = load("path/to/your/file.rhs").unwrap();
    /// println!("Recording duration: {:.2} seconds", rhs_file.duration());
    /// ```
    pub fn duration(&self) -> f32 {
        if let Some(data) = &self.data {
            let num_samples = data.timestamps.len();
            num_samples as f32 / self.header.sample_rate
        } else {
            0.0
        }
    }

    /// Returns the number of samples in the recording.
    ///
    /// If no data is present, returns 0.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use intan_importer::load;
    ///
    /// let rhs_file = load("path/to/your/file.rhs").unwrap();
    /// println!("Number of samples: {}", rhs_file.num_samples());
    /// ```
    pub fn num_samples(&self) -> usize {
        if let Some(data) = &self.data {
            data.timestamps.len()
        } else {
            0
        }
    }
}

/// Custom error types for the Intan importer.
///
/// Represents various error conditions that may occur during file reading
/// and processing.
#[derive(Debug)]
pub enum IntanError {
    /// The file format was not recognized as an Intan RHS file
    UnrecognizedFileFormat,
    /// An invalid channel type was encountered
    InvalidChannelType,
    /// The file size doesn't match what was expected based on data block size
    FileSizeError,
    /// Error reading a string from the file
    StringReadError,
    /// A requested channel was not found
    ChannelNotFound,
    /// An I/O error occurred during file reading
    IoError(io::Error),
    /// A general error with a custom message
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
