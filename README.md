<p align="center">
  <img src="assets/logo.png" alt="Funput" width="128">
</p>

# Funput

> Gõ tiếng Việt trên macOS, Windows và Linux — một bộ gõ, ba nơi làm việc, không cần mang theo cả tủ đồ nghề.

**Funput** là bộ gõ Telex & VNI viết bằng Rust, mã nguồn mở, miễn phí. Cài xong là gõ. Icon **FU** trên menu bar hay khay hệ thống là bạn đã sẵn sàng.

[Tải bản mới nhất →](https://github.com/pcodedynamics/Funput/releases)

---

## Cài đặt

| Nền tảng | Trạng thái | Làm gì |
|----------|-----------|--------|
| **macOS** | Sẵn sàng | Tải `.pkg` → double-click để cài → **System Settings → Keyboard → Input Sources → + → Vietnamese → Funput** |
| **Windows** | Sẵn sàng | Tải `funput.exe` → double-click → icon **FU** xuất hiện ở khay |
| **Linux** | Sẵn sàng | Tải `funput_*_amd64.deb` → `sudo apt install ./funput_*_amd64.deb` → bật **Funput** trong `fcitx5-configtool` |

**macOS:** Không có trên App Store (input method không sandbox được — không phải Funput lười). File `.pkg` đã ký Developer ID + notarized nên cài trực tiếp không bị chặn. **Không có quyền admin?** Tải `.app.zip`, giải nén rồi kéo `Funput.app` vào `~/Library/Input Methods` (Finder → `Cmd+Shift+G` → gõ đường dẫn) — không cần admin. Không thấy Funput trong danh sách? Log out/in một lần là xong.

**Windows:** SmartScreen có thể nhăn mặt lần đầu (chưa ký Authenticode). Mở app → Onboarding chào bạn → gõ luôn.

**Linux:** Cần **Fcitx5** (cài sẵn trên đa số distro, hoặc `sudo apt install fcitx5`). Sau khi cài `.deb`: chạy `fcitx5-configtool` → thêm **Funput** vào danh sách input method (bỏ qua bước này nếu Fcitx5 tự nhận). Đăng nhập lại nếu Fcitx5 chưa chạy. Bật/tắt tiếng Việt bằng **`Ctrl + `` `** hoặc icon Fcitx5 ở khay. Mở **Funput** trong menu ứng dụng để đổi Telex/VNI.

---

## Gõ thử ngay

Bật Funput, gõ thử:

| Kiểu gõ | Bạn gõ | Funput cho ra |
|---------|--------|---------------|
| Telex | `tieesng vieejt` | tiếng việt |
| VNI | `xin chaof` | xin chào |

Đổi Telex/VNI trong **Settings**. Bật/tắt nhanh: menu bar hoặc tray — Windows thêm **`Ctrl + `` `** (backtick) cho nhanh tay.

---

## Funput là gì?

Một engine Rust gõ tiếng Việt, bọc vỏ native cho từng OS:

- **macOS** — input method thật sự, ngồi trong menu bar
- **Windows** — chạy nền, icon ở khay, gõ mọi app
- **Linux** — addon Fcitx5, gõ mọi app trên X11 lẫn Wayland

Cùng một lõi, cùng một cảm giác gõ — dù bạn nhảy giữa MacBook, PC gaming hay máy dev Linux. Không phải cách mạng thế giới, chỉ là **bộ gõ mới, đa nền tảng, làm cho xong việc**.

---

## Có gì hay?

- **Telex & VNI** — gõ kiểu nào cũng được, đổi trong Settings
- **Smart restore** — gõ `text ` xong vẫn là `text `, không biến thành chữ Việt oái oăm
- **Eager restore** — chỉnh độ “nhạy” khi chuyển về Latin, tùy gu
- **Settings & Onboarding** — một giao diện cho mọi nền tảng, không cần đọc hướng dẫn dài

---

## Muốn soi code?

Logic gõ nằm trong Rust; mỗi platform chỉ là lớp vỏ:

```
crates/           → lõi Rust (core, engine, ffi)
platforms/macos   → Input Method (Swift)
platforms/windows → Tauri + keyboard hook
platforms/linux   → Fcitx5 addon (C++) + Settings (Tauri)
platforms/ui      → Settings UI (Svelte)
```

Đào sâu:

- [funput-core](crates/funput-core/README.md) — Telex/VNI, validation
- [funput-engine](crates/funput-engine/README.md) — session, buffer, pipeline
- [funput-ffi](crates/funput-ffi/README.md) — C ABI
- [funput-cli](crates/funput-cli/README.md) — test engine từ terminal
- [platforms/windows](platforms/windows/README.md) — build Windows từ source
- [platforms/linux](platforms/linux/README.md) — build Linux (Fcitx5 + .deb) từ source

```sh
cargo test --workspace
```

Pull request và issue đều được — Funput thích có bạn cùng chơi.

---

## License

[MIT](LICENSE) — © [PulseFu](https://pulsefu.com). Dùng thoải mái, giữ lại dòng copyright là được.
