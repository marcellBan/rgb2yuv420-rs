//! Simple RGB to YUV420 converter
//! (full swing)
//! (only supports 8 bit RGB color depth)

/// Converts an RGB image to YUV420p (planar/3 planes)
///
/// # Arguments
///
/// * `img` - should contain the pixel data in the following format:
/// `[r, g, b, ... , r, g, b, ... , r, g, b, ...]`
///
/// * `bytes_per_pixel` - should contain the number of bytes used by one pixel
/// (eg.: RGB is 3 bytes and RGBA is 4 bytes)
///
/// # Return
///
/// `[y, y, y, ... , u, u, u, ... , v, v, v, ...]`
///
/// # Examples
///
/// ```
/// let rgb = vec![0u8; 12];
/// let yuv = rgb2yuv420::convert_rgb_to_yuv420p(&rgb, 2, 2, 3);
/// assert_eq!(yuv.len(), rgb.len() / 2);
/// ```
pub fn convert_rgb_to_yuv420p(img: &[u8], width: u32, height: u32, bytes_per_pixel: usize) -> Vec<u8> {
    convert_rgb_to_yuv420(img, width, height, bytes_per_pixel, |yuv, uv_index, chroma_size, u, v| {
        yuv[*uv_index] = u;
        yuv[*uv_index + (f32::ceil(chroma_size as f32 / 2.0) as usize)] = v;
        *uv_index += 1;
    })
}

/// Converts an RGB image to YUV420sp NV12 (semi-planar/2 planes)
///
/// # Arguments
///
/// * `img` - should contain the pixel data in the following format:
/// [r, g, b, ... , r, g, b, ... , r, g, b, ...]
///
/// * `bytes_per_pixel` - should contain the number of bytes used by one pixel
/// (eg.: RGB is 3 bytes and RGBA is 4 bytes)
///
/// # Return
///
/// `[y, y, y, ... , u, v, u, v, ...]`
///
/// # Examples
///
/// ```
/// let rgb = vec![0u8; 12];
/// let yuv = rgb2yuv420::convert_rgb_to_yuv420sp_nv12(&rgb, 2, 2, 3);
/// assert_eq!(yuv.len(), rgb.len() / 2);
/// ```
pub fn convert_rgb_to_yuv420sp_nv12(img: &[u8], width: u32, height: u32, bytes_per_pixel: usize) -> Vec<u8> {
    convert_rgb_to_yuv420(img, width, height, bytes_per_pixel, |yuv, uv_index, _cs, u, v| {
        yuv[*uv_index] = u;
        *uv_index += 1;
        yuv[*uv_index] = v;
        *uv_index += 1;
    })
}

fn convert_rgb_to_yuv420<T>(img: &[u8], width: u32, height: u32, bytes_per_pixel: usize, store_uv: T) -> Vec<u8>
    where T: Fn(&mut Vec<u8>, &mut usize, usize, u8, u8) -> () {
    let frame_size: usize = (width * height) as usize;
    let chroma_size: usize = frame_size / 4;
    let mut y_index: usize = 0;
    let mut uv_index = frame_size;
    let mut yuv = vec![0; (width * height * 3 / 2) as usize];
    let mut r: u16;
    let mut g: u16;
    let mut b: u16;
    let mut y: u16;
    let mut u: i16;
    let mut v: i16;
    let mut index: usize = 0;
    for j in 0..height {
        for _ in 0..width {
            r = img[index * bytes_per_pixel] as u16;
            g = img[index * bytes_per_pixel + 1] as u16;
            b = img[index * bytes_per_pixel + 2] as u16;
            index += 1;
            y = (77 * r + 150 * g + 29 * b + 128) >> 8;
            u = ((-43 * r as i16 - 84 * g as i16 + 127 * b as i16 + 128) >> 8) + 128;
            v = ((127 * r as i16 - 106 * g as i16 - 21 * b as i16 + 128) >> 8) + 128;
            yuv[y_index] = clamp(y as i32);
            y_index += 1;
            if j % 2 == 0 && index % 2 == 0 {
                store_uv(&mut yuv, &mut uv_index, chroma_size, clamp(u as i32), clamp(v as i32));
            }
        }
    }
    yuv
}

fn clamp(val: i32) -> u8 {
    match val {
        ref v if v < &0 => 0,
        ref v if v > &255 => 255,
        v => v as u8,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rgb_to_yuv() {
        use super::convert_rgb_to_yuv420p;
        let rgb = vec![0u8; 12];
        let expected = vec![0u8, 0u8, 0u8, 0u8, 128u8, 128u8];
        let yuv = convert_rgb_to_yuv420p(&rgb, 2, 2, 3);
        assert_eq!(yuv.len(), rgb.len() / 2);
        for (val, exp) in yuv.iter().zip(expected.iter()) {
            assert_eq!(val, exp);
        }
    }

    #[test]
    fn rgba_to_yuv_from_file() {
        extern crate png;
        use std::fs::File;
        use super::convert_rgb_to_yuv420p;
        let decoder = png::Decoder::new(File::open("pic/ferris.png").unwrap());
        let (info, mut reader) = decoder.read_info().unwrap();
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf).unwrap();
        let yuv = convert_rgb_to_yuv420p(&buf, info.width, info.height, info.line_size / info.width as usize);
        assert_eq!(yuv.len(), buf.len() / 4 * 3 / 2);
    }
}
