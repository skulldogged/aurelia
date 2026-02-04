// swift-tools-version: 6.2

import PackageDescription

let package = Package(
    name: "AureliaCore",
    platforms: [
        .iOS(.v26),
        .macOS(.v13),
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
