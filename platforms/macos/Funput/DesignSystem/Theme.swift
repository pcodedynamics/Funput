import SwiftUI

/// Spacing / radius / sizing tokens, so the whole app stays visually consistent.
enum Theme {
    enum Spacing {
        static let xs: CGFloat = 4
        static let sm: CGFloat = 8
        static let md: CGFloat = 14
        static let lg: CGFloat = 20
        static let xl: CGFloat = 32
    }

    enum Radius {
        static let card: CGFloat = 16
        static let control: CGFloat = 10
        static let pill: CGFloat = 999
    }

    /// Standard size for the Settings window content.
    static let settingsMinWidth: CGFloat = 720
    static let settingsMinHeight: CGFloat = 460
    static let sidebarWidth: CGFloat = 200
}
