import SwiftUI

struct SheetHeader: View {
    let icon: String
    let title: String
    let subtitle: String
    var tint: Color = .switchboardAccent

    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                Circle().fill(tint).frame(width: 40, height: 40)
                Image(systemName: icon)
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundStyle(.white)
            }
            VStack(alignment: .leading, spacing: 2) {
                Text(title).font(.system(size: 18, weight: .bold))
                Text(subtitle)
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            }
            Spacer()
        }
        .padding(.horizontal, 20)
        .padding(.top, 20)
        .padding(.bottom, 6)
    }
}
