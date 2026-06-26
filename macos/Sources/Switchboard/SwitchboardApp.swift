import SwiftUI

@main
struct SwitchboardApp: App {
    @State private var state = AppState()
    @Environment(\.openWindow) private var openWindow

    var body: some Scene {
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
