struct AppDraftPayload: Encodable {
    let name: String
    let workingDir: String
    let kind: String
    let command: String
    let url: String?
    let envVars: [[String]]
    let autoRestart: Bool
    let startOrder: Int
}
