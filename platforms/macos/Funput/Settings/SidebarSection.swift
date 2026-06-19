import SwiftUI

enum SidebarSection: String, CaseIterable, Identifiable, Hashable {
    case general
    case inputMethod
    case smart
    case keyboard
    case about

    var id: String { rawValue }

    var title: String {
        switch self {
        case .general: "Chung"
        case .inputMethod: "Phương thức gõ"
        case .smart: "Thông minh"
        case .keyboard: "Bàn phím"
        case .about: "Giới thiệu"
        }
    }

    var systemImage: String {
        switch self {
        case .general: "gearshape"
        case .inputMethod: "keyboard"
        case .smart: "sparkles"
        case .keyboard: "command"
        case .about: "info.circle"
        }
    }
}
