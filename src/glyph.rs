// TODO: unknown characters (hardcoded 5*8 box in minecraft src)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlyphProvider {
    Space,
    Mojangles8x8,
    Mojangles9x12,
    UnifontHalfwidth,
    UnifontFullwidth,
}

impl GlyphProvider {
    pub(crate) fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Space,
            1 => Self::Mojangles8x8,
            2 => Self::Mojangles9x12,
            3 => Self::UnifontHalfwidth,
            4 => Self::UnifontFullwidth,
            _ => Self::Space,
        }
    }

    pub(crate) fn pixel_data_len(self) -> usize {
        let (w, h) = self.dimensions();
        (w as usize).div_ceil(8) * h as usize
    }

    pub fn dimensions(self) -> (u8, u8) {
        match self {
            Self::Space => (0, 0),
            Self::Mojangles8x8 => (8, 8),
            Self::Mojangles9x12 => (9, 12),
            Self::UnifontHalfwidth => (8, 16),
            Self::UnifontFullwidth => (16, 16),
        }
    }

    pub fn ascent(self) -> u8 {
        match self {
            Self::Space => 0,
            Self::Mojangles8x8 => 7,
            Self::Mojangles9x12 => 10,
            Self::UnifontHalfwidth => 14,
            Self::UnifontFullwidth => 14,
        }
    }

    pub fn bold_offset(self) -> f32 {
        match self {
            Self::UnifontHalfwidth | Self::UnifontFullwidth => 0.5,
            _ => 1.0,
        }
    }
}

pub struct Glyph {
    base_advance: u8,
    pub provider: GlyphProvider,
    pub width: u8,
    pub height: u8,
    pub ascent: u8,
    #[cfg(feature = "bitmaps")]
    data: &'static [u8],
}

impl Glyph {
    pub(crate) fn new(
        base_advance: u8,
        provider: GlyphProvider,
        #[cfg(feature = "bitmaps")] data: &'static [u8],
    ) -> Self {
        let (width, height) = provider.dimensions();
        let ascent = provider.ascent();
        Glyph {
            base_advance,
            provider,
            width,
            height,
            ascent,
            #[cfg(feature = "bitmaps")]
            data,
        }
    }

    pub fn advance(&self) -> f32 {
        self.base_advance as f32
    }

    pub fn advance_bold(&self) -> f32 {
        self.base_advance as f32 + self.provider.bold_offset()
    }

    #[cfg(feature = "bitmaps")]
    pub fn pixel(&self, x: u8, y: u8) -> Option<bool> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let bytes_per_row = (self.width as usize).div_ceil(8);
        let byte_idx = y as usize * bytes_per_row + (x as usize / 8);
        let bit = 1 << (7 - (x % 8));
        self.data.get(byte_idx).map(|&b| (b & bit) != 0)
    }

    #[cfg(feature = "bitmaps")]
    pub fn rows(&self) -> GlyphRows<'_> {
        GlyphRows {
            data: self.data,
            bytes_per_row: (self.width as usize).div_ceil(8),
            row: 0,
            height: self.height,
        }
    }
}

#[cfg(feature = "bitmaps")]
pub struct GlyphRows<'a> {
    data: &'a [u8],
    bytes_per_row: usize,
    row: u8,
    height: u8,
}

#[cfg(feature = "bitmaps")]
impl<'a> Iterator for GlyphRows<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        if self.row >= self.height {
            return None;
        }
        let start = self.row as usize * self.bytes_per_row;
        self.row += 1;
        Some(&self.data[start..start + self.bytes_per_row])
    }
}
