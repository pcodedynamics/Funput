import InputMethodKit
import SwiftUI

enum WindowID {
    static let settings = "settings"
    static let onboarding = "onboarding"
}

/// Hosts the IMKServer so the bundle works as a system input method. The same
/// process also renders the SwiftUI menu bar / Settings scenes below.
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var server: IMKServer?

    func applicationDidFinishLaunching(_ notification: Notification) {
        let connectionName = Bundle.main.infoDictionary?["InputMethodConnectionName"] as? String
            ?? "Funput_1_Connection"
        server = IMKServer(name: connectionName, bundleIdentifier: Bundle.main.bundleIdentifier)
    }
}

@main
struct FunputApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @State private var settings = AppSettings.shared
    @State private var updater = UpdaterManager()

    var body: some Scene {
        MenuBarExtra(isInserted: menuBarBinding) {
            StatusMenu()
                .environment(settings)
                .environment(updater)
        } label: {
            MenuBarLabel()
                .environment(settings)
        }

        Window("Funput Settings", id: WindowID.settings) {
            SettingsView()
                .environment(settings)
                .environment(updater)
                .frame(minWidth: Theme.settingsMinWidth, minHeight: Theme.settingsMinHeight)
        }
        .windowResizability(.contentSize)
        .defaultSize(width: Theme.settingsMinWidth, height: Theme.settingsMinHeight)

        Window("Chào mừng đến Funput", id: WindowID.onboarding) {
            OnboardingView()
                .environment(settings)
        }
        .windowResizability(.contentSize)
        .defaultPosition(.center)
    }

    private var menuBarBinding: Binding<Bool> {
        Binding(get: { settings.showMenuBarIcon }, set: { settings.showMenuBarIcon = $0 })
    }
}

/// The menu bar icon. Renders at launch, so it also kicks off first-run onboarding.
private struct MenuBarLabel: View {
    @Environment(AppSettings.self) private var settings
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        // "VI" when composing Vietnamese, "EN" when passing through.
        Text(settings.vietnameseEnabled ? "VI" : "EN")
            .task {
                if !settings.hasCompletedOnboarding {
                    openWindow(id: WindowID.onboarding)
                }
            }
    }
}
