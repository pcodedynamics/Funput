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

    func setEnabled(_ enabled: Bool) {
        funput_set_enabled(handle, enabled)
    }

    func clear() {
        funput_clear(handle)
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
