use std::cmp;
use bit_set::BitSet;
use crate::Utf16Str;
use crate::trie::node;


/// DoubleArray構築時に使用可能なノードを割り当てる
pub struct Allocator {
    lnk: Vec<LinkNode>,
    bset: BitSet
}

impl Allocator {
    pub fn new() -> Allocator {
        let mut instance = Allocator {
            lnk: vec![LinkNode::new(0, 0)],
            bset: BitSet::new()
        };
        instance.resize_link(node::chck::CODE_LIMIT as usize * 10);
        instance
    }

    /// 遷移に使用される文字のリストを受け取り、それらを割り当て可能なベースノードのインデックスを返す。
    /// # Arguments
    /// * `codes`  - 遷移文字リスト。昇順にソートされている必要がある
    /// # Return
    /// 引数の遷移文字群を割り当て可能なベースノードのインデックス
    pub fn x_check(&mut self, codes: &Utf16Str) -> i32 {
        let mut cur = self.lnk[node::chck::CODE_LIMIT as usize].next;
        loop {
            let x = cur - codes[0] as usize;
            if !self.bset.contains(x) && self.can_allocate(codes, x) {
                self.bset.insert(x);  // このベースノードは使用中だというマークをつける

                for c in codes {
                    self.alloc(x + *c as usize);
                }
                return x as i32;
            }

            cur = self.lnk[cur].next;
        }
    }

    fn can_allocate(&self, codes: &Utf16Str, x: usize) -> bool {
        for c in codes.iter().skip(1) {
            if (x + *c as usize) < self.lnk.len() && self.lnk[x + *c as usize].next == 0 {
                return false;
            }
        }
        true
    }

    fn alloc(&mut self, node: usize) {
        while node >= (self.lnk.len() - 1) {
            self.resize_link(0);
        }

        let p = self.lnk[node].prev;
        self.lnk[p].next = self.lnk[node].next;
        let n = self.lnk[node].next;
        self.lnk[n].prev = self.lnk[node].prev;
        self.lnk[node].next = 0;
    }

    fn resize_link(&mut self, hint: usize) {
        let lnk_len = self.lnk.len();
        let new_size = cmp::max(hint, lnk_len * 2);
        self.lnk[lnk_len - 1].next = lnk_len;

        for i in lnk_len..new_size {
            self.lnk.push(LinkNode::new(i - 1, i + 1));
        }

        self.lnk[new_size - 1].next = 0;
    }
}

struct LinkNode {
    pub prev: usize,
    pub next: usize
}

impl LinkNode {
    fn new(p: usize, n: usize) -> LinkNode {
        LinkNode {
            prev: p,
            next: n
        }
    }
}
