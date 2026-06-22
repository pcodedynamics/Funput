import SwiftUI

struct AboutPane: View {
    // Optional so SwiftUI previews (which don't inject the manager) still render.
    @Environment(UpdaterManager.self) private var updater: UpdaterManager?

    var body: some View {
        VStack(spacing: Theme.Spacing.lg) {
            GlassCard {
                VStack(spacing: Theme.Spacing.md) {
                    Image(systemName: "character.bubble.fill")
                        .font(.system(size: 56))
                        .foregroundStyle(.tint)
                    Text("Funput")
                        .font(.largeTitle.bold())
                    Text("Bộ gõ tiếng Việt — miễn phí, mã nguồn mở.")
                        .foregroundStyle(.secondary)
                    Text("Phiên bản \(appVersion)")
                        .font(.caption)
                        .foregroundStyle(.secondary)

                    if let updater {
                        Button("Kiểm tra cập nhật…") { updater.checkForUpdates() }
                            .buttonStyle(.glass)
                            .disabled(!updater.canCheckForUpdates)
                            .padding(.top, Theme.Spacing.sm)
                    }

                    HStack(spacing: Theme.Spacing.md) {
                        Link("GitHub", destination: URL(string: "https://github.com/Funput/Funput")!)
                        Link("Website", destination: URL(string: "https://funput.app/")!)
                    }
                    .buttonStyle(.glass)
                    .padding(.top, Theme.Spacing.sm)
                }
                .frame(maxWidth: .infinity)
            }
        }
    }

    private var appVersion: String {
        Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0"
    }
}

#Preview {
    AboutPane()
        .environment(AppSettings.shared)
        .padding(Theme.Spacing.xl)
        .frame(width: 520)
}
