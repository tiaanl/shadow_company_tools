//! Support for loading the different image formats supported by the engine.
//!
//! .raw files are used to store alpha masks for other images or greyscale
//! images like fonts.
//! .bmp files are for RGB images.
//! .pcx files are used for data, such as height maps, a-star map data, etc.

use image::{GrayImage, ImageDecoder, ImageResult, RgbImage, RgbaImage};

/// Load a .raw file from the reader and returns it as a single channel grayscale image.
pub fn load_raw_file<R>(reader: &mut R, width: u32, height: u32) -> ImageResult<GrayImage>
where
    R: std::io::Read,
{
    assert!(width > 0, "Width can not be 0");
    assert!(height > 0, "Height can not be 0");

    let mut buf = vec![0_u8; width as usize * height as usize];
    reader.read_exact(&mut buf)?;

    Ok(GrayImage::from_vec(width, height, buf)
        .expect("Not enought bytes from reader. Are the width or height invalid?"))
}

/// Load a .bmp file from the reader and returns it as a RGB image.
pub fn load_bmp_file<R>(reader: &mut R) -> ImageResult<RgbImage>
where
    R: std::io::Read + std::io::Seek,
{
    use image::codecs::bmp::BmpDecoder;

    let decoder = BmpDecoder::new(std::io::BufReader::new(reader))?;
    let mut buf = vec![0_u8; decoder.total_bytes() as usize];
    let (width, height) = decoder.dimensions();
    decoder.read_image(&mut buf)?;

    Ok(RgbImage::from_vec(width, height, buf).unwrap())
}

/// Combine an RGB image (from load_bmp_file) with a grayscale image (from
/// load_raw_file) to create an RGBA image, using the grayscale image as the alpha channel.
pub fn combine_bmp_and_raw(bmp: &RgbImage, raw: &GrayImage) -> RgbaImage {
    use image::buffer::ConvertBuffer;

    let mut rgba: RgbaImage = bmp.convert();
    for (pixel, alpha) in rgba.pixels_mut().zip(raw.pixels()) {
        // Set the alpha component from the raw image.
        pixel.0[3] = alpha.0[0];
    }

    rgba
}
