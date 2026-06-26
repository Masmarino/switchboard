enum AppKind: String, Codable, CaseIterable {
    case cargo = "Cargo"
    case npm = "Npm"
    case dotnet = "Dotnet"
    case maven = "Maven"
    case python = "Python"
    case go = "Go"
    case raw = "Raw"

    var ffiValue: String {
        switch self {
        case .cargo: "cargo"
        case .npm: "npm"
        case .dotnet: "dotnet"
        case .maven: "maven"
        case .python: "python"
        case .go: "go"
        case .raw: "raw"
        }
    }

    var label: String {
        switch self {
        case .cargo: "CARGO"
        case .npm: "NPM"
        case .dotnet: "DOTNET"
        case .maven: "MAVEN"
        case .python: "PYTHON"
        case .go: "GO"
        case .raw: "RAW"
        }
    }

    var displayName: String {
        switch self {
        case .cargo: "Cargo"
        case .npm: "Npm"
        case .dotnet: "Dotnet"
        case .maven: "Maven"
        case .python: "Python"
        case .go: "Go"
        case .raw: "Raw"
        }
    }
}
