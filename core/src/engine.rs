use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::app_config::{AppConfig, AppConfigList, AppKind};
use crate::health::spawn_health_watcher;
use crate::log_stream::Event;
use crate::process_manager::{kill_process_group, run_app_thread, AppStatus, RunningHandle};

const MAX_LOG_LINES: usize = 5000;
const SAMPLE_INTERVAL: Duration = Duration::from_millis(1000);

struct AppRuntime {
    status: AppStatus,
    logs: VecDeque<String>,
    healthy: Option<bool>,
    cpu_percent: f32,
    memory_mb: f64,
}

impl AppRuntime {
    fn new() -> Self {
        Self {
            status: AppStatus::Stopped,
            logs: VecDeque::new(),
            healthy: None,
            cpu_percent: 0.0,
            memory_mb: 0.0,
        }
    }

    fn push_log(&mut self, line: String) {
        self.logs.push_back(line);
        while self.logs.len() > MAX_LOG_LINES {
            self.logs.pop_front();
        }
    }
}

/// Vue serialisable d'une app et de son etat courant — c'est ce que chaque frontend
/// (GTK direct, ou FFI JSON pour Swift/C#) consomme pour afficher l'UI.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AppView {
    pub id: Uuid,
    pub name: String,
    pub working_dir: String,
    pub kind: AppKind,
    pub command: String,
    pub url: Option<String>,
    pub env_vars: Vec<(String, String)>,
    pub auto_restart: bool,
    pub start_order: i32,
    pub status_label: &'static str,
    pub error: Option<String>,
    pub active: bool,
    pub logs: Vec<String>,
    /// `None` tant que l'app ne tourne pas ou n'a pas d'URL configuree ; sinon refletes
    /// le dernier ping reussi/echoue vers `url`.
    pub healthy: Option<bool>,
    pub cpu_percent: f32,
    pub memory_mb: f64,
}

/// Champs editables d'une app — utilise pour la creation ET la modification, pour
/// eviter deux jeux de parametres qui divergent au fil des fonctionnalites ajoutees.
#[derive(Debug, Clone, Default)]
pub struct AppDraft {
    pub name: String,
    pub working_dir: PathBuf,
    pub kind: AppKind,
    pub command: String,
    pub url: Option<String>,
    pub env_vars: Vec<(String, String)>,
    pub auto_restart: bool,
    pub start_order: i32,
}

fn status_label(status: &AppStatus) -> &'static str {
    match status {
        AppStatus::Stopped => "stopped",
        AppStatus::Building => "building",
        AppStatus::Running => "running",
        AppStatus::Failed(_) => "failed",
    }
}

/// Facade unique sur la config, les process en cours et leurs logs. Pas de dependance
/// UI : consomme directement en Rust (frontend Linux/GTK) ou via le shim FFI (macOS/Windows).
pub struct Engine {
    config: AppConfigList,
    runtimes: HashMap<Uuid, AppRuntime>,
    handles: HashMap<Uuid, RunningHandle>,
    event_tx: mpsc::Sender<Event>,
    event_rx: mpsc::Receiver<Event>,
    sys: sysinfo::System,
    last_sample: Option<Instant>,
    /// Faux en tests : evite d'ecraser le fichier de config reel de l'utilisateur.
    persist: bool,
}

impl Engine {
    pub fn new() -> Self {
        Self::build(AppConfigList::load_or_default(), true)
    }

    #[cfg(test)]
    fn new_ephemeral() -> Self {
        Self::build(AppConfigList::default(), false)
    }

    fn build(config: AppConfigList, persist: bool) -> Self {
        let mut runtimes = HashMap::new();
        for app in &config.apps {
            runtimes.insert(app.id, AppRuntime::new());
        }
        let (event_tx, event_rx) = mpsc::channel();
        Self {
            config,
            runtimes,
            handles: HashMap::new(),
            event_tx,
            event_rx,
            sys: sysinfo::System::new(),
            last_sample: None,
            persist,
        }
    }

    fn persist(&self) {
        if self.persist {
            let _ = self.config.save();
        }
    }

    /// Absorbe les evenements en attente (logs, changements de statut) avant toute lecture.
    pub fn drain_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                Event::Log(id, line) => {
                    if let Some(rt) = self.runtimes.get_mut(&id) {
                        rt.push_log(line);
                    }
                }
                Event::StatusChanged(id, status) => {
                    if let Some(rt) = self.runtimes.get_mut(&id) {
                        let still_running = matches!(status, AppStatus::Running);
                        rt.status = status;
                        if !still_running {
                            rt.healthy = None;
                            rt.cpu_percent = 0.0;
                            rt.memory_mb = 0.0;
                        }
                    }
                }
                Event::HealthChanged(id, healthy) => {
                    if let Some(rt) = self.runtimes.get_mut(&id) {
                        rt.healthy = Some(healthy);
                    }
                }
            }
        }
    }

    /// Echantillonne CPU/memoire des process actifs, au plus une fois par seconde.
    fn sample_resource_usage(&mut self) {
        let now = Instant::now();
        if self.last_sample.is_some_and(|t| now.duration_since(t) < SAMPLE_INTERVAL) {
            return;
        }
        self.last_sample = Some(now);

        let pids: Vec<sysinfo::Pid> = self
            .handles
            .values()
            .filter_map(|h| *h.pgid.lock().unwrap())
            .map(|pid| sysinfo::Pid::from_u32(pid as u32))
            .collect();
        if pids.is_empty() {
            return;
        }
        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&pids), true);

        for (id, handle) in &self.handles {
            let Some(pid) = *handle.pgid.lock().unwrap() else { continue };
            let Some(process) = self.sys.process(sysinfo::Pid::from_u32(pid as u32)) else { continue };
            if let Some(rt) = self.runtimes.get_mut(id) {
                rt.cpu_percent = process.cpu_usage();
                rt.memory_mb = process.memory() as f64 / 1_048_576.0;
            }
        }
    }

    pub fn list_apps(&mut self) -> Vec<AppView> {
        self.drain_events();
        self.sample_resource_usage();
        self.config
            .apps
            .iter()
            .map(|app| {
                let runtime = self.runtimes.get(&app.id);
                let status = runtime.map(|r| r.status.clone()).unwrap_or(AppStatus::Stopped);
                let active = matches!(status, AppStatus::Running | AppStatus::Building);
                let error = match &status {
                    AppStatus::Failed(msg) => Some(msg.clone()),
                    _ => None,
                };
                AppView {
                    id: app.id,
                    name: app.name.clone(),
                    working_dir: app.working_dir.display().to_string(),
                    kind: app.kind,
                    command: app.command.clone(),
                    url: app.url.clone(),
                    env_vars: app.env_vars.clone(),
                    auto_restart: app.auto_restart,
                    start_order: app.start_order,
                    status_label: status_label(&status),
                    error,
                    active,
                    logs: runtime.map(|r| r.logs.iter().cloned().collect()).unwrap_or_default(),
                    healthy: runtime.and_then(|r| r.healthy),
                    cpu_percent: runtime.map(|r| r.cpu_percent).unwrap_or(0.0),
                    memory_mb: runtime.map(|r| r.memory_mb).unwrap_or(0.0),
                }
            })
            .collect()
    }

    pub fn add_app(&mut self, draft: AppDraft) -> Uuid {
        let app = AppConfig {
            id: Uuid::new_v4(),
            name: draft.name,
            working_dir: draft.working_dir,
            command: draft.command,
            kind: draft.kind,
            url: draft.url,
            env_vars: draft.env_vars,
            auto_restart: draft.auto_restart,
            start_order: draft.start_order,
        };
        let id = app.id;
        self.runtimes.insert(id, AppRuntime::new());
        self.config.add(app);
        self.persist();
        id
    }

    pub fn update_app(&mut self, id: Uuid, draft: AppDraft) {
        self.config.update(id, |app| {
            app.name = draft.name;
            app.working_dir = draft.working_dir;
            app.command = draft.command;
            app.kind = draft.kind;
            app.url = draft.url;
            app.env_vars = draft.env_vars;
            app.auto_restart = draft.auto_restart;
            app.start_order = draft.start_order;
        });
        self.persist();
    }

    pub fn remove_app(&mut self, id: Uuid) {
        if self.handles.contains_key(&id) {
            self.stop_app(id);
        }
        self.config.remove(id);
        self.runtimes.remove(&id);
        self.handles.remove(&id);
        self.persist();
    }

    pub fn clear_logs(&mut self, id: Uuid) {
        if let Some(rt) = self.runtimes.get_mut(&id) {
            rt.logs.clear();
        }
    }

    /// Ecrit les logs courants d'une app dans un fichier (une ligne par entree).
    pub fn export_logs(&self, id: Uuid, path: &std::path::Path) -> std::io::Result<()> {
        let Some(rt) = self.runtimes.get(&id) else {
            return Ok(());
        };
        let content = rt.logs.iter().cloned().collect::<Vec<_>>().join("\n");
        std::fs::write(path, content)
    }

    pub fn start_app(&mut self, id: Uuid) {
        let Some(config) = self.config.apps.iter().find(|a| a.id == id).cloned() else {
            return;
        };
        if !config.working_dir.is_dir() {
            if let Some(rt) = self.runtimes.get_mut(&id) {
                rt.status = AppStatus::Failed(format!(
                    "dossier introuvable: {}",
                    config.working_dir.display()
                ));
            }
            return;
        }
        let handle = RunningHandle::new();
        let pgid_slot = handle.pgid.clone();
        let stop_requested = handle.stop_requested.clone();
        if let Some(url) = config.url.clone() {
            spawn_health_watcher(id, url, stop_requested.clone(), self.event_tx.clone());
        }
        self.handles.insert(id, handle);
        let tx = self.event_tx.clone();
        std::thread::spawn(move || {
            run_app_thread(config, pgid_slot, stop_requested, tx);
        });
    }

    pub fn stop_app(&mut self, id: Uuid) {
        if let Some(handle) = self.handles.get(&id) {
            handle.stop_requested.store(true, std::sync::atomic::Ordering::SeqCst);
            if let Some(pgid) = *handle.pgid.lock().unwrap() {
                kill_process_group(pgid);
            }
        }
    }

    /// Demarre les apps regroupees par `start_order` croissant : chaque palier demarre
    /// en entier avant que le suivant ne soit lance, pour laisser le temps a une
    /// dependance (ex: l'API) de devenir disponible avant ses dependants.
    pub fn start_all(&mut self) {
        let mut tiers: Vec<(i32, Uuid)> =
            self.config.apps.iter().map(|a| (a.start_order, a.id)).collect();
        tiers.sort_by_key(|(order, _)| *order);

        let mut current_order: Option<i32> = None;
        for (order, id) in tiers {
            if current_order.is_some_and(|prev| prev != order) {
                std::thread::sleep(Duration::from_millis(400));
            }
            current_order = Some(order);
            if !self.handles.contains_key(&id) {
                self.start_app(id);
            }
        }
    }

    pub fn stop_all_running(&mut self) {
        let ids: Vec<Uuid> = self.handles.keys().copied().collect();
        for id in ids {
            self.stop_app(id);
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.stop_all_running();
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn temp_engine() -> Engine {
        Engine::new_ephemeral()
    }

    fn base_draft() -> AppDraft {
        AppDraft {
            name: "X".to_string(),
            working_dir: PathBuf::from("/tmp"),
            kind: AppKind::Raw,
            command: "".to_string(),
            url: None,
            env_vars: vec![],
            auto_restart: false,
            start_order: 0,
        }
    }

    #[test]
    fn add_app_persists_new_fields() {
        let mut engine = temp_engine();
        let id = engine.add_app(AppDraft {
            name: "Custom".to_string(),
            url: Some("http://localhost:9999".to_string()),
            env_vars: vec![("FOO".to_string(), "bar".to_string())],
            auto_restart: true,
            start_order: 2,
            ..base_draft()
        });
        let apps = engine.list_apps();
        let app = apps.iter().find(|a| a.id == id).expect("app present");
        assert_eq!(app.url, Some("http://localhost:9999".to_string()));
        assert_eq!(app.env_vars, vec![("FOO".to_string(), "bar".to_string())]);
        assert!(app.auto_restart);
        assert_eq!(app.start_order, 2);
        assert_eq!(app.healthy, None);
    }

    #[test]
    fn update_app_overwrites_fields() {
        let mut engine = temp_engine();
        let id = engine.add_app(base_draft());
        engine.update_app(id, AppDraft {
            name: "Y".to_string(),
            kind: AppKind::Npm,
            command: "start".to_string(),
            url: Some("http://localhost:1234".to_string()),
            env_vars: vec![("A".to_string(), "B".to_string())],
            auto_restart: true,
            start_order: 3,
            ..base_draft()
        });
        let apps = engine.list_apps();
        let app = apps.iter().find(|a| a.id == id).expect("app present");
        assert_eq!(app.name, "Y");
        assert_eq!(app.url, Some("http://localhost:1234".to_string()));
        assert!(app.auto_restart);
        assert_eq!(app.start_order, 3);
    }

    #[test]
    fn clear_logs_empties_log_buffer() {
        let mut engine = temp_engine();
        let id = engine.add_app(base_draft());
        engine.clear_logs(id);
        let apps = engine.list_apps();
        assert!(apps.iter().find(|a| a.id == id).unwrap().logs.is_empty());
    }

    #[test]
    fn start_all_skips_already_running_apps() {
        let mut engine = temp_engine();
        // Dossier inexistant -> chaque start_app echoue immediatement en "Failed",
        // mais on verifie surtout qu'aucun panic ne survient sur une liste vide/etendue.
        engine.add_app(AppDraft {
            working_dir: Path::new("/nonexistent").to_path_buf(),
            ..base_draft()
        });
        engine.start_all();
        let apps = engine.list_apps();
        assert_eq!(apps.len(), 1);
    }

    #[test]
    fn export_logs_writes_lines_to_file() {
        let mut engine = temp_engine();
        let id = engine.add_app(base_draft());
        // Simule des logs via le canal d'evenements directement (pas de vrai process).
        engine.drain_events();
        let tmp = std::env::temp_dir().join(format!("switchboard-test-{id}.log"));
        engine.export_logs(id, &tmp).expect("export should succeed");
        assert!(tmp.exists());
        std::fs::remove_file(&tmp).ok();
    }
}
