#[test]
fn test_hex_token() {
    let mut bytes = [0u8; 10];
    unsafe {
        #[link(name = "bcrypt")]
        unsafe extern "system" {
            fn BCryptGenRandom(
                hAlgorithm: *mut std::ffi::c_void,
                pbBuffer: *mut u8,
                cbBuffer: u32,
                dwFlags: u32,
            ) -> i32;
        }
        let status = BCryptGenRandom(std::ptr::null_mut(), bytes.as_mut_ptr(), 10, 2);
        assert_eq!(status, 0);
    }
    let token: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    assert_eq!(token.len(), 20);
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_font_load() {
    use ab_glyph::Font;
    if let Ok(sym_bytes) = std::fs::read("C:\\Windows\\Fonts\\seguisym.ttf") {
        let sym_font = ab_glyph::FontRef::try_from_slice(&sym_bytes).unwrap();
        assert_ne!(sym_font.glyph_id('\u{1F4BE}'), ab_glyph::GlyphId(0));
        assert_ne!(sym_font.glyph_id('\u{1F4C2}'), ab_glyph::GlyphId(0));
        assert_ne!(sym_font.glyph_id('\u{1F5BC}'), ab_glyph::GlyphId(0));
        assert_ne!(sym_font.glyph_id('\u{1F3F7}'), ab_glyph::GlyphId(0));
        assert_ne!(sym_font.glyph_id('\u{1F4B3}'), ab_glyph::GlyphId(0));
        assert_ne!(sym_font.glyph_id('\u{270D}'), ab_glyph::GlyphId(0));
        assert_ne!(sym_font.glyph_id('\u{1F4E6}'), ab_glyph::GlyphId(0));
    }
}
