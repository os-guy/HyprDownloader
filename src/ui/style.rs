use gtk4::{CssProvider, StyleContext};
use gtk4::gdk::Display;

pub fn load_css() {
    // Load CSS
    let provider = CssProvider::new();
    provider.load_from_data(
        "
        .main-container {
            background-color: @theme_bg_color;
            padding: 0;
        }
        
        .floating-window {
            background-color: @theme_bg_color;
            border-radius: 6px;
        }
        
        .content-box {
            background-color: @theme_base_color;
            padding: 0;
        }
        
        .card, .url-card {
            background-color: @theme_base_color;
            border-radius: 10px;
            box-shadow: 0 2px 4px alpha(@theme_fg_color, 0.1);
            padding: 12px;
            margin-bottom: 6px;
        }
        
        .options-box {
            background-color: alpha(@theme_bg_color, 0.4);
            border-radius: 8px;
            padding: 12px;
            margin-top: 4px;
        }
        
        .app-title {
            font-size: 24px;
            font-weight: bold;
            margin: 4px 0 6px 0;
        }
        
        .app-subtitle {
            font-size: 14px;
            margin: 0 0 8px 0;
            color: alpha(@theme_fg_color, 0.7);
        }
        
        .about-button {
            min-height: 28px;
            border-radius: 4px;
            padding: 2px 10px;
            font-size: 12px;
            font-weight: 500;
        }
        
        .section-title {
            font-size: 16px;
            font-weight: bold;
            margin-bottom: 6px;
            margin-top: 0;
        }
        
        .input-label {
            font-weight: 500;
            margin-bottom: 4px;
            font-size: 13px;
        }
        
        .url-entry, .path-entry {
            min-height: 32px;
            border-radius: 5px;
            margin-bottom: 6px;
            padding: 0 6px;
        }
        
        .path-entry {
            min-width: 180px;
        }
        
        .combo-box {
            min-height: 32px;
            border-radius: 5px;
            margin-bottom: 6px;
        }
        
        .fetch-button {
            font-weight: 600;
            min-height: 32px;
            min-width: 70px;
            border-radius: 5px;
            background-color: @theme_selected_bg_color;
            color: @theme_selected_fg_color;
        }
        
        .browse-button {
            font-weight: 500;
            min-height: 34px;
            min-width: 70px;
            border-radius: 5px;
        }
        
        .download-button {
            font-weight: bold;
            min-height: 36px;
            min-width: 100%;
            border-radius: 5px;
            margin-top: 3px;
            margin-bottom: 3px;
            background-color: @success_color;
            color: @theme_selected_fg_color;
        }
        
        .open-folder-button {
            font-weight: 500;
            min-height: 34px;
            min-width: 100%;
            border-radius: 5px;
        }
        
        .download-progress {
            min-height: 6px;
            border-radius: 3px;
            margin: 6px 0;
        }
        
        .status-label {
            font-size: 12px;
            font-style: italic;
            color: alpha(@theme_fg_color, 0.7);
            margin-top: 3px;
            margin-bottom: 3px;
        }
        
        .info-box {
            border-radius: 6px;
            padding: 10px;
            margin: 3px 0;
            background-color: alpha(@theme_bg_color, 0.5);
            border: 1px solid alpha(@theme_fg_color, 0.1);
            font-size: 12px;
        }
        
        .info-title {
            font-weight: bold;
            margin-bottom: 6px;
            font-size: 14px;
        }
        
        /* Video and Audio option boxes */
        .info-box .section-title {
            font-size: 14px;
            margin-bottom: 8px;
            color: alpha(@theme_fg_color, 0.9);
        }
        
        .info-box .combo-box {
            margin-bottom: 4px;
        }

        /* Tab switcher styling */
        stackswitcher button {
            padding: 3px 6px;
            min-height: 22px;
            border-radius: 4px;
            margin: 0 2px;
            font-size: 11px;
            font-weight: 500;
        }
        
        /* Media options stack switcher styling */
        .content-box stackswitcher button {
            padding: 4px 10px;
            min-height: 26px;
            border-radius: 5px;
            margin: 0 2px;
            font-size: 13px;
            font-weight: 500;
        }
        
        stack {
            min-height: 100px;
            border-radius: 4px;
            padding: 0;
            margin: 0;
        }
        "
    );
    
    // Apply CSS to the application
    StyleContext::add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
} 