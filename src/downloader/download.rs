use anyhow::Result;
use std::process::{Command, Child, Stdio};
use std::io::{BufReader, BufRead};

pub fn download_media_with_format(url: &str, output_path: &str, format_id: &str) -> Result<Child> {
    // Start the yt-dlp process with the selected format and capture stdout/stderr
    let child = Command::new("yt-dlp")
        .arg("-f")
        .arg(format_id)
        .arg(url)
        .arg("-P")
        .arg(output_path)
        .arg("--newline")  // Force newlines to ensure consistent output
        .arg("--progress")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    Ok(child)
}

pub fn download_audio_only(url: &str, output_path: &str, format_id: &str) -> Result<Child> {
    // Start the yt-dlp process with the selected format and extract audio
    let child = Command::new("yt-dlp")
        .arg("-f")
        .arg(format_id)
        .arg("-x")  // Extract audio
        .arg("--audio-format")
        .arg("best")  // Use the best audio format available
        .arg("--audio-quality")
        .arg("0")   // Best quality
        .arg("--audio-format-choices")
        .arg("m4a/mp3/aac/opus/vorbis/webm")  // Provide format choices in preferred order
        .arg(url)
        .arg("-P")
        .arg(output_path)
        .arg("--newline")  // Force newlines to ensure consistent output
        .arg("--progress")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    Ok(child)
}

// New function to download audio with specific format
pub fn download_audio_with_format(url: &str, output_path: &str, format_id: &str, audio_format: &str) -> Result<Child> {
    // Start the yt-dlp process with specific audio format extraction
    let child = Command::new("yt-dlp")
        .arg("-f")
        .arg(format_id)
        .arg("-x")  // Extract audio
        .arg("--audio-format")
        .arg(audio_format)  // Specific format like m4a, mp3, etc.
        .arg("--audio-quality") 
        .arg("0")  // Best quality
        .arg(url)
        .arg("-P")
        .arg(output_path)
        .arg("--newline")  // Force newlines to ensure consistent output
        .arg("--progress")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    Ok(child)
}

// Process phases for tracking download progress
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DownloadPhase {
    Initializing,   // 0-5%
    Downloading,    // 5-80%
    Postprocessing, // 80-95%
    Finalizing,     // 95-99%
    Complete        // 100%
}

// Track the actual percentage and phase
#[derive(Debug, Clone)]
pub struct ProgressState {
    pub phase: DownloadPhase,
    pub download_percent: f64,
    pub overall_percent: f64,
    pub status_message: String,
}

impl Default for ProgressState {
    fn default() -> Self {
        ProgressState {
            phase: DownloadPhase::Initializing,
            download_percent: 0.0,
            overall_percent: 0.0,
            status_message: "Initializing download...".to_string(),
        }
    }
}

// Parse a line from yt-dlp output and update the progress state
pub fn update_progress_state(line: &str, state: &mut ProgressState) -> bool {
    let mut updated = false;

    // Initial phases
    if line.contains("[info]") && !line.contains("Downloading") {
        state.phase = DownloadPhase::Initializing;
        state.overall_percent = 0.05;
        state.status_message = "Gathering information...".to_string();
        updated = true;
    }

    // During download phase - parse percentage
    if line.contains("[download]") && line.contains("%") {
        // Extract percentage from line
        if let Some(percent_index) = line.find('%') {
            // Find start of percentage number
            let mut start_index = percent_index;
            while start_index > 0 {
                start_index -= 1;
                let c = line.chars().nth(start_index).unwrap_or(' ');
                if !c.is_digit(10) && c != '.' {
                    start_index += 1;
                    break;
                }
            }
            
            // Extract and parse the percentage
            if let Ok(percent) = line[start_index..percent_index].parse::<f64>() {
                state.phase = DownloadPhase::Downloading;
                state.download_percent = percent / 100.0;
                
                // Map download percentage (0-100%) to overall percentage (5-80%)
                state.overall_percent = 0.05 + (state.download_percent * 0.75);
                
                // Extract speed and ETA if present
                let mut status = format!("Downloading: {:.1}%", percent);
                
                if let Some(speed_index) = line.find("at ") {
                    if let Some(eta_index) = line.find("ETA") {
                        let speed = &line[speed_index + 3..line[speed_index..].find(" ETA").map_or(line.len(), |i| i + speed_index)];
                        let eta = &line[eta_index + 4..];
                        status = format!("Downloading: {:.1}% ({}, ETA: {})", percent, speed, eta);
                    }
                }
                
                state.status_message = status;
                updated = true;
            }
        }
    }

    // Full download complete (100%)
    if line.contains("[download]") && line.contains("100%") {
        state.phase = DownloadPhase::Postprocessing;
        state.download_percent = 1.0;
        state.overall_percent = 0.8;
        state.status_message = "Download complete, processing...".to_string();
        updated = true;
    }

    // Post-processing phases
    if line.contains("[Merger]") {
        state.phase = DownloadPhase::Postprocessing;
        state.overall_percent = 0.85;
        state.status_message = "Merging formats...".to_string();
        updated = true;
    }

    if line.contains("[ExtractAudio]") {
        state.phase = DownloadPhase::Postprocessing;
        state.overall_percent = 0.85;
        state.status_message = "Extracting audio...".to_string();
        updated = true;
    }

    if line.contains("[ffmpeg]") {
        state.phase = DownloadPhase::Postprocessing; 
        state.overall_percent = 0.9;
        state.status_message = "Processing with ffmpeg...".to_string();
        updated = true;
    }

    if line.contains("Deleting original file") {
        state.phase = DownloadPhase::Finalizing;
        state.overall_percent = 0.95;
        state.status_message = "Cleaning up temporary files...".to_string();
        updated = true;
    }

    // Final completion indicators
    if line.contains("has already been downloaded") {
        state.phase = DownloadPhase::Complete;
        state.overall_percent = 1.0;
        state.status_message = "File was already downloaded".to_string();
        updated = true;
    }

    if (line.contains("Downloaded") && line.contains("bytes") && !line.contains("%")) || 
       (line.contains("Destination: ") && line.contains(".") && !line.contains("[download]")) {
        state.phase = DownloadPhase::Complete;
        state.overall_percent = 1.0;
        state.status_message = "Download complete!".to_string();
        updated = true;
    }

    updated
} 