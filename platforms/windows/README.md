# Funput cho Windows

Bộ gõ tiếng Việt cho Windows theo mô hình **keyboard hook + SendInput** (như UniKey/EVKey):
nghe phím toàn cục → đẩy vào `funput-engine` → xoá lùi + chèn lại text đã soạn. App chạy nền ở
**tray icon**; cửa sổ **Cài đặt / Onboarding** là **UI native Slint** (style Fluent, nền Mica),
không còn WebView2/Node. Đóng gói thành **một file `.exe` portable**.

> Crate này chỉ build trên **Windows** (dùng `slint` + crate `windows`). Một bảng `[workspace]`
> rỗng **tách** nó khỏi workspace ở repo gốc nên `cargo test --workspace` trên macOS/Linux vẫn
> xanh. Build từ trong thư mục này.

## Kiến trúc

```
Funput.exe (Rust, Slint) — khởi động ẩn, tray-only
├─ hook.rs        WH_KEYBOARD_LL + WinEvent + tray, trên một thread message-loop
├─ keymap.rs      vkCode + modifier → funput_desktop::KeyEvent (ToUnicodeEx)
├─ inject.rs      InjectPlan → SendInput (VK_BACK ×n, KEYEVENTF_UNICODE), gắn INJECT_TAG
├─ shell.rs       state toàn cục: Engine + settings + per-app override (mutex)
├─ tray.rs        tray-icon + muda: VI/EN, VNI/Telex, Cài đặt…, Hướng dẫn, Thoát
├─ windows_ui.rs  mở cửa sổ Slint (Settings/Onboarding) + áp Mica theo sáng/tối hệ thống
├─ commands.rs    glue: persist settings + autostart (auto-launch) + mở link (open)
└─ ui/*.slint     SettingsWindow + OnboardingWindow (style Fluent)
```

Logic thuần (quyết định inject + phân loại phím) nằm ở crate dùng-chung
[`funput-desktop`](../../crates/funput-desktop) — **có unit test chạy trên mọi nền**. Phần ở đây
chỉ là glue Windows + UI.

- **Tray ↔ event loop:** tray sống trên thread hook (đã có message loop); thao tác mở cửa sổ được
  chuyển sang thread Slint bằng `slint::invoke_from_event_loop`.
- **Chống đệ quy:** mọi event do `SendInput` tạo ra mang `dwExtraInfo = INJECT_TAG`; hook thấy tag
  thì bỏ qua.

## Build

Yêu cầu: Rust ≥ 1.92 (Slint 1.17), toolchain `x86_64-pc-windows-msvc`. Không cần Node/pnpm,
không cần WebView2.

```powershell
cd platforms\windows
cargo build --release          # → target\release\funput.exe (portable, một file)
```

Chạy: double-click `funput.exe` → icon "FU" hiện ở khay hệ thống, gõ được ngay ở mọi app. Lần đầu
mở **Onboarding**; tray có **"Cài đặt…"** (Settings) và **"Hướng dẫn"**. Cài đặt lưu ở
`%APPDATA%\Funput\settings.json` (giữ nguyên schema bản Tauri cũ — nâng cấp không mất cấu hình).

## Cửa sổ Settings / Onboarding

- UI **native Slint** (`ui/app.slint` + `ui/theme.slint`), style **Fluent** chọn trong `build.rs`.
- Nền cửa sổ **Mica** (`window-vibrancy`), tự đổi **sáng/tối** theo cài đặt Windows; widget theo
  `Palette` của Fluent nên cũng đổi theo.
- Mở **on-demand** từ tray, tạo lười và giữ lại để mở lại nhanh.
- Smart/eager restore điều khiển **engine thật**; mọi thay đổi persist ngay và áp vào engine.

## Đóng gói (installer)

Hiện build ra **`funput.exe` portable** (icon + manifest DPI/asInvoker nhúng qua `winresource`).
Bản Tauri trước lo luôn bundler; nếu cần MSI/NSIS, thêm bước riêng (`cargo-wix` hoặc script NSIS)
— **chưa** nằm trong crate này.

## Hạn chế đã biết

- **Không gõ được vào app chạy quyền Admin** (trừ khi `funput.exe` cũng chạy Admin) — bản chất của
  hook không-elevated.
- `ToUnicodeEx` trong LL hook có thể ảnh hưởng trạng thái dead-key ở bố cục bàn phím đặc biệt — ổn
  với layout US/Vietnamese thông dụng.
- Chưa ký Authenticode (có thể bị SmartScreen cảnh báo lần đầu).
- Emoji trong UI hiển thị tuỳ renderer (FemtoVG) — có thể thay bằng icon nếu cần.

## Verify (trên Windows)

Notepad/WordPad/trình duyệt:
- VNI `xin chaof` → `xin chào`; Telex `tieesng vieejt` → `tiếng việt`.
- Backspace sửa dấu: `Phuas` → ⌫ → `s` → `Phú`.
- English-restore: `text ` → `text `.
- Tray: left-click bật/tắt "Tiếng Việt" (icon đổi màu↔mono); right-click đổi VNI/Telex có hiệu lực
  từ chữ kế; "Thoát" để thoát.
- `Ctrl+` ``` bật/tắt nhanh (đổi được trong Settings).
- Settings: đổi kiểu gõ / phím tắt / smart-eager có hiệu lực ngay; **giữ qua restart**.
- Đổi Windows sang Dark/Light → cửa sổ đổi nền Mica + màu chữ theo.
- Bật "Khởi động cùng Windows" → có registry `HKCU\…\Run`.
