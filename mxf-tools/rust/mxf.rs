use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MxfMetadata {
    pub material_package_uid: String,
    pub file_package_uid: Option<String>,
    pub physical_source_package_uid: Option<String>,
    pub timecode: Option<String>,
    pub duration: Option<u64>,
    pub tracks: Vec<MxfTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MxfTrack {
    pub track_id: u32,
    pub track_type: String, // "video", "audio", "data"
    pub codec: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifyMobIdOptions {
    pub input_files: Vec<PathBuf>,
    pub target_mob_id: Option<String>,
    pub reference_file: Option<PathBuf>,
    pub output_dir: PathBuf,
    pub output_type: String, // "avid", "op1a", etc.
}

/// Get the path to the bundled MXF tools based on the platform
fn get_mxf_tool_path(tool_name: &str) -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    let base_path = PathBuf::from("/Users/Editor/Downloads/bmx-ebu/build/apps");
    
    #[cfg(target_os = "windows")]
    let base_path = PathBuf::from("C:\\Program Files\\bmx\\bin");
    
    #[cfg(target_os = "linux")]
    let base_path = PathBuf::from("/usr/local/bin");
    
    let tool_path = match tool_name {
        "mxf2raw" => base_path.join("mxf2raw").join(tool_name),
        "bmxtranswrap" => base_path.join("bmxtranswrap").join(tool_name),
        "raw2bmx" => base_path.join("raw2bmx").join(tool_name),
        _ => return Err(format!("Unknown tool: {}", tool_name)),
    };
    
    // Add .exe extension on Windows
    #[cfg(target_os = "windows")]
    let tool_path = tool_path.with_extension("exe");
    
    if tool_path.exists() {
        Ok(tool_path)
    } else {
        Err(format!("Tool not found: {:?}", tool_path))
    }
}

/// Extract metadata from an MXF file
pub fn extract_mxf_metadata<P: AsRef<Path>>(file_path: P) -> Result<MxfMetadata, String> {
    let file_path = file_path.as_ref();
    
    if !file_path.exists() {
        return Err(format!("File not found: {:?}", file_path));
    }
    
    let mxf2raw = get_mxf_tool_path("mxf2raw")?;
    
    let output = Command::new(&mxf2raw)
        .arg("-i")
        .arg("--avid")
        .arg(file_path)
        .output()
        .map_err(|e| format!("Failed to execute mxf2raw: {}", e))?;
    
    if !output.status.success() {
        return Err(format!(
            "mxf2raw failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse the output to extract metadata
    let mut metadata = MxfMetadata {
        material_package_uid: String::new(),
        file_package_uid: None,
        physical_source_package_uid: None,
        timecode: None,
        duration: None,
        tracks: Vec::new(),
    };
    
    for line in stdout.lines() {
        if line.contains("Material Package UID") {
            if let Some(uid) = line.split('=').nth(1) {
                metadata.material_package_uid = uid.trim().replace(".", "").replace("-", "");
            }
        } else if line.contains("File Source Package UID") {
            if let Some(uid) = line.split('=').nth(1) {
                metadata.file_package_uid = Some(uid.trim().replace(".", "").replace("-", ""));
            }
        } else if line.contains("Physical Source Package UID") {
            if let Some(uid) = line.split('=').nth(1) {
                metadata.physical_source_package_uid = Some(uid.trim().replace(".", "").replace("-", ""));
            }
        } else if line.contains("Start timecode") {
            if let Some(tc) = line.split('=').nth(1) {
                metadata.timecode = Some(tc.trim().to_string());
            }
        } else if line.contains("Duration") {
            if let Some(dur_str) = line.split('=').nth(1) {
                if let Ok(dur) = dur_str.trim().parse::<u64>() {
                    metadata.duration = Some(dur);
                }
            }
        }
    }
    
    if metadata.material_package_uid.is_empty() {
        return Err("Could not extract Material Package UID from file".to_string());
    }
    
    Ok(metadata)
}

/// Unify MOB IDs across multiple MXF files
pub fn unify_mob_ids(options: UnifyMobIdOptions) -> Result<Vec<PathBuf>, String> {
    if options.input_files.is_empty() {
        return Err("No input files provided".to_string());
    }
    
    // Determine the target MOB ID
    let target_mob_id = if let Some(mob_id) = options.target_mob_id {
        // Remove any dots or dashes
        mob_id.replace(".", "").replace("-", "")
    } else if let Some(ref_file) = options.reference_file {
        let metadata = extract_mxf_metadata(&ref_file)?;
        metadata.material_package_uid
    } else {
        // Use MOB ID from first file
        let metadata = extract_mxf_metadata(&options.input_files[0])?;
        metadata.material_package_uid
    };
    
    // Validate MOB ID length
    if target_mob_id.len() != 64 {
        return Err(format!(
            "Invalid MOB ID length: expected 64 hex chars, got {}",
            target_mob_id.len()
        ));
    }
    
    // Create output directory
    std::fs::create_dir_all(&options.output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;
    
    let bmxtranswrap = get_mxf_tool_path("bmxtranswrap")?;
    let mut output_files = Vec::new();
    
    // Process each file
    for input_file in &options.input_files {
        if !input_file.exists() {
            eprintln!("Warning: File not found: {:?}", input_file);
            continue;
        }
        
        let filename = input_file
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid filename: {:?}", input_file))?;
        
        let output_prefix = options.output_dir.join(format!("{}_unified", filename));
        let output_prefix_str = output_prefix.to_str()
            .ok_or_else(|| format!("Invalid output path: {:?}", output_prefix))?;
        
        // Check if file already has the target MOB ID
        let current_metadata = extract_mxf_metadata(input_file)?;
        if current_metadata.material_package_uid == target_mob_id {
            // Just copy the file
            let output_file = options.output_dir.join(format!("{}_unified.mxf", filename));
            std::fs::copy(input_file, &output_file)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
            output_files.push(output_file);
            continue;
        }
        
        // Rewrap with new MOB ID
        let output = Command::new(&bmxtranswrap)
            .arg("-t")
            .arg(&options.output_type)
            .arg("-o")
            .arg(output_prefix_str)
            .arg("--mp-uid")
            .arg(&target_mob_id)
            .arg(input_file)
            .output()
            .map_err(|e| format!("Failed to execute bmxtranswrap: {}", e))?;
        
        if !output.status.success() {
            return Err(format!(
                "bmxtranswrap failed for {:?}: {}",
                input_file,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        // Determine the actual output filename (depends on type)
        let output_file = if options.output_type == "avid" {
            // Avid creates multiple files with suffixes
            options.output_dir.join(format!("{}_unified_v0.mxf", filename))
        } else {
            options.output_dir.join(format!("{}_unified.mxf", filename))
        };
        
        if output_file.exists() {
            output_files.push(output_file);
        }
    }
    
    Ok(output_files)
}

/// Check if MXF files belong together (have the same MOB ID)
pub fn check_mob_id_consistency<P: AsRef<Path>>(files: &[P]) -> Result<bool, String> {
    if files.is_empty() {
        return Ok(true);
    }
    
    let first_metadata = extract_mxf_metadata(&files[0])?;
    let target_mob_id = first_metadata.material_package_uid;
    
    for file in &files[1..] {
        let metadata = extract_mxf_metadata(file)?;
        if metadata.material_package_uid != target_mob_id {
            return Ok(false);
        }
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mob_id_length_validation() {
        let options = UnifyMobIdOptions {
            input_files: vec![PathBuf::from("test.mxf")],
            target_mob_id: Some("invalid".to_string()),
            reference_file: None,
            output_dir: PathBuf::from("output"),
            output_type: "avid".to_string(),
        };
        
        let result = unify_mob_ids(options);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid MOB ID length"));
    }
}

