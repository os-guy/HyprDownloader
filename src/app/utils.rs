use anyhow::Result;
use std::path::PathBuf;
use std::fs;

pub fn get_default_download_path() -> Result<String> {
    let download_dir = dirs::download_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find downloads directory"))?;
    
    // Create a HyprDownloader folder inside the downloads directory
    let hypr_dir = download_dir.join("HyprDownloader");
    
    // Create main directory if it doesn't exist
    if !hypr_dir.exists() {
        fs::create_dir_all(&hypr_dir)?;
    }
    
    // Create subdirectories for audio and video
    let audio_dir = hypr_dir.join("audio");
    let video_dir = hypr_dir.join("video");
    
    if !audio_dir.exists() {
        fs::create_dir_all(&audio_dir)?;
    }
    
    if !video_dir.exists() {
        fs::create_dir_all(&video_dir)?;
    }
    
    Ok(hypr_dir.to_string_lossy().to_string())
} 