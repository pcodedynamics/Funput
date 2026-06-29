import SwiftUI

struct KeyboardPane: View {
    @Environment(AppSettings.self) private var settings

    var body: some View {
        @Bindable var settings = settings

        VStack(alignment: .leading, spacing: Theme.Spacing.lg) {
            GlassCard {
                SettingsRow(
                    title: "Phím chuyển Việt / Anh",
                    subtitle: "Nhấn để tạm tắt gõ tiếng Việt. Bấm rồi nhấn tổ hợp kèm ⌃/⌥/⌘.",
                    systemImage: "globe"
                ) {
                    ShortcutRecorder(
                        combo: Binding(
                            get: { settings.toggleShortcut },
                            set: { if let combo = $0 { settings.toggleShortcut = combo } }
                        ),
                        allowOff: false
                    )
                }
            }

            GlassCard {
                SettingsRow(
                    title: "Phím lật từ vừa gõ",
                    subtitle: "Đổi từ đang gõ giữa tiếng Việt và chữ gốc (card ⇄ cải). Bấm rồi nhấn tổ hợp kèm ⌃/⌥/⌘.",
                    systemImage: "arrow.2.squarepath"
                ) {
                    ShortcutRecorder(combo: $settings.flipShortcut)
                }
            }
        }
    }
}

#Preview {
    KeyboardPane()
        .environment(AppSettings.shared)
        .padding(Theme.Spacing.xl)
        .frame(width: 520)
}
