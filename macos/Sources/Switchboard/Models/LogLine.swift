/// Stable id, since the array index shifts whenever the buffer gets trimmed.
struct LogLine: Identifiable, Equatable {
    let id: Int
    let text: String
}
