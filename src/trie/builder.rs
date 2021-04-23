use std::path::Path;
use std::io::{BufWriter, Write};
use std::fs::File;
use byteorder::{WriteBytesExt, NativeEndian as NE};
use crate::{Utf16Char, Utf16String};
use crate::util::OutputUtil;
use crate::dictionary::build::AppResult;
use crate::trie::{AutoArray, Allocator, KeyStream};
use crate::trie::node;
use crate::trie::shrinktail;
use log::debug;


/// キー文字列のリストから、DoubleArrayを構築し、ファイルに保存する
/// # Arguments
/// * `key_list`  - DoubleArrayのキーとなる文字列のリスト. 破壊的に更新される
/// * `filepath`  - DoubleArrayを保存するファイルのパス
pub fn build(mut key_list: Vec<String>, filepath: &Path) -> AppResult<()> {
    // ソート and ユニーク
    key_list.sort();
    key_list.dedup();

    // UTF-16に変換する
    let mut utf16_key_list = Vec::with_capacity(key_list.len());
    for key in key_list {
        utf16_key_list.push(key.encode_utf16().collect::<Vec<_>>())
    }

    let mut bld = Builder::new(&utf16_key_list);
    let end = bld.ks_list.len();
    bld.build_impl(&mut Allocator::new(), 0, end, 0);
    bld.save(filepath)?;

    Ok(())
}

/// DoubleArrayの構築を行う
struct Builder<'a> {
    ks_list: Vec<KeyStream<'a>>,
    base: Vec<i32>,
    chck: Vec<Utf16Char>,
    begs: Vec<i32>,
    lens: Vec<Utf16Char>,
    tail: Utf16String
}

impl<'a> Builder<'a> {
    fn new(key_list: &'a [Utf16String]) -> Builder<'a> {
        let mut ks_list = Vec::with_capacity(key_list.len());
        for key in key_list {
            ks_list.push(KeyStream::new(key, 0))
        }

        Builder {
            ks_list,
            base: Vec::new(),
            chck: Vec::new(),
            begs: Vec::new(),
            lens: Vec::new(),
            tail: Vec::new()
        }
    }

    /// 構築したDoubleArrayをファイルに保存する
    /// # Arguments
    /// * `filepath`  - DoubleArrayを保存するファイルのパス
    fn save(self, filepath: &Path) -> AppResult<()> {
        let (tail, begs, lens) = shrinktail::shrink(self.tail, self.begs, self.lens);

        let mut node_size = self.chck.len();

        // 末尾の未使用部分を取り除く
        while node_size > 0 && self.chck[node_size - 1] == node::chck::VACANT_CODE {
            node_size -= 1;
        }
        node_size += node::chck::CODE_LIMIT as usize;  // 検索時の範囲外アクセスを防ぐために、余白を設ける
        debug!("node_size: {}, begs: {}, tail: {}", node_size, begs.len(), tail.len());
        let mut writer = BufWriter::new(File::create(filepath)?);
        writer.write_i32::<NE>(node_size as i32)?;
        writer.write_i32::<NE>(begs.len() as i32)?;
        writer.write_i32::<NE>(tail.len() as i32)?;

        // 4byte
        for n in begs {
            writer.write_i32::<NE>(n)?;
        }
        for i in 0..node_size {
            writer.write_i32::<NE>(*self.base.get(i).unwrap_or(&node::base::INIT_VALUE))?;
        }

        // 2byte
        for n in lens {
            writer.write_u16::<NE>(n)?;
        }
        for i in 0..node_size {
            writer.write_u16::<NE>(*self.chck.get(i).unwrap_or(&node::chck::VACANT_CODE))?;
        }

        writer.put_string(&tail)?;
        Ok(writer.flush()?)
    }

    fn build_impl(&mut self, alloca: &mut Allocator, beg: usize, end: usize, root_idx: usize) {
        if (end - beg) == 1 {
            // これ以降は単一の遷移パスしかないので、まとめてTAILに挿入してしまう
            self.insert_tail(beg, root_idx);
            return;
        }

        let mut end_list: Vec<i32> = Vec::new();
        let mut code_list: Utf16String = Vec::new();
        let mut prev: Utf16Char = node::chck::VACANT_CODE;

        // root_idxから遷移する文字を集める
        for i in beg..end {
            let cur = self.ks_list[i].read();
            if prev != cur {
                prev = cur;
                code_list.push(cur);
                end_list.push(i as i32);
            }
        }
        end_list.push(end as i32);

        // root_idxから派生(遷移)するノードを設定し、その各ノードに対して再帰的に処理を繰り返す
        let x = alloca.x_check(&code_list);
        for i in 0..code_list.len() {
            let x_node = self.set_node(code_list[i], root_idx, x);
            self.build_impl(alloca, end_list[i] as usize, end_list[i + 1] as usize, x_node);
        }
    }

    fn set_node(&mut self, code: Utf16Char, prev: usize, x_node: i32) -> usize {
        let next = x_node as usize + code as usize;
        self.base.set_auto(prev, x_node, node::base::INIT_VALUE);
        self.chck.set_auto(next, code, node::chck::VACANT_CODE);
        next
    }

    fn insert_tail(&mut self, beg: usize, node: usize) {
        let rest = self.ks_list[beg].rest();

        self.base.set_auto(node, node::base::ID(self.begs.len() as i32), node::base::INIT_VALUE);

        self.begs.push(self.tail.len() as i32);
        self.tail.extend_from_slice(rest);
        self.lens.push(rest.len() as Utf16Char);
    }
}
