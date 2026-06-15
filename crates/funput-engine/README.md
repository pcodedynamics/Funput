# funput-engine

Crate **điều phối** — nhận sự kiện phím theo thời gian, giữ buffer, gọi `funput-core`, trả kết quả cho platform inject.

**API FROZEN (Phase E4)** — public surface: `Engine`, `Action`, `ImeResult`.

## Ý nghĩa

`funput-core` trả lời “chuỗi này transform thành gì”.  
`funput-engine` trả lời **“sau key này, platform cần làm gì?”**

Đây là **single source of truth** cho trạng thái gõ: buffer đang composition, kiểu gõ đang chọn, bật/tắt, ranh giới từ.

## Trách nhiệm

| Làm | Không làm |
|-----|-----------|
| Session / buffer theo input context | Hook keyboard (CGEventTap, …) |
| Gọi `funput-core` khi có key mới | Inject Backspace / Unicode vào app |
| Tính `backspace` count (buffer cũ vs mới) | UI settings, menu bar |
| Trả `ImeResult` cho platform | Logic Telex/VNI chi tiết |
| Word boundary + English restore | C ABI export (thuộc `funput-ffi`) |
| Bật/tắt engine, đổi Telex/VNI | Shortcut / gõ tắt (phase 2) |

## `ImeResult` — contract với platform

Struct trung tâm mà mọi platform shell consume:

```rust
pub enum Action {
    None,    // Pass key through — không transform
    Send,    // Transform — platform phải inject
    Restore, // Hoàn nguyên buffer (ví dụ ESC, E5+)
}

pub struct ImeResult {
    pub action: Action,
    pub backspace: usize,    // Số ký tự cần xóa trong app
    pub output: String,      // Chuỗi inject sau khi xóa
}
```

`ImeResult` là kiểu Rust-native. `funput-ffi` mới marshal sang struct `#[repr(C)]`
(`backspace: u8`, `chars: [u32; 32]`, `count: u8`) ở biên FFI — giới hạn 32 ký tự /
`u8` và chính sách tràn nằm ở đó, không ở engine.

Platform đọc `ImeResult` rồi quyết định **cách inject** (Backspace, Selection, AX-sync) — logic inject **không** nằm trong crate này.

## Luồng xử lý một phím

```
1. Platform gọi engine.process_char(key)
2. Engine cập nhật buffer / keys
3. Engine gọi funput-core transform (hoặc boundary restore)
4. Engine so sánh buffer trước / sau → tính backspace + output
5. Trả ImeResult
```

### Ví dụ: Telex `a` → `s` → `á`

| Bước | Key | Action | Backspace | Output |
|------|-----|--------|-----------|--------|
| 1 | `a` | `None` | 0 | — (chờ thêm key) |
| 2 | `s` | `Send` | 1 | `á` |

Platform nhận bước 2: xóa 1 ký tự, inject `á`, nuốt key `s`.

## Cấu trúc module (E4)

```
funput-engine/src/
├── lib.rs                # Engine, API FROZEN, re-exports
├── result.rs             # Action, ImeResult
├── session.rs            # enabled, method, buffer, keys
├── boundary.rs           # word boundary + English restore
├── pipeline.rs           # TransformKind → ImeResult
└── diff.rs               # buffer diff → backspace + output

tests/
├── support/mod.rs        # type_keys_with_results, app_text, …
├── fixtures/step_cases.rs
├── engine_fixtures.rs    # engine_full_regression
├── telex_steps.rs
├── vni_steps.rs
├── word_boundary.rs
└── english_restore.rs
```

## Phụ thuộc

```
funput-engine → funput-core
```

## Ai gọi crate này?

| Consumer | Cách gọi |
|----------|----------|
| `funput-ffi` | Wrap API C cho Swift / native |
| `funput-cli` | Test trực tiếp từ terminal |
| `platforms/linux/fcitx5-funput` | Link Rust trực tiếp (không qua FFI) |

Platform macOS/Windows **không** import trực tiếp — đi qua `funput-ffi`.

## Hiện thực

Xem [IMPLEMENTATION.md](./IMPLEMENTATION.md) — roadmap E0–E4 hoàn tất; tiếp theo `funput-cli`.

## Tests

```bash
cargo test -p funput-engine
cargo clippy -p funput-engine -- -D warnings
cargo doc -p funput-engine --no-deps
```

**E4:** Fixture regression (`engine_full_regression`) — buffer parity Telex/VNI với core,
step vectors (`as`, `dd`, `card ` restore), app-text multi-word + English restore.
