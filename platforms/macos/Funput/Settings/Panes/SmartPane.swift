import SwiftUI

struct SmartPane: View {
    @Environment(AppSettings.self) private var settings

    var body: some View {
        @Bindable var settings = settings

        VStack(alignment: .leading, spacing: Theme.Spacing.lg) {
            GlassCard {
                VStack(spacing: Theme.Spacing.md) {
                    SettingsRow(
                        title: "Tự khôi phục từ tiếng Anh",
                        subtitle: "Từ không phải tiếng Việt được giữ nguyên (text → text)",
                        systemImage: "wand.and.stars"
                    ) {
                        Toggle("", isOn: $settings.smartEnglishRestore)
                            .labelsHidden()
                            .toggleStyle(.switch)
                    }
                    Divider()
                    SettingsRow(
                        title: "Khôi phục tức thì",
                        subtitle: "Bật lại tiếng Anh ngay khi biết, không đợi dấu cách",
                        systemImage: "bolt"
                    ) {
                        Toggle("", isOn: $settings.eagerRestore)
                            .labelsHidden()
                            .toggleStyle(.switch)
                            .disabled(!settings.smartEnglishRestore)
                    }
                }
            }

            GlassCard {
                VStack(alignment: .leading, spacing: Theme.Spacing.sm) {
                    SectionHeader(title: "Ví dụ")
                    exampleRow("text", "text", "không phải tiếng Việt → giữ")
                    exampleRow("phus", "phú", "âm tiết hợp lệ → có dấu")
                    exampleRow("mixx", "mix", "gõ đúp dấu để ra tiếng Anh")
                }
            }
        }
    }

    private func exampleRow(_ from: String, _ to: String, _ note: String) -> some View {
        HStack(spacing: Theme.Spacing.sm) {
            Text(from).font(.system(.body, design: .monospaced))
            Image(systemName: "arrow.right").font(.caption).foregroundStyle(.secondary)
            Text(to).font(.system(.body, design: .monospaced)).foregroundStyle(.tint)
            Spacer()
            Text(note).font(.caption).foregroundStyle(.secondary)
        }
    }
}

#Preview {
    SmartPane()
        .environment(AppSettings.shared)
        .padding(Theme.Spacing.xl)
        .frame(width: 520)
}
