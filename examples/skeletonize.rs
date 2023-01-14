use skeletonize::edge_detection::{sobel, sobel4};
use skeletonize::{foreground, thin_image_edges, MarkingMethod};
use structopt::StructOpt;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let mut opt = Opt::from_args();

    // Open image and initialize output filename
    let img = image::DynamicImage::ImageLuma8(image::open(&opt.input)?.to_luma8());
    let output = if let Some(output) = opt.output {
        output
    } else {
        generate_filename(&opt.input)?.into()
    };

    // Process the option strings for matching
    opt.edge.make_ascii_lowercase();
    opt.foreground.make_ascii_lowercase();
    opt.method.make_ascii_lowercase();

    let edge = match opt.edge.as_str() {
        "sobel" | "s" => EdgeDetection::Sobel,
        "sobel4" | "s4" => EdgeDetection::Sobel4,
        "" => EdgeDetection::None,
        _ => return Err("Edge detection must be `sobel`/`s` or `sobel4`/`s4`".into()),
    };
    let foreground = match opt.foreground.as_str() {
        "black" | "b" => Fg::Black,
        "white" | "w" => Fg::White,
        _ => return Err("Foreground color must be `black`/`b` or `white`/`w`".into()),
    };
    let method = match opt.method.as_str() {
        "modified" | "m" => MarkingMethod::Modified,
        "standard" | "s" => MarkingMethod::Standard,
        _ => return Err("Method must be `standard`/`s` or `modified`/`m`".into()),
    };

    // Perform edge detection if one of the Sobel options is passed
    let mut filtered = match edge {
        EdgeDetection::Sobel => match foreground {
            Fg::Black => sobel::<foreground::Black>(&img, opt.threshold)?,
            Fg::White => sobel::<foreground::White>(&img, opt.threshold)?,
        },
        EdgeDetection::Sobel4 => match foreground {
            Fg::Black => sobel4::<foreground::Black>(&img, opt.threshold)?,
            Fg::White => sobel4::<foreground::White>(&img, opt.threshold)?,
        },
        EdgeDetection::None => {
            let mut filtered = img;
            if let Some(t) = opt.threshold {
                skeletonize::threshold(&mut filtered, t)?;
            }
            filtered
        }
    };

    // Skip thinning if `no-thin` flag is passed
    if !opt.no_thin {
        match foreground {
            Fg::Black => {
                thin_image_edges::<foreground::Black>(&mut filtered, method, None)?;
            }
            Fg::White => {
                thin_image_edges::<foreground::White>(&mut filtered, method, None)?;
            }
        }
    }

    Ok(filtered.save(output)?)
}

enum EdgeDetection {
    Sobel,
    Sobel4,
    None,
}

enum Fg {
    Black,
    White,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "skeletonize", about = "Image edge thinning utility")]
pub struct Opt {
    /// Input file.
    #[structopt(short, long, parse(from_os_str))]
    pub input: std::path::PathBuf,

    /// Output filename, defaults to `png` output.
    #[structopt(short, long, parse(from_os_str))]
    pub output: Option<std::path::PathBuf>,

    /// Color of the image's foreground, `black`/`b` or `white`/`w`.
    #[structopt(short, long, default_value = "black")]
    pub foreground: String,

    /// Edge thinning algorithm to use, `standard`/`s` or `modified`/`m`.
    #[structopt(short, long, default_value = "modified")]
    pub method: String,

    /// Brightness value below which pixels will become black, ranges from 0.0
    /// to 1.0.
    ///
    /// 0.15 to 0.45 is a reasonable range to start with, some images may
    /// require higher values such as 0.8.
    #[structopt(short, long)]
    pub threshold: Option<f32>,

    /// Run a Sobel edge detection filter on the image before image thinning.
    /// `sobel`/`s` or `sobel4`/`s4` are available options.
    #[structopt(short, long, default_value = "")]
    pub edge: String,

    /// Disables the edge thinning pass, used for generating images with only
    /// thresholding or edge detection performed.
    #[structopt(long)]
    pub no_thin: bool,
}

/// Appends a timestamp to an input filename to be used as the output filename.
fn generate_filename(path: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    let filename = path
        .file_stem()
        .ok_or("No file stem")?
        .to_str()
        .ok_or("Could not convert filename to string")?
        .to_string();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
    let secs = now.as_secs().to_string();
    let millis = format!("{:03}", now.subsec_millis());

    Ok(filename + "-" + &secs + &millis + ".png")
}
