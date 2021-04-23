use std::io::{self};
use std::cmp::min;
use crate::dictionary::{self, CharCategory, WordDic, SPACE_CHAR};
use crate::Utf16Str;
use crate::util::DirLike;


/// 未知語の検索を行う
pub struct Unknown {
    /// 文字カテゴリ管理クラス
    pub category: CharCategory,
    /// 文字カテゴリがSPACEの文字のID
    pub space_id: i32
}

impl Unknown {
    pub fn new(dir: &mut dyn DirLike) -> io::Result<Unknown> {
        let category = CharCategory::new(dir)?;
        Ok(Unknown {
            space_id: category.category(SPACE_CHAR).id, // NOTE: ' 'の文字カテゴリはSPACEに予約されている
            category
        })
    }

    pub fn search(&self, text: &Utf16Str, start: usize, wdic: &WordDic,
                  callback: &mut dyn dictionary::Callback) {
        let ch = text[start];
        let ct = self.category.category(ch);

        if !callback.is_empty() && !ct.invoke {
            return;
        }

        let is_space = ct.id == self.space_id;
        let limit = min(text.len(), (ct.length as usize) + start);
        for i in start..limit {
            wdic.search_from_trie_id(ct.id, start, (i - start) + 1, is_space, callback);
            if (i + 1) != limit && !self.category.is_compatible(ch, text[i + 1]) {
                return;
            }
        }

        if ct.group && limit < text.len() {
            for i in limit..text.len() {
                if !self.category.is_compatible(ch, text[i]) {
                    wdic.search_from_trie_id(ct.id, start, i - start, is_space, callback);
                    return;
                }
            }
            wdic.search_from_trie_id(ct.id, start, text.len() - start, is_space, callback);
        }
    }
}
