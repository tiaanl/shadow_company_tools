//! Handles .raw image format.
//! Raw files are used to store alpha masks for other images or greyscale
//! images like fonts.

use byteorder::ReadBytesExt;

fn reduce_precision(value: u32) -> u32 {
    (((value >> 4 & 0xf000000 | value & 0xf00000) >> 4 | value & 0xf000) >> 4 | value & 0xf0) >> 4
}

pub fn load_raw_file<R>(
    r: &mut R,
    width: u32,
    height: u32,
) -> image::ImageResult<image::GrayAlphaImage>
where
    R: std::io::Read,
{
    let mut image = image::GrayAlphaImage::new(width, height);

    for (_, _, pixel) in image.enumerate_pixels_mut() {
        let byte = r.read_u8()?;
        // TODO: This conversion isn't 100% correct, but good enough for now to give the idea.
        // TODO: This can be simplified a lot.
        let encoded = reduce_precision(((byte as u32) << 24) | 0xFFFFFF) as u16;
        *pixel = image::LumaA(encoded.to_le_bytes());
    }

    Ok(image)
}
