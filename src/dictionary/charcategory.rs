use std::io::{self, BufReader};

use crate::Utf16Char;
use crate::util::*;

pub const SPACE_CHAR: Utf16Char = 0x0020u16;

#[derive(Debug)]
pub struct Category {
    pub id: i32,
    pub length: i32,
    pub invoke: bool,
    pub group: bool,
}

pub struct CharCategory {
    categories: Box<[Category]>,
    char2id: Box<[i32]>,
    eql_masks: Box<[i32]>,
}

impl CharCategory {
    pub fn new(dir: &mut dyn DirLike) -> io::Result<CharCategory> {
        let path = "code2category";
        let src_len = dir.file_size(path)?;
        let mut reader = BufReader::new(dir.open(path)?);

        Ok(CharCategory {
            categories: Self::read_categories(dir)?,
            char2id: reader.get_int_array((src_len / 4 / 2) as usize)?,
            eql_masks: reader.get_int_array((src_len / 4 / 2) as usize)?,
        })
    }

    pub fn is_compatible(&self, code1: Utf16Char, code2: Utf16Char) -> bool {
        (self.eql_masks[code1 as usize] & self.eql_masks[code2 as usize]) != 0
    }

    fn read_categories(dir: &mut dyn DirLike) -> io::Result<Box<[Category]>> {
        let data = read_all_as_int_array(dir, "char.category")?;
        let size = data.len() / 4;
        let mut v = Vec::with_capacity(size);
        for i in 0..size {
            v.push(Category {
                id: data[i * 4],
                length: data[i * 4 + 1],
                invoke: data[i * 4 + 2] == 1,
                group: data[i * 4 + 3] == 1,
            });
        }
        Ok(v.into_boxed_slice())
    }

    pub fn category(&self, code: Utf16Char) -> &Category {
        &self.categories[self.char2id[code as usize] as usize]
    }
}
