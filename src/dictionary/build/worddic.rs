use std::path::Path;
use std::path::PathBuf;
use std::io::{BufWriter, Write};
use std::fs::File;
use byteorder::{WriteBytesExt, NativeEndian as NE};
use std::cmp;
use glob::glob;
use crate::util::*;
use crate::Utf16String;
use crate::dictionary::build::charcategory;
use crate::dictionary::build::*;
use crate::trie::{builder, Searcher};
use log::debug;


const CRLF: &[char] = &['\r', '\n'];

/// テキスト単語辞書をパースして、バイナリ単語辞書を構築する
pub struct WordDic {
    input_dir: PathBuf,
    encoding: String,
    output_dir: PathBuf,
    delim: String
}

impl WordDic {
    /// コンストラクタ
    /// # Arguments
    /// * `input_dir`  - テキスト単語辞書が配置されているディレクトリのパス
    /// * `encoding`   - テキスト単語辞書の文字列エンコーディング
    /// * `output_dir` - バイナリ単語辞書の保存先ディレクトリ
    /// * `delim`      - 単語辞書内の各項目の区切り文字
    pub fn new(input_dir: &Path, encoding: &str, output_dir: &Path, delim: String) -> WordDic {
        WordDic {
            input_dir: input_dir.to_owned(),
            encoding: encoding.to_owned(),
            output_dir: output_dir.to_owned(),
            delim
        }
    }

    /// 単語の表層形をキーに、対応する一意なIDを値としたtrieを作成し、保存する
    pub fn build_word_id_map(&mut self) -> AppResult<()> {
        let mut key_list: Vec<String> = Vec::new();

        // 未知語定義からキーを集める
        self.collect_key(&self.input_dir.join("unk.def"), &mut key_list, charcategory::KEY_PREFIX)?;

        // 単語辞書からキーを集める
        let paths = glob(self.input_dir.join("*.csv").to_str().unwrap())?;
        for entry in paths {
            match entry {
                Ok(ref csv_file) if csv_file.is_file() =>
                    self.collect_key(csv_file, &mut key_list, "")?,
                _ => (),
            }
        }

        builder::build(key_list, self.output_dir.join("word2id").as_path())?;
        Ok(())
    }

    fn collect_key(&self, path: &Path, key_list: &mut Vec<String>, prefix: &str) -> AppResult<()> {
        debug!("path: {}", path.display());
        let mut rl = ReadLine::new(path, &self.encoding)?;
        let mut s = String::new();

        loop {
            let len = rl.next(&mut s)?;
            if len < 1 {
                break;
            }
            let idx = s.find(&self.delim)
                .ok_or_else(|| rl.parse_error(format!(
                    "Word surface must be terminated with '{}'.", self.delim)))?;
            let key = &s[0..idx];
            //println!("{}: {}, [{}]", line_num, idx, key);
            key_list.push(prefix.to_string() + key);
        }

        Ok(())
    }

    /// バイナリ単語辞書を作成し、保存する。
    pub fn build_word_info(&mut self) -> AppResult<()> {
        let wid = Searcher::from_path(self.output_dir.join("word2id").as_path())?;
        let mut ws: Vec<Vec<WordInfo>> = Vec::with_capacity(wid.size());
        for _ in 0..wid.size() {
            ws.push(Vec::new());
        }

        // 未知語定義からデータを集める
        self.collect_word_info(self.input_dir.join("unk.def").as_path(), &wid, charcategory::KEY_PREFIX, &mut ws)?;

        // 単語辞書からデータを集める
        let paths = glob(self.input_dir.join("*.csv").to_str().unwrap())?;
        for entry in paths {
            match entry {
                Ok(ref csv_file) if csv_file.is_file() =>
                    self.collect_word_info(csv_file, &wid, "", &mut ws)?,
                _ => (),
            }
        }

        // 無駄な項目を削除する
        self.remove_unused_entry(&mut ws);

        // 単語情報を出力
        let mut wdat: Utf16String = Vec::new();
        {
            let mut writer = BufWriter::new(File::create(self.output_dir.join("word.inf").as_path())?);

            for wlist in &ws {
                // dataOffset
                for w in wlist {
                    writer.write_i32::<NE>(wdat.len() as i32)?;
                    wdat.extend_from_slice(&w.data);
                }
            }
            writer.write_i32::<NE>(wdat.len() as i32)?;

            for wlist in &ws {
                // leftId
                for w in wlist {
                    writer.write_i16::<NE>(w.left_id)?;
                }
            }
            writer.write_i16::<NE>(0)?;

            for wlist in &ws {
                // rightId
                for w in wlist {
                    writer.write_i16::<NE>(w.right_id)?;
                }
            }
            writer.write_i16::<NE>(0)?;

            for wlist in &ws {
                // cost
                for w in wlist {
                    writer.write_i16::<NE>(w.cost)?;
                }
            }
            writer.write_i16::<NE>(0)?;
            writer.flush()?
        }
        {
            // 単語データを出力
            let mut writer = BufWriter::new(File::create(self.output_dir.join("word.dat").as_path())?);
            writer.put_string(&wdat)?;
            writer.flush()?
        }
        {
            // 単語情報の配列へのインデックスを保存する
            let mut writer = BufWriter::new(File::create(self.output_dir.join("word.ary.idx").as_path())?);
            let mut beg_index = 0i32;
            for wlist in &ws {
                writer.write_i32::<NE>(beg_index)?;
                beg_index += wlist.len() as i32;
            }
            writer.write_i32::<NE>(beg_index)?;
            writer.flush()?
        }

        Ok(())
    }

    fn collect_word_info(&self, path: &Path, wid: &Searcher, prefix: &str, ws: &mut Vec<Vec<WordInfo>>) -> AppResult<()> {
        debug!("path: {}", path.display());
        let mut rl = ReadLine::new(path, &self.encoding)?;
        let mut s = String::new();

        loop {
            let len = rl.next(&mut s)?;
            if len < 1 {
                break;
            }
            let s = s.trim_end_matches(CRLF);

            // key
            let p1 = s.find(&self.delim).ok_or_else(||
                rl.parse_error(format!("Word surface must be terminated with '{}'.", self.delim)))?;
            if p1 == 0 {
                return Err(rl.parse_error("Empty Word surface."));
            }
            let mut start = p1 + 1;
            // left id
            let p2 = start + (&s[start..]).find(&self.delim).ok_or_else(||
                rl.parse_error(format!("Word left context id must be terminated with '{}'.", self.delim)))?;
            start = p2 + 1;
            // right id
            let p3 = start + (&s[start..]).find(&self.delim).ok_or_else(||
                rl.parse_error(format!("Word right context id must be terminated with '{}'.", self.delim)))?;
            start = p3 + 1;
            // cost
            let p4 = start + (&s[start..]).find(&self.delim).ok_or_else(||
                rl.parse_error(format!("Word cost must be terminated with '{}'.", self.delim)))?;
            start = p4 + 1;

            let data = &s[start..]; // data

//            debug!("p1: {}, p2: {}, p3: {}, p4: {}, s: '{}'", p1, p2, p3, p4, s);
//            debug!("left_id: {}, right_id: {}, cost: {}, data: '{}', key: '{}'",
//                   &s[(p1 + 1)..p2], &s[(p2 + 1)..p3], &s[(p3 + 1)..p4], data, &s[0..p1]);

            let key_utf16 = (prefix.to_string() + &s[0..p1]).encode_utf16().collect::<Vec<_>>();
            let id = wid.search(&key_utf16);
            if id < 0 {
                Err(rl.parse_error(format!("Word '{}' is unregistered in trie", &s[0..p1])))?;
            }

            ws[id as usize].push(WordInfo {
                left_id: (&s[(p1 + 1)..p2]).parse()?,
                right_id: (&s[(p2 + 1)..p3]).parse()?,
                cost: (&s[(p3 + 1)..p4]).parse()?,
                // UTF-16に変換する
                data: data.encode_utf16().collect::<Vec<_>>()
            });
        }

        Ok(())
    }

    /// 単語辞書から無駄な項目を除外する
    /// 参照: http://d.hatena.ne.jp/sile/20100227/1267260585
    fn remove_unused_entry(&self, ws: &mut Vec<Vec<WordInfo>>) {
        for wlist in ws {
            wlist.sort();
            let mut last = 0usize;
            for i in 1..wlist.len() {
                if !(wlist[last].left_id == wlist[i].left_id &&
                    wlist[last].right_id == wlist[i].right_id) {
                    last += 1;
                    wlist[last] = wlist[i].clone();
                }
            }
            wlist.truncate(last + 1);
        }
    }
}

struct WordInfo {
    left_id: i16,
    right_id: i16,
    cost: i16,
    data: Utf16String
}

impl cmp::Ord for WordInfo {
    fn cmp(&self, wi: &Self) -> cmp::Ordering {
        if self.left_id != wi.left_id {
            self.left_id.cmp(&wi.left_id)
        } else if self.right_id != wi.right_id {
            self.right_id.cmp(&wi.right_id)
        } else {
            self.cost.cmp(&wi.cost)
        }
    }
}

impl cmp::PartialOrd for WordInfo {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::PartialEq for WordInfo {
    fn eq(&self, wi: &Self) -> bool {
        self.left_id.eq(&wi.left_id)
            && self.right_id.eq(&wi.right_id)
            && self.cost.eq(&wi.cost)
    }
}

impl cmp::Eq for WordInfo {}

impl Clone for WordInfo {
    fn clone(&self) -> Self {
        WordInfo {
            left_id: self.left_id,
            right_id: self.right_id,
            cost: self.cost,
            data: self.data.clone()
        }
    }
}
