//
// SimProc library
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::error::FromError;
use std::fmt;
use std::fs;
use std::num::Int;

/// A BMP header
#[derive(Debug)]
pub struct Header {
    pub size: u32,
    pub reserved: u32,
    pub offset: u32,
}

/// A BMP header
#[derive(Debug)]
pub struct Dib {
    pub width: u32,
    pub height: u32,
    pub planes: u16,
    pub bpp: u16,
    pub comp: u32,
    pub size: u32,
    pub ppm_x: u32,
    pub ppm_y: u32,
    pub colors: u32,
    pub imp_colors: u32,
}

/// A type to represent the color un RGBX format
#[derive(Debug)]
pub struct Rgbx(u8, u8, u8, u8);

impl Rgbx {

    fn from_u32(n: u32) -> Rgbx { Rgbx(
        (n.to_be() >> 24) as u8,
        (n.to_be() >> 16) as u8,
        (n.to_be() >> 8) as u8,
        (n.to_be() >> 0) as u8,
    )}
}

/// The color table of a BMP
pub type ColorTable = Vec<Rgbx>;

/// BMP pixel data
pub type Pixels = Vec<usize>;

/// A BMP bitmap
#[derive(Debug)] 
pub struct Bitmap {
    pub header: Header,
    pub dib: Dib,
    pub colors: ColorTable,
    pub pixels: Pixels,
}

/// A BMP load error
#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    UnexpectedEof,
    BadMagic,
    UnsupportedDib,
    UnsupportedBpp,
}

impl FromError<io::Error> for LoadError {
    fn from_error(err: io::Error) -> LoadError {
        LoadError::Io(err)
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &LoadError::Io(ref cause) => 
                write!(f, "unexpected IO error: {}", cause),
            &LoadError::UnexpectedEof => 
                write!(f, "unexpected end of file"),
            &LoadError::BadMagic => 
                write!(f, "invalid magic number in BMP header"),
            &LoadError::UnsupportedDib => 
                write!(f, "unsupported DIP block (only BITMAPINFOHEADER is supported)"),
            &LoadError::UnsupportedBpp => 
                write!(f, "unsupported bits per pixel (only 4 bpp supported)"),
        }
    }
}

macro_rules! word {
    ($b:expr, $i:expr) => (Int::from_le($b[$i] as u16 | (($b[$i+1] as u16) << 8)))
}

macro_rules! dword {
    ($b:expr, $i:expr) => (
        Int::from_le($b[$i] as u32 | (($b[$i+1] as u32) << 8) | 
        (($b[$i+2] as u32) << 16) | (($b[$i+3] as u32) << 24)))
}

impl Bitmap {

    /// Load a bitmap from the given file. 
    pub fn load(filename: &str) -> Result<Bitmap, LoadError> {
        let mut file = try!(fs::File::open(filename));
        Bitmap::read(&mut file)
    }

    /// Read a bitmap
    pub fn read<R: io::Read>(input: &mut R) -> Result<Bitmap, LoadError> {
        let mut binput = io::BufReader::new(input);
        let hd = try!(Bitmap::read_header(&mut binput));
        let dib = try!(Bitmap::read_dib(&mut binput));
        let ct = try!(Bitmap::read_color_table(&mut binput, dib.colors as usize));
        let pixels = try!(Bitmap::read_pixels(
            &mut binput, dib.width as usize, dib.height as usize, dib.bpp));
        Ok(Bitmap { header: hd, dib: dib , colors: ct, pixels: pixels })
    }

    fn read_section<R: io::Read>(input: &mut R, ebytes: usize) -> Result<Vec<u8>, LoadError> {
        let mut buff = vec![0u8; ebytes];
        let nbytes = try!(input.read(&mut buff));        
        
        if nbytes != ebytes { Err(LoadError::UnexpectedEof)}
        else { Ok(buff) }
    }

    fn read_header<R: io::Read>(input: &mut R) -> Result<Header, LoadError> {
        let buff = try!(Bitmap::read_section(input, 14));

        // First two bytes must be `BM` in ASCII
        if buff[0] != 0x42 || buff[1] != 0x4d { return Err(LoadError::BadMagic)}

        // Read the size, reserved and offset
        let size = dword!(buff, 2);
        let reserved = dword!(buff, 6);
        let offset = dword!(buff, 10);

        Ok(Header { size: size, reserved: reserved, offset: offset })
    }

    fn read_dib<R: io::Read>(input: &mut R) -> Result<Dib, LoadError> {
        let buff = try!(Bitmap::read_section(input, 40));

        // The indicated DIB length must be 40
        if dword!(buff, 0) != 40 { return Err(LoadError::UnsupportedDib)}

        // Read the fields
        let width = dword!(buff, 4);
        let height = dword!(buff, 8);
        let planes = word!(buff, 12);
        let bpp = word!(buff, 14);
        let compression = dword!(buff, 16);
        let size = dword!(buff, 20);
        let ppm_x = dword!(buff, 24);
        let ppm_y = dword!(buff, 28);
        let colors = dword!(buff, 32);
        let imp_colors = dword!(buff, 36);

        Ok(Dib { 
            width: width, 
            height: height, 
            planes: planes,
            bpp: bpp,
            comp: compression,
            size: size,
            ppm_x: ppm_x,
            ppm_y: ppm_y,
            colors: colors,
            imp_colors: imp_colors,
        })
    }

    fn read_color_table<R: io::Read>(
            input: &mut R, ncolors: usize) -> Result<ColorTable, LoadError> {
        let buff = try!(Bitmap::read_section(input, 4*ncolors));
        let mut table = ColorTable::new();
        for i in 0..ncolors {
            table.push(Rgbx::from_u32(dword!(buff, 4*i)));
        }
        Ok(table)
    }

    fn read_pixels<R: io::Read>(
            input: &mut R, cols: usize, rows: usize, bpp: u16) -> Result<Pixels, LoadError> {
        match bpp {
            4 => Bitmap::read_pixels_4bpp(input, cols, rows),
            _ => Err(LoadError::UnsupportedBpp),
        }
    }

    fn read_pixels_4bpp<R: io::Read>(
            input: &mut R, cols: usize, rows: usize) -> Result<Pixels, LoadError> {
        let rbytes = ((4 * cols + 31) / 32) * 4;
        let ebytes = rows * rbytes;
        let buff = try!(Bitmap::read_section(input, ebytes));
        let mut pixels = Pixels::new();

        for r in 0..rows {
            for c in 0..cols {
                let b = buff[r * rbytes + c / 2];
                pixels.push((if c % 2 == 0 { b >> 4 } else { b & 0x0f }) as usize);
            }
        }
        Ok(pixels)
    }
}

#[cfg(test)]
mod test {

    use std::io::Cursor;

    use super::*;

    #[test]
    #[should_fail(expected = "BadMagic")]
    fn should_fail_read_bad_magic() {
        let buff: Vec<u8> = vec![
            0xcc, 0x0dd, // <-- 0xccdd is a bad magic
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        Bitmap::read(&mut Cursor::new(buff)).unwrap();
    }

    #[test]
    #[should_fail(expected = "UnexpectedEof")]
    fn should_fail_read_unexpected_eof_in_header() {
        let buff: Vec<u8> = vec![
            0x42, 0x04d, 
            0x00, 0x00, 0x00, 0x00,
                                    // <-- more bytes expected
        ];
        Bitmap::read(&mut Cursor::new(buff)).unwrap();
    }

    #[test]
    #[should_fail(expected = "UnsupportedDib")]
    fn should_fail_read_unsupported_dib() {
        let buff: Vec<u8> = vec![
            0x42, 0x04d, 
            0x52, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x46, 0x00, 0x00, 0x00,

            0xaa, 0xbb, 0xcc, 0xdd, // <-- 0xaabbccdd is a unknown size
            0x03, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00,
            0x01, 0x00, 
            0x04, 0x00, 
            0x00, 0x00, 0x00, 0x00,
            0x0c, 0x00, 0x00, 0x00,
            0x13, 0x0b, 0x00, 0x00,
            0x13, 0x0b, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
        ];
        Bitmap::read(&mut Cursor::new(buff)).unwrap();
    }

    #[test]
    #[should_fail(expected = "UnexpectedEof")]
    fn should_fail_read_unexpected_eof_in_color_table() {
        let buff: Vec<u8> = vec![
            0x42, 0x04d, 
            0x52, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x46, 0x00, 0x00, 0x00,

            0x28, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00,
            0x01, 0x00, 
            0x04, 0x00, 
            0x00, 0x00, 0x00, 0x00,
            0x0c, 0x00, 0x00, 0x00,
            0x13, 0x0b, 0x00, 0x00,
            0x13, 0x0b, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,

            0x00, 0x00, 0x00, 0x00,
            0xff, 0x00, 0x00, 0x00,
                                    // <-- more bytes expected
        ];
        Bitmap::read(&mut Cursor::new(buff)).unwrap();
    }

    #[test]
    #[should_fail(expected = "UnexpectedEof")]
    fn should_fail_read_unexpected_eof_in_pixel_store() {
        let buff: Vec<u8> = vec![
            0x42, 0x04d, 
            0x52, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x46, 0x00, 0x00, 0x00,

            0x28, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00,
            0x01, 0x00, 
            0x04, 0x00, 
            0x00, 0x00, 0x00, 0x00,
            0x0c, 0x00, 0x00, 0x00,
            0x13, 0x0b, 0x00, 0x00,
            0x13, 0x0b, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,

            0x00, 0x00, 0x00, 0x00,
            0xff, 0x00, 0x00, 0x00,
            0x00, 0xff, 0x00, 0x00,
            0x00, 0x00, 0xff, 0x00,

            0x13, 0x20, 0x00, 0x00,
                                    // <-- more bytes expected
        ];
        Bitmap::read(&mut Cursor::new(buff)).unwrap();
    }

    #[test]
    #[should_fail(expected = "UnsupportedBpp")]
    fn should_fail_read_unsupported_pixel_format() {
        let buff: Vec<u8> = vec![
            0x42, 0x04d, 
            0x52, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x46, 0x00, 0x00, 0x00,

            0x28, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00,
            0x01, 0x00, 
            0x01, 0x00, // <-- 0x0100 unsupported
            0x00, 0x00, 0x00, 0x00,
            0x0c, 0x00, 0x00, 0x00,
            0x13, 0x0b, 0x00, 0x00,
            0x13, 0x0b, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,

            0x00, 0x00, 0x00, 0x00,
            0xff, 0x00, 0x00, 0x00,
            0x00, 0xff, 0x00, 0x00,
            0x00, 0x00, 0xff, 0x00,

            0x13, 0x20, 0x00, 0x00,
            0x30, 0x30, 0x00, 0x00,
            0x23, 0x10, 0x00, 0x00,
        ];
        Bitmap::read(&mut Cursor::new(buff)).unwrap();
    }

    #[test]
    fn should_read() {
        let buff: Vec<u8> = vec![
            0x42, 0x04d, 
            0x52, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x46, 0x00, 0x00, 0x00,

            0x28, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00,
            0x01, 0x00, 
            0x04, 0x00, 
            0x00, 0x00, 0x00, 0x00,
            0x0c, 0x00, 0x00, 0x00,
            0x13, 0x0b, 0x00, 0x00,
            0x13, 0x0b, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,

            0x00, 0x00, 0x00, 0x00,
            0xff, 0x00, 0x00, 0x00,
            0x00, 0xff, 0x00, 0x00,
            0x00, 0x00, 0xff, 0x00,

            0x13, 0x20, 0x00, 0x00,
            0x30, 0x30, 0x00, 0x00,
            0x23, 0x10, 0x00, 0x00,
        ];
        Bitmap::read(&mut Cursor::new(buff)).unwrap();
    }
}
