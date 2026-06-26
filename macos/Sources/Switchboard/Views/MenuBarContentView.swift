import AppKit
import SwiftUI

struct MenuBarContentView: View {
    @Bindable var state: AppState

    var body: some View {
        ForEach(state.apps) { app in
            Button(action: { app.active ? state.stopApp(app.id) : state.startApp(app.id) }) {
                Text("\(app.active ? "■" : "▶")  \(app.name) — \(app.statusLabel)")
            }
        }

        Divider()

        Button("Tout démarrer") { state.startAll() }
        Button("Tout arrêter") { state.stopAll() }

        Divider()

        Button("Quitter Switchboard") {
            NSApplication.shared.terminate(nil)
        }
        .keyboardShortcut("q")
    }
}
