import AppKit
import SwiftUI

struct ConfigExportSheet: View {
    @Environment(\.dismiss) private var dismiss
    let apps: [AppEntry]
    let onExport: (_ ids: [String], _ includeEnvVars: Bool) -> String?

    @State private var selectedIDs: Set<String> = []
    @State private var includeEnvVars = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            header

            VStack(alignment: .leading, spacing: 20) {
                section("Apps à exporter") {
                    ForEach(apps) { app in
                        Toggle(app.name, isOn: binding(for: app.id))
                    }
                }
                section("Options") {
                    Toggle("Inclure les variables d'environnement", isOn: $includeEnvVars)
                }
            }
            .padding(20)

            Divider()

            HStack {
                Spacer()
                Button("Annuler") { dismiss() }
                Button("Exporter…") { export() }
                    .buttonStyle(.borderedProminent)
                    .disabled(selectedIDs.isEmpty)
            }
            .padding(16)
        }
        .frame(width: 420)
        .onAppear { selectedIDs = Set(apps.map(\.id)) }
    }

    private var header: some View {
        HStack(spacing: 12) {
            ZStack {
                Circle().fill(Color.switchboardAccent).frame(width: 40, height: 40)
                Image(systemName: "square.and.arrow.up")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundStyle(.white)
            }
            VStack(alignment: .leading, spacing: 2) {
                Text("Exporter la config").font(.system(size: 18, weight: .bold))
                Text("Choisis les apps à inclure dans le fichier exporté")
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            }
            Spacer()
        }
        .padding(.horizontal, 20)
        .padding(.top, 20)
        .padding(.bottom, 6)
    }

    @ViewBuilder
    private func section<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(title.uppercased())
                .font(.system(size: 11, weight: .bold))
                .foregroundStyle(Color.switchboardAccent)
                .kerning(0.6)
            VStack(alignment: .leading, spacing: 12) {
                content()
            }
            .padding(14)
            .background(Color.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        }
    }

    private func binding(for id: String) -> Binding<Bool> {
        Binding(
            get: { selectedIDs.contains(id) },
            set: { isOn in
                if isOn { selectedIDs.insert(id) } else { selectedIDs.remove(id) }
            }
        )
    }

    private func export() {
        guard let json = onExport(Array(selectedIDs), includeEnvVars) else { return }
        let panel = NSSavePanel()
        panel.nameFieldStringValue = "switchboard-config.json"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        try? json.write(to: url, atomically: true, encoding: .utf8)
        dismiss()
    }
}
