# funput-engine

Crate **điều phối có trạng thái**: nhận từng phím theo thời gian, giữ buffer đang soạn, gọi
`funput-core` rồi trả về một `ImeResult` cho biết **platform cần làm gì** (xoá mấy ký tự, chèn chuỗi
gì). Không hook bàn phím, không inject, không C ABI, không UI.

## Crate này làm gì

`funput-core` trả lời "chuỗi này transform thành gì". `funput-engine` trả lời:

> Sau phím vừa nhập, **platform cần làm gì** — pass phím, nuốt phím, xoá bao nhiêu ký tự, chèn gì?

Đây là **single source of truth** cho trạng thái gõ: buffer composition, kiểu gõ (Telex/VNI), kiểu
đặt dấu, bật/tắt, ranh giới từ, và khôi phục tiếng Anh.

## `ImeResult` — contract với platform

```rust
pub enum Action {
    None,    // Pass phím qua app — không transform
    Send,    // Transform — xoá `backspace` ký tự rồi chèn `output`
    Restore, // Hoàn nguyên về Latin gốc (ESC, …)
}

pub struct ImeResult {
    pub action: Action,
    pub backspace: usize,  // số ký tự cần xoá trong app
    pub output: String,    // chuỗi chèn sau khi xoá
}
```

`ImeResult` là kiểu Rust-native. `funput-ffi` mới marshal nó sang `#[repr(C)]` ở biên FFI
(`backspace: u32`, `count: u32`, `chars: [u32; 64]`) — giới hạn 64 ký tự và chính sách tràn nằm ở
đó, **không** ở engine. Platform đọc `ImeResult` rồi tự quyết **cách** inject (Backspace+Unicode,
preedit/marked text…); logic inject không thuộc crate này.

## Engine API

| Method | Mô tả |
|--------|-------|
| `new()` | Tạo engine (mặc định: bật, Telex, Traditional) |
| `process_char(key: char) -> ImeResult` | Xử lý một Unicode scalar (platform tự map keycode → char) |
| `on_backspace() -> ImeResult` | User bấm Backspace khi đang soạn → đồng bộ buffer |
| `set_enabled(bool)` / `is_enabled()` | Bật/tắt gõ tiếng Việt (English pass-through) |
| `set_method(InputMethod)` / `method()` | Telex ↔ VNI |
| `set_tone_style(ToneStyle)` / `tone_style()` | Kiểu đặt dấu (truyền thống `hòa` / hiện đại `hoà`) |
| `set_smart_restore(bool)` | Tự khôi phục từ không phải tiếng Việt về Latin gốc |
| `set_eager_restore(bool)` | Khôi phục ngay khi biết chắc, không đợi dấu cách |
| `clear()` | Reset buffer + keys (ranh giới từ, đổi focus) |
| `buffer() -> &str` | Text đang soạn — platform dùng để vẽ preedit/marked text |
| `keys() -> &str` | Chuỗi phím thô từ ranh giới từ gần nhất — dùng để khôi phục tiếng Anh |
| `add_shortcut(trigger, expansion)` | Định nghĩa một gõ tắt (`vn` → `Việt Nam`); trigger rỗng bị bỏ qua |
| `remove_shortcut(&str)` | Xoá một gõ tắt theo trigger |
| `clear_shortcuts()` | Xoá toàn bộ bảng gõ tắt (clear + add lại = replace-all khi sync config) |
| `shortcuts() -> &HashMap<String, String>` | Đọc bảng gõ tắt hiện tại |

4 method gõ tắt là thay đổi **additive** lên API E4 (chỉ thêm, không phá vỡ surface cũ).

Re-export: `Action`, `ImeResult` (từ `result.rs`). Đổi breaking cần đồng bộ semver với `funput-ffi`.

## Luồng xử lý một phím

```
platform → engine.process_char(key)
   ├─ ranh giới từ (space / dấu câu)? → boundary: (English restore | giữ) rồi clear()
   └─ ngược lại → funput-core::apply(buffer, key, method, tone_style)
        → diff(buffer cũ, buffer mới) → (backspace, output)
        → cập nhật session.buffer → ImeResult
```

Ví dụ Telex `a` → `s` → `á`:

| Phím | Action | backspace | output |
|------|--------|-----------|--------|
| `a` | `None` | 0 | — (chờ phím sau) |
| `s` | `Send` | 1 | `á` |

Platform ở bước 2: xoá 1 ký tự, chèn `á`, nuốt phím `s`.

## `TransformKind` → `ImeResult`

Quy tắc ánh xạ kết quả của core sang hành động platform:

| `TransformKind` (core) | `action` | `session.buffer` sau | backspace / output |
|------------------------|----------|----------------------|--------------------|
| `Pending` | `None` (pass phím) | `result.text` (= cũ + phím) | 0 / — |
| `Ignored` | `None` (pass phím) | `cũ + phím` (engine tự append) | 0 / — |
| `Applied` | `Send` (nuốt phím) | `result.text` | diff(cũ, mới) |
| `Reverted` | `Send` (nuốt phím) | `result.text` | diff(cũ, mới) |

`Pending`/`Ignored` đều biến phím thành ký tự thường để app ↔ buffer luôn đồng bộ — chỉ khác ở chỗ
core đã append sẵn (`Pending`) hay engine tự ghép (`Ignored`). Khi `enabled = false`, engine bỏ qua
core hoàn toàn (`Action::None`).

### diff (buffer cũ → mới)

```
prefix  = số ký tự chung ở đầu
backspace = cũ.chars().count() - prefix
output    = mới.chars().skip(prefix).collect()
```

`hoa` → `hoà`: prefix 2 (`ho`) → backspace 1, output `à`. `diff` trả `(usize, String)`, **không**
cap — ràng buộc kích thước chỉ áp ở `funput-ffi`.

## Khôi phục tiếng Anh

Khi gõ từ tiếng Anh, core vẫn bỏ dấu (`card` → `cảd`). Tại ranh giới từ, nếu buffer **không** phải
âm tiết tiếng Việt hoàn chỉnh (`funput_core::is_complete_syllable`, strict) và `keys != buffer` →
engine `Send` lại **chuỗi phím thô** (`keys`) + phím ranh giới, rồi `clear()`. `eager_restore` làm
việc này ngay khi buffer trở thành dead-end thay vì đợi dấu cách.

Không từ điển: từ tiếng Anh tình cờ là âm tiết VN hợp lệ (`test` → `tét`) sẽ **không** auto-restore
— đổi lại không bao giờ phá tiếng Việt đang gõ đúng (giống UniKey không từ điển).

## Gõ tắt / Text expansion (macro)

Bảng trigger → expansion do người dùng định nghĩa (`vn` → `Việt Nam`, `kg` → `không`). Tại **ranh
giới từ**, engine khớp **chuỗi phím thô** (`keys`) — **phân biệt hoa/thường** — với bảng gõ tắt:

- Trúng → `Send`: xoá phần đang hiển thị (`backspace = buffer.chars().count()`), chèn `expansion +
  phím ranh giới`, rồi `clear()`. Backspace đếm theo buffer hiển thị nên `as` → `á` (1 ký tự) vẫn xoá
  đúng.
- Gõ tắt **ưu tiên hơn** khôi phục tiếng Anh và hơn việc giữ buffer đã compose.
- Chỉ bung ở ranh giới từ (không bung giữa từ), và chỉ khi IME **đang bật** (`process_char` return
  sớm khi tắt) — gõ tắt là một phần của bộ gõ.

Bảng gõ tắt là **config sống suốt session**: `clear()` (ranh giới từ / đổi focus) không đụng tới nó,
chỉ `clear_shortcuts()` mới xoá. Engine không đọc/ghi file — platform load từ config rồi gọi
`add_shortcut`.

## Cấu trúc module

```
src/
├── lib.rs        # Engine + public API, re-export Action/ImeResult
├── result.rs     # Action, ImeResult
├── session.rs    # state: enabled, method, tone_style, buffer, keys, smart/eager restore
├── pipeline.rs   # process(session, key): TransformKind → ImeResult
├── boundary.rs   # ranh giới từ + quyết định English restore
└── diff.rs       # buffer diff → (backspace, output)
```

## Phụ thuộc & ai gọi

- Phụ thuộc: chỉ `funput-core`. **Không** `serde`, không platform crate.
- **Consumer link engine trực tiếp (Rust):** `funput-ffi`, `funput-cli`, và **Windows shell**
  (`platforms/windows/src-tauri` giữ `Engine` trong process).
- **Qua `funput-ffi` (C ABI):** macOS (Swift IMKit) và addon Fcitx5 trên Linux (C++). Hai nền này
  **không** link engine trực tiếp.

## Tests

```bash
cargo test   -p funput-engine
cargo test   -p funput-engine engine_full_regression
cargo clippy -p funput-engine -- -D warnings
cargo doc    -p funput-engine --no-deps
```

`tests/`: step vectors Telex/VNI (`telex_steps.rs`, `vni_steps.rs`), ranh giới từ
(`word_boundary.rs`), khôi phục tiếng Anh (`english_restore.rs`), fixture regression
(`engine_fixtures.rs` + `fixtures/step_cases.rs`), helper ở `support/`.
