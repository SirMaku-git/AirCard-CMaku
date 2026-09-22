# AirCard-CMaku 🎴

> **Customized Apple Wallet Card Skinner & Lockscreen Passcode Themer for iOS 18+ (No Jailbreak Required)**  
> Native Windows Rust client modded by **SirMaku**, originally ported by **Lumid-Off**. Powered by the `airlift` AirTraffic sync exploit.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011-0078D6.svg)]()
[![Rust: 2024](https://img.shields.io/badge/Rust-2024%20Edition-DEA584.svg)]()

---

## 📢 Project Notice & Disclaimer (Lưu Ý Quan Trọng)

- 🎮 **For Fun Project**: Đây là dự án phi thương mại được thực hiện hoàn toàn vì mục đích học tập, giải trí và sáng tạo cá nhân. Nên dự án có thể sẽ bị bỏ dỡ bất cứ lúc nào. (**For fun / Non-commercial**).
- 🤖 **AI-Assisted Mod**: Dự án phiên bản tùy biến này được thực hiện và hoàn thiện với sự trợ giúp của **AI (Google Antigravity / Gemini 3.8 Flash)** kết hợp cùng ý tưởng và định hướng của **SirMaku**.
- 🎯 **Phạm Vi Tùy Biến (Scope of Customization)**:
  - **SirMaku CHỈ tập trung tùy biến và mở rộng các tính năng liên quan đến Apple Pay / Apple Wallet Card Studio** (tạo hiệu ứng đa lớp, upload texture/foil, upload custom EMV chip, căn chỉnh logo và giải quyết xung đột điều khiển preview).
  - **Tất cả các thành phần kỹ thuật cốt lõi khác** (như exploit AirTraffic sync irlift, cơ chế kết nối thiết bị qua USB usbmuxd, sao lưu/phục hồi snapshot qua Books, và tính năng cài đặt giao diện bàn phím Passcode passthm) **KHÔNG bị can thiệp nhiều và hoàn toàn mang tính kế thừa nguyên bản từ dự án của tác giả gốc ([@Lumid-Off](https://github.com/Lumid-Off)) cùng các tác giả tiền nhiệm ([@mak5er](https://github.com/mak5er), [0xjohnny](https://github.com/0xjohnnydev))**.

- 📜 **Mã Nguồn Mở & Giấy Phép (Open Source & Free to Use)**:
  - Dự án hoàn toàn miễn phí và mở mã nguồn (MIT License). Bạn được tự do tải về, tham khảo, sử dụng và chỉnh sửa code theo ý muốn.
  - **QUY ĐỊNH BẮT BUỘC**: Mọi hành vi chia sẻ, phân phối lại hoặc phát triển tiếp nối mã nguồn này **bắt buộc phải ghi nhận đầy đủ tên tác giả gốc ([@Lumid-Off](https://github.com/Lumid-Off)) và người tùy biến/modder ([@SirMaku](https://github.com/SirMaku-git))**.

---

### 🌐 English Summary & Customization Scope Notice

- 🎯 **Customization Scope**: SirMaku **ONLY** customized and enhanced features related to **Apple Pay / Apple Wallet (Card Studio)** (interactive multi-layer studio, custom texture/foil finishes, custom EMV chip upload & positioning, layer alignment engine, and live card preview).
- ⚙️ **Core Exploit & Inheritance**: All other core technical mechanisms — including the AirTraffic sync sandbox escape (`airlift`), USB device communication (`usbmuxd`), Books snapshot backup/restore, and Passcode theming (`passthm`) — were **NOT** heavily modified and are directly inherited from the original upstream project by **[@Lumid-Off](https://github.com/Lumid-Off)** and upstream contributors (**[@mak5er](https://github.com/mak5er)**, **[0xjohnny](https://github.com/0xjohnnydev)**).
- 🎮 **For Fun / Non-commercial**: This is a non-commercial, hobbyist project created for fun, learning, and personalization.
- 🤖 **AI-Assisted**: Development was assisted by AI (**Google Antigravity / Gemini**).
- 📜 **Attribution Requirement**: Free and open source under MIT License. Attribution to both the original author (**[@Lumid-Off](https://github.com/Lumid-Off)**) and customizer (**[@SirMaku](https://github.com/SirMaku-git)**) is strictly required upon any redistribution or modification.

---

## ✨ Điểm Mới Trong Phiên Bản AirCard-CMaku (SirMaku Mod)

Phiên bản **AirCard-CMaku** bổ sung bộ công cụ **Card Studio Đa Tầng (Multi-Layer Card Studio)** cực kỳ mạnh mẽ:

1. ✨ **Tùy Chọn Finish Đa Dạng & Custom Texture / Foil Upload**:
   - Bổ sung tùy chọn `Custom Texture / Foil (Upload)` trong danh sách `Finish:`.
   - Cho phép upload ảnh Foil/Texture ánh kim, vân carbon hoặc hiệu ứng hologram riêng (PNG, JPG, WebP).
   - Tích hợp thanh trượt điều chỉnh độ mờ đục **Opacity** (5% - 100%), tỷ lệ **Scale** (20% - 500%), và tự do kéo di chuyển vị trí texture trên thẻ.
2. 💳 **Upload Custom EMV Smartcard Chip (PNG)**:
   - Cho phép upload ảnh Chip kim loại riêng (ảnh PNG trong suốt) để thay thế chip mặc định nếu muốn độ chân thực cao hoặc thiết kế thẻ đen sang trọng.
   - Tự do tùy chỉnh tỷ lệ phóng to thu nhỏ (**Chip Scale: 30% - 250%**), tọa độ **X-Pos**, **Y-Pos** và nút `↺ Reset` đưa về chuẩn vị trí thẻ ngân hàng ISO 7810.
   - Nút **Use Default Gold** giúp khôi phục chip kim loại nguyên bản bất kỳ lúc nào.
3. 🏷️ **Điều Chỉnh Vị Trí & Tỷ Lệ Brand Logo**:
   - Cho phép phóng to thu nhỏ (**Logo Scale**) và di chuyển tọa độ **X**, **Y** cho tất cả các logo mạng lưới thanh toán (Visa, Mastercard, Napas, JCB hoặc Custom Logo).
4. 🎯 **Hệ Thống Giải Quyết Xung Đột Điều Khiển Đa Lớp (Multi-Layer Control Engine)**:
   - Thanh chọn lớp trực quan đặt ngay trên thẻ: `[🖼️ Background] [✨ Finish] [💳 Chip] [🏷️ Logo]`.
   - **Direct-Click Hit Testing**: Click chuột trực tiếp vào Chip hoặc Logo trên thẻ preview sẽ tự động kích hoạt lớp điều khiển tương ứng.
   - **Figma-Style Bounding Box**: Khung viền màu tím sáng hiển thị bao quanh đúng phần tử đang được chọn để người dùng căn chỉnh dễ dàng.
   - Thao tác kéo chuột (Drag) và cuộn chuột (Scroll zoom) chỉ tác động trực tiếp vào lớp đang được chọn, không làm trôi ảnh nền.
5. 🌐 **Tab Sources Vinh Danh & Bản Quyền**:
   - Tab **Sources** tích hợp sẵn trên giao diện ứng dụng để người dùng dễ dàng xem thông tin tác giả gốc, người mod, các nhà nghiên cứu bảo mật và liên kết dự án.

---

## 🚀 Tính Năng Chính (Core Features)

- 🎨 **Tùy biến thẻ Apple Wallet**: Tùy chỉnh ảnh nền, màu sắc thẻ, khung viền logo, dập nổi thông tin thẻ (Cardholder, Card Number, Expiry Date, Bank Name).
- 🔢 **Lock Screen Passcode Themes (.passthm)**: Cài đặt giao diện bàn phím số màn hình khóa từ các gói theme Cowabunga & Nugget (.passthm) cho iOS 16 - 18+.
- ⚡ **100% Native & Siêu Nhẹ**: File thực thi duy nhất `AirCard-cmaku.exe` dung lượng ~7.5 MB, khởi động tức thì, không cần Python hay runtime cồng kềnh.
- 🪟 **Giao diện Material Design 3 Dark**: Tối ưu thẩm mỹ hiện đại với `egui` / `eframe`.
- 📱 **Bắt Pass Hash Thẻ Tự Động**: Bắt mã hash thẻ thời gian thực qua `syslog_relay` khi chạm thẻ trong Apple Wallet trên iPhone.
- 🔄 **An Toàn Tuyệt Đối**: Sao lưu và khôi phục trạng thái nguyên vẹn qua cơ chế snapshot của Apple Books.
- 🔓 **Không Cần Jailbreak**: Khai thác giao thức đồng bộ AirTraffic hợp lệ của Apple mà không can thiệp phân vùng hệ thống.

---

## 💻 Yêu Cầu Hệ Thống (Requirements)

- **Windows 10 / 11 (64-bit)**
- **Apple Mobile Device Support / iTunes 64-bit** (cần thiết để nạp driver giao tiếp USB của Apple).
- Cáp kết nối Lightning hoặc USB-C chính hãng / chuẩn MFi.

---

## ⚠️ Khắc Phục Lỗi Driver (Troubleshooting)

> [!TIP]
> **Nếu ứng dụng không nhận iPhone hoặc quá trình sync bị treo:**  
> Xung đột driver Apple trên Windows là nguyên nhân phổ biến nhất. Hãy làm theo các bước sau:
> 1. Tải và cài đặt **[3uTools](https://www.3u.com/)**.
> 2. **Rút cáp iPhone** khỏi máy tính.
> 3. Trong 3uTools, chọn **Toolbox ➔ Repair Driver**.
> 4. Nhấn **Repair Now** và chờ công cụ hoàn tất cài đặt lại Apple Mobile Device driver.
> 5. Cắm lại iPhone, mở khóa màn hình, bấm **Trust this Computer (Tin cậy máy tính này)** và mở lại **AirCard-CMaku**.

---

## 📖 Hướng Dẫn Sử Dụng (How to Use)

### 1. Đổi Ảnh Thẻ Apple Wallet
1. Cắm iPhone vào máy tính và mở khóa màn hình.
2. Trên AirCard-CMaku, tại tab **Wallet**, nhấn nút **Scan**.
3. Mở ứng dụng **Apple Wallet** trên iPhone, chạm vào thẻ bạn muốn thay skin (app sẽ tự động nhận diện mã hash và hiển thị thông báo). Nhấn **Stop**.
4. Nhấn **Choose Image...** để chọn ảnh nền hoặc chọn các Preset màu có sẵn.
5. Tùy chỉnh lớp phủ trong phần **Card Studio**:
   - Chọn Finish (Standard, Sheen, Carbon, hoặc Custom Texture).
   - Bật/tắt Chip EMV, Contactless Wave, Logo thương hiệu, dập nổi thông tin.
   - Dùng chuột kéo hoặc cuộn bánh xe để zoom/di chuyển các lớp trên thẻ preview.
6. Nhấn **Apply Card Skin** và chờ thông báo hoàn thành.
7. Mở App Switcher trên iPhone (vuốt từ dưới màn hình lên), vuốt tắt ứng dụng **Wallet**, sau đó mở lại Wallet để chiêm ngưỡng thẻ mới!

### 2. Cài Đặt Theme Bàn Phím Số (.passthm)
1. Chuyển sang tab **Passcode** trong AirCard-CMaku.
2. Nhấn **Choose .passthm...** và chọn file theme (tương thích các theme từ Cowabunga / Nugget).
3. Chọn phiên bản TelephonyUI phù hợp:
   - **Auto (TelephonyUI-10)** — Cho iOS 18+ (Mặc định).
   - **TelephonyUI-9** — Cho iOS 16 - 17.
   - **TelephonyUI-8** — Cho iOS cũ hơn.
4. Nhấn **Apply Passcode Theme**.
5. Khóa màn hình iPhone để kiểm tra giao diện phím số mới!

> [!IMPORTANT]
> **Tắt Chữ Đậm (Turn OFF Bold Text):**  
> Trên iPhone, vào **Cài đặt ➔ Màn hình & Độ sáng** và đảm bảo tùy chọn **Chữ đậm (Bold Text)** đang ở trạng thái **TẮT**. Nếu bật chữ đậm, iOS sẽ bỏ qua ảnh nút tùy biến và hiển thị font vector mặc định của hệ thống.

---

## 🛠️ Biên Dịch Từ Mã Nguồn (Building from Source)

Yêu cầu: Đã cài đặt [Rust](https://rustup.rs/) toolchain (`stable-x86_64-pc-windows-msvc`).

```powershell
# Chạy bộ kiểm thử (16 unit & integration tests)
cargo test

# Biên dịch bản release tối ưu hóa
cargo build --release
```

File thực thi sau khi biên dịch nằm tại: `target\release\AirCard-cmaku.exe`.

---

## 👥 Contributors & Credits

### Contributors
- **[@Lumid-Off](https://github.com/Lumid-Off)** (Windows Native Rust Port & Maintainer) — [GitHub](https://github.com/Lumid-Off) · [Twitter / X](https://x.com/LumidOff)
- **[@mak5er](https://github.com/mak5er)** (Original macOS App & Exploit Research) — [GitHub](https://github.com/mak5er) · [Twitter / X](https://x.com/mak5er)
- **[AirLift](https://github.com/0xjohnnydev/airlift)** by **[0xjohnny (@0xjohnnydev)](https://github.com/0xjohnnydev)**: Original AirTraffic/ATAirlock sandbox escape & proof of concept underlying `AirliftFFI`.
- **[@SirMaku](https://github.com/SirMaku-git)** (Customizer): Thiết kế và phát triển Card Studio Đa Tầng, Custom Texture & Foil upload, Custom EMV Chip engine, layer alignment, và UI Sources.

### Credits
- Khai thác cốt lõi dựa trên `airlift` (AirTraffic sync escape).
- Định dạng theme dialer lấy cảm hứng từ [Cowabunga](https://github.com/leminlimez/Cowabunga) và [Nugget](https://github.com/leminlimez/Nugget).
- Dự án mod được hỗ trợ thực hiện bởi **AI (Google Antigravity / Gemini 3.8 Flash)**.

---

## 📄 License

Dự án phát hành theo giấy phép [MIT License](LICENSE).
Mọi hành vi tái phân phối hoặc phát triển tiếp nối bắt buộc phải giữ lại ghi danh tác giả gốc ([@Lumid-Off](https://github.com/Lumid-Off)) và người tùy biến ([@SirMaku](https://github.com/SirMaku-git)).
