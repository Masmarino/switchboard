import SwiftUI

struct KindBadge: View {
    let kind: AppKind

    var body: some View {
        Text(kind.label)
            .font(.system(size: 10, weight: .semibold, design: .monospaced))
            .foregroundStyle(.secondary)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(.secondary.opacity(0.12), in: .rect(cornerRadius: 5))
    }
}
