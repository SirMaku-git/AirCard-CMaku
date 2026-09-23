use anyhow::Result;
use eframe::egui;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};

use crate::card_studio::types::{
    CardBackgroundPreset, CardDetails, CardFinish, CardFontPreset, CardOverlayOptions, ImageTransform,
    LayerAdjustments, LogoBadgeStyle, LogoColorTheme, PaymentNetwork, TextBackdropStyle,
    EmbossStyle, CARD_WIDTH, CARD_HEIGHT,
};
use crate::card_studio::widget::CustomWidgetData;

// ---------------------------------------------------------------------------
// Authentic Anti-Aliased Card Assets (Embedded via include_bytes!)
// ---------------------------------------------------------------------------
const ASSET_VISA: &[u8] = include_bytes!("../assets/visa.png");
const ASSET_MASTERCARD: &[u8] = include_bytes!("../assets/mastercard.png");
const ASSET_NAPAS: &[u8] = include_bytes!("../assets/napas.png");
const ASSET_JCB: &[u8] = include_bytes!("../assets/jcb.png");
const ASSET_EMV_CHIP: &[u8] = include_bytes!("../assets/emv_chip.png");
const ASSET_CONTACTLESS: &[u8] = include_bytes!("../assets/contactless.png");
const ASSET_CARD_OCR: &[u8] = include_bytes!("../assets/card_ocr.ttf");
const ASSET_CARD_SANS: &[u8] = include_bytes!("../assets/card_sans.ttf");

#[inline]
pub fn blend_pixel(img: &mut RgbaImage, x: i32, y: i32, r: u8, g: u8, b: u8, a: u8) {
    if x < 0 || y < 0 || x >= CARD_WIDTH as i32 || y >= CARD_HEIGHT as i32 || a == 0 {
        return;
    }
    let px = img.get_pixel_mut(x as u32, y as u32);
    if a == 255 {
        *px = Rgba([r, g, b, 255]);
    } else {
        let alpha = a as u32;
        let inv_alpha = 255 - alpha;
        let dst_r = px[0] as u32;
        let dst_g = px[1] as u32;
        let dst_b = px[2] as u32;
        let out_r = ((r as u32 * alpha + dst_r * inv_alpha) / 255) as u8;
        let out_g = ((g as u32 * alpha + dst_g * inv_alpha) / 255) as u8;
        let out_b = ((b as u32 * alpha + dst_b * inv_alpha) / 255) as u8;
        *px = Rgba([out_r, out_g, out_b, 255]);
    }
}

pub fn draw_rounded_rect(
    img: &mut RgbaImage,
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    radius: f32,
    color: [u8; 4],
) {
    let r2 = radius * radius;
    let x1 = x0 + w;
    let y1 = y0 + h;

    for y in y0..y1 {
        for x in x0..x1 {
            let mut inside = true;
            let cx = if ((x - x0) as f32) < radius {
                x0 as f32 + radius
            } else if ((x1 - 1 - x) as f32) < radius {
                x1 as f32 - 1.0 - radius
            } else {
                x as f32
            };

            let cy = if ((y - y0) as f32) < radius {
                y0 as f32 + radius
            } else if ((y1 - 1 - y) as f32) < radius {
                y1 as f32 - 1.0 - radius
            } else {
                y as f32
            };

            if cx != x as f32 && cy != y as f32 {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                if dx * dx + dy * dy > r2 {
                    inside = false;
                }
            }

            if inside {
                blend_pixel(img, x, y, color[0], color[1], color[2], color[3]);
            }
        }
    }
}

pub fn draw_rounded_rect_outline(
    img: &mut RgbaImage,
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    radius: f32,
    thickness: f32,
    color: [u8; 4],
) {
    let r_out = radius;
    let r_in = (radius - thickness).max(0.0);
    let r_out2 = r_out * r_out;
    let r_in2 = r_in * r_in;
    let x1 = x0 + w;
    let y1 = y0 + h;

    for y in y0..y1 {
        for x in x0..x1 {
            let cx = if ((x - x0) as f32) < radius {
                x0 as f32 + radius
            } else if ((x1 - 1 - x) as f32) < radius {
                x1 as f32 - 1.0 - radius
            } else {
                x as f32
            };

            let cy = if ((y - y0) as f32) < radius {
                y0 as f32 + radius
            } else if ((y1 - 1 - y) as f32) < radius {
                y1 as f32 - 1.0 - radius
            } else {
                y as f32
            };

            let on_edge = if cx != x as f32 && cy != y as f32 {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let d2 = dx * dx + dy * dy;
                d2 <= r_out2 && d2 >= r_in2
            } else {
                let on_left = (x - x0) < thickness as i32;
                let on_right = (x1 - 1 - x) < thickness as i32;
                let on_top = (y - y0) < thickness as i32;
                let on_bottom = (y1 - 1 - y) < thickness as i32;
                on_left || on_right || on_top || on_bottom
            };

            if on_edge {
                blend_pixel(img, x, y, color[0], color[1], color[2], color[3]);
            }
        }
    }
}

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta).rem_euclid(6.0))
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    let s = if max == 0.0 { 0.0 } else { delta / max };
    let v = max / 255.0;
    (h, s, v)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h_prime = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - (h_prime.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = v - c;
    ((r1 + m) * 255.0, (g1 + m) * 255.0, (b1 + m) * 255.0)
}

fn apply_adjustments_to_color(
    mut r: f32,
    mut g: f32,
    mut b: f32,
    mut a: f32,
    adj: &LayerAdjustments,
    theme: LogoColorTheme,
) -> (u8, u8, u8, u8) {
    match theme {
        LogoColorTheme::Original => {}
        LogoColorTheme::MonochromeWhite => {
            r = 255.0;
            g = 255.0;
            b = 255.0;
        }
        LogoColorTheme::LuxuryGold => {
            let lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
            r = (212.0 * lum + 30.0).clamp(0.0, 255.0);
            g = (175.0 * lum + 20.0).clamp(0.0, 255.0);
            b = (55.0 * lum + 5.0).clamp(0.0, 255.0);
        }
        LogoColorTheme::SilverPlatinum => {
            let lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
            let val = (lum * 220.0 + 35.0).clamp(0.0, 255.0);
            r = val;
            g = val;
            b = (val * 1.05).clamp(0.0, 255.0);
        }
        LogoColorTheme::StealthBlack => {
            let lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
            let val = (lum * 60.0 + 15.0).clamp(0.0, 255.0);
            r = val;
            g = val;
            b = val;
        }
    }

    if adj.hue_shift != 0.0 || (adj.saturation - 1.0).abs() > 0.001 {
        let (h, s, v) = rgb_to_hsv(r, g, b);
        let new_h = (h + adj.hue_shift).rem_euclid(360.0);
        let new_s = (s * adj.saturation).clamp(0.0, 1.0);
        let (nr, ng, nb) = hsv_to_rgb(new_h, new_s, v);
        r = nr;
        g = ng;
        b = nb;
    }

    if adj.tint_amount > 0.001 {
        let t_amt = adj.tint_amount.clamp(0.0, 1.0);
        let tr = adj.tint_color[0] as f32;
        let tg = adj.tint_color[1] as f32;
        let tb = adj.tint_color[2] as f32;
        r = r * (1.0 - t_amt) + (r * tr / 255.0) * t_amt;
        g = g * (1.0 - t_amt) + (g * tg / 255.0) * t_amt;
        b = b * (1.0 - t_amt) + (b * tb / 255.0) * t_amt;
    }

    a = (a * adj.opacity.clamp(0.0, 1.0)).clamp(0.0, 255.0);

    (
        r.clamp(0.0, 255.0).round() as u8,
        g.clamp(0.0, 255.0).round() as u8,
        b.clamp(0.0, 255.0).round() as u8,
        a.round() as u8,
    )
}

pub fn render_transformed_asset(
    img: &mut RgbaImage,
    asset: &RgbaImage,
    center_x: f32,
    center_y: f32,
    scale: f32,
    base_w: f32,
    base_h: f32,
    adj: &LayerAdjustments,
    theme: LogoColorTheme,
    has_shadow: bool,
) {
    let asset_w = asset.width() as f32;
    let asset_h = asset.height() as f32;
    if asset_w <= 0.0 || asset_h <= 0.0 {
        return;
    }

    let scale = scale.clamp(0.1, 10.0);
    let render_w = (base_w * scale).max(1.0);
    let render_h = (base_h * scale).max(1.0);

    let radius = ((render_w * render_w + render_h * render_h).sqrt() * 0.5 + 4.0).ceil();
    let min_x = (center_x - radius).floor().max(0.0) as i32;
    let max_x = (center_x + radius).ceil().min(CARD_WIDTH as f32 - 1.0) as i32;
    let min_y = (center_y - radius).floor().max(0.0) as i32;
    let max_y = (center_y + radius).ceil().min(CARD_HEIGHT as f32 - 1.0) as i32;

    if min_x > max_x || min_y > max_y {
        return;
    }

    let rad = (-adj.rotation).to_radians();
    let cos_r = rad.cos();
    let sin_r = rad.sin();

    if has_shadow && adj.opacity > 0.05 {
        let shadow_off_x = 3.0;
        let shadow_off_y = 4.0;
        let shadow_alpha_factor = 0.45 * adj.opacity;

        for py in min_y..=max_y {
            let dy = py as f32 + 0.5 - (center_y + shadow_off_y);
            for px in min_x..=max_x {
                let dx = px as f32 + 0.5 - (center_x + shadow_off_x);
                let rot_x = dx * cos_r - dy * sin_r;
                let rot_y = dx * sin_r + dy * cos_r;
                let u = rot_x / render_w + 0.5;
                let v = rot_y / render_h + 0.5;
                if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 {
                    continue;
                }
                let sx = (u * (asset_w - 1.0)).clamp(0.0, asset_w - 1.0);
                let sy = (v * (asset_h - 1.0)).clamp(0.0, asset_h - 1.0);
                let sx0 = sx.floor() as u32;
                let sx1 = (sx0 + 1).min(asset.width() - 1);
                let sy0 = sy.floor() as u32;
                let sy1 = (sy0 + 1).min(asset.height() - 1);
                let fx = sx - sx0 as f32;
                let fy = sy - sy0 as f32;

                let a00 = asset.get_pixel(sx0, sy0)[3] as f32;
                let a10 = asset.get_pixel(sx1, sy0)[3] as f32;
                let a01 = asset.get_pixel(sx0, sy1)[3] as f32;
                let a11 = asset.get_pixel(sx1, sy1)[3] as f32;
                let a = (a00 * (1.0 - fx) + a10 * fx) * (1.0 - fy) + (a01 * (1.0 - fx) + a11 * fx) * fy;
                if a > 8.0 {
                    let s_a = ((a / 255.0) * shadow_alpha_factor * 255.0).clamp(0.0, 255.0) as u8;
                    blend_pixel(img, px, py, 0, 0, 0, s_a);
                }
            }
        }
    }

    for py in min_y..=max_y {
        let dy = py as f32 + 0.5 - center_y;
        for px in min_x..=max_x {
            let dx = px as f32 + 0.5 - center_x;
            let rot_x = dx * cos_r - dy * sin_r;
            let rot_y = dx * sin_r + dy * cos_r;
            let u = rot_x / render_w + 0.5;
            let v = rot_y / render_h + 0.5;
            if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 {
                continue;
            }

            let sx = (u * (asset_w - 1.0)).clamp(0.0, asset_w - 1.0);
            let sy = (v * (asset_h - 1.0)).clamp(0.0, asset_h - 1.0);
            let sx0 = sx.floor() as u32;
            let sx1 = (sx0 + 1).min(asset.width() - 1);
            let sy0 = sy.floor() as u32;
            let sy1 = (sy0 + 1).min(asset.height() - 1);
            let fx = sx - sx0 as f32;
            let fy = sy - sy0 as f32;

            let p00 = asset.get_pixel(sx0, sy0);
            let p10 = asset.get_pixel(sx1, sy0);
            let p01 = asset.get_pixel(sx0, sy1);
            let p11 = asset.get_pixel(sx1, sy1);

            let interp = |idx: usize| -> f32 {
                let top = p00[idx] as f32 * (1.0 - fx) + p10[idx] as f32 * fx;
                let bot = p01[idx] as f32 * (1.0 - fx) + p11[idx] as f32 * fx;
                top * (1.0 - fy) + bot * fy
            };

            let a = interp(3);
            if a < 2.0 {
                continue;
            }

            let r = interp(0);
            let g = interp(1);
            let b = interp(2);

            let (out_r, out_g, out_b, out_a) = apply_adjustments_to_color(r, g, b, a, adj, theme);
            if out_a > 0 {
                blend_pixel(img, px, py, out_r, out_g, out_b, out_a);
            }
        }
    }
}

pub fn render_transformed_custom_image(
    image: &DynamicImage,
    transform: ImageTransform,
    bg_adj: &LayerAdjustments,
) -> RgbaImage {
    let (src_w, src_h) = image.dimensions();
    if src_w == 0 || src_h == 0 {
        return RgbaImage::from_pixel(CARD_WIDTH, CARD_HEIGHT, Rgba([18, 18, 22, 255]));
    }
    let src_rgba = image.to_rgba8();

    let mut dest = RgbaImage::from_pixel(CARD_WIDTH, CARD_HEIGHT, Rgba([18, 18, 22, 255]));

    let zoom = transform.zoom.clamp(0.05, 50.0);
    let scale0 = (CARD_WIDTH as f32 / src_w as f32).max(CARD_HEIGHT as f32 / src_h as f32);
    let scale = scale0 * zoom;

    let inv_scale = 1.0 / scale;
    let src_raw = src_rgba.as_raw();
    let stride = src_w as usize * 4;
    let max_x_idx = src_w - 1;
    let max_y_idx = src_h - 1;

    let center_x = CARD_WIDTH as f32 * 0.5 + transform.pan_x;
    let center_y = CARD_HEIGHT as f32 * 0.5 + transform.pan_y;
    let src_cx = src_w as f32 * 0.5;
    let src_cy = src_h as f32 * 0.5;

    let total_rot = transform.rotation + bg_adj.rotation;
    let rad = (-total_rot).to_radians();
    let cos_r = rad.cos();
    let sin_r = rad.sin();

    for y in 0..CARD_HEIGHT {
        let dy = y as f32 + 0.5 - center_y;
        for x in 0..CARD_WIDTH {
            let dx = x as f32 + 0.5 - center_x;
            let rx = dx * cos_r - dy * sin_r;
            let ry = dx * sin_r + dy * cos_r;

            let sx = rx * inv_scale + src_cx;
            let sy = ry * inv_scale + src_cy;

            if sx < 0.0 || sx > max_x_idx as f32 || sy < 0.0 || sy > max_y_idx as f32 {
                continue;
            }

            let sx_clamped = sx.clamp(0.0, max_x_idx as f32);
            let sy_clamped = sy.clamp(0.0, max_y_idx as f32);
            let x0 = sx_clamped.floor() as u32;
            let x1 = (x0 + 1).min(max_x_idx);
            let y0 = sy_clamped.floor() as u32;
            let y1 = (y0 + 1).min(max_y_idx);

            let fx = sx_clamped - x0 as f32;
            let fy = sy_clamped - y0 as f32;
            let inv_fx = 1.0 - fx;
            let inv_fy = 1.0 - fy;

            let w00 = inv_fx * inv_fy;
            let w10 = fx * inv_fy;
            let w01 = inv_fx * fy;
            let w11 = fx * fy;

            let row0 = y0 as usize * stride;
            let row1 = y1 as usize * stride;

            let idx00 = row0 + x0 as usize * 4;
            let idx10 = row0 + x1 as usize * 4;
            let idx01 = row1 + x0 as usize * 4;
            let idx11 = row1 + x1 as usize * 4;

            let r = src_raw[idx00] as f32 * w00
                + src_raw[idx10] as f32 * w10
                + src_raw[idx01] as f32 * w01
                + src_raw[idx11] as f32 * w11;
            let g = src_raw[idx00 + 1] as f32 * w00
                + src_raw[idx10 + 1] as f32 * w10
                + src_raw[idx01 + 1] as f32 * w01
                + src_raw[idx11 + 1] as f32 * w11;
            let b = src_raw[idx00 + 2] as f32 * w00
                + src_raw[idx10 + 2] as f32 * w10
                + src_raw[idx01 + 2] as f32 * w01
                + src_raw[idx11 + 2] as f32 * w11;
            let a = src_raw[idx00 + 3] as f32 * w00
                + src_raw[idx10 + 3] as f32 * w10
                + src_raw[idx01 + 3] as f32 * w01
                + src_raw[idx11 + 3] as f32 * w11;

            let (out_r, out_g, out_b, out_a) = apply_adjustments_to_color(r, g, b, a, bg_adj, LogoColorTheme::Original);
            dest.put_pixel(x, y, Rgba([out_r, out_g, out_b, out_a]));
        }
    }

    dest
}

pub fn generate_card_preset_canvas(
    preset: CardBackgroundPreset,
    maybe_image: Option<&DynamicImage>,
    transform: ImageTransform,
    bg_adj: &LayerAdjustments,
) -> RgbaImage {
    let mut img = RgbaImage::new(CARD_WIDTH, CARD_HEIGHT);

    match preset {
        CardBackgroundPreset::CustomImage => {
            if let Some(custom) = maybe_image {
                return render_transformed_custom_image(custom, transform, bg_adj);
            } else {
                for y in 0..CARD_HEIGHT {
                    let fy = y as f32 / CARD_HEIGHT as f32;
                    let v = (25.0 + fy * 15.0) as u8;
                    for x in 0..CARD_WIDTH {
                        img.put_pixel(x, y, Rgba([v, v, v + 4, 255]));
                    }
                }
            }
        }
        CardBackgroundPreset::MatteBlack => {
            for y in 0..CARD_HEIGHT {
                let fy = y as f32 / CARD_HEIGHT as f32;
                for x in 0..CARD_WIDTH {
                    let fx = x as f32 / CARD_WIDTH as f32;
                    let base = 22.0 - fy * 10.0 + fx * 4.0;
                    let diag_dist = ((x as f32 * 0.7 + y as f32) - 800.0).abs();
                    let specular = if diag_dist < 400.0 {
                        ((400.0 - diag_dist) / 400.0).powi(2) * 22.0
                    } else {
                        0.0
                    };
                    let val = ((base + specular).clamp(10.0, 65.0)) as u8;
                    img.put_pixel(x, y, Rgba([val, val, (val as f32 * 1.05).min(255.0) as u8, 255]));
                }
            }
            draw_rounded_rect_outline(&mut img, 20, 20, (CARD_WIDTH - 40) as i32, (CARD_HEIGHT - 40) as i32, 28.0, 1.5, [255, 255, 255, 25]);
        }
        CardBackgroundPreset::OceanNavy => {
            let center_x = CARD_WIDTH as f32 * 0.40;
            let center_y = CARD_HEIGHT as f32 * 0.35;
            for y in 0..CARD_HEIGHT {
                let dy = (y as f32 - center_y) / CARD_HEIGHT as f32;
                for x in 0..CARD_WIDTH {
                    let dx = (x as f32 - center_x) / CARD_WIDTH as f32;
                    let dist2 = dx * dx + dy * dy;
                    let glow = (1.0 - dist2.sqrt() * 1.3).clamp(0.0, 1.0);
                    let r = (4.0 + glow * 24.0) as u8;
                    let g = (32.0 + glow * 58.0) as u8;
                    let b = (95.0 + glow * 85.0) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
            for angle_step in 1..=4 {
                let cy = -200i32 + angle_step * 250;
                draw_rounded_rect_outline(&mut img, 100, cy, 1336, 600, 300.0, 1.0, [255, 255, 255, 12]);
            }
        }
        CardBackgroundPreset::BrushedGold => {
            for y in 0..CARD_HEIGHT {
                let fy = y as f32 / CARD_HEIGHT as f32;
                let base_r = 210.0 - fy * 45.0;
                let base_g = 168.0 - fy * 40.0;
                let base_b = 58.0 - fy * 18.0;
                for x in 0..CARD_WIDTH {
                    let noise = (((x * 17 + y * 73) % 29) as f32 - 14.0) * 0.9;
                    let r = (base_r + noise).clamp(0.0, 255.0) as u8;
                    let g = (base_g + noise).clamp(0.0, 255.0) as u8;
                    let b = (base_b + noise * 0.5).clamp(0.0, 255.0) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
            draw_rounded_rect_outline(&mut img, 20, 20, (CARD_WIDTH - 40) as i32, (CARD_HEIGHT - 40) as i32, 28.0, 2.0, [255, 235, 160, 60]);
        }
        CardBackgroundPreset::EmeraldLuxury => {
            for y in 0..CARD_HEIGHT {
                let fy = y as f32 / CARD_HEIGHT as f32;
                for x in 0..CARD_WIDTH {
                    let fx = x as f32 / CARD_WIDTH as f32;
                    let d = ((fx - 0.5) * (fx - 0.5) + (fy - 0.5) * (fy - 0.5)).sqrt();
                    let factor = (1.0 - d * 0.9).clamp(0.0, 1.0);
                    let r = (5.0 + factor * 25.0) as u8;
                    let g = (45.0 + factor * 75.0) as u8;
                    let b = (35.0 + factor * 45.0) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
            draw_rounded_rect_outline(&mut img, 20, 20, (CARD_WIDTH - 40) as i32, (CARD_HEIGHT - 40) as i32, 28.0, 1.5, [140, 255, 200, 45]);
        }
        CardBackgroundPreset::TitaniumMinimal => {
            for y in 0..CARD_HEIGHT {
                let fy = y as f32 / CARD_HEIGHT as f32;
                let val = (75.0 + fy * 20.0) as u8;
                for x in 0..CARD_WIDTH {
                    let noise = (((x * 19 + y * 53) % 17) as f32 - 8.0) * 0.6;
                    let v = (val as f32 + noise).clamp(0.0, 255.0) as u8;
                    img.put_pixel(x, y, Rgba([v, (v as f32 * 1.02).min(255.0) as u8, (v as f32 * 1.05).min(255.0) as u8, 255]));
                }
            }
            draw_rounded_rect_outline(&mut img, 20, 20, (CARD_WIDTH - 40) as i32, (CARD_HEIGHT - 40) as i32, 28.0, 1.5, [255, 255, 255, 45]);
        }
        CardBackgroundPreset::CrimsonVelvet => {
            for y in 0..CARD_HEIGHT {
                let fy = y as f32 / CARD_HEIGHT as f32;
                for x in 0..CARD_WIDTH {
                    let fx = x as f32 / CARD_WIDTH as f32;
                    let r = (75.0 - fy * 30.0 + fx * 15.0).clamp(0.0, 255.0) as u8;
                    let g = (12.0 - fy * 5.0).clamp(0.0, 255.0) as u8;
                    let b = (24.0 - fy * 10.0).clamp(0.0, 255.0) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
            draw_rounded_rect_outline(&mut img, 20, 20, (CARD_WIDTH - 40) as i32, (CARD_HEIGHT - 40) as i32, 28.0, 1.5, [255, 140, 160, 40]);
        }
        CardBackgroundPreset::DeepCyberViolet => {
            for y in 0..CARD_HEIGHT {
                let fy = y as f32 / CARD_HEIGHT as f32;
                for x in 0..CARD_WIDTH {
                    let fx = x as f32 / CARD_WIDTH as f32;
                    let r = (45.0 + fx * 35.0 - fy * 15.0).clamp(0.0, 255.0) as u8;
                    let g = (12.0 + fx * 10.0).clamp(0.0, 255.0) as u8;
                    let b = (75.0 + fy * 40.0).clamp(0.0, 255.0) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
            draw_rounded_rect_outline(&mut img, 20, 20, (CARD_WIDTH - 40) as i32, (CARD_HEIGHT - 40) as i32, 28.0, 1.5, [200, 160, 255, 45]);
        }
    }

    if bg_adj.tint_amount > 0.001 || bg_adj.hue_shift != 0.0 || (bg_adj.saturation - 1.0).abs() > 0.001 || bg_adj.opacity < 0.999 {
        for p in img.pixels_mut() {
            let (r, g, b, a) = apply_adjustments_to_color(p[0] as f32, p[1] as f32, p[2] as f32, p[3] as f32, bg_adj, LogoColorTheme::Original);
            *p = Rgba([r, g, b, a]);
        }
    }

    img
}

pub fn draw_emv_chip(
    img: &mut RgbaImage,
    custom_chip: Option<&RgbaImage>,
    chip_x: f32,
    chip_y: f32,
    chip_scale: f32,
    chip_adj: &LayerAdjustments,
) {
    let base_w = 205.0f32;
    let base_h = 155.0f32;
    if let Some(chip) = custom_chip {
        render_transformed_asset(
            img,
            chip,
            chip_x,
            chip_y,
            chip_scale,
            base_w,
            base_h,
            chip_adj,
            LogoColorTheme::Original,
            true,
        );
    } else if let Ok(dyn_img) = image::load_from_memory(ASSET_EMV_CHIP) {
        let chip = dyn_img.to_rgba8();
        render_transformed_asset(
            img,
            &chip,
            chip_x,
            chip_y,
            chip_scale,
            base_w,
            base_h,
            chip_adj,
            LogoColorTheme::Original,
            true,
        );
    }
}

pub fn draw_contactless_wave(
    img: &mut RgbaImage,
    wave_x: f32,
    wave_y: f32,
    wave_scale: f32,
    wave_adj: &LayerAdjustments,
) {
    if let Ok(dyn_img) = image::load_from_memory(ASSET_CONTACTLESS) {
        let wave = dyn_img.to_rgba8();
        render_transformed_asset(
            img,
            &wave,
            wave_x,
            wave_y,
            wave_scale,
            75.0,
            95.0,
            wave_adj,
            LogoColorTheme::MonochromeWhite,
            true,
        );
    }
}

pub fn draw_payment_network(
    img: &mut RgbaImage,
    network: PaymentNetwork,
    badge_style: LogoBadgeStyle,
    color_theme: LogoColorTheme,
    custom_logo: Option<&RgbaImage>,
    logo_x: f32,
    logo_y: f32,
    logo_scale: f32,
    logo_adj: &LayerAdjustments,
) {
    let scale = logo_scale.clamp(0.2, 5.0);
    let bw = (245.0 * scale).round() as i32;
    let bh = (94.0 * scale).round() as i32;
    let bx0 = logo_x.round() as i32;
    let by0 = logo_y.round() as i32;
    let radius = (18.0 * scale).max(4.0);

    draw_logo_badge_frame(img, bx0, by0, bw, bh, radius, badge_style);

    let maybe_logo_img = match network {
        PaymentNetwork::None => None,
        PaymentNetwork::Visa => image::load_from_memory(ASSET_VISA).ok().map(|d| d.to_rgba8()),
        PaymentNetwork::Mastercard => image::load_from_memory(ASSET_MASTERCARD).ok().map(|d| d.to_rgba8()),
        PaymentNetwork::Napas => image::load_from_memory(ASSET_NAPAS).ok().map(|d| d.to_rgba8()),
        PaymentNetwork::Jcb => image::load_from_memory(ASSET_JCB).ok().map(|d| d.to_rgba8()),
        PaymentNetwork::Custom => custom_logo.cloned(),
    };

    if let Some(logo) = maybe_logo_img {
        let has_shadow = badge_style == LogoBadgeStyle::Transparent || badge_style == LogoBadgeStyle::ThinOutline;
        render_transformed_asset(
            img,
            &logo,
            logo_x,
            logo_y,
            logo_scale,
            245.0,
            94.0,
            logo_adj,
            color_theme,
            has_shadow,
        );
    }
}

pub fn draw_logo_badge_frame(
    img: &mut RgbaImage,
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    radius: f32,
    style: LogoBadgeStyle,
) {
    match style {
        LogoBadgeStyle::Transparent => {}
        LogoBadgeStyle::ThinOutline => {
            draw_rounded_rect_outline(img, x0, y0, w, h, radius, 2.0, [255, 255, 255, 130]);
        }
        LogoBadgeStyle::FrostedGlass => {
            draw_rounded_rect(img, x0 + 3, y0 + 4, w, h, radius, [0, 0, 0, 95]);
            draw_rounded_rect(img, x0, y0, w, h, radius, [255, 255, 255, 42]);
            draw_rounded_rect_outline(img, x0, y0, w, h, radius, 1.5, [255, 255, 255, 125]);
        }
        LogoBadgeStyle::SolidDark => {
            draw_rounded_rect(img, x0 + 3, y0 + 4, w, h, radius, [0, 0, 0, 130]);
            draw_rounded_rect(img, x0, y0, w, h, radius, [22, 24, 28, 235]);
            draw_rounded_rect_outline(img, x0, y0, w, h, radius, 1.0, [75, 80, 90, 150]);
        }
        LogoBadgeStyle::SolidLight => {
            draw_rounded_rect(img, x0 + 3, y0 + 4, w, h, radius, [0, 0, 0, 90]);
            draw_rounded_rect(img, x0, y0, w, h, radius, [248, 249, 252, 245]);
            draw_rounded_rect_outline(img, x0, y0, w, h, radius, 1.0, [210, 215, 225, 200]);
        }
        LogoBadgeStyle::SubtleGlow => {
            let cx = x0 + w / 2;
            let cy = y0 + h / 2;
            let max_r = 95.0f32;
            let r2 = max_r * max_r;
            for dy in -95..=95 {
                for dx in -120..=120 {
                    let d2 = (dx * dx + dy * dy) as f32;
                    if d2 <= r2 {
                        let factor = ((max_r - d2.sqrt()) / max_r).powi(2) * 45.0;
                        blend_pixel(img, cx + dx, cy + dy, 255, 255, 255, factor as u8);
                    }
                }
            }
        }
    }
}

fn render_text_subpixel(
    img: &mut RgbaImage,
    font: &FontArc,
    text: &str,
    start_x: f32,
    start_y: f32,
    scale_px: f32,
    color: [u8; 4],
    letter_spacing: f32,
) -> f32 {
    let scale = PxScale::from(scale_px);
    let scaled = font.as_scaled(scale);
    let mut cur_x = start_x;

    for c in text.chars() {
        if c == ' ' {
            cur_x += scaled.h_advance(font.glyph_id(' ')) + letter_spacing;
            continue;
        }
        let id = font.glyph_id(c);
        let advance = scaled.h_advance(id);
        let glyph = id.with_scale_and_position(scale, point(cur_x, start_y));
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, cov| {
                if cov > 0.01 {
                    let px = bounds.min.x as i32 + gx as i32;
                    let py = bounds.min.y as i32 + gy as i32;
                    let a = ((color[3] as f32) * cov).round() as u8;
                    blend_pixel(img, px, py, color[0], color[1], color[2], a);
                }
            });
        }
        cur_x += advance + letter_spacing;
    }
    cur_x
}

fn draw_embossed_string_vector(
    img: &mut RgbaImage,
    font: &FontArc,
    start_x: f32,
    start_y: f32,
    text: &str,
    scale_px: f32,
    style: EmbossStyle,
    letter_spacing: f32,
) -> f32 {
    let (shadow_col, highlight_col, face_col, is_3d) = match style {
        EmbossStyle::EmbossedSilver => (
            [10, 10, 15, 230],
            [255, 255, 255, 220],
            [225, 232, 240, 255],
            true,
        ),
        EmbossStyle::EmbossedGold => (
            [35, 25, 5, 230],
            [255, 250, 215, 220],
            [240, 198, 68, 255],
            true,
        ),
        EmbossStyle::CrispWhite => (
            [0, 0, 0, 210],
            [0, 0, 0, 0],
            [255, 255, 255, 255],
            false,
        ),
        EmbossStyle::StealthDark => (
            [80, 85, 95, 160],
            [200, 205, 215, 100],
            [35, 38, 45, 255],
            false,
        ),
    };

    if shadow_col[3] > 0 {
        let soft_alpha = (shadow_col[3] as f32 * 0.35) as u8;
        render_text_subpixel(
            img,
            font,
            text,
            start_x + 3.0,
            start_y + 4.0,
            scale_px,
            [shadow_col[0], shadow_col[1], shadow_col[2], soft_alpha],
            letter_spacing,
        );
        render_text_subpixel(
            img,
            font,
            text,
            start_x + 1.0,
            start_y + 3.0,
            scale_px,
            [shadow_col[0], shadow_col[1], shadow_col[2], soft_alpha],
            letter_spacing,
        );
        let mid_alpha = (shadow_col[3] as f32 * 0.70) as u8;
        render_text_subpixel(
            img,
            font,
            text,
            start_x + 2.0,
            start_y + 2.0,
            scale_px,
            [shadow_col[0], shadow_col[1], shadow_col[2], mid_alpha],
            letter_spacing,
        );
    }

    if is_3d && highlight_col[3] > 0 {
        render_text_subpixel(
            img,
            font,
            text,
            start_x - 1.5,
            start_y - 1.5,
            scale_px,
            highlight_col,
            letter_spacing,
        );
    }

    render_text_subpixel(
        img,
        font,
        text,
        start_x,
        start_y,
        scale_px,
        face_col,
        letter_spacing,
    )
}

fn draw_text_backdrop(img: &mut RgbaImage, backdrop: TextBackdropStyle, vertical_offset: i32) {
    match backdrop {
        TextBackdropStyle::None => {}
        TextBackdropStyle::FrostedGlassStrip => {
            let y0 = (530 + vertical_offset).clamp(100, 750);
            let h = 330;
            draw_rounded_rect(img, 80, y0, 1376, h, 24.0, [15, 18, 26, 125]);
            draw_rounded_rect_outline(img, 80, y0, 1376, h, 24.0, 1.5, [255, 255, 255, 45]);
        }
        TextBackdropStyle::SubtleDarkGradient => {
            let y_start = (480 + vertical_offset).clamp(200, 700) as u32;
            for y in y_start..CARD_HEIGHT {
                let factor = ((y - y_start) as f32 / (CARD_HEIGHT - y_start) as f32).powf(1.4);
                let alpha = (factor * 125.0) as u8;
                for x in 0..CARD_WIDTH {
                    blend_pixel(img, x as i32, y as i32, 12, 14, 20, alpha);
                }
            }
        }
        TextBackdropStyle::FrostedPills => {
            let vo = vertical_offset;
            let y_num = (575 + vo - 12).clamp(100, 850);
            draw_rounded_rect(img, 115, y_num, 930, 80, 18.0, [15, 18, 26, 115]);
            draw_rounded_rect_outline(img, 115, y_num, 930, 80, 18.0, 1.2, [255, 255, 255, 38]);

            let y_exp = (670 + vo - 8).clamp(100, 900);
            draw_rounded_rect(img, 520, y_exp, 320, 56, 14.0, [15, 18, 26, 115]);
            draw_rounded_rect_outline(img, 520, y_exp, 320, 56, 14.0, 1.2, [255, 255, 255, 38]);

            let y_name = (765 + vo - 8).clamp(100, 920);
            draw_rounded_rect(img, 115, y_name, 650, 64, 16.0, [15, 18, 26, 115]);
            draw_rounded_rect_outline(img, 115, y_name, 650, 64, 16.0, 1.2, [255, 255, 255, 38]);
        }
    }
}

pub fn resolve_font(
    preset: CardFontPreset,
    custom_font: Option<&[u8]>,
    is_number: bool,
) -> FontArc {
    if preset == CardFontPreset::Custom {
        if let Some(bytes) = custom_font {
            if let Ok(font) = FontArc::try_from_vec(bytes.to_vec()) {
                return font;
            }
        }
    }

    match preset {
        CardFontPreset::ClassicOcr => {
            FontArc::try_from_slice(ASSET_CARD_OCR).expect("ASSET_CARD_OCR must be valid")
        }
        CardFontPreset::ModernSans => {
            FontArc::try_from_slice(ASSET_CARD_SANS).expect("ASSET_CARD_SANS must be valid")
        }
        CardFontPreset::Monospace => {
            for font_path in [
                "C:\\Windows\\Fonts\\consola.ttf",
                "C:\\Windows\\Fonts\\consolab.ttf",
                "C:\\Windows\\Fonts\\lucon.ttf",
            ] {
                if let Ok(bytes) = std::fs::read(font_path) {
                    if let Ok(font) = FontArc::try_from_vec(bytes) {
                        return font;
                    }
                }
            }
            FontArc::try_from_slice(ASSET_CARD_OCR).expect("ASSET_CARD_OCR must be valid")
        }
        CardFontPreset::SerifLuxury => {
            for font_path in [
                "C:\\Windows\\Fonts\\georgiab.ttf",
                "C:\\Windows\\Fonts\\georgia.ttf",
                "C:\\Windows\\Fonts\\timesbd.ttf",
                "C:\\Windows\\Fonts\\times.ttf",
            ] {
                if let Ok(bytes) = std::fs::read(font_path) {
                    if let Ok(font) = FontArc::try_from_vec(bytes) {
                        return font;
                    }
                }
            }
            FontArc::try_from_slice(ASSET_CARD_SANS).expect("ASSET_CARD_SANS must be valid")
        }
        CardFontPreset::Custom => {
            if is_number {
                FontArc::try_from_slice(ASSET_CARD_OCR).expect("ASSET_CARD_OCR must be valid")
            } else {
                FontArc::try_from_slice(ASSET_CARD_SANS).expect("ASSET_CARD_SANS must be valid")
            }
        }
    }
}

pub fn draw_card_details(
    img: &mut RgbaImage,
    details: &CardDetails,
    details_adj: &LayerAdjustments,
    custom_font: Option<&[u8]>,
) {
    let num_font = resolve_font(details.number_font, custom_font, true);
    let text_font = resolve_font(details.text_font, custom_font, false);

    let vo = details.vertical_offset as f32;
    let ho = details.horizontal_offset as f32;
    let scale = if details.scale > 0.05 { details.scale } else { 1.0 };

    // 1. Draw text backdrop directly onto img
    draw_text_backdrop(img, details.backdrop, details.vertical_offset);

    // 2. Render all embossed details onto an intermediate layer
    let mut details_img = RgbaImage::new(CARD_WIDTH, CARD_HEIGHT);

    if !details.card_type_or_bank.is_empty() {
        draw_embossed_string_vector(
            &mut details_img,
            &text_font,
            140.0 + ho,
            125.0 + vo * 0.2,
            &details.card_type_or_bank.to_uppercase(),
            44.0 * scale,
            details.emboss_style,
            1.5 * scale,
        );
    }

    if !details.card_number.is_empty() {
        draw_embossed_string_vector(
            &mut details_img,
            &num_font,
            140.0 + ho,
            580.0 + vo,
            &details.card_number,
            58.0 * scale,
            details.emboss_style,
            3.0 * scale,
        );
    }

    if !details.card_expiry.is_empty() {
        draw_embossed_string_vector(
            &mut details_img,
            &text_font,
            535.0 + ho,
            672.0 + vo,
            "VALID THRU",
            22.0 * scale,
            details.emboss_style,
            1.0 * scale,
        );
        draw_embossed_string_vector(
            &mut details_img,
            &num_font,
            680.0 + ho,
            670.0 + vo,
            &details.card_expiry,
            38.0 * scale,
            details.emboss_style,
            2.0 * scale,
        );
    }

    if !details.card_holder.is_empty() {
        draw_embossed_string_vector(
            &mut details_img,
            &text_font,
            140.0 + ho,
            775.0 + vo,
            &details.card_holder.to_uppercase(),
            42.0 * scale,
            details.emboss_style,
            2.0 * scale,
        );
    }

    // 3. Composite details_img onto img applying details_adj (rotation, tint, opacity, hue, sat)
    let has_rotation = details_adj.rotation.abs() > 0.01;
    let center_x = 700.0f32 + ho;
    let center_y = 680.0f32 + vo;

    if has_rotation {
        let rad = (-details_adj.rotation).to_radians();
        let cos_r = rad.cos();
        let sin_r = rad.sin();

        let radius = 850.0f32;
        let min_y = ((center_y - radius).max(0.0) as u32).min(CARD_HEIGHT);
        let max_y = ((center_y + radius).min(CARD_HEIGHT as f32) as u32).min(CARD_HEIGHT);
        let min_x = ((center_x - radius).max(0.0) as u32).min(CARD_WIDTH);
        let max_x = ((center_x + radius).min(CARD_WIDTH as f32) as u32).min(CARD_WIDTH);

        for py in min_y..max_y {
            let dy = py as f32 + 0.5 - center_y;
            for px in min_x..max_x {
                let dx = px as f32 + 0.5 - center_x;
                let src_x = (dx * cos_r - dy * sin_r + center_x).round() as i32;
                let src_y = (dx * sin_r + dy * cos_r + center_y).round() as i32;

                if src_x >= 0 && src_x < CARD_WIDTH as i32 && src_y >= 0 && src_y < CARD_HEIGHT as i32 {
                    let p = details_img.get_pixel(src_x as u32, src_y as u32);
                    if p[3] > 0 {
                        let (out_r, out_g, out_b, out_a) = apply_adjustments_to_color(
                            p[0] as f32,
                            p[1] as f32,
                            p[2] as f32,
                            p[3] as f32,
                            details_adj,
                            LogoColorTheme::Original,
                        );
                        if out_a > 0 {
                            blend_pixel(img, px as i32, py as i32, out_r, out_g, out_b, out_a);
                        }
                    }
                }
            }
        }
    } else {
        // Fast path: direct blend with adjustments
        for py in 0..CARD_HEIGHT {
            for px in 0..CARD_WIDTH {
                let p = details_img.get_pixel(px, py);
                if p[3] > 0 {
                    let (out_r, out_g, out_b, out_a) = apply_adjustments_to_color(
                        p[0] as f32,
                        p[1] as f32,
                        p[2] as f32,
                        p[3] as f32,
                        details_adj,
                        LogoColorTheme::Original,
                    );
                    if out_a > 0 {
                        blend_pixel(img, px as i32, py as i32, out_r, out_g, out_b, out_a);
                    }
                }
            }
        }
    }
}

pub fn apply_custom_finish_texture(
    img: &mut RgbaImage,
    texture: &RgbaImage,
    pan_x: f32,
    pan_y: f32,
    scale: f32,
    opacity: f32,
    finish_adj: &LayerAdjustments,
) {
    let mut adj = *finish_adj;
    adj.opacity *= opacity;
    render_transformed_asset(
        img,
        texture,
        CARD_WIDTH as f32 * 0.5 + pan_x,
        CARD_HEIGHT as f32 * 0.5 + pan_y,
        scale,
        CARD_WIDTH as f32,
        CARD_HEIGHT as f32,
        &adj,
        LogoColorTheme::Original,
        false,
    );
}

pub fn apply_surface_finish(
    img: &mut RgbaImage,
    finish: CardFinish,
    custom_finish: Option<&RgbaImage>,
    options: &CardOverlayOptions,
) {
    match finish {
        CardFinish::Standard => {}
        CardFinish::MetallicSheen => {
            let cx = CARD_WIDTH as f32 * 0.45;
            let cy = CARD_HEIGHT as f32 * 0.45;

            for y in 0..CARD_HEIGHT {
                for x in 0..CARD_WIDTH {
                    let d = ((x as f32 + y as f32) - (cx + cy)).abs();
                    if d < 320.0 {
                        let factor = ((320.0 - d) / 320.0).powi(2) * 0.28;
                        let px = img.get_pixel_mut(x, y);
                        let r = (px[0] as f32 + (255.0 - px[0] as f32) * factor).min(255.0) as u8;
                        let g = (px[1] as f32 + (255.0 - px[1] as f32) * factor).min(255.0) as u8;
                        let b = (px[2] as f32 + (255.0 - px[2] as f32) * factor).min(255.0) as u8;
                        *px = Rgba([r, g, b, 255]);
                    }
                }
            }
        }
        CardFinish::CarbonWeave => {
            for y in 0..CARD_HEIGHT {
                for x in 0..CARD_WIDTH {
                    let block_x = (x / 6) % 2;
                    let block_y = (y / 6) % 2;
                    let is_bright = block_x == block_y;
                    let delta = if is_bright { 18.0 } else { -18.0 };

                    let px = img.get_pixel_mut(x, y);
                    let r = (px[0] as f32 + delta).clamp(0.0, 255.0) as u8;
                    let g = (px[1] as f32 + delta).clamp(0.0, 255.0) as u8;
                    let b = (px[2] as f32 + delta).clamp(0.0, 255.0) as u8;
                    *px = Rgba([r, g, b, 255]);
                }
            }
        }
        CardFinish::CustomTexture => {
            if let Some(texture) = custom_finish {
                apply_custom_finish_texture(
                    img,
                    texture,
                    options.finish_x,
                    options.finish_y,
                    options.finish_scale,
                    options.finish_opacity,
                    &options.finish_adj,
                );
            }
        }
    }
}

pub fn apply_card_rounded_corners(img: &mut RgbaImage, radius: f32) {
    let w = img.width() as f32;
    let h = img.height() as f32;
    let r2 = radius * radius;
    let r_inner = radius - 0.75;
    let r_outer = radius + 0.75;

    for y in 0..img.height() {
        let yf = y as f32 + 0.5;
        let cy = if yf < radius {
            radius
        } else if yf > h - radius {
            h - radius
        } else {
            continue;
        };

        for x in 0..img.width() {
            let xf = x as f32 + 0.5;
            let cx = if xf < radius {
                radius
            } else if xf > w - radius {
                w - radius
            } else {
                continue;
            };

            let dx = xf - cx;
            let dy = yf - cy;
            let dist2 = dx * dx + dy * dy;

            if dist2 > r2 {
                let dist = dist2.sqrt();
                if dist >= r_outer {
                    img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                } else {
                    let coverage = ((r_outer - dist) / (r_outer - r_inner)).clamp(0.0, 1.0);
                    let p = img.get_pixel_mut(x, y);
                    p[3] = (p[3] as f32 * coverage).round() as u8;
                }
            }
        }
    }
}

pub fn render_preview_canvas(
    maybe_image: Option<&DynamicImage>,
    options: &CardOverlayOptions,
    custom_logo: Option<&RgbaImage>,
    custom_chip: Option<&RgbaImage>,
    custom_finish: Option<&RgbaImage>,
    custom_font: Option<&[u8]>,
    custom_widgets: &[(&CustomWidgetData, &RgbaImage)],
) -> Result<(RgbaImage, egui::ColorImage)> {
    let mut rgba = generate_card_preset_canvas(
        options.bg_preset,
        maybe_image,
        options.transform,
        &options.bg_adj,
    );

    // 1. Apply surface finish effects (including Custom Texture if uploaded)
    apply_surface_finish(&mut rgba, options.finish, custom_finish, options);

    // 2. Draw EMV Gold / Custom Chip
    if options.show_chip {
        draw_emv_chip(
            &mut rgba,
            custom_chip,
            options.chip_x,
            options.chip_y,
            options.chip_scale,
            &options.chip_adj,
        );
    }

    // 3. Draw Contactless Waves (Shockwave) with position, scale, and rotation
    if options.show_contactless {
        draw_contactless_wave(
            &mut rgba,
            options.wave_x,
            options.wave_y,
            options.wave_scale,
            &options.wave_adj,
        );
    }

    // 4. Draw Payment Network Brand Badge with Frame, Color Theme, and custom position/scale/rotation
    if options.network != PaymentNetwork::None {
        draw_payment_network(
            &mut rgba,
            options.network,
            options.logo_style,
            options.logo_color,
            custom_logo,
            options.logo_x,
            options.logo_y,
            options.logo_scale,
            &options.logo_adj,
        );
    }

    // 5. Draw Card Details (Embossed numbers, cardholder, expiry, bank name)
    if options.details.show_details {
        draw_card_details(&mut rgba, &options.details, &options.details_adj, custom_font);
    }

    // 5.5. Draw Custom User Widgets
    for (w_data, w_img) in custom_widgets {
        if w_data.visible {
            render_transformed_asset(
                &mut rgba,
                w_img,
                w_data.x,
                w_data.y,
                w_data.scale,
                w_data.base_w,
                w_data.base_h,
                &w_data.adjustments,
                LogoColorTheme::Original,
                w_data.has_shadow,
            );
        }
    }

    // 6. Clip Card to authentic ISO 7810 rounded corners (Anti-aliased)
    apply_card_rounded_corners(&mut rgba, 58.0);

    let preview = egui::ColorImage::from_rgba_unmultiplied(
        [CARD_WIDTH as usize, CARD_HEIGHT as usize],
        rgba.as_raw(),
    );

    Ok((rgba, preview))
}
