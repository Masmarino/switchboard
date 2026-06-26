struct AppEntry: Codable, Identifiable {
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
    let healthy: Bool?
    let cpuPercent: Double
    let memoryMb: Double

    var envVarsText: String {
        envVars.compactMap { pair in
            guard pair.count == 2 else { return nil }
            return "\(pair[0])=\(pair[1])"
        }.joined(separator: "\n")
    }

    var subtitle: String {
        guard active else { return statusLabel }
        let resource = String(format: "%.0f%% CPU · %.0f Mo", cpuPercent, memoryMb)
        switch healthy {
        case .some(true): return "\(statusLabel) · ✓ healthy · \(resource)"
        case .some(false): return "\(statusLabel) · ✗ ne répond pas · \(resource)"
        case .none: return "\(statusLabel) · \(resource)"
        }
    }
}
