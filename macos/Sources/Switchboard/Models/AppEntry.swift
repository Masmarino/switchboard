struct AppEntry: Codable, Identifiable, Equatable {
    let id: String
    let name: String
    let workingDir: String
    let kind: AppKind
    let command: String
    let url: String?
    let envVars: [[String]]
    let autoRestart: Bool
    let startOrder: Int
    let statusLabel: String
    let error: String?
    let active: Bool
    let logs: [String]
    let logsBaseSeq: UInt64
    let logsReplace: Bool
    let healthy: Bool?
    let cpuPercent: Double
    let memoryMb: Double

    var envVarsText: String {
        envVars.compactMap { pair in
            guard pair.count == 2 else { return nil }
            return "\(pair[0])=\(pair[1])"
        }.joined(separator: "\n")
    }

    var resourceLine: String {
        let resource = String(format: "%.0f%% CPU · %.0f Mo", cpuPercent, memoryMb)
        switch healthy {
        case .some(true): return "✓ healthy · \(resource)"
        case .some(false): return "✗ ne répond pas · \(resource)"
        case .none: return resource
        }
    }

    /// Ignores logs/logsBaseSeq/logsReplace — AppState compares fetched apps against the
    /// current list to skip unnecessary writes, and logs are tracked separately anyway.
    /// cpuPercent/memoryMb compare at the rounded precision resourceLine actually displays,
    /// so per-tick measurement jitter doesn't force a reassignment/re-render on its own.
    static func == (lhs: AppEntry, rhs: AppEntry) -> Bool {
        lhs.id == rhs.id && lhs.name == rhs.name && lhs.workingDir == rhs.workingDir
            && lhs.kind == rhs.kind && lhs.command == rhs.command && lhs.url == rhs.url
            && lhs.envVars == rhs.envVars && lhs.autoRestart == rhs.autoRestart
            && lhs.startOrder == rhs.startOrder && lhs.statusLabel == rhs.statusLabel
            && lhs.error == rhs.error && lhs.active == rhs.active && lhs.healthy == rhs.healthy
            && lhs.cpuPercent.rounded() == rhs.cpuPercent.rounded()
            && lhs.memoryMb.rounded() == rhs.memoryMb.rounded()
    }
}
