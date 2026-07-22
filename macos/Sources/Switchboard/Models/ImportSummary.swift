struct ImportSummary: Decodable {
    let toAdd: [String]
    let toReplace: [String]
    let invalid: Int
}
