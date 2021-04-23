///! DoubleArrayのノード用の定数などが定義されているモジュール

/// BASEノード用の定数およびメソッドが定義されているモジュール
pub mod base {
    use std::i32;

    /// BASEノードの初期値
    pub const INIT_VALUE: i32 = i32::MIN;

    /// BASEノードに格納するID値をエンコードするためのメソッド
    /// BASEノードに格納されているID値をデコードするためにも用いられる
    #[allow(non_snake_case)]
    pub fn ID(id: i32) -> i32 {
        -id - 1
    }
}

/// CHECKノード用の定数が定義されているモジュール
pub mod chck {
    use crate::Utf16Char;

    /// 文字列の終端を表す文字定数
    /// この文字はシステムにより予約されており、辞書内の形態素の表層形および解析対象テキストに含まれていた場合の動作は未定義
    pub const TERMINATE_CODE: Utf16Char = 0u16;

    /// CHECKノードが未使用だということを示すための文字定数
    /// この文字はシステムにより予約されており、辞書内の形態素の表層形および解析対象テキストに含まれていた場合の動作は未定義
    pub const VACANT_CODE: Utf16Char = 1u16;

    /// 使用可能な文字の最大値
    pub const CODE_LIMIT: Utf16Char = 0xFFFFu16;
}
