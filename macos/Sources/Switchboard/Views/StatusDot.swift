import SwiftUI

struct StatusDot: View {
    let label: String
    let active: Bool

    private var color: Color {
        switch label {
        case "running": .green
        case "building": .orange
        case "failed": .red
        default: .gray
        }
    }

    var body: some View {
        Circle()
            .fill(color)
            .frame(width: 9, height: 9)
            .shadow(color: active ? color.opacity(0.7) : .clear, radius: active ? 5 : 0)
    }
}
