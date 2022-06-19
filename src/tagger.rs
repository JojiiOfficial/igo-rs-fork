use std::io;
use std::path::Path;
use std::rc::Rc;

use log::debug;

use crate::dictionary::{self, Matrix, Unknown, ViterbiNode, WordDic};
use crate::morpheme::Morpheme;
use crate::util::DirLike;
use crate::{Utf16Str, Utf16String};

type ViterbiNodeList = Vec<Rc<ViterbiNode>>;

fn bos_nodes() -> ViterbiNodeList {
    vec![Rc::new(ViterbiNode::make_boseos())]
}

fn empty_vec() -> ViterbiNodeList {
    Vec::new()
}

/// 形態素解析を行う
#[derive(Clone)]
pub struct Tagger {
    wdc: WordDic,
    unk: Unknown,
    mtx: Matrix,
}

impl Tagger {
    /// バイナリ辞書を読み込んで、形態素解析器のインスタンスを作成する
    /// # Arguments
    /// * `data_dir` - バイナリ辞書があるディレクトリ
    pub fn new(data_dir: &Path) -> io::Result<Tagger> {
        let mut dir_like = data_dir;
        Ok(Tagger {
            wdc: WordDic::new(&mut dir_like)?,
            unk: Unknown::new(&mut dir_like)?,
            mtx: Matrix::new(&mut dir_like)?,
        })
    }

    /// zip等にアーカイブしたバイナリ辞書を読み込んで、形態素解析器のインスタンスを作成する
    ///
    /// WebAssembly等、ファイルシステムに直接アクセスできない環境向け
    /// # Arguments
    /// * `dir` - アーカイブファイルのイメージ
    pub fn load_from_dir(dir: &mut dyn DirLike) -> io::Result<Tagger> {
        Ok(Tagger {
            wdc: WordDic::new(dir)?,
            unk: Unknown::new(dir)?,
            mtx: Matrix::new(dir)?,
        })
    }

    /// 形態素解析を行う
    /// # Arguments
    /// * `text` - 解析対象テキスト
    pub fn parse<'a, 'b>(&'a self, text: &'b str) -> Vec<Morpheme<'a, 'b>> {
        let utf16_text: Utf16String = text.encode_utf16().collect::<Vec<_>>();
        let utf8_offsets = utf8_char_offsets(text, utf16_text.len());

        self.parse_impl(&utf16_text)
            .into_iter()
            .map(|n| {
                let from = utf8_offsets[n.start];
                let to = utf8_offsets[n.start + (n.length as usize)];

                Morpheme {
                    surface: &text[from..to],
                    feature: self.wdc.word_data(n.word_id),
                    start: n.start,
                }
            })
            .collect()
    }

    /// 分かち書きを行う
    /// # Arguments
    /// * `text` - 分かち書きされるテキスト
    pub fn wakati(&self, text: &str) -> Vec<String> {
        let utf16_text: Utf16String = text.encode_utf16().collect::<Vec<_>>();
        self.parse_impl(&utf16_text)
            .into_iter()
            .map(|n| String::from_utf16_lossy(&utf16_text[n.start..n.start + (n.length as usize)]))
            .collect()
    }

    fn parse_impl(&self, utf16_text: &Utf16Str) -> Vec<Rc<ViterbiNode>> {
        let len = utf16_text.len();
        debug!("utf16_text.len: {}", len);
        let mut nodes_ary: Vec<ViterbiNodeList> = Vec::with_capacity(len + 1);
        nodes_ary.push(bos_nodes());
        for _ in 1..=len {
            nodes_ary.push(empty_vec());
        }

        let mut f = MakeLattice::new(self, nodes_ary.into_boxed_slice());
        for i in 0..len {
            if !f.nodes_ary[i].is_empty() {
                f.set(i);
                self.wdc.search(utf16_text, i, &mut f); // 単語辞書から形態素を検索
                self.unk.search(utf16_text, i, &self.wdc, &mut f); // 未知語辞書から形態素を検索
            }
        }
        let nodes_ary: Box<[ViterbiNodeList]> = f.into_inner();

        let mut cur: Rc<ViterbiNode> = self
            .set_mincost_node(ViterbiNode::make_boseos(), &nodes_ary[len])
            .prev
            .unwrap();

        // reverse
        let mut result: Vec<Rc<ViterbiNode>> = Vec::with_capacity(len / 2);
        result.push(cur.clone());
        while cur.prev.is_some() {
            cur = cur.prev.as_ref().cloned().unwrap();
            result.push(cur.clone());
        }
        result.pop();
        result.reverse();

        result
    }

    fn set_mincost_node(&self, mut vn: ViterbiNode, prevs: &ViterbiNodeList) -> ViterbiNode {
        let mut min_idx = 0;
        let p = &prevs[0];
        let mut min_cost: i32 = p.cost + self.mtx.link_cost(p.right_id, vn.left_id);

        for i in 1..prevs.len() {
            let p = &prevs[i];
            let cost = p.cost + self.mtx.link_cost(p.right_id, vn.left_id);
            if cost < min_cost {
                min_cost = cost;
                min_idx = i;
            }
        }

        vn.cost += min_cost;
        vn.prev = Some(prevs[min_idx].clone());

        vn
    }
}

fn utf8_char_offsets(text: &str, num_chars: usize) -> Box<[usize]> {
    let mut utf8_offsets: Vec<usize> = Vec::with_capacity(num_chars + 1);
    let mut offset = 0usize;
    for c in text.chars() {
        utf8_offsets.push(offset);
        if c.len_utf16() == 2 {
            utf8_offsets.push(offset);
        }
        offset += c.len_utf8();
    }
    utf8_offsets.push(offset);
    utf8_offsets.into_boxed_slice()
}

struct MakeLattice<'a> {
    tagger: &'a Tagger,
    nodes_ary: Box<[ViterbiNodeList]>,
    i: usize,
    prevs: ViterbiNodeList,
    empty: bool,
}

impl<'a> MakeLattice<'a> {
    fn new(tagger: &Tagger, nodes_ary: Box<[ViterbiNodeList]>) -> MakeLattice {
        MakeLattice {
            tagger,
            nodes_ary,
            i: 0,
            prevs: empty_vec(),
            empty: true,
        }
    }

    fn set(&mut self, i: usize) {
        self.i = i;
        self.prevs = self.nodes_ary[i].clone();
        self.nodes_ary[i] = empty_vec();
        self.empty = true;
    }

    fn into_inner(self) -> Box<[ViterbiNodeList]> {
        self.nodes_ary
    }
}

impl<'a> dictionary::Callback for MakeLattice<'a> {
    fn call(&mut self, vn: ViterbiNode) {
        self.empty = false;
        let end = self.i + (vn.length as usize);

        if vn.is_space {
            self.nodes_ary[end].extend(self.prevs.iter().cloned());
        } else {
            self.nodes_ary[end].push(Rc::new(self.tagger.set_mincost_node(vn, &self.prevs)));
        }
    }

    fn is_empty(&self) -> bool {
        self.empty
    }
}

#[cfg(test)]
impl Tagger {
    pub fn unknown(&self) -> &Unknown {
        &self.unk
    }
}
