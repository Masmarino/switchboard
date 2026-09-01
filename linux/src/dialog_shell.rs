use adw::prelude::*;

/// Window + header (primary/cancel buttons) + content page, shared by add_dialog and
/// config_export_dialog. Cancel already closes the dialog; caller wires the primary button.
pub(crate) fn dialog_shell(
    parent: &impl IsA<gtk::Window>,
    title: &str,
    title_widget: Option<&adw::WindowTitle>,
    primary_label: &str,
    default_size: (i32, i32),
    page: &adw::PreferencesPage,
) -> (adw::Window, gtk::Button) {
    let dialog = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .default_width(default_size.0)
        .default_height(default_size.1)
        .build();

    let primary_btn = gtk::Button::with_label(primary_label);
    primary_btn.add_css_class("suggested-action");
    let cancel_btn = gtk::Button::with_label("Annuler");

    let header_builder = adw::HeaderBar::builder().show_end_title_buttons(false);
    let header_builder = match title_widget {
        Some(title_widget) => header_builder.title_widget(title_widget),
        None => header_builder,
    };
    let header = header_builder.build();
    header.pack_end(&primary_btn);
    header.pack_start(&cancel_btn);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);
    content.append(page);
    dialog.set_content(Some(&content));

    {
        let dialog = dialog.clone();
        cancel_btn.connect_clicked(move |_| dialog.close());
    }

    (dialog, primary_btn)
}
