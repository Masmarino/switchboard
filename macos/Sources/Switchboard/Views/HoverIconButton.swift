import SwiftUI

struct HoverIconButton: View {
    let systemName: String
    let help: String
    let action: () -> Void
    var tint: Color = .primary
    var disabled: Bool = false

    @State private var isHovering = false

    var body: some View {
        Button(action: action) {
            Image(systemName: systemName)
                .foregroundStyle(disabled ? Color.secondary.opacity(0.35) : tint)
                .frame(width: 22, height: 20)
                .background(
                    isHovering && !disabled ? Color.secondary.opacity(0.18) : .clear,
                    in: RoundedRectangle(cornerRadius: 5, style: .continuous)
                )
        }
        .buttonStyle(.plain)
        .disabled(disabled)
        .help(help)
        .onHover { hovering in isHovering = hovering }
    }
}
