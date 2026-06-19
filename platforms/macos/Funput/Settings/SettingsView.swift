import SwiftUI

/// The Settings window: sidebar navigation + detail pane (Liquid Glass).
struct SettingsView: View {
    @State private var selection: SidebarSection? = .inputMethod

    var body: some View {
        NavigationSplitView {
            List(selection: $selection) {
                ForEach(SidebarSection.allCases) { section in
                    Label(section.title, systemImage: section.systemImage)
                        .tag(section)
                }
            }
            .navigationSplitViewColumnWidth(Theme.sidebarWidth)
        } detail: {
            ScrollView {
                pane
                    .padding(Theme.Spacing.xl)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .navigationTitle(selection?.title ?? "Funput")
        }
        .tint(.accentColor)
    }

    @ViewBuilder private var pane: some View {
        switch selection ?? .inputMethod {
        case .general: GeneralPane()
        case .inputMethod: InputMethodPane()
        case .smart: SmartPane()
        case .keyboard: KeyboardPane()
        case .about: AboutPane()
        }
    }
}

#Preview {
    SettingsView()
        .environment(AppSettings.shared)
        .frame(width: Theme.settingsMinWidth, height: Theme.settingsMinHeight)
}
