#[test]
fn space_advance() {
    let adv = minecraft_font::advance(' ');
    assert!(
        (adv - 4.0).abs() < 0.01,
        "space advance={adv}, expected 4.0"
    );
}

#[test]
fn zwnj_advance() {
    let adv = minecraft_font::advance('\u{200c}');
    assert!((adv - 0.0).abs() < 0.01, "ZWNJ advance={adv}, expected 0.0");
}

#[test]
fn null_advance() {
    let adv = minecraft_font::advance('\0');
    // null exists as mojangles empty glyph
    assert!((adv - 1.0).abs() < 0.01, "null advance={adv}, expected 1.0");
}

#[test]
fn glyph_a_advance() {
    let adv = minecraft_font::advance('A');
    assert!(adv > 0.0, "'A' advance={adv}, expected > 0");
}

#[test]
fn cjk_advance() {
    let adv = minecraft_font::advance('\u{4E2D}'); // 中
    assert!((adv - 9.0).abs() < 0.01, "CJK advance={adv}, expected 9.0");
}

#[test]
fn hangul_syllable_advance() {
    let adv = minecraft_font::advance('\u{AC00}'); // 가
    // Hangul Syllables size_overrides: left=1, right=15 -> width=15 -> advance=15/2+1=8
    assert!(
        (adv - 8.0).abs() < 0.01,
        "Hangul advance={adv}, expected 8.0"
    );
}

#[test]
fn fullwidth_forms_advance() {
    let adv = minecraft_font::advance('\u{FF01}'); // ！
    // Fullwidth Forms: left=0, right=15 -> width=16 -> advance=9
    assert!(
        (adv - 9.0).abs() < 0.01,
        "Fullwidth advance={adv}, expected 9.0"
    );
}

#[test]
fn hangul_jamo_advance() {
    let adv = minecraft_font::advance('\u{1100}'); // ᄀ
    // Hangul Jamo: left=0, right=15 -> width=16 -> advance=9
    assert!(
        (adv - 9.0).abs() < 0.01,
        "Hangul Jamo advance={adv}, expected 9.0"
    );
}

#[test]
fn string_width_basic() {
    let w = minecraft_font::string_width("Hello!");
    assert!(w > 0.0);
}

#[test]
fn split_at_width_works() {
    let (a, _b) = minecraft_font::split_at_width("Hello World", 20.0);
    assert!(!a.is_empty());
}

#[test]
fn bold_advance_mojangles() {
    let normal = minecraft_font::advance('A');
    let bold = minecraft_font::advance_bold('A', true);
    assert!(
        (bold - normal - 1.0).abs() < 0.01,
        "'A' bold offset: normal={normal} bold={bold}, expected diff 1.0"
    );
}

#[test]
fn bold_advance_unifont() {
    let normal = minecraft_font::advance('\u{4E2D}');
    let bold = minecraft_font::advance_bold('\u{4E2D}', true);
    assert!(
        (bold - normal - 0.5).abs() < 0.01,
        "CJK bold offset: normal={normal} bold={bold}, expected diff 0.5"
    );
}

#[test]
fn em_quad_advance() {
    let adv = minecraft_font::advance('\u{2001}'); // half-width empty glyph
    assert!((adv - 5.0) < 0.01, "em quad advance={adv}, expected 5.0");
}

#[test]
fn ideographic_space_advance() {
    let adv = minecraft_font::advance('\u{3000}'); // full-width empty glyph
    assert!(
        (adv - 9.0) < 0.01,
        "ideographic space advance={adv}, expected 9.0"
    );
}

#[test]
fn glyph_provider() {
    let space = minecraft_font::glyph(' ');
    assert_eq!(space.provider, minecraft_font::GlyphProvider::Space);
    assert_eq!(space.width, 0);
    assert_eq!(space.height, 0);

    let a = minecraft_font::glyph('A');
    assert_eq!(a.provider, minecraft_font::GlyphProvider::Mojangles8x8);
    assert_eq!(a.width, 8);
    assert_eq!(a.height, 8);
    assert_eq!(a.ascent, 7);

    let euro = minecraft_font::glyph('\u{00C0}'); // À in accented
    assert_eq!(euro.provider, minecraft_font::GlyphProvider::Mojangles9x12);
    assert_eq!(euro.width, 9);
    assert_eq!(euro.height, 12);
    assert_eq!(euro.ascent, 10);

    let cjk = minecraft_font::glyph('\u{4E2D}');
    assert_eq!(
        cjk.provider,
        minecraft_font::GlyphProvider::UnifontFullwidth
    );
    assert_eq!(cjk.width, 16);
    assert_eq!(cjk.height, 16);
    assert_eq!(cjk.ascent, 14);
}

#[test]
fn missing_glyph() {
    // U+FFFF is a defined as a noncharacter
    let g = minecraft_font::glyph('\u{FFFF}');
    assert_eq!(g.provider, minecraft_font::GlyphProvider::Missing);
    assert_eq!(g.width, 5);
    assert_eq!(g.height, 8);
    assert_eq!(g.ascent, 7);
    assert!((g.advance() - 6.0).abs() < 0.01);
    assert!((g.advance_bold() - 7.0).abs() < 0.01);

    let adv = minecraft_font::advance('\u{FFFF}');
    assert!((adv - 6.0).abs() < 0.01);
}

#[test]
fn missing_glyph_bold_advance() {
    let normal = minecraft_font::advance('\u{FFFF}');
    let bold = minecraft_font::advance_bold('\u{FFFF}', true);
    assert!(
        (bold - normal - 1.0).abs() < 0.01,
        "missing bold offset: normal={normal} bold={bold}, expected diff 1.0"
    );
}

#[cfg(feature = "bitmaps")]
#[test]
fn glyph_pixel_access() {
    let glyph = minecraft_font::glyph('A');
    assert!(glyph.rows().any(|row| row.iter().any(|&b| b != 0)));
}

#[cfg(feature = "bitmaps")]
#[test]
fn glyph_cjk_pixels() {
    let glyph = minecraft_font::glyph('\u{4E2D}');
    assert!(glyph.rows().any(|row| row.iter().any(|&b| b != 0)));
}

#[cfg(feature = "bitmaps")]
#[test]
fn missing_glyph_pixels() {
    let g = minecraft_font::glyph('\u{FFFF}');
    // border pixels should be set
    assert_eq!(g.pixel(0, 0), Some(true)); // top-left corner
    assert_eq!(g.pixel(2, 2), Some(false)); // interior
    assert_eq!(g.pixel(4, 7), Some(true)); // bottom-right corner
    assert_eq!(g.pixel(5, 0), None); // out of bounds
}
