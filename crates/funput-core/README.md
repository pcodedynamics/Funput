# funput-core

Crate **thuần logic** gõ tiếng Việt: cho một buffer ký tự Latin + một phím vừa gõ, theo Telex/VNI,
trả về buffer mới. Không hook bàn phím, không I/O, không config file, không biết
macOS/Windows/Linux.

## Crate này làm gì

Trả lời đúng một câu hỏi:

> Buffer hiện tại + phím vừa nhập, theo Telex/VNI, trở thành chuỗi gì?

Đây là nơi **duy nhất** chứa:

- Quy tắc VNI (`1`→sắc, `6`→`â`, `7`→`ơ`/`ư`, `8`→`ă`, `9`→`đ`, …)
- Quy tắc Telex (`s`→sắc, `f`→huyền, `aa`→`â`, `w`→móc/breve, `dd`→`đ`, …)
- Đặt dấu thanh (kiểu cũ/mới) + dấu mũ/móc/trần
- Revert khi gõ đúp phím dấu (`a11` → `a1`)
- Validation cấu trúc âm tiết (onset / vần / coda)
- Bảng Unicode (dấu thanh, vowel shape, nguyên âm)

**Stateless & pure:** không giữ session, không đếm backspace, không tự khôi phục tiếng Anh — đó là
việc của `funput-engine`. Engine so sánh buffer trước/sau để suy ra số backspace cần gửi.

## Public API

Bề mặt nhỏ và ổn định cho `funput-engine`. Đổi breaking cần đồng bộ semver với engine.

| Symbol | Mô tả |
|--------|-------|
| `InputMethod` | `Telex` \| `Vni` |
| `ToneStyle` | `Traditional` (mặc định) \| `Modern` — kiểu đặt dấu (xem mục dưới) |
| `TransformKind` | `Pending` \| `Applied` \| `Reverted` \| `Ignored` |
| `TransformResult` | `{ kind, text }` — trạng thái sau một phím |
| `apply(buffer, key, method, tone_style) -> TransformResult` | Transform một bước |
| `is_valid(buffer) -> bool` | Buffer **có thể** còn là âm tiết VN hợp lệ (lenient) |
| `is_complete_syllable(buffer) -> bool` | Buffer là âm tiết VN **hoàn chỉnh** (strict) |
| `is_definitely_invalid(buffer) -> bool` | Buffer **chắc chắn** không thể thành âm tiết VN |

Ba hàm `is_*` là để `funput-engine` quyết định giữ chữ Việt hay khôi phục về Latin gốc:
`is_complete_syllable` dùng ở ranh giới từ; `is_definitely_invalid` dùng cho eager-restore (đổi lại
ngay khi biết chắc không phải tiếng Việt).

```rust
use funput_core::{apply, InputMethod, ToneStyle, TransformKind};

let r = apply("a", '1', InputMethod::Vni, ToneStyle::Traditional);
assert_eq!(r.kind, TransformKind::Applied);
assert_eq!(r.text, "á");
```

### TransformKind

- `Pending` — phím được append, chưa transform (chờ phím sau). Cũng dùng cho modifier
  **pass-through** trên text không phải tiếng Việt: `text` + `1` → `"text1"`.
- `Applied` — tone / shape / stroke / reposition tạo ra `text` mới.
- `Reverted` — gõ đúp modifier: bỏ dấu rồi chèn lại phím thô (`a11` → `a1`, Telex `ass` → `as`).
- `Ignored` — modifier bị từ chối, `text` không đổi (`ng` + `1`, stroke trên ký tự không phải `d`).

## Kiểu đặt dấu — `ToneStyle`

Hai kiểu **chỉ khác nhau** ở nhóm vần mở khởi đầu bằng bán nguyên âm: `oa`, `oe`, `uy`.

| Vần | `Traditional` (mặc định) | `Modern` |
|-----|--------------------------|----------|
| `oa` | hòa (dấu trên `o`) | hoà (dấu trên `a`) |
| `oe` | khỏe | khoẻ |
| `uy` | thúy | thuý |

Mọi trường hợp khác **giống hệt** ở cả hai kiểu:

- `ia`/`ua` → dấu nguyên âm đầu: `mía`, `múa`.
- Có âm cuối: `hoàn`, `toán`, `huýt`.
- Nguyên âm mang mũ/móc (`â ê ô ơ ư ă`) luôn nhận dấu: `trường`, `việt`, `người`.
- Tam hợp âm: dấu ở nguyên âm giữa: `ngoài`, `xoáy`.

Đặt dấu **không phụ thuộc vị trí gõ phím dấu** — `hoaf` và `hofa` cho cùng kết quả theo kiểu đang
chọn. Tham khảo: [Quy tắc đặt dấu thanh của chữ Quốc ngữ](https://vi.wikipedia.org/wiki/Quy_t%E1%BA%AFc_%C4%91%E1%BA%B7t_d%E1%BA%A5u_thanh_c%E1%BB%A7a_ch%E1%BB%AF_Qu%E1%BB%91c_ng%E1%BB%AF).

## Cấu trúc module

```
src/
├── lib.rs                    # Public API + apply()
├── input_method/             # Phân loại phím → KeyAction. Chỗ DUY NHẤT khác nhau giữa VNI và Telex.
│   ├── vni.rs                # 1–9
│   └── telex.rs              # s/f/r/x/j, aa/dd/ee/oo, w (buffer-aware)
├── composition/              # Pipeline transform, dùng chung VNI + Telex
│   ├── transform.rs          # Orchestrate: revert → validate → apply
│   ├── apply.rs              # Áp stroke / tone / shape lên buffer
│   └── revert.rs             # Bỏ dấu khi gõ đúp phím modifier
├── validation/
│   ├── parse.rs              # Tách onset / nucleus / coda
│   ├── rhyme.rs              # Bảng vần (vần) hợp lệ — lõi quyết định "có là tiếng Việt"
│   └── syllable.rs           # is_valid / is_complete_syllable / is_definitely_invalid + gate modifier
└── unicode/
    ├── marks.rs              # Bảng dấu thanh
    ├── shapes.rs             # Bảng mũ / móc / breve
    ├── vowels.rs             # Nguồn sự thật về nguyên âm
    └── tone_position.rs      # Chọn nguyên âm nhận dấu (Traditional/Modern) + reposition
```

## Pipeline một keystroke

```
key
 └─ input_method::{vni,telex}::classify_key(buffer, key) → KeyAction (Tone / Shape / Stroke / RemoveTone / Normal)
     └─ composition::transform::apply_action
         ├─ thử revert (gõ đúp modifier)     → Reverted
         ├─ validation::syllable (gate)       → Ignored | PassThrough(Pending)
         └─ composition::apply                → Applied
             └─ unicode::{tone_position, marks, shapes}
```

Nhánh `Tone` và `Normal` nhận thêm `ToneStyle` để đặt/reposition dấu. Stage đầu khớp thì dừng.

## Ví dụ hành vi

VNI (mặc định `Traditional`):

| Gõ | Kết quả | Ghi chú |
|----|---------|---------|
| `a1` | `á` | tone |
| `d9` | `đ` | stroke |
| `uo7` | `uơ` | vần mở (`thuo73` → `thuở`) |
| `hoa2` | `hòa` | đặt dấu (Traditional) |
| `vie5t` | `việt` | `ie` → tonal `ê` |
| `ngu7o7i2` | `người` | dấu trên `ơ` |
| `a11` | `a1` | Reverted |
| `text` + `1` | `text1` | Pending (engine sẽ restore) |

Telex:

| Gõ | Kết quả |
|----|---------|
| `as` | `á` |
| `dd` | `đ` |
| `uow` | `uơ` (`thuowr` → `thuở`) |
| `nuocws` | `nước` |
| `hoaf` | `hòa` (Traditional) / `hoà` (Modern) |
| `ass` | `a` (revert tone) |

Khác biệt giữa hai kiểu đặt dấu:

| Gõ (VNI) | `Traditional` | `Modern` |
|----------|---------------|----------|
| `hoa2` | `hòa` | `hoà` |
| `thuy3` | `thủy` | `thuỷ` |
| `khoe3` | `khỏe` | `khoẻ` |
| `mua1` | `múa` | `múa` (không đổi) |

## Tests

```bash
cargo test   -p funput-core
cargo test   -p funput-core vni_full_regression
cargo test   -p funput-core telex_full_regression
cargo clippy -p funput-core -- -D warnings
cargo doc    -p funput-core --no-deps
```

Fixture canonical là **Rust const** (không serde, không JSON loader runtime):
`tests/fixtures/{vni_cases,telex_cases,telex_parity}.rs`. Helper chung ở `tests/support/`.

## Phụ thuộc & ranh giới

- **Không** phụ thuộc crate Funput nào khác; chỉ dùng `std`. **Không** `serde`, `libc`, `std::os`.
- Chỉ `funput-engine` (và tests) gọi crate này. Platform code (macOS IMKit, Windows hook, addon
  Fcitx5) **không** import `funput-core` trực tiếp — nó đi qua `funput-engine`.

| funput-core | funput-engine |
|-------------|---------------|
| `apply()` thuần, một bước | Giữ buffer session; tính backspace từ diff |
| Validate cấu trúc âm tiết | Quyết định auto-restore tiếng Anh ở ranh giới từ |
| Bảng Unicode + quy tắc đặt dấu | Map keycode phần cứng → char |
