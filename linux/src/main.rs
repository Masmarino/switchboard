mod add_dialog;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use adw::prelude::*;
use switchboard_core::{AppKind, AppView, Engine};
use gtk::{gdk, gio, glib};
use uuid::Uuid;

const APP_ID: &str = "com.skolln.switchboard";

fn status_css_class(view: &AppView) -> &'static str {
    match view.status_label {
        "running" => "status-running",
        "building" => "status-building",
        "failed" => "status-failed",
        _ => "status-stopped",
    }
}

fn load_styles() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
        .status-dot { min-width: 10px; min-height: 10px; border-radius: 6px; margin: 0 4px; }
        .status-stopped { background-color: #8e8e93; }
        .status-building { background-color: #ff9f0a; }
        .status-running { background-color: #30d158; box-shadow: 0 0 6px 2px #30d15880; }
        .status-failed { background-color: #ff453a; }
        .terminal { background-color: #1c1c1e; color: #d6d6d8; font-family: monospace; padding: 10px; }
        ",
    );
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("no display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

struct Ui {
    app: adw::Application,
    engine: Rc<RefCell<Engine>>,
    list_box: gtk::ListBox,
    log_view: gtk::TextView,
    search_entry: gtk::SearchEntry,
    header_label: adw::WindowTitle,
    selected: Rc<RefCell<Option<Uuid>>>,
    /// Dernier statut connu par app, pour detecter les transitions vers "failed"
    /// et declencher une notification — pas pour l'affichage (qui relit `list_apps`).
    last_status: Rc<RefCell<HashMap<Uuid, &'static str>>>,
}

impl Ui {
    fn refresh(&self) {
        let apps = self.engine.borrow_mut().list_apps();
        self.notify_new_failures(&apps);

        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let selected = *self.selected.borrow();
        let mut selected_view: Option<AppView> = None;

        for view in &apps {
            if Some(view.id) == selected {
                selected_view = Some(view.clone());
            }
            let row = self.build_row(view);
            self.list_box.append(&row);
        }

        if selected_view.is_none() {
            if let Some(first) = apps.first() {
                *self.selected.borrow_mut() = Some(first.id);
                selected_view = Some(first.clone());
            }
        }

        if let Some(view) = selected_view {
            self.header_label.set_subtitle(&view.name);
            self.render_logs(&view);
        } else {
            self.header_label.set_subtitle("Aucune app configurée");
            self.log_view.buffer().set_text("");
        }
    }

    fn render_logs(&self, view: &AppView) {
        let filter = self.search_entry.text().to_string().to_lowercase();
        let buffer = self.log_view.buffer();
        if view.logs.is_empty() {
            buffer.set_text("Pas encore de logs. Démarre l'app pour voir sa sortie ici.");
            return;
        }
        let text = if filter.is_empty() {
            view.logs.join("\n")
        } else {
            view.logs.iter().filter(|l| l.to_lowercase().contains(&filter)).cloned().collect::<Vec<_>>().join("\n")
        };
        buffer.set_text(&text);
        let mut end = buffer.end_iter();
        self.log_view.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
    }

    fn notify_new_failures(&self, apps: &[AppView]) {
        let mut last = self.last_status.borrow_mut();
        for app in apps {
            let previous = last.insert(app.id, app.status_label);
            if app.status_label == "failed" && previous != Some("failed") {
                let notification = gio::Notification::new(&format!("{} a crashé", app.name));
                if let Some(err) = &app.error {
                    notification.set_body(Some(err));
                }
                notification.set_priority(gio::NotificationPriority::High);
                self.app.send_notification(Some(&format!("crash-{}", app.id)), &notification);
            }
        }
    }

    fn build_row(&self, view: &AppView) -> gtk::Widget {
        let subtitle = match (&view.error, view.active, view.healthy) {
            (Some(err), _, _) => err.clone(),
            (None, true, Some(true)) => format!(
                "{} · ✓ healthy · {:.0}% CPU · {:.0} Mo",
                view.status_label, view.cpu_percent, view.memory_mb
            ),
            (None, true, Some(false)) => format!(
                "{} · ✗ ne répond pas · {:.0}% CPU · {:.0} Mo",
                view.status_label, view.cpu_percent, view.memory_mb
            ),
            (None, true, None) => format!(
                "{} · {:.0}% CPU · {:.0} Mo",
                view.status_label, view.cpu_percent, view.memory_mb
            ),
            (None, false, _) => view.status_label.to_string(),
        };

        let row = adw::ActionRow::builder()
            .title(&view.name)
            .subtitle(&subtitle)
            .activatable(true)
            .build();

        let dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        dot.add_css_class("status-dot");
        dot.add_css_class(status_css_class(view));
        dot.set_valign(gtk::Align::Center);
        dot.set_halign(gtk::Align::Center);
        dot.set_vexpand(false);
        dot.set_size_request(10, 10);
        row.add_prefix(&dot);

        let kind_label = gtk::Label::new(Some(match view.kind {
            AppKind::Cargo => "CARGO",
            AppKind::Npm => "NPM",
            AppKind::Dotnet => "DOTNET",
            AppKind::Maven => "MAVEN",
            AppKind::Python => "PYTHON",
            AppKind::Go => "GO",
            AppKind::Raw => "RAW",
        }));
        kind_label.add_css_class("dim-label");
        kind_label.add_css_class("caption");
        row.add_suffix(&kind_label);

        if let Some(url) = view.url.clone() {
            let open_btn = gtk::Button::from_icon_name("web-browser-symbolic");
            open_btn.set_valign(gtk::Align::Center);
            open_btn.add_css_class("flat");
            open_btn.set_tooltip_text(Some("Ouvrir dans le navigateur"));
            open_btn.connect_clicked(move |_| {
                let _ = gio::AppInfo::launch_default_for_uri(&url, None::<&gio::AppLaunchContext>);
            });
            row.add_suffix(&open_btn);
        }

        let edit_btn = gtk::Button::from_icon_name("document-edit-symbolic");
        edit_btn.set_valign(gtk::Align::Center);
        edit_btn.add_css_class("flat");
        let start_btn = gtk::Button::from_icon_name("media-playback-start-symbolic");
        start_btn.set_valign(gtk::Align::Center);
        start_btn.set_sensitive(!view.active);
        start_btn.add_css_class("flat");
        let stop_btn = gtk::Button::from_icon_name("media-playback-stop-symbolic");
        stop_btn.set_valign(gtk::Align::Center);
        stop_btn.set_sensitive(view.active);
        stop_btn.add_css_class("flat");
        let delete_btn = gtk::Button::from_icon_name("user-trash-symbolic");
        delete_btn.set_valign(gtk::Align::Center);
        delete_btn.add_css_class("flat");

        let id = view.id;
        {
            let engine = self.engine.clone();
            let selected = self.selected.clone();
            let this_weak = self.weak_refresh();
            start_btn.connect_clicked(move |_| {
                engine.borrow_mut().start_app(id);
                *selected.borrow_mut() = Some(id);
                this_weak();
            });
        }
        {
            let engine = self.engine.clone();
            let this_weak = self.weak_refresh();
            stop_btn.connect_clicked(move |_| {
                engine.borrow_mut().stop_app(id);
                this_weak();
            });
        }
        {
            let engine = self.engine.clone();
            let this_weak = self.weak_refresh();
            delete_btn.connect_clicked(move |_| {
                engine.borrow_mut().remove_app(id);
                this_weak();
            });
        }
        {
            let engine = self.engine.clone();
            let this_weak = self.weak_refresh();
            let row_for_window = row.clone();
            let view = view.clone();
            edit_btn.connect_clicked(move |_| {
                let Some(window) = row_for_window.root().and_then(|r| r.downcast::<gtk::Window>().ok()) else { return };
                let engine = engine.clone();
                let this_weak = this_weak.clone();
                add_dialog::show_app_dialog(&window, Some(&view), move |draft| {
                    engine.borrow_mut().update_app(id, draft);
                    this_weak();
                });
            });
        }

        row.add_suffix(&edit_btn);
        row.add_suffix(&start_btn);
        row.add_suffix(&stop_btn);
        row.add_suffix(&delete_btn);

        {
            let selected = self.selected.clone();
            let this_weak = self.weak_refresh();
            row.connect_activate(move |_| {
                *selected.borrow_mut() = Some(id);
                this_weak();
            });
        }

        row.upcast()
    }

    /// Petit helper pour rappeler `refresh()` depuis une closure de callback GTK
    /// sans dupliquer la capture de tous les widgets a chaque fois.
    fn weak_refresh(&self) -> impl Fn() + Clone + 'static {
        let app = self.app.clone();
        let engine = self.engine.clone();
        let list_box = self.list_box.clone();
        let log_view = self.log_view.clone();
        let search_entry = self.search_entry.clone();
        let header_label = self.header_label.clone();
        let selected = self.selected.clone();
        let last_status = self.last_status.clone();
        move || {
            let ui = Ui {
                app: app.clone(),
                engine: engine.clone(),
                list_box: list_box.clone(),
                log_view: log_view.clone(),
                search_entry: search_entry.clone(),
                header_label: header_label.clone(),
                selected: selected.clone(),
                last_status: last_status.clone(),
            };
            ui.refresh();
        }
    }
}

fn build_ui(app: &adw::Application) {
    load_styles();

    let engine = Rc::new(RefCell::new(Engine::new()));

    let header_label = adw::WindowTitle::new("Switchboard", "");
    let header = adw::HeaderBar::builder().title_widget(&header_label).build();

    let add_btn = gtk::Button::from_icon_name("list-add-symbolic");
    add_btn.set_tooltip_text(Some("Ajouter une app"));
    header.pack_start(&add_btn);

    let start_all_btn = gtk::Button::from_icon_name("media-playback-start-symbolic");
    start_all_btn.set_tooltip_text(Some("Tout démarrer"));
    let stop_all_btn = gtk::Button::from_icon_name("media-playback-stop-symbolic");
    stop_all_btn.set_tooltip_text(Some("Tout arrêter"));
    header.pack_start(&start_all_btn);
    header.pack_start(&stop_all_btn);

    let clear_logs_btn = gtk::Button::from_icon_name("edit-clear-all-symbolic");
    clear_logs_btn.set_tooltip_text(Some("Effacer les logs"));
    header.pack_end(&clear_logs_btn);

    let export_logs_btn = gtk::Button::from_icon_name("document-save-symbolic");
    export_logs_btn.set_tooltip_text(Some("Exporter les logs…"));
    header.pack_end(&export_logs_btn);

    let about_btn = gtk::Button::from_icon_name("help-about-symbolic");
    about_btn.set_tooltip_text(Some("À propos de Switchboard"));
    header.pack_end(&about_btn);

    let list_box = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(vec!["boxed-list".to_string()])
        .build();

    let sidebar_scroller = gtk::ScrolledWindow::builder().child(&list_box).vexpand(true).build();
    let sidebar_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    sidebar_box.append(&sidebar_scroller);
    sidebar_box.set_width_request(420);

    let search_entry = gtk::SearchEntry::builder().placeholder_text("Filtrer les logs…").build();

    let log_view = gtk::TextView::builder()
        .editable(false)
        .monospace(true)
        .css_classes(vec!["terminal".to_string()])
        .build();
    let log_scroller = gtk::ScrolledWindow::builder().child(&log_view).hexpand(true).vexpand(true).build();

    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content_box.append(&search_entry);
    content_box.append(&log_scroller);

    let split = gtk::Paned::builder().orientation(gtk::Orientation::Horizontal).build();
    split.set_start_child(Some(&sidebar_box));
    split.set_end_child(Some(&content_box));
    split.set_position(420);

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&split));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Switchboard")
        .default_width(1080)
        .default_height(620)
        .content(&toolbar_view)
        .build();

    let ui = Rc::new(Ui {
        app: app.clone(),
        engine: engine.clone(),
        list_box,
        log_view,
        search_entry: search_entry.clone(),
        header_label,
        selected: Rc::new(RefCell::new(None)),
        last_status: Rc::new(RefCell::new(HashMap::new())),
    });

    {
        let ui = ui.clone();
        let window = window.clone();
        add_btn.connect_clicked(move |_| {
            let engine = ui.engine.clone();
            let this_weak = ui.weak_refresh();
            add_dialog::show_app_dialog(&window, None, move |draft| {
                engine.borrow_mut().add_app(draft);
                this_weak();
            });
        });
    }
    {
        let ui = ui.clone();
        start_all_btn.connect_clicked(move |_| {
            ui.engine.borrow_mut().start_all();
            ui.refresh();
        });
    }
    {
        let ui = ui.clone();
        stop_all_btn.connect_clicked(move |_| {
            ui.engine.borrow_mut().stop_all_running();
            ui.refresh();
        });
    }
    {
        let ui = ui.clone();
        clear_logs_btn.connect_clicked(move |_| {
            if let Some(id) = *ui.selected.borrow() {
                ui.engine.borrow_mut().clear_logs(id);
            }
            ui.refresh();
        });
    }
    {
        let ui = ui.clone();
        search_entry.connect_search_changed(move |_| ui.refresh());
    }
    {
        let ui = ui.clone();
        let window = window.clone();
        export_logs_btn.connect_clicked(move |_| {
            let Some(id) = *ui.selected.borrow() else { return };
            let dialog = gtk::FileDialog::builder()
                .title("Exporter les logs")
                .initial_name(format!("{id}.log"))
                .build();
            let engine = ui.engine.clone();
            let window = window.clone();
            glib::spawn_future_local(async move {
                if let Ok(file) = dialog.save_future(Some(&window)).await {
                    if let Some(path) = file.path() {
                        let _ = engine.borrow().export_logs(id, &path);
                    }
                }
            });
        });
    }
    {
        let window = window.clone();
        about_btn.connect_clicked(move |_| {
            let dialog = gtk::AboutDialog::builder()
                .transient_for(&window)
                .modal(true)
                .program_name("Switchboard")
                .version("0.1.0")
                .comments("Démarre, supervise et orchestre tes process de dev locaux — quel que soit le langage.")
                .website("https://github.com/masmarino/switchboard")
                .website_label("Code source (GPLv3)")
                .license_type(gtk::License::Gpl30)
                .authors(vec!["SkollN".to_string()])
                .build();
            dialog.add_credit_section("Développé par", &["SkollN — skolln.com"]);
            dialog.add_credit_section("Découvre aussi", &["Alume — alume.skolln.com"]);
            dialog.present();
        });
    }

    ui.refresh();

    // Poll periodique : les logs/statuts arrivent depuis des threads de process en
    // arriere-plan, on les fait passer dans la boucle GTK via un timeout.
    {
        let ui = ui.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
            ui.refresh();
            glib::ControlFlow::Continue
        });
    }

    window.present();
}

fn main() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).flags(gio::ApplicationFlags::empty()).build();
    app.connect_activate(build_ui);
    app.run()
}
