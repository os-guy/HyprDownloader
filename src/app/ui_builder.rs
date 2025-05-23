use gtk4::{
    prelude::*,
    Application, ApplicationWindow, Box as GtkBox, 
    Orientation, Align, Stack, StackSwitcher, Label, Entry, Button
};
use std::thread;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::BufRead;
use glib;
use std::collections::HashSet;
use rand;

use crate::ui::style;
use crate::ui::components;
use crate::downloader::{fetch_available_formats, FormatMessage};
use crate::app::get_default_download_path;

// Map resolution to (FPS, format_id, ext) list for video
type ResolutionMap = Vec<(String, Vec<(u32, String, String)>)>;

// Map bitrate to (format_id, ext) list for audio
type AudioBitrateMap = Vec<(u32, Vec<(String, String)>)>;

pub fn build_ui(app: &Application) {
    // Load CSS
    style::load_css();

    // Create a main container
    let main_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();
    
    main_container.add_css_class("main-container");

    // Header container with app branding
    let header_container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_top(10)
        .margin_bottom(4)
        .margin_start(16)
        .margin_end(16)
        .build();

    // Left side of header - Title and subtitle
    let header_left = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .hexpand(true)
        .build();
        
    // Title and subtitle
    let header = components::create_title_with_subtitle(
        "HyprDownloader",
        "Download videos and media with your preferred quality and format"
    );
    
    header_left.append(&header);
    
    // Right side of header - About button
    let about_button = Button::builder()
        .label("About")
        .valign(Align::Center)
        .margin_start(8)
        .build();
    
    about_button.add_css_class("about-button");
    
    // Add sides to header container
    header_container.append(&header_left);
    header_container.append(&about_button);
    
    // Create a content container with padding
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_start(16)
        .margin_end(16)
        .margin_bottom(16)
        .build();

    container.add_css_class("content-box");

    // URL section with fetch button in a standalone card
    let url_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .margin_bottom(8)
        .build();
    
    url_card.add_css_class("url-card");
    
    let url_section_title = components::create_section_title("Media Source");
    
    // Create a horizontal box for URL label, entry and fetch button
    let url_input_container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
        
    // URL label
    let url_label = Label::builder()
        .label("URL:")
        .halign(Align::Start)
        .valign(Align::Center)
        .width_request(40)
        .build();
        
    url_label.add_css_class("input-label");
        
    // URL entry
    let url_entry = Entry::builder()
        .placeholder_text("Enter URL here (YouTube, Vimeo, etc.)")
        .hexpand(true)
        .build();
        
    url_entry.add_css_class("url-entry");
        
    // Create fetch button
    let fetch_button = components::create_button("Fetch", "fetch-button");
    
    // Make fetch button height match the text box
    fetch_button.set_valign(Align::Fill);
    
    // Add components to the horizontal container
    url_input_container.append(&url_label);
    url_input_container.append(&url_entry);
    url_input_container.append(&fetch_button);
    
    // Add components to url card
    url_card.append(&url_section_title);
    url_card.append(&url_input_container);
    
    // Spinner and status
    let (spinner_box, spinner, status_label) = components::create_spinner_with_label();
    url_card.append(&spinner_box);
    
    // Add URL card to the main container
    container.append(&url_card);
    
    // Main content area - using a card-based design with two columns layout
    let main_area = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(4)
        .build();
    
    // Left side - Media options (video/audio)
    let left_panel = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .hexpand(true)
        .build();
    
    left_panel.add_css_class("options-panel");
    
    // Quality section with modern card style
    let quality_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();
        
    quality_card.add_css_class("card");
    
    let quality_section_title = components::create_section_title("Media Options");
    
    // Create a container for video and audio options
    let media_options_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .hexpand(true)
        .build();
    
    // Create a Stack for switchable media options
    let media_stack = Stack::builder()
        .transition_type(gtk4::StackTransitionType::SlideLeft)
        .build();
    
    // Create a stack switcher (tabs) for media options
    let media_stack_switcher = StackSwitcher::builder()
        .stack(&media_stack)
        .halign(Align::Center)
        .build();
    
    // Add switcher to media options container
    media_options_container.append(&media_stack_switcher);
    
    // Video options container
    let video_options_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(6)
        .hexpand(true)
        .build();
        
    video_options_container.add_css_class("options-box");
    
    // Quality and FPS selection in a grid layout
    let video_grid = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .hexpand(true)
        .build();

    // Quality selection row
    let quality_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .hexpand(true)
        .build();
    
    // Quality selection
    let (quality_box, quality_combo) = components::create_dropdown("Quality:");
    quality_box.set_hexpand(true);
    
    // FPS selection
    let (fps_box, fps_combo) = components::create_dropdown("FPS:");
    fps_box.set_hexpand(true);
    
    quality_row.append(&quality_box);
    quality_row.append(&fps_box);
    
    // Format Type selection row
    let format_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .hexpand(true)
        .build();
    
    let (format_box, format_combo) = components::create_dropdown("Format:");
    format_box.set_hexpand(true);
    
    format_row.append(&format_box);
    
    // Add rows to the grid
    video_grid.append(&quality_row);
    video_grid.append(&format_row);
    
    // Add grid to video options
    video_options_container.append(&video_grid);
    
    // Audio options container
    let audio_options_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(6)
        .hexpand(true)
        .build();
        
    audio_options_container.add_css_class("options-box");
    
    // Audio quality and format selection in a grid
    let audio_grid = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .hexpand(true)
        .build();
    
    // Audio quality row
    let audio_quality_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .hexpand(true)
        .build();
        
    // Audio quality selection
    let (audio_quality_box, audio_quality_combo) = components::create_dropdown("Quality (kbps):");
    audio_quality_box.set_hexpand(true);
    
    audio_quality_row.append(&audio_quality_box);
    
    // Audio format row
    let audio_format_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .hexpand(true)
        .build();
    
    // Audio format selection
    let (audio_format_box, audio_format_combo) = components::create_dropdown("Format:");
    audio_format_box.set_hexpand(true);
    
    audio_format_row.append(&audio_format_box);
    
    // Add rows to the audio grid
    audio_grid.append(&audio_quality_row);
    audio_grid.append(&audio_format_row);
    
    // Add grid to audio options container
    audio_options_container.append(&audio_grid);
    
    // Add pages to the media stack
    media_stack.add_titled(&video_options_container, Some("video"), "Video");
    media_stack.add_titled(&audio_options_container, Some("audio"), "Audio");
    
    // Add stack to media options container
    media_options_container.append(&media_stack);
    
    // Format info message
    let format_info = components::create_status_label("Select quality, FPS and file format");
    format_info.set_margin_top(4);
    
    // Add components to quality card
    quality_card.append(&quality_section_title);
    quality_card.append(&media_options_container);
    quality_card.append(&format_info);
    
    // Add quality card to left panel
    left_panel.append(&quality_card);
    
    // Right side - info boxes and download controls
    let right_panel = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .width_request(260)
        .build();
    
    right_panel.add_css_class("info-panel");
    
    // Info card
    let info_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();
        
    info_card.add_css_class("card");
    
    // Create a Stack for switchable info content
    let info_stack = Stack::builder()
        .transition_type(gtk4::StackTransitionType::SlideLeft)
        .build();
    
    // Create a stack switcher (tabs)
    let stack_switcher = StackSwitcher::builder()
        .stack(&info_stack)
        .halign(Align::Center)
        .build();
    
    let info_title = components::create_section_title("Information");
    
    // Add title and switcher to info card
    info_card.append(&info_title);
    info_card.append(&stack_switcher);
    
    // URL helper info container
    let url_info_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(6)
        .build();
    
    // URL helper info
    let url_info = components::create_info_box(
        "Supported Sites",
        "YouTube, Vimeo, Dailymotion, and many more sites supported by yt-dlp."
    );
    
    url_info_container.append(&url_info);
    
    // Format info container
    let format_info_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(6)
        .build();
    
    // Create info box for the format info
    let format_info_box = components::create_info_box(
        "Format Selection",
        "Select resolution and FPS for best viewing experience."
    );
    
    format_info_container.append(&format_info_box);
    
    // Help info container
    let help_info_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(6)
        .build();
    
    // Create help info box
    let help_info_box = components::create_info_box(
        "Quick Help",
        "1. Enter URL\n2. Fetch\n3. Select quality\n4. Download"
    );
    
    help_info_container.append(&help_info_box);
    
    // Add pages to the stack
    info_stack.add_titled(&url_info_container, Some("sites"), "Sites");
    info_stack.add_titled(&format_info_container, Some("formats"), "Formats");
    info_stack.add_titled(&help_info_container, Some("help"), "Help");
    
    // Add stack to info card
    info_card.append(&info_stack);
    
    // Add info card to right panel
    right_panel.append(&info_card);
    
    // Download card
    let download_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();
        
    download_card.add_css_class("card");
    
    let download_title = components::create_section_title("Download Options");
    download_card.append(&download_title);
    
    // Output path selection row
    let default_path = get_default_download_path().unwrap_or_else(|_| String::from("."));
    
    // Create the label
    let path_label = Label::builder()
        .label("Download folder:")
        .halign(Align::Start)
        .build();
    
    path_label.add_css_class("input-label");
    
    // Create the entry field
    let path_entry = Entry::builder()
        .placeholder_text("Path to save the downloaded media")
        .text(&default_path)
        .build();
    
    path_entry.add_css_class("path-entry");
    path_entry.set_hexpand(true);
    
    // Create a browse button for selecting the output directory
    let browse_button = components::create_button("Browse", "browse-button");
    
    // Create a container for the label
    let label_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(3)
        .build();
    
    label_container.append(&path_label);
    
    // Create a horizontal box to hold the entry and browse button
    let entry_with_button = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    
    // Add the entry and button to the horizontal box
    entry_with_button.append(&path_entry);
    entry_with_button.append(&browse_button);
    
    // Add the entry+button to the label container
    label_container.append(&entry_with_button);
    
    // Add path selection to download card
    download_card.append(&label_container);
    
    // Path entry clones for various handlers
    let path_entry_for_download = path_entry.clone();
    let path_entry_for_open = path_entry.clone();
    let path_entry_for_browse = path_entry.clone();
    let path_entry_for_fetch = path_entry.clone();
    let path_entry_for_tab_change = path_entry.clone();
    
    // Download button with action area
    let download_button_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(6)
        .build();
    
    // Download button
    let download_button = components::create_button("Download Media", "download-button");
    download_button.set_sensitive(false);
    download_button.set_hexpand(true);
    
    // Download status
    let download_status = components::create_status_label("Ready to download");
    
    // Add a progress bar for download progress
    let progress_bar = gtk4::ProgressBar::builder()
        .show_text(true)
        .text("0%")
        .fraction(0.0)
        .visible(false)
        .build();
        
    progress_bar.add_css_class("download-progress");
    
    // Create Open Folder button (initially hidden)
    let open_folder_button = components::create_button("Open Folder", "open-folder-button");
    open_folder_button.set_visible(false);
    open_folder_button.set_hexpand(true);
    
    // Add download components to container
    download_button_container.append(&download_button);
    download_button_container.append(&download_status);
    download_button_container.append(&progress_bar);
    download_button_container.append(&open_folder_button);
    
    // Add download components to download card
    download_card.append(&download_button_container);
    
    // Add download card to right panel
    right_panel.append(&download_card);
    
    // Add panels to main area
    main_area.append(&left_panel);
    main_area.append(&right_panel);
    
    // Add main area to container
    container.append(&main_area);
    
    // Create the main window with properties that help with floating
    let window = ApplicationWindow::builder()
        .application(app)
        .title("HyprDownloader")
        .default_width(800)
        .default_height(530)
        .resizable(true)
        .decorated(true)
        .modal(false)
        .child(&main_container)
        .build();
    
    // Add CSS class for styling
    window.add_css_class("floating-window");
    
    // Set window properties
    window.set_destroy_with_parent(false);
    window.set_hide_on_close(false);
    
    // Create a shared resolution map for the form
    let resolution_map = Rc::new(RefCell::new(ResolutionMap::new()));
    
    // Create a shared audio bitrate map
    let audio_bitrate_map = Rc::new(RefCell::new(AudioBitrateMap::new()));

    // Connect fetch button
    let url_entry_clone = url_entry.clone();
    let spinner_clone = spinner.clone();
    let status_label_clone = status_label.clone();
    let quality_combo_clone = quality_combo.clone();
    let fps_combo_clone = fps_combo.clone();
    let format_combo_clone = format_combo.clone();
    let audio_quality_combo_clone = audio_quality_combo.clone();
    let audio_format_combo_clone = audio_format_combo.clone();
    let download_button_clone = download_button.clone();
    let format_info_clone = format_info.clone();
    let resolution_map_clone = Rc::clone(&resolution_map);
    let audio_bitrate_map_clone = Rc::clone(&audio_bitrate_map);
    let media_stack_for_fetch = media_stack.clone();
    let window_clone = window.clone();
    
    fetch_button.connect_clicked(move |_| {
        let url = url_entry_clone.text().to_string();
        
        if url.is_empty() {
            components::show_error_dialog(&window_clone, "Invalid URL", "Please enter a URL");
            status_label_clone.set_text("Please enter a URL");
            status_label_clone.set_visible(true);
            return;
        }
        
        // Show processing UI
        status_label_clone.set_text("Fetching available formats...");
        status_label_clone.set_visible(true);
        spinner_clone.start();
        
        // Reset UI and state
        {
            let mut map = resolution_map_clone.borrow_mut();
            map.clear();
            
            let mut audio_map = audio_bitrate_map_clone.borrow_mut();
            audio_map.clear();
        }
        
        quality_combo_clone.remove_all();
        fps_combo_clone.remove_all();
        format_combo_clone.remove_all();
        audio_quality_combo_clone.remove_all();
        audio_format_combo_clone.remove_all();
        
        quality_combo_clone.set_sensitive(false);
        fps_combo_clone.set_sensitive(false);
        format_combo_clone.set_sensitive(false);
        audio_quality_combo_clone.set_sensitive(false);
        audio_format_combo_clone.set_sensitive(false);
        download_button_clone.set_sensitive(false);
        format_info_clone.set_text("Analyzing media source...");
        
        // Create a channel to communicate between threads
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        
        // Spawn a thread to fetch formats
        let url_clone = url.clone();
        thread::spawn(move || {
            match fetch_available_formats(&url_clone) {
                Ok(fetched_formats) => {
                    let _ = sender.send(FormatMessage::Formats(fetched_formats));
                },
                Err(e) => {
                    println!("Error fetching formats: {:?}", e);
                    let _ = sender.send(FormatMessage::Error);
                }
            }
        });
        
        // Prepare for the receiver
        let quality_combo = quality_combo_clone.clone();
        let fps_combo = fps_combo_clone.clone();
        let format_combo = format_combo_clone.clone();
        let audio_quality_combo = audio_quality_combo_clone.clone();
        let audio_format_combo = audio_format_combo_clone.clone();
        let status_label = status_label_clone.clone();
        let format_info = format_info_clone.clone();
        let download_button = download_button_clone.clone();
        let spinner = spinner_clone.clone();
        let resolution_map = Rc::clone(&resolution_map_clone);
        let audio_bitrate_map = Rc::clone(&audio_bitrate_map_clone);
        let window_clone = window_clone.clone();
        let path_entry_clone = path_entry_for_fetch.clone();
        let media_stack_for_fetch = media_stack_for_fetch.clone();
        
        // Handle messages from the thread
        receiver.attach(None, move |message| {
            match message {
                FormatMessage::Formats(formats) => {
                    // Separate video and audio formats
                    let video_formats: Vec<_> = formats.iter()
                        .filter(|format| !format.is_audio_only)
                        .collect();
                        
                    let audio_formats: Vec<_> = formats.iter()
                        .filter(|format| format.is_audio_only)
                        .collect();
                        
                    println!("Found {} video formats and {} audio formats", 
                        video_formats.len(), audio_formats.len());
                    
                    // Process video formats
                    {
                        let mut map = resolution_map.borrow_mut();
                        map.clear();
                        
                        // Organize formats by resolution and FPS
                        for format in &video_formats {
                            let resolution = &format.resolution;
                            let fps = format.fps;
                            let format_id = &format.format_id;
                            let ext = &format.ext;
                            
                            // Don't skip zero FPS formats - they might be valid
                            // if fps == 0 {
                            //     continue;  // Skip formats without FPS information
                            // }
                            
                            // Find if this resolution is already in our map
                            let pos = map.iter().position(|(res, _)| res == resolution);
                            
                            match pos {
                                Some(idx) => {
                                    // Check if this FPS and extension combination is already in the list
                                    let fps_list = &mut map[idx].1;
                                    if !fps_list.iter().any(|(existing_fps, _, existing_ext)| 
                                        *existing_fps == fps && existing_ext == ext) {
                                        fps_list.push((fps, format_id.clone(), ext.clone()));
                                    }
                                },
                                None => {
                                    // Add this resolution with its first FPS
                                    map.push((resolution.clone(), vec![(fps, format_id.clone(), ext.clone())]));
                                }
                            }
                        }
                        
                        // Sort resolutions by height (descending)
                        map.sort_by(|(a, _), (b, _)| {
                            let a_height: u32 = a.split('x').nth(1)
                                .and_then(|h| h.parse().ok())
                                .unwrap_or(0);
                            let b_height: u32 = b.split('x').nth(1)
                                .and_then(|h| h.parse().ok())
                                .unwrap_or(0);
                            
                            b_height.cmp(&a_height)
                        });
                        
                        // Sort FPS values for each resolution (descending)
                        for (_, fps_list) in map.iter_mut() {
                            fps_list.sort_by(|(a, _, _), (b, _, _)| b.cmp(a));
                        }
                    }
                    
                    // Process audio formats
                    {
                        let mut map = audio_bitrate_map.borrow_mut();
                        map.clear();
                        
                        // First try to use dedicated audio formats
                        if !audio_formats.is_empty() {
                            println!("Processing {} dedicated audio formats", audio_formats.len());
                        for format in &audio_formats {
                            let bitrate = format.abr;
                            let format_id = &format.format_id;
                            let ext = &format.ext;
                            let acodec = &format.acodec;
                            
                                println!("Audio format: id={}, codec={}, bitrate={}kbps, ext={}", 
                                format_id, acodec, bitrate, ext);
                            
                                // Skip formats with 0 bitrate after our processing
                                if bitrate == 0 {
                                    println!("Skipping audio format with 0 bitrate: {}", format_id);
                                    continue;
                                }
                            
                            // Find if this bitrate is already in our map
                                let pos = map.iter().position(|(br, _)| *br == bitrate);
                            
                            match pos {
                                Some(idx) => {
                                        // Add this format ID to the existing bitrate
                                        map[idx].1.push((format_id.clone(), ext.clone()));
                                        println!("Added to existing bitrate {}kbps", bitrate);
                                },
                                None => {
                                    // Add this bitrate with its first format
                                        map.push((bitrate, vec![(format_id.clone(), ext.clone())]));
                                        println!("Added new bitrate {}kbps", bitrate);
                                }
                            }
                        }
                        } else {
                            // No audio-only formats found, extract audio info from video formats
                            println!("No audio-only formats found, looking for audio in video formats");
                            
                            // Collect all non-zero audio bitrates from video formats
                            let mut audio_bitrates = HashSet::new();
                            for format in &video_formats {
                                if format.acodec != "none" {
                                    // Only use non-zero bitrates
                                    if format.abr > 0 {
                                        audio_bitrates.insert(format.abr);
                                        println!("Found audio in video format: id={}, bitrate={}kbps, codec={}", 
                                            format.format_id, format.abr, format.acodec);
                                        } else {
                                        println!("Skipping zero bitrate audio in video format: id={}, codec={}", 
                                            format.format_id, format.acodec);
                                    }
                                }
                            }
                            
                            println!("Audio bitrates found:");
                            for bitrate in &audio_bitrates {
                                println!("  - {} kbps", bitrate);
                                
                                // Find all format_ids that provide this audio bitrate
                                let format_pairs: Vec<(String, String)> = video_formats.iter()
                                    .filter(|f| f.abr == *bitrate && f.acodec != "none")
                                    .map(|f| (f.format_id.clone(), f.ext.clone()))
                                    .collect();
                                
                                map.push((*bitrate, format_pairs));
                            }
                        }
                        
                        // Sort audio bitrates (descending)
                        map.sort_by(|(a, _), (b, _)| b.cmp(a));
                    }
                    
                    // Populate the resolution combo box
                    {
                        let map = resolution_map.borrow();
                        
                        if map.is_empty() {
                            status_label.set_text("No suitable video formats found");
                            format_info.set_text("Try a different URL or check if the media source is valid");
                        } else {
                            for (i, (resolution, _)) in map.iter().enumerate() {
                                quality_combo.append(Some(&i.to_string()), resolution);
                            }
                            
                            // If we have resolutions, activate the first one and populate FPS
                            quality_combo.set_sensitive(true);
                            
                            // Set the first resolution as active
                            quality_combo.set_active(Some(0));
                        }
                    }
                    
                    // Populate the audio bitrate combo box
                    {
                        let audio_map = audio_bitrate_map.borrow();
                        
                        if audio_map.is_empty() {
                            println!("No suitable audio formats found");
                        } else {
                            for (i, (bitrate, _)) in audio_map.iter().enumerate() {
                                audio_quality_combo.append(Some(&i.to_string()), &format!("{} kbps", bitrate));
                            }
                            
                            // If we have bitrates, activate the first one and populate formats
                            audio_quality_combo.set_sensitive(true);
                            
                            // Set the first bitrate as active
                            audio_quality_combo.set_active(Some(0));
                            
                            // Populate audio format options (m4a, mp3, etc.)
                            audio_format_combo.remove_all();
                            
                            // First collect the available extensions from the formats
                            let mut available_extensions = HashSet::new();
                            for format_list in audio_map.iter() {
                                for (_, ext) in &format_list.1 {
                                    available_extensions.insert(ext.clone().to_lowercase());
                                }
                            }
                            
                            println!("Available audio extensions: {:?}", available_extensions);
                            
                            // Common audio formats to check for
                            let preferred_audio_extensions = ["m4a", "mp3", "opus", "aac", "wav", "ogg"];
                            
                            // Add only the available formats in the preferred order
                            let mut found_formats = 0;
                            for ext in preferred_audio_extensions.iter() {
                                if available_extensions.contains(&ext.to_string()) {
                                    audio_format_combo.append(Some(ext), &ext.to_uppercase());
                                    found_formats += 1;
                                }
                            }
                            
                            // If we didn't find any of the preferred formats, add all available formats
                            if found_formats == 0 {
                                // Convert HashSet to Vec for predictable iteration
                                let exts: Vec<String> = available_extensions.into_iter().collect();
                                for ext_str in exts {
                                    audio_format_combo.append(Some(&ext_str), &ext_str.to_uppercase());
                                }
                            }
                        }
                    }
                    
                    status_label.set_text("Formats fetched successfully");
                    format_info.set_text("Select your preferred quality, FPS and file format");
                    
                    // Update the download path based on the active tab
                    let default_path = get_default_download_path().unwrap_or_else(|_| String::from("."));
                    let active_tab = media_stack_for_fetch.visible_child_name().unwrap_or_else(|| "video".into());
                    
                    // Get current path
                    let current_path = path_entry_clone.text().to_string();
                    
                    // Only change the path if it contains "HyprDownloader" (meaning it's using our default path structure)
                    // or if it's empty/default
                    let should_update_path = current_path.contains("HyprDownloader") || 
                                             current_path.is_empty() || 
                                             current_path == "." || 
                                             current_path == default_path;
                    
                    if should_update_path {
                        if active_tab == "video" {
                            // Set path to video subfolder
                            let video_path = format!("{}/video", default_path);
                            path_entry_clone.set_text(&video_path);
                        } else if active_tab == "audio" {
                            // Set path to audio subfolder
                            let audio_path = format!("{}/audio", default_path);
                            path_entry_clone.set_text(&audio_path);
                        }
                    }
                },
                FormatMessage::Error => {
                    status_label.set_text("Error fetching formats");
                    format_info.set_text("Check your URL or internet connection");
                    components::show_error_dialog(&window_clone, "URL Error", 
                        "Failed to fetch video formats. Please check if the URL is valid and your internet connection is working.");
                }
            }
            
            // Hide processing UI once done
            spinner.stop();
            glib::Continue(true)
        });
    });

    // Connect quality combo box to update FPS options
    let fps_combo_clone = fps_combo.clone();
    let format_combo_clone = format_combo.clone();
    let download_button_clone = download_button.clone();
    let format_info_clone = format_info.clone();
    let resolution_map_clone = Rc::clone(&resolution_map);
    
    quality_combo.connect_changed(move |combo| {
        // Clear the FPS combo
        fps_combo_clone.remove_all();
        format_combo_clone.remove_all();
        
        if let Some(idx_str) = combo.active_id() {
            if let Some(idx) = idx_str.parse::<usize>().ok() {
                let map = resolution_map_clone.borrow();
                if idx < map.len() {
                    // Now populate the FPS dropdown with available FPS values
                    let fps_list = &map[idx].1;
                    fps_combo_clone.remove_all();
                    
                    // Extract FPS values
                    let mut fps_values = Vec::new();
                    for (fps, _, _) in fps_list {
                        fps_values.push(*fps);
                    }
                    
                    // Remove duplicates by converting to a set and back
                    fps_values.sort_unstable();
                    fps_values.dedup();
                    
                    // Add each FPS value to the combo box
                    if fps_values.is_empty() || (fps_values.len() == 1 && fps_values[0] == 0) {
                        // If there are no FPS values or only zero FPS, show a default option
                        fps_combo_clone.append(Some("0"), "Default");
                    } else {
                        for fps in fps_values {
                            if fps > 0 {  // Skip zero FPS as it's handled above
                                fps_combo_clone.append(Some(&fps.to_string()), &format!("{} fps", fps));
                            }
                        }
                    }
                    
                    // Make the FPS dropdown active
                        fps_combo_clone.set_active(Some(0));
                        fps_combo_clone.set_sensitive(true);
                    
                    // Update format info text
                    let resolution = &map[idx].0;
                        format_info_clone.set_text(&format!("Selected quality: {}", resolution));
                }
            }
        }
    });
    
    // Connect FPS combo box to update format options
    let format_combo_clone = format_combo.clone();
    let download_button_clone = download_button.clone();
    let format_info_clone = format_info.clone();
    let resolution_map_clone = Rc::clone(&resolution_map);
    let quality_combo_clone = quality_combo.clone();
    
    fps_combo.connect_changed(move |combo| {
        // Clear the format type combo
        format_combo_clone.remove_all();
        
        if let Some(quality_idx_str) = quality_combo_clone.active_id() {
            if let Some(quality_idx) = quality_idx_str.parse::<usize>().ok() {
                if let Some(fps_text) = combo.active_text() {
                    // Extract the numeric part from "XX fps"
                    let selected_fps = fps_text.split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(0);
                    
                    println!("Selected FPS: {}", selected_fps);
                    
                    let map = resolution_map_clone.borrow();
                    if quality_idx < map.len() {
                        let resolution = &map[quality_idx].0;
                        let fps_values = &map[quality_idx].1;
                        
                        // Find all formats that match the selected resolution and FPS
                        let mut available_formats = Vec::new();
                        
                        // First, collect all formats for this FPS
                        for (fps, format_id, ext) in fps_values.iter() {
                            if *fps == selected_fps {
                                println!("Matching format found: fps={}, format_id={}, ext={}", fps, format_id, ext);
                                available_formats.push((format_id.clone(), ext.clone()));
                            }
                        }
                        
                        println!("Total matching formats: {}", available_formats.len());
                        
                        // Extract all unique extensions
                        let mut unique_extensions = std::collections::HashSet::new();
                        for (_, ext) in &available_formats {
                            unique_extensions.insert(ext.clone());
                        }
                        
                        println!("Unique extensions: {:?}", unique_extensions);
                        
                        // Add each unique format type to the combo box
                        let mut added_formats = Vec::new();
                        let mut format_index = 0;
                        
                        // Convert HashSet to Vec for sorting
                        let mut ext_vec: Vec<String> = unique_extensions.into_iter().collect();
                        ext_vec.sort(); // Sort alphabetically
                        
                        // Make sure we're actually adding all formats to the dropdown
                        println!("Adding {} format options to dropdown", ext_vec.len());
                        
                        for ext in ext_vec {
                            println!("Adding format option: {}", ext);
                            // Make sure we're appending to the format_combo_clone
                            format_combo_clone.append(Some(&format_index.to_string()), &ext.to_uppercase());
                            added_formats.push(ext);
                            format_index += 1;
                        }
                        
                        println!("Added formats: {:?}", added_formats);

                        // Set the first format type as active if available
                        if !added_formats.is_empty() {
                            format_combo_clone.set_active(Some(0));
                            format_combo_clone.set_sensitive(true);
                            download_button_clone.set_sensitive(true);
                            format_info_clone.set_text(&format!("Selected: {} at {} fps", resolution, selected_fps));
                        } else {
                            format_combo_clone.set_sensitive(false);
                            download_button_clone.set_sensitive(false);
                            format_info_clone.set_text("No format options available");
                        }
                    }
                }
            }
        }
    });

    // Connect audio quality combo box to update audio format options
    let audio_format_combo_clone = audio_format_combo.clone();
    let download_button_clone = download_button.clone();
    let format_info_clone = format_info.clone();
    let audio_bitrate_map_clone = Rc::clone(&audio_bitrate_map);
    
    audio_quality_combo.connect_changed(move |combo| {
        // Clear the audio format combo
        audio_format_combo_clone.remove_all();
        
        if let Some(idx_str) = combo.active_id() {
            if let Some(idx) = idx_str.parse::<usize>().ok() {
                let audio_map = audio_bitrate_map_clone.borrow();
                if idx < audio_map.len() {
                    // Get selected bitrate
                    let bitrate = audio_map[idx].0;
                    
                    // Get format values for this bitrate
                    let format_values = &audio_map[idx].1;
                    
                    // Extract all unique extensions
                    let mut unique_extensions = std::collections::HashSet::new();
                    for (_, ext) in format_values {
                        unique_extensions.insert(ext.clone());
                    }
                    
                    // Convert HashSet to Vec for sorting
                    let mut ext_vec: Vec<String> = unique_extensions.into_iter().collect();
                    ext_vec.sort(); // Sort alphabetically
                    
                    println!("Adding {} audio format options for {} kbps", ext_vec.len(), bitrate);
                    
                    // Add each format to the combo box
                    for (i, ext) in ext_vec.iter().enumerate() {
                        audio_format_combo_clone.append(Some(&i.to_string()), &ext.to_uppercase());
                    }
                    
                    // Set the first format as active if available
                    if !ext_vec.is_empty() {
                        audio_format_combo_clone.set_active(Some(0));
                        audio_format_combo_clone.set_sensitive(true);
                        download_button_clone.set_sensitive(true);
                        format_info_clone.set_text(&format!("Selected audio: {} kbps", bitrate));
                    } else {
                        audio_format_combo_clone.set_sensitive(false);
                        format_info_clone.set_text("No audio format options available");
                    }
                }
            }
        }
    });

    // Connect the download button to the download function
    let url_entry_clone = url_entry.clone();
    let output_entry_clone = path_entry_for_download.clone();
    let quality_combo_clone = quality_combo.clone();
    let fps_combo_clone = fps_combo.clone();
    let format_combo_clone = format_combo.clone();
    let audio_quality_combo_clone = audio_quality_combo.clone();
    let audio_format_combo_clone = audio_format_combo.clone();
    let download_status_clone = download_status.clone();
    let progress_bar_clone = progress_bar.clone();
    let open_folder_button_clone = open_folder_button.clone();
    let resolution_map_clone = Rc::clone(&resolution_map);
    let audio_bitrate_map_clone = Rc::clone(&audio_bitrate_map);
    let media_stack_for_download = media_stack.clone();
    let window_clone = window.clone();
    
    download_button.connect_clicked(move |button| {
        let url = url_entry_clone.text().to_string();
        let output_path = output_entry_clone.text().to_string();
        
        if url.is_empty() {
            download_status_clone.set_text("Please enter a URL");
            components::show_error_dialog(&window_clone, "Invalid URL", "Please enter a URL");
            return;
        }

        if output_path.is_empty() {
            download_status_clone.set_text("Please specify a download folder");
            components::show_error_dialog(&window_clone, "Missing Download Location", "Please specify a download folder");
            return;
        }

        // Reset and show progress bar, hide open folder button
        progress_bar_clone.set_fraction(0.0);
        progress_bar_clone.set_text(Some("0%"));
        progress_bar_clone.set_visible(true);
        open_folder_button_clone.set_visible(false);
        
        // Disable the download button during download
        button.set_sensitive(false);

        // Get the active tab
        let active_tab = media_stack_for_download.visible_child_name().unwrap_or_else(|| "video".into());
        
        if active_tab == "video" {
            // Download video
            // Get selected quality index
            if let Some(quality_idx_str) = quality_combo_clone.active_id() {
                if let Some(quality_idx) = quality_idx_str.parse::<usize>().ok() {
                    // Get FPS from the active text
                    if let Some(fps_text) = fps_combo_clone.active_text() {
                        // Extract the numeric part from "XX fps"
                        let selected_fps = fps_text.split_whitespace()
                            .next()
                            .and_then(|s| s.parse::<u32>().ok())
                            .unwrap_or(0);
                        
                        // Get format type
                        if let Some(selected_format) = format_combo_clone.active_text() {
                            let map = resolution_map_clone.borrow();
                            if quality_idx < map.len() {
                                let resolution = &map[quality_idx].0;
                                let fps_values = &map[quality_idx].1;
                                
                                // Convert to lowercase for case-insensitive comparison
                                let selected_format_lower = selected_format.to_string().to_lowercase();
                                
                                println!("Selected for download: resolution={}, fps={}, format={}",
                                    resolution, selected_fps, selected_format_lower);
                                
                                // Find the format_id that matches both FPS and format type
                                let matching_formats: Vec<_> = fps_values.iter()
                                    .filter(|(fps, _, ext)| {
                                        // If the selected FPS is 0, we match all formats with the same extension
                                        // Otherwise, we match formats with the exact FPS
                                        (*fps == selected_fps || selected_fps == 0) && 
                                        ext.to_lowercase() == selected_format_lower
                                    })
                                    .collect();
                                
                                println!("Found {} matching formats for download", matching_formats.len());
                                
                                if !matching_formats.is_empty() {
                                    let format_id = matching_formats[0].1.clone();
                                    let resolution_str = resolution.clone();
                                    
                                    download_status_clone.set_text(
                                        &format!("Downloading {} @ {} fps ({})", 
                                            resolution, selected_fps, selected_format.to_string().to_uppercase())
                                    );
                                    
                                    // Create a clone of the output path for the worker thread
                                    let output_path_clone = output_path.clone();
                                    let url_clone = url.clone();
                                    let download_status_clone2 = download_status_clone.clone();
                                    let progress_bar_clone2 = progress_bar_clone.clone();
                                    let open_folder_button_clone2 = open_folder_button_clone.clone();
                                    let button_clone = button.clone();
                                    let window_clone2 = window_clone.clone();
                                    
                                    // Create a channel for communication between threads
                                    let (sender, receiver) = glib::MainContext::channel::<String>(glib::PRIORITY_DEFAULT);
                                    
                                    // Handle progress updates in the UI thread
                                    let last_progress = Rc::new(RefCell::new(-1.0)); // Track last progress to avoid duplicates
                                    
                                    receiver.attach(None, move |msg: String| {
                                        if msg == "start" {
                                            // Reset the progress bar state
                                            progress_bar_clone2.set_fraction(0.0);
                                            progress_bar_clone2.set_text(Some("0.0%"));
                                            download_status_clone2.set_text("Download started");
                                            *last_progress.borrow_mut() = 0.0;
                                        } else if msg.starts_with("progress:") {
                                            // New format: "progress:0.123:Status message text"
                                            let parts: Vec<&str> = msg.splitn(3, ':').collect();
                                            
                                            if parts.len() >= 2 {
                                                // Parse the progress percentage
                                                let progress = parts[1].parse::<f64>().unwrap_or(0.0);
                                                
                                                // Only update if progress has changed significantly (at least 0.5%)
                                                let progress_diff = (progress - *last_progress.borrow()) * 100.0;
                                                if progress_diff >= 0.5 || progress >= 0.99 {
                                                    println!("Updating progress bar: {:.1}%", progress * 100.0);
                                                    
                                                    // Update progress bar
                                                    progress_bar_clone2.set_fraction(progress);
                                                    progress_bar_clone2.set_text(Some(&format!("{:.1}%", progress * 100.0)));
                                                    
                                                    // Update status with the provided message if we have one
                                                    if parts.len() >= 3 {
                                                        download_status_clone2.set_text(parts[2]);
                                                    }
                                                    
                                                    // Update last progress
                                                    *last_progress.borrow_mut() = progress;
                                                }
                                            }
                                        } else if msg.starts_with("complete:success") {
                                            // Ensure the progress bar shows 100%
                                            progress_bar_clone2.set_fraction(1.0);
                                            progress_bar_clone2.set_text(Some("100.0%"));
                                            download_status_clone2.set_text("Download complete!");
                                            
                                            // Show Open Folder button
                                            open_folder_button_clone2.set_visible(true);
                                            
                                            // Re-enable download button
                                            button_clone.set_sensitive(true);
                                        } else if msg.starts_with("complete:error:") {
                                            // Download failed
                                            let error_msg = msg.strip_prefix("complete:error:")
                                                .unwrap_or("Unknown error");
                                            
                                            download_status_clone2.set_text(&format!("Download failed: {}", error_msg));
                                            progress_bar_clone2.set_visible(false);
                                            
                                            // Re-enable download button
                                            button_clone.set_sensitive(true);
                                            
                                            components::show_error_dialog(&window_clone2, "Download Error", 
                                                &format!("Download failed: {}", error_msg));
                                        } else if msg.starts_with("error:") {
                                            // Error starting download
                                            let error_msg = msg.strip_prefix("error:")
                                                .unwrap_or("Unknown error");
                                                
                                            download_status_clone2.set_text(&format!("Error starting download: {}", error_msg));
                                            progress_bar_clone2.set_visible(false);
                                            
                                            // Re-enable download button
                                            button_clone.set_sensitive(true);
                                            
                                            components::show_error_dialog(&window_clone2, "Download Error", 
                                                &format!("Error starting download: {}", error_msg));
                                        }
                                        
                                        glib::Continue(true)
                                    });
                                    
                                    // Spawn a thread to monitor the download progress
                                    let sender_clone = sender.clone();
                                    thread::spawn(move || {
                                        // Start the download process
                                        match crate::downloader::download_media_with_format(&url_clone, &output_path_clone, &format_id) {
                                            Ok(mut child) => {
                                                // Send initial start message
                                                sender_clone.send("start".to_string()).unwrap();
                                                
                                                // Get stdout and stderr from the child process
                                                let stdout = child.stdout.take().expect("Failed to capture stdout");
                                                let stderr = child.stderr.take().expect("Failed to capture stderr");
                                                
                                                // Create a buffer reader for stdout and stderr
                                                let stdout_reader = std::io::BufReader::new(stdout);
                                                let stderr_reader = std::io::BufReader::new(stderr);
                                                
                                                // Create a thread to read stdout and track progress
                                                let sender_stdout = sender_clone.clone();
                                                let stdout_thread = thread::spawn(move || {
                                                    let mut reader = stdout_reader.lines();
                                                    let mut progress_state = crate::downloader::ProgressState::default();
                                                    
                                                    while let Some(line_result) = reader.next() {
                                                        match line_result {
                                                            Ok(line) => {
                                                                // Only log important lines
                                                                if line.contains("%") || line.contains("download") || 
                                                                   line.contains("ffmpeg") || line.contains("Merger") {
                                                                    println!("STDOUT: {}", line);
                                                                }
                                                                
                                                                // Update progress state based on this line
                                                                if crate::downloader::update_progress_state(&line, &mut progress_state) {
                                                                    // Send progress update with percentage and message
                                                                    sender_stdout.send(format!(
                                                                        "progress:{:.3}:{}", 
                                                                        progress_state.overall_percent,
                                                                        progress_state.status_message
                                                                    )).unwrap();
                                                                }
                                                            },
                                                            Err(e) => {
                                                                println!("Error reading stdout: {}", e);
                                                                break;
                                                            }
                                                        }
                                                    }
                                                });
                                                
                                                // Create a thread to read stderr and track progress
                                                let sender_stderr = sender_clone.clone();
                                                let stderr_thread = thread::spawn(move || {
                                                    let mut reader = stderr_reader.lines();
                                                    let mut progress_state = crate::downloader::ProgressState::default();
                                                    
                                                    while let Some(line_result) = reader.next() {
                                                        match line_result {
                                                            Ok(line) => {
                                                                // Only log important lines
                                                                if line.contains("%") || line.contains("download") || 
                                                                   line.contains("ffmpeg") || line.contains("Merger") {
                                                                    println!("STDERR: {}", line);
                                                                }
                                                                
                                                                // Update progress state based on this line
                                                                if crate::downloader::update_progress_state(&line, &mut progress_state) {
                                                                    // Send progress update with percentage and message
                                                                    sender_stderr.send(format!(
                                                                        "progress:{:.3}:{}", 
                                                                        progress_state.overall_percent,
                                                                        progress_state.status_message
                                                                    )).unwrap();
                                                                }
                                                            },
                                                            Err(e) => {
                                                                println!("Error reading stderr: {}", e);
                                                                break;
                                                            }
                                                        }
                                                    }
                                                });
                                                
                                                // Wait for the child process to complete
                                                match child.wait() {
                                                    Ok(status) => {
                                                        if status.success() {
                                                            // Ensure progress is 100% when truly complete
                                                            sender_clone.send("progress:1.0:Download complete!".to_string()).unwrap();
                                                            // Slight delay to let UI update before sending completion message
                                                            thread::sleep(std::time::Duration::from_millis(200));
                                                            sender_clone.send("complete:success".to_string()).unwrap();
                                                        } else {
                                                            sender_clone.send(format!("complete:error:{}", status)).unwrap();
                                                        }
                                                    },
                                                    Err(e) => {
                                                        sender_clone.send(format!("complete:error:{}", e)).unwrap();
                                                    }
                                                }
                                                
                                                // Wait for stdout and stderr threads to complete
                                                let _ = stdout_thread.join();
                                                let _ = stderr_thread.join();
                                            },
                                            Err(e) => {
                                                sender_clone.send(format!("error:{}", e)).unwrap();
                                            }
                                        }
                                    });
                                } else {
                                    download_status_clone.set_text("No matching format found");
                                    progress_bar_clone.set_visible(false);
                                    
                                    // Re-enable download button
                                    button.set_sensitive(true);
                                    
                                    components::show_error_dialog(&window_clone, "Format Error", 
                                        "No matching format found for the selected quality, FPS and format type.");
                                }
                            }
                        }
                    }
                }
            }
        } else if active_tab == "audio" {
            // Similar approach for audio downloads
            // Get selected audio quality index
            if let Some(quality_idx_str) = audio_quality_combo_clone.active_id() {
                if let Some(quality_idx) = quality_idx_str.parse::<usize>().ok() {
                    // Get format type
                    if let Some(selected_format) = audio_format_combo_clone.active_text() {
                        let audio_map = audio_bitrate_map_clone.borrow();
                        if quality_idx < audio_map.len() {
                            let bitrate = audio_map[quality_idx].0;
                            let format_values = &audio_map[quality_idx].1;
                            
                            // Convert to lowercase for case-insensitive comparison
                            let selected_format_lower = selected_format.to_string().to_lowercase();
                            
                            println!("Selected for audio download: bitrate={}kbps, format={}",
                                bitrate, selected_format_lower);
                            
                            // Find the format_id that matches the format type
                            let matching_formats: Vec<_> = format_values.iter()
                                .filter(|(_, ext)| ext.to_lowercase() == selected_format_lower)
                                .collect();
                            
                            println!("Found {} matching audio formats for download", matching_formats.len());
                            
                            if !matching_formats.is_empty() {
                                let format_id = matching_formats[0].0.clone();
                                
                                download_status_clone.set_text(
                                    &format!("Downloading audio @ {} kbps ({})", 
                                        bitrate, selected_format.to_string().to_uppercase())
                                );
                                
                                // Use specific audio format for download
                                let target_format = match selected_format_lower.as_str() {
                                    "m4a" => "m4a",
                                    "mp3" => "mp3",
                                    "opus" => "opus",
                                    "webm" => "webm",
                                    _ => "best" // Default to best
                                };
                                
                                // Create a clone of the output path for the worker thread
                                let output_path_clone = output_path.clone();
                                let url_clone = url.clone();
                                let target_format = target_format.to_string();
                                let download_status_clone2 = download_status_clone.clone();
                                let progress_bar_clone2 = progress_bar_clone.clone();
                                let open_folder_button_clone2 = open_folder_button_clone.clone();
                                let button_clone = button.clone();
                                let window_clone2 = window_clone.clone();
                                
                                // Create a channel for communication between threads
                                let (sender, receiver) = glib::MainContext::channel::<String>(glib::PRIORITY_DEFAULT);
                                
                                // Handle progress updates in the UI thread
                                let last_progress = Rc::new(RefCell::new(-1.0)); // Track last progress to avoid duplicates
                                
                                receiver.attach(None, move |msg: String| {
                                    if msg == "start" {
                                        // Reset the progress bar state
                                        progress_bar_clone2.set_fraction(0.0);
                                        progress_bar_clone2.set_text(Some("0.0%"));
                                        download_status_clone2.set_text("Audio download started");
                                        *last_progress.borrow_mut() = 0.0;
                                    } else if msg.starts_with("progress:") {
                                        // New format: "progress:0.123:Status message text"
                                        let parts: Vec<&str> = msg.splitn(3, ':').collect();
                                        
                                        if parts.len() >= 2 {
                                            // Parse the progress percentage
                                            let progress = parts[1].parse::<f64>().unwrap_or(0.0);
                                            
                                            // Only update if progress has changed significantly (at least 0.5%)
                                            let progress_diff = (progress - *last_progress.borrow()) * 100.0;
                                            if progress_diff >= 0.5 || progress >= 0.99 {
                                                println!("Updating progress bar: {:.1}%", progress * 100.0);
                                                
                                                // Update progress bar
                                                progress_bar_clone2.set_fraction(progress);
                                                progress_bar_clone2.set_text(Some(&format!("{:.1}%", progress * 100.0)));
                                                
                                                // Update status with the provided message if we have one
                                                if parts.len() >= 3 {
                                                    download_status_clone2.set_text(parts[2]);
                                                }
                                                
                                                // Update last progress
                                                *last_progress.borrow_mut() = progress;
                                            }
                                        }
                                    } else if msg.starts_with("complete:success") {
                                        // Ensure the progress bar shows 100%
                                        progress_bar_clone2.set_fraction(1.0);
                                        progress_bar_clone2.set_text(Some("100.0%"));
                                        download_status_clone2.set_text("Audio download complete!");
                                        
                                        // Show Open Folder button
                                        open_folder_button_clone2.set_visible(true);
                                        
                                        // Re-enable download button
                                        button_clone.set_sensitive(true);
                                    } else if msg.starts_with("complete:error:") {
                                        // Download failed
                                        let error_msg = msg.strip_prefix("complete:error:")
                                            .unwrap_or("Unknown error");
                                        
                                        download_status_clone2.set_text(&format!("Audio download failed: {}", error_msg));
                                        progress_bar_clone2.set_visible(false);
                                        
                                        // Re-enable download button
                                        button_clone.set_sensitive(true);
                                        
                                        components::show_error_dialog(&window_clone2, "Download Error", 
                                            &format!("Audio download failed: {}", error_msg));
                                    } else if msg.starts_with("error:") {
                                        // Error starting download
                                        let error_msg = msg.strip_prefix("error:")
                                            .unwrap_or("Unknown error");
                                            
                                        download_status_clone2.set_text(&format!("Error starting audio download: {}", error_msg));
                                        progress_bar_clone2.set_visible(false);
                                        
                                        // Re-enable download button
                                        button_clone.set_sensitive(true);
                                        
                                        components::show_error_dialog(&window_clone2, "Download Error", 
                                            &format!("Error starting audio download: {}", error_msg));
                                    }
                                    
                                    glib::Continue(true)
                                });
                                
                                // Spawn a thread to monitor the download progress
                                let sender_clone = sender.clone();
                                thread::spawn(move || {
                                    // Start the download process
                                    match crate::downloader::download_audio_with_format(&url_clone, &output_path_clone, &format_id, &target_format) {
                                        Ok(mut child) => {
                                            // Send initial start message
                                            sender_clone.send("start".to_string()).unwrap();
                                            
                                            // Get stdout and stderr from the child process
                                            let stdout = child.stdout.take().expect("Failed to capture stdout");
                                            let stderr = child.stderr.take().expect("Failed to capture stderr");
                                            
                                            // Create a buffer reader for stdout and stderr
                                            let stdout_reader = std::io::BufReader::new(stdout);
                                            let stderr_reader = std::io::BufReader::new(stderr);
                                            
                                            // Create a thread to read stdout and track progress
                                            let sender_stdout = sender_clone.clone();
                                            let stdout_thread = thread::spawn(move || {
                                                let mut reader = stdout_reader.lines();
                                                let mut progress_state = crate::downloader::ProgressState::default();
                                                
                                                while let Some(line_result) = reader.next() {
                                                    match line_result {
                                                        Ok(line) => {
                                                            // Only log important lines
                                                            if line.contains("%") || line.contains("download") || 
                                                               line.contains("ffmpeg") || line.contains("Extracting") {
                                                                println!("STDOUT: {}", line);
                                                            }
                                                            
                                                            // Update progress state based on this line
                                                            if crate::downloader::update_progress_state(&line, &mut progress_state) {
                                                                // Add "Audio" to status message for audio downloads
                                                                let status = if !progress_state.status_message.contains("Audio") && 
                                                                              !progress_state.status_message.starts_with("Download") {
                                                                    format!("Audio {}", progress_state.status_message.to_lowercase())
                                                                } else {
                                                                    progress_state.status_message.clone()
                                                                };
                                                                
                                                                // Send progress update with percentage and message
                                                                sender_stdout.send(format!(
                                                                    "progress:{:.3}:{}", 
                                                                    progress_state.overall_percent,
                                                                    status
                                                                )).unwrap();
                                                                
                                                                // If complete, send completion message
                                                                if progress_state.phase == crate::downloader::DownloadPhase::Complete {
                                                                    sender_stdout.send("complete:success".to_string()).unwrap();
                                                                }
                                                            }
                                                        },
                                                        Err(e) => {
                                                            println!("Error reading stdout: {}", e);
                                                            break;
                                                        }
                                                    }
                                                }
                                            });
                                            
                                            // Create a thread to read stderr and track progress
                                            let sender_stderr = sender_clone.clone();
                                            let stderr_thread = thread::spawn(move || {
                                                let mut reader = stderr_reader.lines();
                                                let mut progress_state = crate::downloader::ProgressState::default();
                                                
                                                while let Some(line_result) = reader.next() {
                                                    match line_result {
                                                        Ok(line) => {
                                                            // Only log important lines
                                                            if line.contains("%") || line.contains("download") || 
                                                               line.contains("ffmpeg") || line.contains("Extracting") {
                                                                println!("STDERR: {}", line);
                                                            }
                                                            
                                                            // Update progress state based on this line
                                                            if crate::downloader::update_progress_state(&line, &mut progress_state) {
                                                                // Add "Audio" to status message for audio downloads
                                                                let status = if !progress_state.status_message.contains("Audio") && 
                                                                              !progress_state.status_message.starts_with("Download") {
                                                                    format!("Audio {}", progress_state.status_message.to_lowercase())
                                                                } else {
                                                                    progress_state.status_message.clone()
                                                                };
                                                                
                                                                // Send progress update with percentage and message
                                                                sender_stderr.send(format!(
                                                                    "progress:{:.3}:{}", 
                                                                    progress_state.overall_percent,
                                                                    status
                                                                )).unwrap();
                                                                
                                                                // If complete, send completion message
                                                                if progress_state.phase == crate::downloader::DownloadPhase::Complete {
                                                                    sender_stderr.send("complete:success".to_string()).unwrap();
                                                                }
                                                            }
                                                        },
                                                        Err(e) => {
                                                            println!("Error reading stderr: {}", e);
                                                            break;
                                                        }
                                                    }
                                                }
                                            });
                                            
                                            // Wait for the child process to complete
                                            match child.wait() {
                                                Ok(status) => {
                                                    if status.success() {
                                                        // Ensure progress is 100% when truly complete
                                                        sender_clone.send("progress:1.0:Audio download complete!".to_string()).unwrap();
                                                        // Slight delay to let UI update before sending completion message
                                                        thread::sleep(std::time::Duration::from_millis(200));
                                                        sender_clone.send("complete:success".to_string()).unwrap();
                                                    } else {
                                                        sender_clone.send(format!("complete:error:{}", status)).unwrap();
                                                    }
                                                },
                                                Err(e) => {
                                                    sender_clone.send(format!("complete:error:{}", e)).unwrap();
                                                }
                                            }
                                            
                                            // Wait for stdout and stderr threads to complete
                                            let _ = stdout_thread.join();
                                            let _ = stderr_thread.join();
                                        },
                                        Err(e) => {
                                            sender_clone.send(format!("error:{}", e)).unwrap();
                                        }
                                    }
                                });
                            } else {
                                download_status_clone.set_text("No matching audio format found");
                                progress_bar_clone.set_visible(false);
                                
                                // Re-enable download button
                                button.set_sensitive(true);
                                
                                components::show_error_dialog(&window_clone, "Format Error", 
                                    "No matching format found for the selected audio quality and format type.");
                            }
                        }
                    }
                }
            }
        } else {
            download_status_clone.set_text("Please select video or audio options first");
            progress_bar_clone.set_visible(false);
            
            // Re-enable download button
            button.set_sensitive(true);
            
            components::show_error_dialog(&window_clone, "Selection Error", 
                "Please select video or audio options before downloading.");
        }
    });

    // Connect the Open Folder button
    let output_entry_clone = path_entry_for_open.clone();
    let window_clone = window.clone();
    open_folder_button.connect_clicked(move |_| {
        let path = output_entry_clone.text().to_string();
        
        if path.is_empty() {
            components::show_error_dialog(&window_clone, "Path Error", "No download folder specified");
            return;
        }
        
        // Use xdg-open to open the folder (Linux)
        match std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn() {
                Ok(_) => {},
                Err(e) => {
                    components::show_error_dialog(&window_clone, "Open Folder Error", 
                        &format!("Failed to open folder: {}", e));
                }
            };
    });

    // Connect media stack to update the download button state and download path
    let format_info_clone = format_info.clone();
    let path_entry_clone = path_entry_for_tab_change.clone();
    let default_path = get_default_download_path().unwrap_or_else(|_| String::from("."));
    let media_stack_for_tab_change = media_stack.clone();
    
    media_stack_for_tab_change.connect_visible_child_name_notify(move |stack| {
        let active_tab = stack.visible_child_name().unwrap_or_else(|| "video".into());
        
        // Get current path
        let current_path = path_entry_clone.text().to_string();
        
        // Only change the path if it contains "HyprDownloader" (meaning it's using our default path structure)
        // or if it's empty/default
        let should_update_path = current_path.contains("HyprDownloader") || 
                               current_path.is_empty() || 
                               current_path == "." || 
                               current_path == default_path;
        
        if active_tab == "video" {
            format_info_clone.set_text("Video options selected. Choose quality, FPS and format.");
            
            // Set path to video subfolder only if using default path
            if should_update_path {
                let video_path = format!("{}/video", default_path);
                path_entry_clone.set_text(&video_path);
            }
        } else if active_tab == "audio" {
            format_info_clone.set_text("Audio options selected. Choose quality and format.");
            
            // Set path to audio subfolder only if using default path
            if should_update_path {
                let audio_path = format!("{}/audio", default_path);
                path_entry_clone.set_text(&audio_path);
            }
        }
    });

    // Connect the browse button to open a folder chooser dialog
    let output_entry_clone = path_entry_for_browse.clone();
    let window_clone = window.clone();
    browse_button.connect_clicked(move |_| {
        let dialog = crate::ui::components::create_folder_chooser_dialog(&window_clone, "Select Download Location");
        
        let output_entry_clone2 = output_entry_clone.clone();
        dialog.connect_response(move |dialog, response| {
            if response == gtk4::ResponseType::Accept {
                if let Some(folder) = dialog.file().and_then(|file| file.path()) {
                    if let Some(path_str) = folder.to_str() {
                        output_entry_clone2.set_text(path_str);
                    }
                }
            }
            dialog.destroy();
        });
        
        dialog.present();
    });

    // Connect the About button to show information dialog
    let window_clone = window.clone();
    about_button.connect_clicked(move |_| {
        let about_dialog = gtk4::AboutDialog::builder()
            .program_name("HyprDownloader")
            .logo_icon_name("video-display")
            .version("1.0")
            .copyright("© 2025")
            .comments("Download videos and audio from various online sources")
            .website("https://github.com/os-guy/HyprDownloader")
            .website_label("GitHub Repository")
            .license_type(gtk4::License::Gpl30)
            .modal(true)
            .transient_for(&window_clone)
            .build();
            
        about_dialog.present();
    });

    // Connect format combo to show format suggestions
    let video_format_info = format_info.clone();
    format_combo.connect_changed(move |combo| {
        if let Some(selected_format) = combo.active_text() {
            let format_suggestion = match selected_format.to_lowercase().as_str() {
                "mp4" => "MP4 - Good compatibility, balanced quality/size ratio.",
                "webm" => "WebM - Better compression, may not play on all devices.",
                "mov" => "MOV - Good quality but larger file size.",
                "mkv" => "MKV - Best for quality and features but lower compatibility.",
                "avi" => "AVI - Older format, good compatibility but less efficient.",
                _ => "Selected format"
            };
            
            video_format_info.set_text(format_suggestion);
        }
    });
    
    // Connect audio format combo to show format suggestions
    let audio_format_info = format_info.clone();
    audio_format_combo.connect_changed(move |combo| {
        if let Some(selected_format) = combo.active_text() {
            let format_suggestion = match selected_format.to_lowercase().as_str() {
                "mp3" => "MP3 - Widely compatible, good quality at 256kbps+.",
                "m4a" => "M4A - Better quality than MP3 at same bitrate.",
                "opus" => "Opus - Best quality-to-size ratio, newer devices only.",
                "ogg" => "OGG - Free format, good quality but less compatible.",
                "wav" => "WAV - Lossless but very large files.",
                "flac" => "FLAC - Lossless with compression, large files.",
                _ => "Selected audio format"
            };
            
            audio_format_info.set_text(format_suggestion);
        }
    });

    // Add widgets to the container
    main_container.append(&header_container);
    container.append(&main_area);
    main_container.append(&container);

    window.present();
} 