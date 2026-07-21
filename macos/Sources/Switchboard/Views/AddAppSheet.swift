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
        VStack(alignment: .leading, spacing: 0) {
            header

            VStack(alignment: .leading, spacing: 20) {
                section("Général") {
                    field("Nom", icon: "textformat") {
                        TextField("Alume API", text: $name)
                            .textFieldStyle(.roundedBorder)
                    }
                    field("Dossier", icon: "folder") {
                        HStack(spacing: 8) {
                            TextField("/chemin/vers/le/projet", text: $workingDir)
                                .textFieldStyle(.roundedBorder)
                            Button("Parcourir…") { pickFolder() }
                        }
                    }
                    field("Type", icon: "shippingbox") {
                        Picker("", selection: $kind) {
                            ForEach(AppKind.allCases, id: \.self) { kind in
                                Text(kind.displayName).tag(kind)
                            }
                        }
                        .labelsHidden()
                    }
                    if kind != .cargo {
                        field("Commande", icon: "terminal") {
                            TextField("start", text: $command)
                                .textFieldStyle(.roundedBorder)
                        }
                    }
                }

                section("Exécution") {
                    field("URL", icon: "globe") {
                        TextField("http://localhost:3000 (optionnel)", text: $url)
                            .textFieldStyle(.roundedBorder)
                    }
                    field("Auto-restart", icon: "arrow.clockwise") {
                        Toggle("", isOn: $autoRestart)
                            .labelsHidden()
                            .toggleStyle(.switch)
                    }
                    field("Ordre de démarrage", icon: "list.number") {
                        Stepper(value: $startOrder, in: 0...99) {
                            Text("\(startOrder)")
                        }
                    }
                }

                section("Avancé") {
                    field("Variables d'env", icon: "curlybraces") {
                        TextEditor(text: $envVarsText)
                            .font(.system(size: 12, design: .monospaced))
                            .frame(height: 70)
                            .overlay(RoundedRectangle(cornerRadius: 6).stroke(.separator))
                            .overlay(alignment: .topLeading) {
                                if envVarsText.isEmpty {
                                    Text("CLE=valeur\nAUTRE_CLE=valeur")
                                        .font(.system(size: 12, design: .monospaced))
                                        .foregroundStyle(.tertiary)
                                        .padding(6)
                                        .allowsHitTesting(false)
                                }
                            }
                    }
                }
            }
            .padding(20)

            Divider()

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
            .padding(16)
        }
        .frame(width: 500)
        .onAppear { prefill() }
    }

    private var header: some View {
        HStack(spacing: 12) {
            ZStack {
                Circle()
                    .fill(sectionTitleColor)
                    .frame(width: 40, height: 40)
                Image(systemName: existing != nil ? "pencil" : "plus")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundStyle(.white)
            }
            VStack(alignment: .leading, spacing: 2) {
                Text(existing != nil ? "Modifier l'app" : "Ajouter une app")
                    .font(.system(size: 18, weight: .bold))
                Text(existing != nil ? "Mets à jour la configuration de cette app" : "Configure une nouvelle app à superviser")
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            }
            Spacer()
        }
        .padding(.horizontal, 20)
        .padding(.top, 20)
        .padding(.bottom, 6)
    }

    private let sectionTitleColor = Color.switchboardAccent
    private let labelColumnWidth: CGFloat = 165

    @ViewBuilder
    private func section<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(title.uppercased())
                .font(.system(size: 11, weight: .bold))
                .foregroundStyle(sectionTitleColor)
                .kerning(0.6)
            VStack(alignment: .leading, spacing: 12) {
                content()
            }
            .padding(14)
            .background(Color.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        }
    }

    @ViewBuilder
    private func field<Content: View>(_ label: String, icon: String, @ViewBuilder content: () -> Content) -> some View {
        LabeledContent {
            content()
        } label: {
            HStack(spacing: 6) {
                Image(systemName: icon)
                    .foregroundStyle(.secondary)
                    .frame(width: 16)
                Text(label)
                    .foregroundStyle(.primary)
                Spacer(minLength: 0)
            }
            .frame(width: labelColumnWidth, alignment: .leading)
        }
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
}
