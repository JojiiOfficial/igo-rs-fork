use std::env;
use std::path::PathBuf;

use igo::Tagger;

fn main() {
    let dic_dir = PathBuf::from(env::var("IGO_DIC").unwrap_or("data/ipadic".to_string()));
    let tagger = Tagger::new(dic_dir.as_path()).unwrap();
    let text = "すもももももも\u{1F351}もものうち";

    let results = tagger.parse(text);
    for ref m in &results {
        println!("{}\t{}", m.surface, m.feature);
    }
    println!("EOS");

    for ref m in results {
        println!("{:?}", m);
    }
}
