import SwiftUI

struct LogPanelView: View {
    let hasApp: Bool
    let lines: [LogLine]

    var body: some View {
        Group {
            if hasApp, !lines.isEmpty {
                LogTextView(lines: lines)
            } else {
                VStack(spacing: 4) {
                    Text("Pas encore de logs")
                        .font(.system(size: 13, design: .monospaced))
                        .foregroundStyle(.secondary)
                    Text("Démarre l'app pour voir sa sortie ici.")
                        .font(.system(size: 11, design: .monospaced))
                        .foregroundStyle(.tertiary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .background(Color(red: 0.11, green: 0.11, blue: 0.12), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .strokeBorder(Color.white.opacity(0.08))
        )
    }
}
