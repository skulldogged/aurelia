// swift-tools-version: 6.1

import PackageDescription

let package = Package(
    name: "AureliaCore",
    platforms: [
        .iOS("26.0"),
        .macOS("26.0"),
    ],
    products: [
        .library(
            name: "AureliaCore",
            targets: ["AureliaCore"]
        ),
    ],
    targets: [
        .target(
            name: "AureliaCore",
            dependencies: ["aurelia_coreFFI"],
            path: "Sources"
        ),
        .binaryTarget(
            name: "aurelia_coreFFI",
            path: "AureliaCoreFFI.xcframework"
        ),
        .testTarget(
            name: "AureliaCoreTests",
            dependencies: ["AureliaCore"],
            path: "Tests"
        ),
    ]
)
