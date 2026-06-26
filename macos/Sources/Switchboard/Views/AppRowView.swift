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
                Text(app.error ?? app.subtitle)
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundStyle(app.error != nil ? Color.red : .secondary)
                    .lineLimit(1)
                Spacer()
                if let url = app.url, let nsurl = URL(string: url) {
                    Button(action: { NSWorkspace.shared.open(nsurl) }) {
                        Image(systemName: "safari")
                    }
                    .buttonStyle(.borderless)
                    .help("Ouvrir dans le navigateur")
                }
                Button(action: onEdit) {
                    Image(systemName: "pencil")
                }
                .buttonStyle(.borderless)

                Button(action: onStart) {
                    Image(systemName: "play.fill")
                }
                .buttonStyle(.borderless)
                .disabled(app.active)

                Button(action: onStop) {
                    Image(systemName: "stop.fill")
                }
                .buttonStyle(.borderless)
                .disabled(!app.active)

                Button(action: onRemove) {
                    Image(systemName: "trash")
                }
                .buttonStyle(.borderless)
            }
        }
        .padding(.vertical, 4)
    }
}
