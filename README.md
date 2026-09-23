# AirCard-CMaku 🎴

> **Customized Apple Wallet Card Skinner & Lockscreen Passcode Themer for iOS 18+ (No Jailbreak Required)**  
> Bản mod Rust native trên Windows duy trì bởi **SirMaku**, phái sinh từ dự án gốc của **Lumid-Off**. Sử dụng cơ chế đồng bộ AirTraffic (`airlift`).  
> 🔗 **Original Upstream Repository**: [Lumid-Off/AirCard-Windows](https://github.com/Lumid-Off/AirCard-Windows)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011-0078D6.svg)]()
[![Rust: 2024](https://img.shields.io/badge/Rust-2024%20Edition-DEA584.svg)]()

---

## 📢 Tuyên Bố Pháp Lý, Bản Quyền & Miễn Trừ Trách Nhiệm (Disclaimers & Notice)

### 1. Mục Đích Phát Triển & Cảnh Báo Rủi Ro (Academic Purpose & Risk Warning)
- **Mục đích sử dụng**: Dự án này được phát triển hoàn toàn cho **mục đích học tập, nghiên cứu kỹ thuật và sử dụng cá nhân**. 
- **Trách nhiệm người dùng**: **Người dùng tự chịu toàn bộ rủi ro và trách nhiệm** khi tải về, biên dịch và chạy công cụ trên thiết bị của mình.
- **Rủi ro kỹ thuật**: Quá trình can thiệp và ghi file có thể làm biến đổi cấu trúc dữ liệu cục bộ trên thiết bị, có khả năng phát sinh lỗi timeout giao tiếp AirTraffic, lỗi khôi phục snapshot hoặc không tương thích trên một số phiên bản iOS nhất định, dẫn đến việc phải khôi phục lại thiết bị nếu quá trình ghi gặp sự cố giữa chừng.
- **Mức độ an toàn**: Mặc dù công cụ tích hợp cơ chế snapshot và restore qua Apple Books để giảm thiểu rủi ro, **dự án KHÔNG THỂ và KHÔNG ĐẢM BẢO an toàn tuyệt đối**. Khuyến cáo mạnh mẽ: **Người dùng nên chủ động sao lưu (backup) toàn bộ dữ liệu thiết bị qua iTunes / Finder / iCloud trước khi sử dụng.**

### 2. Quan Hệ Với Apple (Apple Non-Affiliation Disclaimer)
- **Công cụ độc lập**: Đây là một công cụ mã nguồn mở độc lập, **hoàn toàn KHÔNG được Apple Inc. tài trợ, ủy quyền, xác nhận hoặc bảo đảm**.
- **Cơ chế AirTraffic**: AirTraffic là một cơ chế đồng bộ có sẵn trong hệ sinh thái Apple, tuy nhiên phương thức khai thác và ứng dụng trong dự án này hoàn toàn không đại diện cho quan điểm, khuyến nghị hay sự chấp thuận từ phía Apple.
- **Thương hiệu**: Các nhãn hiệu "Apple", "Apple Pay", "Apple Wallet", "iPhone", "iOS" thuộc quyền sở hữu của Apple Inc. Dự án chỉ đề cập đến các tên gọi này nhằm mục đích mô tả chức năng tương thích kỹ thuật.

### 3. Phân Định Vai Trò Tác Giả & Mối Quan Hệ Với Tác Giả Gốc (Author Roles & Upstream Notice)
- **Bản phái sinh (Derivative Work)**: `AirCard-CMaku` là bản mod phái sinh từ `AirCard-Windows`.
  - **Tác giả nền tảng gốc**: **[@Lumid-Off](https://github.com/Lumid-Off)** và các cộng tác viên là những người phát triển nền tảng Windows nguyên bản, xây dựng kiến trúc Rust native, cơ chế snapshot/restore và bộ kết nối `usbmuxd`.
  - **Nghiên cứu kỹ thuật tiền đề**: **[@mak5er](https://github.com/mak5er)** (tác giả ứng dụng macOS ban đầu) và **[0xjohnny (@0xjohnnydev)](https://github.com/0xjohnnydev)** (tác giả exploit vượt sandbox AirTraffic/ATAirlock `airlift`).
  - **Phạm vi đóng góp của SirMaku**: **[@SirMaku](https://github.com/SirMaku-git)** chỉ là maintainer của bản mod phái sinh, tập trung nghiên cứu và xây dựng bộ công cụ **Card Studio Đa Tầng** (Interactive Multi-Layer Studio, Custom Texture/Foil Upload, Custom EMV Chip engine, Brand Logo alignment, Hit-testing direct click, và giao diện Sources); **không phải tác giả gốc của toàn bộ nền tảng**.
- **Mối quan hệ**: **SirMaku KHÔNG có bất kỳ mối quan hệ cá nhân, tổ chức hay liên kết nào với các tác giả gốc.**
- **Chính sách gỡ bỏ thiện chí (Good-Faith Takedown Policy)**: Nếu tác giả gốc (đặc biệt là Lumid-Off) có bất kỳ yêu cầu gỡ bỏ kho lưu trữ này khỏi GitHub công khai vì bất kỳ lý do gì, người duy trì sẽ hoàn toàn vui lòng tuân thủ và gỡ bỏ ngay lập tức, chỉ lưu giữ bản mod nội bộ để học tập cá nhân và chia sẻ cho bạn bè.
- **Lời tri ân**: Xin gửi lời cảm ơn và lòng biết ơn chân thành, sâu sắc nhất đến tác giả gốc **[@Lumid-Off](https://github.com/Lumid-Off)** cùng các nhà nghiên cứu bảo mật đã mở đường và chia sẻ mã nguồn cho cộng đồng.

### 4. Giấy Phép & Ghi Nhận Bản Quyền (License & Attribution)
- Dự án được phân phối theo giấy phép mã nguồn mở tự do **[MIT License](LICENSE)**.
- Bản quyền mã nguồn được phân tách rõ ràng:
  - Phần mã nguồn gốc nền tảng: Thuộc bản quyền của `Lumid-Off and contributors` ([Lumid-Off/AirCard-Windows](https://github.com/Lumid-Off/AirCard-Windows)).
  - Phần mã nguồn chỉnh sửa và tiện ích Card Studio mở rộng: Thuộc bản quyền của `SirMaku` ([SirMaku-git/AirCard-CMaku](https://github.com/SirMaku-git/AirCard-CMaku)).
  - SirMaku không sở hữu bản quyền toàn bộ dự án.
- Mọi hoạt động sao chép, trích xuất hoặc phân phối lại mã nguồn này phải bảo lưu đầy đủ thông báo bản quyền (Copyright Notice) của cả tác giả gốc và người đóng góp bản mod theo đúng quy định của giấy phép MIT.

---

### 🌐 English Summary & Legal Disclaimers

- 🎯 **Derivative Work & Scope**: `AirCard-CMaku` is an independent derivative work based on `AirCard-Windows`. SirMaku maintains this mod and developed the **Multi-Layer Card Studio** (custom texture/foil uploads, custom EMV chip positioning, logo alignment, and interactive canvas preview). SirMaku is **NOT** the original author of the underlying platform.
- ⚙️ **Original Upstream Authors**: The core platform, AirTraffic sync exploit (`airlift`), USB device communication (`usbmuxd`), and Books snapshot restore mechanisms were designed and built by original creator **[@Lumid-Off](https://github.com/Lumid-Off)**, with upstream research by **[@mak5er](https://github.com/mak5er)** and **[0xjohnny (@0xjohnnydev)](https://github.com/0xjohnnydev)**.
- 🤝 **No Affiliation & Good-Faith Takedown Notice**: SirMaku has **NO affiliation or relationship** with the original authors. If the original authors request the removal of this repository from GitHub for any reason, the maintainer will gladly comply immediately and only maintain the mod privately for personal study and friends. Deep gratitude is expressed to the original creators for their pioneering open-source work.
- 🍎 **Apple Non-Affiliation**: This tool is an unofficial, independent project and is **NOT affiliated with, sponsored, authorized, or endorsed by Apple Inc.** AirTraffic is an existing mechanism in the Apple ecosystem, but its utilization here does not represent Apple's views or authorization.
- ⚠️ **Academic Purpose & Risk Notice**: Created strictly for personal learning and technical research. Users assume all risks. The tool modifies device-level data, which may fail due to USB disconnects, restore errors, or iOS incompatibilities. The Books snapshot/restore mechanism reduces risk but **CANNOT guarantee absolute safety**. Always backup your device before use.
- 📜 **MIT License & Attribution**: Open-source under MIT License. Copyright belongs respectively to Lumid-Off & contributors for original work, and SirMaku for Card Studio modifications. Standard MIT attribution applies.

---

## 📱 Khả Năng Tương Thích & Phạm Vi Đã Kiểm Thử (Compatibility & Testing Matrix)

> [!IMPORTANT]
> **Khả năng tương thích thay đổi theo phiên bản iOS, mẫu thiết bị và driver Apple. "iOS 18+" là phạm vi mục tiêu, không phải bảo đảm hoạt động trên mọi phiên bản.**

### Bảng Ma Trận Kiểm Thử Thực Tế

| Hạng mục | Phạm vi kiểm thử thực tế | Ghi chú & Giới hạn kỹ thuật |
| :--- | :--- | :--- |
| **Phiên bản iOS** | **iOS 18.0 - 18.2** (Apple Wallet & Card Studio); **iOS 16.0 - 17.x** (Passcode Dialer) | Apple có thể sửa đổi hoặc chặn cơ chế đồng bộ AirTraffic trong các bản vá iOS tương lai. |
| **Thiết bị** | **iPhone 11 series đến iPhone 16 series** (Lightning & USB-C) | Đã kiểm thử thực tế và hoạt động trơn tru với cáp truyền dữ liệu chuẩn MFi. |
| **Tính năng ổn định** | • Thay skin thẻ Apple Wallet<br>• Card Studio đa tầng (Foil, Texture, EMV Chip, Logo)<br>• Cài đặt theme bàn phím số Lock Screen (.passthm) | Hoạt động thông qua Books snapshot và local AirTraffic sync. |
| **Lưu ý Passcode** | Cần tắt tính năng **Chữ đậm (Bold Text)** | Trong **Cài đặt ➔ Màn hình & Độ sáng ➔ Chữ đậm: TẮT**. Nếu bật chữ đậm, iOS sẽ bỏ qua theme ảnh nút. |
| **Yêu cầu Driver** | Cần đầy đủ bộ driver **Apple Mobile Device** trên Windows | Xung đột driver Windows là lỗi phổ biến nhất (xem hướng dẫn khắc phục bên dưới). |

---

## ✨ Điểm Mới Trong Phiên Bản AirCard-CMaku (SirMaku Mod)

Phiên bản **AirCard-CMaku** bổ sung bộ công cụ **Card Studio Đa Tầng (Multi-Layer Card Studio)**:

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
   - Thanh chọn lớp trực quan đặt ngay trên thẻ: `[🖼️ Background] [✨ Finish] [💳 Chip] [📶 Wave] [🏷️ Logo] [🔢 Details] [🧩 Custom Widgets] [+]`.
   - **Direct-Click Hit Testing**: Click chuột trực tiếp vào Chip, Wave, Logo, hoặc bất kỳ Widget tự tải lên trên thẻ preview sẽ tự động kích hoạt lớp điều khiển tương ứng.
   - **Figma-Style Bounding Box**: Khung viền màu tím sáng hiển thị bao quanh đúng phần tử đang được chọn để người dùng căn chỉnh dễ dàng.
   - Thao tác kéo chuột (Drag) và cuộn chuột (Scroll zoom) chỉ tác động trực tiếp vào lớp đang được chọn, không làm trôi ảnh nền.
   - **Sửa lỗi Xoay bằng Phím Shift (Shift + Mouse Wheel Fix)**: Nhận diện chính xác sự kiện cuộn chuột ngang/dọc trên Windows khi giữ phím `Shift`, cho phép xoay mượt mà 360° mọi đối tượng được chọn.
5. 🧩 **Nút `+` Thêm Widget Tùy Ý (Custom Widgets Upload)**:
   - Nút `+` (Add Widget) trong danh sách lớp hoặc khung bên hông cho phép tải lên bất kỳ hình ảnh nào (sticker, logo phụ, chip vẽ tay, QR cá nhân, huy hiệu, graphic độc quyền...) mà không bị tù túng bởi các preset có sẵn.
   - **Tùy biến đầy đủ tương đương 100%**: Hỗ trợ kéo thả vị trí (X/Y), phóng to thu nhỏ (Scale), xoay tự do (Rotation), độ trong suốt (Opacity), nhuộm màu RGB (Tint), độ bão hòa (Saturation), dịch tông màu (Hue Shift) và bật/tắt bóng đổ (Drop Shadow).
   - Cho phép thêm đồng thời nhiều widget, quản lý ẩn/hiện hoặc xóa từng widget nhanh chóng.
6. 🌐 **Tab Sources Vinh Danh & Bản Quyền**:
   - Tab **Sources** tích hợp sẵn trên giao diện ứng dụng để người dùng dễ dàng xem thông tin tác giả gốc, người mod, các nhà nghiên cứu bảo mật, chính sách gỡ bỏ và liên kết dự án.

---

## 🚀 Tính Năng Chính (Core Features)

- 🎨 **Tùy biến thẻ Apple Wallet**: Tùy chỉnh ảnh nền, màu sắc thẻ, khung viền logo, dập nổi thông tin thẻ (Cardholder, Card Number, Expiry Date, Bank Name).
- 🔢 **Lock Screen Passcode Themes (.passthm)**: Cài đặt giao diện bàn phím số màn hình khóa từ các gói theme Cowabunga & Nugget (.passthm) cho iOS 16 - 18+.
- ⚡ **100% Native & Siêu Nhẹ**: File thực thi duy nhất `AirCard-cmaku.exe` dung lượng ~7.5 MB, khởi động tức thì, không cần Python hay runtime cồng kềnh.
- 🪟 **Giao diện Material Design 3 Dark**: Tối ưu thẩm mỹ hiện đại với `egui` / `eframe`.
- 📱 **Bắt Pass Hash Thẻ Tự Động**: Bắt mã hash thẻ thời gian thực qua `syslog_relay` khi chạm thẻ trong Apple Wallet trên iPhone.
- 🔄 **Cơ Chế Snapshot & Khôi Phục (Snapshot & Restore)**: Tích hợp sao lưu và khôi phục trạng thái nguyên vẹn qua cơ chế snapshot của Apple Books nhằm giảm thiểu rủi ro (lưu ý: không đảm bảo an toàn tuyệt đối, người dùng nên chủ động backup thiết bị).
- 🔓 **Không Cần Jailbreak**: Khai thác cơ chế đồng bộ AirTraffic có sẵn trên thiết bị mà không cần jailbreak hay can thiệp phân vùng hệ thống (công cụ phi chính thức, độc lập với Apple).

---

## 💻 Yêu Cầu Hệ Thống & Cài Đặt Driver (Requirements & Setup)

- **Hệ điều hành**: Windows 10 / 11 (64-bit).
- **Cáp kết nối**: Cáp Lightning hoặc USB-C hỗ trợ truyền dữ liệu (chuẩn MFi hoặc cáp zin theo máy).
- **Bộ Driver Apple**: Cần cài đặt đầy đủ driver giao tiếp USB **Apple Mobile Device Support**.

### ⚠️ Hướng Dẫn Cài Driver Chuẩn Đã Kiểm Thử (Tested with 3uTools)

> [!TIP]
> **Kinh nghiệm thực tế kiểm thử thành công:**  
> Để ứng dụng hoạt động trơn tru và nhận diện thiết bị ổn định nhất trên Windows, quy trình cài driver qua **3uTools** đã được kiểm chứng hoạt động tốt nhất:
> 1. Tải và cài đặt phần mềm **[3uTools](https://www.3u.com/)**.
> 2. Mở 3uTools, chuyển sang mục **Toolbox** (Hộp công cụ).
> 3. Trong Toolbox, chọn cài đặt **iTunes** (hoặc chọn mục **Repair Driver**).
> 4. Nhấn **Repair Now** để 3uTools tự động tải và cài đặt đồng bộ toàn bộ driver Apple Mobile Device còn thiếu trên máy tính Windows.
> 5. Cắm iPhone vào máy tính, mở khóa màn hình, nhấn **Trust this Computer (Tin cậy máy tính này)**, sau đó khởi chạy **AirCard-CMaku**.

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

---

## 🛠️ Biên Dịch Từ Mã Nguồn (Building from Source)

Yêu cầu: Đã cài đặt [Rust](https://rustup.rs/) toolchain (`stable-x86_64-pc-windows-msvc`).

```powershell
# Chạy bộ kiểm thử tự động (17 tests)
cargo test

# Biên dịch bản release tối ưu hóa
cargo build --release
```

File thực thi sau khi biên dịch nằm tại: `target\release\AirCard-cmaku.exe`.

---

## 👥 Contributors & Credits

### Contributors
- **[@Lumid-Off](https://github.com/Lumid-Off)** (Windows Native Rust Port & Maintainer) — [Official Repository: Lumid-Off/AirCard-Windows](https://github.com/Lumid-Off/AirCard-Windows) · [GitHub Profile](https://github.com/Lumid-Off) · [Twitter / X](https://x.com/LumidOff)
- **[@mak5er](https://github.com/mak5er)** (Original macOS App & Exploit Research) — [GitHub](https://github.com/mak5er) · [Twitter / X](https://x.com/mak5er)
- **[AirLift](https://github.com/0xjohnnydev/airlift)** by **[0xjohnny (@0xjohnnydev)](https://github.com/0xjohnnydev)**: Original AirTraffic/ATAirlock sandbox escape & proof of concept underlying `AirliftFFI`.
- **[@SirMaku](https://github.com/SirMaku-git)** (Maintainer & Customizer): Thiết kế và phát triển Card Studio Đa Tầng, Custom Texture & Foil upload, Custom EMV Chip engine, layer alignment, và UI Sources.

### Credits & Acknowledgments
- Khai thác cốt lõi dựa trên `airlift` (AirTraffic sync escape).
- Định dạng theme dialer lấy cảm hứng từ [Cowabunga](https://github.com/leminlimez/Cowabunga) và [Nugget](https://github.com/leminlimez/Nugget).
- Dự án mod được hỗ trợ thực hiện bởi **AI (Google Antigravity / Gemini 3.8 Flash)**.
- **Lời tri ân**: Xin gửi lời cảm ơn trân trọng nhất đến tác giả **Lumid-Off** và toàn thể các nhà nghiên cứu đã đóng góp cho hệ sinh thái mở.

---

## 📄 License

Dự án được phân phối theo giấy phép [MIT License](LICENSE):
- Original work Copyright (c) 2026 Lumid-Off and contributors ([Lumid-Off/AirCard-Windows](https://github.com/Lumid-Off/AirCard-Windows))
- Modified work Copyright (c) 2026 SirMaku ([SirMaku-git/AirCard-CMaku](https://github.com/SirMaku-git/AirCard-CMaku))

Mọi hành vi tái phân phối hoặc phát triển tiếp nối phải tuân thủ điều khoản giấy phép MIT và bảo lưu đầy đủ thông báo bản quyền của tác giả gốc và người đóng góp bản mod.
