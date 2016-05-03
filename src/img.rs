
use std::path;
use std::iter::Iterator;
use qrcode::QrCode;

use image;
use image::{ImageBuffer, DynamicImage, GenericImage};


pub struct Image {
    pub width: u32,
    pub height: u32,
    img_buf: DynamicImage,
}

impl Image {
    pub fn new<P: AsRef<path::Path> + ToString>(path: P) -> Image {
        let img_buf = image::open(&path).unwrap();
        let (width, height) = img_buf.dimensions();
        Image {
            width: width,
            height: height,
            img_buf: img_buf
        }
    }

    pub fn from(img_buf: DynamicImage) -> Image {
        let (width, height) = img_buf.dimensions();
        Image {
            width: width,
            height: height,
            img_buf: img_buf
        }
    }

    pub fn from_qr(code: &str, width: u32) -> Image {
        let code = QrCode::new(code.as_bytes()).unwrap();
        let code_width = code.width();
        let point_width = width / (code_width as u32 + 2);
        let quite_width = (width % (code_width as u32 + 2)) / 2 + point_width;
        let img_buf = ImageBuffer::from_fn(width, width, |x, y| {
            let is_white =  x < quite_width || y < quite_width
                || x >= (width - quite_width) || y >= (width - quite_width)
                || !code[(
                    ((x-quite_width) / point_width) as usize,
                    ((y-quite_width) / point_width) as usize
                )];
            if is_white {
                image::Rgb([0xFF, 0xFF, 0xFF])
            } else {
                image::Rgb([0, 0, 0])
            }
        });
        Image {
            width: width,
            height: width,
            img_buf: DynamicImage::ImageRgb8(img_buf)
        }
    }

    pub fn is_blank_pixel(&self, x: u32, y: u32) -> bool {
        let pixel = self.img_buf.get_pixel(x, y);
        // full transprant OR is white
        if pixel[3] == 0 || (pixel[0] & pixel[1] & pixel[2]) == 0xFF {
            true
        } else {
            false
        }
    }

    pub fn bitimage_lines(&self, density: u32) -> BitimageLines {
        BitimageLines {line: 0, density: density, image: self}
    }

    fn get_line(&self, num: u32, density: u32) -> Option<Box<[u8]>> {
        let n = self.height as u32 / density;
        let y = num - 1;
        if y >= n {
            return None
        }

        let c = density / 8;
        let mut data: Vec<u8> = vec![0; (self.width * c) as usize];
        // println!(">>> num={}, density={}, n={}, y={}, c={}, data.len()={}",
        //          num, density, n, y, c, data.len());
        for x in 0..self.width {
            for b in 0..density {
                let i = x * c + (b >> 3);
                // println!("x={}, b={}, i={}, b>>8={}", x, b, i, b>>3);
                let l = y * density + b;
                if l < self.height && !self.is_blank_pixel(x, l) {
                    data[i as usize] += 0x80 >> (b & 0x07);
                }
            }
        }
        Some(data.into_boxed_slice())
    }

    pub fn get_raster(&self) -> Box<[u8]>{
        let n = self.width / 8;
        let mut data: Vec<u8> = vec![0; (self.width * self.height) as usize];
        for y in 0..self.height {
            for x in 0..n {
                for b in 0..8 {
                    let i = x * 8 + b;
                    if i < self.width && !self.is_blank_pixel(i, y) {
                        data[(y*n + x) as usize] += 0x80 >> (b & 0x7);
                    }
                }
            }
        }
        data.into_boxed_slice()
    }
}


pub struct BitimageLines<'a>{
    line: u32,
    density: u32,
    image: &'a Image,
}

impl<'a> Iterator for BitimageLines<'a> {
    type Item = Box<[u8]>;

    fn next(&mut self) -> Option<Box<[u8]>> {
        self.line += 1;
        self.image.get_line(self.line, self.density)
    }
}
