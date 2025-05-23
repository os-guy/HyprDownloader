use gtk4::{prelude::*, Box as GtkBox, Label, Entry, Button, ComboBoxText, Orientation, Spinner, Image};
use gtk4::{Align, MessageDialog, DialogFlags, MessageType, ButtonsType, Window};

pub fn create_title_with_subtitle(title: &str, subtitle: &str) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .margin_bottom(8)
        .halign(Align::Center)
        .build();

    let title_label = Label::builder()
        .label(title)
        .halign(Align::Center)
        .build();
    
    title_label.add_css_class("app-title");
    
    let subtitle_label = Label::builder()
        .label(subtitle)
        .halign(Align::Center)
        .wrap(true)
        .build();
    
    subtitle_label.add_css_class("app-subtitle");
    
    container.append(&title_label);
    container.append(&subtitle_label);
    
    container
}

pub fn create_section_title(text: &str) -> Label {
    let label = Label::builder()
        .label(text)
        .halign(Align::Start)
        .build();
    
    label.add_css_class("section-title");
    label
}

pub fn create_info_box(title: &str, message: &str) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    
    container.add_css_class("info-box");
    
    let title_label = Label::builder()
        .label(title)
        .halign(Align::Start)
        .build();
    
    title_label.add_css_class("info-title");
    
    let message_label = Label::builder()
        .label(message)
        .halign(Align::Start)
        .wrap(true)
        .xalign(0.0)
        .build();
    
    container.append(&title_label);
    container.append(&message_label);
    
    container
}

pub fn create_status_label(initial_text: &str) -> Label {
    let label = Label::builder()
        .label(initial_text)
        .halign(Align::Start)
        .wrap(true)
        .xalign(0.0)
        .build();
    
    label.add_css_class("status-label");
    label
}

pub fn create_labeled_entry(label_text: &str, placeholder: &str, default_text: Option<&str>) -> (GtkBox, Entry) {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .build();
        
    let label = Label::builder()
        .label(label_text)
        .halign(Align::Start)
        .build();
    
    label.add_css_class("input-label");
    
    let entry = Entry::builder()
        .placeholder_text(placeholder)
        .build();

    if let Some(text) = default_text {
        entry.set_text(text);
    }
    
    if label_text.contains("Download folder") {
        entry.add_css_class("path-entry");
    } else {
        entry.add_css_class("url-entry");
    }
    
    container.append(&label);
    container.append(&entry);
    
    (container, entry)
}

pub fn create_dropdown(label_text: &str) -> (GtkBox, ComboBoxText) {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();
        
    let label = Label::builder()
        .label(label_text)
        .halign(Align::Start)
        .build();
    
    label.add_css_class("input-label");
    
    let combo = ComboBoxText::new();
    combo.set_sensitive(false);
    combo.add_css_class("combo-box");
    
    container.append(&label);
    container.append(&combo);
    
    (container, combo)
}

pub fn create_button(label_text: &str, css_class: &str) -> Button {
    let button = Button::builder()
        .label(label_text)
        .build();
    
    button.add_css_class(css_class);
    button
}

pub fn create_spinner_with_label() -> (GtkBox, Spinner, Label) {
    let container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .margin_top(4)
        .margin_bottom(4)
        .halign(Align::Start)
        .build();

    let spinner = Spinner::new();
    
    let label = Label::builder()
        .label("Processing...")
        .halign(Align::Start)
        .visible(false)
        .build();
    
    label.add_css_class("status-label");
    
    container.append(&spinner);
    container.append(&label);
    
    (container, spinner, label)
}

pub fn show_error_dialog(parent: &impl IsA<Window>, title: &str, message: &str) {
    let dialog = MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .buttons(ButtonsType::Ok)
        .message_type(MessageType::Error)
        .text(title)
        .secondary_text(message)
        .build();
        
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    
    dialog.present();
}

pub fn create_folder_chooser_dialog(parent: &impl IsA<Window>, title: &str) -> gtk4::FileChooserDialog {
    let dialog = gtk4::FileChooserDialog::builder()
        .title(title)
        .transient_for(parent)
        .modal(true)
        .action(gtk4::FileChooserAction::SelectFolder)
        .build();
    
    // Add buttons
    dialog.add_button("Cancel", gtk4::ResponseType::Cancel.into());
    dialog.add_button("Select", gtk4::ResponseType::Accept.into());
    
    dialog
} 