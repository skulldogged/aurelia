import Foundation

enum ServerURLNormalizer {
    static func normalizeForServer(raw: String) -> String? {
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return nil }

        let withScheme: String
        if hasSupportedScheme(trimmed) {
            withScheme = trimmed
        } else {
            withScheme = "https://\(trimmed)"
        }

        guard var components = URLComponents(string: withScheme) else { return nil }
        guard let scheme = components.scheme?.lowercased(), scheme == "http" || scheme == "https" else {
            return nil
        }
        guard let host = components.host, !host.isEmpty else { return nil }

        components.scheme = scheme

        var path = components.percentEncodedPath
        if path == "/" {
            path = ""
        } else if path.count > 1 {
            while path.hasSuffix("/") {
                path.removeLast()
            }
        }
        components.percentEncodedPath = path

        guard let normalized = components.string else { return nil }
        return normalized
    }

    static func isValidServerURL(_ value: String) -> Bool {
        guard let components = URLComponents(string: value) else { return false }
        guard let scheme = components.scheme?.lowercased(), scheme == "http" || scheme == "https" else {
            return false
        }
        guard let host = components.host, !host.isEmpty else { return false }
        return components.query == nil && components.fragment == nil
    }

    private static func hasSupportedScheme(_ value: String) -> Bool {
        let lowercased = value.lowercased()
        return lowercased.hasPrefix("http://") || lowercased.hasPrefix("https://")
    }
}
