# AirCard-CMaku (Windows Edition) 🎴

> **Apple Wallet Card Skinner, Passcode Themer & Decoupled Card Studio for iOS 18+ (No Jailbreak Required)**  
> Native Windows client written in Rust. Powered by the upstream `airlift` engine by **[@Lumid-Off](https://github.com/Lumid-Off/AirCard-Windows)**.

---

### ⭐ Star Campaign / Ủng Hộ Tác Giả Gốc
> **Please visit and Star the upstream repository!**  
> Dự án này được xây dựng trên nền tảng kỹ thuật xuất sắc của **[@Lumid-Off](https://github.com/Lumid-Off)**. Hãy dành 5 giây ghé thăm và tặng một ngôi sao (Star ⭐) cho tác giả gốc tại:  
> 👉 **[https://github.com/Lumid-Off/AirCard-Windows](https://github.com/Lumid-Off/AirCard-Windows)**  
> 
> *⚠️ Lưu ý / Issue Demarcation: Mọi thắc mắc, đóng góp hoặc vấn đề phát sinh từ Card Studio / bản mod này vui lòng mở Issue tại [SirMaku-git/AirCard-CMaku](https://github.com/SirMaku-git/AirCard-CMaku), KHÔNG mở Issue lên repo của tác giả gốc Lumid-Off.*

---

## 🌟 Highlights & Key Features / Tính Năng Nổi Bật

- 🎨 **Decoupled Interactive Card Studio:**
  - Thiết kế và tùy biến thẻ ngân hàng đa lớp (Card Details, Custom EMV Chip, Contactless Waves, Surface Finishes/Foils, Textures).
  - Tự do di chuyển, xoay, phóng to thu nhỏ và căn chỉnh từng layer trực tiếp trên canvas 1536 × 969.
  - Công cụ trung lập (Neutral Graphic Tooling): Không tích hợp sẵn logo thương hiệu thanh toán (Visa, Mastercard, JCB, Napas) để bảo đảm tuân thủ bản quyền, người dùng có thể tự tải lên huy hiệu cá nhân (Custom Upload).
- 🇻🇳 **Hỗ Trợ Tiếng Việt Hoàn Hảo (Zero Glitches):**
  - Tích hợp hệ thống fallback font Segoe UI, Consolas, Arial, Tahoma trực tiếp từ Windows — khắc phục triệt để lỗi mất dấu, rớt ký tự (`Đ`, `Ă`, `Â`, `Ê`, `ễ`, `ơ`, `ư`,...), ô vuông `□` hoặc dấu chấm hỏi.
  - Chuẩn hóa Unicode NFC tự động khi render văn bản nổi dập thẻ (Cardholder name, Bank name).
  - Giao diện đa ngôn ngữ: Tiếng Việt, English, 简体中文 (Simplified Chinese).
- 📶 **Kết Nối Linh Hoạt (USB & WiFi Sync Transport):**
  - Quét bắt mã thẻ qua `syslog_relay` và nạp skin thẻ qua cáp USB hoặc mạng WiFi nội bộ đã ghép nối mà không cần cắm dây.
- 🔄 **An Toàn & Khôi Phục (Snapshot & Backup Restore):**
  - Tự động sao lưu giao diện thẻ gốc trước khi nạp skin mới.
  - Nút **"Restore Original" (Khôi phục Thẻ gốc)** giúp hoàn nguyên thẻ về trạng thái xuất xưởng bất kỳ lúc nào và xóa cache Wallet.
- 🔒 **Passcode Dialer Themer (.passthm):**
  - Nạp theme bàn phím số màn hình khóa từ Cowabunga & Nugget cho iOS 18+ (`TelephonyUI-10`), iOS 16-17 (`TelephonyUI-9`), và iOS cũ.
- ⚡ **100% Native & Lightweight:**
  - File chạy đơn nhất `aircard-cmaku.exe` (~8.7 MB), tối ưu LTO, không cần Python hay runtime cồng kềnh.

---

## 🏛️ Kiến Trúc Tách Biệt (Decoupled Architecture)

Để đảm bảo có thể đồng bộ (pull / clone) các cập nhật kỹ thuật mới nhất từ tác giả gốc **Lumid-Off** một cách dễ dàng và không gây xung đột (zero merge conflicts), toàn bộ 10 tệp mã nguồn cốt lõi được giữ **nguyên bản 100% (byte-for-byte untouched)**:
- `src/afc.rs`
- `src/airlift.rs`
- `src/airtraffic.rs`
- `src/apple.rs`
- `src/device.rs`
- `src/flasher.rs`
- `src/image_skin.rs`
- `src/passthm.rs`
- `src/scanner.rs`
- `src/wallet_backup.rs`

Tất cả các tính năng mở rộng của Card Studio, hệ thống font tiếng Việt và i18n được đóng gói độc lập trong `src/card_studio/` và `src/i18n.rs`.

---

## 📋 Yêu Cầu Hệ Thống (Requirements)
- **Windows 10 / 11 (64-bit)**
- **Apple Mobile Device Support / iTunes 64-bit** (bắt buộc để giao tiếp với thiết bị iOS).
- Cáp USB Lightning hoặc USB-C cho lần kết nối và bấm "Tin cậy" đầu tiên.
- Chế độ WiFi yêu cầu PC và iPhone kết nối chung mạng WiFi nội bộ và đã bật "Sync with this iPhone over Wi-Fi" trong iTunes / Apple Devices.

---

## 🛠️ Hướng Dẫn Sử Dụng (Quick Start)

### 1. Bắt Mã Thẻ (Scan Card Hash)
1. Kết nối iPhone với máy tính và mở khóa màn hình.
2. Tại tab **Wallet**, bấm nút **"Quét thẻ" (Scan)**.
3. Trên iPhone, mở ứng dụng **Apple Wallet** (hoặc nhấp đúp nút Nguồn) và chạm vào thẻ cần đổi.
4. Ứng dụng sẽ tự động bắt mã hash thẻ và lưu vào danh sách. Bấm **"Dừng" (Stop)**.

### 2. Thiết Kế & Nạp Thẻ (Card Studio & Flash)
1. Chọn thẻ mục tiêu hoặc nhập mã hash.
2. Tùy biến hình ảnh nền, chip EMV, sóng contactless, và thông tin dập nổi trong Card Studio (hỗ trợ gõ tiếng Việt có dấu đầy đủ).
3. Bấm **"Nạp Thẻ Vào iPhone" (Apply Card Skin)**.
4. Sau khi báo thành công, mở **App Switcher** trên iPhone (vuốt từ đáy màn hình lên), **vuốt tắt hoàn toàn ứng dụng Apple Wallet** rồi mở lại để thấy giao diện mới!

### 3. Khôi Phục Thẻ Gốc (Restore Original)
- Bất kỳ lúc nào muốn quay lại giao diện ban đầu của ngân hàng, chỉ cần chọn thẻ và bấm nút **"Khôi phục Thẻ gốc" (Restore Original)**.

---

## 🔨 Biên Dịch Từ Mã Nguồn (Building from Source)

Yêu cầu: [Rust toolchain](https://rustup.rs/) (`stable-x86_64-pc-windows-msvc`).

```powershell
# Clone kho lưu trữ
git clone https://github.com/SirMaku-git/AirCard-CMaku.git
cd AirCard-CMaku

# Kiểm tra và chạy test suites
cargo test

# Biên dịch bản release độc lập
cargo build --release
```

Tệp thực thi đầu ra: `target\release\aircard-cmaku.exe`.

---

## 👥 Tri Ân & Đóng Góp (Credits & Contributors)
- **[@Lumid-Off](https://github.com/Lumid-Off)**: Tác giả bản Windows Native Rust Port & duy trì upstream `AirCard-Windows`.
- **[@mak5er](https://github.com/mak5er)**: Tác giả ứng dụng macOS gốc và nghiên cứu kỹ thuật exploit.
- **[0xjohnny (@0xjohnnydev)](https://github.com/0xjohnnydev)**: Tác giả của **AirLift**, mã nguồn gốc vượt sandbox AirTraffic/ATAirlock nền tảng.
- **SirMaku**: Phát triển Card Studio tương tác, bộ engine font tiếng Việt, kiến trúc module tách biệt và tối ưu hóa UI.
- Gói theme bàn phím số lấy cảm hứng từ các dự án cộng đồng [Cowabunga](https://github.com/leminlimez/Cowabunga) & [Nugget](https://github.com/leminlimez/Nugget).

---

## 📜 Giấy Phép & Miễn Trừ Trách Nhiệm (License & Disclaimer)
- Dự án được phát hành theo giấy phép **MIT License**.
- Dự án phát triển phục vụ mục đích học tập, nghiên cứu kỹ thuật cá nhân. Người dùng tự chịu trách nhiệm khi sử dụng.
- Phần mềm hoàn toàn độc lập, không liên kết, không được tài trợ hay chứng thực bởi Apple Inc.
