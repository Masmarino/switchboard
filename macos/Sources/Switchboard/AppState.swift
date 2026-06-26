import Foundation
import Observation
import UserNotifications

@MainActor
@Observable
final class AppState {
    private let engine = DevtoolEngine()
    private var pollTask: Task<Void, Never>?
    private var lastStatus: [String: String] = [:]

    var apps: [AppEntry] = []
    var selectedID: String?
    var addSheetPresented = false
    var editingApp: AppEntry?
    var logFilter = ""

    var selected: AppEntry? {
        apps.first { $0.id == selectedID } ?? apps.first
    }

    var filteredLogs: [String] {
        guard let selected else { return [] }
        guard !logFilter.isEmpty else { return selected.logs }
        let needle = logFilter.lowercased()
        return selected.logs.filter { $0.lowercased().contains(needle) }
    }

    func start() {
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { _, _ in }
        refresh()
        pollTask = Task {
            while !Task.isCancelled {
                try? await Task.sleep(for: .milliseconds(200))
                refresh()
            }
        }
    }

    func stop() {
        pollTask?.cancel()
    }

    func refresh() {
        apps = engine.listApps()
        notifyNewFailures()
        if selectedID == nil {
            selectedID = apps.first?.id
        }
    }

    private func notifyNewFailures() {
        for app in apps {
            let previous = lastStatus[app.id]
            lastStatus[app.id] = app.statusLabel
            if app.statusLabel == "failed", previous != "failed" {
                let content = UNMutableNotificationContent()
                content.title = "\(app.name) a crashé"
                content.body = app.error ?? "Le process s'est arrêté de manière inattendue."
                content.sound = .default
                let request = UNNotificationRequest(identifier: "crash-\(app.id)", content: content, trigger: nil)
                UNUserNotificationCenter.current().add(request)
            }
        }
    }

    func startApp(_ id: String) {
        engine.startApp(id: id)
        selectedID = id
        refresh()
    }

    func stopApp(_ id: String) {
        engine.stopApp(id: id)
        refresh()
    }

    func removeApp(_ id: String) {
        engine.removeApp(id: id)
        refresh()
    }

    func startAll() {
        engine.startAll()
        refresh()
    }

    func stopAll() {
        engine.stopAll()
        refresh()
    }

    func clearLogs() {
        guard let id = selectedID else { return }
        engine.clearLogs(id: id)
        refresh()
    }

    @discardableResult
    func exportLogs(id: String, toPath path: String) -> Bool {
        engine.exportLogs(id: id, path: path)
    }

    func addApp(_ draft: AppDraftPayload) {
        engine.addApp(draft)
        refresh()
    }

    func updateApp(id: String, draft: AppDraftPayload) {
        engine.updateApp(id: id, draft: draft)
        refresh()
    }
}
