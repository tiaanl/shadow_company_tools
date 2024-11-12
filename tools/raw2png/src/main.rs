use clap::Parser;
use image::{buffer::ConvertBuffer, ImageResult, RgbaImage};

#[derive(Parser)]
struct Opts {
    /// Path to the .raw file to convert.
    path: std::path::PathBuf,
    /// The width of the .raw image. If the width is not specified, try to detect it.
    width: Option<u32>,
    /// The height of the .raw image. If the width is not specified, try to detect it.
    height: Option<u32>,
}

fn main() {
    let opts = Opts::parse();

    if !opts.path.exists() {
        eprintln!("File does not exist: {}", opts.path.display());
        return;
    }

    let png_image = match convert_to_png(&opts) {
        Ok(image) => image,
        Err(err) => {
            eprintln!("Could not generate .png image. {}", err);
            return;
        }
    };

    let png_path = opts.path.with_extension("png");
    let mut png_file = match std::fs::File::create(&png_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Could not write .png file. {}", err);
            return;
        }
    };

    match png_image.write_to(&mut png_file, image::ImageFormat::Png) {
        Ok(..) => println!("Generated {}", png_path.display()),
        Err(err) => eprintln!("Could not write .png file. {}", err),
    }
}

fn convert_to_png(opts: &Opts) -> ImageResult<RgbaImage> {
    let mut file = std::fs::File::open(&opts.path)?;
    println!("RAW file: {}", opts.path.display());

    // Check if there is a corresponding .bmp file alongside this one.
    let bmp_path = opts.path.with_extension("bmp");
    let bmp_image = if bmp_path.exists() {
        println!("BMP file: {}", bmp_path.display());
        let mut bmp_file = std::fs::File::open(&bmp_path)?;
        Some(shadow_company_tools::images::load_bmp_file(&mut bmp_file)?)
    } else {
        None
    };

    Ok(if let Some(bmp) = bmp_image {
        let (width, height) = bmp.dimensions();
        let raw = shadow_company_tools::images::load_raw_file(&mut file, width, height)?;

        let mut rgba: RgbaImage = bmp.convert();
        for (pixel, alpha) in rgba.pixels_mut().zip(raw.pixels()) {
            // Set the alpha component from the raw image.
            pixel.0[3] = alpha.0[0];
        }

        rgba
    } else {
        let width = match opts.width {
            Some(value) => value,
            None => todo!(),
        };
        let height = match opts.height {
            Some(value) => value,
            None => todo!(),
        };

        shadow_company_tools::images::load_raw_file(&mut file, width, height)?.convert()
    })
}
