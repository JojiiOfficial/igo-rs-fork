use crate::dictionary::build::*;
use crate::dictionary::charcategory::{Category, SPACE_CHAR};
use crate::trie::Searcher;
use crate::util::*;
use byteorder::{NativeEndian as NE, WriteBytesExt};
use std::cmp;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

pub const KEY_PREFIX: &str = "\x02";

/// 文字カテゴリ定義を保持したバイナリデータを作成する
pub struct CharCategory {
    input_dir: PathBuf,
    encoding: String,
    output_dir: PathBuf,
}

impl CharCategory {
    /// コンストラクタ
    /// # Arguments
    /// * `input_dir`  - テキスト単語辞書が配置されているディレクトリのパス
    /// * `encoding`   - テキスト単語辞書の文字列エンコーディング
    /// * `output_dir` - バイナリ単語辞書の保存先ディレクトリ
    pub fn new(input_dir: &Path, encoding: &str, output_dir: &Path) -> CharCategory {
        CharCategory {
            input_dir: input_dir.to_owned(),
            encoding: encoding.to_owned(),
            output_dir: output_dir.to_owned(),
        }
    }

    /// 文字カテゴリ定義のバイナリデータを作成する
    pub fn build(self) -> AppResult<()> {
        // 文字カテゴリの定義を取得する
        let ccmap = self.parse_char_category_def()?;

        {
            // 文字カテゴリの定義を保存する
            let mut categories: Vec<&Category> = Vec::new();
            for e in ccmap.values() {
                categories.push(e);
            }
            self.save_char_category_map(categories)?;
        }

        // 文字とカテゴリのマッピングを取得/保存する
        self.build_code_category_map(ccmap)?;

        Ok(())
    }

    fn parse_char_category_def(&self) -> AppResult<HashMap<String, Category>> {
        let path = self.input_dir.join("char.def");
        let too_few_fields = |rl: &ReadLine| -> AppError {
            rl.parse_error("Invalid char category definition (too few fields).")
        };
        let parse_0or1 = |str: Option<&str>, rl: &ReadLine| -> AppResult<bool> {
            str.ok_or_else(|| too_few_fields(rl)).and_then(|s| {
                if s == "1" {
                    Ok(true)
                } else if s == "0" {
                    Ok(false)
                } else {
                    Err(rl.parse_error(
                        "Invalid char category definition (INVOKE must be '0' or '1').",
                    ))
                }
            })
        };
        let mut rl = ReadLine::new(path.as_path(), &self.encoding)?;
        let srch = Searcher::from_path(self.output_dir.join("word2id").as_path())?;
        let mut map = HashMap::new();

        let mut s = String::new();
        loop {
            let len = rl.next(&mut s).map_err(|e| rl.convert_error(e))?;
            if len < 1 {
                break;
            }
            let line = s.trim_end();
            if line.is_empty() || line.starts_with('#') || line.starts_with('0') {
                continue;
            }

            let mut ss = line.split_whitespace();
            let name = ss.next().ok_or_else(|| too_few_fields(&rl))?;
            let invoke: bool = parse_0or1(ss.next(), &rl)?; // 0 or 1
            let group: bool = parse_0or1(ss.next(), &rl)?; // 0 or 1
                                                           // positive integer
            let length: i32 = ss
                .next()
                .ok_or_else(|| too_few_fields(&rl))
                .and_then(|s| s.parse().map_err(AppError::from))?;
            let key_utf16 = (KEY_PREFIX.to_string() + name)
                .encode_utf16()
                .collect::<Vec<_>>();
            let id = srch.search(&key_utf16);

            if length < 0 {
                return Err(rl.parse_error(
                    "Invalid char category definition (LENGTH must be 0 or positive integer).",
                ));
            }
            if id < 0 {
                return Err(rl.parse_error(format!("Category '{}' is unregistered in trie", name)));
            }
            map.insert(
                name.to_string(),
                Category {
                    id,
                    length,
                    invoke,
                    group,
                },
            );
        }

        // "DEFAULT"と"SPACE"は必須カテゴリ
        if !map.contains_key("DEFAULT") {
            return Err(rl.parse_error("Missing mandatory category 'DEFAULT'."));
        }
        if !map.contains_key("SPACE") {
            return Err(rl.parse_error("Missing mandatory category 'SPACE'."));
        }
        Ok(map)
    }

    fn save_char_category_map(&self, mut categories: Vec<&Category>) -> AppResult<()> {
        let mut writer = BufWriter::new(File::create(
            self.output_dir.join("char.category").as_path(),
        )?);
        categories.sort();
        for e in categories {
            writer.write_i32::<NE>(e.id)?;
            writer.write_i32::<NE>(e.length)?;
            writer.write_i32::<NE>(if e.invoke { 1 } else { 0 })?;
            writer.write_i32::<NE>(if e.group { 1 } else { 0 })?;
        }
        Ok(writer.flush()?)
    }

    fn build_code_category_map(&self, map: HashMap<String, Category>) -> AppResult<()> {
        let mut chars: Vec<Rc<CharId>> = Vec::with_capacity(0x10_000);
        {
            let dft = Rc::new(CharId::new(map["DEFAULT"].id));
            for _ in 0..0x10_000 {
                chars.push(dft.clone());
            }
        }

        {
            let path = self.input_dir.join("char.def");
            let mut rl = ReadLine::new(path.as_path(), &self.encoding)?;
            let mut s = String::new();
            loop {
                let len = rl.next(&mut s).map_err(|e| rl.parse_error(e.to_string()))?;
                if len < 1 {
                    break;
                }
                let line = s.trim_end();
                if line.is_empty() || !line.starts_with('0') {
                    continue;
                }

                let mut ss = line.split_whitespace();
                let beg: i32;
                let end: i32;
                let ss0 = ss.next().ok_or_else(|| rl.parse_error("Too few fields"))?;
                if let Some(idx) = ss0.find("..") {
                    beg = i32::from_str_radix(&ss0[2..idx], 16).map_err(|e| rl.convert_error(e))?;
                    end = i32::from_str_radix(&ss0[(idx + 2 + 2)..], 16)
                        .map_err(|e| rl.convert_error(e))?;
                } else {
                    beg = i32::from_str_radix(&ss0[2..], 16).map_err(|e| rl.convert_error(e))?;
                    end = beg;
                }

                if !(0 <= beg && beg <= 0xFFFF && 0 <= end && end <= 0xFFFF && beg <= end) {
                    return Err(rl.parse_error("Wrong UCS2 code specified."));
                }

                // 文字カテゴリ及び互換カテゴリの取得
                let category_name = ss.next().ok_or_else(|| rl.parse_error("Too few fields"))?;
                let category = map.get(category_name).ok_or_else(|| {
                    rl.parse_error(format!("Category '{}' is undefined.", category_name))
                })?;
                let ch = {
                    let mut ch = CharId::new(category.id);
                    while let Some(f) = ss.next() {
                        if f.starts_with('#') {
                            break;
                        }
                        let category = map.get(f).ok_or_else(|| {
                            rl.parse_error(format!("Category '{}' is undefined.", f))
                        })?;
                        ch.add(category.id);
                    }
                    Rc::new(ch)
                };

                // カテゴリ登録
                for i in beg..=end {
                    chars[i as usize] = ch.clone();
                }
            }

            if chars[SPACE_CHAR as usize].id != map["SPACE"].id {
                return Err(rl.parse_error("0x0020 is reserved for 'SPACE' category"));
            }
        }

        let mut writer = BufWriter::new(File::create(
            self.output_dir.join("code2category").as_path(),
        )?);
        for c in &chars {
            writer.write_i32::<NE>(c.id)?;
        }
        for c in &chars {
            writer.write_i32::<NE>(c.mask)?;
        }

        Ok(writer.flush()?)
    }
}

impl cmp::Ord for Category {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl cmp::PartialOrd for Category {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl cmp::Eq for Category {}

struct CharId {
    id: i32,
    mask: i32,
}

impl CharId {
    pub fn new(id: i32) -> CharId {
        let mut c = CharId { id, mask: 0 };
        c.add(id);
        c
    }

    pub fn add(&mut self, i: i32) {
        self.mask |= 1 << i;
    }
}
