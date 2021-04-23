use std::cmp;
use crate::{Utf16Char, Utf16String};


/// TAIL配列(文字列)の圧縮を行うクラス
/// TAILに格納されている文字列群の内で、末尾部分が重複するものは、同じ領域を使用するようにする。
///
/// 圧縮対象となるTAIL配列および、TAIL配列へのインデックスを渡してインスタンスを初期化する。
/// 引数に渡した各オブジェクトはshrinkメソッドの呼び出しに伴い、破壊的に更新される。
pub fn shrink(tail: Utf16String, mut begs: Vec<i32>, mut lens: Vec<Utf16Char>)
              -> (Utf16String, Vec<i32>, Vec<Utf16Char>) {
    let mut new_tail: Utf16String;
    {
        // TAILに格納されている文字列群を、その末尾から比較してソートする
        let sorted: Vec<TailString> = {
            let mut list = Vec::with_capacity(begs.len());
            for i in 0..begs.len() {
                list.push(TailString {
                    id: i,
                    s: &tail[(begs[i] as usize)..(begs[i] as usize + lens[i] as usize)]
                });
            }
            list.sort();
            list
        };

        // 新しいTAILを用意する
        // その際に、末尾部分が重複する文字列同士は、領域を共有するようにする
        new_tail = Vec::new();
        for i in 0..sorted.len() {
            let ts = &sorted[i];

            let mut beg_index = new_tail.len();
            if i > 0 && sorted[i - 1].s.ends_with(ts.s) {
                beg_index -= ts.s.len();  // 末尾文字列を共有する
            } else {
                new_tail.extend_from_slice(ts.s);       // 新しく追加する
            }

            // TAIL配列へのポインタを更新する
            begs[ts.id] = beg_index as i32;
            lens[ts.id] = ts.s.len() as Utf16Char;
        }
    }

    (new_tail, begs, lens)
}

struct TailString<'a> {
    id: usize,
    s: &'a [Utf16Char]
}

impl<'a> cmp::Ord for TailString<'a> {
    // TailStringを文字列の末尾から順に比較する
    fn cmp(&self, ts: &Self) -> cmp::Ordering {
        let mut i = self.s.len() as isize - 1;
        let mut j = ts.s.len() as isize - 1;
        loop {
            if i < 0 {
                if j < 0 {
                    return cmp::Ordering::Equal;
                } else {
                    return cmp::Ordering::Greater;
                }
            } else if j < 0 || self.s[i as usize] > ts.s[j as usize] {
                return cmp::Ordering::Less;
            } else if self.s[i as usize] < ts.s[j as usize] {
                return cmp::Ordering::Greater;
            }

            i -= 1;
            j -= 1;
        }
    }
}

impl<'a> cmp::PartialOrd for TailString<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> cmp::PartialEq for TailString<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.s.eq(other.s)
    }
}

impl<'a> cmp::Eq for TailString<'a> {}
