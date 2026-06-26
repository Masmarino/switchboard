// Genere par cbindgen — ne pas editer a la main.

#ifndef SWITCHBOARD_FFI_H
#define SWITCHBOARD_FFI_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Facade unique sur la config, les process en cours et leurs logs. Pas de dependance
 * UI : consomme directement en Rust (frontend Linux/GTK) ou via le shim FFI (macOS/Windows).
 */
typedef struct Engine Engine;

struct Engine *switchboard_engine_new(void);

/**
 * # Safety
 * `engine` doit provenir de [`switchboard_engine_new`] et ne doit plus etre utilise apres cet appel.
 */
void switchboard_engine_free(struct Engine *engine);

/**
 * Retourne la liste des apps (et leur etat courant) en JSON. La chaine retournee
 * doit etre liberee avec [`switchboard_string_free`].
 *
 * # Safety
 * `engine` doit etre un pointeur valide retourne par [`switchboard_engine_new`].
 */
char *switchboard_engine_list_apps_json(struct Engine *engine);

/**
 * `draft_json` doit correspondre a [`FfiAppDraft`] (cf. doc du module).
 *
 * # Safety
 * `engine` doit etre un pointeur valide. `draft_json` doit etre une chaine C valide.
 */
void switchboard_engine_add_app_json(struct Engine *engine, const char *draft_json);

/**
 * # Safety
 * `engine` doit etre un pointeur valide. `id` et `draft_json` doivent etre des
 * chaines C valides ; `draft_json` doit correspondre a [`FfiAppDraft`].
 */
void switchboard_engine_update_app_json(struct Engine *engine,
                                        const char *id,
                                        const char *draft_json);

/**
 * # Safety
 * `engine` doit etre un pointeur valide. `id` doit etre une chaine C valide (UUID).
 */
void switchboard_engine_remove_app(struct Engine *engine, const char *id);

/**
 * # Safety
 * `engine` doit etre un pointeur valide. `id` doit etre une chaine C valide (UUID).
 */
void switchboard_engine_start_app(struct Engine *engine, const char *id);

/**
 * # Safety
 * `engine` doit etre un pointeur valide. `id` doit etre une chaine C valide (UUID).
 */
void switchboard_engine_stop_app(struct Engine *engine, const char *id);

/**
 * # Safety
 * `engine` doit etre un pointeur valide. `id` doit etre une chaine C valide (UUID).
 */
void switchboard_engine_clear_logs(struct Engine *engine, const char *id);

/**
 * Exporte les logs courants d'une app vers un fichier. Retourne `true` en cas de succes.
 *
 * # Safety
 * `engine` doit etre un pointeur valide. `id` et `path` doivent etre des chaines C valides.
 */
bool switchboard_engine_export_logs(struct Engine *engine, const char *id, const char *path);

/**
 * # Safety
 * `engine` doit etre un pointeur valide.
 */
void switchboard_engine_start_all(struct Engine *engine);

/**
 * # Safety
 * `engine` doit etre un pointeur valide.
 */
void switchboard_engine_stop_all(struct Engine *engine);

/**
 * Libere une chaine retournee par une fonction de ce module.
 *
 * # Safety
 * `s` doit provenir d'une fonction de ce module (ou etre NULL), et ne doit plus
 * etre utilise apres cet appel.
 */
void switchboard_string_free(char *s);

#endif  /* SWITCHBOARD_FFI_H */
