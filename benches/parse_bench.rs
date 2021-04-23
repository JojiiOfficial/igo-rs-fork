#![cfg_attr(feature = "unstable", feature(test))]

#[cfg(all(feature = "unstable", test))]
mod bench {
    extern crate test;

    use test::Bencher;
    //use test::black_box;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::BufRead;
    use std::path::PathBuf;
    use igo::Tagger;


    fn setup_tagger() -> Tagger {
        let dic_dir = PathBuf::from("data/ipadic");
        Tagger::new(&dic_dir).unwrap()
    }

    //#[bench]
    #[allow(dead_code)]
    fn bench_tagger_init(b: &mut Bencher) {
        b.iter(|| {
            setup_tagger();
        });
    }

    #[bench]
    fn bench_parse(b: &mut Bencher) {
        let reader = BufReader::new(File::open("data/text1.txt").unwrap());
        let lines: Vec<_> = reader.lines().map(|l| l.unwrap()).collect();

        let tagger = setup_tagger();
        b.iter(|| {
            for line in &lines {
                tagger.parse(line);
            }
        });
    }
}
