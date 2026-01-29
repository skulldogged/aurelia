// swift-tools-version: 6.2

import PackageDescription

let package = Package(
    name: "AureliaCore",
    platforms: [
        .iOS(.v26),
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
            dependencies: ["AureliaCoreFFI"],
            path: "Sources"
        ),
        .binaryTarget(
            name: "AureliaCoreFFI",
            path: "AureliaCoreFFI.xcframework"
        ),
    ]
)
