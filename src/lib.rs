#![no_std]

use core::cmp::Ordering;

mod glyph;
pub use glyph::{Glyph, GlyphProvider};

include!(concat!(env!("OUT_DIR"), "/glyph_data.rs"));

fn glyph_index(codepoint: u32) -> Option<usize> {
    let idx = RANGES
        .binary_search_by(|&(start, end, _)| {
            if codepoint < start {
                Ordering::Greater
            } else if codepoint > end {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .ok()?;
    let (start, _, offset) = RANGES[idx];
    Some(offset as usize + (codepoint - start) as usize)
}

fn unpack(packed: u8) -> (u8, GlyphProvider) {
    (packed & 0x1F, GlyphProvider::from_u8(packed >> 5))
}

pub fn advance(codepoint: char) -> f32 {
    advance_bold(codepoint, false)
}

pub fn advance_bold(codepoint: char, bold: bool) -> f32 {
    let idx = glyph_index(codepoint as u32);
    match idx {
        Some(glyph_idx) => {
            let (base, provider) = unpack(GLYPH_DATA[glyph_idx]);
            let offset = if bold { provider.bold_offset() } else { 0.0 };
            base as f32 + offset
        }
        None => 0.0,
    }
}

pub fn string_width(s: &str) -> f32 {
    string_width_bold(s, false)
}

pub fn string_width_bold(s: &str, bold: bool) -> f32 {
    s.chars().map(|c| advance_bold(c, bold)).sum()
}

pub fn split_at_width(s: &str, max_width: f32) -> (&str, &str) {
    split_at_width_bold(s, max_width, false)
}

pub fn split_at_width_bold(s: &str, max_width: f32, bold: bool) -> (&str, &str) {
    let mut width = 0.0_f32;
    for (byte_offset, ch) in s.char_indices() {
        let ch_width = advance_bold(ch, bold);
        if width + ch_width > max_width {
            return (&s[..byte_offset], &s[byte_offset..]);
        }
        width += ch_width;
    }
    (s, "")
}

pub fn glyph(codepoint: char) -> Option<Glyph> {
    let idx = glyph_index(codepoint as u32)?;
    let (advance, provider) = unpack(GLYPH_DATA[idx]);
    Some(make_glyph(idx, advance, provider))
}

#[cfg(feature = "bitmaps")]
fn make_glyph(idx: usize, advance: u8, provider: GlyphProvider) -> Glyph {
    let offset = BITMAP_OFFSETS[idx] as usize;
    let len = provider.pixel_data_len();
    // for 0-length glyphs (space provider) this will be an empty slice
    let data = &BITMAP_DATA[offset..][..len];
    Glyph::new(advance, provider, data)
}

#[cfg(not(feature = "bitmaps"))]
fn make_glyph(_idx: usize, advance: u8, provider: GlyphProvider) -> Glyph {
    Glyph::new(advance, provider)
}
