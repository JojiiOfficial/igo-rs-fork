use std::convert::From;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::mem;
use std::path::Path;

use byteorder::{NativeEndian as NE, ReadBytesExt, WriteBytesExt};
use encoding_rs::*;

use crate::dictionary::build::*;
use crate::Utf16Char;

pub trait InputUtil: io::Read {
    fn get_int(&mut self) -> io::Result<i32> {
        self.read_i32::<NE>()
    }

    fn get_int_array(&mut self, count: usize) -> io::Result<Box<[i32]>> {
        let mut v = vec![0i32; count];
        for i in 0..count {
            v[i] = self.read_i32::<NE>()?;
        }
        Ok(v.into_boxed_slice())
    }

    fn get_short_array(&mut self, count: usize) -> io::Result<Box<[i16]>> {
        let mut v = vec![0i16; count];
        for i in 0..count {
            v[i] = self.read_i16::<NE>()?;
        }
        Ok(v.into_boxed_slice())
    }

    fn get_char_array(&mut self, count: usize) -> io::Result<Box<[Utf16Char]>> {
        let mut v = vec![0u16; count];
        for i in 0..count {
            v[i] = self.read_u16::<NE>()?;
        }
        Ok(v.into_boxed_slice())
    }

    fn get_string(&mut self, count: usize) -> io::Result<Box<[Utf16Char]>> {
        let mut v = vec![0u16; count];
        for i in 0..count {
            v[i] = self.read_u16::<NE>()?;
        }
        Ok(v.into_boxed_slice())
    }
}

impl<R: io::Read + ?Sized> InputUtil for R {}

pub fn read_all_as_chars(dir: &mut dyn DirLike, path: &str) -> io::Result<Box<[Utf16Char]>> {
    let file_len = dir.file_size(path)?;
    let file = dir.open(path)?;
    let mut reader = BufReader::new(file);
    reader.get_string((file_len as usize) / mem::size_of::<u16>())
}

pub fn read_all_as_int_array(dir: &mut dyn DirLike, path: &str) -> io::Result<Box<[i32]>> {
    let file_len = dir.file_size(path)?;
    let file = dir.open(path)?;
    let mut reader = BufReader::new(file);
    reader.get_int_array((file_len as usize) / mem::size_of::<i32>())
}

pub trait OutputUtil: io::Write {
    fn put_string(&mut self, str: &[Utf16Char]) -> io::Result<()> {
        for i in 0..str.len() {
            self.write_u16::<NE>(str[i])?;
        }
        Ok(())
    }
}

impl<W: io::Write + ?Sized> OutputUtil for W {}

pub struct ReadLine<'a> {
    reader: BufReader<File>,
    line_number: i32,
    path: &'a Path,
    decoder: Option<&'static Encoding>,
    encoded_buf: Vec<u8>,
}

impl<'a> ReadLine<'a> {
    pub fn new(file_path: &'a Path, encoding_name: &str) -> AppResult<ReadLine<'a>> {
        let file = File::open(file_path)?;
        let encoding = Encoding::for_label(encoding_name.as_bytes())
            .ok_or_else(|| format!("Unknown encoding; {}", encoding_name))?;

        //        debug!("encoding.name: {}", encoding.name());
        Ok(ReadLine {
            reader: BufReader::new(file),
            line_number: 0,
            path: file_path,
            decoder: if encoding != UTF_8 {
                Some(encoding)
            } else {
                None
            },
            encoded_buf: Vec::new(),
        })
    }

    pub fn next(&mut self, read_buf: &mut String) -> AppResult<usize> {
        match self.decoder {
            Some(encoding) => {
                self.next_with_decoder(encoding.new_decoder_without_bom_handling(), read_buf)
            }
            None => self.next_without_decoder(read_buf),
        }
    }

    fn next_without_decoder(&mut self, read_buf: &mut String) -> AppResult<usize> {
        read_buf.clear();
        let r = self.reader.read_line(read_buf);
        if r.as_ref().map(|len| *len > 0).unwrap_or(false) {
            self.line_number += 1;
        }
        r.map_err(AppError::from)
    }

    fn next_with_decoder(
        &mut self,
        mut decoder: Decoder,
        decode_buf: &mut String,
    ) -> AppResult<usize> {
        self.encoded_buf.clear();
        decode_buf.clear();
        let len = self
            .reader
            .read_until(b'\n', self.encoded_buf.as_mut())
            .map_err(AppError::from)?;
        if len < 1 {
            Ok(0) // EOF
        } else {
            decode_buf.reserve(
                decoder
                    .max_utf8_buffer_length_without_replacement(len)
                    .expect("overflow"),
            );
            let (result, _) =
                decoder.decode_to_string_without_replacement(&self.encoded_buf, decode_buf, true);
            match result {
                DecoderResult::InputEmpty => {
                    self.line_number += 1;
                    Ok(decode_buf.len())
                }
                DecoderResult::Malformed(_, _) => Err(AppError::from("Decoder Malformed")),
                DecoderResult::OutputFull => Err(AppError::from("Decoder OutputFull")),
            }
        }
    }

    pub fn parse_error<S: Into<String>>(&self, msg: S) -> AppError {
        AppError::Parse {
            message: msg.into(),
            path: self.path.to_path_buf(),
            line_number: self.line_number,
        }
    }

    pub fn convert_error<E: Error>(&self, e: E) -> AppError {
        self.parse_error(e.to_string())
    }
}

/// Virtual directory trait
///
/// Local file system or archive file (Zip, Tar ...)
pub trait DirLike {
    /// 指定したパスのファイルサイズを取得
    fn file_size(&mut self, path: &str) -> io::Result<u64>;
    /// 指定したパスのファイルを開く
    fn open(&mut self, path: &str) -> io::Result<Box<dyn io::Read>>;
}

/// DirLike implement for Local file system
impl DirLike for &Path {
    fn file_size(&mut self, path: &str) -> io::Result<u64> {
        let buf = self.join(path);
        Ok(fs::metadata(buf)?.len())
    }

    fn open(&mut self, path: &str) -> io::Result<Box<dyn io::Read>> {
        let buf = self.join(path);
        File::open(buf).map(|f| Box::new(f) as Box<dyn io::Read>)
    }
}

#[allow(dead_code)]
pub mod debug {
    use std::fs::File;
    use std::io::Write;

    pub fn dump_string_list(list: &[String], path: &str) {
        let mut f = File::create(path).unwrap();
        for s in list {
            f.write_all(s.as_bytes()).unwrap();
            f.write_all(b"\n").unwrap();
        }
    }
}
