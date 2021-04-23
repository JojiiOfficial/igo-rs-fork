use std::env;
use std::path::PathBuf;

use getopts::Options;

use igo::dictionary::build::*;

fn print_usage(program: &str, opts: Options) {
    println!("{}", opts.usage(&format!(
        "Usage:\n {} [options] <output directory> <input directory> <encoding>", program)));
}

fn build_dic_cli() -> AppResult<i32> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("d", "delimiter", "set delimiter to STR.", "STR");
    opts.optflag("v", "verbose", "enable verbose mode.");
    opts.optflag("", "help", "show this usage message.");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(_) => {
            print_usage(&program, opts);
            return Ok(1);
        }
    };
    if matches.opt_present("help") {
        print_usage(&program, opts);
        return Ok(1);
    }
    if matches.opt_present("verbose") {
        simple_logger::init().unwrap();
    }
    let delimiter = matches.opt_str("d").unwrap_or_else(|| ",".to_string());
    let mut args = matches.free.into_iter();
    let output_dir = if let Some(s) = args.next() {
        PathBuf::from(s)
    } else {
        print_usage(&program, opts);
        return Ok(1);
    };
    let input_dir = if let Some(s) = args.next() {
        PathBuf::from(s)
    } else {
        print_usage(&program, opts);
        return Ok(1);
    };
    let encoding = if let Some(s) = args.next() {
        s
    } else {
        print_usage(&program, opts);
        return Ok(1);
    };

    build_dic(&input_dir, &output_dir, delimiter, &encoding)
}

fn main() {
    match build_dic_cli() {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(error) => eprintln!("Error: {}", error)
    }
}
