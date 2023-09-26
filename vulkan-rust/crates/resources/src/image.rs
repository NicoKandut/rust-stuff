use std::{fs::File, io};

use png::OutputInfo;

pub fn write_image(
    path: &str,
    pixels: &[u8],
    width: usize,
    height: usize,
) -> Result<(), io::Error> {
    let image = File::create(path)?;

    let mut encoder = png::Encoder::new(image, width as u32, height as u32);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;

    Ok(())
}

pub fn read_image(path: &str) -> Result<(OutputInfo, Vec<u8>), io::Error> {
    let image = File::open(path)?;
    let decoder = png::Decoder::new(image);
    let (info, mut reader) = decoder.read_info()?;

    let mut pixels = vec![0; info.buffer_size() * info.bit_depth as usize / 8];
    reader.next_frame(&mut pixels)?;

    assert_eq!(info.color_type, png::ColorType::RGBA);
    assert_eq!(info.bit_depth, png::BitDepth::Eight);

    Ok((info, pixels))
}
