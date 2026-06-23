import AppKit
import Carbon
import SwiftUI

/// Guides the user to add Funput in System Settings → Keyboard → Input Sources,
/// reflecting live whether Funput is currently enabled as an input source.
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
        .onAppear(perform: refreshEnabled)
        // System Settings changes the enabled/selected sources in another process;
        // these distributed notifications let us update without polling.
        .onReceive(tisNotification(kTISNotifyEnabledKeyboardInputSourcesChanged)) { _ in
            refreshEnabled()
        }
        .onReceive(tisNotification(kTISNotifySelectedKeyboardInputSourceChanged)) { _ in
            refreshEnabled()
        }
    }

    private func tisNotification(_ name: CFString!) -> NotificationCenter.Publisher {
        DistributedNotificationCenter.default().publisher(for: Notification.Name(name as String))
    }

    private func refreshEnabled() {
        isEnabled = Self.funputIsEnabled()
    }

    /// True when a Funput input source is in the enabled list. Matches by bundle id
    /// so it covers every input mode the bundle vends.
    private static func funputIsEnabled() -> Bool {
        let bundleID = Bundle.main.bundleIdentifier ?? "app.funput.inputmethod.Funput"
        guard let list = TISCreateInputSourceList(nil, false)?.takeRetainedValue() as? [TISInputSource]
        else {
            return false
        }
        return list.contains { source in
            guard let ptr = TISGetInputSourceProperty(source, kTISPropertyBundleID) else {
                return false
            }
            let id = Unmanaged<CFString>.fromOpaque(ptr).takeUnretainedValue() as String
            return id == bundleID
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
