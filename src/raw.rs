//! Loads .raw image format.
//! .raw files are used to store alpha masks for other images or greyscale
//! images like fonts.

/// Load a .raw file from the reader and returns it as a single channel grayscale image.
pub fn load_raw_file<R>(
    reader: &mut R,
    width: u32,
    height: u32,
) -> image::ImageResult<image::GrayImage>
where
    R: std::io::Read,
{
    assert!(width > 0, "Width can not be 0");
    assert!(height > 0, "Height can not be 0");

    let mut buf = vec![0_u8; width as usize * height as usize];
    reader.read_exact(&mut buf)?;

    Ok(image::GrayImage::from_vec(width, height, buf)
        .expect("Not enought bytes from reader. Are the width or height invalid?"))
}
