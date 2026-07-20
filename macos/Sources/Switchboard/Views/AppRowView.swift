import SwiftUI
import AppKit

struct AppRowView: View {
    let app: AppEntry
    let onStart: () -> Void
    let onStop: () -> Void
    let onRemove: () -> Void
    let onEdit: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                StatusDot(label: app.statusLabel, active: app.active)
                Text(app.name).font(.system(size: 14, weight: .semibold))
                Spacer()
                KindBadge(kind: app.kind)
            }
            HStack {
                Text(app.error ?? app.statusLabel)
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundStyle(app.error != nil ? Color.red : .secondary)
                    .lineLimit(1)
                Spacer()
                if let url = app.url, let nsurl = URL(string: url) {
                    HoverIconButton(systemName: "safari", help: "Ouvrir dans le navigateur") {
                        NSWorkspace.shared.open(nsurl)
                    }
                }
                HoverIconButton(systemName: "pencil", help: "Modifier", action: onEdit)

                HoverIconButton(systemName: "play.fill", help: "Démarrer", action: onStart, disabled: app.active)

                HoverIconButton(systemName: "stop.fill", help: "Arrêter", action: onStop, disabled: !app.active)

                HoverIconButton(systemName: "trash", help: "Supprimer", action: onRemove, tint: .red)
            }
            if app.active, app.error == nil {
                Text(app.resourceLine)
                    .font(.system(size: 10, design: .monospaced))
                    .foregroundStyle(.tertiary)
            }
        }
        .padding(.vertical, 4)
    }
}
