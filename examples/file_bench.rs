use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::path::PathBuf;
use std::time::Instant;
use std::env;

use igo::Tagger;

fn setup_tagger() -> Tagger {
    let dic_dir = PathBuf::from("data/ipadic");
    Tagger::new(&dic_dir).unwrap()
}

fn main() {
    let repeat: usize = env::args().nth(1)
        .and_then(|s| s.parse().ok()).unwrap_or(1000);
    let input_path = env::args().nth(2)
        .unwrap_or("data/text1.txt".to_string());

    println!("input file path: {}", input_path);
    let reader = BufReader::new(File::open(input_path).unwrap());
    let lines: Vec<_> = reader.lines().map(|l| l.unwrap()).collect();

    let tagger = setup_tagger();
    let start_time = Instant::now();

    println!("start {} iter", repeat);

    for _ in 0..repeat {
        for line in &lines {
            tagger.parse(line);
        }
    }

    let elapsed = start_time.elapsed();
    let ms = (((elapsed.as_secs() as f64) * 1000.0)
        + ((elapsed.subsec_nanos() as f64) / 1_000_000.0)) / (repeat as f64);
    println!("elapsed: {} ms/iter", ms);
}
