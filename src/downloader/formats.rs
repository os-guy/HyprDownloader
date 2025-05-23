use anyhow::{Result, Context};
use std::process::Command;
use serde_json::Value;
use std::str;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct FormatOption {
    pub format_id: String,
    pub resolution: String,
    pub fps: u32,
    pub ext: String,
    pub format_note: String,
    pub vcodec: String,
    pub acodec: String,
    pub abr: u32,     // Audio bitrate in kbps
    pub is_audio_only: bool,
}

#[derive(Debug)]
pub enum FormatMessage {
    Formats(Vec<FormatOption>),
    Error,
}

pub fn fetch_available_formats(url: &str) -> Result<Vec<FormatOption>> {
    println!("Fetching formats for URL: {}", url);
    
    // Use --list-formats to get a full list of all available formats
    let output = Command::new("yt-dlp")
        .arg("--dump-json")
        .arg("--no-playlist")
        .arg(url)
        .output()
        .context("Failed to execute yt-dlp command")?;
    
    if !output.status.success() {
        let stderr = str::from_utf8(&output.stderr)
            .unwrap_or("Unable to decode error output");
        println!("yt-dlp error: {}", stderr);
        return Err(anyhow::anyhow!("yt-dlp failed: {}", stderr));
    }
    
    println!("Successfully fetched data from yt-dlp");
    
    let json_str = str::from_utf8(&output.stdout)
        .context("Failed to decode yt-dlp output")?;
    
    let json: Value = serde_json::from_str(json_str)
        .context("Failed to parse JSON from yt-dlp output")?;
    
    let mut formats = Vec::new();
    let mut found_extensions = HashSet::new();
    
    if let Some(format_array) = json["formats"].as_array() {
        println!("Found {} format entries", format_array.len());
        
        // First, log all available extensions for debugging
        for format in format_array.iter() {
            if let Some(ext) = format["ext"].as_str() {
                found_extensions.insert(ext.to_string());
            }
        }
        
        println!("Available extensions in response: {:?}", found_extensions);
        
        // Log all available audio codecs and bitrates
        println!("\nAudio format details from API response:");
        for (i, format) in format_array.iter().enumerate() {
            let acodec = format["acodec"].as_str().unwrap_or("none");
            if acodec != "none" {
                let format_id = format["format_id"].as_str().unwrap_or("");
                let abr = format["abr"].as_u64().unwrap_or(0);
                let tbr = format["tbr"].as_u64().unwrap_or(0); // Total bitrate might include audio info
                let asr = format["asr"].as_u64().unwrap_or(0); // Audio sampling rate
                let format_note = format["format_note"].as_str().unwrap_or("");
                let ext = format["ext"].as_str().unwrap_or("");
                
                println!("Format {}: id={}, codec={}, ext={}, abr={}kbps, tbr={}kbps, asr={}, note='{}'",
                    i, format_id, acodec, ext, abr, tbr, asr, format_note);
            }
        }
        println!("End of audio format details\n");
        
        // Process each format entry
        for (i, format) in format_array.iter().enumerate() {
            let vcodec = format["vcodec"].as_str().unwrap_or("none").to_string();
            let acodec = format["acodec"].as_str().unwrap_or("none").to_string();
            let is_audio_only = vcodec == "none" && acodec != "none";
            
            // Skip formats with no audio and no video
            if vcodec == "none" && acodec == "none" {
                println!("Skipping format {} - no audio or video codec", i);
                continue;
            }
            
            let format_id = format["format_id"].as_str().unwrap_or("").to_string();
            
            // Get video resolution
            let width = format["width"].as_u64().unwrap_or(0);
            let height = format["height"].as_u64().unwrap_or(0);
            let resolution = if width > 0 && height > 0 {
                format!("{}x{}", width, height)
            } else {
                "audio only".to_string()
            };
            
            // Get FPS (only relevant for video)
            let fps = if !is_audio_only {
                format["fps"].as_u64().unwrap_or(0) as u32
            } else {
                0 // Audio-only formats don't have FPS
            };
            
            // Get audio bitrate (primarily for audio formats)
            let abr = format["abr"].as_u64().unwrap_or(0) as u32;
            
            // If bitrate is 0, try to use a default value based on codec
            let effective_abr = if abr == 0 && acodec != "none" {
                // Assign a reasonable default based on codec
                if acodec.starts_with("mp4a") {
                    192  // AAC audio typically around 192kbps
                } else if acodec.starts_with("opus") {
                    160  // Opus is efficient, so lower bitrate is fine
                } else if acodec.starts_with("mp3") {
                    128  // MP3 default
                } else {
                    128  // Generic default
                }
            } else {
                abr
            };
            
            // Get extension (actual file format)
            let ext = format["ext"].as_str().unwrap_or("mp4").to_string();
            
            // Get format note (quality description)
            let format_note = format["format_note"].as_str().unwrap_or("").to_string();
            
            if is_audio_only {
                println!("Adding audio format: id={}, bitrate={}kbps, ext={}, acodec={}",
                    format_id, effective_abr, ext, acodec);
            } else {
                println!("Adding video format: id={}, resolution={}, fps={}, ext={}, vcodec={}",
                    format_id, resolution, fps, ext, vcodec);
            }
            
            formats.push(FormatOption {
                format_id,
                resolution,
                fps,
                ext,
                format_note,
                vcodec,
                acodec,
                abr: effective_abr,
                is_audio_only,
            });
        }
    } else {
        println!("No formats array found in JSON response");
        println!("Full response: {}", json_str);
    }
    
    // Log all unique extensions found
    let mut unique_exts = HashSet::new();
    for format in &formats {
        unique_exts.insert(&format.ext);
    }
    println!("Unique extensions in processed formats: {:?}", unique_exts);
    
    // Check if all formats have zero FPS
    let all_zero_fps = formats.iter()
        .filter(|f| !f.is_audio_only)
        .all(|f| f.fps == 0);
    
    // Only filter by FPS if we have some formats with non-zero FPS
    if !all_zero_fps {
        // Filter out formats with 0 FPS for video formats only
        formats.retain(|f| f.is_audio_only || f.fps > 0);
        println!("After filtering zero FPS: {} formats remain", formats.len());
    } else {
        println!("All video formats have zero FPS - skipping FPS filtering");
    }
    
    // Sort by resolution (height), FPS, and then file extension
    formats.sort_by(|a, b| {
        if a.is_audio_only && !b.is_audio_only {
            return std::cmp::Ordering::Greater;
        } else if !a.is_audio_only && b.is_audio_only {
            return std::cmp::Ordering::Less;
        }
        
        if a.is_audio_only && b.is_audio_only {
            // Sort audio formats by bitrate (descending)
            return b.abr.cmp(&a.abr)
                .then(a.ext.cmp(&b.ext));
        } else {
            // Sort video formats by resolution, FPS, and extension
            let a_height: u32 = a.resolution.split('x').nth(1)
                .and_then(|h| h.parse().ok())
                .unwrap_or(0);
            let b_height: u32 = b.resolution.split('x').nth(1)
                .and_then(|h| h.parse().ok())
                .unwrap_or(0);
            
            return b_height.cmp(&a_height)
                .then(b.fps.cmp(&a.fps))
                .then(a.ext.cmp(&b.ext));
        }
    });
    
    Ok(formats)
} 