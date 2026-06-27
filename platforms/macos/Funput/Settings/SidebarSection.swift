import SwiftUI

enum SidebarSection: String, CaseIterable, Identifiable, Hashable {
    case general
    case inputMethod
    case smart
    case shortcuts
    case keyboard
    case apps
    case about

    var id: String { rawValue }

    var title: String {
        switch self {
        case .general: "Chung"
        case .inputMethod: "Phương thức gõ"
        case .smart: "Thông minh"
        case .shortcuts: "Gõ tắt"
        case .keyboard: "Bàn phím"
        case .apps: "Ứng dụng"
        case .about: "Giới thiệu"
        }
    }

    var systemImage: String {
        switch self {
        case .general: "gearshape"
        case .inputMethod: "keyboard"
        case .smart: "sparkles"
        case .shortcuts: "text.append"
        case .keyboard: "command"
        case .apps: "app.badge"
        case .about: "info.circle"
        }
    }
}
