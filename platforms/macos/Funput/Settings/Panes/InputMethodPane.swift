import SwiftUI

struct InputMethodPane: View {
    @Environment(AppSettings.self) private var settings
    @State private var demoText = ""

    var body: some View {
        @Bindable var settings = settings

        VStack(alignment: .leading, spacing: Theme.Spacing.lg) {
            GlassCard {
                VStack(alignment: .leading, spacing: Theme.Spacing.md) {
                    Picker("Phương thức", selection: $settings.inputMethod) {
                        ForEach(InputMethod.allCases) { method in
                            Text(method.displayName).tag(method)
                        }
                    }
                    .pickerStyle(.segmented)
                    .labelsHidden()

                    Text(settings.inputMethod.blurb)
                        .font(.callout)
                        .foregroundStyle(.secondary)
                }
            }

            GlassCard {
                VStack(alignment: .leading, spacing: Theme.Spacing.sm) {
                    SectionHeader(title: "Gõ thử")
                    TextField("Gõ ở đây…", text: $demoText, axis: .vertical)
                        .textFieldStyle(.plain)
                        .font(.title3)
                        .lineLimit(3, reservesSpace: true)
                    Text("Bản xem trước giao diện — gõ tiếng Việt thật sau khi bật Funput trong app.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
        }
    }
}

#Preview {
    InputMethodPane()
        .environment(AppSettings.shared)
        .padding(Theme.Spacing.xl)
        .frame(width: 520)
}
