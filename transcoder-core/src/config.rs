//! Transcoding configuration and codec presets

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Video codec options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VideoCodec {
    #[serde(rename = "prores")]
    ProRes,
    #[serde(rename = "prores_ks")]
    ProResKS,
    #[serde(rename = "dnxhd")]
    DNxHD,
    #[serde(rename = "dnxhr")]
    DNxHR,
    #[serde(rename = "h264")]
    H264,
    #[serde(rename = "h265")]
    H265,
    #[serde(rename = "copy")]
    Copy,
}

/// Audio codec options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AudioCodec {
    #[serde(rename = "pcm_s16le")]
    PCM16,
    #[serde(rename = "pcm_s24le")]
    PCM24,
    #[serde(rename = "aac")]
    AAC,
    #[serde(rename = "copy")]
    Copy,
}

/// Container format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContainerFormat {
    #[serde(rename = "mov")]
    MOV,
    #[serde(rename = "mp4")]
    MP4,
    #[serde(rename = "mxf")]
    MXF,
    #[serde(rename = "wav")]
    WAV,
    #[serde(rename = "auto")]
    Auto,
}

/// ProRes profile
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProResProfile {
    #[serde(rename = "proxy")]
    Proxy,        // Profile 0
    #[serde(rename = "lt")]
    LT,           // Profile 1
    #[serde(rename = "standard")]
    Standard,     // Profile 2
    #[serde(rename = "hq")]
    HQ,           // Profile 3
    #[serde(rename = "4444")]
    ProRes4444,   // Profile 4
    #[serde(rename = "4444xq")]
    ProRes4444XQ, // Profile 5
}

/// DNxHR profile (for HD and above)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DnxhrProfile {
    #[serde(rename = "lb")]
    LB,      // Low Bandwidth (8-bit, ~45 Mbps @ 1080p)
    #[serde(rename = "sq")]
    SQ,      // Standard Quality (8-bit, ~100 Mbps @ 1080p)
    #[serde(rename = "hq")]
    HQ,      // High Quality (8-bit, ~145 Mbps @ 1080p)
    #[serde(rename = "hqx")]
    HQX,     // High Quality 10-bit (10-bit, ~220 Mbps @ 1080p)
    #[serde(rename = "444")]
    DNxHR444, // Highest Quality 10-bit 4:4:4 (10-bit, ~440 Mbps @ 1080p)
}

impl ProResProfile {
    pub fn profile_number(&self) -> i32 {
        match self {
            ProResProfile::Proxy => 0,
            ProResProfile::LT => 1,
            ProResProfile::Standard => 2,
            ProResProfile::HQ => 3,
            ProResProfile::ProRes4444 => 4,
            ProResProfile::ProRes4444XQ => 5,
        }
    }
}

impl DnxhrProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            DnxhrProfile::LB => "dnxhr_lb",
            DnxhrProfile::SQ => "dnxhr_sq",
            DnxhrProfile::HQ => "dnxhr_hq",
            DnxhrProfile::HQX => "dnxhr_hqx",
            DnxhrProfile::DNxHR444 => "dnxhr_444",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DnxhrProfile::LB => "Low Bandwidth (8-bit, ~45 Mbps @ 1080p)",
            DnxhrProfile::SQ => "Standard Quality (8-bit, ~100 Mbps @ 1080p)",
            DnxhrProfile::HQ => "High Quality (8-bit, ~145 Mbps @ 1080p)",
            DnxhrProfile::HQX => "High Quality 10-bit (10-bit, ~220 Mbps @ 1080p)",
            DnxhrProfile::DNxHR444 => "Highest Quality 4:4:4 (10-bit, ~440 Mbps @ 1080p)",
        }
    }
}

/// Transcoding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodeConfig {
    pub video_codec: VideoCodec,
    pub audio_codec: AudioCodec,
    pub container: ContainerFormat,
    pub video_bitrate: Option<String>,
    pub audio_bitrate: Option<String>,
    pub audio_sample_rate: Option<u32>,
    pub resolution: Option<String>,
    pub frame_rate: Option<f32>,
    pub prores_profile: Option<ProResProfile>,
    pub dnxhr_profile: Option<DnxhrProfile>,
    pub extra_args: Vec<String>,
    pub hw_accel: bool,                  // Hardware acceleration (VideoToolbox on macOS)
    pub extract_bwf: bool,               // Extract BWF audio files alongside video
    pub map_all_audio: bool,             // Map all audio tracks (not just first)
    pub lut_path: Option<String>,        // Path to LUT file (.cube) for color grading
    pub create_ale: bool,                // Create Avid Log Exchange file
}

impl Default for TranscodeConfig {
    fn default() -> Self {
        Self {
            video_codec: VideoCodec::ProResKS,
            audio_codec: AudioCodec::PCM24,
            container: ContainerFormat::MOV,
            video_bitrate: None,
            audio_bitrate: None,
            audio_sample_rate: Some(48000),
            resolution: None,
            frame_rate: None,
            prores_profile: Some(ProResProfile::HQ),
            dnxhr_profile: None,
            extra_args: Vec::new(),
            hw_accel: true,              // Enable by default on macOS
            extract_bwf: false,          // Optional feature
            map_all_audio: true,         // Map all audio tracks by default
            lut_path: None,              // Optional LUT file
            create_ale: false,           // Optional ALE creation
        }
    }
}

impl TranscodeConfig {
    /// Build FFmpeg command arguments from config
    pub fn to_ffmpeg_args(&self, input: &str, output: &str) -> Vec<String> {
        let mut args = vec![];
        
        // Hardware acceleration (before input)
        if self.hw_accel {
            #[cfg(target_os = "macos")]
            {
                args.push("-hwaccel".to_string());
                args.push("videotoolbox".to_string());
            }
            #[cfg(target_os = "linux")]
            {
                // Try VAAPI on Linux
                args.push("-hwaccel".to_string());
                args.push("vaapi".to_string());
            }
            #[cfg(target_os = "windows")]
            {
                // Try D3D11VA on Windows
                args.push("-hwaccel".to_string());
                args.push("d3d11va".to_string());
            }
        }
        
        args.push("-i".to_string());
        args.push(input.to_string());
        args.push("-y".to_string()); // Overwrite output

        // Video codec
        match &self.video_codec {
            VideoCodec::ProRes => {
                args.push("-c:v".to_string());
                args.push("prores".to_string());
            }
            VideoCodec::ProResKS => {
                args.push("-c:v".to_string());
                args.push("prores_ks".to_string());
                if let Some(profile) = &self.prores_profile {
                    args.push("-profile:v".to_string());
                    args.push(profile.profile_number().to_string());
                }
            }
            VideoCodec::DNxHD => {
                args.push("-c:v".to_string());
                args.push("dnxhd".to_string());
            }
            VideoCodec::DNxHR => {
                args.push("-c:v".to_string());
                args.push("dnxhd".to_string()); // FFmpeg uses dnxhd encoder for both
                if let Some(profile) = &self.dnxhr_profile {
                    args.push("-profile:v".to_string());
                    args.push(profile.as_str().to_string());
                }
                // Add pixel format for DNxHR compatibility (8-bit for LB/SQ/HQ, 10-bit for HQX/444)
                if let Some(profile) = &self.dnxhr_profile {
                    match profile {
                        DnxhrProfile::LB | DnxhrProfile::SQ | DnxhrProfile::HQ => {
                            args.push("-pix_fmt".to_string());
                            args.push("yuv422p".to_string()); // 8-bit
                        }
                        DnxhrProfile::HQX | DnxhrProfile::DNxHR444 => {
                            args.push("-pix_fmt".to_string());
                            args.push("yuv422p10le".to_string()); // 10-bit
                        }
                    }
                }
            }
            VideoCodec::H264 => {
                args.push("-c:v".to_string());
                if self.hw_accel {
                    #[cfg(target_os = "macos")]
                    args.push("h264_videotoolbox".to_string());
                    #[cfg(not(target_os = "macos"))]
                    args.push("libx264".to_string());
                } else {
                    args.push("libx264".to_string());
                }
                if !self.hw_accel {
                    args.push("-preset".to_string());
                    args.push("medium".to_string());
                }
            }
            VideoCodec::H265 => {
                args.push("-c:v".to_string());
                if self.hw_accel {
                    #[cfg(target_os = "macos")]
                    args.push("hevc_videotoolbox".to_string());
                    #[cfg(not(target_os = "macos"))]
                    args.push("libx265".to_string());
                } else {
                    args.push("libx265".to_string());
                }
            }
            VideoCodec::Copy => {
                args.push("-c:v".to_string());
                args.push("copy".to_string());
            }
        }

        // Video bitrate
        if let Some(bitrate) = &self.video_bitrate {
            args.push("-b:v".to_string());
            args.push(bitrate.clone());
        }

        // Resolution
        if let Some(resolution) = &self.resolution {
            args.push("-s".to_string());
            args.push(resolution.clone());
        }

        // Frame rate
        if let Some(fps) = self.frame_rate {
            args.push("-r".to_string());
            args.push(fps.to_string());
        }

        // LUT application (3D LUT for color grading)
        if let Some(lut_path) = &self.lut_path {
            args.push("-vf".to_string());
            args.push(format!("lut3d=file='{}'", lut_path));
        }

        // Audio codec
        match &self.audio_codec {
            AudioCodec::PCM16 => {
                args.push("-c:a".to_string());
                args.push("pcm_s16le".to_string());
            }
            AudioCodec::PCM24 => {
                args.push("-c:a".to_string());
                args.push("pcm_s24le".to_string());
            }
            AudioCodec::AAC => {
                args.push("-c:a".to_string());
                args.push("aac".to_string());
            }
            AudioCodec::Copy => {
                args.push("-c:a".to_string());
                args.push("copy".to_string());
            }
        }

        // Audio sample rate
        if let Some(sample_rate) = self.audio_sample_rate {
            args.push("-ar".to_string());
            args.push(sample_rate.to_string());
        }

        // Audio bitrate
        if let Some(bitrate) = &self.audio_bitrate {
            args.push("-b:a".to_string());
            args.push(bitrate.clone());
        }

        // Audio/Video mapping
        if self.map_all_audio {
            args.push("-map".to_string());
            args.push("0:v:0".to_string()); // First video stream
            args.push("-map".to_string());
            args.push("0:a".to_string());   // All audio streams
        }
        
        // Multi-threaded encoding
        args.push("-threads".to_string());
        args.push("0".to_string()); // Auto-detect optimal thread count

        // Container format
        match &self.container {
            ContainerFormat::MOV => {
                args.push("-f".to_string());
                args.push("mov".to_string());
            }
            ContainerFormat::MP4 => {
                args.push("-f".to_string());
                args.push("mp4".to_string());
            }
            ContainerFormat::MXF => {
                args.push("-f".to_string());
                args.push("mxf".to_string());
            }
            ContainerFormat::WAV => {
                args.push("-f".to_string());
                args.push("wav".to_string());
            }
            ContainerFormat::Auto => {} // Let FFmpeg auto-detect
        }

        // Extra arguments
        args.extend(self.extra_args.clone());

        // Output file
        args.push(output.to_string());

        args
    }
}

/// Codec preset templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodecPreset {
    pub name: String,
    pub description: String,
    pub config: TranscodeConfig,
}

impl CodecPreset {
    /// ProRes HQ preset (broadcast quality)
    pub fn prores_hq() -> Self {
        Self {
            name: "ProRes HQ".to_string(),
            description: "High Quality ProRes for broadcast".to_string(),
            config: TranscodeConfig {
                video_codec: VideoCodec::ProResKS,
                audio_codec: AudioCodec::PCM24,
                container: ContainerFormat::MOV,
                prores_profile: Some(ProResProfile::HQ),
                audio_sample_rate: Some(48000),
                ..Default::default()
            },
        }
    }

    /// ProRes 422 preset (standard quality)
    pub fn prores_422() -> Self {
        Self {
            name: "ProRes 422".to_string(),
            description: "Standard ProRes 422".to_string(),
            config: TranscodeConfig {
                video_codec: VideoCodec::ProResKS,
                prores_profile: Some(ProResProfile::Standard),
                audio_codec: AudioCodec::PCM24,
                container: ContainerFormat::MOV,
                audio_sample_rate: Some(48000),
                ..Default::default()
            },
        }
    }

    /// ProRes LT preset (lower quality, smaller files)
    pub fn prores_lt() -> Self {
        Self {
            name: "ProRes LT".to_string(),
            description: "ProRes LT for offline editing".to_string(),
            config: TranscodeConfig {
                video_codec: VideoCodec::ProResKS,
                prores_profile: Some(ProResProfile::LT),
                audio_codec: AudioCodec::PCM24,
                container: ContainerFormat::MOV,
                audio_sample_rate: Some(48000),
                ..Default::default()
            },
        }
    }

    /// H.264 high quality preset
    pub fn h264_high() -> Self {
        Self {
            name: "H.264 High".to_string(),
            description: "H.264 high quality for delivery".to_string(),
            config: TranscodeConfig {
                video_codec: VideoCodec::H264,
                audio_codec: AudioCodec::AAC,
                container: ContainerFormat::MP4,
                video_bitrate: Some("20M".to_string()),
                audio_bitrate: Some("320k".to_string()),
                audio_sample_rate: Some(48000),
                ..Default::default()
            },
        }
    }

    /// DNxHR HQX preset (10-bit, for 4K/UHD)
    pub fn dnxhr_hqx() -> Self {
        Self {
            name: "DNxHR HQX".to_string(),
            description: "Avid DNxHR HQX 10-bit for 4K/UHD editing".to_string(),
            config: TranscodeConfig {
                video_codec: VideoCodec::DNxHR,
                audio_codec: AudioCodec::PCM24,
                container: ContainerFormat::MOV,
                dnxhr_profile: Some(DnxhrProfile::HQX),
                audio_sample_rate: Some(48000),
                ..Default::default()
            },
        }
    }

    /// DNxHR HQ preset (8-bit, for 4K/UHD)
    pub fn dnxhr_hq() -> Self {
        Self {
            name: "DNxHR HQ".to_string(),
            description: "Avid DNxHR HQ 8-bit for 4K/UHD editing".to_string(),
            config: TranscodeConfig {
                video_codec: VideoCodec::DNxHR,
                audio_codec: AudioCodec::PCM24,
                container: ContainerFormat::MOV,
                dnxhr_profile: Some(DnxhrProfile::HQ),
                audio_sample_rate: Some(48000),
                ..Default::default()
            },
        }
    }

    /// DNxHR SQ preset (standard quality)
    pub fn dnxhr_sq() -> Self {
        Self {
            name: "DNxHR SQ".to_string(),
            description: "Avid DNxHR SQ for offline editing".to_string(),
            config: TranscodeConfig {
                video_codec: VideoCodec::DNxHR,
                audio_codec: AudioCodec::PCM24,
                container: ContainerFormat::MOV,
                dnxhr_profile: Some(DnxhrProfile::SQ),
                audio_sample_rate: Some(48000),
                ..Default::default()
            },
        }
    }

    /// DNxHR LB preset (low bandwidth, fast transcode with HW accel)
    pub fn dnxhr_lb_fast() -> Self {
        Self {
            name: "DNxHR LB (Fast)".to_string(),
            description: "Avid DNxHR LB with hardware acceleration - FASTEST!".to_string(),
            config: TranscodeConfig {
                video_codec: VideoCodec::DNxHR,
                audio_codec: AudioCodec::PCM24,
                container: ContainerFormat::MOV,
                dnxhr_profile: Some(DnxhrProfile::LB),
                audio_sample_rate: Some(48000),
                hw_accel: true,           // Hardware acceleration enabled
                extract_bwf: false,       // Can be enabled in UI
                map_all_audio: true,      // Map all audio tracks
                ..Default::default()
            },
        }
    }

    /// H.264 hardware-accelerated preset (fast, delivery quality)
    pub fn h264_fast() -> Self {
        Self {
            name: "H.264 (Fast HW)".to_string(),
            description: "Hardware-accelerated H.264 for fast delivery/proxies".to_string(),
            config: TranscodeConfig {
                video_codec: VideoCodec::H264,
                audio_codec: AudioCodec::AAC,
                container: ContainerFormat::MP4,
                video_bitrate: Some("15M".to_string()),  // Good quality at reasonable size
                audio_bitrate: Some("256k".to_string()),
                audio_sample_rate: Some(48000),
                hw_accel: true,           // Hardware acceleration enabled
                extract_bwf: false,
                map_all_audio: true,
                ..Default::default()
            },
        }
    }

    /// H.264 high quality hardware-accelerated preset
    pub fn h264_hq_fast() -> Self {
        Self {
            name: "H.264 HQ (Fast HW)".to_string(),
            description: "Hardware-accelerated H.264 high quality for delivery".to_string(),
            config: TranscodeConfig {
                video_codec: VideoCodec::H264,
                audio_codec: AudioCodec::AAC,
                container: ContainerFormat::MP4,
                video_bitrate: Some("25M".to_string()),  // Higher bitrate for quality
                audio_bitrate: Some("320k".to_string()),
                audio_sample_rate: Some(48000),
                hw_accel: true,
                extract_bwf: false,
                map_all_audio: true,
                ..Default::default()
            },
        }
    }

    /// HEVC/H.265 hardware-accelerated preset (best compression)
    pub fn hevc_fast() -> Self {
        Self {
            name: "HEVC/H.265 (Fast HW)".to_string(),
            description: "Hardware-accelerated HEVC for best compression - half the size of H.264!".to_string(),
            config: TranscodeConfig {
                video_codec: VideoCodec::H265,
                audio_codec: AudioCodec::AAC,
                container: ContainerFormat::MP4,
                video_bitrate: Some("12M".to_string()),  // HEVC needs less bitrate for same quality
                audio_bitrate: Some("256k".to_string()),
                audio_sample_rate: Some(48000),
                hw_accel: true,
                extract_bwf: false,
                map_all_audio: true,
                ..Default::default()
            },
        }
    }

    /// HEVC/H.265 high quality hardware-accelerated preset
    pub fn hevc_hq_fast() -> Self {
        Self {
            name: "HEVC/H.265 HQ (Fast HW)".to_string(),
            description: "Hardware-accelerated HEVC high quality - amazing compression!".to_string(),
            config: TranscodeConfig {
                video_codec: VideoCodec::H265,
                audio_codec: AudioCodec::AAC,
                container: ContainerFormat::MP4,
                video_bitrate: Some("18M".to_string()),  // Higher bitrate for quality
                audio_bitrate: Some("320k".to_string()),
                audio_sample_rate: Some(48000),
                hw_accel: true,
                extract_bwf: false,
                map_all_audio: true,
                ..Default::default()
            },
        }
    }

    /// Get all built-in presets
    pub fn all_presets() -> HashMap<String, CodecPreset> {
        let mut presets = HashMap::new();
        
        let preset_list = vec![
            // Hardware-accelerated presets (FAST!)
            Self::dnxhr_lb_fast(),     // DNxHR LB with HW accel
            Self::h264_fast(),         // H.264 with HW accel
            Self::h264_hq_fast(),      // H.264 HQ with HW accel
            Self::hevc_fast(),         // HEVC with HW accel
            Self::hevc_hq_fast(),      // HEVC HQ with HW accel
            
            // Standard presets
            Self::prores_hq(),
            Self::prores_422(),
            Self::prores_lt(),
            Self::dnxhr_hqx(),
            Self::dnxhr_hq(),
            Self::dnxhr_sq(),
            Self::h264_high(),         // Software H.264 (slower but more compatible)
        ];

        for preset in preset_list {
            presets.insert(preset.name.clone(), preset);
        }

        presets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TranscodeConfig::default();
        assert_eq!(config.video_codec, VideoCodec::ProResKS);
        assert_eq!(config.audio_codec, AudioCodec::PCM24);
    }

    #[test]
    fn test_ffmpeg_args() {
        let config = TranscodeConfig::default();
        let args = config.to_ffmpeg_args("input.mxf", "output.mov");
        
        assert!(args.contains(&"-i".to_string()));
        assert!(args.contains(&"input.mxf".to_string()));
        assert!(args.contains(&"output.mov".to_string()));
    }

    #[test]
    fn test_presets() {
        let presets = CodecPreset::all_presets();
        assert!(presets.contains_key("ProRes HQ"));
        assert!(presets.contains_key("H.264 High"));
    }
}

