use std::{io::Read, path::PathBuf};

use clap::Parser;
use image::{GrayImage, ImageResult, RgbImage, Rgba};

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

    let mut file = match std::fs::File::open(&opts.path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Could not open file. ({})", err);
            return;
        }
    };

    println!("RAW file: {}", opts.path.display());

    // Check if there is a corresponding .bmp file alongside this one.
    let bmp_path = opts.path.with_extension("bmp");
    let bmp_image = if bmp_path.exists() {
        println!("BMP file: {}", bmp_path.display());

        let bmp_image = match read_bmp_file(&bmp_path) {
            Ok(image) => image,
            Err(err) => {
                eprintln!("Could not load .bmp file. ({})", err);
                return;
            }
        };

        Some(bmp_image)
    } else {
        None
    };

    let png_path = opts.path.with_extension("png");

    if let Some(bmp) = bmp_image {
        let width = bmp.width();
        let height = bmp.height();
        let raw_image = match shadow_company_tools::raw::load_raw_file(&mut file, width, height) {
            Ok(raw) => raw,
            Err(err) => {
                eprintln!("Could not read .raw file. ({})", err);
                return;
            }
        };

        match bmp_raw_to_png(&png_path, &bmp, &raw_image) {
            Ok(_) => println!("Finished!"),
            Err(err) => {
                eprintln!("Could not create .png file. ({})", err);
                return;
            }
        }
    } else {
        let width = match opts.width {
            Some(width) => width,
            None => {
                eprintln!(
                    "No .bmp file found for .raw file, pass width argument on the command line."
                );
                return;
            }
        };

        let height = match opts.height {
            Some(height) => height,
            None => {
                eprintln!(
                    "No .bmp file found for .raw file, pass height argument on the command line."
                );
                return;
            }
        };

        let raw_image = match shadow_company_tools::raw::load_raw_file(&mut file, width, height) {
            Ok(raw) => raw,
            Err(err) => {
                eprintln!("Could not read .raw file. ({})", err);
                return;
            }
        };

        match raw_to_png(&png_path, &raw_image) {
            Ok(_) => println!("Finished!"),
            Err(err) => {
                eprintln!("Could not create .png file. ({})", err);
                return;
            }
        }
    }
}

fn read_bmp_file(bmp_path: &PathBuf) -> ImageResult<RgbImage> {
    let mut file = std::fs::File::open(bmp_path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let p = image::load_from_memory_with_format(&mut buf, image::ImageFormat::Bmp)?;
    Ok(p.into_rgb8())
}

fn bmp_raw_to_png(png_path: &PathBuf, bmp: &RgbImage, raw: &GrayImage) -> image::ImageResult<()> {
    assert!(bmp.dimensions() == raw.dimensions());

    let mut result = image::RgbaImage::new(bmp.width(), bmp.height());

    for ((_, _, r), ((_, _, bmp), (_, _, raw))) in result
        .enumerate_pixels_mut()
        .zip(bmp.enumerate_pixels().zip(raw.enumerate_pixels()))
    {
        *r = Rgba([bmp.0[0], bmp.0[1], bmp.0[2], raw.0[0]]);
    }

    let mut output = std::fs::File::create(png_path)?;
    result.write_to(&mut output, image::ImageFormat::Png)
}

fn raw_to_png(png_path: &PathBuf, raw: &GrayImage) -> image::ImageResult<()> {
    let mut output = std::fs::File::create(png_path)?;
    raw.write_to(&mut output, image::ImageFormat::Png)
}
