use std::io::{self, BufReader};
use crate::trie::Searcher;
use crate::dictionary;
use crate::dictionary::ViterbiNode;
use crate::util::*;
use crate::{Utf16Char, Utf16Str};
use log::debug;


pub struct WordDic {
    trie: Searcher,
    data: String,
    indices: Box<[i32]>,

    /// costs[単語ID] = 単語のコスト
    costs: Box<[i16]>,
    /// left_ids[単語ID] = 単語の左文脈ID
    left_ids: Box<[i16]>,
    /// right_ids[単語ID] = 単語の右文脈ID
    right_ids: Box<[i16]>,
    /// data_offsets[単語ID] = 単語の素性データの開始位置
    data_offsets: Box<[i32]>
}

impl WordDic {
    pub fn new(dir: &mut dyn DirLike) -> io::Result<WordDic> {
        let word2id_path = "word2id";
        let dat_path ="word.dat";
        let idx_path = "word.ary.idx";
        let inf_path = "word.inf";

        let inf_size = dir.file_size(inf_path)?;
        let mut reader = BufReader::new(dir.open(inf_path)?);
        let word_count = (inf_size / (4 + 2 + 2 + 2)) as usize;
        debug!("word_count: {}", word_count);

        let word_data = read_all_as_chars(dir, dat_path)?;
        let data_offsets = reader.get_int_array(word_count)?;
        let (word_data, data_offsets) = convert2utf8_data(&word_data, &data_offsets);

        Ok(WordDic {
            trie: Searcher::new(dir.open(word2id_path)?)?,
            data: word_data,
            indices: read_all_as_int_array(dir, idx_path)?,

            data_offsets,
            left_ids: reader.get_short_array(word_count)?,
            right_ids: reader.get_short_array(word_count)?,
            costs: reader.get_short_array(word_count)?
        })
    }


    pub fn search(&self, text: &Utf16Str, start: usize, callback: &mut dyn dictionary::Callback) {
        self.trie.each_common_prefix(text, start, |start: usize, offset: i32, trie_id: i32| {
            /*
             * common-prefix検索でキーが見つかった場合に呼び出されるクロージャー
             * each_common_prefix()で該当するキーの部分文字列が見つかった都度に呼び出される
             *
             * @param start  入力テキストの検索開始位置
             * @param offset 一致した部分文字列の終端位置
             * @param trie_id 一致した部分文字列のID
             */
            let trie_id = trie_id as usize;
            let end: i32 = self.indices[trie_id + 1];

            for i in self.indices[trie_id]..end {
                let idx = i as usize;
                callback.call(ViterbiNode {
                    word_id: i,
                    start,
                    length: offset as i16,
                    cost: i32::from(self.costs[idx]),
                    left_id: self.left_ids[idx],
                    right_id: self.right_ids[idx],
                    is_space: false,
                    prev: None
                });
            }
        });
    }

    pub fn search_from_trie_id(&self, trie_id: i32, start: usize, word_length: usize,
                               is_space: bool, callback: &mut dyn dictionary::Callback) {
        let trie_id = trie_id as usize;
        let end = self.indices[trie_id + 1];
        for i in self.indices[trie_id]..end {
            let idx = i as usize;
            callback.call(ViterbiNode {
                word_id: i,
                start,
                length: word_length as i16,
                cost: i32::from(self.costs[idx]),
                left_id: self.left_ids[idx],
                right_id: self.right_ids[idx],
                is_space,
                prev: None
            });
        }
    }

    pub fn word_data(&self, word_id: i32) -> &str {
        let word_id = word_id as usize;
        &self.data[
            (self.data_offsets[word_id] as usize) .. (self.data_offsets[word_id + 1] as usize)]
    }
}

// word_data()用に、予めString型へ変換しておく
fn convert2utf8_data(utf16_str: &[Utf16Char], offsets: &[i32]) -> (String, Box<[i32]>) {
    let mut buf = String::with_capacity(utf16_str.len() * 3);
    let mut new_offset = vec![0i32; offsets.len()];

    for word_id in 0..(offsets.len() - 1) {
        let offset = offsets[word_id] as usize;
        let next_offset = offsets[word_id + 1] as usize;
        let word_data = String::from_utf16_lossy(&utf16_str[offset..next_offset]);
        buf.push_str(&word_data);
        new_offset[word_id + 1] = buf.len() as i32;
    }
    debug!("buf size: {} / {}", buf.len(), buf.capacity());

    (buf, new_offset.into_boxed_slice())
}
