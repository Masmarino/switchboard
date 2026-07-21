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
        // `appDelegate` needs a reference to `state` to stop supervised apps on quit;
        // wiring it here (rather than `init`) is safe since `body` always runs at least
        // once before `applicationWillTerminate` could possibly fire.
        let _ = { appDelegate.state = state }()
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
