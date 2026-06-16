# funput-ffi — Tài liệu hiện thực

Lớp **C ABI boundary** để shell **non-Rust** (Swift IMKit, C# Windows, C terminal
interposer, Fcitx5 C++) gọi `funput-engine`. Consumer Rust **không** dùng crate này —
link `funput-engine` trực tiếp.

**Tiền đề:** `funput-engine` E4 (API frozen, `ImeResult { action, backspace: usize, output: String }`).

---

## Ranh giới

| `funput-engine` | `funput-ffi` |
|-----------------|--------------|
| `Engine` + `ImeResult` (Rust-native) | `extern "C"` + `#[repr(C)] FunputResult` |
| `process_char(char)` | `funput_process_char(handle, u32 codepoint)` |
| Không platform | Marshal Rust ↔ C, null-safety |

Không logic Telex/VNI, không hook/inject — chỉ chuyển đổi tại biên.

---

## C API (`include/funput.h`, sinh bằng cbindgen)

```c
typedef struct FunputEngine FunputEngine;   // opaque handle

typedef struct {
    uint8_t  action;        // 0=None, 1=Send, 2=Restore
    uint32_t backspace;
    uint32_t count;         // số codepoint hợp lệ trong chars (<= 64)
    uint32_t chars[64];     // UTF-32; chars[0..count] valid
} FunputResult;

FunputEngine* funput_engine_new(void);
void          funput_engine_free(FunputEngine*);
void          funput_set_method(FunputEngine*, uint8_t method);   // 0=Telex,1=VNI
void          funput_set_enabled(FunputEngine*, bool enabled);
void          funput_clear(FunputEngine*);
FunputResult  funput_process_char(FunputEngine*, uint32_t codepoint);
```

- **Handle-based**, **by-value result** (không `ime_free`), **codepoint** input.
- Mọi hàm **null-safe**: handle null / codepoint không hợp lệ (vd surrogate) → result `None`.
- Caller áp result: `action==None` → nhận phím; ngược lại xoá `backspace` ký tự rồi inject `chars[0..count]`.

---

## Marshalling (`src/types.rs`)

`FunputResult::from_ime(&ImeResult)`:
- `Action::{None,Send,Restore}` → `0/1/2`.
- `output.chars()` → `chars[..count]` (cắt ở `CHARS_CAP = 64`), `backspace as u32`.
- Input: `char::from_u32(codepoint)`; `None` → result rỗng.

---

## Cấu trúc

```
crates/funput-ffi/
├── Cargo.toml          # crate-type = ["cdylib","staticlib","rlib"]
├── cbindgen.toml       # config sinh header
├── include/funput.h    # GENERATED — commit; regen bằng scripts/gen-header.sh
├── scripts/gen-header.sh
├── src/
│   ├── lib.rs          # extern "C" + opaque FunputEngine + null-safety
│   └── types.rs        # #[repr(C)] FunputResult + from_ime()
└── tests/round_trip.rs # gọi extern "C" như C caller
```

`FunputEngine` là newtype `{ inner: Engine }` → cbindgen xuất opaque `typedef struct FunputEngine FunputEngine;` mà không cần parse crate dep.

---

## Edition 2024 lưu ý
- `#[unsafe(no_mangle)]` (không phải `#[no_mangle]`).
- Thân `unsafe fn` cần `unsafe { }` tường minh quanh `Box::from_raw` / `ptr.as_mut()`.

---

## Header generation (build nhẹ)
- `cbindgen` **không** nằm trong build thường. Cài: `cargo install cbindgen`.
- Regen: `bash scripts/gen-header.sh` → ghi `include/funput.h` (đã commit).

---

## Tests
- **Unit (`types.rs`)**: `from_ime` cho None/Send/Restore; `"á"`→count=1,chars[0]=U+00E1; truncate > 64.
- **Round-trip (`tests/round_trip.rs`)**: gọi `extern "C"` mô phỏng C caller — Telex/VNI/English-restore + null-safety + surrogate.
- **Smoke C thật** (manual, không cần cho CI):
  ```bash
  cc demo.c -Icrates/funput-ffi/include -Ltarget/debug -lfunput_ffi -o demo
  ```
  (đã verify: header compile + link staticlib + chạy OK.)
- **Nice-to-have:** `cargo +nightly miri test -p funput-ffi` soi UB.

---

## Phase

| Phase | Nội dung | Trạng thái |
|-------|----------|------------|
| F0 | Setup crate-type + workspace member | ✅ |
| F1 | `types.rs` FunputResult + from_ime | ✅ |
| F2 | extern fns + null-safety | ✅ |
| F3 | round-trip tests | ✅ |
| F4 | cbindgen header + script | ✅ |
| F5 | doc + README + clippy/doc sạch | ✅ |

---

## Verification

```bash
cargo test -p funput-ffi
cargo clippy -p funput-ffi --all-targets
cargo build -p funput-ffi && ls target/debug/libfunput_ffi.*   # .a .dylib .rlib
bash crates/funput-ffi/scripts/gen-header.sh
```

---

## Khác README cũ (có chủ đích)
codepoint (không keycode) · handle (không `ime_init` singleton) · by-value (không `ime_free`) · `chars[64]` (không 32) · prefix `funput_*`. Tất cả vì khớp engine frozen + nhẹ/nhanh/ổn định.

## Ngoài phạm vi
Hook/inject, terminal interposer (PTY/ConPTY) — tầng consumer dùng FFI này hoặc link engine trực tiếp (nếu Rust).
