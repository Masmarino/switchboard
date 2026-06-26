import CSwitchboardFFI
import Foundation
import OSLog

private let logger = Logger(subsystem: "com.skolln.switchboard", category: "Engine")

final class DevtoolEngine {
    private let handle: OpaquePointer

    init() {
        handle = switchboard_engine_new()
    }

    deinit {
        switchboard_engine_free(handle)
    }

    func listApps() -> [AppEntry] {
        guard let raw = switchboard_engine_list_apps_json(handle) else { return [] }
        defer { switchboard_string_free(raw) }
        let data = Data(String(cString: raw).utf8)
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        do {
            return try decoder.decode([AppEntry].self, from: data)
        } catch {
            logger.error("Failed to decode app list: \(error.localizedDescription)")
            return []
        }
    }

    func addApp(_ draft: AppDraftPayload) {
        withDraftJSON(draft) { json in
            withCStrings(json) { json in switchboard_engine_add_app_json(handle, json) }
        }
    }

    func updateApp(id: String, draft: AppDraftPayload) {
        withDraftJSON(draft) { json in
            withCStrings(id, json) { id, json in switchboard_engine_update_app_json(handle, id, json) }
        }
    }

    func removeApp(id: String) {
        withCStrings(id) { id in switchboard_engine_remove_app(handle, id) }
    }

    func startApp(id: String) {
        withCStrings(id) { id in switchboard_engine_start_app(handle, id) }
    }

    func stopApp(id: String) {
        withCStrings(id) { id in switchboard_engine_stop_app(handle, id) }
    }

    func startAll() {
        switchboard_engine_start_all(handle)
    }

    func stopAll() {
        switchboard_engine_stop_all(handle)
    }

    func clearLogs(id: String) {
        withCStrings(id) { id in switchboard_engine_clear_logs(handle, id) }
    }

    @discardableResult
    func exportLogs(id: String, path: String) -> Bool {
        var result = false
        withCStrings(id, path) { id, path in result = switchboard_engine_export_logs(handle, id, path) }
        return result
    }

    private func withDraftJSON(_ draft: AppDraftPayload, _ body: (String) -> Void) {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        guard let data = try? encoder.encode(draft), let json = String(data: data, encoding: .utf8) else {
            logger.error("Failed to encode app draft")
            return
        }
        body(json)
    }
}

private func withCStrings(_ a: String, _ body: (UnsafePointer<CChar>) -> Void) {
    a.withCString { body($0) }
}

private func withCStrings(
    _ a: String, _ b: String,
    _ body: (UnsafePointer<CChar>, UnsafePointer<CChar>) -> Void
) {
    a.withCString { a in
        b.withCString { b in
            body(a, b)
        }
    }
}
