import AppKit
import Foundation
import Observation
import UniformTypeIdentifiers
import UserNotifications

@MainActor
@Observable
final class AppState {
    private let engine = DevtoolEngine()
    private var pollTask: Task<Void, Never>?
    private var lastStatus: [String: String] = [:]
    private var lastSeenRevision: UInt64 = 0
    private var sinceSeq: UInt64 = 0
    var selectedLogs: [String] = []
    /// Mirrors the Rust engine's own MAX_LOG_LINES cap — without this, a client that stays
    /// caught up with the server never hits the "replace" fallback that would otherwise
    /// reset this array, so it grows unbounded for the lifetime of the process.
    private static let maxDisplayedLogLines = 5000

    var apps: [AppEntry] = []
    var selectedID: String? {
        didSet {
            guard selectedID != oldValue else { return }
            sinceSeq = 0
            selectedLogs = []
            refresh(force: true)
        }
    }
    var addSheetPresented = false
    var exportSheetPresented = false
    var editingApp: AppEntry?
    var logFilter = ""

    var selected: AppEntry? {
        apps.first { $0.id == selectedID } ?? apps.first
    }

    var filteredLogs: [String] {
        guard selected != nil else { return [] }
        guard !logFilter.isEmpty else { return selectedLogs }
        let needle = logFilter.lowercased()
        return selectedLogs.filter { $0.lowercased().contains(needle) }
    }

    func start() {
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { _, _ in }
        refresh(force: true)
        pollTask = Task {
            while !Task.isCancelled {
                try? await Task.sleep(for: .milliseconds(200))
                let rev = engine.revision()
                if rev != lastSeenRevision {
                    lastSeenRevision = rev
                    refresh(force: false)
                }
            }
        }
    }

    func stop() {
        pollTask?.cancel()
    }

    func refresh(force: Bool) {
        let fetched = engine.listApps(selectedID: selectedID, sinceSeq: sinceSeq)
        if let id = selectedID, let selectedEntry = fetched.first(where: { $0.id == id }) {
            if selectedEntry.logsReplace {
                selectedLogs = selectedEntry.logs
            } else if !selectedEntry.logs.isEmpty {
                selectedLogs.append(contentsOf: selectedEntry.logs)
                if selectedLogs.count > Self.maxDisplayedLogLines {
                    selectedLogs.removeFirst(selectedLogs.count - Self.maxDisplayedLogLines)
                }
            }
            sinceSeq = selectedEntry.logsBaseSeq + UInt64(selectedEntry.logs.count)
        }
        let strippedForComparison = fetched.map { entry in
            AppEntry(
                id: entry.id, name: entry.name, workingDir: entry.workingDir, kind: entry.kind,
                command: entry.command, url: entry.url, envVars: entry.envVars, autoRestart: entry.autoRestart,
                startOrder: entry.startOrder, statusLabel: entry.statusLabel, error: entry.error, active: entry.active,
                logs: [], logsBaseSeq: 0, logsReplace: false,
                healthy: entry.healthy, cpuPercent: entry.cpuPercent, memoryMb: entry.memoryMb
            )
        }
        if force || strippedForComparison != apps {
            apps = strippedForComparison
        }
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
        refresh(force: true)
    }

    func stopApp(_ id: String) {
        engine.stopApp(id: id)
        refresh(force: true)
    }

    func removeApp(_ id: String) {
        engine.removeApp(id: id)
        if selectedID == id {
            selectedID = nil
        }
        refresh(force: true)
    }

    func startAll() {
        engine.startAll()
        refresh(force: true)
    }

    func stopAll() {
        engine.stopAll()
        refresh(force: true)
    }

    /// `applicationWillTerminate` fires before the process exits, but `DevtoolEngine.deinit`
    /// does not — the OS reclaims the process without running Swift's deinit chain on
    /// normal quit, which would otherwise leave supervised app process trees running as
    /// orphans holding their ports. Call this explicitly from the app delegate instead.
    func stopAllForShutdown() {
        engine.stopAll()
    }

    func clearLogs() {
        guard let id = selectedID else { return }
        engine.clearLogs(id: id)
        selectedLogs = []
        sinceSeq = 0
        refresh(force: true)
    }

    @discardableResult
    func exportLogs(id: String, toPath path: String) -> Bool {
        engine.exportLogs(id: id, path: path)
    }

    func addApp(_ draft: AppDraftPayload) {
        engine.addApp(draft)
        refresh(force: true)
    }

    func updateApp(id: String, draft: AppDraftPayload) {
        engine.updateApp(id: id, draft: draft)
        refresh(force: true)
    }

    func exportConfig(ids: [String], includeEnvVars: Bool) -> String? {
        engine.exportConfig(ids: ids, includeEnvVars: includeEnvVars)
    }

    /// Point d'entree unique du flux d'import : panneau d'ouverture natif -> apercu
    /// -> confirmation -> application. Auto-suffisant (pas de sheet SwiftUI
    /// necessaire), contrairement a l'export qui a besoin d'une UI de selection
    /// personnalisee.
    func importConfig() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.json]
        panel.allowsMultipleSelection = false
        guard panel.runModal() == .OK, let url = panel.url, let json = try? String(contentsOf: url, encoding: .utf8) else {
            return
        }
        guard let preview = engine.previewImportConfig(json: json) else {
            showAlert(title: "Fichier invalide", message: "Ce fichier ne contient pas une configuration Switchboard valide.")
            return
        }
        if preview.toAdd.isEmpty && preview.toReplace.isEmpty {
            showAlert(title: "Rien à importer", message: "Ce fichier ne contient aucune app à ajouter ou remplacer.")
            return
        }
        var lines: [String] = []
        if !preview.toAdd.isEmpty {
            lines.append("\(preview.toAdd.count) app(s) seront ajoutées : \(preview.toAdd.joined(separator: ", "))")
        }
        if !preview.toReplace.isEmpty {
            lines.append("\(preview.toReplace.count) app(s) seront remplacées : \(preview.toReplace.joined(separator: ", "))")
        }
        let alert = NSAlert()
        alert.messageText = "Importer cette configuration ?"
        alert.informativeText = lines.joined(separator: "\n")
        alert.addButton(withTitle: "Importer")
        alert.addButton(withTitle: "Annuler")
        guard alert.runModal() == .alertFirstButtonReturn else { return }
        _ = engine.applyImportConfig(json: json)
        refresh(force: true)
    }

    private func showAlert(title: String, message: String) {
        let alert = NSAlert()
        alert.messageText = title
        alert.informativeText = message
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }
}
