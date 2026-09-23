use eframe::egui;
use crate::app::md3;

fn m3_card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(md3::SURFACE_CONTAINER)
        .corner_radius(16)
        .inner_margin(egui::Margin::same(20))
        .show(ui, |ui| ui.vertical(add_contents).inner)
        .inner
}

pub fn draw_sources_tab(ui: &mut egui::Ui, is_vi: bool) {
    ui.columns(2, |cols| {
        let left = &mut cols[0];
        m3_card(left, |ui| {
            let title = if is_vi { "AirCard-CMaku (Phiên bản Windows)" } else { "AirCard-CMaku (Windows Edition)" };
            let subtitle = if is_vi {
                "Bản mod tùy biến dành cho Windows của AirCard Apple Wallet & Passcode Studio"
            } else {
                "Customized Native Windows port of the AirCard Apple Wallet & Passcode studio"
            };
            ui.label(egui::RichText::new(title).strong().size(16.0).color(md3::ON_SURFACE));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(subtitle).size(12.0).color(md3::ON_SURFACE_VARIANT));
            ui.add_space(14.0);

            // Author & GitHub
            let orig_title = if is_vi { "Tác giả gốc & Kho lưu trữ chính thức" } else { "Original Author & Official Repository" };
            ui.label(egui::RichText::new(orig_title).strong().size(12.0).color(md3::ON_SURFACE));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let author_lbl = if is_vi { "Tác giả:" } else { "Author:" };
                ui.label(egui::RichText::new(author_lbl).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("Lumid-Off").strong().size(11.5).color(md3::PRIMARY));
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("GitHub:").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.hyperlink_to(
                    egui::RichText::new("Lumid-Off/AirCard-Windows").size(11.5).color(md3::PRIMARY).underline(),
                    "https://github.com/Lumid-Off/AirCard-Windows",
                );
            });

            ui.add_space(14.0);

            // Customizer / Modder
            let mod_title = if is_vi { "Người tùy biến & Mod lại" } else { "Customized & Modded By" };
            ui.label(egui::RichText::new(mod_title).strong().size(12.0).color(md3::ON_SURFACE));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let mod_lbl = if is_vi { "Người mod / Customizer:" } else { "Modder / Customizer:" };
                ui.label(egui::RichText::new(mod_lbl).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new("SirMaku").strong().size(11.5).color(md3::PRIMARY));
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("GitHub:").size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.hyperlink_to(
                    egui::RichText::new("SirMaku-git").size(11.5).color(md3::PRIMARY).underline(),
                    "https://github.com/SirMaku-git",
                );
            });
            ui.add_space(2.0);
            let mod_features = if is_vi {
                "Mở rộng với Card Studio tương tác đa lớp, tùy biến chất liệu Foil/Texture, bộ engine gắn & định vị chip EMV, hệ thống căn chỉnh layer tự do và xem trước trực tiếp."
            } else {
                "Enhanced with Interactive Multi-Layer Card Studio, Custom Texture/Foil overlays, Custom EMV Chip engine, layer alignment controls, and UI improvements."
            };
            ui.label(
                egui::RichText::new(mod_features)
                    .size(11.0)
                    .color(md3::ON_SURFACE_VARIANT),
            );

            ui.add_space(12.0);

            // EXPLICIT SCOPE & SAFETY DISCLAIMER FRAME
            egui::Frame::new()
                .fill(md3::SURFACE_CONTAINER)
                .corner_radius(10)
                .inner_margin(egui::Margin::same(10))
                .stroke(egui::Stroke::new(1.0_f32, md3::PRIMARY))
                .show(ui, |ui| {
                    let scope_header = if is_vi {
                        "⚠️ LƯU Ý VỀ PHẠM VI TÙY BIẾN & AN TOÀN:"
                    } else {
                        "⚠️ SCOPE & SAFETY NOTICE:"
                    };
                    ui.label(
                        egui::RichText::new(scope_header)
                            .strong()
                            .size(11.5)
                            .color(md3::PRIMARY),
                    );
                    ui.add_space(4.0);
                    let scope_body = if is_vi {
                        "• SirMaku CHỈ tập trung tùy biến và mở rộng các tính năng liên quan đến Apple Pay / Apple Wallet (Card Studio).
• Tất cả cơ chế kỹ thuật cốt lõi khác — bao gồm exploit AirTraffic sync (airlift), kết nối thiết bị USB (usbmuxd), Books snapshot, và Passcode — KHÔNG bị can thiệp nhiều và hoàn toàn mang tính kế thừa nguyên bản từ tác giả gốc @Lumid-Off và các tác giả tiền nhiệm (@mak5er, 0xjohnny).
• An toàn & Sao lưu: Có cơ chế snapshot và restore để giảm rủi ro, nhưng không thể đảm bảo an toàn tuyệt đối. Người dùng nên sao lưu thiết bị trước khi sử dụng."
                    } else {
                        "• SirMaku ONLY customized and extended features related to Apple Pay / Apple Wallet (Card Studio).
• All other core technical mechanisms — including the AirTraffic sync exploit (airlift), USB usbmuxd communication, Books snapshot restore, and Passcode theming — were directly inherited from original author @Lumid-Off and upstream authors (@mak5er, 0xjohnny).
• Safety & Backup: Snapshot/restore reduces risk but cannot guarantee absolute safety. Always backup your device before use."
                    };
                    ui.label(
                        egui::RichText::new(scope_body)
                            .size(10.5)
                            .color(md3::ON_SURFACE),
                    );
                });

            ui.add_space(14.0);

            // Open-source license, takedown policy & attribution notice
            egui::Frame::new()
                .fill(md3::SURFACE_CONTAINER_HIGH)
                .corner_radius(12)
                .inner_margin(egui::Margin::same(12))
                .stroke(egui::Stroke::new(1.0_f32, md3::OUTLINE))
                .show(ui, |ui| {
                    let notice_header = if is_vi {
                        "📜 Bản Quyền, Miễn Trừ Trách Nhiệm & Tri Ân"
                    } else {
                        "📜 License, Disclaimers & Acknowledgments"
                    };
                    ui.label(
                        egui::RichText::new(notice_header)
                            .strong()
                            .size(11.5)
                            .color(md3::PRIMARY),
                    );
                    ui.add_space(6.0);
                    let notice_rules = if is_vi {
                        "• Mục đích: Dự án phát triển phục vụ học tập, nghiên cứu và cá nhân. Người dùng tự chịu trách nhiệm khi sử dụng.
• Quan hệ tác giả & Gỡ bỏ thiện chí: SirMaku không có liên kết với tác giả gốc. Nếu tác giả gốc có yêu cầu gỡ bỏ, kho lưu trữ sẽ được gỡ bỏ ngay lập tức khỏi GitHub và chỉ lưu hành nội bộ phục vụ bạn bè.
• Quan hệ với Apple: Công cụ độc lập, phi chính thức, không được Apple tài trợ, xác nhận hay bảo đảm.
• Bản quyền (MIT): Nền tảng gốc thuộc Lumid-Off & cộng tác viên; phần tiện ích Card Studio thuộc SirMaku. Bắt buộc bảo lưu thông báo bản quyền khi chia sẻ."
                    } else {
                        "• Purpose: Developed strictly for personal study and technical research. Users assume all responsibility.
• Upstream & Takedown: SirMaku has no affiliation with original authors. If requested by upstream authors, this repository will be promptly removed from GitHub and kept private for friends.
• Apple Non-Affiliation: Unofficial independent tool, not affiliated with or endorsed by Apple Inc.
• License (MIT): Original platform belongs to Lumid-Off & contributors; Card Studio mod belongs to SirMaku. Standard MIT attribution applies."
                    };
                    ui.label(
                        egui::RichText::new(notice_rules)
                            .size(10.5)
                            .color(md3::ON_SURFACE),
                    );
                });
        });

        let right = &mut cols[1];
        m3_card(right, |ui| {
            let r_title = if is_vi { "Người đóng góp & Lời cảm ơn" } else { "Contributors & Credits" };
            let r_subtitle = if is_vi {
                "Ghi nhận công lao của các tác giả và nhà nghiên cứu bảo mật"
            } else {
                "Credits to the original creators and security researchers"
            };
            ui.label(egui::RichText::new(r_title).strong().size(16.0).color(md3::ON_SURFACE));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(r_subtitle).size(12.0).color(md3::ON_SURFACE_VARIANT));
            ui.add_space(14.0);

            let contrib_hdr = if is_vi { "Người đóng góp" } else { "Contributors" };
            ui.label(egui::RichText::new(contrib_hdr).strong().size(12.0).color(md3::ON_SURFACE));
            ui.add_space(6.0);

            // Lumid-Off
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("• @Lumid-Off").strong().size(11.5).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new(if is_vi { "(Windows Native Rust Port & Duy trì)" } else { "(Windows Native Rust Port & Maintainer)" }).size(11.0).color(md3::ON_SURFACE_VARIANT));
            });
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.hyperlink_to(egui::RichText::new("GitHub").size(11.0).color(md3::PRIMARY).underline(), "https://github.com/Lumid-Off");
                ui.label(egui::RichText::new("·").size(11.0).color(md3::OUTLINE_VARIANT));
                ui.hyperlink_to(egui::RichText::new("Twitter / X").size(11.0).color(md3::PRIMARY).underline(), "https://x.com/LumidOff");
            });

            ui.add_space(8.0);

            // mak5er
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("• @mak5er").strong().size(11.5).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new(if is_vi { "(Ứng dụng macOS gốc & Nghiên cứu exploit)" } else { "(Original macOS App & Exploit Research)" }).size(11.0).color(md3::ON_SURFACE_VARIANT));
            });
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.hyperlink_to(egui::RichText::new("GitHub").size(11.0).color(md3::PRIMARY).underline(), "https://github.com/mak5er");
                ui.label(egui::RichText::new("·").size(11.0).color(md3::OUTLINE_VARIANT));
                ui.hyperlink_to(egui::RichText::new("Twitter / X").size(11.0).color(md3::PRIMARY).underline(), "https://x.com/mak5er");
            });

            ui.add_space(8.0);

            // AirLift / 0xjohnny
            ui.horizontal(|ui| {
                ui.hyperlink_to(egui::RichText::new("• AirLift").strong().size(11.5).color(md3::PRIMARY).underline(), "https://github.com/0xjohnnydev/airlift");
                ui.label(egui::RichText::new(if is_vi { "bởi" } else { "by" }).size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.hyperlink_to(egui::RichText::new("0xjohnny (@0xjohnnydev)").strong().size(11.0).color(md3::PRIMARY).underline(), "https://github.com/0xjohnnydev");
            });
            let poc_desc = if is_vi {
                "  Mã nguồn gốc vượt sandbox AirTraffic/ATAirlock nền tảng cho AirliftFFI."
            } else {
                "  Original AirTraffic/ATAirlock sandbox escape and proof of concept underlying AirliftFFI."
            };
            ui.label(
                egui::RichText::new(poc_desc)
                    .size(10.5)
                    .color(md3::ON_SURFACE_VARIANT),
            );

            ui.add_space(16.0);

            let credits_hdr = if is_vi { "Ghi nhận khác" } else { "Credits" };
            ui.label(egui::RichText::new(credits_hdr).strong().size(12.0).color(md3::ON_SURFACE));
            ui.add_space(6.0);
            let cred1 = if is_vi {
                "• Exploit cốt lõi dựa trên airlift (vượt cơ chế đồng bộ AirTraffic)."
            } else {
                "• Core exploit based on airlift (AirTraffic sync escape)."
            };
            ui.label(egui::RichText::new(cred1).size(11.0).color(md3::ON_SURFACE_VARIANT));
            ui.horizontal(|ui| {
                let cred2 = if is_vi { "• Định dạng theme lấy cảm hứng từ" } else { "• Theme format inspired by" };
                ui.label(egui::RichText::new(cred2).size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.hyperlink_to(egui::RichText::new("Cowabunga").size(11.0).color(md3::PRIMARY).underline(), "https://github.com/leminlimez/Cowabunga");
                ui.label(egui::RichText::new(if is_vi { "và" } else { "and" }).size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.hyperlink_to(egui::RichText::new("Nugget").size(11.0).color(md3::PRIMARY).underline(), "https://github.com/leminlimez/Nugget");
                ui.label(egui::RichText::new(".").size(11.0).color(md3::ON_SURFACE_VARIANT));
            });

            ui.add_space(8.0);
            let thanks_txt = if is_vi {
                "💖 Lời tri ân: Xin gửi lời cảm ơn chân thành và sâu sắc nhất đến tác giả gốc @Lumid-Off cùng cộng đồng mã nguồn mở đã tạo dựng nền tảng xuất sắc này."
            } else {
                "💖 Acknowledgments: Sincere gratitude to original creator @Lumid-Off and the open-source community for building this excellent platform."
            };
            ui.label(egui::RichText::new(thanks_txt).size(10.5).color(md3::PRIMARY));
        });
    });
}
