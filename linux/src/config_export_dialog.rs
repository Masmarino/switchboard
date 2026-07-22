use adw::prelude::*;
use gtk::glib;
use switchboard_core::AppView;
use uuid::Uuid;

/// Fenetre de selection avant export : une case a cocher par app (toutes cochees
/// par defaut) + un toggle "inclure les variables d'environnement", puis panneau
/// de sauvegarde natif. Mirrors `add_dialog`'s adw::Window + PreferencesGroup
/// pattern.
pub fn show_export_dialog(
    parent: &impl IsA<gtk::Window>,
    apps: &[AppView],
    on_export: impl Fn(Vec<Uuid>, bool) -> String + 'static,
) {
    let apps_group = adw::PreferencesGroup::builder().title("Apps à exporter").build();
    let mut checks: Vec<(Uuid, gtk::CheckButton)> = Vec::new();
    for app in apps {
        let row = adw::ActionRow::builder().title(app.name.as_str()).build();
        let check = gtk::CheckButton::builder().active(true).valign(gtk::Align::Center).build();
        row.add_prefix(&check);
        row.set_activatable_widget(Some(&check));
        apps_group.add(&row);
        checks.push((app.id, check));
    }

    let options_group = adw::PreferencesGroup::builder().title("Options").build();
    let env_row = adw::ActionRow::builder().title("Inclure les variables d'environnement").build();
    let env_switch = gtk::Switch::builder().active(false).valign(gtk::Align::Center).build();
    env_row.add_suffix(&env_switch);
    options_group.add(&env_row);

    let page = adw::PreferencesPage::new();
    page.add(&apps_group);
    page.add(&options_group);

    let dialog = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("Exporter la config")
        .default_width(420)
        .default_height(480)
        .build();

    let export_btn = gtk::Button::with_label("Exporter…");
    export_btn.add_css_class("suggested-action");
    let cancel_btn = gtk::Button::with_label("Annuler");
    let header = adw::HeaderBar::builder().show_end_title_buttons(false).build();
    header.pack_end(&export_btn);
    header.pack_start(&cancel_btn);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);
    content.append(&page);
    dialog.set_content(Some(&content));

    {
        let dialog = dialog.clone();
        cancel_btn.connect_clicked(move |_| dialog.close());
    }
    {
        let dialog = dialog.clone();
        export_btn.connect_clicked(move |_| {
            let selected_ids: Vec<Uuid> =
                checks.iter().filter(|(_, check)| check.is_active()).map(|(id, _)| *id).collect();
            if selected_ids.is_empty() {
                return;
            }
            let include_env_vars = env_switch.is_active();
            let json = on_export(selected_ids, include_env_vars);

            let file_dialog = gtk::FileDialog::builder()
                .title("Exporter la config")
                .initial_name("switchboard-config.json")
                .build();
            let dialog = dialog.clone();
            glib::spawn_future_local(async move {
                if let Ok(file) = file_dialog.save_future(Some(&dialog)).await {
                    if let Some(path) = file.path() {
                        let _ = std::fs::write(path, json);
                    }
                }
                dialog.close();
            });
        });
    }

    dialog.present();
}
