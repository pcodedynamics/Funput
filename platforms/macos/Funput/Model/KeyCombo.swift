import AppKit
import SwiftUI

/// A user-recorded keyboard shortcut: a key plus its ⌃/⌥/⌘/⇧ modifiers.
///
/// Matching is by `keyCode` + the exact masked modifier set (layout-independent).
/// `label` is the display string captured when recorded, so we don't need a
/// keyCode→name table for every layout.
struct KeyCombo: Codable, Equatable {
    let keyCode: UInt16
    /// Raw value of the masked `NSEvent.ModifierFlags` (⌃⌥⌘⇧ only).
    let modifiers: UInt
    let label: String

    /// The modifier keys that make a combo a real shortcut (Shift alone doesn't —
    /// it's part of normal typing).
    private static let commandModifiers: NSEvent.ModifierFlags = [.control, .option, .command]
    private static let allModifiers: NSEvent.ModifierFlags = [.control, .option, .command, .shift]

    /// Default VI/EN toggle — `⌃\` (backslash key, keyCode 42).
    static let defaultToggle = KeyCombo(
        keyCode: 42,
        modifiers: NSEvent.ModifierFlags.control.rawValue,
        label: "\\"
    )

    /// Build a combo from a key-down event, or `nil` when it isn't a usable shortcut
    /// (no ⌃/⌥/⌘ held — a bare key would fire while typing).
    static func from(_ event: NSEvent) -> KeyCombo? {
        let mods = event.modifierFlags.intersection(allModifiers)
        guard !mods.intersection(commandModifiers).isEmpty else { return nil }
        return KeyCombo(keyCode: event.keyCode, modifiers: mods.rawValue, label: label(for: event))
    }

    /// True when `event` is exactly this combo.
    func matches(_ event: NSEvent) -> Bool {
        event.keyCode == keyCode
            && event.modifierFlags.intersection(Self.allModifiers).rawValue == modifiers
    }

    /// Keycaps for display, e.g. `["⌃", "⇧", "Z"]`.
    var keyCaps: [String] {
        Self.modifierSymbols(NSEvent.ModifierFlags(rawValue: modifiers)) + [label]
    }

    /// The modifier symbols held in `flags`, in display order (⌃⌥⇧⌘).
    static func modifierSymbols(_ flags: NSEvent.ModifierFlags) -> [String] {
        var caps: [String] = []
        if flags.contains(.control) { caps.append("⌃") }
        if flags.contains(.option) { caps.append("⌥") }
        if flags.contains(.shift) { caps.append("⇧") }
        if flags.contains(.command) { caps.append("⌘") }
        return caps
    }

    /// Human-readable label for the pressed key, preferring the produced character.
    private static func label(for event: NSEvent) -> String {
        if let chars = event.charactersIgnoringModifiers,
           let first = chars.first,
           first.isLetter || first.isNumber || first.isPunctuation || first.isSymbol {
            return chars.uppercased()
        }
        return specialKeyNames[event.keyCode] ?? "•"
    }

    /// Display names for common non-printing keys (fallback when there's no character).
    private static let specialKeyNames: [UInt16: String] = [
        49: "Space", 36: "↩", 48: "⇥", 51: "⌫", 53: "⎋",
        123: "←", 124: "→", 125: "↓", 126: "↑",
    ]
}

extension KeyCombo {
    /// The SwiftUI shortcut for menu annotations (and app-level handling while a
    /// Funput window is focused). `nil` when the key can't be represented.
    var keyboardShortcut: KeyboardShortcut? {
        keyEquivalent.map { KeyboardShortcut($0, modifiers: eventModifiers) }
    }

    private var eventModifiers: EventModifiers {
        let flags = NSEvent.ModifierFlags(rawValue: modifiers)
        var mods: EventModifiers = []
        if flags.contains(.control) { mods.insert(.control) }
        if flags.contains(.option) { mods.insert(.option) }
        if flags.contains(.shift) { mods.insert(.shift) }
        if flags.contains(.command) { mods.insert(.command) }
        return mods
    }

    private var keyEquivalent: KeyEquivalent? {
        if let special = Self.specialKeyEquivalents[keyCode] { return special }
        // Printable keys: `label` is the character; lowercase letters so Shift is
        // expressed via the modifiers, not the case.
        guard let ch = label.first, ch != "•" else { return nil }
        return KeyEquivalent(Character(ch.lowercased()))
    }

    private static let specialKeyEquivalents: [UInt16: KeyEquivalent] = [
        49: .space, 36: .return, 48: .tab, 51: .delete, 53: .escape,
        123: .leftArrow, 124: .rightArrow, 125: .downArrow, 126: .upArrow,
    ]
}
