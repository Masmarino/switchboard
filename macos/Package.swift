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
                // Lien statique direct contre le .a (pas -L/-l, qui preferent le
                // .dylib voisin) — evite toute dependance a un chemin dylib au
                // runtime une fois l'app sortie de cet arbre de build.
                .unsafeFlags(["\(rustTargetDir)/libswitchboard_ffi.a"]),
                // sysinfo (dependance de switchboard-core) resout les noms
                // d'utilisateurs macOS via OpenDirectory — pas auto-linke par SwiftPM.
                .linkedFramework("OpenDirectory"),
            ]
        ),
    ]
)
