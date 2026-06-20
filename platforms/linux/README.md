# Funput cho Linux (Fcitx5)

Bộ gõ tiếng Việt cho Linux theo mô hình **input method engine** thật sự, chạy trong **Fcitx5** —
khác Windows (`global hook + SendInput`): Wayland chặn hook + inject phím toàn cục, nên con đường
đúng là một addon Fcitx5. Addon hiển thị từ đang soạn dưới dạng **preedit gạch chân** rồi **commit**
khi tới ranh giới từ — đúng mô hình của bản macOS (IMKit), **không** dùng đường backspace-injection
của Windows.

Tách đôi như các nền khác:

```
platforms/linux/
├─ fcitx5/      Addon C++ (libfunput.so) — gọi funput-ffi (C ABI). PHẦN "gõ".
│  ├─ src/funput_engine.cpp  InputMethodEngineV2: keyEvent → preedit/commit
│  ├─ src/ffi_handle.h       RAII wrapper quanh FunputEngine* (funput.h)
│  ├─ src/settings.cpp       đọc ~/.config/Funput/settings.json (reload theo mtime)
│  └─ data/*.conf.in         metadata addon + input method
├─ src-tauri/   App Settings/Onboarding (Tauri 2 + Svelte). KHÔNG hook, KHÔNG engine.
└─ packaging/   .desktop launcher (CPack gộp vào .deb)
```

> `src-tauri` chỉ build trên **Linux** (kéo theo webkit2gtk) và **bị `exclude`** khỏi workspace gốc,
> nên `cargo test --workspace` trên macOS vẫn xanh. Addon là project **CMake** riêng.

## Hai tiến trình, một file settings

Addon `.so` do daemon `fcitx5` nạp; app Tauri là tiến trình riêng → **không** chia sẻ state trong
process như Windows. Cầu nối là `~/.config/Funput/settings.json` (cùng file `dirs::config_dir()` mà
bản Windows ghi). App Tauri ghi file; addon đọc lại khi **focus-in** (so mtime, rẻ). Bật/tắt VI–EN
bằng hotkey trong addon cũng ghi ngược lại file để UI phản ánh đúng.

Tray + biểu tượng trạng thái dùng luôn của **Fcitx5** (không dựng tray riêng).

## Build từ source

Yêu cầu (Debian/Ubuntu):

```sh
sudo apt install \
  cmake fcitx5 libfcitx5core-dev libfcitx5utils-dev libfcitx5config-dev \
  nlohmann-json3-dev \
  libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
# + Rust (rustup), pnpm.
```

Một lệnh dựng cả gói `.deb`:

```sh
platforms/linux/build.sh
# → cargo build -p funput-ffi  (cdylib)
# → pnpm -C platforms/ui build (Svelte → dist)
# → cargo build (Tauri Settings)
# → cmake + cpack             (addon + .deb)
# Kết quả: platforms/linux/build/funput_*_amd64.deb
```

Cài thử: `sudo apt install ./platforms/linux/build/funput_*_amd64.deb`.

## Cài & bật

1. `fcitx5-configtool` → **+** → thêm **Funput** (nhóm Vietnamese).
2. Nếu Fcitx5 chưa chạy: đăng nhập lại, hoặc đảm bảo `GTK_IM_MODULE=fcitx`, `QT_IM_MODULE=fcitx`,
   `XMODIFIERS=@im=fcitx` (X11) — Wayland đời mới dùng `text-input-v3` nên thường không cần.
3. Bật/tắt tiếng Việt: **`Ctrl + `` `** (mặc định) hoặc icon Fcitx5 ở khay.
4. Đổi Telex/VNI, smart/eager restore: mở **Funput** trong menu ứng dụng (app Tauri Settings).

## Verify (trên Linux, X11 lẫn Wayland)

Mở một app GTK (gedit) và một app Qt:

- VNI `xin chaof` → `xin chào`; Telex `tieesng vieejt` → `tiếng việt` (preedit gạch chân, commit khi
  gõ dấu cách).
- Backspace giữa từ: `Phuas` → ⌫ → `Phú`.
- Smart-restore: `card ` giữ `card` (không thành `cảd`).
- `Ctrl+` ``` bật/tắt — chữ ra Latin thường khi tắt.
- Đổi Telex/VNI trong app Settings → focus lại ô text → có hiệu lực (addon reload từ file).

## Hạn chế đã biết / để sau

- **Chỉ Fcitx5** (chưa IBus). **Chỉ `.deb`** (chưa rpm/AppImage).
- Hotkey `alt_shift` (combo chỉ-modifier) chưa hỗ trợ; dùng `ctrl_backtick`/`ctrl_space`.
- Settings đồng bộ theo **mtime ở lần focus-in kế tiếp** — có thể trễ một nhịp (inotify để sau).
- Cửa sổ Settings dùng nền trong suốt; cần compositor (GNOME/KDE đời mới có sẵn).
