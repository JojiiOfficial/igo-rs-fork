mod tagger;
mod trie;
mod util;
pub use tagger::Tagger;
mod morpheme;
pub use morpheme::{Morpheme, MorphemeBuf};
pub mod dictionary;
pub use util::DirLike;

type Utf16Char = u16;
type Utf16String = Vec<Utf16Char>;
type Utf16Str = [Utf16Char];

#[cfg(test)]
mod tests {
    use crate::dictionary::build;
    use crate::morpheme::MorphemeBuf;
    use crate::tagger::Tagger;
    use std::path::{Path, PathBuf};

    const TEST_DIC_SRC_PATH: &str = "tiny_test_dic/src";
    const TEST_DIC_DST_PATH: &str = "tiny_test_dic/out";

    fn setup_tagger() -> Tagger {
        let dic_dir = PathBuf::from(TEST_DIC_DST_PATH);
        Tagger::new(&dic_dir).unwrap()
    }

    #[test]
    fn build_dic_then_test_tagger() {
        build_dic();

        test_tagger();
        test_wakati();
    }

    fn build_dic() {
        build_matrix_def_file();

        build::build_dic(
            &Path::new(TEST_DIC_SRC_PATH),
            &Path::new(TEST_DIC_DST_PATH),
            ",".to_string(),
            "UTF-8",
        )
        .unwrap();
    }

    fn build_matrix_def_file() {
        let src_dir = Path::new(TEST_DIC_SRC_PATH);
        let sparse_matrix = src_dir.join("matrix.def.sparse");
        let dense_matrix = src_dir.join("matrix.def");
        let default_cost = ::std::i16::MAX;
        build::matrix::convert_sparse2dense(&sparse_matrix, &dense_matrix, default_cost).unwrap();

        if !dense_matrix.exists() {
            panic!("failed to convert: {}", dense_matrix.display());
        }
    }

    fn test_tagger() {
        let tagger = setup_tagger();
        assert_eq!(9, tagger.unknown().space_id);

        let text = "すもももももももものうち";
        let results = tagger.parse(text);
        assert_eq!(7, results.len());

        assert_eq!("すもも", results[0].surface);
        assert_eq!("も", results[1].surface);
        assert_eq!("もも", results[2].surface);
        assert_eq!("も", results[3].surface);
        assert_eq!("もも", results[4].surface);
        assert_eq!("の", results[5].surface);
        assert_eq!("うち", results[6].surface);

        assert_eq!("名詞,一般,*,*,*,*,すもも,スモモ,スモモ", results[0].feature);
        assert_eq!("助詞,係助詞,*,*,*,*,も,モ,モ", results[1].feature);
        assert_eq!("名詞,一般,*,*,*,*,もも,モモ,モモ", results[2].feature);
        assert_eq!("助詞,係助詞,*,*,*,*,も,モ,モ", results[3].feature);
        assert_eq!("名詞,一般,*,*,*,*,もも,モモ,モモ", results[4].feature);
        assert_eq!("助詞,連体化,*,*,*,*,の,ノ,ノ", results[5].feature);
        assert_eq!(
            "名詞,非自立,副詞可能,*,*,*,うち,ウチ,ウチ",
            results[6].feature
        );

        assert_eq!(0, results[0].start);
        assert_eq!(3, results[1].start);
        assert_eq!(4, results[2].start);
        assert_eq!(6, results[3].start);
        assert_eq!(7, results[4].start);
        assert_eq!(9, results[5].start);
        assert_eq!(10, results[6].start);

        // MorphemeBuf test
        let buf: MorphemeBuf = results[4].to_owned();
        assert_eq!("もも", buf.surface);
        assert_eq!("名詞,一般,*,*,*,*,もも,モモ,モモ", buf.feature);
        assert_eq!(7, buf.start);
    }

    fn test_wakati() {
        let tagger = setup_tagger();

        let text = "すもももももももものうち";
        let results = tagger.wakati(text);
        assert_eq!(7, results.len());

        let v = vec!["すもも", "も", "もも", "も", "もも", "の", "うち"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        assert_eq!(v, results);
    }
}
