import Foundation
import Observation

/// User preferences for Funput, persisted in `UserDefaults`. The Settings UI and
/// (later) the `IMKInputController` live in the same process, so they share this
/// store directly. `@Observable` drives live SwiftUI updates.
@Observable
final class AppSettings {
    static let shared = AppSettings()

    var inputMethod: InputMethod {
        didSet { defaults.set(inputMethod.rawValue, forKey: Keys.inputMethod) }
    }
    /// Whether Vietnamese composition is active (vs. English pass-through). Flipped by
    /// the toggle shortcut and the menu bar; read live by `FunputInputController`.
    var vietnameseEnabled: Bool {
        didSet { defaults.set(vietnameseEnabled, forKey: Keys.vietnameseEnabled) }
    }
    /// Auto-restore words that aren't valid Vietnamese (English typing).
    var smartEnglishRestore: Bool {
        didSet { defaults.set(smartEnglishRestore, forKey: Keys.smartEnglishRestore) }
    }
    /// Restore the instant a word becomes non-Vietnamese, without waiting for Space.
    var eagerRestore: Bool {
        didSet { defaults.set(eagerRestore, forKey: Keys.eagerRestore) }
    }
    var toggleShortcut: ToggleShortcut {
        didSet { defaults.set(toggleShortcut.rawValue, forKey: Keys.toggleShortcut) }
    }
    var launchAtLogin: Bool {
        didSet { defaults.set(launchAtLogin, forKey: Keys.launchAtLogin) }
    }
    var showMenuBarIcon: Bool {
        didSet { defaults.set(showMenuBarIcon, forKey: Keys.showMenuBarIcon) }
    }
    var hasCompletedOnboarding: Bool {
        didSet { defaults.set(hasCompletedOnboarding, forKey: Keys.hasCompletedOnboarding) }
    }

    @ObservationIgnored private let defaults: UserDefaults

    private init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        defaults.register(defaults: [
            Keys.smartEnglishRestore: true,
            Keys.eagerRestore: true,
            Keys.showMenuBarIcon: true,
            Keys.vietnameseEnabled: true,
        ])
        inputMethod = InputMethod(rawValue: defaults.integer(forKey: Keys.inputMethod)) ?? .telex
        vietnameseEnabled = defaults.bool(forKey: Keys.vietnameseEnabled)
        smartEnglishRestore = defaults.bool(forKey: Keys.smartEnglishRestore)
        eagerRestore = defaults.bool(forKey: Keys.eagerRestore)
        toggleShortcut = defaults.string(forKey: Keys.toggleShortcut)
            .flatMap(ToggleShortcut.init(rawValue:)) ?? .controlBackslash
        launchAtLogin = defaults.bool(forKey: Keys.launchAtLogin)
        showMenuBarIcon = defaults.bool(forKey: Keys.showMenuBarIcon)
        hasCompletedOnboarding = defaults.bool(forKey: Keys.hasCompletedOnboarding)
    }

    private enum Keys {
        static let inputMethod = "inputMethod"
        static let vietnameseEnabled = "vietnameseEnabled"
        static let smartEnglishRestore = "smartEnglishRestore"
        static let eagerRestore = "eagerRestore"
        static let toggleShortcut = "toggleShortcut"
        static let launchAtLogin = "launchAtLogin"
        static let showMenuBarIcon = "showMenuBarIcon"
        static let hasCompletedOnboarding = "hasCompletedOnboarding"
    }
}
