import AppKit
import SwiftUI

/// Contents of the menu bar dropdown: switch method, open windows, quit.
struct StatusMenu: View {
    @Environment(AppSettings.self) private var settings
    @Environment(UpdaterManager.self) private var updater
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        @Bindable var settings = settings

        Toggle(isOn: $settings.vietnameseEnabled) {
            Text(settings.vietnameseEnabled ? "Tiếng Việt (đang bật)" : "Tiếng Việt (đang tắt)")
        }
        .keyboardShortcut("\\", modifiers: .control)

        Divider()

        Picker("Phương thức gõ", selection: $settings.inputMethod) {
            ForEach(InputMethod.allCases) { method in
                Text(method.displayName).tag(method)
            }
        }
        .pickerStyle(.inline)

        Divider()

        Button("Kiểm tra cập nhật…") { updater.checkForUpdates() }
            .disabled(!updater.canCheckForUpdates)
        Button("Cài đặt Funput…") { open(WindowID.settings) }
            .keyboardShortcut(",", modifiers: .command)
        Button("Hướng dẫn…") { open(WindowID.onboarding) }

        Divider()

        Button("Thoát Funput") { NSApp.terminate(nil) }
            .keyboardShortcut("q", modifiers: .command)
    }

    private func open(_ id: String) {
        openWindow(id: id)
        NSApp.activate(ignoringOtherApps: true)
    }
}
