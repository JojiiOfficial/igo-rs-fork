use std::rc::Rc;


/// Viterbiアルゴリズムで使用されるノード
#[derive(Debug)]
pub struct ViterbiNode {
    /// 始点からノードまでの総コスト
    pub cost: i32,
    /// コスト最小の前方のノードへのリンク
    pub prev: Option<Rc<ViterbiNode>>,

    /// 単語ID
    pub word_id: i32,
    /// 左文脈ID
    pub left_id: i16,
    /// 右文脈ID
    pub right_id: i16,
    /// 入力テキスト内での形態素の開始位置
    pub start: usize,
    /// 形態素の表層形の長さ(文字数)
    pub length: i16,

    /// 形態素の文字種(文字カテゴリ)が空白文字かどうか
    pub is_space: bool
}

impl ViterbiNode {
    pub fn make_boseos() -> ViterbiNode {
        ViterbiNode {
            word_id: 0,
            start: 0,
            length: 0,
            cost: 0,
            left_id: 0,
            right_id: 0,
            is_space: false,
            prev: None
        }
    }
}