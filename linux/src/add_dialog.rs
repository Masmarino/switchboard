use adw::prelude::*;
use gtk::glib;
use switchboard_core::{AppDraft, AppKind, AppView};

const KIND_IDS: &[AppKind] = &[
    AppKind::Cargo,
    AppKind::Npm,
    AppKind::Dotnet,
    AppKind::Maven,
    AppKind::Python,
    AppKind::Go,
    AppKind::Raw,
];
const KIND_LABELS: [&str; 7] = ["Cargo", "Npm", "Dotnet", "Maven", "Python", "Go", "Raw"];

fn kind_to_index(kind: Option<AppKind>) -> u32 {
    KIND_IDS.iter().position(|k| Some(*k) == kind).unwrap_or(0) as u32
}

/// Construit puis affiche la fenetre d'ajout/edition d'une app. `existing` preremplit
/// les champs en mode edition ; `on_save` recoit le draft valide au clic sur "Enregistrer".
pub fn show_app_dialog(
    parent: &impl IsA<gtk::Window>,
    existing: Option<&AppView>,
    on_save: impl Fn(AppDraft) + 'static,
) {
    let title = if existing.is_some() { "Modifier l'app" } else { "Ajouter une app" };

    let name_entry = gtk::Entry::builder().text(existing.map(|a| a.name.as_str()).unwrap_or("")).build();
    let dir_entry = gtk::Entry::builder()
        .text(existing.map(|a| a.working_dir.as_str()).unwrap_or(""))
        .hexpand(true)
        .build();
    let browse_dir_btn = gtk::Button::with_label("Parcourir…");
    let dir_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    dir_box.append(&dir_entry);
    dir_box.append(&browse_dir_btn);
    let url_entry = gtk::Entry::builder()
        .text(existing.and_then(|a| a.url.as_deref()).unwrap_or(""))
        .placeholder_text("http://localhost:3000 (optionnel)")
        .build();
    let command_entry = gtk::Entry::builder().text(existing.map(|a| a.command.as_str()).unwrap_or("")).build();

    let kind_dropdown = gtk::DropDown::builder()
        .model(&gtk::StringList::new(&KIND_LABELS))
        .build();
    kind_dropdown.set_selected(kind_to_index(existing.map(|a| a.kind)));

    let auto_restart_switch = gtk::Switch::builder()
        .active(existing.map(|a| a.auto_restart).unwrap_or(false))
        .valign(gtk::Align::Center)
        .build();

    let start_order_spin = gtk::SpinButton::with_range(0.0, 99.0, 1.0);
    start_order_spin.set_value(existing.map(|a| a.start_order).unwrap_or(0) as f64);

    let env_vars_text = existing
        .map(|a| a.env_vars.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default();
    let env_buffer = gtk::TextBuffer::builder().text(&env_vars_text).build();
    let env_view = gtk::TextView::builder().buffer(&env_buffer).monospace(true).build();
    let env_scroller = gtk::ScrolledWindow::builder()
        .child(&env_view)
        .height_request(80)
        .css_classes(vec!["card".to_string()])
        .build();

    let grid = gtk::Grid::builder().row_spacing(8).column_spacing(12).margin_top(12).margin_bottom(12).margin_start(12).margin_end(12).build();
    let mut row = 0;
    for (label, widget, expand) in [
        ("Nom", name_entry.clone().upcast::<gtk::Widget>(), true),
        ("Dossier", dir_box.clone().upcast::<gtk::Widget>(), true),
        ("Type", kind_dropdown.clone().upcast::<gtk::Widget>(), true),
        ("Commande", command_entry.clone().upcast::<gtk::Widget>(), true),
        ("URL", url_entry.clone().upcast::<gtk::Widget>(), true),
        ("Auto-restart", auto_restart_switch.clone().upcast::<gtk::Widget>(), false),
        ("Ordre de démarrage", start_order_spin.clone().upcast::<gtk::Widget>(), true),
    ] {
        let lbl = gtk::Label::builder().label(label).halign(gtk::Align::Start).build();
        grid.attach(&lbl, 0, row, 1, 1);
        if expand {
            widget.set_hexpand(true);
        } else {
            widget.set_halign(gtk::Align::Start);
        }
        grid.attach(&widget, 1, row, 1, 1);
        row += 1;
    }
    let env_label = gtk::Label::builder().label("Variables d'env\n(KEY=VALUE)").halign(gtk::Align::Start).build();
    grid.attach(&env_label, 0, row, 1, 1);
    grid.attach(&env_scroller, 1, row, 1, 1);

    let dialog = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .default_width(420)
        .build();

    let save_btn = gtk::Button::with_label(if existing.is_some() { "Enregistrer" } else { "Ajouter" });
    save_btn.add_css_class("suggested-action");
    let cancel_btn = gtk::Button::with_label("Annuler");

    let header = adw::HeaderBar::builder().show_end_title_buttons(false).build();
    header.pack_end(&save_btn);
    header.pack_start(&cancel_btn);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);
    content.append(&grid);
    dialog.set_content(Some(&content));

    {
        let dialog = dialog.clone();
        let dir_entry = dir_entry.clone();
        browse_dir_btn.connect_clicked(move |_| {
            let file_dialog = gtk::FileDialog::builder().title("Choisir le dossier").build();
            let dir_entry = dir_entry.clone();
            let dialog = dialog.clone();
            glib::spawn_future_local(async move {
                if let Ok(folder) = file_dialog.select_folder_future(Some(&dialog)).await {
                    if let Some(path) = folder.path() {
                        dir_entry.set_text(&path.to_string_lossy());
                    }
                }
            });
        });
    }
    {
        let dialog = dialog.clone();
        cancel_btn.connect_clicked(move |_| dialog.close());
    }
    {
        let dialog = dialog.clone();
        save_btn.connect_clicked(move |_| {
            let env_vars = env_buffer
                .text(&env_buffer.start_iter(), &env_buffer.end_iter(), false)
                .lines()
                .filter_map(|line| line.split_once('='))
                .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                .filter(|(k, _)| !k.is_empty())
                .collect::<Vec<_>>();

            let url = url_entry.text().trim().to_string();
            let draft = AppDraft {
                name: name_entry.text().trim().to_string(),
                working_dir: dir_entry.text().trim().into(),
                kind: KIND_IDS
                    .get(kind_dropdown.selected() as usize)
                    .copied()
                    .unwrap_or(AppKind::Cargo),
                command: command_entry.text().trim().to_string(),
                url: if url.is_empty() { None } else { Some(url) },
                env_vars,
                auto_restart: auto_restart_switch.is_active(),
                start_order: start_order_spin.value() as i32,
            };
            on_save(draft);
            dialog.close();
        });
    }

    dialog.present();
}
