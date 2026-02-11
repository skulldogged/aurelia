import SwiftUI

enum AureliaSpacing {
    static let xs: CGFloat = 6
    static let s: CGFloat = 10
    static let m: CGFloat = 16
    static let l: CGFloat = 24
    static let xl: CGFloat = 32
    static let xxl: CGFloat = 40
}

enum AureliaRadius {
    static let s: CGFloat = 10
    static let m: CGFloat = 16
    static let l: CGFloat = 24
    static let xl: CGFloat = 32
}

enum AureliaLayout {
    static func isWide(_ width: CGFloat) -> Bool {
        width >= 720
    }
}

enum AureliaPalette {
    static func tint(for scheme: ColorScheme) -> Color {
        switch scheme {
        case .dark:
            Color(red: 0.70, green: 0.55, blue: 1.00)
        default:
            Color(red: 0.46, green: 0.34, blue: 0.95)
        }
    }

    static func glassBorder(for scheme: ColorScheme) -> Color {
        switch scheme {
        case .dark:
            Color.white.opacity(0.18)
        default:
            Color.white.opacity(0.35)
        }
    }

    static func shadowColor(for scheme: ColorScheme) -> Color {
        scheme == .dark ? Color.black.opacity(0.45) : Color.black.opacity(0.15)
    }
}

struct AureliaBackground: View {
    var body: some View {
        Color(.systemBackground)
            .ignoresSafeArea()
    }
}

struct GlassCard<Content: View>: View {
    @Environment(\.colorScheme) private var scheme
    var cornerRadius: CGFloat = AureliaRadius.l
    var padding: CGFloat = AureliaSpacing.m
    var showsShadow: Bool = true
    @ViewBuilder var content: () -> Content

    var body: some View {
        content()
            .padding(padding)
            .background(
                .ultraThinMaterial,
                in: RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
            )
            .clipShape(RoundedRectangle(cornerRadius: cornerRadius, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
                    .stroke(AureliaPalette.glassBorder(for: scheme), lineWidth: 1)
            )
            .shadow(
                color: showsShadow ? AureliaPalette.shadowColor(for: scheme) : .clear,
                radius: showsShadow ? 14 : 0,
                x: 0,
                y: 8
            )
    }
}

struct AureliaSectionHeader: View {
    let title: String
    var subtitle: String?

    var body: some View {
        HStack(alignment: .firstTextBaseline) {
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.title3.bold())
                if let subtitle {
                    Text(subtitle)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            Spacer()
        }
        .padding(.horizontal, AureliaSpacing.m)
    }
}

extension View {
    func aureliaScreen() -> some View {
        #if targetEnvironment(macCatalyst)
            background(AureliaBackground())
        #else
            background(AureliaBackground())
                .navigationBarTitleDisplayMode(.inline)
        #endif
    }

    func aureliaInsetCard(cornerRadius: CGFloat = AureliaRadius.l) -> some View {
        GlassCard(cornerRadius: cornerRadius) {
            self
        }
    }

    func aureliaRootTabHeader(_ title: String) -> some View {
        modifier(AureliaRootTabHeaderModifier(title: title))
    }
}

private struct AureliaRootTabHeaderModifier: ViewModifier {
    let title: String
    @Environment(\.tabBarPlacement) private var tabBarPlacement

    private var resolvedTitle: String {
        if tabBarPlacement == .sidebar {
            return title
        }
        if tabBarPlacement == .topBar {
            return ""
        }
        return title
    }

    func body(content: Content) -> some View {
        #if targetEnvironment(macCatalyst)
            content
                .navigationTitle(resolvedTitle)
        #else
            content
                .navigationTitle(resolvedTitle)
                .navigationBarTitleDisplayMode(.inline)
        #endif
    }
}
