use num_traits::NumCast;

/**
 * Resample an image.
 *
 * If the source and destination area differ in size, the area will be resized
 * using bilinear interpolation.
 *
 * Parameters:
 *   dst  - The destination image.
 *   src  - The source image.
 *   dstW - The width of the area to copy to.
 *   dstH - The height of the area to copy to.
 *   srcW - The width of the area to copy from.
 *   srcH - The height of the area to copy from.
 */
pub fn image_resample<I: GenericImageView<Pixel = Rgba<u8>>>(
    src: &I,
    dst_w: u32,
    dst_h: u32,
) -> ImageBuffer<I::Pixel, Vec<<I::Pixel as Pixel>::Subpixel>>
where
    I::Pixel: 'static,
    <I::Pixel as Pixel>::Subpixel: 'static,
{
    let mut dst = ImageBuffer::new(dst_w, dst_h);

    let src_w = src.width();
    let src_h = src.height();

    println!("{:?} {:?}", src_w, src_h);
    for y in 0..dst_h {
        for x in 0..dst_w {
            let (mut sx1, mut sx2): (f32, f32);
            let (mut sx, mut sy): (f32, f32);
            let mut s_pixels: f32 = 0.0;
            let (mut red, mut green, mut blue, mut alpha): (f32, f32, f32, f32) =
                (0.0, 0.0, 0.0, 0.0);
            let mut alpha_factor: f32;
            let (mut alpha_sum, mut contrib_sum): (f32, f32) = (0.0, 0.0);
            let sy1 = (y as f32) * (src_h as f32) / (dst_h as f32);
            let sy2 = ((y + 1) as f32) * (src_h as f32) / (dst_h as f32);
            sy = sy1;

            while sy < sy2 {
                let mut y_portion: f32;
                if sy.floor() == sy1.floor() {
                    y_portion = 1.0 - (sy - sy.floor());
                    if y_portion > (sy2 - sy1) {
                        y_portion = sy2 - sy1;
                    }
                    sy = sy.floor();
                } else if sy == sy2.floor() {
                    y_portion = sy2 - sy2.floor();
                } else {
                    y_portion = 1.0;
                }
                sx1 = (x as f32) * (src_w as f32) / (dst_w as f32);
                sx2 = ((x + 1) as f32) * (src_w as f32) / (dst_w as f32);
                sx = sx1;
                while sx < sx2 {
                    let mut x_portion: f32;
                    let p_contribution: f32;
                    if sx.floor() == sx1.floor() {
                        x_portion = 1.0 - (sx - sx.floor());
                        if x_portion > (sx2 - sx1) {
                            x_portion = sx2 - sx1;
                        }
                        sx = sx.floor();
                    } else if sx == sx2.floor() {
                        x_portion = sx2 - sx2.floor();
                    } else {
                        x_portion = 1.0;
                    }
                    p_contribution = x_portion * y_portion;
                    // Should be RGBA?
                    let p = src.get_pixel(sx as u32, sy as u32);
                    #[allow(deprecated)]
                    let (p1, p2, p3, p4) = p.channels4();
                    // Possible rounding error here from primitive
                    let p1: f32 = NumCast::from(p1).unwrap();
                    let p2: f32 = NumCast::from(p2).unwrap();
                    let p3: f32 = NumCast::from(p3).unwrap();
                    let p4: f32 = NumCast::from(p4).unwrap();

                    alpha_factor = (127.0) * p_contribution; // should be 0 for these tests
                    red += p1 * alpha_factor;
                    green += p2 * alpha_factor;
                    blue += p3 * alpha_factor;
                    alpha += p4 * alpha_factor;
                    alpha_sum += alpha_factor;
                    contrib_sum += p_contribution;
                    s_pixels += x_portion * y_portion;
                    sx += 1.0;
                }
                sy += 1.0;
            }

            if s_pixels != 0.0 {
                red /= s_pixels;
                green /= s_pixels;
                blue /= s_pixels;
                alpha /= s_pixels;
            }
            if alpha_sum != 0.0 {
                if contrib_sum != 0.0 {
                    alpha_sum /= contrib_sum;
                }
                red /= alpha_sum;
                green /= alpha_sum;
                blue /= alpha_sum;
            }
            /* Round up closest next channel value and clamp to max channel value */
            red = if red >= 255.5 { 255.0 } else { red + 0.5 };
            blue = if blue >= 255.5 { 255.0 } else { blue + 0.5 };
            green = if green >= 255.5 { 255.0 } else { green + 0.5 };
            let alpha = if alpha >= 127.0 + 0.5 {
                127.0
            } else {
                alpha + 0.5
            };
            let t = Rgba([red as u8, green as u8, blue as u8, alpha as u8]);
            //println!("First pix: {:?}", t);
            dst.put_pixel(x, y, t);
            //println!("dst pixel: {:?}", dst.get_pixel(x, y));
        }
    }
    dst
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::Path;
    const PATH: &str = "reference/";
    const RESIZE: [&str; 3] = ["r_resize.txt", "g_resize.txt", "b_resize.txt"];
    const ORIGINAL: [&str; 3] = ["r_buf.txt", "g_buf.txt", "b_buf.txt"];

    #[test]
    fn image_resample_test() {
        let img = image::open("files/peppers.jpg").unwrap();
        let mut imgbuf = image::ImageBuffer::new(200, 200);
        let r_vec: Vec<u8> = read_i32_vector_file((PATH.to_owned() + ORIGINAL[0]).to_string());
        let g_vec: Vec<u8> = read_i32_vector_file((PATH.to_owned() + ORIGINAL[1]).to_string());
        let b_vec: Vec<u8> = read_i32_vector_file((PATH.to_owned() + ORIGINAL[2]).to_string());

        let img_dimensions = (img.height() * img.width()) as usize;
        assert_eq!(img_dimensions, r_vec.len());
        assert_eq!(img_dimensions, r_vec.len());
        assert_eq!(img_dimensions, b_vec.len());

        // Check if the image matches the source
        for (p, pix) in izip!(img.pixels(), imgbuf.enumerate_pixels_mut()) {
            let index: usize = pix.0 as usize + (200 * (pix.1 as usize));
            *pix.2 = image::Rgba([r_vec[index], g_vec[index], b_vec[index], 0]);
            //println!("{:?}", r_vec[index]);
            //assert_eq!(p.2[0], r_vec[index]);
        }

        imgbuf = image_resample(&imgbuf, 128, 128);
        //println!("imgbuf {:?}", imgbuf.get_pixel(0, 0));
        //let mut dynam = image::DynamicImage::ImageRgb8(imgbuf);
        imgbuf.save("test.jpg").unwrap();

        let r_resize: Vec<u8> = read_i32_vector_file((PATH.to_owned() + RESIZE[0]).to_string());
        let g_resize: Vec<u8> = read_i32_vector_file((PATH.to_owned() + RESIZE[1]).to_string());
        let b_resize: Vec<u8> = read_i32_vector_file((PATH.to_owned() + RESIZE[2]).to_string());

        for (pix, r, g, b) in izip!(imgbuf.enumerate_pixels(), r_resize, g_resize, b_resize) {
            println!("xy: {:?} {:?}", pix.0, pix.1);
            assert_eq!(pix.2[0], r);
            assert_eq!(pix.2[1], g);
            assert_eq!(pix.2[2], b);
        }
    }
}