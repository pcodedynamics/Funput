import Foundation

/// Safe Swift wrapper around the `funput-ffi` C engine handle.
///
/// One instance per `IMKInputController` (input is single-threaded on the main
/// run loop). Named `FunputComposer` to avoid colliding with the C `FunputEngine`
/// type imported from the bridging header.
final class FunputComposer {
    private let handle: OpaquePointer

    init() {
        handle = funput_engine_new()
    }

    deinit {
        funput_engine_free(handle)
    }

    func setMethod(_ method: InputMethod) {
        funput_set_method(handle, UInt8(method.rawValue))
    }

    /// Tone-mark placement style (traditional `hòa` vs modern `hoà`).
    func setToneStyle(_ style: ToneStyle) {
        funput_set_tone_style(handle, UInt8(style.rawValue))
    }

    func setEnabled(_ enabled: Bool) {
        funput_set_enabled(handle, enabled)
    }

    /// Auto-restore non-Vietnamese words to their raw Latin keystrokes.
    func setSmartRestore(_ on: Bool) {
        funput_set_smart_restore(handle, on)
    }

    /// Restore the instant a word dead-ends, without waiting for a word boundary.
    func setEagerRestore(_ on: Bool) {
        funput_set_eager_restore(handle, on)
    }

    /// Spell-check ("Kiểm tra chính tả"): only place a diacritic when it forms a
    /// valid Vietnamese syllable, otherwise keep the modifier key literal.
    func setSpellCheck(_ on: Bool) {
        funput_set_spell_check(handle, on)
    }

    /// Auto-capitalize ("Tự động viết hoa"): uppercase the first letter at the start
    /// of a sentence.
    func setAutoCapitalize(_ on: Bool) {
        funput_set_auto_capitalize(handle, on)
    }

    /// Arm capitalization for the next word (call on focus so the first letter typed
    /// in a field is capitalized). A no-op unless auto-capitalize is on.
    func armCapitalization() {
        funput_arm_capitalization(handle)
    }

    func clear() {
        funput_clear(handle)
    }

    /// Remove every text-expansion shortcut (gõ tắt). Pair with `addShortcut` to
    /// replace the whole table when syncing from `AppSettings`.
    func clearShortcuts() {
        funput_clear_shortcuts(handle)
    }

    /// Define a text-expansion shortcut: typing `trigger` then a word boundary injects
    /// `expansion` (`vn` → `Việt Nam`). Both strings cross the C ABI as UTF-32.
    func addShortcut(trigger: String, expansion: String) {
        let t = trigger.unicodeScalars.map(\.value)
        let e = expansion.unicodeScalars.map(\.value)
        funput_add_shortcut(handle, t, UInt(t.count), e, UInt(e.count))
    }

    /// The composed syllable buffer — the text shown as marked (underlined) text.
    func buffer() -> String {
        var out = [UInt32](repeating: 0, count: Int(CHARS_CAP))
        let count = funput_buffer(handle, &out, UInt(out.count))
        return Self.scalars(out, count)
    }

    /// Feed one Unicode scalar; returns the platform instruction.
    @discardableResult
    func process(_ scalar: Unicode.Scalar) -> FunputResult {
        funput_process_char(handle, scalar.value)
    }

    /// Drop the last composed character (in-composition Backspace).
    @discardableResult
    func backspace() -> FunputResult {
        funput_backspace(handle)
    }

    /// Decode a `FunputResult`'s inline `chars` (a C `uint32_t[64]`, imported as a
    /// tuple) into a `String`.
    static func output(of result: FunputResult) -> String {
        var copy = result
        let count = Int(result.count)
        return withUnsafePointer(to: &copy.chars) { tuplePtr in
            tuplePtr.withMemoryRebound(to: UInt32.self, capacity: Int(CHARS_CAP)) { buf in
                var view = String.UnicodeScalarView()
                for i in 0..<count {
                    if let s = Unicode.Scalar(buf[i]) { view.append(s) }
                }
                return String(view)
            }
        }
    }

    private static func scalars(_ codepoints: [UInt32], _ count: UInt) -> String {
        var view = String.UnicodeScalarView()
        for i in 0..<Int(count) {
            if let s = Unicode.Scalar(codepoints[i]) { view.append(s) }
        }
        return String(view)
    }
}
