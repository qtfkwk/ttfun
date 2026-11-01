#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/t/LIB.md"))]

//--------------------------------------------------------------------------------------------------
// Crates

use {
    ab_glyph::PxScale,
    anyhow::{Result, anyhow},
    clap::ValueEnum,
    image::{
        DynamicImage, GenericImage, ImageBuffer, Rgb, RgbImage, buffer::ConvertBuffer,
        imageops::FilterType,
    },
    imageproc::drawing::{draw_text_mut, text_size},
    indexmap::IndexSet,
    owo_colors::OwoColorize,
    std::{fmt::Write, path::Path},
};

pub use ab_glyph::FontArc;

//--------------------------------------------------------------------------------------------------
// Macros

/// Conditional [`println`]
#[macro_export]
macro_rules! vprintln {
    ($verbose:expr, $($arg:tt)*) => {
        if $verbose {
            println!($($arg)*);
        }
    };
}

//--------------------------------------------------------------------------------------------------
// Constants

const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
const WHITE: Rgb<u8> = Rgb([255, 255, 255]);

const DEFAULT_RESCALE: Rescale = Rescale {
    width: 1.0,
    height: 1.0,
};

/// Embedded EB Garamond TTF font data
pub const EB_GARAMOND: &[u8] = include_bytes!("../fonts/eb-garamond/eb-garamond.ttf");

/// Embedded Iosevka TTF font data
pub const IOSEVKA: &[u8] = include_bytes!("../fonts/iosevka/iosevka.ttf");

//--------------------------------------------------------------------------------------------------
// Enums

/// Unicode bitmap style
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ValueEnum)]
pub enum Style {
    /// `█𜷥𜷤▆𜷣▟𜷢𜷡𜷠𜷟▙𜷞𜷝𜷜𜷛▄...`
    Octant,

    /// `⣿⣾⣽⣼⣻⣺⣹⣸⣷⣶⣵⣴⣳⣲⣱⣰...`
    OctantSep,

    /// `█🬻🬺🬹🬸🬷🬶🬵🬴🬳🬲🬱🬰🬯🬮🬭...`
    Sextant,

    /// `𜺏𜺎𜺍𜺌𜺋𜺊𜺉𜺈𜺇𜺆𜺅𜺄𜺃𜺂𜺁𜺀...`
    SextantSep,

    /// `█▛▜▀▙▌▚▘▟▞▐▝▄▖▗ `
    Quadrant,

    /// `𜰯𜰧𜰫𜰣𜰭𜰥𜰩𜰡𜰮𜰦𜰪𜰢𜰬𜰤𜰨 `
    QuadrantSep,

    /// `█▀▄ `
    Half,

    /// `█ `
    Full,

    /// `█` (colorized)
    FullColor,

    /// `█▓▒░ `
    FullShade,

    /// All styles
    All,
}

impl Style {
    /// All styles
    #[must_use]
    pub fn all() -> IndexSet<Style> {
        IndexSet::from([
            Style::Octant,
            Style::OctantSep,
            Style::Sextant,
            Style::SextantSep,
            Style::Quadrant,
            Style::QuadrantSep,
            Style::Half,
            Style::Full,
            Style::FullColor,
            Style::FullShade,
        ])
    }

    /// Render an image
    pub fn render(
        &self,
        img: &RgbImage,
        invert: bool,
        threshold: u8,
        full_char: &str,
    ) -> Result<String> {
        match self {
            Style::Octant => style_octant(img, invert, threshold),
            Style::OctantSep => style_octant_sep(img, invert, threshold),
            Style::Sextant => style_sextant(img, invert, threshold),
            Style::SextantSep => style_sextant_sep(img, invert, threshold),
            Style::Quadrant => style_quadrant(img, invert, threshold),
            Style::QuadrantSep => style_quadrant_sep(img, invert, threshold),
            Style::Half => style_half(img, invert, threshold),
            Style::Full => style_full(img, invert, threshold, full_char),
            Style::FullColor => style_full_color(img, invert, full_char),
            Style::FullShade => style_full_shade(img, invert),
            Style::All => Err(anyhow!("Style `{self:?}` does not render")),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Structs

/// Rescale factor
#[derive(Debug, PartialEq)]
pub struct Rescale {
    width: f32,
    height: f32,
}

impl Rescale {
    /// Parse from a `WIDTHxHEIGHT` string where `WIDTH` and `HEIGHT` are floats > 0
    pub fn parse(s: &str) -> Result<Rescale> {
        if let Some((width, height)) = s.split_once('x') {
            let width = width.parse::<f32>()?;
            let height = height.parse::<f32>()?;
            match (width < 0.0, height < 0.0) {
                (true, true) => Err(anyhow!("Rescale width and height factors are negative")),
                (true, false) => Err(anyhow!("Rescale width factor is negative")),
                (false, true) => Err(anyhow!("Rescale height factor is negative")),
                (false, false) => Ok(Rescale { width, height }),
            }
        } else if let Ok(f) = s.parse::<f32>() {
            if f > 0.0 {
                Ok(Rescale {
                    width: f,
                    height: f,
                })
            } else {
                Err(anyhow!("Rescale factor is negative"))
            }
        } else {
            Err(anyhow!("Failed to parse rescale scale factor from `{s}`"))
        }
    }

    /// Rescale an image
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_sign_loss)]
    #[must_use]
    pub fn image(&self, img: RgbImage) -> RgbImage {
        if *self == DEFAULT_RESCALE {
            img
        } else {
            let width = ((img.width() as f32) * self.width) as u32;
            let height = ((img.height() as f32) * self.height) as u32;
            let dyn_img: DynamicImage = img.into();
            image::imageops::resize(&dyn_img, width, height, FilterType::Lanczos3).convert()
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Padding
pub struct Padding {
    top: u32,
    right: u32,
    bottom: u32,
    left: u32,
}

impl Padding {
    /// Create a padding with the same number of pixels
    #[must_use]
    pub fn all(pixels: u32) -> Padding {
        Padding {
            top: pixels,
            right: pixels,
            bottom: pixels,
            left: pixels,
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Functions

/// Render text to a font image
#[allow(clippy::too_many_arguments)]
pub fn text_to_image(
    input: &str,
    font: &FontArc,
    size: f32,
    rescale: &Rescale,
    padding: &Padding,
) -> Result<RgbImage> {
    let mut lines = vec![];

    for line in input.lines() {
        let line = if line.is_empty() { " " } else { line };

        let px_scale = PxScale::from(size);
        let (text_width, text_height) = text_size(px_scale, font, line);

        let img_width = padding.left + text_width + padding.right + 1;
        // INFO: Add 1 extra pixel to ensure an inverted image with high threshold has at least one
        // background pixel to the right.
        let img_height = padding.top + text_height + padding.bottom;

        let mut img = new_image(img_width, img_height);

        draw_text_mut(
            &mut img,
            BLACK,
            padding.left.cast_signed(),
            padding.top.cast_signed(),
            px_scale,
            font,
            line,
        );

        lines.push(img);
    }

    // Concat line images vertically
    let img = if lines.len() == 1 {
        lines.remove(0)
    } else {
        concat_vert(&lines)?
    };

    Ok(rescale.image(img))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

fn new_image(width: u32, height: u32) -> RgbImage {
    ImageBuffer::from_pixel(width, height, WHITE)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

fn ensure_rows_is_multiple_of(m: u32, img: &RgbImage) -> RgbImage {
    let height = img.height();
    let add_rows = m - height % m;
    if add_rows == 0 {
        img.to_owned()
    } else {
        let mut new_img = new_image(img.width(), height + add_rows);
        image::imageops::replace(&mut new_img, img, 0, 0);
        new_img
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// If pixel byte is less than `threshold` return `0`, otherwise `value`
fn bit(value: usize, threshold: u8, pixel: u8) -> usize {
    if pixel < threshold { 0 } else { value }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// If pixel byte is less than `threshold` return `value`, otherwise `0`
fn bit2(value: u8, threshold: u8, pixel: u8) -> u8 {
    if pixel < threshold { value } else { 0 }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Concatenate two or more images vertically
pub fn concat_vert(images: &[RgbImage]) -> Result<RgbImage> {
    match images.len() {
        0 => Err(anyhow!("No images provided")),
        1 => Err(anyhow!("One image provided")),
        _ => {
            let width = images
                .iter()
                .map(image::ImageBuffer::width)
                .max()
                .unwrap_or(0);

            if width == 0 {
                return Err(anyhow!("Width of images is zero"));
            }

            let total_height: u32 = images.iter().map(image::ImageBuffer::height).sum();

            let mut output = new_image(width, total_height);

            let mut y = 0;
            for img in images {
                output.copy_from(img, 0, y)?;
                y += img.height();
            }

            Ok(output)
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Iterate style/threshold/inverts
pub fn render<'a>(
    styles: &'a IndexSet<Style>,
    thresholds: &'a [u8],
    inverts: &'a [bool],
    full_char: &'a str,
    img: &'a RgbImage,
) -> Result<Vec<(&'a Style, u8, bool, String)>> {
    let mut r = vec![];
    for style in styles {
        for threshold in thresholds {
            for invert in inverts {
                r.push((
                    style,
                    *threshold,
                    *invert,
                    style.render(img, *invert, *threshold, full_char)?,
                ));
            }
        }
    }

    Ok(r)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Ensure that a directory exists
pub fn mkdir_p(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)
        .map_err(|e| anyhow!("Failed to create directory `{}`: {e}", dir.display()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

fn pixel_rgb(pixel: Rgb<u8>, invert: bool) -> (u8, u8, u8) {
    if invert {
        (255 - pixel[0], 255 - pixel[1], 255 - pixel[2])
    } else {
        (pixel[0], pixel[1], pixel[2])
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Convert an image to `octant-sep` style
fn style_octant_sep(img: &RgbImage, invert: bool, threshold: u8) -> Result<String> {
    let mut s = String::new();

    let img = ensure_rows_is_multiple_of(4, img);
    let rows = img.rows().collect::<Vec<_>>();
    for four_rows in rows.as_slice().chunks(4) {
        if let [row_1, row_2, row_3, row_4] = four_rows {
            let row_1 = row_1.clone().collect::<Vec<_>>();
            let row_2 = row_2.clone().collect::<Vec<_>>();
            let row_3 = row_3.clone().collect::<Vec<_>>();
            let row_4 = row_4.clone().collect::<Vec<_>>();

            for a in (0..row_1.len()).filter(|x| x % 2 == 0) {
                let b = a + 1;
                if b < row_1.len() {
                    /*
                     1  8
                     2 10
                     4 20
                    40 80
                    */
                    let b_01 = bit2(0x01, threshold, row_1[a][0]);
                    let b_08 = bit2(0x08, threshold, row_1[b][0]);
                    let b_02 = bit2(0x02, threshold, row_2[a][0]);
                    let b_10 = bit2(0x10, threshold, row_2[b][0]);
                    let b_04 = bit2(0x04, threshold, row_3[a][0]);
                    let b_20 = bit2(0x20, threshold, row_3[b][0]);
                    let b_40 = bit2(0x40, threshold, row_4[a][0]);
                    let b_80 = bit2(0x80, threshold, row_4[b][0]);

                    let mut j = b_01 + b_02 + b_04 + b_08 + b_10 + b_20 + b_40 + b_80;

                    if invert {
                        j = !j;
                    }

                    write!(&mut s, "{}", braille(j))?;
                }
            }
            writeln!(&mut s)?;
        }
    }

    Ok(s)
}

fn braille(bits: u8) -> char {
    char::from_u32(0x2800 | u32::from(bits)).expect("char")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Convert an image to `octant` style
fn style_octant(img: &RgbImage, invert: bool, threshold: u8) -> Result<String> {
    let mut s = String::new();

    let img = ensure_rows_is_multiple_of(4, img);
    let rows = img.rows().collect::<Vec<_>>();
    for four_rows in rows.as_slice().chunks(4) {
        let mut four_rows = four_rows.to_vec();
        while four_rows.len() < 4 {
            four_rows.push(rows[0].clone());
        }
        if let [row_1, row_2, row_3, row_4] = &four_rows[..] {
            let row_1 = row_1.clone().collect::<Vec<_>>();
            let row_2 = row_2.clone().collect::<Vec<_>>();
            let row_3 = row_3.clone().collect::<Vec<_>>();
            let row_4 = row_4.clone().collect::<Vec<_>>();

            for a in (0..row_1.len()).filter(|x| x % 2 == 0) {
                let b = a + 1;
                if b < row_1.len() {
                    /*
                     1  2
                     4  8
                    10 20
                    40 80
                    */
                    let b_01 = bit2(0x01, threshold, row_1[a][0]);
                    let b_02 = bit2(0x02, threshold, row_1[b][0]);
                    let b_04 = bit2(0x04, threshold, row_2[a][0]);
                    let b_08 = bit2(0x08, threshold, row_2[b][0]);
                    let b_10 = bit2(0x10, threshold, row_3[a][0]);
                    let b_20 = bit2(0x20, threshold, row_3[b][0]);
                    let b_40 = bit2(0x40, threshold, row_4[a][0]);
                    let b_80 = bit2(0x80, threshold, row_4[b][0]);

                    let mut j = b_01 + b_02 + b_04 + b_08 + b_10 + b_20 + b_40 + b_80;

                    if invert {
                        j = !j;
                    }

                    write!(&mut s, "{}", octant(j))?;
                }
            }
            writeln!(&mut s)?;
        }
    }

    Ok(s)
}

const OCTANT_MAP: [(u8, u8); 26] = [
    (0x00, 0x00), // U+00A0 NO-BREAK SPACE
    (0x01, 0xA8), // U+1CEA8
    (0x02, 0xAB), // U+1CEAB
    (0x03, 0xC2), // U+1FB82
    (0x05, 0x98), // U+2598
    (0x0A, 0x9D), // U+259D
    (0x0F, 0x80), // U+2580
    (0x14, 0xE6), // U+1FBE6
    (0x28, 0xE7), // U+1FBE7
    (0x3F, 0xC5), // U+1FB85
    (0x40, 0xA3), // U+1CEA3
    (0x50, 0x96), // U+2596
    (0x55, 0x8C), // U+258C
    (0x5A, 0x9E), // U+259E
    (0x5F, 0x9B), // U+259B
    (0x80, 0xA0), // U+1CEA0
    (0xA0, 0x97), // U+2597
    (0xA5, 0x9A), // U+259A
    (0xAA, 0x90), // U+2590
    (0xAF, 0x9C), // U+259C
    (0xC0, 0x82), // U+2582
    (0xF0, 0x84), // U+2584
    (0xF5, 0x99), // U+2599
    (0xFA, 0x9F), // U+259F
    (0xFC, 0x86), // U+2586
    (0xFF, 0x88), // U+2588
];

fn octant(bits: u8) -> char {
    match OCTANT_MAP.binary_search_by_key(&bits, |&(key, _)| key) {
        Ok(0) => '\u{00A0}', // blank => nbsp
        Ok(index) => {
            let data = OCTANT_MAP[index].1;
            let base = match (data >> 5) & 3 {
                0 => 0x2580,
                1 => 0x1CEA0,
                2 => 0x1FB80,
                _ => 0x1FBE6,
            };

            char::from_u32(base | (u32::from(data) & 0x1F)).unwrap()
        }
        Err(index) => {
            let codepoint = 0x1CD00 + u32::from(bits) - u32::try_from(index).expect("usize to u32");
            char::from_u32(codepoint).unwrap()
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Convert an image to `sextant-sep` style
fn style_sextant_sep(img: &RgbImage, invert: bool, threshold: u8) -> Result<String> {
    let mut s = String::new();

    let img = ensure_rows_is_multiple_of(3, img);
    let rows = img.rows().collect::<Vec<_>>();

    for three_rows in rows.chunks(3) {
        if let [row_1, row_2, row_3] = three_rows {
            let row_1 = row_1.clone().collect::<Vec<_>>();
            let row_2 = row_2.clone().collect::<Vec<_>>();
            let row_3 = row_3.clone().collect::<Vec<_>>();

            for a in (0..row_1.len()).step_by(2) {
                let b = a + 1;

                if b < row_1.len() {
                    /*
                     1  2
                     4  8
                    10 20
                    */
                    let mut bits = bit2(0x01, threshold, row_1[a][0])
                        | bit2(0x02, threshold, row_1[b][0])
                        | bit2(0x04, threshold, row_2[a][0])
                        | bit2(0x08, threshold, row_2[b][0])
                        | bit2(0x10, threshold, row_3[a][0])
                        | bit2(0x20, threshold, row_3[b][0]);

                    if invert {
                        bits ^= 0x3F;
                    }

                    write!(&mut s, "{}", sep_sextant(bits))?;
                }
            }

            writeln!(&mut s)?;
        }
    }

    Ok(s)
}

fn sep_sextant(bits: u8) -> char {
    match bits & 0x3F {
        0 => '\u{00A0}', // NO-BREAK SPACE
        bits => char::from_u32(0x1CE50 + u32::from(bits)).unwrap(),
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Convert an image to `sextant` style
fn style_sextant(img: &RgbImage, invert: bool, threshold: u8) -> Result<String> {
    let mut s = String::new();

    let img = ensure_rows_is_multiple_of(3, img);
    let rows = img.rows().collect::<Vec<_>>();

    for three_rows in rows.chunks(3) {
        if let [row_1, row_2, row_3] = three_rows {
            let row_1 = row_1.clone().collect::<Vec<_>>();
            let row_2 = row_2.clone().collect::<Vec<_>>();
            let row_3 = row_3.clone().collect::<Vec<_>>();

            for a in (0..row_1.len()).step_by(2) {
                let b = a + 1;

                if b < row_1.len() {
                    /*
                     1  2
                     4  8
                    10 20
                    */
                    let b_01 = bit2(0x01, threshold, row_1[a][0]);
                    let b_02 = bit2(0x02, threshold, row_1[b][0]);
                    let b_04 = bit2(0x04, threshold, row_2[a][0]);
                    let b_08 = bit2(0x08, threshold, row_2[b][0]);
                    let b_10 = bit2(0x10, threshold, row_3[a][0]);
                    let b_20 = bit2(0x20, threshold, row_3[b][0]);

                    let mut bits = b_01 | b_02 | b_04 | b_08 | b_10 | b_20;

                    if invert {
                        bits ^= 0x3F;
                    }

                    write!(&mut s, "{}", sextant(bits))?;
                }
            }

            writeln!(&mut s)?;
        }
    }

    Ok(s)
}

fn sextant(bits: u8) -> char {
    match bits & 0x3F {
        0x00 => '\u{00A0}', // NO-BREAK SPACE
        0x15 => '▌',        // LEFT HALF BLOCK
        0x2A => '▐',        // RIGHT HALF BLOCK
        0x3F => '█',        // FULL BLOCK
        bits => {
            // U+1FB00..U+1FB3B omit the two half-block patterns.
            let codepoint =
                0x1FB00 + u32::from(bits) - 1 - u32::from(bits > 0x15) - u32::from(bits > 0x2A);

            char::from_u32(codepoint).unwrap()
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Convert an image to `quadrant-sep` style
fn style_quadrant_sep(img: &RgbImage, invert: bool, threshold: u8) -> Result<String> {
    let mut s = String::new();

    let mut chars = vec![
        '\u{1CC2F}', // 𜰯 SEPARATED BLOCK QUADRANT-1234
        '\u{1CC27}', // 𜰧 SEPARATED BLOCK QUADRANT-123
        '\u{1CC2B}', // 𜰫 SEPARATED BLOCK QUADRANT-124
        '\u{1CC23}', // 𜰣 SEPARATED BLOCK QUADRANT-12
        '\u{1CC2D}', // 𜰭 SEPARATED BLOCK QUADRANT-134
        '\u{1CC25}', // 𜰥 SEPARATED BLOCK QUADRANT-13
        '\u{1CC29}', // 𜰩 SEPARATED BLOCK QUADRANT-14
        '\u{1CC21}', // 𜰡 SEPARATED BLOCK QUADRANT-1
        '\u{1CC2E}', // 𜰮 SEPARATED BLOCK QUADRANT-234
        '\u{1CC26}', // 𜰦 SEPARATED BLOCK QUADRANT-23
        '\u{1CC2A}', // 𜰪 SEPARATED BLOCK QUADRANT-24
        '\u{1CC22}', // 𜰢 SEPARATED BLOCK QUADRANT-2
        '\u{1CC2C}', // 𜰬 SEPARATED BLOCK QUADRANT-34
        '\u{1CC24}', // 𜰤 SEPARATED BLOCK QUADRANT-3
        '\u{1CC28}', // 𜰨 SEPARATED BLOCK QUADRANT-4
        '\u{00A0}',  //   NBSP
    ];

    if invert {
        chars.reverse();
    }

    let img = ensure_rows_is_multiple_of(2, img);
    let rows = img.rows().collect::<Vec<_>>();
    for two_rows in rows.as_slice().chunks(2) {
        if let [row_1, row_2] = two_rows {
            let row_1 = row_1.clone().collect::<Vec<_>>();
            let row_2 = row_2.clone().collect::<Vec<_>>();
            for i in (0..row_1.len()).filter(|x| x % 2 == 0) {
                if i < row_1.len() - 1 {
                    let tl = bit(8, threshold, row_1[i][0]);
                    let tr = bit(4, threshold, row_1[i + 1][0]);
                    let bl = bit(2, threshold, row_2[i][0]);
                    let br = bit(1, threshold, row_2[i + 1][0]);
                    let j = tl | tr | bl | br;
                    write!(&mut s, "{}", chars[j])?;
                }
            }
            writeln!(&mut s)?;
        }
    }

    Ok(s)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Convert an image to `quadrant` style
fn style_quadrant(img: &RgbImage, invert: bool, threshold: u8) -> Result<String> {
    let mut s = String::new();

    let mut chars = vec![
        '\u{2588}', // 0000 █ Full block
        '\u{259B}', // 0001 ▛ Quadrant upper left and upper right and lower left
        '\u{259C}', // 0010 ▜ Quadrant upper left and upper right and lower right
        '\u{2580}', // 0011 ▀ Upper half block
        '\u{2599}', // 0100 ▙ Quadrant upper left and lower left and lower right
        '\u{258C}', // 0101 ▌ Left half block
        '\u{259A}', // 0110 ▚ Quadrant upper left and lower right
        '\u{2598}', // 0111 ▘ Quadrant upper left
        '\u{259F}', // 1000 ▟ Quadrant upper right and lower left and lower right
        '\u{259E}', // 1001 ▞ Quadrant upper right and lower left
        '\u{2590}', // 1010 ▐ Right half block
        '\u{259D}', // 1011 ▝ Quadrant upper right
        '\u{2584}', // 1100 ▄ Lower half block
        '\u{2596}', // 1101 ▖ Quadrant lower left
        '\u{2597}', // 1110 ▗ Quadrant lower right
        '\u{00A0}', // 1111   NBSP
    ];

    if invert {
        chars.reverse();
    }

    let img = ensure_rows_is_multiple_of(2, img);
    let rows = img.rows().collect::<Vec<_>>();
    for two_rows in rows.as_slice().chunks(2) {
        if let [row_1, row_2] = two_rows {
            let row_1 = row_1.clone().collect::<Vec<_>>();
            let row_2 = row_2.clone().collect::<Vec<_>>();
            for i in (0..row_1.len()).filter(|x| x % 2 == 0) {
                if i < row_1.len() - 1 {
                    let tl = bit(8, threshold, row_1[i][0]);
                    let tr = bit(4, threshold, row_1[i + 1][0]);
                    let bl = bit(2, threshold, row_2[i][0]);
                    let br = bit(1, threshold, row_2[i + 1][0]);
                    let j = tl | tr | bl | br;
                    write!(&mut s, "{}", chars[j])?;
                }
            }
            writeln!(&mut s)?;
        }
    }

    Ok(s)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Convert an image to `half` style
fn style_half(img: &RgbImage, invert: bool, threshold: u8) -> Result<String> {
    let mut s = String::new();

    let mut chars = [
        '\u{2588}', // 00 █ Full block
        '\u{2580}', // 01 ▀ Upper half block
        '\u{2584}', // 10 ▄ Lower half block
        '\u{00A0}', // 11   NBSP
    ];

    if invert {
        chars.reverse();
    }

    let img = ensure_rows_is_multiple_of(2, img);
    let rows = img.rows().collect::<Vec<_>>();
    for two_rows in rows.as_slice().chunks(2) {
        if let [row_1, row_2] = two_rows {
            let row_1 = row_1.clone().collect::<Vec<_>>();
            let row_2 = row_2.clone().collect::<Vec<_>>();
            for i in 0..row_1.len() {
                let t = bit(2, threshold, row_1[i][0]);
                let b = bit(1, threshold, row_2[i][0]);
                let j = t + b;
                write!(&mut s, "{}", chars[j])?;
            }
            writeln!(&mut s)?;
        }
    }

    Ok(s)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Convert an image to `full` style
fn style_full(img: &RgbImage, invert: bool, threshold: u8, full_char: &str) -> Result<String> {
    let mut s = String::new();

    let mut chars = [
        full_char,  // █ Full block
        "\u{00A0}", //   NBSP
    ];

    if invert {
        chars.reverse();
    }

    for row in img.rows() {
        for pixel in row {
            if pixel.0[0] < threshold {
                write!(&mut s, "{}", chars[0])?;
            } else {
                write!(&mut s, "{}", chars[1])?;
            }
        }
        writeln!(&mut s)?;
    }

    Ok(s)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Convert an image to `full-shade` style
fn style_full_shade(img: &RgbImage, invert: bool) -> Result<String> {
    let mut s = String::new();

    let mut chars = [
        '\u{2588}', // █ Full block
        '\u{2593}', // ▓ Dark shade
        '\u{2592}', // ▒ Medium shade
        '\u{2591}', // ░ Light shade
        '\u{00A0}', //   NBSP
    ];

    if invert {
        chars.reverse();
    }

    let n = u8::try_from(chars.len()).expect("usize to u8");
    let threshold = 255 / n;
    let thresholds = (0..n).map(|x| x * threshold).collect::<Vec<_>>();
    for row in img.rows() {
        for pixel in row {
            for (i, threshold) in thresholds.iter().enumerate().rev() {
                if pixel.0[0] >= *threshold {
                    write!(&mut s, "{}", chars[i])?;
                    break;
                }
            }
        }
        writeln!(&mut s)?;
    }

    Ok(s)
}
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Convert an image to `full-color` style
fn style_full_color(img: &RgbImage, invert: bool, full_char: &str) -> Result<String> {
    let mut s = String::new();

    for row in img.rows() {
        for pixel in row {
            let (r, g, b) = pixel_rgb(*pixel, invert);
            write!(&mut s, "{}", full_char.truecolor(r, g, b))?;
        }
        writeln!(&mut s)?;
    }

    Ok(s)
}
