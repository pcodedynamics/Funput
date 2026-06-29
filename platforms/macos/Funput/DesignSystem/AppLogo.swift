import AppKit
import SwiftUI

/// The Funput app icon shown as a Liquid Glass medallion, for hero / brand areas
/// (About, onboarding welcome). Falls back to an SF Symbol if the icon is
/// unavailable (e.g. in previews).
struct AppLogo: View {
    var size: CGFloat = 92

    var body: some View {
        Group {
            if let icon = NSApplication.shared.applicationIconImage {
                Image(nsImage: icon)
                    .resizable()
                    .interpolation(.high)
                    .scaledToFit()
            } else {
                Image(systemName: "character.bubble.fill")
                    .resizable()
                    .scaledToFit()
                    .foregroundStyle(.tint)
                    .padding(size * 0.16)
            }
        }
        .frame(width: size, height: size)
        .padding(Theme.Spacing.md)
        .glassEffect(.regular, in: .circle)
    }
}
