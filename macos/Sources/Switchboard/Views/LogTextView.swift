import AppKit
import SwiftUI

/// Real NSTextView instead of a stack of Text views — gives native multi-line
/// selection and copy across rows, which SwiftUI can't do per-view.
struct LogTextView: NSViewRepresentable {
    let lines: [LogLine]

    func makeNSView(context: Context) -> NSScrollView {
        let textView = NSTextView()
        textView.isEditable = false
        textView.isSelectable = true
        textView.isRichText = false
        textView.drawsBackground = false
        textView.textContainerInset = NSSize(width: 10, height: 10)
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = false
        textView.autoresizingMask = [.width]
        textView.textContainer?.widthTracksTextView = true

        let scrollView = NSScrollView()
        scrollView.documentView = textView
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = false
        scrollView.drawsBackground = false

        context.coordinator.textView = textView
        return scrollView
    }

    func updateNSView(_ scrollView: NSScrollView, context: Context) {
        context.coordinator.update(lines: lines)
    }

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    @MainActor
    final class Coordinator {
        weak var textView: NSTextView?
        private var lastLines: [LogLine] = []

        private let attributes: [NSAttributedString.Key: Any] = [
            .font: NSFont.monospacedSystemFont(ofSize: 11, weight: .regular),
            .foregroundColor: NSColor(white: 0.84, alpha: 1),
        ]

        func update(lines: [LogLine]) {
            guard let textView, let storage = textView.textStorage else { return }
            defer { lastLines = lines }

            guard let firstID = lines.first?.id,
                  let matchIndex = lastLines.firstIndex(where: { $0.id == firstID }) else {
                storage.setAttributedString(NSAttributedString(string: lines.map(\.text).joined(separator: "\n"), attributes: attributes))
                textView.scrollToEndOfDocument(nil)
                return
            }

            // Patch just the delta (dropped prefix, appended suffix) instead of a full rebuild.
            if matchIndex > 0 {
                let droppedText = lastLines[0..<matchIndex].map(\.text).joined(separator: "\n") + "\n"
                let length = min((droppedText as NSString).length, storage.length)
                storage.deleteCharacters(in: NSRange(location: 0, length: length))
            }
            let survivorCount = lastLines.count - matchIndex
            if lines.count > survivorCount {
                let appended = lines.suffix(lines.count - survivorCount)
                let chunk = (storage.length > 0 ? "\n" : "") + appended.map(\.text).joined(separator: "\n")
                storage.append(NSAttributedString(string: chunk, attributes: attributes))
            }
            if matchIndex > 0 || lines.count > survivorCount {
                textView.scrollToEndOfDocument(nil)
            }
        }
    }
}
