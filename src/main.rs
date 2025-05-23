mod ui;
mod app;
mod downloader;

use anyhow::Result;
use gtk4::{prelude::*, Application};

const APP_ID: &str = "com.github.mediadownloader";

fn main() -> Result<()> {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(app::build_ui);
    app.run();

    Ok(())
}
