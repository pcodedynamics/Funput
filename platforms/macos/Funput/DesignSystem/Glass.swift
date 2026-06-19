import SwiftUI

/// A rounded Liquid Glass container for grouping content.
struct GlassCard<Content: View>: View {
    @ViewBuilder var content: Content

    var body: some View {
        content
            .padding(Theme.Spacing.lg)
            .frame(maxWidth: .infinity, alignment: .leading)
            .glassEffect(.regular, in: .rect(cornerRadius: Theme.Radius.card))
    }
}

/// A labelled settings row: title + optional subtitle on the left, control on the right.
struct SettingsRow<Control: View>: View {
    let title: String
    var subtitle: String? = nil
    var systemImage: String? = nil
    @ViewBuilder var control: Control

    var body: some View {
        HStack(alignment: .center, spacing: Theme.Spacing.md) {
            if let systemImage {
                Image(systemName: systemImage)
                    .font(.system(size: 16, weight: .medium))
                    .foregroundStyle(.tint)
                    .frame(width: 22)
            }
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.body)
                if let subtitle {
                    Text(subtitle)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            Spacer(minLength: Theme.Spacing.md)
            control
        }
        .padding(.vertical, Theme.Spacing.xs)
    }
}

/// A small section title above a group of rows/cards.
struct SectionHeader: View {
    let title: String

    var body: some View {
        Text(title)
            .font(.headline)
            .foregroundStyle(.secondary)
            .frame(maxWidth: .infinity, alignment: .leading)
    }
}

/// A keyboard key rendered as a glass keycap (for shortcut display).
struct KeyCap: View {
    let label: String

    var body: some View {
        Text(label)
            .font(.system(size: 12, weight: .semibold, design: .rounded))
            .monospacedDigit()
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .glassEffect(.regular, in: .rect(cornerRadius: 6))
    }
}

/// Horizontal stack of keycaps for a shortcut (`⌃` `\`).
struct ShortcutCaps: View {
    let caps: [String]

    var body: some View {
        HStack(spacing: Theme.Spacing.xs) {
            ForEach(caps, id: \.self) { KeyCap(label: $0) }
        }
    }
}

#Preview("Glass components") {
    VStack(alignment: .leading, spacing: Theme.Spacing.lg) {
        SectionHeader(title: "Preview")
        GlassCard {
            VStack(spacing: Theme.Spacing.sm) {
                SettingsRow(title: "Một tuỳ chọn", subtitle: "Mô tả ngắn", systemImage: "sparkles") {
                    Toggle("", isOn: .constant(true)).labelsHidden()
                }
                SettingsRow(title: "Phím chuyển") {
                    ShortcutCaps(caps: ["⌃", "\\"])
                }
            }
        }
    }
    .padding(Theme.Spacing.xl)
    .frame(width: 460)
}
