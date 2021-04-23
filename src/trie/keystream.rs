use crate::{Utf16Char, Utf16Str};
use super::node;


/// 文字列を文字のストリームとして扱うためのクラス。
/// readメソッドで個々の文字を順に読み込み、文字列の終端に達した場合には
/// `node::chck::TERMINATE_CODE` が返される。
pub struct KeyStream<'a> {
    s: &'a Utf16Str,
    cur: usize
}

impl<'a> KeyStream<'a> {
    pub fn new(key: &Utf16Str, start: usize) -> KeyStream {
        KeyStream {
            s: key,
            cur: start
        }
    }

    // このメソッドは動作的には、 rest().startsWith(prefix.substring(beg, len)) と等価。
    pub fn starts_with(&self, prefix: &[Utf16Char], beg: usize, len: usize) -> bool {
        if (self.s.len() - self.cur) < len {
            return false;
        }

        for i in 0..len {
            if self.s[self.cur + i] != prefix[beg + i] {
                return false;
            }
        }

        true
    }

    pub fn rest(&self) -> &[Utf16Char] {
        &self.s[self.cur..]
    }

    pub fn read(&mut self) -> Utf16Char {
        if self.eos() {
            node::chck::TERMINATE_CODE
        } else {
            let c = self.s[self.cur];
            self.cur += 1;
            c
        }
    }

    pub fn eos(&self) -> bool {
        self.cur == self.s.len()
    }
}
