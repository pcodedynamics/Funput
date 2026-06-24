# Funput cho Linux (Fcitx5 & IBus)

Bộ gõ tiếng Việt cho Linux theo mô hình **input method engine** thật sự, chạy trong **Fcitx5**
hoặc **IBus** — khác Windows (`global hook + SendInput`): Wayland chặn hook + inject phím toàn cục,
nên con đường đúng là một IM engine. Shell hiển thị từ đang soạn dưới dạng **preedit** rồi **commit**
khi tới ranh giới từ — đúng mô hình của bản macOS (IMKit), **không** dùng đường backspace-injection
của Windows.

Cả hai shell chỉ là lớp vỏ mỏng gọi cùng một lõi Rust (`funput-ffi`, C ABI) và dùng chung bộ đọc
settings ở `platforms/linux/common/`. Khác biệt: addon Fcitx5 là `.so` nạp **in-process**; engine
IBus là **tiến trình riêng** do `ibus-daemon` khởi chạy qua D-Bus.

Tách đôi như các nền khác:

```
platforms/linux/
├─ common/      Code dùng chung mọi shell Linux (Fcitx5, IBus). Khép kín, không
│  │            biết gì về framework — link vào qua add_subdirectory().
│  ├─ ffi_handle.h           RAII wrapper quanh FunputEngine* (funput.h)
│  ├─ settings.{h,cpp}       đọc/ghi ~/.config/Funput/settings.json (mtime + reload())
│  └─ settings_watch.{h,cpp} theo dõi file (inotify) → áp dụng settings tức thì
├─ fcitx5/      Addon C++ (libfunput.so) — gọi funput-ffi (C ABI), link common. PHẦN "gõ".
│  ├─ src/funput_engine.cpp  InputMethodEngineV2: keyEvent → preedit/commit
│  └─ data/*.conf.in         metadata addon + input method
├─ ibus/        Engine C++ (ibus-engine-funput) — tiến trình IBus, link common. PHẦN "gõ".
│  ├─ src/engine.cpp         IBusEngine: process_key_event → preedit/commit
│  ├─ src/main.cpp           IBusBus + IBusFactory + ibus_main (cờ --ibus/--xml)
│  └─ data/funput.xml.in     component manifest (ibus-daemon đăng ký + exec engine)
├─ settings-gtk/ App Settings/Onboarding (GTK4 + libadwaita, Rust). KHÔNG hook, KHÔNG engine.
└─ packaging/   .desktop launcher (CPack gộp vào .deb)
```

> `settings-gtk` chỉ build trên **Linux** (link gtk4/libadwaita hệ thống) và **bị `exclude`** khỏi
> workspace gốc, nên `cargo test --workspace` trên macOS vẫn xanh. Addon là project **CMake** riêng.

## Hai tiến trình, một file settings

Addon `.so` do daemon `fcitx5` nạp; app Settings (GTK) là tiến trình riêng → **không** chia sẻ state
trong process như Windows. Cầu nối là `~/.config/Funput/settings.json` (cùng file `dirs::config_dir()`
mà bản Windows ghi). App Settings ghi file; addon đọc lại khi **focus-in** (so mtime, rẻ). Bật/tắt
VI–EN bằng hotkey trong addon cũng ghi ngược lại file để UI phản ánh đúng.

Tray + biểu tượng trạng thái dùng luôn của **Fcitx5** (không dựng tray riêng).

## Build từ source

Yêu cầu (Debian/Ubuntu):

```sh
sudo apt install \
  cmake nlohmann-json3-dev \
  fcitx5 libfcitx5core-dev libfcitx5utils-dev libfcitx5config-dev \
  ibus libibus-1.0-dev libglib2.0-dev \
  libgtk-4-dev libadwaita-1-dev librsvg2-dev
# + Rust (rustup). Yêu cầu Ubuntu 24.04+ (GTK 4.14 / libadwaita 1.5) cho app Settings.
# Chỉ dựng một shell? Bỏ qua dev-lib của shell kia (fcitx5* hoặc ibus/libibus*).
```

Một lệnh dựng cả hai gói `.deb`:

```sh
platforms/linux/build.sh
# → cargo build -p funput-ffi  (cdylib)
# → cargo build (Settings app — GTK4 + libadwaita)
# → cmake + cpack × {fcitx5, ibus}
# Kết quả:
#   platforms/linux/build/fcitx5/funput_<version>_<arch>.deb
#   platforms/linux/build/ibus/funput-ibus_<version>_<arch>.deb   (arch = host)

# Chỉ một shell:
FUNPUT_FRAMEWORK=ibus   platforms/linux/build.sh
FUNPUT_FRAMEWORK=fcitx5 platforms/linux/build.sh
```

Cài thử: `sudo apt install ./platforms/linux/build/fcitx5/funput_*.deb`
(hoặc `./platforms/linux/build/ibus/funput-ibus_*.deb`).

> Cài đúng kiến trúc: gói build trên máy nào ra kiến trúc máy đó (`amd64`/`arm64`).
> Apple Silicon (Apple Virtualization) là `arm64`; gói `amd64` sẽ báo mọi dependency
> "not installable" do lệch kiến trúc.
>
> Nếu `apt`/`dpkg` báo lỗi đọc file (`could not locate member control.tar`,
> `unexpected end of file…`) → file `.deb` tải về bị **hỏng/cắt cụt**, không phải lỗi
> gói. Tải lại trực tiếp trên máy đích và đối chiếu `sha256sum` với file `.sha256`.

## Chọn framework nào?

- **GNOME / Ubuntu mặc định** → **IBus** (cài sẵn, không cần dựng thêm gì). Gói `funput-ibus`.
- **KDE Plasma, hoặc muốn đầy đủ tính năng** (per-app auto-switch) → **Fcitx5**. Gói `funput`.

Đừng cài cả hai cùng lúc cho cùng một desktop — chọn đúng framework mà session đang chạy.

## Cài & bật — IBus (GNOME)

1. Cài gói: `sudo apt install ./.../ibus/funput-ibus_*.deb`.
2. Nạp lại engine mới đăng ký: `ibus restart` (hoặc đăng nhập lại).
3. **Settings → Keyboard → Input Sources → +** → **Vietnamese** → **Funput**.
4. Chuyển nguồn nhập: **`Super + Space`**. Bật/tắt VI–EN khi đang ở Funput: **`Ctrl + `` `**.
5. Đổi Telex/VNI, smart/eager restore: mở **Funput** trong menu ứng dụng (app Settings GTK).

> Smoke test không cần session: `ibus-engine-funput --xml` phải in ra component XML hợp lệ.

## Cài & bật — Fcitx5

1. `fcitx5-configtool` → **+** → thêm **Funput** (nhóm Vietnamese).
2. Nếu Fcitx5 chưa chạy: đăng nhập lại, hoặc đảm bảo `GTK_IM_MODULE=fcitx`, `QT_IM_MODULE=fcitx`,
   `XMODIFIERS=@im=fcitx` (X11) — Wayland đời mới dùng `text-input-v3` nên thường không cần.
3. Bật/tắt tiếng Việt: **`Ctrl + `` `** (mặc định) hoặc icon Fcitx5 ở khay.
4. Đổi Telex/VNI, smart/eager restore: mở **Funput** trong menu ứng dụng (app Settings GTK).

## Verify (trên Linux, X11 lẫn Wayland)

Mở một app GTK (gedit) và một app Qt:

- VNI `xin chaof` → `xin chào`; Telex `tieesng vieejt` → `tiếng việt` (preedit gạch chân, commit khi
  gõ dấu cách).
- Backspace giữa từ: `Phuas` → ⌫ → `Phú`.
- Smart-restore: `card ` giữ `card` (không thành `cảd`).
- `Ctrl+` ``` bật/tắt — chữ ra Latin thường khi tắt.
- Đổi Telex/VNI trong app Settings → focus lại ô text → có hiệu lực (addon reload từ file).

## Hạn chế đã biết / để sau

- **IBus v1 chưa có per-app auto-switch** (app trong danh sách loại trừ → tiếng Anh). IBus không
  cấp id app đang focus đáng tin cậy, nhất là trên Wayland. Fcitx5 vẫn hỗ trợ đầy đủ tính năng này.
  App Settings **ẩn hẳn trang "Ứng dụng bỏ qua" khi đang chạy IBus** (phát hiện qua session D-Bus +
  env IM-module), chỉ hiện khi Fcitx5 là IME đang hoạt động.
- **Chỉ `.deb`** (chưa rpm/AppImage). Hai gói `funput` (Fcitx5) và `funput-ibus` hiện đóng gói riêng,
  mỗi gói tự bundle `libfunput_ffi.so` + app Settings (gói chung `funput-common` để sau).
- Hotkey `alt_shift` (combo chỉ-modifier) chưa hỗ trợ; UI Linux chỉ hiện `ctrl_backtick`/`ctrl_space`.
- Settings **áp dụng tức thì** qua theo dõi file (inotify, `common/settings_watch.*`): Fcitx5 wire fd
  vào event loop riêng, IBus qua `g_unix_fd_add` trên GLib loop; vẫn giữ fallback so mtime lúc focus-in.
- App Settings là **GTK4 + libadwaita**, yêu cầu **Ubuntu 24.04+** (GTK 4.14 / libadwaita 1.5).
