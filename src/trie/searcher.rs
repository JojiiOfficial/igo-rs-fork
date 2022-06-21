use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use byteorder::{NativeEndian as NE, ReadBytesExt};

use crate::util::InputUtil;
use crate::{Utf16Char, Utf16Str};

use super::keystream::KeyStream;
use super::node;

/// DoubleArray検索用のstruct
#[derive(Clone)]
pub struct Searcher {
    key_set_size: usize,
    base: Box<[i32]>,
    chck: Box<[Utf16Char]>,
    begs: Box<[i32]>,
    lens: Box<[i16]>,
    tail: Box<[Utf16Char]>,
}

impl Searcher {
    /// 保存されているDoubleArrayを読み込んで、このstructのインスタンスを作成する
    pub fn new<R: io::Read>(src: R) -> io::Result<Searcher> {
        let mut reader = BufReader::new(src);

        let node_sz = reader.read_i32::<NE>()?;
        let tind_sz = reader.read_i32::<NE>()?;
        let tail_sz = reader.read_i32::<NE>()?;
        //        debug!("tind_sz: {}, node_sz: {}, tail_sz: {}", tind_sz, node_sz, tail_sz);

        Ok(Searcher {
            key_set_size: tind_sz as usize,
            begs: reader.get_int_array(tind_sz as usize)?,
            base: reader.get_int_array(node_sz as usize)?,
            lens: reader.get_short_array(tind_sz as usize)?,
            chck: reader.get_char_array(node_sz as usize)?,
            tail: reader.get_string(tail_sz as usize)?,
        })
    }

    pub fn from_path(file_path: &Path) -> io::Result<Searcher> {
        Self::new(File::open(file_path)?)
    }

    /// DoubleArrayに格納されているキーの数を返す
    pub fn size(&self) -> usize {
        self.key_set_size
    }

    /// キーを検索する
    /// キーが見つかった場合はそのIDを、見つからなかった場合は-1を返す
    /// # Arguments
    /// * `key` - 検索対象のキー文字列
    pub fn search(&self, key: &Utf16Str) -> i32 {
        let mut node = self.base[0];
        let mut input = KeyStream::new(key, 0);

        loop {
            let code = input.read();
            let idx = node as usize + code as usize;
            node = self.base[idx];

            if self.chck[idx] == code {
                if node >= 0 {
                    continue;
                } else if input.eos() || self.key_exists(&input, node) {
                    return node::base::ID(node);
                }
            }
            return -1;
        }
    }

    /// common-prefix検索を行う
    ///
    /// 条件に一致するキーが見つかる度に、callbackが呼び出される
    /// # Arguments
    /// * `key` - 検索対象のキー文字列
    /// * `start` - 検索対象となるキー文字列の最初の添字
    /// * `callback` - 一致を検出した場合に呼び出されるクロージャー
    pub fn each_common_prefix<F>(&self, key: &Utf16Str, start: usize, mut callback: F)
    where
        F: FnMut(usize, i32, i32) -> (),
    {
        let mut node = self.base[0];
        let mut offset: i32 = -1;
        let mut input = KeyStream::new(key, start);

        loop {
            let code = input.read();
            offset += 1;
            let terminal_idx = (node as usize) + (node::chck::TERMINATE_CODE as usize);

            if self.chck[terminal_idx] == node::chck::TERMINATE_CODE {
                callback(start, offset, node::base::ID(self.base[terminal_idx]));
                if code == node::chck::TERMINATE_CODE {
                    return;
                }
            }

            let idx = (node as usize) + (code as usize);
            node = self.base[idx];
            if self.chck[idx] == code {
                if node >= 0 {
                    continue;
                } else {
                    self.call_if_key_including(&input, node, start, offset, callback);
                }
            }
            return;
        }
    }

    fn call_if_key_including<F>(
        &self,
        input: &KeyStream,
        node: i32,
        start: usize,
        offset: i32,
        mut callback: F,
    ) where
        F: FnMut(usize, i32, i32) -> (),
    {
        let id = node::base::ID(node) as usize;
        if self.begs.len() <= id || self.lens.len() <= id {
            return;
        }
        if input.starts_with(
            self.tail.as_ref(),
            self.begs[id] as usize,
            self.lens[id] as usize,
        ) {
            callback(start, offset + i32::from(self.lens[id]) + 1, id as i32);
        }
    }

    fn key_exists(&self, input: &KeyStream, node: i32) -> bool {
        let id = node::base::ID(node) as usize;
        let s =
            &self.tail[(self.begs[id] as usize)..(self.begs[id] as usize + self.lens[id] as usize)];
        *input.rest() == *s
    }
}
