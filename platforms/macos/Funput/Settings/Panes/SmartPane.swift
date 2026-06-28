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
                    Divider()
                    SettingsRow(
                        title: "Kiểm tra chính tả",
                        subtitle: "Chỉ đặt dấu khi tạo thành âm tiết tiếng Việt hợp lệ",
                        systemImage: "checkmark.seal"
                    ) {
                        Toggle("", isOn: $settings.spellCheckEnabled)
                            .labelsHidden()
                            .toggleStyle(.switch)
                    }
                    Divider()
                    SettingsRow(
                        title: "Tự động viết hoa",
                        subtitle: "Viết hoa chữ đầu câu, sau dấu chấm và đầu dòng",
                        systemImage: "textformat.abc.dottedunderline"
                    ) {
                        Toggle("", isOn: $settings.autoCapitalizeEnabled)
                            .labelsHidden()
                            .toggleStyle(.switch)
                    }
                }
            }

            GlassCard {
                VStack(alignment: .leading, spacing: Theme.Spacing.sm) {
                    SectionHeader(title: "Ví dụ")
                    exampleRow("text", "text", "không phải tiếng Việt → giữ")
                    exampleRow("phus", "phú", "âm tiết hợp lệ → có dấu")
                    exampleRow("mixx", "mix", "gõ đúp dấu để ra tiếng Anh")
                    exampleRow("tetf", "tetf", "kiểm tra chính tả → không đặt dấu sai")
                    exampleRow("xong. roi", "Xong. Rồi", "tự động viết hoa đầu câu")
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
