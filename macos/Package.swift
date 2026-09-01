// swift-tools-version: 6.2
import Foundation
import PackageDescription

let rustTargetDir = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .appendingPathComponent("target/release")
    .path

let package = Package(
    name: "Switchboard",
    platforms: [.macOS(.v26)],
    targets: [
        .systemLibrary(name: "CSwitchboardFFI"),
        .executableTarget(
            name: "Switchboard",
            dependencies: ["CSwitchboardFFI"],
            linkerSettings: [
                // Lien direct sur le .a, pas -L/-l : evite toute dependance a un chemin
                // dylib une fois l'app sortie de cet arbre de build.
                .unsafeFlags(["\(rustTargetDir)/libswitchboard_ffi.a"]),
                // sysinfo (via switchboard-core) resout les noms d'utilisateurs macOS par
                // OpenDirectory, que SwiftPM ne linke pas automatiquement.
                .linkedFramework("OpenDirectory"),
            ]
        ),
    ]
)
