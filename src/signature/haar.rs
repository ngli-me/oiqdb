use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgba};
use image::imageops::FilterType;
use num_traits::NumCast;
use std::cmp::Ordering;

pub const NUM_PIXELS: usize = 128;
pub const NUM_PIXELS_SQUARED: usize = NUM_PIXELS.pow(2);
pub const NUM_COEFS: usize = 40;
const GD_ALPHA_MAX: u8 = 127;

struct ValStruct {
    v: f32,
}

impl PartialOrd for ValStruct {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let v1 = f32::abs(self.v);
        let v2 = f32::abs(other.v);
        v1.partial_cmp(&v2)
    }
}

impl PartialEq for ValStruct {
    fn eq(&self, other: &Self) -> bool {
        let v1 = f32::abs(self.v);
        let v2 = f32::abs(other.v);
        v1 == v2
    }
}

pub type LuminT = [f32; 3];
pub struct SignatureT ([i16; 3 * NUM_COEFS]);

impl Default for SignatureT {
    fn default() -> Self {
        SignatureT([0; 3 * NUM_COEFS])
    }
}

pub fn transform_char(img: DynamicImage) ->
    (Vec<f32>, Vec<f32>, Vec<f32>) {

    let img = img.resize_exact(128, 128, FilterType::Triangle);
    let (mut a, mut b, mut c) = rgb_to_yiq_conversion(img);
    haar_2d(&mut a);
    haar_2d(&mut b);
    haar_2d(&mut c);

    // Reintroduce the skipped scaling factors
    a[0] /= 256.0 * 128.0;
    b[0] /= 256.0 * 128.0;
    c[0] /= 256.0 * 128.0;

    (a, b, c)
}

fn haar_2d(a: &mut Vec<f32>) {
    // Rows
    for i in (0..NUM_PIXELS_SQUARED).step_by(NUM_PIXELS) {
        let (mut h, mut h1): (usize, usize);
        let mut c: f32 = 1.0;
        h = NUM_PIXELS;
        while h > 1 {
            let (mut j1, mut j2, mut k): (usize, usize, usize) = (i, i, 0);

            h1 = h >> 1; // h1 = h / 2
            c = c * 0.7071; // 1/sqrt(2)
            let mut t: Vec<f32> = vec![0.0; h1];
            while k < h1 {
                let j21: usize = j2 + 1;
                t[k]  = (a[j2] - a[j21]) * c;
                a[j1] = a[j2] + a[j21];

                k  += 1;
                j1 += 1;
                j2 += 2;
            }

            // Write back subtraction results:
            a.splice((i + h1)..(i + h), t);

            h = h1;
        }
        // Fix first element of each row:
        a[i] *= c;
    }

    // Columns
    for i in 0..NUM_PIXELS {
        let (mut h, mut h1): (usize, usize);
        let mut c: f32 = 1.0;
        h = NUM_PIXELS;
        while h > 1 {
            let (mut j1, mut j2, mut k): (usize, usize, usize) = (i, i, 0);

            h1 = h >> 1; // h1 = h / 2
            c = c * 0.7071; // 1/sqrt(2)
            let mut t: Vec<f32> = vec![0.0; h1];
            while k < h1 {
                let j21: usize = j2 + NUM_PIXELS;
                t[k]  = (a[j2] - a[j21]) * c;
                a[j1] = a[j2] + a[j21];

                k  += 1;
                j1 += NUM_PIXELS;
                j2 += NUM_PIXELS * 2;
            }

            // Write back subtraction results:
            let mut j1 = i + (h1 * NUM_PIXELS);
            for k in 0..h1 {
                a[j1] = t[k];
                j1 += NUM_PIXELS;
            }
            h = h1;
        }
        // Fix first element of each row:
        a[i] *= c;
    }
}

fn rgb_to_yiq_conversion(img: DynamicImage) ->
    (Vec<f32>, Vec<f32>, Vec<f32>) {

    let mut cdata1: Vec<f32> = Vec::with_capacity(128 * 128);
    let mut cdata2: Vec<f32> = Vec::with_capacity(128 * 128);
    let mut cdata3: Vec<f32> = Vec::with_capacity(128 * 128);

    for p in img.pixels() {
        // The iteration order is x = 0 to width then y = 0 to height
        // RGB -> YIQ colorspace conversion; Y luminance, I,Q chrominance.
        // If RGB in [0..255] then Y in [0..255] and I,Q in [-127..127].
        let r: f32 = p.2[0] as f32;
        let g: f32 = p.2[1] as f32;
        let b: f32 = p.2[2] as f32;

        cdata1.push(0.299 * r + 0.587 * g + 0.114 * b);
        cdata2.push(0.596 * r - 0.275 * g - 0.321 * b);
        cdata3.push(0.212 * r - 0.523 * g + 0.311 * b);
    }

    (cdata1, cdata2, cdata3)
}

// Find the NUM_COEFS largest numbers in cdata[] (in magnitude that is)
// and store their indices in sig[].
fn get_m_largest(mut cdata: Vec<f32>) -> Vec<i16> {
    let mut cnt: usize = 0;
    let i: i16 = 0;
    let mut vq: Vec<ValStruct> = Vec::new();

    // Skip i = 0, since it goes into avglf
    for val in cdata.iter_mut().skip(1) {
        vq.push(ValStruct { v: *val });
    }

    vq.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Set sorted size to NUM_COEFS, discard others
    let vq = &vq[..NUM_COEFS];

    // The index is positive if the coefficient was positive; the index is
    // negative if the coefficient was negative.
    for val in vq {

    }

    vec![0; NUM_PIXELS]
}

// Determines a total of NUM_COEFS positions in the image that have the
// largest magnitude (absolute value) in color value. Returns linearized
// coordinates in sig1, sig2, and sig3. avgl are the [0,0] values.
// The order of occurrence of the coordinates in sig doesn't matter.
// Complexity is 3 x NUM_PIXELS^2 x 2log(NUM_COEFS).
pub fn calc_haar(cdata1: Vec<f32>, cdata2: Vec<f32>, cdata3: Vec<f32>) {//->
    //(LuminT, SignatureT) {
    let avglf: Vec<f32> = vec![cdata1[0], cdata2[0], cdata3[0]];

    // Color channel 1
    let mut c: Vec<i16>  = get_m_largest(cdata1);

    // Color channel 2
    //c.append(get_m_largest(cdata2));

    // Color channel 3
    //c.append(get_m_largest(cdata2));

    //(avglf, SignatureT{[i16; 128 * 3]})
}

/**
 * Function: gdImageCopyResampled
 *
 * Copy a resampled area from an image to another image
 *
 * If the source and destination area differ in size, the area will be resized
 * using bilinear interpolation for truecolor images, and nearest-neighbor
 * interpolation for palette images.
 *
 * Parameters:
 *   dst  - The destination image.
 *   src  - The source image.
 *   dstX - The x-coordinate of the upper left corner to copy to.
 *   dstY - The y-coordinate of the upper left corner to copy to.
 *   srcX - The x-coordinate of the upper left corner to copy from.
 *   srcY - The y-coordinate of the upper left corner to copy from.
 *   dstW - The width of the area to copy to.
 *   dstH - The height of the area to copy to.
 *   srcW - The width of the area to copy from.
 *   srcH - The height of the area to copy from.
 *
 * See also:
 *   - <gdImageCopyResized>
 *   - <gdImageScale>
 */
pub fn gd_image_resample<I: GenericImageView<Pixel=Rgba<u8>>>(
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
    for y in 0..dst_h {
        for x in 0..dst_w {
            let (sy1, sy2, mut sx1, mut sx2): (f32, f32, f32, f32);
            let (mut sx, mut sy): (f32, f32);
            let mut s_pixels: f32 = 0.0;
            let (mut red, mut green, mut blue, mut alpha): (f32, f32, f32, f32) = (0.0, 0.0, 0.0, 0.0);
            let mut alpha_factor: f32;
            let (mut alpha_sum, mut contrib_sum): (f32, f32) = (0.0, 0.0);
            sy1 = (y as f32) * (src_h as f32) / (dst_h as f32);
            sy2 = ((y + 1) as f32) * (src_h as f32) / (dst_h as f32);
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
                sx1 = (x as f32) * (src_w as f32) / (dst_h as f32);
                sx2 = ((x + 1) as f32) * (src_w as f32) / (dst_h as f32);
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

                    alpha_factor = p_contribution;
                    red   += p1 * alpha_factor;
                    green += p2 * alpha_factor;
                    blue  += p3 * alpha_factor;
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
                red   /= alpha_sum;
                green /= alpha_sum;
                blue  /= alpha_sum;
            }
            /* Round up closest next channel value and clamp to max channel value */
            red   = if red   >= 255.5 { 255.0 } else { red   + 0.5 };
            blue  = if blue  >= 255.5 { 255.0 } else { blue  + 0.5 };
            green = if green >= 255.5 { 255.0 } else { green + 0.5 };
            let alpha = if alpha >= GD_ALPHA_MAX as f32 + 0.5 { GD_ALPHA_MAX as f32 } else { alpha + 0.5 };
            let t = Rgba([
                red   as u8,
                green as u8,
                blue  as u8,
                alpha as u8
            ]);
            dst.put_pixel(x, y, t);
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
    use image::imageops::FilterType;
    use itertools::izip;

    #[test]
    fn compare_resize() {
        let img = image::open("files/peppers.jpg").unwrap();
        let filecontent = img.resize_exact(128, 128, FilterType::Triangle);
        filecontent.save("test2.jpg").unwrap();
        let mut cdata1: Vec<i32> = Vec::with_capacity(128 * 128);
        let mut cdata2: Vec<i32> = Vec::with_capacity(128 * 128);
        let mut cdata3: Vec<i32> = Vec::with_capacity(128 * 128);
        let mut imgbuf = image::ImageBuffer::new(128, 128);

        let r_vec: Vec<i32> = read_i32_vector_file("files/r_peppers".to_string());
        let g_vec: Vec<i32> = read_i32_vector_file("files/g_peppers".to_string());
        let b_vec: Vec<i32> = read_i32_vector_file("files/b_peppers".to_string());

        for (p, pix) in izip!(filecontent.pixels(), imgbuf.enumerate_pixels_mut()) {
            // The iteration order is x = 0 to width then y = 0 to height
            // RGB -> YIQ colorspace conversion; Y luminance, I,Q chrominance.
            // If RGB in [0..255] then Y in [0..255] and I,Q in [-127..127].
            let r: i32 = p.2[0].into();
            let g: i32 = p.2[1].into();
            let b: i32 = p.2[2].into();
            let index: usize = pix.0 as usize + (128 * (pix.1 as usize));
            *pix.2 = image::Rgb([r_vec[index] as u8, g_vec[index] as u8, b_vec[index] as u8]);

            cdata1.push(r);
            cdata2.push(g);
            cdata3.push(b);
        }
        imgbuf.save("test3.jpg").unwrap();

        compare_vals_ints(r_vec, cdata1);
        compare_vals_ints(g_vec, cdata2);
        compare_vals_ints(b_vec, cdata3);
    }


    //#[test]
    fn compare_yiq_conversion() {
        let img = image::open("files/peppers.jpg").unwrap();
        let filecontent = img.resize_exact(128, 128, FilterType::Triangle);
        let (a, b, c) = rgb_to_yiq_conversion(filecontent);

        let y = read_i32_vector_file("files/y_peppers".to_string());
        let i = read_i32_vector_file("files/i_peppers".to_string());
        let q = read_i32_vector_file("files/q_peppers".to_string());

        compare_vals(y, a);
        compare_vals(i, b);
        compare_vals(q, c);
    }

    fn read_i32_vector_file(filename: String) -> Vec<i32> {
        let mut ret: Vec<i32> = vec![];
        if let Ok(lines) = read_lines(filename) {
            // Consumes the iterator, returns an (Optional) String
            // For these I'm using only 1 line
            for line in lines.flatten() {
                let iter = line[4..(line.len() - 1)].split(",");

                for value in iter {
                    match value.parse::<i32>() {
                        Ok(n) => ret.push(n),
                        Err(e) => println!("Parsing error for : {}, with error {}", value, e),
                    }
                }
            }
        }
        ret
    }

    fn compare_vals(v1: Vec<i32>, v2: Vec<f32>) {
        for (reference, val) in izip!(v1, v2) {
            assert_eq!(reference, val.trunc() as i32);
        }
    }

    fn compare_vals_ints(v1: Vec<i32>, v2: Vec<i32>) {
        for (reference, val) in izip!(v1, v2) {
            assert_eq!(reference, val);
        }
    }

    // The output is wrapped in a Result to allow matching on errors.
    // Returns an Iterator to the Reader of the lines of the file.
    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
}
