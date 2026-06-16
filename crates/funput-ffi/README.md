# funput-ffi

Crate **C ABI boundary** — export API ổn định cho platform native (Swift trên macOS, C#/C++ trên Windows) gọi `funput-engine` mà không cần viết Rust ở phía UI.

## Ý nghĩa

Rust engine chạy trong `.dylib` / `.dll` / `.so`. Platform shell (Swift, C#) load library và gọi hàm C — đây là cầu nối giữa **Rust core** và **native hook/inject layer**.

Linux Fcitx5 có thể link `funput-engine` trực tiếp và **không cần** crate này.

## Trách nhiệm

| Làm | Không làm |
|-----|-----------|
| Export `extern "C"` functions | Logic Telex/VNI |
| Chuyển `ImeResult` → struct C (`#[repr(C)]`) | CGEventTap, keyboard hook |
| Quản lý vòng đời init / free result | Settings UI |
| Thread-safe singleton engine (nếu cần) | Inject text vào app |
| `cbindgen` / header generation | Fcitx5 integration |

## API (C ABI — `include/funput.h`)

Handle-based, kết quả trả **theo giá trị** (không cần free), input là **codepoint**
(platform tự map keycode→char):

```c
typedef struct FunputEngine FunputEngine;   // opaque handle

typedef struct {
    uint8_t  action;        // 0=None, 1=Send, 2=Restore
    uint32_t backspace;     // số ký tự xoá trước khi inject
    uint32_t count;         // số codepoint hợp lệ trong chars (<= 64)
    uint32_t chars[64];     // UTF-32 output; chars[0..count] valid
} FunputResult;

FunputEngine* funput_engine_new(void);
void          funput_engine_free(FunputEngine*);
void          funput_set_method(FunputEngine*, uint8_t method);   // 0=Telex, 1=VNI
void          funput_set_enabled(FunputEngine*, bool enabled);
void          funput_clear(FunputEngine*);                        // word boundary
FunputResult  funput_process_char(FunputEngine*, uint32_t codepoint);
```

Mọi hàm **null-safe**. Header sinh bằng `cbindgen` (xem `scripts/gen-header.sh`).

## Luồng trên macOS (ví dụ)

```
CGEventTap callback (Swift)
       ↓ keycode → codepoint (platform map)
funput_process_char(engine, cp)    ← funput-ffi
       ↓
funput-engine
       ↓
FunputResult (by value) → Swift đọc action / backspace / chars[0..count]
       ↓
Inject layer (Backspace / AX-sync)   ← ngoài funput-ffi
```

## Memory ownership

| Bên | Trách nhiệm |
|-----|------------|
| Rust (`funput_engine_new`) | Cấp phát handle |
| Caller (Swift/C#) | Gọi `funput_engine_free()` đúng một lần mỗi handle |
| `funput_process_char` | Trả **by value** — **không** cấp phát, **không** free per-result |

Chỉ cần free **handle** (thường `defer funput_engine_free(e)` trong Swift). Result là POD trên stack → không rò rỉ.

## Cấu trúc module

```
funput-ffi/
├── src/lib.rs            # extern "C" exports + opaque FunputEngine
├── src/types.rs          # #[repr(C)] FunputResult + from_ime()
├── cbindgen.toml
├── scripts/gen-header.sh
└── include/funput.h      # Generated via cbindgen (committed)
```

## Build output

| Platform | Artifact |
|----------|----------|
| macOS | `libfunput_ffi.dylib` + `funput.h` |
| Windows | `funput_ffi.dll` + `.lib` |
| Linux | Không bắt buộc (Fcitx5 link trực tiếp engine) |

Script build trong `platforms/macos/scripts/build-rust.sh` compile crate này với target phù hợp.

## Phụ thuộc

```
funput-ffi → funput-engine → funput-core
```

## Ai gọi crate này?

| Consumer | Ghi chú |
|----------|---------|
| `platforms/macos/Funput/Bridge/` | Swift qua bridging header |
| `platforms/windows/` | P/Invoke hoặc C++ link |
| **Không** | `funput-cli`, Fcitx5 (link engine trực tiếp) |

## Tests

FFI layer nên có test round-trip:

```bash
cargo test -p funput-ffi
```

Kiểm tra: gọi `ime_key` qua C ABI → parse result → `ime_free` không leak.
