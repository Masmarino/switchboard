//! Shim C ABI pour le moteur `switchboard-core`, consomme par les frontends natifs
//! macOS (Swift) et Windows (C#). Toutes les structures riches passent en JSON pour
//! eviter le marshaling manuel de structs a travers la frontiere FFI — l'app ne
//! manipule qu'une poignee d'apps et quelques lignes de logs par seconde, le cout
//! JSON est negligeable face a la simplicite gagnee.

use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;
use std::str::FromStr;

use switchboard_core::{AppDraft, AppKind, Engine};
use serde::Deserialize;
use uuid::Uuid;

/// # Safety
/// `ptr` doit etre un pointeur C valide vers une chaine NUL-terminee, ou NULL.
unsafe fn c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().ok().map(str::to_owned)
}

fn string_to_c(s: String) -> *mut c_char {
    CString::new(s).map(CString::into_raw).unwrap_or(std::ptr::null_mut())
}

/// Miroir JSON de `AppDraft` — c'est le format echange par `switchboard_engine_add_app_json`
/// et `switchboard_engine_update_app_json`.
#[derive(Debug, Deserialize)]
struct FfiAppDraft {
    name: String,
    working_dir: String,
    kind: String,
    command: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    env_vars: Vec<(String, String)>,
    #[serde(default)]
    auto_restart: bool,
    #[serde(default)]
    start_order: i32,
}

impl From<FfiAppDraft> for AppDraft {
    fn from(d: FfiAppDraft) -> Self {
        let kind = match d.kind.as_str() {
            "cargo" => AppKind::Cargo,
            "npm" => AppKind::Npm,
            "dotnet" => AppKind::Dotnet,
            "maven" => AppKind::Maven,
            "python" => AppKind::Python,
            "go" => AppKind::Go,
            _ => AppKind::Raw,
        };
        AppDraft {
            name: d.name,
            working_dir: PathBuf::from(d.working_dir),
            kind,
            command: d.command,
            url: d.url,
            env_vars: d.env_vars,
            auto_restart: d.auto_restart,
            start_order: d.start_order,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn switchboard_engine_new() -> *mut Engine {
    Box::into_raw(Box::new(Engine::new()))
}

/// # Safety
/// `engine` doit provenir de [`switchboard_engine_new`] et ne doit plus etre utilise apres cet appel.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switchboard_engine_free(engine: *mut Engine) {
    if !engine.is_null() {
        drop(unsafe { Box::from_raw(engine) });
    }
}

/// Retourne la liste des apps (et leur etat courant) en JSON. La chaine retournee
/// doit etre liberee avec [`switchboard_string_free`].
///
/// # Safety
/// `engine` doit etre un pointeur valide retourne par [`switchboard_engine_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switchboard_engine_list_apps_json(engine: *mut Engine) -> *mut c_char {
    let engine = unsafe { &mut *engine };
    let apps = engine.list_apps();
    string_to_c(serde_json::to_string(&apps).unwrap_or_else(|_| "[]".to_string()))
}

/// `draft_json` doit correspondre a [`FfiAppDraft`] (cf. doc du module).
///
/// # Safety
/// `engine` doit etre un pointeur valide. `draft_json` doit etre une chaine C valide.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switchboard_engine_add_app_json(engine: *mut Engine, draft_json: *const c_char) {
    let engine = unsafe { &mut *engine };
    let Some(json) = (unsafe { c_str_to_string(draft_json) }) else { return };
    if let Ok(draft) = serde_json::from_str::<FfiAppDraft>(&json) {
        engine.add_app(draft.into());
    }
}

/// # Safety
/// `engine` doit etre un pointeur valide. `id` et `draft_json` doivent etre des
/// chaines C valides ; `draft_json` doit correspondre a [`FfiAppDraft`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switchboard_engine_update_app_json(
    engine: *mut Engine,
    id: *const c_char,
    draft_json: *const c_char,
) {
    let engine = unsafe { &mut *engine };
    let Some(id) = (unsafe { c_str_to_string(id) }).and_then(|s| Uuid::from_str(&s).ok()) else { return };
    let Some(json) = (unsafe { c_str_to_string(draft_json) }) else { return };
    if let Ok(draft) = serde_json::from_str::<FfiAppDraft>(&json) {
        engine.update_app(id, draft.into());
    }
}

/// # Safety
/// `engine` doit etre un pointeur valide. `id` doit etre une chaine C valide (UUID).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switchboard_engine_remove_app(engine: *mut Engine, id: *const c_char) {
    let engine = unsafe { &mut *engine };
    if let Some(id) = unsafe { c_str_to_string(id) }.and_then(|s| Uuid::from_str(&s).ok()) {
        engine.remove_app(id);
    }
}

/// # Safety
/// `engine` doit etre un pointeur valide. `id` doit etre une chaine C valide (UUID).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switchboard_engine_start_app(engine: *mut Engine, id: *const c_char) {
    let engine = unsafe { &mut *engine };
    if let Some(id) = unsafe { c_str_to_string(id) }.and_then(|s| Uuid::from_str(&s).ok()) {
        engine.start_app(id);
    }
}

/// # Safety
/// `engine` doit etre un pointeur valide. `id` doit etre une chaine C valide (UUID).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switchboard_engine_stop_app(engine: *mut Engine, id: *const c_char) {
    let engine = unsafe { &mut *engine };
    if let Some(id) = unsafe { c_str_to_string(id) }.and_then(|s| Uuid::from_str(&s).ok()) {
        engine.stop_app(id);
    }
}

/// # Safety
/// `engine` doit etre un pointeur valide. `id` doit etre une chaine C valide (UUID).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switchboard_engine_clear_logs(engine: *mut Engine, id: *const c_char) {
    let engine = unsafe { &mut *engine };
    if let Some(id) = unsafe { c_str_to_string(id) }.and_then(|s| Uuid::from_str(&s).ok()) {
        engine.clear_logs(id);
    }
}

/// Exporte les logs courants d'une app vers un fichier. Retourne `true` en cas de succes.
///
/// # Safety
/// `engine` doit etre un pointeur valide. `id` et `path` doivent etre des chaines C valides.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switchboard_engine_export_logs(
    engine: *mut Engine,
    id: *const c_char,
    path: *const c_char,
) -> bool {
    let engine = unsafe { &*engine };
    let Some(id) = (unsafe { c_str_to_string(id) }).and_then(|s| Uuid::from_str(&s).ok()) else { return false };
    let Some(path) = (unsafe { c_str_to_string(path) }) else { return false };
    engine.export_logs(id, std::path::Path::new(&path)).is_ok()
}

/// # Safety
/// `engine` doit etre un pointeur valide.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switchboard_engine_start_all(engine: *mut Engine) {
    let engine = unsafe { &mut *engine };
    engine.start_all();
}

/// # Safety
/// `engine` doit etre un pointeur valide.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switchboard_engine_stop_all(engine: *mut Engine) {
    let engine = unsafe { &mut *engine };
    engine.stop_all_running();
}

/// Libere une chaine retournee par une fonction de ce module.
///
/// # Safety
/// `s` doit provenir d'une fonction de ce module (ou etre NULL), et ne doit plus
/// etre utilise apres cet appel.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switchboard_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(unsafe { CString::from_raw(s) });
    }
}
