//! ttfun - CLI

//--------------------------------------------------------------------------------------------------
// Crates

use ttfun::*;

use {
    ab_glyph::FontArc,
    anyhow::{Result, anyhow},
    clap::{ArgAction, CommandFactory, Parser, error::ErrorKind},
    clap_cargo::style::CLAP_STYLING,
    image::RgbImage,
    indexmap::{IndexMap, IndexSet},
    std::path::{Path, PathBuf},
};

#[cfg(unix)]
use pager2::Pager;

//--------------------------------------------------------------------------------------------------
// Constants

const DEMO: &str = include_str!("../demo.txt");
const README: &str = include_str!("../README.md");

//--------------------------------------------------------------------------------------------------
// Structs

#[derive(Parser)]
#[command(about, version, max_term_width = 80, styles = CLAP_STYLING)]
struct Args {
    /// Input file(s)
    #[arg(value_name = "PATH")]
    input_files: Vec<PathBuf>,

    /// Input string(s)
    #[arg(short = 'i', value_name = "STRING")]
    input_strings: Vec<String>,

    /// Input image(s)
    #[arg(short = 'I', value_name = "PATH")]
    input_images: Vec<PathBuf>,

    /// Save image(s) to PNG files in a directory
    #[arg(short = 'o', value_name = "DIRPATH")]
    output_dir: Option<PathBuf>,

    /// Font file(s) (ttf); Built-in: `iosevka`, `eb-garamond`
    #[arg(short = 'f', value_name = "PATH", default_value = "iosevka")]
    font_files: Vec<PathBuf>,

    /// Font size(s)
    #[arg(short = 's', value_name = "FLOAT", default_value = "18")]
    sizes: Vec<f32>,

    /// Style(s) [default: octant]
    #[arg(short = 'S', value_name = "NAME")]
    styles: Vec<Style>,

    /// String used to represent each pixel for the `full` and `full-color` styles
    #[arg(short = 'F', value_name = "STRING", default_value = "\u{2588}")]
    full_char: String,

    /// Rescale image by factor `WIDTHxHEIGHT` where `WIDTH` and `HEIGHT` are floats > 0
    #[arg(
        short = 'r',
        value_name = "STR",
        default_value = "1x1",
        allow_negative_numbers = true
    )]
    rescale: String,

    /// Threshold(s) used for non `*-color` styles (0-255)
    #[arg(short = 't', value_name = "U8", default_value = "180")]
    thresholds: Vec<u8>,

    /// Invert foreground/background "colors"; `-nn`: regular & inverted, `-nnn`: inverted & regular
    #[arg(short = 'n', action = ArgAction::Count)]
    invert: u8,

    /// Use the demo file as the default input
    #[arg(short = 'd')]
    demo: bool,

    /// Enable verbose Markdown output
    #[arg(short = 'v')]
    verbose: bool,

    /// Print readme
    #[arg(short = 'R')]
    readme: bool,
}

//--------------------------------------------------------------------------------------------------
// Functions

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    // Process arguments
    let args = Args::parse();
    if args.readme {
        #[cfg(unix)]
        Pager::with_pager("bat -pl md").setup();

        print!("{README}");
        return Ok(());
    }
    let mut styles = args.styles.iter().copied().collect::<IndexSet<_>>();
    if let Some(dir) = &args.output_dir {
        mkdir_p(dir)?;
    } else if styles.is_empty() {
        styles.insert(Style::Octant);
    }
    if styles.contains(&Style::All) {
        styles = Style::all();
    }
    let v = args.verbose;
    let rescale = Rescale::parse(&args.rescale)?;
    let inverts: &[bool] = match args.invert {
        0 => &[false],
        1 => &[true],
        2 => &[false, true],
        3 => &[true, false],
        _ => Args::command()
            .error(ErrorKind::TooManyValues, "-nnnn+ is not supported")
            .exit(),
    };

    // Load fonts
    let mut fonts = IndexMap::new();
    let eb_garamond_path = Path::new("eb-garamond");
    let iosevka_path = Path::new("iosevka");
    for path in &args.font_files {
        if !fonts.contains_key(path) {
            fonts.insert(
                path,
                FontArc::try_from_vec(if path == eb_garamond_path {
                    EB_GARAMOND.to_vec()
                } else if path == iosevka_path {
                    IOSEVKA.to_vec()
                } else {
                    std::fs::read(path)?
                })?,
            );
        }
    }

    // Read inputs
    let n_inputs = (args.input_strings.len() + args.input_files.len()).max(1);
    let mut inputs: IndexSet<String> = IndexSet::with_capacity(n_inputs);
    for s in &args.input_strings {
        inputs.insert(s.trim_matches('\n').into());
    }
    for path in &args.input_files {
        inputs.insert(std::fs::read_to_string(path)?.trim_matches('\n').into());
    }
    if inputs.is_empty() && args.input_images.is_empty() {
        if args.demo {
            inputs.insert(DEMO.into());
        } else {
            inputs.insert(env!("CARGO_PKG_NAME").into());
        }
    }

    // Ensure no input is empty
    for (i, input) in inputs.iter().enumerate() {
        if input.is_empty() {
            return Err(anyhow!("Input {} is empty", i + 1));
        }
    }

    // No "extra" padding (for now)
    let padding = Padding::all(0);

    // Iterate inputs
    if !inputs.is_empty() {
        let w = inputs.len().ilog10() as usize + 1;
        for (i, input) in inputs.iter().enumerate() {
            vprintln!(v, "# Input {}\n\n```text\n{}\n```\n", i + 1, input);

            // Iterate font/sizes
            for (font_path, font) in &fonts {
                for size in &args.sizes {
                    vprintln!(v, "## Font `{}` ({size})\n", font_path.display());

                    // Render input to an image
                    let img = text_to_image(input, font, *size, &rescale, &padding)?;

                    if let Some(dir) = &args.output_dir {
                        // Save the image (if `-o`)
                        let save_path = dir.join(format!("{:0w$}.png", i + 1));
                        println!("- `{}`", save_path.display());
                        img.save(save_path)?;
                    } else {
                        // Render to text
                        render_all(&styles, &args.thresholds, inverts, &args.full_char, &img, v)?;
                    }
                }
            }
        }
    }

    // Iterate images
    if args.output_dir.is_none() {
        let o = inputs.len();
        for (i, path) in args.input_images.iter().enumerate() {
            vprintln!(v, "# Input {} `{}`\n\n## ...\n", i + o, path.display());
            let img = rescale.image(image::open(path)?.to_rgb8());
            render_all(&styles, &args.thresholds, inverts, &args.full_char, &img, v)?;
        }
    }

    Ok(())
}

fn render_all(
    styles: &IndexSet<Style>,
    thresholds: &[u8],
    inverts: &[bool],
    full_char: &str,
    img: &RgbImage,
    v: bool,
) -> Result<()> {
    for (style, threshold, invert, result) in render(styles, thresholds, inverts, full_char, img)? {
        if v {
            if *style == Style::FullColor {
                vprintln!(
                    v,
                    "\
                    ### Style `{style:?}` + \
                    Invert `{invert:?}`\n\
                    \n\
                    ```text\
                ",
                );
            } else {
                vprintln!(
                    v,
                    "\
                    ### Style `{style:?}` + \
                    Threshold `{threshold}` + \
                    Invert `{invert:?}`\n\
                    \n\
                    ```text\
                ",
                );
            }
        }

        print!("{result}");

        vprintln!(v, "```\n");
    }

    Ok(())
}
