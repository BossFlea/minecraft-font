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
fn euro_advance() {
    let adv = minecraft_font::advance('\u{20AC}'); // € in nonlatin_european
    assert!(adv > 0.0, "euro advance={adv}, expected > 0");
}

#[test]
fn glyph_provider() {
    let space = minecraft_font::glyph(' ').unwrap();
    assert_eq!(space.provider, minecraft_font::GlyphProvider::Space);
    assert_eq!(space.width, 0);
    assert_eq!(space.height, 0);

    let a = minecraft_font::glyph('A').unwrap();
    assert_eq!(a.provider, minecraft_font::GlyphProvider::Mojangles8x8);
    assert_eq!(a.width, 8);
    assert_eq!(a.height, 8);
    assert_eq!(a.ascent, 7);

    let euro = minecraft_font::glyph('\u{00C0}').unwrap(); // À in accented
    assert_eq!(euro.provider, minecraft_font::GlyphProvider::Mojangles9x12);
    assert_eq!(euro.width, 9);
    assert_eq!(euro.height, 12);
    assert_eq!(euro.ascent, 10);

    let cjk = minecraft_font::glyph('\u{4E2D}').unwrap();
    assert_eq!(
        cjk.provider,
        minecraft_font::GlyphProvider::UnifontFullwidth
    );
    assert_eq!(cjk.width, 16);
    assert_eq!(cjk.height, 16);
    assert_eq!(cjk.ascent, 14);
}

#[cfg(feature = "bitmaps")]
#[test]
fn glyph_pixel_access() {
    let glyph = minecraft_font::glyph('A').unwrap();
    assert!(glyph.rows().any(|row| row.iter().any(|&b| b != 0)));
}

#[cfg(feature = "bitmaps")]
#[test]
fn glyph_cjk_pixels() {
    let glyph = minecraft_font::glyph('\u{4E2D}').unwrap();
    assert!(glyph.rows().any(|row| row.iter().any(|&b| b != 0)));
}
