import ServiceManagement
import SwiftUI

struct GeneralPane: View {
    @Environment(AppSettings.self) private var settings

    var body: some View {
        @Bindable var settings = settings

        VStack(alignment: .leading, spacing: Theme.Spacing.lg) {
            GlassCard {
                VStack(spacing: Theme.Spacing.md) {
                    SettingsRow(
                        title: "Khởi động cùng máy",
                        subtitle: "Tự chạy Funput khi đăng nhập",
                        systemImage: "power"
                    ) {
                        Toggle("", isOn: $settings.launchAtLogin)
                            .labelsHidden()
                            .toggleStyle(.switch)
                            .onChange(of: settings.launchAtLogin) { _, enabled in
                                LoginItem.setEnabled(enabled)
                            }
                    }
                    Divider()
                    SettingsRow(
                        title: "Hiện biểu tượng thanh menu",
                        subtitle: "Đổi nhanh Telex/VNI và mở cài đặt",
                        systemImage: "menubar.rectangle"
                    ) {
                        Toggle("", isOn: $settings.showMenuBarIcon)
                            .labelsHidden()
                            .toggleStyle(.switch)
                    }
                }
            }
        }
    }
}

/// Register/unregister Funput as a login item (best-effort; silent in unsigned dev builds).
private enum LoginItem {
    static func setEnabled(_ enabled: Bool) {
        do {
            if enabled {
                try SMAppService.mainApp.register()
            } else {
                try SMAppService.mainApp.unregister()
            }
        } catch {
            // Ignore — typically only fails for unsigned/dev builds.
        }
    }
}

#Preview {
    GeneralPane()
        .environment(AppSettings.shared)
        .padding(Theme.Spacing.xl)
        .frame(width: 520)
}
