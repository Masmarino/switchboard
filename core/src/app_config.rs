use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppKind {
    Cargo,
    Npm,
    Dotnet,
    Maven,
    Python,
    Go,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub id: Uuid,
    pub name: String,
    pub working_dir: PathBuf,
    pub command: String,
    pub kind: AppKind,
    /// URL locale de l'app (ex: http://localhost:3000), pour "Ouvrir dans le navigateur".
    #[serde(default)]
    pub url: Option<String>,
    /// Variables d'environnement injectees dans le process lance.
    #[serde(default)]
    pub env_vars: Vec<(String, String)>,
    /// Relance automatiquement le process en cas de crash (sortie non nulle, hors arret demande).
    #[serde(default)]
    pub auto_restart: bool,
    /// Ordre de demarrage pour "Tout demarrer" (croissant). Les apps de meme ordre
    /// demarrent en parallele ; un ordre plus petit demarre avant un ordre plus grand
    /// (ex: une API avant les apps web qui en dependent).
    #[serde(default)]
    pub start_order: i32,
}

impl Default for AppKind {
    fn default() -> Self {
        Self::Raw
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfigList {
    pub apps: Vec<AppConfig>,
}

impl AppConfigList {
    pub fn add(&mut self, app: AppConfig) {
        self.apps.push(app);
    }

    pub fn remove(&mut self, id: Uuid) {
        self.apps.retain(|a| a.id != id);
    }

    pub fn update(&mut self, id: Uuid, mutate: impl FnOnce(&mut AppConfig)) {
        if let Some(app) = self.apps.iter_mut().find(|a| a.id == id) {
            mutate(app);
        }
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }

    pub fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("com", "skolln", "switchboard")
            .map(|dirs| dirs.config_dir().join("apps.json"))
    }

    /// Charge la config sauvegardee, ou une liste vide au premier lancement — pas de
    /// projet pre-rempli : Switchboard est un outil generique, sans connaissance des
    /// projets de la personne qui l'utilise.
    pub fn load_or_default() -> Self {
        if let Some(path) = Self::config_path() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(list) = Self::from_json(&contents) {
                    return list;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "no config dir available"))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = self
            .to_json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        std::fs::write(path, json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_appends_a_new_app() {
        let mut list = AppConfigList::default();
        assert_eq!(list.apps.len(), 0);
        list.add(AppConfig {
            id: Uuid::new_v4(),
            name: "Custom".to_string(),
            working_dir: PathBuf::from("/repo/custom"),
            command: "run".to_string(),
            kind: AppKind::Raw,
            url: None,
            env_vars: Vec::new(),
            auto_restart: false,
            start_order: 0,
        });
        assert_eq!(list.apps.len(), 1);
        assert_eq!(list.apps[0].name, "Custom");
    }

    #[test]
    fn remove_drops_app_by_id() {
        let mut list = AppConfigList::default();
        let app = AppConfig { name: "X".to_string(), ..Default::default() };
        let id = app.id;
        list.add(app);
        list.remove(id);
        assert!(list.apps.is_empty());
    }

    #[test]
    fn update_mutates_matching_app() {
        let mut list = AppConfigList::default();
        let app = AppConfig { name: "X".to_string(), ..Default::default() };
        let id = app.id;
        list.add(app);
        list.update(id, |app| {
            app.auto_restart = true;
            app.env_vars.push(("FOO".to_string(), "bar".to_string()));
        });
        assert!(list.apps[0].auto_restart);
        assert_eq!(list.apps[0].env_vars, vec![("FOO".to_string(), "bar".to_string())]);
    }

    #[test]
    fn json_round_trip_preserves_apps() {
        let mut list = AppConfigList::default();
        list.add(AppConfig {
            name: "X".to_string(),
            url: Some("http://localhost:3000".to_string()),
            ..Default::default()
        });
        let json = list.to_json().expect("serialize");
        let parsed = AppConfigList::from_json(&json).expect("deserialize");
        assert_eq!(parsed.apps.len(), list.apps.len());
        assert_eq!(parsed.apps[0].id, list.apps[0].id);
        assert_eq!(parsed.apps[0].name, list.apps[0].name);
        assert_eq!(parsed.apps[0].url, list.apps[0].url);
    }

    #[test]
    fn old_json_without_new_fields_still_parses() {
        let old_json = r#"{"apps":[{"id":"550e8400-e29b-41d4-a716-446655440000","name":"X","working_dir":"/r","command":"","kind":"Cargo"}]}"#;
        let list = AppConfigList::from_json(old_json).expect("should parse with defaults");
        assert_eq!(list.apps[0].url, None);
        assert_eq!(list.apps[0].start_order, 0);
        assert!(!list.apps[0].auto_restart);
        assert!(list.apps[0].env_vars.is_empty());
    }
}
