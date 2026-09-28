use eframe::egui;
use crate::app::md3;
use crate::card_studio::types::*;
use crate::card_studio::CardStudioState;

fn m3_button_tonal(ui: &mut egui::Ui, label: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label).size(13.0).color(md3::ON_SECONDARY_CONTAINER),
    )
    .fill(md3::SECONDARY_CONTAINER)
    .corner_radius(20)
    .stroke(egui::Stroke::NONE);
    ui.add(btn).clicked()
}

fn studio_accordion_card<R>(
    ui: &mut egui::Ui,
    id_source: &str,
    title: &str,
    default_open: bool,
    badge: Option<&str>,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    let id = ui.make_persistent_id(id_source);
    let mut is_open = ui.data_mut(|d| d.get_temp::<bool>(id).unwrap_or(default_open));

    let frame = egui::Frame::NONE
        .fill(if is_open { md3::SURFACE_CONTAINER } else { md3::SURFACE })
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .stroke(egui::Stroke::new(
            1.0_f32,
            if is_open { md3::OUTLINE_VARIANT } else { egui::Color32::from_white_alpha(18) },
        ));

    let mut result = None;

    frame.show(ui, |ui| {
        let arrow_char = if is_open { "▼" } else { "▶" };
        let mut toggle = false;

        ui.horizontal(|ui| {
            let btn = egui::Button::new(
                egui::RichText::new(arrow_char).size(11.0).color(md3::PRIMARY).strong(),
            )
            .fill(egui::Color32::TRANSPARENT)
            .corner_radius(4.0);

            if ui.add(btn).clicked() {
                toggle = true;
            }

            let resp = ui.add(
                egui::Label::new(
                    egui::RichText::new(title).strong().size(12.0).color(md3::ON_SURFACE),
                )
                .sense(egui::Sense::click()),
            );
            if resp.clicked() {
                toggle = true;
            }

            if let Some(b) = badge {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(b).size(10.5).color(md3::ON_SURFACE_VARIANT));
                });
            }
        });

        if toggle {
            is_open = !is_open;
            ui.data_mut(|d| d.insert_temp(id, is_open));
        }

        if is_open {
            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);
            result = Some(body(ui));
        }
    });

    ui.add_space(4.0);
    result
}

pub fn draw_studio_sidebar(state: &mut CardStudioState, ui: &mut egui::Ui, is_vi: bool) -> bool {
    let mut overlay_changed = false;

    // Top Header: Title and Reset button
    ui.horizontal(|ui| {
        let studio_lbl = if is_vi { "Card Studio & Tùy biến" } else { "Card Studio & Customization" };
        ui.label(egui::RichText::new(studio_lbl).strong().size(12.0).color(md3::ON_SURFACE));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let reset_def_lbl = if is_vi { "↺ Khôi phục mặc định" } else { "↺ Reset to Default" };
            let reset_def_hover = if is_vi { "Khôi phục toàn bộ tùy biến thẻ, lớp phủ và logo về mặc định" } else { "Reset all card customizations, overlays, and logos to default" };
            if ui.button(egui::RichText::new(reset_def_lbl).size(10.5).color(md3::PRIMARY))
                .on_hover_text(reset_def_hover)
                .clicked()
            {
                state.reset_to_defaults();
                overlay_changed = true;
            }
        });
    });
    ui.add_space(4.0);

    // SECTION 1: Workspace AirCard (.wcm)
    studio_accordion_card(
        ui,
        "studio_sec_workspace",
        if is_vi { "💾 Quản Lý Workspace (.wcm)" } else { "💾 Workspace Manager (.wcm)" },
        false,
        Some(if is_vi { "Lưu / Mở" } else { "Save / Open" }),
        |ui| {
            let hint = if is_vi {
                "Lưu toàn bộ bố cục thẻ, ảnh nền, logo, chip, texture, font chữ và các widget vào 1 file .wcm để tải lại bất cứ lúc nào."
            } else {
                "Save complete card layout, artwork, logo, chip, finish, font, and widgets to a .wcm file to reload anytime."
            };
            ui.label(egui::RichText::new(hint).size(10.5).color(md3::ON_SURFACE_VARIANT));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let save_txt = if is_vi { "💾 Lưu Workspace (.wcm)..." } else { "💾 Save Workspace (.wcm)..." };
                if m3_button_tonal(ui, save_txt) {
                    state.save_workspace_requested = true;
                }
                let load_txt = if is_vi { "📂 Mở Workspace (.wcm)..." } else { "📂 Open Workspace (.wcm)..." };
                if m3_button_tonal(ui, load_txt) {
                    state.load_workspace_requested = true;
                }
            });
        },
    );

    // SECTION 2: Background Preset & Finish
    let bg_name = state.overlay_options.bg_preset.display_name_lang(is_vi);
    studio_accordion_card(
        ui,
        "studio_sec_bg_finish",
        if is_vi { "🖼 Nền Thẻ & Phủ Bề Mặt (Finish)" } else { "🖼 Background & Surface Finish" },
        true,
        Some(bg_name),
        |ui| {
            ui.horizontal(|ui| {
                let bg_lbl = if is_vi { "Nền:" } else { "Background:" };
                ui.label(egui::RichText::new(bg_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                let cur_bg = state.overlay_options.bg_preset;
                egui::ComboBox::from_id_salt("bg_preset_select")
                    .width(155.0)
                    .selected_text(egui::RichText::new(cur_bg.display_name_lang(is_vi)).size(11.0).color(md3::ON_SURFACE))
                    .show_ui(ui, |ui| {
                        for preset in [
                            CardBackgroundPreset::CustomImage,
                            CardBackgroundPreset::MatteBlack,
                            CardBackgroundPreset::OceanNavy,
                            CardBackgroundPreset::BrushedGold,
                            CardBackgroundPreset::EmeraldLuxury,
                            CardBackgroundPreset::TitaniumMinimal,
                            CardBackgroundPreset::CrimsonVelvet,
                            CardBackgroundPreset::DeepCyberViolet,
                        ] {
                            let is_sel = cur_bg == preset;
                            if ui.selectable_label(is_sel, preset.display_name_lang(is_vi)).clicked() {
                                if state.overlay_options.bg_preset != preset {
                                    state.overlay_options.bg_preset = preset;
                                    overlay_changed = true;
                                }
                            }
                        }
                    });

                let finish_lbl = if is_vi { "Chất liệu:" } else { "Finish:" };
                ui.label(egui::RichText::new(finish_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                let cur_finish = state.overlay_options.finish;
                egui::ComboBox::from_id_salt("finish_select")
                    .width(130.0)
                    .selected_text(egui::RichText::new(cur_finish.display_name_lang(is_vi)).size(11.0).color(md3::ON_SURFACE))
                    .show_ui(ui, |ui| {
                        for finish in [
                            CardFinish::Standard,
                            CardFinish::MetallicSheen,
                            CardFinish::CarbonWeave,
                            CardFinish::CustomTexture,
                        ] {
                            let is_sel = cur_finish == finish;
                            if ui.selectable_label(is_sel, finish.display_name_lang(is_vi)).clicked() {
                                if state.overlay_options.finish != finish {
                                    state.overlay_options.finish = finish;
                                    if finish == CardFinish::CustomTexture {
                                        state.active_layer = ActiveTransformLayer::Finish;
                                    }
                                    overlay_changed = true;
                                }
                            }
                        }
                    });
            });

            if state.overlay_options.finish == CardFinish::CustomTexture || state.custom_finish_image.is_some() {
                ui.add_space(3.0);
                ui.horizontal(|ui| {
                    let tex_title = if let Some(path) = &state.custom_finish_path {
                        format!("Texture: {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("texture.png"))
                    } else {
                        if is_vi { "Tải lên Texture / Foil (PNG, SVG, JPG)...".to_string() } else { "Upload Texture / Foil...".to_string() }
                    };
                    if m3_button_tonal(ui, &tex_title) {
                        if state.select_custom_finish().is_ok() {
                            overlay_changed = true;
                        }
                    }
                    if state.custom_finish_image.is_some() {
                        let clear_txt = if is_vi { "Xóa" } else { "Clear" };
                        if ui.button(egui::RichText::new(clear_txt).size(11.0).color(md3::ERROR)).clicked() {
                            state.custom_finish_image = None;
                            state.custom_finish_path = None;
                            state.overlay_options.finish = CardFinish::Standard;
                            overlay_changed = true;
                        }
                    }
                });

                ui.horizontal(|ui| {
                    let op_lbl = if is_vi { "Độ mờ:" } else { "Opacity:" };
                    ui.label(egui::RichText::new(op_lbl).size(10.5).color(md3::ON_SURFACE_VARIANT));
                    let mut op_pct = (state.overlay_options.finish_opacity * 100.0).round() as i32;
                    if ui.add(egui::Slider::new(&mut op_pct, 5..=100).suffix("%")).changed() {
                        state.overlay_options.finish_opacity = (op_pct as f32 / 100.0).clamp(0.05, 1.0);
                        overlay_changed = true;
                    }

                    let sc_lbl = if is_vi { "Tỉ lệ:" } else { "Scale:" };
                    ui.label(egui::RichText::new(sc_lbl).size(10.5).color(md3::ON_SURFACE_VARIANT));
                    let mut sc_pct = (state.overlay_options.finish_scale * 100.0).round() as i32;
                    if ui.add(egui::Slider::new(&mut sc_pct, 20..=500).suffix("%")).changed() {
                        state.overlay_options.finish_scale = (sc_pct as f32 / 100.0).clamp(0.1, 10.0);
                        overlay_changed = true;
                    }

                    let reset_tex_lbl = if is_vi { "↺ Đặt lại" } else { "↺ Reset" };
                    if ui.button(egui::RichText::new(reset_tex_lbl).size(10.5).color(md3::PRIMARY)).clicked() {
                        state.overlay_options.finish_x = 0.0;
                        state.overlay_options.finish_y = 0.0;
                        state.overlay_options.finish_scale = 1.0;
                        state.overlay_options.finish_opacity = 0.60;
                        overlay_changed = true;
                    }
                });
            }
        },
    );

    // SECTION 3: Brand & Logo
    let brand_status = state.overlay_options.network.display_name_lang(is_vi);
    let logo_open = state.overlay_options.network != PaymentNetwork::None;
    studio_accordion_card(
        ui,
        "studio_sec_brand",
        if is_vi { "🏷 Thương Hiệu & Logo" } else { "🏷 Brand & Logo" },
        logo_open,
        Some(brand_status),
        |ui| {
            ui.horizontal(|ui| {
                let brand_lbl = if is_vi { "Thương hiệu:" } else { "Brand:" };
                ui.label(egui::RichText::new(brand_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                let cur_net = state.overlay_options.network;
                egui::ComboBox::from_id_salt("network_select")
                    .width(95.0)
                    .selected_text(egui::RichText::new(cur_net.display_name_lang(is_vi)).size(11.0).color(md3::ON_SURFACE))
                    .show_ui(ui, |ui| {
                        for net in [
                            PaymentNetwork::None,
                            PaymentNetwork::Custom,
                        ] {
                            let is_sel = cur_net == net;
                            if ui.selectable_label(is_sel, net.display_name_lang(is_vi)).clicked() {
                                if state.overlay_options.network != net {
                                    state.overlay_options.network = net;
                                    if net != PaymentNetwork::None {
                                        state.active_layer = ActiveTransformLayer::Logo;
                                    }
                                    overlay_changed = true;
                                }
                            }
                        }
                    });

                ui.label(egui::RichText::new("Frame:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                let cur_badge = state.overlay_options.logo_style;
                egui::ComboBox::from_id_salt("logo_badge_select")
                    .width(115.0)
                    .selected_text(egui::RichText::new(cur_badge.display_name()).size(11.0).color(md3::ON_SURFACE))
                    .show_ui(ui, |ui| {
                        for style in [
                            LogoBadgeStyle::Transparent,
                            LogoBadgeStyle::ThinOutline,
                            LogoBadgeStyle::FrostedGlass,
                            LogoBadgeStyle::SolidDark,
                            LogoBadgeStyle::SolidLight,
                            LogoBadgeStyle::SubtleGlow,
                        ] {
                            let is_sel = cur_badge == style;
                            if ui.selectable_label(is_sel, style.display_name_lang(is_vi)).clicked() {
                                if state.overlay_options.logo_style != style {
                                    state.overlay_options.logo_style = style;
                                    overlay_changed = true;
                                }
                            }
                        }
                    });

                let color_lbl = if is_vi { "Màu sắc:" } else { "Color:" };
                ui.label(egui::RichText::new(color_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                let cur_color = state.overlay_options.logo_color;
                egui::ComboBox::from_id_salt("logo_color_select")
                    .width(110.0)
                    .selected_text(egui::RichText::new(cur_color.display_name_lang(is_vi)).size(11.0).color(md3::ON_SURFACE))
                    .show_ui(ui, |ui| {
                        for theme in [
                            LogoColorTheme::Original,
                            LogoColorTheme::MonochromeWhite,
                            LogoColorTheme::LuxuryGold,
                            LogoColorTheme::SilverPlatinum,
                            LogoColorTheme::StealthBlack,
                        ] {
                            let is_sel = cur_color == theme;
                            if ui.selectable_label(is_sel, theme.display_name_lang(is_vi)).clicked() {
                                if state.overlay_options.logo_color != theme {
                                    state.overlay_options.logo_color = theme;
                                    overlay_changed = true;
                                }
                            }
                        }
                    });
            });

            if state.overlay_options.network == PaymentNetwork::Custom || state.custom_logo_image.is_some() {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let btn_title = if let Some(path) = &state.custom_logo_path {
                        format!("Custom: {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("logo.png"))
                    } else {
                        if is_vi { "Tải Logo riêng (PNG, SVG, JPG)...".to_string() } else { "Upload Custom Logo...".to_string() }
                    };
                    if m3_button_tonal(ui, &btn_title) {
                        if state.select_custom_logo().is_ok() {
                            overlay_changed = true;
                        }
                    }
                    if state.custom_logo_image.is_some() {
                        let clear_logo = if is_vi { "Xóa" } else { "Clear" };
                        if ui.button(egui::RichText::new(clear_logo).size(11.0).color(md3::ERROR)).clicked() {
                            state.custom_logo_image = None;
                            state.custom_logo_path = None;
                            state.overlay_options.network = PaymentNetwork::None;
                            overlay_changed = true;
                        }
                    }
                });
            }

            if state.overlay_options.network != PaymentNetwork::None {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    let logo_sc_lbl = if is_vi { "Tỉ lệ Logo:" } else { "Logo Scale:" };
                    ui.label(egui::RichText::new(logo_sc_lbl).size(10.5).color(md3::ON_SURFACE_VARIANT));
                    let mut logo_sc_pct = (state.overlay_options.logo_scale * 100.0).round() as i32;
                    if ui.add(egui::Slider::new(&mut logo_sc_pct, 30..=250).suffix("%")).changed() {
                        state.overlay_options.logo_scale = (logo_sc_pct as f32 / 100.0).clamp(0.2, 5.0);
                        overlay_changed = true;
                    }

                    ui.label(egui::RichText::new("X:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                    if ui.add(egui::DragValue::new(&mut state.overlay_options.logo_x).range(0.0..=1500.0).speed(1.0)).changed() {
                        overlay_changed = true;
                    }

                    ui.label(egui::RichText::new("Y:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                    if ui.add(egui::DragValue::new(&mut state.overlay_options.logo_y).range(0.0..=1000.0).speed(1.0)).changed() {
                        overlay_changed = true;
                    }

                    let reset_logo_lbl = if is_vi { "↺ Đặt lại" } else { "↺ Reset" };
                    if ui.button(egui::RichText::new(reset_logo_lbl).size(10.5).color(md3::PRIMARY)).clicked() {
                        state.overlay_options.logo_x = 1205.0;
                        state.overlay_options.logo_y = 800.0;
                        state.overlay_options.logo_scale = 1.0;
                        overlay_changed = true;
                    }
                });
            }
        },
    );

    // SECTION 4: EMV Chip & Contactless Wave
    let chip_badge = if state.overlay_options.show_chip { "Chip: Bật" } else { "Chip: Tắt" };
    let chip_open = state.overlay_options.show_chip || state.overlay_options.show_contactless;
    studio_accordion_card(
        ui,
        "studio_sec_chip",
        if is_vi { "💳 Chip EMV & Sóng Contactless" } else { "💳 EMV Chip & Contactless Wave" },
        chip_open,
        Some(chip_badge),
        |ui| {
            ui.horizontal(|ui| {
                let chip_cb_lbl = if is_vi { "Chip EMV kim loại" } else { "EMV Gold Chip" };
                if ui.checkbox(&mut state.overlay_options.show_chip, egui::RichText::new(chip_cb_lbl).size(11.5).color(md3::ON_SURFACE)).changed() {
                    if state.overlay_options.show_chip {
                        state.active_layer = ActiveTransformLayer::Chip;
                    }
                    overlay_changed = true;
                }
                let wave_cb_lbl = if is_vi { "Sóng Contactless" } else { "Contactless Wave" };
                if ui.checkbox(&mut state.overlay_options.show_contactless, egui::RichText::new(wave_cb_lbl).size(11.5).color(md3::ON_SURFACE)).changed() {
                    if state.overlay_options.show_contactless {
                        state.active_layer = ActiveTransformLayer::Wave;
                    }
                    overlay_changed = true;
                }
            });

            if state.overlay_options.show_chip {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    let chip_title = if let Some(path) = &state.custom_chip_path {
                        format!("Chip: {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("chip.png"))
                    } else {
                        if is_vi { "Tải Chip riêng (PNG, SVG, JPG)...".to_string() } else { "Upload Custom Chip...".to_string() }
                    };
                    if m3_button_tonal(ui, &chip_title) {
                        if state.select_custom_chip().is_ok() {
                            overlay_changed = true;
                        }
                    }
                    if state.custom_chip_image.is_some() {
                        let def_gold_lbl = if is_vi { "Dùng Chip vàng mặc định" } else { "Use Default Gold" };
                        if ui.button(egui::RichText::new(def_gold_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT)).clicked() {
                            state.custom_chip_image = None;
                            state.custom_chip_path = None;
                            overlay_changed = true;
                        }
                    }
                });

                ui.horizontal(|ui| {
                    let chip_sc_lbl = if is_vi { "Tỉ lệ Chip:" } else { "Chip Scale:" };
                    ui.label(egui::RichText::new(chip_sc_lbl).size(10.5).color(md3::ON_SURFACE_VARIANT));
                    let mut chip_sc_pct = (state.overlay_options.chip_scale * 100.0).round() as i32;
                    if ui.add(egui::Slider::new(&mut chip_sc_pct, 30..=250).suffix("%")).changed() {
                        state.overlay_options.chip_scale = (chip_sc_pct as f32 / 100.0).clamp(0.2, 5.0);
                        overlay_changed = true;
                    }

                    ui.label(egui::RichText::new("X:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                    if ui.add(egui::DragValue::new(&mut state.overlay_options.chip_x).range(0.0..=1500.0).speed(1.0)).changed() {
                        overlay_changed = true;
                    }

                    ui.label(egui::RichText::new("Y:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                    if ui.add(egui::DragValue::new(&mut state.overlay_options.chip_y).range(0.0..=1000.0).speed(1.0)).changed() {
                        overlay_changed = true;
                    }

                    if ui.button(egui::RichText::new("↺ Reset").size(10.5).color(md3::PRIMARY)).clicked() {
                        state.overlay_options.chip_x = 188.0;
                        state.overlay_options.chip_y = 398.0;
                        state.overlay_options.chip_scale = 1.0;
                        state.overlay_options.chip_adj = LayerAdjustments::default();
                        overlay_changed = true;
                    }
                });
            }

            if state.overlay_options.show_contactless {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    let wave_sc_lbl = if is_vi { "Tỉ lệ Sóng:" } else { "Wave Scale:" };
                    ui.label(egui::RichText::new(wave_sc_lbl).size(10.5).color(md3::ON_SURFACE_VARIANT));
                    let mut wave_sc_pct = (state.overlay_options.wave_scale * 100.0).round() as i32;
                    if ui.add(egui::Slider::new(&mut wave_sc_pct, 30..=250).suffix("%")).changed() {
                        state.overlay_options.wave_scale = (wave_sc_pct as f32 / 100.0).clamp(0.2, 5.0);
                        overlay_changed = true;
                    }

                    ui.label(egui::RichText::new("X:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                    if ui.add(egui::DragValue::new(&mut state.overlay_options.wave_x).range(0.0..=1500.0).speed(1.0)).changed() {
                        overlay_changed = true;
                    }

                    ui.label(egui::RichText::new("Y:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                    if ui.add(egui::DragValue::new(&mut state.overlay_options.wave_y).range(0.0..=1000.0).speed(1.0)).changed() {
                        overlay_changed = true;
                    }

                    if ui.button(egui::RichText::new("↺ Reset").size(10.5).color(md3::PRIMARY)).clicked() {
                        state.overlay_options.wave_x = 375.0;
                        state.overlay_options.wave_y = 375.0;
                        state.overlay_options.wave_scale = 1.0;
                        state.overlay_options.wave_adj = LayerAdjustments::default();
                        overlay_changed = true;
                    }
                });
            }
        },
    );

    // SECTION 5: Card Details & Typography
    state.overlay_options.details.ensure_items();
    let num_items = state.overlay_options.details.items.len();
    let text_badge = format!("{} dòng chữ", num_items);
    studio_accordion_card(
        ui,
        "studio_sec_details",
        if is_vi { "📝 Thông Tin Thẻ & Quản Lý Font" } else { "📝 Card Text & Typography" },
        true,
        Some(&text_badge),
        |ui| {
            let details_cb_lbl = if is_vi { "Dập nổi thông tin thẻ" } else { "Emboss Card Details" };
            if ui.checkbox(&mut state.overlay_options.details.show_details, egui::RichText::new(details_cb_lbl).size(11.5).color(md3::PRIMARY)).changed() {
                overlay_changed = true;
            }

            if state.overlay_options.details.show_details {
                ui.add_space(4.0);

                // Row 1: Global Style, Backdrop, Offset, Scale
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Style:").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    let cur_emboss = state.overlay_options.details.emboss_style;
                    egui::ComboBox::from_id_salt("emboss_style_select")
                        .width(120.0)
                        .selected_text(egui::RichText::new(cur_emboss.display_name()).size(11.0).color(md3::ON_SURFACE))
                        .show_ui(ui, |ui| {
                            for style in [
                                EmbossStyle::EmbossedSilver,
                                EmbossStyle::EmbossedGold,
                                EmbossStyle::CrispWhite,
                                EmbossStyle::StealthDark,
                            ] {
                                let is_sel = cur_emboss == style;
                                if ui.selectable_label(is_sel, style.display_name_lang(is_vi)).clicked() {
                                    if state.overlay_options.details.emboss_style != style {
                                        state.overlay_options.details.emboss_style = style;
                                        overlay_changed = true;
                                    }
                                }
                            }
                        });

                    let backdrop_lbl = if is_vi { "Nền chữ:" } else { "Backdrop:" };
                    ui.label(egui::RichText::new(backdrop_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    let cur_backdrop = state.overlay_options.details.backdrop;
                    egui::ComboBox::from_id_salt("text_backdrop_select")
                        .width(130.0)
                        .selected_text(egui::RichText::new(cur_backdrop.display_name_lang(is_vi)).size(11.0).color(md3::ON_SURFACE))
                        .show_ui(ui, |ui| {
                            for b in [
                                TextBackdropStyle::None,
                                TextBackdropStyle::FrostedGlassStrip,
                                TextBackdropStyle::SubtleDarkGradient,
                                TextBackdropStyle::FrostedPills,
                            ] {
                                let is_sel = cur_backdrop == b;
                                if ui.selectable_label(is_sel, b.display_name_lang(is_vi)).clicked() {
                                    if state.overlay_options.details.backdrop != b {
                                        state.overlay_options.details.backdrop = b;
                                        overlay_changed = true;
                                    }
                                }
                            }
                        });

                    let ypos_lbl = if is_vi { "Độ lệch Y:" } else { "Offset Y:" };
                    ui.label(egui::RichText::new(ypos_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    if ui.add(egui::DragValue::new(&mut state.overlay_options.details.vertical_offset).range(-250..=250).speed(1.0)).changed() {
                        overlay_changed = true;
                    }

                    let xpos_lbl = if is_vi { "X:" } else { "X:" };
                    ui.label(egui::RichText::new(xpos_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    if ui.add(egui::DragValue::new(&mut state.overlay_options.details.horizontal_offset).range(-350..=350).speed(1.0)).changed() {
                        overlay_changed = true;
                    }

                    let sc_lbl = if is_vi { "Cỡ tổng:" } else { "Scale:" };
                    ui.label(egui::RichText::new(sc_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    let mut sc_pct = (state.overlay_options.details.scale * 100.0).round() as i32;
                    if ui.add(egui::Slider::new(&mut sc_pct, 40..=250).suffix("%")).changed() {
                        state.overlay_options.details.scale = (sc_pct as f32 / 100.0).clamp(0.4, 2.5);
                        overlay_changed = true;
                    }
                });

                ui.add_space(6.0);

                // PROMINENT FONT MANAGEMENT CARD
                egui::Frame::NONE
                    .fill(md3::SURFACE_CONTAINER_HIGH)
                    .corner_radius(6.0)
                    .inner_margin(8.0)
                    .stroke(egui::Stroke::new(1.0_f32, md3::PRIMARY))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(if is_vi { "🔤 QUẢN LÝ FONT CHỮ (Typography)" } else { "🔤 TYPOGRAPHY & FONTS" }).strong().size(11.5).color(md3::PRIMARY));
                        });
                        ui.add_space(3.0);

                        let active_font_name = state.custom_font_name.clone();
                        if let Some(name) = active_font_name {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("✅ Font đang dùng: {}", name)).strong().size(11.0).color(md3::ON_SURFACE));
                                if ui.button(egui::RichText::new(if is_vi { "📂 Đổi font..." } else { "📂 Change..." }).size(10.5).color(md3::PRIMARY)).clicked() {
                                    if state.load_custom_font().is_ok() {
                                        overlay_changed = true;
                                    }
                                }
                                if ui.button(egui::RichText::new("✕").size(10.5).color(md3::ERROR))
                                    .on_hover_text(if is_vi { "Gỡ bỏ font tùy chỉnh này" } else { "Remove custom font" })
                                    .clicked()
                                {
                                    state.clear_custom_font();
                                    overlay_changed = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new(if is_vi { "⚡ Áp dụng font này cho tất cả dòng chữ" } else { "⚡ Apply to all text fields" }).size(10.5).color(md3::PRIMARY))
                                    .on_hover_text(if is_vi { "Đặt tất cả các dòng chữ bên dưới dùng font này" } else { "Set all text fields below to use this custom font" })
                                    .clicked()
                                {
                                    state.apply_custom_font_to_all_items();
                                    overlay_changed = true;
                                }
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(if is_vi { "ℹ Font: Hệ thống (Segoe UI / OCR)" } else { "ℹ Font: System default" }).size(10.5).color(md3::ON_SURFACE_VARIANT));
                                if ui.button(egui::RichText::new(if is_vi { "📁 Tải lên font tùy chỉnh (.ttf, .otf)..." } else { "📁 Upload custom font (.ttf, .otf)..." }).size(11.0).color(md3::PRIMARY).strong())
                                    .on_hover_text(if is_vi { "Chọn file font TTF hoặc OTF bất kỳ từ máy tính của bạn" } else { "Select any TTF or OTF font file from your PC" })
                                    .clicked()
                                {
                                    if state.load_custom_font().is_ok() {
                                        overlay_changed = true;
                                    }
                                }
                            });
                        }
                        ui.label(egui::RichText::new(if is_vi {
                            "💡 Tải font lên tại đây. Tất cả các dòng chữ sẽ tự động dùng font mới, hoặc bạn có thể chỉnh riêng từng dòng thành font khác tùy ý."
                        } else {
                            "💡 Upload your font here. All text fields will automatically switch to it, or you can pick presets per-item."
                        }).size(10.0).color(md3::ON_SURFACE_VARIANT));
                    });

                ui.add_space(6.0);

                // Section header for modular text fields
                ui.horizontal(|ui| {
                    let sec_hdr = if is_vi { "📝 Các dòng chữ trên thẻ" } else { "📝 Card Text Fields" };
                    ui.label(egui::RichText::new(sec_hdr).strong().size(11.5).color(md3::ON_SURFACE));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let add_lbl = if is_vi { "➕ Thêm dòng chữ" } else { "➕ Add Text Field" };
                        let add_btn = egui::Button::new(
                            egui::RichText::new(add_lbl).size(10.5).color(md3::PRIMARY).strong()
                        )
                        .fill(md3::SURFACE_CONTAINER_HIGH)
                        .corner_radius(12)
                        .stroke(egui::Stroke::new(1.0_f32, md3::PRIMARY));

                        if ui.add(add_btn)
                            .on_hover_text(if is_vi { "Thêm một dòng chữ mới tùy chỉnh lên thẻ" } else { "Add a new customizable text field to the card" })
                            .clicked()
                        {
                            state.add_text_item();
                            overlay_changed = true;
                        }
                    });
                });
                ui.add_space(4.0);

                // Modular text items list
                let mut remove_idx = None;
                let items_len = state.overlay_options.details.items.len();
                for idx in 0..items_len {
                    let is_selected = state.active_layer == ActiveTransformLayer::Details && state.selected_text_index == idx;
                    let item = &mut state.overlay_options.details.items[idx];

                    let frame_fill = if is_selected {
                        md3::SURFACE_CONTAINER_HIGH
                    } else {
                        md3::SURFACE_CONTAINER
                    };
                    let frame_stroke = if is_selected {
                        egui::Stroke::new(1.0_f32, md3::PRIMARY)
                    } else {
                        egui::Stroke::new(0.5_f32, md3::OUTLINE_VARIANT)
                    };

                    egui::Frame::NONE
                        .fill(frame_fill)
                        .corner_radius(6.0)
                        .inner_margin(6.0)
                        .stroke(frame_stroke)
                        .show(ui, |ui| {
                            // Row 1: Select Chip, Visibility, Label, Uppercase, Frosted Pill toggle, Delete
                            ui.horizontal(|ui| {
                                let sel_btn_lbl = format!("#{}", idx + 1);
                                if ui.selectable_label(is_selected, sel_btn_lbl).clicked() {
                                    state.active_layer = ActiveTransformLayer::Details;
                                    state.selected_text_index = idx;
                                }

                                if ui.checkbox(&mut item.visible, "").on_hover_text(if is_vi { "Ẩn/hiện dòng chữ này" } else { "Show/hide this text item" }).changed() {
                                    overlay_changed = true;
                                }

                                ui.label(egui::RichText::new(if is_vi { "Nhãn:" } else { "Label:" }).size(10.5).color(md3::ON_SURFACE_VARIANT));
                                if ui.add(egui::TextEdit::singleline(&mut item.label).desired_width(90.0)).changed() {
                                    overlay_changed = true;
                                }

                                let case_btn = if item.is_uppercase { "🔠 HOA" } else { "🔡 Thường" };
                                if ui.button(egui::RichText::new(case_btn).size(10.0).color(md3::ON_SURFACE_VARIANT))
                                    .on_hover_text(if is_vi { "Chuyển đổi VIẾT HOA / Chữ thường" } else { "Toggle UPPERCASE / Normal" })
                                    .clicked()
                                {
                                    item.is_uppercase = !item.is_uppercase;
                                    overlay_changed = true;
                                }

                                if ui.checkbox(&mut item.has_backdrop, if is_vi { "🧊 Nền mờ" } else { "🧊 Pill" })
                                    .on_hover_text(if is_vi { "Bật/tắt hộp nền mờ bo góc riêng cho dòng chữ này" } else { "Toggle frosted pill background for this text line" })
                                    .changed()
                                {
                                    overlay_changed = true;
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(egui::RichText::new("🗑").size(11.0).color(md3::ERROR))
                                        .on_hover_text(if is_vi { "Xóa dòng chữ này" } else { "Delete this text field" })
                                        .clicked()
                                    {
                                        remove_idx = Some(idx);
                                    }
                                });
                            });

                            ui.add_space(2.0);

                            // Row 2: Text content input
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(if is_vi { "Chữ:" } else { "Text:" }).size(10.5).color(md3::ON_SURFACE_VARIANT));
                                if ui.add(egui::TextEdit::singleline(&mut item.content).desired_width(280.0)).changed() {
                                    overlay_changed = true;
                                }
                            });

                            ui.add_space(2.0);

                            // Row 3: Font preset, Font size, X, Y, Letter spacing
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_salt(format!("font_sel_{}", item.id))
                                    .width(115.0)
                                    .selected_text(egui::RichText::new(item.font.display_name_lang(is_vi)).size(10.0).color(md3::ON_SURFACE))
                                    .show_ui(ui, |ui| {
                                        for f in [
                                            CardFontPreset::ClassicOcr,
                                            CardFontPreset::ModernSans,
                                            CardFontPreset::Monospace,
                                            CardFontPreset::SerifLuxury,
                                            CardFontPreset::Custom,
                                        ] {
                                            if ui.selectable_label(item.font == f, f.display_name_lang(is_vi)).clicked() {
                                                item.font = f;
                                                overlay_changed = true;
                                            }
                                        }
                                    });

                                ui.label(egui::RichText::new("Cỡ:").size(10.0).color(md3::ON_SURFACE_VARIANT));
                                if ui.add(egui::DragValue::new(&mut item.font_size).range(10.0..=120.0).speed(0.5)).changed() {
                                    overlay_changed = true;
                                }

                                ui.label(egui::RichText::new("X:").size(10.0).color(md3::ON_SURFACE_VARIANT));
                                if ui.add(egui::DragValue::new(&mut item.x).range(-100.0..=1600.0).speed(1.0)).changed() {
                                    overlay_changed = true;
                                }

                                ui.label(egui::RichText::new("Y:").size(10.0).color(md3::ON_SURFACE_VARIANT));
                                if ui.add(egui::DragValue::new(&mut item.y).range(-50.0..=1000.0).speed(1.0)).changed() {
                                    overlay_changed = true;
                                }

                                ui.label(egui::RichText::new("Dãn:").size(10.0).color(md3::ON_SURFACE_VARIANT));
                                if ui.add(egui::DragValue::new(&mut item.letter_spacing).range(-2.0..=20.0).speed(0.1)).changed() {
                                    overlay_changed = true;
                                }
                            });
                        });
                    ui.add_space(2.0);
                }

                if let Some(rm_idx) = remove_idx {
                    state.remove_text_item(rm_idx);
                    overlay_changed = true;
                }
            }
        },
    );

    // SECTION 6: Custom Widgets
    let num_w = state.custom_widgets.len();
    let w_badge = format!("{} widget", num_w);
    let w_open = !state.custom_widgets.is_empty();
    studio_accordion_card(
        ui,
        "studio_sec_widgets",
        if is_vi { "📦 Widget Tùy Chọn (Custom Widgets)" } else { "📦 Custom Widgets" },
        w_open,
        Some(&w_badge),
        |ui| {
            ui.horizontal(|ui| {
                let add_btn_text = if is_vi { "➕ Thêm Widget (PNG, SVG, JPG)..." } else { "➕ Add Widget..." };
                if ui.button(egui::RichText::new(add_btn_text).size(10.5).color(md3::PRIMARY).strong())
                    .on_hover_text(if is_vi { "Tải lên ảnh PNG, SVG, JPG hoặc WebP bất kỳ làm widget trên thẻ" } else { "Upload any image as a card widget" })
                    .clicked()
                {
                    if state.add_custom_widget().is_ok() {
                        overlay_changed = true;
                    }
                }
            });

            if state.custom_widgets.is_empty() {
                let empty_hint = if is_vi {
                    "Chưa có widget nào. Nhấn \"➕ Thêm Widget...\" để tải lên hình ảnh bất kỳ (logo, sticker, chip, QR, chữ...)"
                } else {
                    "No custom widgets added yet. Click \"➕ Add Widget...\" to upload any image (logo, sticker, custom chip, QR, badge...)"
                };
                ui.label(egui::RichText::new(empty_hint).size(10.5).color(md3::ON_SURFACE_VARIANT));
            } else {
                let mut to_remove = None;
                for (idx, w) in state.custom_widgets.iter_mut().enumerate() {
                    ui.add_space(4.0);
                    let is_active = state.active_layer == ActiveTransformLayer::Widget(idx);
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut w.data.visible, "").changed() {
                            overlay_changed = true;
                        }

                        let btn = egui::Button::new(
                            egui::RichText::new(format!("🧩 {}", w.data.name))
                                .size(11.0)
                                .color(if is_active { md3::ON_PRIMARY_CONTAINER } else { md3::ON_SURFACE })
                                .strong(),
                        )
                        .fill(if is_active { md3::PRIMARY_CONTAINER } else { egui::Color32::TRANSPARENT })
                        .corner_radius(6.0);
                        if ui.add(btn).clicked() {
                            state.active_layer = ActiveTransformLayer::Widget(idx);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(egui::RichText::new("🗑").size(11.0).color(md3::ERROR))
                                .on_hover_text(if is_vi { "Xóa widget này" } else { "Delete this widget" })
                                .clicked()
                            {
                                to_remove = Some(idx);
                            }
                        });
                    });

                    if is_active {
                        ui.horizontal(|ui| {
                            let sc_lbl = if is_vi { "Tỉ lệ:" } else { "Scale:" };
                            ui.label(egui::RichText::new(sc_lbl).size(10.0).color(md3::ON_SURFACE_VARIANT));
                            let mut sc_pct = (w.data.scale * 100.0).round() as i32;
                            if ui.add(egui::Slider::new(&mut sc_pct, 10..=400).suffix("%")).changed() {
                                w.data.scale = (sc_pct as f32 / 100.0).clamp(0.1, 10.0);
                                overlay_changed = true;
                            }

                            ui.label(egui::RichText::new("X:").size(10.0).color(md3::ON_SURFACE_VARIANT));
                            if ui.add(egui::DragValue::new(&mut w.data.x).range(0.0..=1536.0).speed(1.0)).changed() {
                                overlay_changed = true;
                            }

                            ui.label(egui::RichText::new("Y:").size(10.0).color(md3::ON_SURFACE_VARIANT));
                            if ui.add(egui::DragValue::new(&mut w.data.y).range(0.0..=969.0).speed(1.0)).changed() {
                                overlay_changed = true;
                            }
                        });
                    }
                }
                if let Some(idx) = to_remove {
                    state.remove_custom_widget(idx);
                    overlay_changed = true;
                }
            }
        },
    );

    overlay_changed
}


pub fn draw_studio_preview(
    state: &mut CardStudioState,
    ui: &mut egui::Ui,
    skin_texture: Option<&egui::TextureHandle>,
    is_vi: bool,
) -> bool {
    let mut preview_changed = false;

    // Multi-layer control: Layer Selector Bar
    ui.horizontal_wrapped(|ui| {
        let layer_hdr = if is_vi { "Lớp:" } else { "Layer:" };
        ui.label(egui::RichText::new(layer_hdr).strong().size(11.0).color(md3::ON_SURFACE));
        let layer_buttons: &[(ActiveTransformLayer, &str)] = if is_vi {
            &[
                (ActiveTransformLayer::Background, "🖼 Nền"),
                (ActiveTransformLayer::Finish, "✨ Phủ bề mặt (Finish)"),
                (ActiveTransformLayer::Chip, "💳 Chip"),
                (ActiveTransformLayer::Wave, "📶 Sóng"),
                (ActiveTransformLayer::Logo, "🏷 Logo"),
                (ActiveTransformLayer::Details, "🔢 Chữ / Thông tin"),
            ]
        } else {
            &[
                (ActiveTransformLayer::Background, "🖼 Background"),
                (ActiveTransformLayer::Finish, "✨ Surface Finish"),
                (ActiveTransformLayer::Chip, "💳 Chip"),
                (ActiveTransformLayer::Wave, "📶 Wave"),
                (ActiveTransformLayer::Logo, "🏷 Logo"),
                (ActiveTransformLayer::Details, "🔢 Card Text"),
            ]
        };
        for &(layer, icon_label) in layer_buttons {
            let is_active = state.active_layer == layer;
            let btn = egui::Button::new(
                egui::RichText::new(icon_label)
                    .size(11.0)
                    .color(if is_active { md3::ON_PRIMARY_CONTAINER } else { md3::ON_SURFACE_VARIANT })
                    .strong(),
            )
            .fill(if is_active { md3::PRIMARY_CONTAINER } else { egui::Color32::TRANSPARENT })
            .corner_radius(12)
            .stroke(egui::Stroke::new(1.0_f32, if is_active { md3::PRIMARY } else { md3::OUTLINE_VARIANT }));

            let hover_hint = if layer == ActiveTransformLayer::Finish {
                if is_vi { "Lớp phủ bề mặt: nhám, ánh kim, vân carbon hoặc texture riêng (Surface Finish)" } else { "Surface finish layer: matte, metallic, carbon, or custom texture" }
            } else if layer == ActiveTransformLayer::Details {
                if is_vi { "Thông tin chữ trên thẻ: số thẻ, tên, ngày hết hạn, ngân hàng hoặc text tùy ý" } else { "Card text fields: number, name, expiry, bank or custom text" }
            } else {
                ""
            };

            let mut resp = ui.add(btn);
            if !hover_hint.is_empty() {
                resp = resp.on_hover_text(hover_hint);
            }
            if resp.clicked() {
                state.active_layer = layer;
            }
        }

        for (idx, w) in state.custom_widgets.iter().enumerate() {
            let is_active = state.active_layer == ActiveTransformLayer::Widget(idx);
            let w_name = if w.data.name.len() > 8 {
                format!("🧩 {}..", &w.data.name[..8])
            } else {
                format!("🧩 {}", w.data.name)
            };
            let btn = egui::Button::new(
                egui::RichText::new(w_name)
                    .size(11.0)
                    .color(if is_active { md3::ON_PRIMARY_CONTAINER } else { md3::ON_SURFACE_VARIANT })
                    .strong(),
            )
            .fill(if is_active { md3::PRIMARY_CONTAINER } else { egui::Color32::TRANSPARENT })
            .corner_radius(12)
            .stroke(egui::Stroke::new(1.0_f32, if is_active { md3::PRIMARY } else { md3::OUTLINE_VARIANT }));

            if ui.add(btn).clicked() {
                state.active_layer = ActiveTransformLayer::Widget(idx);
            }
        }

        let add_layer_btn = egui::Button::new(
            egui::RichText::new("➕")
                .size(11.0)
                .color(md3::PRIMARY)
                .strong(),
        )
        .fill(md3::SURFACE_CONTAINER_HIGH)
        .corner_radius(12)
        .stroke(egui::Stroke::new(1.0_f32, md3::PRIMARY));

        if ui.add(add_layer_btn)
            .on_hover_text(if is_vi { "Thêm widget tùy chỉnh mới (Upload PNG/JPG/WebP)" } else { "Add new custom widget (Upload PNG/JPG/WebP)" })
            .clicked()
        {
            if state.add_custom_widget().is_ok() {
                preview_changed = true;
            }
        }
    });

    ui.add_space(10.0);

    let pass_w = (ui.available_width() - 8.0).clamp(250.0, 400.0);
    let pass_h = pass_w * (969.0 / 1536.0);
    let corner_r = pass_w * (58.0 / 1536.0);

    ui.vertical_centered(|ui| {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(pass_w, pass_h), egui::Sense::click_and_drag());
        let canvas_w = 1536.0_f32;
        let canvas_to_preview = pass_w / canvas_w;

        // Direct hit-testing: click directly on text item, widget, chip, or logo to select that layer
        if let Some(pos) = response.hover_pos() {
            if rect.contains(pos) && (response.clicked() || response.drag_started()) {
                let cx = (pos.x - rect.left()) / canvas_to_preview;
                let cy = (pos.y - rect.top()) / canvas_to_preview;

                // 1. Text items hit test (checked first if details are enabled)
                let mut text_hit = None;
                if state.overlay_options.details.show_details {
                    state.overlay_options.details.ensure_items();
                    let ho = state.overlay_options.details.horizontal_offset as f32;
                    let vo = state.overlay_options.details.vertical_offset as f32;
                    let scale = if state.overlay_options.details.scale > 0.05 { state.overlay_options.details.scale } else { 1.0 };
                    for (idx, item) in state.overlay_options.details.items.iter().enumerate().rev() {
                        if !item.visible || item.content.is_empty() {
                            continue;
                        }
                        let item_x = item.x + ho;
                        let item_y = item.y + vo;
                        let font_sz = item.font_size * scale;
                        let font = crate::card_studio::render::resolve_font(item.font, state.custom_font_bytes.as_deref(), false);
                        let text = if item.is_uppercase { item.content.to_uppercase() } else { item.content.clone() };
                        let (_, offset_y, w, h) = crate::card_studio::render::calculate_text_bounds(&font, &text, font_sz, item.letter_spacing * scale);
                        let pad = 6.0;
                        let text_rect = egui::Rect::from_min_size(
                            egui::pos2(item_x - pad, item_y + offset_y - pad),
                            egui::vec2(w + pad * 2.0, h + pad * 2.0),
                        );
                        if text_rect.contains(egui::pos2(cx, cy)) {
                            text_hit = Some(idx);
                            break;
                        }
                    }
                }

                // 2. Custom widgets hit test
                let mut widget_hit = None;
                for (idx, w) in state.custom_widgets.iter().enumerate().rev() {
                    if !w.data.visible {
                        continue;
                    }
                    let rw = w.data.base_w * w.data.scale;
                    let rh = w.data.base_h * w.data.scale;
                    let w_rect = egui::Rect::from_center_size(
                        egui::pos2(w.data.x, w.data.y),
                        egui::vec2(rw, rh),
                    );
                    if w_rect.contains(egui::pos2(cx, cy)) {
                        widget_hit = Some(idx);
                        break;
                    }
                }

                // 3. Chip hit test
                let chip_scale = state.overlay_options.chip_scale.clamp(0.2, 5.0);
                let chip_bw = 205.0 * chip_scale;
                let chip_bh = 155.0 * chip_scale;
                let chip_cx = state.overlay_options.chip_x + chip_bw * 0.5;
                let chip_cy = state.overlay_options.chip_y + chip_bh * 0.5;
                let (chip_draw_w, chip_draw_h) = if let Some(chip) = &state.custom_chip_image {
                    let cw = chip.width() as f32;
                    let ch = chip.height() as f32;
                    if cw > 0.0 && ch > 0.0 {
                        let aspect = cw / ch;
                        let target_aspect = 205.0 / 155.0;
                        if aspect > target_aspect {
                            (205.0 * chip_scale, (205.0 / aspect) * chip_scale)
                        } else {
                            ((155.0 * aspect) * chip_scale, 155.0 * chip_scale)
                        }
                    } else {
                        (chip_bw, chip_bh)
                    }
                } else {
                    (chip_bw, chip_bh)
                };
                let chip_rect = egui::Rect::from_center_size(
                    egui::pos2(chip_cx, chip_cy),
                    egui::vec2(chip_draw_w, chip_draw_h),
                );

                // 4. Logo hit test
                let logo_scale = state.overlay_options.logo_scale.clamp(0.2, 5.0);
                let logo_bw = 245.0 * logo_scale;
                let logo_bh = 94.0 * logo_scale;
                let logo_cx = state.overlay_options.logo_x + logo_bw * 0.5;
                let logo_cy = state.overlay_options.logo_y + logo_bh * 0.5;
                let (logo_draw_w, logo_draw_h) = if state.overlay_options.logo_style == LogoBadgeStyle::Transparent {
                    if let Some(logo) = &state.custom_logo_image {
                        let lw = logo.width() as f32;
                        let lh = logo.height() as f32;
                        if lw > 0.0 && lh > 0.0 {
                            let aspect = lw / lh;
                            let target_aspect = 245.0 / 94.0;
                            if aspect > target_aspect {
                                (245.0 * logo_scale, (245.0 / aspect) * logo_scale)
                            } else {
                                ((94.0 * aspect) * logo_scale, 94.0 * logo_scale)
                            }
                        } else {
                            (logo_bw, logo_bh)
                        }
                    } else {
                        (logo_bw, logo_bh)
                    }
                } else {
                    (logo_bw, logo_bh)
                };
                let logo_rect = egui::Rect::from_center_size(
                    egui::pos2(logo_cx, logo_cy),
                    egui::vec2(logo_draw_w, logo_draw_h),
                );

                // 5. Wave hit test
                let wave_scale = state.overlay_options.wave_scale.clamp(0.2, 5.0);
                let wave_bw = 75.0 * wave_scale;
                let wave_bh = 95.0 * wave_scale;
                let wave_cx = state.overlay_options.wave_x + wave_bw * 0.5;
                let wave_cy = state.overlay_options.wave_y + wave_bh * 0.5;
                let wave_rect = egui::Rect::from_center_size(
                    egui::pos2(wave_cx, wave_cy),
                    egui::vec2(wave_bw, wave_bh),
                );

                if let Some(txt_idx) = text_hit {
                    state.active_layer = ActiveTransformLayer::Details;
                    state.selected_text_index = txt_idx;
                } else if let Some(idx) = widget_hit {
                    state.active_layer = ActiveTransformLayer::Widget(idx);
                } else if state.overlay_options.show_chip && chip_rect.contains(egui::pos2(cx, cy)) {
                    state.active_layer = ActiveTransformLayer::Chip;
                } else if state.overlay_options.show_contactless && wave_rect.contains(egui::pos2(cx, cy)) {
                    state.active_layer = ActiveTransformLayer::Wave;
                } else if state.overlay_options.network != PaymentNetwork::None && logo_rect.contains(egui::pos2(cx, cy)) {
                    state.active_layer = ActiveTransformLayer::Logo;
                } else if state.overlay_options.finish == CardFinish::CustomTexture && state.active_layer == ActiveTransformLayer::Finish {
                    // Keep finish selected
                } else if !matches!(state.active_layer, ActiveTransformLayer::Widget(_)) && state.active_layer != ActiveTransformLayer::Finish {
                    state.active_layer = ActiveTransformLayer::Background;
                }
            }
        }

        if response.dragged_by(egui::PointerButton::Primary) {
            let delta = response.drag_delta();
            if delta.x != 0.0 || delta.y != 0.0 {
                let scale_factor = 1536.0 / pass_w;
                match state.active_layer {
                    ActiveTransformLayer::Background => {
                        state.overlay_options.transform.pan_x += delta.x * scale_factor;
                        state.overlay_options.transform.pan_y += delta.y * scale_factor;
                    }
                    ActiveTransformLayer::Finish => {
                        state.overlay_options.finish_x += delta.x * scale_factor;
                        state.overlay_options.finish_y += delta.y * scale_factor;
                    }
                    ActiveTransformLayer::Chip => {
                        state.overlay_options.chip_x += delta.x * scale_factor;
                        state.overlay_options.chip_y += delta.y * scale_factor;
                    }
                    ActiveTransformLayer::Wave => {
                        state.overlay_options.wave_x += delta.x * scale_factor;
                        state.overlay_options.wave_y += delta.y * scale_factor;
                    }
                    ActiveTransformLayer::Logo => {
                        state.overlay_options.logo_x += delta.x * scale_factor;
                        state.overlay_options.logo_y += delta.y * scale_factor;
                    }
                    ActiveTransformLayer::Details => {
                        state.overlay_options.details.ensure_items();
                        if let Some(item) = state.overlay_options.details.items.get_mut(state.selected_text_index) {
                            item.x = (item.x + delta.x * scale_factor).clamp(-200.0, 1600.0);
                            item.y = (item.y + delta.y * scale_factor).clamp(-100.0, 1100.0);
                        }
                    }
                    ActiveTransformLayer::Widget(idx) => {
                        if let Some(w) = state.custom_widgets.get_mut(idx) {
                            w.data.x += delta.x * scale_factor;
                            w.data.y += delta.y * scale_factor;
                        }
                    }
                }
                preview_changed = true;
            }
        }

        if response.hovered() {
            let (scroll_x, scroll_y, shift_down) = ui.input(|i| {
                let y = if i.smooth_scroll_delta.y.abs() > 0.001 {
                    i.smooth_scroll_delta.y
                } else {
                    i.raw_scroll_delta.y
                };
                let x = if i.smooth_scroll_delta.x.abs() > 0.001 {
                    i.smooth_scroll_delta.x
                } else {
                    i.raw_scroll_delta.x
                };
                (x, y, i.modifiers.shift)
            });

            if shift_down {
                let rot_input = if scroll_x.abs() > scroll_y.abs() { scroll_x } else { scroll_y };
                if rot_input.abs() > 0.001 {
                    let rot_delta = if rot_input > 0.0 { 3.0 } else { -3.0 };
                    match state.active_layer {
                        ActiveTransformLayer::Background => {
                            state.overlay_options.transform.rotation = (state.overlay_options.transform.rotation + rot_delta).clamp(-180.0, 180.0);
                            state.overlay_options.bg_adj.rotation = state.overlay_options.transform.rotation;
                        }
                        ActiveTransformLayer::Finish => {
                            state.overlay_options.finish_adj.rotation = (state.overlay_options.finish_adj.rotation + rot_delta).clamp(-180.0, 180.0);
                        }
                        ActiveTransformLayer::Chip => {
                            state.overlay_options.chip_adj.rotation = (state.overlay_options.chip_adj.rotation + rot_delta).clamp(-180.0, 180.0);
                        }
                        ActiveTransformLayer::Wave => {
                            state.overlay_options.wave_adj.rotation = (state.overlay_options.wave_adj.rotation + rot_delta).clamp(-180.0, 180.0);
                        }
                        ActiveTransformLayer::Logo => {
                            state.overlay_options.logo_adj.rotation = (state.overlay_options.logo_adj.rotation + rot_delta).clamp(-180.0, 180.0);
                        }
                        ActiveTransformLayer::Details => {
                            state.overlay_options.details.ensure_items();
                            if let Some(item) = state.overlay_options.details.items.get_mut(state.selected_text_index) {
                                let old_spacing = item.letter_spacing;
                                let spacing_delta = if rot_delta > 0.0 { 0.25 } else { -0.25 };
                                item.letter_spacing = (old_spacing + spacing_delta).clamp(-2.0, 20.0);
                            }
                        }
                        ActiveTransformLayer::Widget(idx) => {
                            if let Some(w) = state.custom_widgets.get_mut(idx) {
                                w.data.adjustments.rotation = (w.data.adjustments.rotation + rot_delta).clamp(-180.0, 180.0);
                            }
                        }
                    }
                    preview_changed = true;
                }
            } else {
                let scroll_input = if scroll_y.abs() > scroll_x.abs() { scroll_y } else { scroll_x };
                if scroll_input.abs() > 0.001 {
                    let factor = if scroll_input > 0.0 { 1.08 } else { 1.0 / 1.08 };
                    match state.active_layer {
                        ActiveTransformLayer::Background => {
                            let old_zoom = state.overlay_options.transform.zoom;
                            let new_zoom = (old_zoom * factor).clamp(0.05, 50.0);
                            if (new_zoom - old_zoom).abs() > 0.001 {
                                state.overlay_options.transform.zoom = new_zoom;
                                preview_changed = true;
                            }
                        }
                        ActiveTransformLayer::Finish => {
                            let old_scale = state.overlay_options.finish_scale;
                            let new_scale = (old_scale * factor).clamp(0.1, 10.0);
                            if (new_scale - old_scale).abs() > 0.001 {
                                state.overlay_options.finish_scale = new_scale;
                                preview_changed = true;
                            }
                        }
                        ActiveTransformLayer::Chip => {
                            let old_scale = state.overlay_options.chip_scale;
                            let new_scale = (old_scale * factor).clamp(0.2, 5.0);
                            if (new_scale - old_scale).abs() > 0.001 {
                                state.overlay_options.chip_scale = new_scale;
                                preview_changed = true;
                            }
                        }
                        ActiveTransformLayer::Wave => {
                            let old_scale = state.overlay_options.wave_scale;
                            let new_scale = (old_scale * factor).clamp(0.2, 5.0);
                            if (new_scale - old_scale).abs() > 0.001 {
                                state.overlay_options.wave_scale = new_scale;
                                preview_changed = true;
                            }
                        }
                        ActiveTransformLayer::Logo => {
                            let old_scale = state.overlay_options.logo_scale;
                            let new_scale = (old_scale * factor).clamp(0.2, 5.0);
                            if (new_scale - old_scale).abs() > 0.001 {
                                state.overlay_options.logo_scale = new_scale;
                                preview_changed = true;
                            }
                        }
                        ActiveTransformLayer::Details => {
                            state.overlay_options.details.ensure_items();
                            if let Some(item) = state.overlay_options.details.items.get_mut(state.selected_text_index) {
                                let old_size = item.font_size;
                                let new_size = (old_size * factor).clamp(10.0, 150.0);
                                if (new_size - old_size).abs() > 0.1 {
                                    item.font_size = new_size;
                                    preview_changed = true;
                                }
                            }
                        }
                        ActiveTransformLayer::Widget(idx) => {
                            if let Some(w) = state.custom_widgets.get_mut(idx) {
                                let old_scale = w.data.scale;
                                let new_scale = (old_scale * factor).clamp(0.1, 10.0);
                                if (new_scale - old_scale).abs() > 0.001 {
                                    w.data.scale = new_scale;
                                    preview_changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        if response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            ui.ctx().request_repaint();
        } else if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }

        let painter = ui.painter();
        if let Some(tex) = skin_texture {
            painter.rect_filled(rect.translate(egui::vec2(2.0, 5.0)), corner_r, egui::Color32::from_black_alpha(100));
            painter.rect_filled(rect.translate(egui::vec2(1.0, 2.0)), corner_r, egui::Color32::from_black_alpha(60));

            painter.image(tex.id(), rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE);
            painter.rect_stroke(rect, corner_r,
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_premultiplied(255, 255, 255, 45)),
                egui::StrokeKind::Inside);

            match state.active_layer {
                ActiveTransformLayer::Chip if state.overlay_options.show_chip => {
                    let scale = state.overlay_options.chip_scale.clamp(0.2, 5.0);
                    let bw = 205.0 * scale;
                    let bh = 155.0 * scale;
                    let center_x = state.overlay_options.chip_x + bw * 0.5;
                    let center_y = state.overlay_options.chip_y + bh * 0.5;
                    let (draw_w, draw_h) = if let Some(chip) = &state.custom_chip_image {
                        let cw = chip.width() as f32;
                        let ch = chip.height() as f32;
                        if cw > 0.0 && ch > 0.0 {
                            let aspect = cw / ch;
                            let target_aspect = 205.0 / 155.0;
                            if aspect > target_aspect {
                                (205.0 * scale, (205.0 / aspect) * scale)
                            } else {
                                ((155.0 * aspect) * scale, 155.0 * scale)
                            }
                        } else {
                            (bw, bh)
                        }
                    } else {
                        (bw, bh)
                    };
                    let cx = rect.left() + (center_x - draw_w * 0.5) * canvas_to_preview;
                    let cy = rect.top() + (center_y - draw_h * 0.5) * canvas_to_preview;
                    let cw = draw_w * canvas_to_preview;
                    let ch = draw_h * canvas_to_preview;
                    let chip_box = egui::Rect::from_min_size(egui::pos2(cx, cy), egui::vec2(cw, ch));
                    painter.rect_stroke(
                        chip_box,
                        4.0,
                        egui::Stroke::new(1.5_f32, md3::PRIMARY),
                        egui::StrokeKind::Outside,
                    );
                }
                ActiveTransformLayer::Wave if state.overlay_options.show_contactless => {
                    let scale = state.overlay_options.wave_scale.clamp(0.2, 5.0);
                    let bw = 75.0 * scale;
                    let bh = 95.0 * scale;
                    let center_x = state.overlay_options.wave_x + bw * 0.5;
                    let center_y = state.overlay_options.wave_y + bh * 0.5;
                    let wx = rect.left() + (center_x - bw * 0.5) * canvas_to_preview;
                    let wy = rect.top() + (center_y - bh * 0.5) * canvas_to_preview;
                    let ww = bw * canvas_to_preview;
                    let wh = bh * canvas_to_preview;
                    let wave_box = egui::Rect::from_min_size(egui::pos2(wx, wy), egui::vec2(ww, wh));
                    painter.rect_stroke(
                        wave_box,
                        4.0,
                        egui::Stroke::new(1.5_f32, md3::PRIMARY),
                        egui::StrokeKind::Outside,
                    );
                }
                ActiveTransformLayer::Logo if state.overlay_options.network != PaymentNetwork::None => {
                    let scale = state.overlay_options.logo_scale.clamp(0.2, 5.0);
                    let bw = 245.0 * scale;
                    let bh = 94.0 * scale;
                    let center_x = state.overlay_options.logo_x + bw * 0.5;
                    let center_y = state.overlay_options.logo_y + bh * 0.5;
                    let (draw_w, draw_h) = if state.overlay_options.logo_style == LogoBadgeStyle::Transparent {
                        if let Some(logo) = &state.custom_logo_image {
                            let lw = logo.width() as f32;
                            let lh = logo.height() as f32;
                            if lw > 0.0 && lh > 0.0 {
                                let aspect = lw / lh;
                                let target_aspect = 245.0 / 94.0;
                                if aspect > target_aspect {
                                    (245.0 * scale, (245.0 / aspect) * scale)
                                } else {
                                    ((94.0 * aspect) * scale, 94.0 * scale)
                                }
                            } else {
                                (bw, bh)
                            }
                        } else {
                            (bw, bh)
                        }
                    } else {
                        (bw, bh)
                    };
                    let lx = rect.left() + (center_x - draw_w * 0.5) * canvas_to_preview;
                    let ly = rect.top() + (center_y - draw_h * 0.5) * canvas_to_preview;
                    let lw = draw_w * canvas_to_preview;
                    let lh = draw_h * canvas_to_preview;
                    let logo_box = egui::Rect::from_min_size(egui::pos2(lx, ly), egui::vec2(lw, lh));
                    painter.rect_stroke(
                        logo_box,
                        4.0,
                        egui::Stroke::new(1.5_f32, md3::PRIMARY),
                        egui::StrokeKind::Outside,
                    );
                }
                ActiveTransformLayer::Details if state.overlay_options.details.show_details => {
                    state.overlay_options.details.ensure_items();
                    let ho = state.overlay_options.details.horizontal_offset as f32;
                    let vo = state.overlay_options.details.vertical_offset as f32;
                    let scale = if state.overlay_options.details.scale > 0.05 { state.overlay_options.details.scale } else { 1.0 };
                    for (i, item) in state.overlay_options.details.items.iter().enumerate() {
                        if !item.visible || item.content.is_empty() {
                            continue;
                        }
                        let item_x = item.x + ho;
                        let item_y = item.y + vo;
                        let font_sz = item.font_size * scale;
                        let font = crate::card_studio::render::resolve_font(item.font, state.custom_font_bytes.as_deref(), false);
                        let text = if item.is_uppercase { item.content.to_uppercase() } else { item.content.clone() };
                        let (_, offset_y, w, h) = crate::card_studio::render::calculate_text_bounds(&font, &text, font_sz, item.letter_spacing * scale);
                        let bx = rect.left() + item_x * canvas_to_preview;
                        let by = rect.top() + (item_y + offset_y) * canvas_to_preview;
                        let bw = w * canvas_to_preview;
                        let bh = h * canvas_to_preview;
                        let pad = 2.5;
                        let box_rect = egui::Rect::from_min_size(
                            egui::pos2(bx - pad, by - pad),
                            egui::vec2(bw + pad * 2.0, bh + pad * 2.0),
                        );
                        if i == state.selected_text_index {
                            painter.rect_stroke(
                                box_rect,
                                3.0,
                                egui::Stroke::new(1.5_f32, md3::PRIMARY),
                                egui::StrokeKind::Outside,
                            );
                        } else {
                            painter.rect_stroke(
                                box_rect,
                                2.0,
                                egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_premultiplied(200, 200, 255, 40)),
                                egui::StrokeKind::Outside,
                            );
                        }
                    }
                }
                ActiveTransformLayer::Finish if state.overlay_options.finish == CardFinish::CustomTexture => {
                    painter.rect_stroke(
                        rect,
                        corner_r,
                        egui::Stroke::new(2.0_f32, md3::PRIMARY),
                        egui::StrokeKind::Inside,
                    );
                }
                ActiveTransformLayer::Widget(idx) => {
                    if let Some(w) = state.custom_widgets.get(idx) {
                        if w.data.visible {
                            let wx = rect.left() + w.data.x * canvas_to_preview;
                            let wy = rect.top() + w.data.y * canvas_to_preview;
                            let ww = w.data.base_w * w.data.scale * canvas_to_preview;
                            let wh = w.data.base_h * w.data.scale * canvas_to_preview;
                            let widget_box = egui::Rect::from_center_size(egui::pos2(wx, wy), egui::vec2(ww, wh));
                            painter.rect_stroke(
                                widget_box,
                                4.0,
                                egui::Stroke::new(1.5_f32, md3::PRIMARY),
                                egui::StrokeKind::Outside,
                            );
                        }
                    }
                }
                _ => {}
            }
        } else {
            painter.rect_filled(rect.translate(egui::vec2(1.0, 3.0)), corner_r, egui::Color32::from_black_alpha(50));
            painter.rect_filled(rect, corner_r, md3::SURFACE_CONTAINER_HIGH);
            let no_art_lbl = if is_vi { "Chưa nạp ảnh thẻ" } else { "No artwork loaded" };
            painter.text(rect.center(), egui::Align2::CENTER_CENTER,
                no_art_lbl, egui::FontId::proportional(14.0), md3::ON_SURFACE_VARIANT);
        }
    });

    ui.add_space(8.0);

    // -----------------------------------------------------------
    // Interactive Layer Inspector
    // -----------------------------------------------------------
    let active_layer = state.active_layer;
    let layer_name_title = if is_vi {
        match active_layer {
            ActiveTransformLayer::Background => "🎨 Tuỳ biến Lớp: Nền thẻ (Background)".to_string(),
            ActiveTransformLayer::Finish => "🎨 Tuỳ biến Lớp: Finish Surface".to_string(),
            ActiveTransformLayer::Chip => "🎨 Tuỳ biến Lớp: Chip EMV".to_string(),
            ActiveTransformLayer::Wave => "🎨 Tuỳ biến Lớp: Sóng Contactless (Shockwave)".to_string(),
            ActiveTransformLayer::Logo => "🎨 Tuỳ biến Lớp: Logo Thương hiệu".to_string(),
            ActiveTransformLayer::Details => "🎨 Tuỳ biến Lớp: Thông tin dập nổi (Details)".to_string(),
            ActiveTransformLayer::Widget(idx) => {
                if let Some(w) = state.custom_widgets.get(idx) {
                    format!("🎨 Tuỳ biến Widget: {}", w.data.name)
                } else {
                    "🎨 Tuỳ biến Widget".to_string()
                }
            }
        }
    } else {
        match active_layer {
            ActiveTransformLayer::Background => "🎨 Layer Inspector: Background".to_string(),
            ActiveTransformLayer::Finish => "🎨 Layer Inspector: Finish Surface".to_string(),
            ActiveTransformLayer::Chip => "🎨 Layer Inspector: EMV Chip".to_string(),
            ActiveTransformLayer::Wave => "🎨 Layer Inspector: Contactless Wave".to_string(),
            ActiveTransformLayer::Logo => "🎨 Layer Inspector: Brand Logo".to_string(),
            ActiveTransformLayer::Details => "🎨 Layer Inspector: Card Details".to_string(),
            ActiveTransformLayer::Widget(idx) => {
                if let Some(w) = state.custom_widgets.get(idx) {
                    format!("🎨 Widget Inspector: {}", w.data.name)
                } else {
                    "🎨 Widget Inspector".to_string()
                }
            }
        }
    };

    let mut adj = match active_layer {
        ActiveTransformLayer::Background => state.overlay_options.bg_adj,
        ActiveTransformLayer::Finish => state.overlay_options.finish_adj,
        ActiveTransformLayer::Chip => state.overlay_options.chip_adj,
        ActiveTransformLayer::Wave => state.overlay_options.wave_adj,
        ActiveTransformLayer::Logo => state.overlay_options.logo_adj,
        ActiveTransformLayer::Details => state.overlay_options.details_adj,
        ActiveTransformLayer::Widget(idx) => {
            state.custom_widgets.get(idx).map(|w| w.data.adjustments).unwrap_or_default()
        }
    };

    let mut inspector_changed = false;

    egui::Frame::NONE
        .fill(md3::SURFACE_CONTAINER)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .stroke(egui::Stroke::new(1.0_f32, md3::OUTLINE_VARIANT))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&layer_name_title).strong().size(11.5).color(md3::PRIMARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let reset_layer_lbl = if is_vi { "↺ Reset Lớp" } else { "↺ Reset Layer" };
                    if ui.button(egui::RichText::new(reset_layer_lbl).size(10.5).color(md3::ON_SURFACE_VARIANT)).clicked() {
                        adj = LayerAdjustments::default();
                        inspector_changed = true;
                    }
                });
            });

            ui.add_space(6.0);

            // Row 1: RGB Color Picker & Tint Intensity & Opacity
            ui.horizontal_wrapped(|ui| {
                let rgb_lbl = if is_vi { "Màu phủ RGB:" } else { "RGB Tint:" };
                ui.label(egui::RichText::new(rgb_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                let mut srgb = [adj.tint_color[0], adj.tint_color[1], adj.tint_color[2]];
                if ui.color_edit_button_srgb(&mut srgb).changed() {
                    adj.tint_color = srgb;
                    if adj.tint_amount < 0.05 {
                        adj.tint_amount = 0.85;
                    }
                    inspector_changed = true;
                }

                let tint_lbl = if is_vi { "Mức phủ:" } else { "Tint:" };
                ui.label(egui::RichText::new(tint_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                let mut tint_pct = (adj.tint_amount * 100.0).round() as i32;
                if ui.add(egui::Slider::new(&mut tint_pct, 0..=100).suffix("%")).changed() {
                    adj.tint_amount = tint_pct as f32 / 100.0;
                    inspector_changed = true;
                }

                let op_lbl = if is_vi { "Độ mờ:" } else { "Opacity:" };
                ui.label(egui::RichText::new(op_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                let mut op_pct = (adj.opacity * 100.0).round() as i32;
                if ui.add(egui::Slider::new(&mut op_pct, 0..=100).suffix("%")).changed() {
                    adj.opacity = op_pct as f32 / 100.0;
                    inspector_changed = true;
                }
            });

            ui.add_space(4.0);

            // Row 2: Hue Shift & Saturation
            ui.horizontal_wrapped(|ui| {
                let hue_lbl = if is_vi { "Chuyển sắc độ (Hue):" } else { "Hue Shift:" };
                ui.label(egui::RichText::new(hue_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                let mut hue_deg = adj.hue_shift.round() as i32;
                if ui.add(egui::Slider::new(&mut hue_deg, -180..=180).suffix("°")).changed() {
                    adj.hue_shift = hue_deg as f32;
                    inspector_changed = true;
                }

                let sat_lbl = if is_vi { "Bão hòa (Sat):" } else { "Saturation:" };
                ui.label(egui::RichText::new(sat_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                let mut sat_pct = (adj.saturation * 100.0).round() as i32;
                if ui.add(egui::Slider::new(&mut sat_pct, 0..=200).suffix("%")).changed() {
                    adj.saturation = sat_pct as f32 / 100.0;
                    inspector_changed = true;
                }
            });

            ui.add_space(4.0);

            // Row 3: Rotation slider & quick angle presets
            ui.horizontal_wrapped(|ui| {
                let rot_lbl = if is_vi { "Xoay (Rotation):" } else { "Rotation:" };
                ui.label(egui::RichText::new(rot_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                let mut cur_rot = adj.rotation.round() as i32;
                if ui.add(egui::Slider::new(&mut cur_rot, -180..=180).suffix("°")).changed() {
                    adj.rotation = cur_rot as f32;
                    inspector_changed = true;
                }

                for deg in [-90, 0, 90, 180] {
                    if ui.button(egui::RichText::new(format!("{}°", deg)).size(10.0).color(md3::ON_SURFACE_VARIANT)).clicked() {
                        adj.rotation = deg as f32;
                        inspector_changed = true;
                    }
                }
            });

            if active_layer == ActiveTransformLayer::Details {
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                state.overlay_options.details.ensure_items();
                let sel_idx = state.selected_text_index.min(state.overlay_options.details.items.len().saturating_sub(1));

                // Style & Global Scale
                ui.horizontal_wrapped(|ui| {
                    let style_lbl = if is_vi { "Hiệu ứng chữ:" } else { "Emboss Style:" };
                    ui.label(egui::RichText::new(style_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    egui::ComboBox::from_id_salt("inspector_details_style")
                        .width(135.0)
                        .selected_text(egui::RichText::new(state.overlay_options.details.emboss_style.display_name_lang(is_vi)).size(11.0).color(md3::ON_SURFACE))
                        .show_ui(ui, |ui| {
                            for s in [
                                EmbossStyle::EmbossedSilver,
                                EmbossStyle::EmbossedGold,
                                EmbossStyle::CrispWhite,
                                EmbossStyle::StealthDark,
                            ] {
                                if ui.selectable_value(&mut state.overlay_options.details.emboss_style, s, s.display_name_lang(is_vi)).clicked() {
                                    inspector_changed = true;
                                }
                            }
                        });

                    let scale_lbl = if is_vi { "Cỡ chữ chung:" } else { "Global Scale:" };
                    ui.label(egui::RichText::new(scale_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    let mut sc_pct = (state.overlay_options.details.scale * 100.0).round() as i32;
                    if ui.add(egui::Slider::new(&mut sc_pct, 40..=250).suffix("%")).changed() {
                        state.overlay_options.details.scale = (sc_pct as f32 / 100.0).clamp(0.4, 2.5);
                        inspector_changed = true;
                    }
                });

                // Row 2: Selected Text Item Content & Backdrop
                if let Some(item) = state.overlay_options.details.items.get_mut(sel_idx) {
                    ui.add_space(3.0);
                    ui.horizontal_wrapped(|ui| {
                        let item_badge = format!("#{}", sel_idx + 1);
                        ui.label(egui::RichText::new(item_badge).strong().size(11.0).color(md3::PRIMARY));

                        ui.label(egui::RichText::new(if is_vi { "Nhãn:" } else { "Label:" }).size(11.0).color(md3::ON_SURFACE_VARIANT));
                        if ui.add(egui::TextEdit::singleline(&mut item.label).desired_width(100.0)).changed() {
                            inspector_changed = true;
                        }

                        ui.label(egui::RichText::new(if is_vi { "Chữ:" } else { "Text:" }).size(11.0).color(md3::ON_SURFACE_VARIANT));
                        if ui.add(egui::TextEdit::singleline(&mut item.content).desired_width(150.0)).changed() {
                            inspector_changed = true;
                        }

                        let case_txt = if item.is_uppercase { "HOA" } else { "thường" };
                        if ui.button(egui::RichText::new(case_txt).size(10.5).color(md3::ON_SURFACE_VARIANT))
                            .on_hover_text(if is_vi { "Đổi chữ IN HOA / Thường" } else { "Toggle UPPERCASE / Normal" })
                            .clicked()
                        {
                            item.is_uppercase = !item.is_uppercase;
                            inspector_changed = true;
                        }

                        if ui.checkbox(&mut item.has_backdrop, if is_vi { "🧊 Nền mờ" } else { "🧊 Backdrop" })
                            .on_hover_text(if is_vi { "Bật/tắt hộp nền mờ bo góc riêng cho dòng chữ này" } else { "Toggle frosted pill background for this text line" })
                            .changed()
                        {
                            inspector_changed = true;
                        }
                    });

                    // Row 3: Font, Font Size, X, Y, Letter Spacing
                    ui.add_space(3.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(if is_vi { "Font:" } else { "Font:" }).size(11.0).color(md3::ON_SURFACE_VARIANT));
                        egui::ComboBox::from_id_salt("inspector_item_font")
                            .width(135.0)
                            .selected_text(egui::RichText::new(item.font.display_name_lang(is_vi)).size(10.5).color(md3::ON_SURFACE))
                            .show_ui(ui, |ui| {
                                for f in [
                                    CardFontPreset::ClassicOcr,
                                    CardFontPreset::ModernSans,
                                    CardFontPreset::Monospace,
                                    CardFontPreset::SerifLuxury,
                                    CardFontPreset::Custom,
                                ] {
                                    if ui.selectable_label(item.font == f, f.display_name_lang(is_vi)).clicked() {
                                        item.font = f;
                                        inspector_changed = true;
                                    }
                                }
                            });

                        ui.label(egui::RichText::new("Cỡ:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                        if ui.add(egui::DragValue::new(&mut item.font_size).range(10.0..=120.0).speed(0.5)).changed() {
                            inspector_changed = true;
                        }

                        ui.label(egui::RichText::new("X:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                        if ui.add(egui::DragValue::new(&mut item.x).range(-100.0..=1600.0).speed(1.0)).changed() {
                            inspector_changed = true;
                        }

                        ui.label(egui::RichText::new("Y:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                        if ui.add(egui::DragValue::new(&mut item.y).range(-50.0..=1000.0).speed(1.0)).changed() {
                            inspector_changed = true;
                        }

                        ui.label(egui::RichText::new("Dãn:").size(10.5).color(md3::ON_SURFACE_VARIANT));
                        if ui.add(egui::DragValue::new(&mut item.letter_spacing).range(-2.0..=20.0).speed(0.1)).changed() {
                            inspector_changed = true;
                        }
                    });
                }
            }
        });

    if inspector_changed {
        match active_layer {
            ActiveTransformLayer::Background => {
                state.overlay_options.bg_adj = adj;
                state.overlay_options.transform.rotation = adj.rotation;
            }
            ActiveTransformLayer::Finish => state.overlay_options.finish_adj = adj,
            ActiveTransformLayer::Chip => state.overlay_options.chip_adj = adj,
            ActiveTransformLayer::Wave => state.overlay_options.wave_adj = adj,
            ActiveTransformLayer::Logo => state.overlay_options.logo_adj = adj,
            ActiveTransformLayer::Details => state.overlay_options.details_adj = adj,
            ActiveTransformLayer::Widget(idx) => {
                if let Some(w) = state.custom_widgets.get_mut(idx) {
                    w.data.adjustments = adj;
                }
            }
        }
        preview_changed = true;
    }

    ui.add_space(6.0);
    let layer_hint = if is_vi {
        match state.active_layer {
            ActiveTransformLayer::Background => "Lớp đang chọn: Nền thẻ (Kéo chuột để di chuyển • Cuộn chuột để thu phóng • Giữ Shift+Cuộn để xoay)".to_string(),
            ActiveTransformLayer::Finish => "Lớp đang chọn: Finish Texture (Kéo chuột để di chuyển • Cuộn chuột để thu phóng • Giữ Shift+Cuộn để xoay)".to_string(),
            ActiveTransformLayer::Chip => "Lớp đang chọn: Chip EMV (Kéo chuột để di chuyển • Cuộn chuột để đổi cỡ • Giữ Shift+Cuộn để xoay)".to_string(),
            ActiveTransformLayer::Wave => "Lớp đang chọn: Sóng Contactless (Kéo chuột để di chuyển • Cuộn chuột để đổi cỡ • Giữ Shift+Cuộn để xoay)".to_string(),
            ActiveTransformLayer::Logo => "Lớp đang chọn: Logo thương hiệu (Kéo chuột để di chuyển • Cuộn chuột để đổi cỡ • Giữ Shift+Cuộn để xoay)".to_string(),
            ActiveTransformLayer::Details => "Lớp đang chọn: Thông tin dập nổi (Kéo chuột để di chuyển • Cuộn chuột để đổi cỡ • Giữ Shift+Cuộn để xoay)".to_string(),
            ActiveTransformLayer::Widget(idx) => {
                let name = state.custom_widgets.get(idx).map(|w| w.data.name.as_str()).unwrap_or("Widget");
                format!("Lớp đang chọn: Widget \"{}\" (Kéo chuột để di chuyển • Cuộn chuột để đổi cỡ • Giữ Shift+Cuộn để xoay)", name)
            }
        }
    } else {
        match state.active_layer {
            ActiveTransformLayer::Background => "Active Layer: Background (Drag card to pan • Scroll to zoom • Shift+Scroll to rotate)".to_string(),
            ActiveTransformLayer::Finish => "Active Layer: Finish Texture (Drag card to pan • Scroll to zoom • Shift+Scroll to rotate)".to_string(),
            ActiveTransformLayer::Chip => "Active Layer: EMV Chip (Drag to reposition • Scroll to resize • Shift+Scroll to rotate)".to_string(),
            ActiveTransformLayer::Wave => "Active Layer: Contactless Wave (Drag to reposition • Scroll to resize • Shift+Scroll to rotate)".to_string(),
            ActiveTransformLayer::Logo => "Active Layer: Brand Logo (Drag to reposition • Scroll to resize • Shift+Scroll to rotate)".to_string(),
            ActiveTransformLayer::Details => "Active Layer: Card Details (Drag to reposition • Scroll to resize • Shift+Scroll to rotate)".to_string(),
            ActiveTransformLayer::Widget(idx) => {
                let name = state.custom_widgets.get(idx).map(|w| w.data.name.as_str()).unwrap_or("Widget");
                format!("Active Layer: Widget \"{}\" (Drag to reposition • Scroll to resize • Shift+Scroll to rotate)", name)
            }
        }
    };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("💡 {}", layer_hint)).size(10.5).color(md3::PRIMARY));
    });

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("1536x969").size(11.0).color(md3::ON_SURFACE_VARIANT));
        ui.label(egui::RichText::new("|").size(11.0).color(md3::OUTLINE_VARIANT));
        let ratio_lbl = if is_vi { "Tỉ lệ 1.585" } else { "1.585 ratio" };
        ui.label(egui::RichText::new(ratio_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
        ui.label(egui::RichText::new("|").size(11.0).color(md3::OUTLINE_VARIANT));
        if skin_texture.is_some() {
            let ready_lbl = if is_vi { "Sẵn sàng" } else { "Ready" };
            ui.label(egui::RichText::new(ready_lbl).size(11.0).color(md3::SUCCESS));
        } else {
            let no_img_lbl = if is_vi { "Chưa có ảnh" } else { "No image" };
            ui.label(egui::RichText::new(no_img_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));
        }
    });
    ui.add_space(8.0);
    let note_lbl = if is_vi {
        "Sau khi áp dụng, vuốt tắt ứng dụng Apple Wallet trên iPhone và mở lại."
    } else {
        "After applying, force close Apple Wallet and reopen it."
    };
    ui.label(egui::RichText::new(note_lbl).size(11.0).color(md3::ON_SURFACE_VARIANT));

    preview_changed
}
