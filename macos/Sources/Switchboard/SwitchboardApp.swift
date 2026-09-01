import SwiftUI

final class AppDelegate: NSObject, NSApplicationDelegate {
    var state: AppState?

    func applicationWillTerminate(_ notification: Notification) {
        state?.stopAllForShutdown()
    }
}

@main
struct SwitchboardApp: App {
    @State private var state = AppState()
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @Environment(\.openWindow) private var openWindow

    var body: some Scene {
        // Wired here rather than `init` — `body` always runs at least once before
        // `applicationWillTerminate` could possibly fire. `let _ =`, not a bare statement,
        // because SceneBuilder tries to treat a plain expression-statement as a Scene.
        let _ = (appDelegate.state = state)
        WindowGroup {
            ContentView(state: state)
        }
        .windowResizability(.contentSize)
        .defaultSize(width: 900, height: 600)
        .commands {
            CommandGroup(replacing: .appInfo) {
                Button("À propos de Switchboard") {
                    openWindow(id: "about")
                }
            }

            CommandGroup(after: .newItem) {
                Button("Exporter la config…") {
                    state.exportSheetPresented = true
                }
                .keyboardShortcut("e", modifiers: .command)

                Button("Importer une config…") {
                    state.importConfig()
                }
                .keyboardShortcut("i", modifiers: .command)
            }
        }

        WindowGroup(id: "about") {
            AboutView()
        }
        .windowResizability(.contentSize)
        .defaultPosition(.center)

        MenuBarExtra("Switchboard", systemImage: "terminal") {
            MenuBarContentView(state: state)
        }
    }
}
