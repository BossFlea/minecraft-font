use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

use serde::Deserialize;

const DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");
const OUTPUT_FILE: &str = "glyph_data.rs";

fn main() {
    let mut glyphs: BTreeMap<u32, GlyphData> = BTreeMap::new();

    process_space(&mut glyphs);
    process_mojangles(&mut glyphs);
    process_unifont(&mut glyphs);

    let ranges = build_ranges(&glyphs);
    let output_path = Path::new(&env::var("OUT_DIR").unwrap()).join(OUTPUT_FILE);
    let mut out = BufWriter::new(File::create(output_path).unwrap());

    write_output(&mut out, &glyphs, &ranges);
}

#[derive(Clone, Debug)]
struct GlyphData {
    advance: u8,
    provider: u8,
    #[allow(dead_code)]
    rows: Vec<u8>,
}

fn process_space(glyphs: &mut BTreeMap<u32, GlyphData>) {
    let data = std::fs::read_to_string(format!("{DATA_DIR}/space.json")).unwrap();
    let parsed: SpaceJson = serde_json::from_str(&data).unwrap();
    for provider in parsed.providers {
        for (ch, advance) in provider.advances {
            let cp = ch.chars().next().unwrap() as u32;
            glyphs.entry(cp).or_insert(GlyphData {
                advance,
                provider: 0,
                rows: Vec::new(),
            });
        }
    }
}

fn process_mojangles(glyphs: &mut BTreeMap<u32, GlyphData>) {
    let data = std::fs::read_to_string(format!("{DATA_DIR}/mojangles/default.json")).unwrap();
    let parsed: MojanglesJson = serde_json::from_str(&data).unwrap();

    for raw in parsed.providers {
        let (file, gw, gh) = match raw.file.as_str() {
            "minecraft:font/ascii.png" => ("mojangles/ascii.png", 8u32, 8u32),
            "minecraft:font/accented.png" => ("mojangles/accented.png", 9u32, 12u32),
            "minecraft:font/nonlatin_european.png" => {
                ("mojangles/nonlatin_european.png", 8u32, 8u32)
            }
            _ => continue,
        };
        let img = load_png(file);
        let bpr = gw.div_ceil(8) as usize;

        for (y, row) in raw.chars.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let cp = ch as u32;
                let (ox, oy) = (x as u32 * gw, y as u32 * gh);
                let mut rows = vec![0u8; gh as usize * bpr];
                for r in 0..gh {
                    for c in 0..gw {
                        let px = img.get_pixel(ox + c, oy + r);
                        if px.0[3] != 0 {
                            let bi = r as usize * bpr + (c as usize / 8);
                            rows[bi] |= 1 << (7 - (c % 8));
                        }
                    }
                }
                let actual = mojangles_rightmost_width(&rows, gw);
                let advance = ((0.5 + actual as f64) as u8) + 1;
                let prov = if gw == 8 { 1 } else { 2 };
                glyphs.entry(cp).or_insert(GlyphData {
                    advance,
                    provider: prov,
                    rows,
                });
            }
        }
    }
}

fn process_unifont(glyphs: &mut BTreeMap<u32, GlyphData>) {
    let cfg = std::fs::read_to_string(format!("{DATA_DIR}/unifont.json")).unwrap();
    let parsed: UnihexJson = serde_json::from_str(&cfg).unwrap();

    let overrides: Vec<OverrideRange> = parsed
        .providers
        .iter()
        .filter(|p| p.hex_file == "minecraft:font/unifont.zip")
        .flat_map(|p| &p.size_overrides)
        .map(|o| OverrideRange {
            from: o.from.chars().next().unwrap() as u32,
            to: o.to.chars().next().unwrap() as u32,
            left: o.left,
            right: o.right,
        })
        .collect();

    let zip_data = std::fs::read(format!("{DATA_DIR}/unifont.zip")).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_data)).unwrap();
    let hex_content = {
        let mut f = archive.by_name("unifont_all_no_pua-16.0.03.hex").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };

    for line in hex_content.lines() {
        let Some((cp_hex, data_hex)) = line.split_once(':') else {
            continue;
        };
        let Ok(cp) = u32::from_str_radix(cp_hex, 16) else {
            continue;
        };
        let rows = match data_hex.len() {
            32 => parse_unifont_halfwidth(data_hex),
            64 => parse_unifont_fullwidth(data_hex),
            _ => continue,
        };
        let width = compute_unifont_width(cp, &rows, &overrides);
        let advance = (width / 2 + 1) as u8;
        let prov = if rows.len() == 32 { 4 } else { 3 };
        glyphs.entry(cp).or_insert(GlyphData {
            advance,
            provider: prov,
            rows,
        });
    }
}

fn mojangles_rightmost_width(rows: &[u8], glyph_w: u32) -> u8 {
    let bpr = glyph_w.div_ceil(8) as usize;
    if bpr == 0 {
        return 0;
    }
    let gh = rows.len() / bpr;
    let mut rightmost = 0u32;
    let mut found = false;
    for r in 0..gh {
        for c in 0..glyph_w {
            let bi = r * bpr + (c as usize / 8);
            if rows[bi] & (1 << (7 - (c % 8))) != 0 {
                found = true;
                if c > rightmost {
                    rightmost = c;
                }
            }
        }
    }
    if found { (rightmost + 1) as u8 } else { 0 }
}

fn compute_unifont_width(cp: u32, rows: &[u8], overrides: &[OverrideRange]) -> u8 {
    for o in overrides {
        if cp >= o.from && cp <= o.to {
            return o.right - o.left + 1;
        }
    }

    let is_fullwidth = rows.len() == 32;
    if is_fullwidth {
        let mut or_mask = 0u16;
        for i in 0..16 {
            let row = ((rows[i * 2] as u16) << 8) | rows[i * 2 + 1] as u16;
            or_mask |= row;
        }
        if or_mask == 0 {
            return 0;
        }
        let left = or_mask.leading_zeros();
        let right = 15 - or_mask.trailing_zeros();
        (right - left + 1) as u8
    } else {
        let mut or_mask = 0u8;
        for &row in rows {
            or_mask |= row;
        }
        if or_mask == 0 {
            return 0;
        }
        let left = or_mask.leading_zeros();
        let right = 7 - or_mask.trailing_zeros();
        (right - left + 1) as u8
    }
}

// TODO: check if some of these can return fixed-size arrays instead of dynamic Vecs
fn parse_unifont_halfwidth(hex: &str) -> Vec<u8> {
    let bytes = hex.as_bytes();
    let mut rows = vec![0u8; 16];
    for i in 0..16 {
        rows[i] = hex_val(bytes[i * 2]) << 4 | hex_val(bytes[i * 2 + 1]);
    }
    rows
}

fn parse_unifont_fullwidth(hex: &str) -> Vec<u8> {
    let bytes = hex.as_bytes();
    let mut rows = vec![0u8; 32];
    for i in 0..16 {
        let hi = hex_val(bytes[i * 4]) << 4 | hex_val(bytes[i * 4 + 1]);
        let lo = hex_val(bytes[i * 4 + 2]) << 4 | hex_val(bytes[i * 4 + 3]);
        rows[i * 2] = hi;
        rows[i * 2 + 1] = lo;
    }
    rows
}

fn hex_val(ch: u8) -> u8 {
    match ch {
        b'0'..=b'9' => ch - b'0',
        b'a'..=b'f' => ch - b'a' + 10,
        b'A'..=b'F' => ch - b'A' + 10,
        _ => 0,
    }
}

fn load_png(rel: &str) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    image::open(format!("{DATA_DIR}/{rel}")).unwrap().to_rgba8()
}

fn build_ranges(glyphs: &BTreeMap<u32, GlyphData>) -> Vec<(u32, u32, u32)> {
    let mut ranges = Vec::new();
    let mut iter = glyphs.keys().copied();
    let Some(first) = iter.next() else {
        return ranges;
    };
    let mut start = first;
    let mut prev = first;
    let mut start_offset = 0u32;
    let mut offset = 1u32;
    for cp in iter {
        if cp == prev + 1 {
            prev = cp;
            offset += 1;
        } else {
            ranges.push((start, prev, start_offset));
            start = cp;
            prev = cp;
            start_offset += offset;
            offset = 1;
        }
    }
    ranges.push((start, prev, start_offset));
    ranges
}

fn write_output(
    out: &mut BufWriter<File>,
    glyphs: &BTreeMap<u32, GlyphData>,
    ranges: &[(u32, u32, u32)],
) {
    writeln!(out, "pub(crate) static RANGES: &[(u32, u32, u32)] = &[").unwrap();
    for &(start, end, offset) in ranges {
        writeln!(out, "    (0x{start:06X}, 0x{end:06X}, {offset}),").unwrap();
    }
    writeln!(out, "];").unwrap();

    writeln!(out, "pub(crate) static GLYPH_DATA: &[u8] = &[").unwrap();
    let values: Vec<u8> = glyphs
        .values()
        .map(|g| (g.provider << 5) | g.advance)
        .collect();
    for chunk in values.chunks(20) {
        write!(out, "   ").unwrap();
        for &v in chunk {
            write!(out, " 0x{v:02X},").unwrap();
        }
        writeln!(out).unwrap();
    }
    writeln!(out, "];").unwrap();

    #[cfg(feature = "bitmaps")]
    write_bitmaps(out, glyphs);
}

#[cfg(feature = "bitmaps")]
fn write_bitmaps(out: &mut BufWriter<File>, glyphs: &BTreeMap<u32, GlyphData>) {
    writeln!(out, "pub(crate) static BITMAP_OFFSETS: &[u32] = &[").unwrap();
    let mut flat: Vec<u8> = Vec::new();
    for g in glyphs.values() {
        if g.rows.is_empty() {
            writeln!(out, "    0,").unwrap();
        } else {
            writeln!(out, "    {},", flat.len()).unwrap();
            flat.extend_from_slice(&g.rows);
        }
    }
    writeln!(out, "];").unwrap();

    writeln!(out, "pub(crate) static BITMAP_DATA: &[u8] = &[").unwrap();
    for chunk in flat.chunks(30) {
        write!(out, "   ").unwrap();
        for &b in chunk {
            write!(out, " {b},").unwrap();
        }
        writeln!(out).unwrap();
    }
    writeln!(out, "];").unwrap();
}

#[derive(Deserialize)]
struct SpaceJson {
    providers: Vec<SpaceProvider>,
}

#[derive(Deserialize)]
struct SpaceProvider {
    advances: BTreeMap<String, u8>,
}

#[derive(Deserialize)]
struct MojanglesJson {
    providers: Vec<MojanglesRawProvider>,
}

#[derive(Deserialize)]
struct MojanglesRawProvider {
    #[allow(dead_code)]
    #[serde(rename = "type")]
    kind: String,
    file: String,
    #[allow(dead_code)]
    ascent: Option<u8>,
    chars: Vec<String>,
}

#[derive(Deserialize)]
struct UnihexJson {
    providers: Vec<UnihexProvider>,
}

#[derive(Deserialize)]
struct UnihexProvider {
    #[allow(dead_code)]
    hex_file: String,
    size_overrides: Vec<SizeOverride>,
}

#[derive(Deserialize)]
struct SizeOverride {
    from: String,
    to: String,
    left: u8,
    right: u8,
}

struct OverrideRange {
    from: u32,
    to: u32,
    left: u8,
    right: u8,
}
