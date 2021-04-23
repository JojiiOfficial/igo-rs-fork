/// 形態素
#[derive(Debug)]
pub struct Morpheme<'tagger, 'text> {
    /// 形態素の表層形
    pub surface: &'text str,

    /// 形態素の素性
    pub feature: &'tagger str,

    /// テキスト内での形態素の出現開始位置
    pub start: usize
}

impl<'tagger, 'text> Morpheme<'tagger, 'text> {
    pub fn to_owned(&self) -> MorphemeBuf {
        MorphemeBuf {
            surface: self.surface.to_owned(),
            feature: self.feature.to_owned(),
            start: self.start
        }
    }
}

/// 形態素
#[derive(Debug)]
pub struct MorphemeBuf {
    /// 形態素の表層形
    pub surface: String,

    /// 形態素の素性
    pub feature: String,

    /// テキスト内での形態素の出現開始位置
    pub start: usize
}
