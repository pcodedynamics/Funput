import Foundation
import Observation

/// An app where Funput stays out of the way: while it's the focused client, keys
/// pass straight through (English), regardless of the VI/EN toggle. Matched by the
/// client's bundle identifier; `name` is only for display in Settings.
struct ExcludedApp: Codable, Identifiable, Hashable {
    let id: String
    let name: String
}

/// A text-expansion shortcut (gõ tắt): typing `trigger` then a word boundary injects
/// `expansion` (`vn` → `Việt Nam`). `id` is a stable UUID — not the trigger — so a row
/// can be edited inline while its trigger is briefly empty or duplicated.
struct TextShortcut: Codable, Identifiable, Hashable {
    var id = UUID()
    var trigger: String
    var expansion: String
}

/// User preferences for Funput, persisted in `UserDefaults`. The Settings UI and
/// (later) the `IMKInputController` live in the same process, so they share this
/// store directly. `@Observable` drives live SwiftUI updates.
@Observable
final class AppSettings {
    static let shared = AppSettings()

    var inputMethod: InputMethod {
        didSet { defaults.set(inputMethod.rawValue, forKey: Keys.inputMethod) }
    }
    /// Tone-mark placement style (traditional `hòa` vs modern `hoà`). Read live by
    /// `FunputInputController` and pushed to the engine.
    var toneStyle: ToneStyle {
        didSet { defaults.set(toneStyle.rawValue, forKey: Keys.toneStyle) }
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
    /// Spell-check ("Kiểm tra chính tả"): only place a diacritic when it forms a valid
    /// Vietnamese syllable, otherwise keep the modifier key literal. Off by default.
    var spellCheckEnabled: Bool {
        didSet { defaults.set(spellCheckEnabled, forKey: Keys.spellCheckEnabled) }
    }
    /// Auto-capitalize ("Tự động viết hoa"): uppercase the first letter at the start of
    /// a sentence. Off by default.
    var autoCapitalizeEnabled: Bool {
        didSet { defaults.set(autoCapitalizeEnabled, forKey: Keys.autoCapitalizeEnabled) }
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
    /// Apps where Vietnamese input is suppressed (English pass-through). Read live by
    /// `FunputInputController` against the focused client's bundle identifier.
    var excludedApps: [ExcludedApp] {
        didSet { defaults.set(try? JSONEncoder().encode(excludedApps), forKey: Keys.excludedApps) }
    }
    /// Text-expansion shortcuts (gõ tắt). Persisted as JSON and pushed to the engine by
    /// `FunputInputController`, which re-syncs whenever `shortcutsRevision` changes.
    var shortcuts: [TextShortcut] {
        didSet {
            defaults.set(try? JSONEncoder().encode(shortcuts), forKey: Keys.shortcuts)
            shortcutsRevision &+= 1
        }
    }
    /// Bumped on every `shortcuts` mutation so the controller knows when to re-marshal
    /// the table to the engine (instead of doing it on every keystroke). Not persisted.
    @ObservationIgnored private(set) var shortcutsRevision = 0

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
        toneStyle = ToneStyle(rawValue: defaults.integer(forKey: Keys.toneStyle)) ?? .traditional
        vietnameseEnabled = defaults.bool(forKey: Keys.vietnameseEnabled)
        smartEnglishRestore = defaults.bool(forKey: Keys.smartEnglishRestore)
        eagerRestore = defaults.bool(forKey: Keys.eagerRestore)
        spellCheckEnabled = defaults.bool(forKey: Keys.spellCheckEnabled)
        autoCapitalizeEnabled = defaults.bool(forKey: Keys.autoCapitalizeEnabled)
        toggleShortcut = defaults.string(forKey: Keys.toggleShortcut)
            .flatMap(ToggleShortcut.init(rawValue:)) ?? .controlBackslash
        launchAtLogin = defaults.bool(forKey: Keys.launchAtLogin)
        showMenuBarIcon = defaults.bool(forKey: Keys.showMenuBarIcon)
        hasCompletedOnboarding = defaults.bool(forKey: Keys.hasCompletedOnboarding)
        excludedApps = defaults.data(forKey: Keys.excludedApps)
            .flatMap { try? JSONDecoder().decode([ExcludedApp].self, from: $0) } ?? []
        shortcuts = defaults.data(forKey: Keys.shortcuts)
            .flatMap { try? JSONDecoder().decode([TextShortcut].self, from: $0) } ?? []
    }

    // MARK: - Excluded apps

    /// Whether the focused client's app is on the exclusion list.
    func isExcluded(_ bundleId: String?) -> Bool {
        guard let bundleId, !bundleId.isEmpty else { return false }
        return excludedApps.contains { $0.id == bundleId }
    }

    /// Add an app to the exclusion list (no-op if already present).
    func addExcludedApp(_ app: ExcludedApp) {
        guard !excludedApps.contains(where: { $0.id == app.id }) else { return }
        excludedApps.append(app)
    }

    /// Remove an app from the exclusion list by bundle identifier.
    func removeExcludedApp(_ id: String) {
        excludedApps.removeAll { $0.id == id }
    }

    // MARK: - Shortcuts (gõ tắt)

    /// Append an empty shortcut for the user to fill in inline.
    func addShortcut() {
        shortcuts.append(TextShortcut(trigger: "", expansion: ""))
    }

    /// Remove a shortcut by its row id.
    func removeShortcut(_ id: UUID) {
        shortcuts.removeAll { $0.id == id }
    }

    private enum Keys {
        static let inputMethod = "inputMethod"
        static let toneStyle = "toneStyle"
        static let vietnameseEnabled = "vietnameseEnabled"
        static let smartEnglishRestore = "smartEnglishRestore"
        static let eagerRestore = "eagerRestore"
        static let spellCheckEnabled = "spellCheckEnabled"
        static let autoCapitalizeEnabled = "autoCapitalizeEnabled"
        static let toggleShortcut = "toggleShortcut"
        static let launchAtLogin = "launchAtLogin"
        static let showMenuBarIcon = "showMenuBarIcon"
        static let hasCompletedOnboarding = "hasCompletedOnboarding"
        static let excludedApps = "excludedApps"
        static let shortcuts = "shortcuts"
    }
}
