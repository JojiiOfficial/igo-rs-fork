# igo-rs

[![Build status](https://ci.appveyor.com/api/projects/status/b5y8yg8mkrxy4u80?svg=true)](https://ci.appveyor.com/project/yasuhara/igo-rs)

Pure Rust port of the Igo, a POS(Part-Of-Speech) tagger for Japanese.

The [original version](http://igo.osdn.jp/) was written in Java.

WebAssembly ready as of version 0.3 ([Online Demo](https://yshryk.github.io/igo-wasm-demo/)).

*Japanese follows the English*

## Requirement
* Binary dictionary files built with [original version](http://igo.osdn.jp/)
  (You can download prebuilt dictionary from [here](https://bitbucket.org/yshryk/igo-rs/downloads))

## Demo (as a CLI tool)

```shell
% cp -r somewhere/original_java_igo/dic/ipadic data
% cargo build --release
% ./target/release/igo -t "すもももももも🍑もものうち" data/ipadic

すもも	名詞,一般,*,*,*,*,すもも,スモモ,スモモ
も	助詞,係助詞,*,*,*,*,も,モ,モ
もも	名詞,一般,*,*,*,*,もも,モモ,モモ
も	助詞,係助詞,*,*,*,*,も,モ,モ
🍑	記号,一般,*,*,*,*,*
もも	名詞,一般,*,*,*,*,もも,モモ,モモ
の	助詞,連体化,*,*,*,*,の,ノ,ノ
うち	名詞,非自立,副詞可能,*,*,*,うち,ウチ,ウチ
EOS
```

## Usage (as a library)

```rust
extern crate igo;

use std::path::PathBuf;
use igo::Tagger;

fn main() {
    let dic_dir = PathBuf::from("data/ipadic");
    let tagger = Tagger::new(&dic_dir).unwrap();
    let text = "すもももももももものうち";

    let results = tagger.parse(text);
    for ref m in results {
        println!("{}\t{}", m.surface, m.feature);
    }
    println!("EOS");
}
```

## Building binary dictionary

```shell
% cargo build --release
% ./target/release/igo_build_dic data/ipadic data/mecab-ipadic-2.7.0-20070801-utf8 UTF-8

### Build word trie
### Build word dictionary
### Build matrix
### Build char-category dictionary
DONE
```

## License

The MIT License.

## 概要
Takeru Ohta氏がJavaで書いた日本語形態素解析ライブラリ Igo をRustに移植したものです。

## 特徴
* pure Rustなので、Windows等でも実行環境を整えやすい。
* unsafe な処理を使用していない。
* WebAssembly にコンパイル可能 ([オンラインデモ](https://yshryk.github.io/igo-wasm-demo/)).

## 必要なもの
* Java版Igoで構築したバイナリ辞書（ディレクトリごと）。
  ([ここ](https://bitbucket.org/yshryk/igo-rs/downloads) から構築済のIPA辞書をダウンロード出来ます)
* このRust版Igoでもバイナリ辞書を構築出来るようにしました。上の Building binary dictionary を見てください。

## 使い方
Rust用ライブラリとして、またはコマンドラインプログラムとして使用可能です。
上の Demo と Usage を見てください。

### 注意事項
* 高速化の為、処理結果 `igo::Morpheme` は 処理対象テキストと `igo::Tagger` 
への参照を保持しています。 `surface` は処理対象テキストのスライスであり、
 `feature` は辞書内テキストのスライスです。
 必要に応じて `Morpheme#to_owned()` 等の処理をしてください。
* バイナリ辞書を構築する場合、文字エンコーディングはUTF-8を推奨します。

### メモリ使用量
`examples/parse.rs` を64-bit Linux環境で `valgrind --tool=massif` を使って調べたところ、
最大で102.6MBでした。

### rubyバインディング
[igo-rs-ruby](https://bitbucket.org/yshryk/igo-rs-ruby) -- rubyの拡張ライブラリとして呼び出せるようにするテスト
