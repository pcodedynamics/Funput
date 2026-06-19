import AppKit
import SwiftUI

/// Guides the user to add Funput in System Settings → Keyboard → Input Sources.
/// The "enabled" check is stubbed here; it is wired to `TISCreateInputSourceList`
/// in the IME integration phase.
struct EnableInputSourceStep: View {
    @State private var isEnabled = false

    var body: some View {
        OnboardingStep(
            icon: "keyboard.badge.ellipsis",
            title: "Bật Funput",
            subtitle: "Thêm Funput vào nguồn nhập của macOS để dùng ở mọi app."
        ) {
            VStack(alignment: .leading, spacing: Theme.Spacing.md) {
                instruction(1, "Mở System Settings → Keyboard → Input Sources")
                instruction(2, "Bấm “+”, chọn Tiếng Việt → Funput")
                instruction(3, "Chọn Funput từ menu bàn phím trên thanh menu")

                Button {
                    openInputSources()
                } label: {
                    Label("Mở cài đặt Nguồn nhập", systemImage: "arrow.up.forward.app")
                }
                .buttonStyle(.glassProminent)

                statusRow
            }
            .frame(maxWidth: 380)
            .padding(.top, Theme.Spacing.sm)
        }
    }

    private func instruction(_ n: Int, _ text: String) -> some View {
        HStack(alignment: .top, spacing: Theme.Spacing.sm) {
            Text("\(n)")
                .font(.caption.bold())
                .frame(width: 20, height: 20)
                .glassEffect(.regular, in: .circle)
            Text(text)
            Spacer()
        }
    }

    private var statusRow: some View {
        HStack(spacing: Theme.Spacing.sm) {
            Image(systemName: isEnabled ? "checkmark.circle.fill" : "circle.dashed")
                .foregroundStyle(isEnabled ? AnyShapeStyle(.green) : AnyShapeStyle(.secondary))
            Text(isEnabled ? "Funput đã được bật" : "Chưa phát hiện Funput")
                .font(.callout)
                .foregroundStyle(.secondary)
        }
        .padding(.top, Theme.Spacing.xs)
    }

    private func openInputSources() {
        guard let url = URL(string: "x-apple.systempreferences:com.apple.Keyboard-Settings.extension") else {
            return
        }
        NSWorkspace.shared.open(url)
    }
}

#Preview {
    EnableInputSourceStep()
        .environment(AppSettings.shared)
        .frame(width: 580, height: 500)
}
