// swift-tools-version: 6.1

import PackageDescription

let package = Package(
    name: "AureliaMac",
    platforms: [
        .macOS("26.0"),
    ],
    products: [
        .executable(
            name: "AureliaMac",
            targets: ["AureliaMac"]
        ),
    ],
    dependencies: [
        .package(path: "../../mobile/ios/AureliaCore"),
    ],
    targets: [
        .executableTarget(
            name: "AureliaMac",
            dependencies: [
                .product(name: "AureliaCore", package: "AureliaCore"),
            ],
            path: "AureliaMac",
            exclude: ["Info.plist"]
        ),
        .testTarget(
            name: "AureliaMacTests",
            dependencies: ["AureliaMac"],
            path: "Tests"
        ),
    ]
)
