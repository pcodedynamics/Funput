import Foundation

/// Vietnamese input method. Raw value matches `funput_core` (`Telex = 0`, `VNI = 1`),
/// so it can be passed straight to `funput_set_method` once the IME is wired up.
enum InputMethod: Int, CaseIterable, Identifiable, Codable {
    case telex = 0
    case vni = 1

    var id: Int { rawValue }

    var displayName: String {
        switch self {
        case .telex: "Telex"
        case .vni: "VNI"
        }
    }

    var blurb: String {
        switch self {
        case .telex: "Dấu bằng chữ cái — aa→â, ow→ơ, as→á, dd→đ"
        case .vni: "Dấu bằng chữ số — a6→â, o7→ơ, a1→á, d9→đ"
        }
    }
}

/// Hotkey that switches between Vietnamese and English (pass-through) typing.
enum ToggleShortcut: String, CaseIterable, Identifiable, Codable {
    case controlBackslash
    case controlSpace
    case rightCommand
    case rightOption

    var id: String { rawValue }

    var keyCaps: [String] {
        switch self {
        case .controlBackslash: ["⌃", "\\"]
        case .controlSpace: ["⌃", "Space"]
        case .rightCommand: ["⌘", "(phải)"]
        case .rightOption: ["⌥", "(phải)"]
        }
    }

    var label: String { keyCaps.joined(separator: " ") }
}
