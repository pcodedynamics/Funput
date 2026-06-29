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

/// Tone-mark placement style. Raw value matches `funput_core`
/// (`Traditional = 0`, `Modern = 1`), so it passes straight to `funput_set_tone_style`.
enum ToneStyle: Int, CaseIterable, Identifiable, Codable {
    case traditional = 0
    case modern = 1

    var id: Int { rawValue }

    var displayName: String {
        switch self {
        case .traditional: "Truyền thống"
        case .modern: "Hiện đại"
        }
    }

    var blurb: String {
        switch self {
        case .traditional: "Dấu kiểu cũ — hòa, khỏe, thúy"
        case .modern: "Dấu kiểu mới — hoà, khoẻ, thuý"
        }
    }
}
