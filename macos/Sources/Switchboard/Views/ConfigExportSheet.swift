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
                SectionCard(title: "Apps à exporter") {
                    ForEach(apps) { app in
                        Toggle(app.name, isOn: binding(for: app.id))
                    }
                }
                SectionCard(title: "Options") {
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
        SheetHeader(
            icon: "square.and.arrow.up",
            title: "Exporter la config",
            subtitle: "Choisis les apps à inclure dans le fichier exporté"
        )
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
