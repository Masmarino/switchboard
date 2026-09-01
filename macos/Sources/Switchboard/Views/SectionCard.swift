import SwiftUI

struct SectionCard<Content: View>: View {
    let title: String
    var spacing: CGFloat = 12
    var padding: EdgeInsets = EdgeInsets(top: 14, leading: 14, bottom: 14, trailing: 14)
    @ViewBuilder let content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(title.uppercased())
                .font(.system(size: 11, weight: .bold))
                .foregroundStyle(Color.switchboardAccent)
                .kerning(0.6)
            VStack(alignment: .leading, spacing: spacing) {
                content
            }
            .padding(padding)
            .background(Color.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        }
    }
}
