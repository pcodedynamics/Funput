import AppKit
import InputMethodKit

/// Bridges macOS key events to `funput-engine` via marked (underlined) text.
///
/// Each text client gets its own controller + `FunputComposer`. The composing word
/// is shown as marked text; it commits on a word boundary, Enter/Tab, focus change,
/// or navigation key. Settings (method, Vietnamese on/off) are read live from
/// `AppSettings.shared` — same process, so changes apply to the next keystroke.
@objc(FunputInputController)
final class FunputInputController: IMKInputController {
    private let composer = FunputComposer()
    /// Last `AppSettings.shortcutsRevision` pushed to the engine. `-1` forces a sync on
    /// the first `syncSettings()` so the engine starts with the saved table.
    private var lastSyncedShortcutsRevision = -1

    private enum KeyCode {
        static let backspace: UInt16 = 51
        static let backslash: UInt16 = 42
        static let space: UInt16 = 49
        static let rightCommand: UInt16 = 54
        static let rightOption: UInt16 = 61
    }

    private static let notFound = NSRange(location: NSNotFound, length: 0)

    override init!(server: IMKServer!, delegate: Any!, client inputClient: Any!) {
        super.init(server: server, delegate: delegate, client: inputClient)
        syncSettings()
    }

    /// Also receive flagsChanged so right-Command / right-Option can toggle.
    override func recognizedEvents(_ sender: Any!) -> Int {
        Int(NSEvent.EventTypeMask.keyDown.rawValue | NSEvent.EventTypeMask.flagsChanged.rawValue)
    }

    // MARK: - Event entry point

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event, let client = sender as? IMKTextInput else { return false }

        if event.type == .flagsChanged {
            if matchesModifierToggle(event) { toggleEnabled() }
            return false
        }
        guard event.type == .keyDown else { return false }

        syncSettings()

        if matchesComboToggle(event) {
            toggleEnabled()
            return true
        }

        // English mode: pass everything straight through to the app. The VI/EN state
        // is set per-app on focus change (see `activateServer`) and can be toggled.
        guard AppSettings.shared.vietnameseEnabled else { return false }

        // Keyboard shortcuts (⌘C, ⌃A, …) are not text: end composition and let
        // the app handle them. Control/Command combos carry control characters in
        // `event.characters`, which would otherwise be fed to the composer.
        if !event.modifierFlags.isDisjoint(with: [.command, .control]) {
            commit(into: client)
            return false
        }

        if event.keyCode == KeyCode.backspace {
            guard !composer.buffer().isEmpty else { return false }
            composer.backspace()
            setMarked(composer.buffer(), client)
            return true
        }

        guard let scalar = event.characters?.unicodeScalars.first else {
            commit(into: client) // dead keys with no character end composition
            return false
        }

        // Navigation / function / editing keys carry a character (arrows live in the
        // function-key private-use area, Esc is U+001B) but are not text. End the
        // composition and let the app move the cursor / dismiss / delete forward.
        if isNonTextKey(event, scalar) {
            commit(into: client)
            return false
        }

        if isBoundary(scalar) {
            return commitBoundary(scalar, into: client)
        }

        composer.process(scalar)
        setMarked(composer.buffer(), client)
        return true
    }

    override func commitComposition(_ sender: Any!) {
        if let client = sender as? IMKTextInput { commit(into: client) }
    }

    /// Focus moved into this client. Apply the per-app default: apps on the exclusion
    /// list switch to English, every other app switches to Vietnamese. This updates
    /// `vietnameseEnabled` (so the menu bar reflects it immediately) and the user can
    /// still toggle manually afterwards until focus changes again.
    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        applyPerAppDefault()
        syncSettings()
        // Focus on a field is the start of input: arm so the first letter is capitalized.
        if AppSettings.shared.autoCapitalizeEnabled { composer.armCapitalization() }
    }

    override func deactivateServer(_ sender: Any!) {
        if let client = sender as? IMKTextInput { commit(into: client) }
        super.deactivateServer(sender)
    }

    /// Set VI/EN from the frontmost app's exclusion status. No-op when the list is
    /// empty so users who don't use the feature keep a plain global toggle.
    private func applyPerAppDefault() {
        let settings = AppSettings.shared
        guard !settings.excludedApps.isEmpty else { return }
        let front = NSWorkspace.shared.frontmostApplication?.bundleIdentifier
        let vietnamese = !settings.isExcluded(front)
        guard settings.vietnameseEnabled != vietnamese else { return }
        settings.vietnameseEnabled = vietnamese
        composer.setEnabled(vietnamese)
    }

    // MARK: - Commit

    private func commit(into client: IMKTextInput) {
        let text = composer.buffer()
        if !text.isEmpty {
            client.insertText(text, replacementRange: Self.notFound)
        }
        composer.clear()
    }

    /// Boundary key (space / punctuation / Enter / Tab) while composing.
    private func commitBoundary(_ scalar: Unicode.Scalar, into client: IMKTextInput) -> Bool {
        let pre = composer.buffer()
        guard !pre.isEmpty else {
            // Not composing: the app inserts the key itself. Still feed it to the engine
            // so auto-capitalize can track sentence context across the gap — e.g. the
            // space in "End. Next" arrives here with an empty buffer.
            if AppSettings.shared.autoCapitalizeEnabled { _ = composer.process(scalar) }
            return false
        }

        // The engine decides English-restore, then clears the session. On restore it
        // returns Action::Send with output = rawKeys + boundaryChar; otherwise keep `pre`.
        let result = composer.process(scalar)
        let word: String
        if result.action == ACTION_SEND {
            word = String(FunputComposer.output(of: result).dropLast()) // drop boundary char
        } else {
            word = pre
        }

        if scalar == "\n" || scalar == "\r" || scalar == "\t" {
            client.insertText(word, replacementRange: Self.notFound)
            return false // let the app process Enter / Tab itself
        }
        client.insertText(word + String(scalar), replacementRange: Self.notFound)
        return true
    }

    // MARK: - Toggle Vietnamese / English

    private func toggleEnabled() {
        let settings = AppSettings.shared
        settings.vietnameseEnabled.toggle()
        composer.setEnabled(settings.vietnameseEnabled)
    }

    private func matchesComboToggle(_ event: NSEvent) -> Bool {
        switch AppSettings.shared.toggleShortcut {
        case .controlBackslash:
            return event.modifierFlags.contains(.control) && event.keyCode == KeyCode.backslash
        case .controlSpace:
            return event.modifierFlags.contains(.control) && event.keyCode == KeyCode.space
        case .rightCommand, .rightOption:
            return false
        }
    }

    private func matchesModifierToggle(_ event: NSEvent) -> Bool {
        switch AppSettings.shared.toggleShortcut {
        case .rightCommand:
            return event.keyCode == KeyCode.rightCommand && event.modifierFlags.contains(.command)
        case .rightOption:
            return event.keyCode == KeyCode.rightOption && event.modifierFlags.contains(.option)
        case .controlBackslash, .controlSpace:
            return false
        }
    }

    // MARK: - Helpers

    private func syncSettings() {
        let settings = AppSettings.shared
        composer.setMethod(settings.inputMethod)
        composer.setToneStyle(settings.toneStyle)
        composer.setEnabled(settings.vietnameseEnabled)
        composer.setSmartRestore(settings.smartEnglishRestore)
        composer.setEagerRestore(settings.eagerRestore)
        composer.setSpellCheck(settings.spellCheckEnabled)
        composer.setAutoCapitalize(settings.autoCapitalizeEnabled)

        // Re-marshal the gõ tắt table only when it actually changed (cheap on the
        // common keystroke path where nothing changed).
        if lastSyncedShortcutsRevision != settings.shortcutsRevision {
            composer.clearShortcuts()
            for shortcut in settings.shortcuts where !shortcut.trigger.isEmpty {
                composer.addShortcut(trigger: shortcut.trigger, expansion: shortcut.expansion)
            }
            lastSyncedShortcutsRevision = settings.shortcutsRevision
        }
    }

    private func setMarked(_ text: String, _ client: IMKTextInput) {
        // Thin underline, no highlight: pass an attributed string so apps don't
        // fall back to the "selected text" style for plain marked strings.
        let marked = NSAttributedString(string: text, attributes: [
            .underlineStyle: NSUnderlineStyle.single.rawValue,
            .underlineColor: NSColor.labelColor,
        ])
        client.setMarkedText(
            marked,
            selectionRange: NSRange(location: text.utf16.count, length: 0),
            replacementRange: Self.notFound
        )
    }

    /// Navigation, function, and editing keys (arrows, Home/End, Page Up/Down,
    /// forward-delete, F-keys, Esc) — not text, so composition ends and the app
    /// handles them. Arrow/Home/End/forward-delete set the `.function` flag and live
    /// in the function-key private-use area; Esc and stray controls are < U+0020.
    private func isNonTextKey(_ event: NSEvent, _ s: Unicode.Scalar) -> Bool {
        if event.modifierFlags.contains(.function) { return true }
        let v = s.value
        if (0xF700...0xF8FF).contains(v) { return true }
        if v < 0x20 && s != "\t" && s != "\n" && s != "\r" { return true }
        return false
    }

    /// Word boundary — whitespace or ASCII punctuation. Mirrors `funput_core`'s rule
    /// (digits excluded: VNI uses them as tone modifiers).
    private func isBoundary(_ s: Unicode.Scalar) -> Bool {
        guard s.isASCII else { return false }
        if s == " " || s == "\t" || s == "\n" || s == "\r" { return true }
        let v = s.value
        return (0x21...0x2F).contains(v) || (0x3A...0x40).contains(v)
            || (0x5B...0x60).contains(v) || (0x7B...0x7E).contains(v)
    }
}
