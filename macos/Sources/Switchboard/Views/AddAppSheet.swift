import SwiftUI
import AppKit

struct AddAppSheet: View {
    @Environment(\.dismiss) private var dismiss
    let existing: AppEntry?
    let onSave: (AppDraftPayload) -> Void

    @State private var name = ""
    @State private var workingDir = ""
    @State private var kind: AppKind = .cargo
    @State private var command = ""
    @State private var url = ""
    @State private var autoRestart = false
    @State private var envVarsText = ""
    @State private var startOrder = 0

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text(existing != nil ? "Modifier l'app" : "Ajouter une app").font(.headline)

            VStack(alignment: .leading, spacing: 10) {
                LabeledContent("Nom") { TextField("", text: $name) }
                LabeledContent("Dossier") {
                    HStack {
                        TextField("", text: $workingDir)
                        Button("Parcourir…") { pickFolder() }
                    }
                }
                LabeledContent("Type") {
                    Picker("", selection: $kind) {
                        ForEach(AppKind.allCases, id: \.self) { kind in
                            Text(kind.displayName).tag(kind)
                        }
                    }
                    .labelsHidden()
                }
                if kind != .cargo {
                    LabeledContent("Commande") { TextField("", text: $command) }
                }
                LabeledContent("URL") {
                    TextField("http://localhost:3000 (optionnel)", text: $url)
                }
                LabeledContent("Auto-restart") {
                    Toggle("", isOn: $autoRestart).labelsHidden()
                }
                LabeledContent("Ordre de démarrage") {
                    Stepper(value: $startOrder, in: 0...99) {
                        Text("\(startOrder)")
                    }
                }
                LabeledContent("Variables d'env") {
                    TextEditor(text: $envVarsText)
                        .font(.system(size: 12, design: .monospaced))
                        .frame(height: 70)
                        .overlay(RoundedRectangle(cornerRadius: 6).stroke(.separator))
                }
            }

            HStack {
                Spacer()
                Button("Annuler") { dismiss() }
                Button(existing != nil ? "Enregistrer" : "Ajouter") {
                    onSave(makeDraft())
                    dismiss()
                }
                .buttonStyle(.borderedProminent)
                .disabled(name.isEmpty || workingDir.isEmpty)
            }
        }
        .padding(20)
        .frame(width: 420)
        .onAppear { prefill() }
    }

    private func prefill() {
        guard let existing else { return }
        name = existing.name
        workingDir = existing.workingDir
        kind = existing.kind
        command = existing.command
        url = existing.url ?? ""
        autoRestart = existing.autoRestart
        envVarsText = existing.envVarsText
        startOrder = existing.startOrder
    }

    private func makeDraft() -> AppDraftPayload {
        let envVars = envVarsText
            .split(separator: "\n")
            .compactMap { line -> [String]? in
                let parts = line.split(separator: "=", maxSplits: 1).map(String.init)
                guard parts.count == 2, !parts[0].trimmingCharacters(in: .whitespaces).isEmpty else { return nil }
                return [parts[0].trimmingCharacters(in: .whitespaces), parts[1].trimmingCharacters(in: .whitespaces)]
            }
        let trimmedURL = url.trimmingCharacters(in: .whitespaces)
        return AppDraftPayload(
            name: name.trimmingCharacters(in: .whitespaces),
            workingDir: workingDir.trimmingCharacters(in: .whitespaces),
            kind: kind.ffiValue,
            command: command.trimmingCharacters(in: .whitespaces),
            url: trimmedURL.isEmpty ? nil : trimmedURL,
            envVars: envVars,
            autoRestart: autoRestart,
            startOrder: startOrder
        )
    }

    private func pickFolder() {
        let panel = NSOpenPanel()
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.allowsMultipleSelection = false
        if panel.runModal() == .OK, let url = panel.url {
            workingDir = url.path
        }
    }
}
