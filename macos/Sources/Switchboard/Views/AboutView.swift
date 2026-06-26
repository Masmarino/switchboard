import AppKit
import SwiftUI

struct AboutView: View {
    var body: some View {
        VStack(spacing: 18) {
            Image(nsImage: NSApplication.shared.applicationIconImage ?? NSImage())
                .resizable()
                .frame(width: 96, height: 96)
                .clipShape(RoundedRectangle(cornerRadius: 20, style: .continuous))

            VStack(spacing: 4) {
                Text("Switchboard").font(.title2.bold())
                Text("Version 0.1.0").font(.caption).foregroundStyle(.secondary)
            }

            Text("Démarre, supervise et orchestre tes process de dev locaux — quel que soit le langage.")
                .font(.callout)
                .multilineTextAlignment(.center)
                .foregroundStyle(.secondary)
                .frame(maxWidth: 280)

            Divider()

            VStack(spacing: 10) {
                aboutLink(
                    title: "Développé par SkollN",
                    subtitle: "skolln.com",
                    url: "https://www.skolln.com"
                )
                aboutLink(
                    title: "Découvre aussi Alume",
                    subtitle: "Agrégateur de contenus avec IA intégrée",
                    url: "https://alume.skolln.com"
                )
                aboutLink(
                    title: "Code source",
                    subtitle: "Open source sous licence GPLv3",
                    url: "https://github.com/masmarino/switchboard"
                )
            }
        }
        .padding(28)
        .frame(width: 360)
    }

    private func aboutLink(title: String, subtitle: String, url: String) -> some View {
        Link(destination: URL(string: url)!) {
            VStack(alignment: .leading, spacing: 1) {
                Text(title).font(.subheadline.weight(.medium))
                Text(subtitle).font(.caption).foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .buttonStyle(.plain)
    }
}
