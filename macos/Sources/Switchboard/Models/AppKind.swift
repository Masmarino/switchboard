enum AppKind: String, Codable, CaseIterable {
    case cargo = "Cargo"
    case npm = "Npm"
    case dotnet = "Dotnet"
    case maven = "Maven"
    case python = "Python"
    case go = "Go"
    case raw = "Raw"

    var ffiValue: String { rawValue.lowercased() }
    var label: String { rawValue.uppercased() }
    var displayName: String { rawValue }
}
