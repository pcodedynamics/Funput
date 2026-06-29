import AppKit
import SwiftUI

/// Assigns a keyboard shortcut: shows the current keys, with an explicit "Đổi"
/// button that opens a guided popover ("Nhấn tổ hợp phím…") with live feedback,
/// validation, and Esc-to-cancel. Binds to an optional `KeyCombo` (`nil` = off);
/// requires a ⌃/⌥/⌘ modifier.
struct ShortcutRecorder: View {
    @Binding var combo: KeyCombo?
    /// Whether the shortcut can be cleared (off). Disable for shortcuts that must
    /// always have a value, e.g. the VI/EN toggle.
    var allowOff = true

    @State private var recording = false
    @State private var held: NSEvent.ModifierFlags = []
    @State private var invalid = false
    @State private var monitor: Any?

    var body: some View {
        HStack(spacing: Theme.Spacing.sm) {
            if let combo {
                ShortcutCaps(caps: combo.keyCaps)
            } else {
                Text("Chưa đặt").font(.callout).foregroundStyle(.secondary)
            }

            Button { recording = true } label: {
                Label(combo == nil ? "Đặt phím" : "Đổi", systemImage: "pencil")
                    .font(.callout.weight(.medium))
                    .foregroundStyle(.primary)
                    .padding(.horizontal, Theme.Spacing.md)
                    .padding(.vertical, Theme.Spacing.xs + 2)
                    .glassEffect(.regular.interactive(), in: .capsule)
                    .contentShape(.capsule)
            }
            .buttonStyle(.plain)
            .popover(isPresented: $recording, arrowEdge: .bottom) {
                recordingPopover
                    .onAppear(perform: startRecording)
                    .onDisappear(perform: stopRecording)
            }

            if combo != nil, allowOff {
                Button { combo = nil } label: {
                    Image(systemName: "xmark.circle.fill")
                        .font(.system(size: 15))
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
                .help("Tắt phím tắt")
            }
        }
    }

    /// The guided capture popover: a prompt, a live preview of the keys held so far,
    /// and a hint that turns into an error when the combo lacks a ⌃/⌥/⌘ modifier.
    private var recordingPopover: some View {
        VStack(spacing: Theme.Spacing.md) {
            Text("Nhấn tổ hợp phím mới")
                .font(.headline)

            ShortcutCaps(caps: previewCaps)
                .frame(minHeight: 30)
                .animation(.easeOut(duration: 0.1), value: held)

            Text(invalid ? "Cần kèm ⌃ / ⌥ / ⌘" : "Giữ ⌃ / ⌥ / ⌘ rồi nhấn phím · ⎋ để huỷ")
                .font(.caption)
                .foregroundStyle(invalid ? Color.red : .secondary)
        }
        .padding(Theme.Spacing.lg)
        .frame(width: 260)
    }

    /// Modifier symbols held so far, plus a placeholder for the key still to come.
    private var previewCaps: [String] {
        KeyCombo.modifierSymbols(held) + ["?"]
    }

    // MARK: - Capture

    private func startRecording() {
        held = []
        invalid = false
        monitor = NSEvent.addLocalMonitorForEvents(matching: [.keyDown, .flagsChanged]) { event in
            if event.type == .flagsChanged {
                held = event.modifierFlags.intersection([.control, .option, .command, .shift])
                invalid = false
                return nil
            }
            if event.keyCode == 53 { // Esc cancels
                recording = false
            } else if let captured = KeyCombo.from(event) {
                combo = captured
                recording = false
            } else {
                invalid = true // a key without a ⌃/⌥/⌘ modifier
            }
            return nil
        }
    }

    private func stopRecording() {
        if let monitor { NSEvent.removeMonitor(monitor) }
        monitor = nil
    }
}
