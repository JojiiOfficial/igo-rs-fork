use std;
use std::borrow::Cow;
use std::convert::From;
use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use glob;
use log::info;

pub use self::charcategory::*;
pub use self::worddic::*;

mod charcategory;
pub mod matrix;
mod worddic;

#[derive(Debug)]
pub enum AppError {
    Message(String),
    Io(io::Error),
    Parse {
        message: String,
        path: PathBuf,
        line_number: i32,
    },
}

pub type AppResult<T> = Result<T, AppError>;

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppError::Message(ref err) => write!(f, "error: {}", err),
            AppError::Io(ref err) => write!(f, "IO error: {}", err),
            AppError::Parse {
                ref message,
                ref path,
                ref line_number,
            } => write!(
                f,
                "{}\t{{file: {}, line: {}}}",
                message,
                path.display(),
                line_number
            ),
        }
    }
}

impl error::Error for AppError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            AppError::Message(_) | AppError::Parse { .. } => None,
            AppError::Io(ref err) => Some(err),
        }
    }
}

impl From<String> for AppError {
    fn from(e: String) -> Self {
        AppError::Message(e)
    }
}

impl<'a> From<&'a str> for AppError {
    fn from(e: &str) -> Self {
        AppError::Message(e.to_string())
    }
}

impl From<Cow<'static, str>> for AppError {
    fn from(e: Cow<'static, str>) -> Self {
        AppError::Message(e.to_string())
    }
}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<glob::PatternError> for AppError {
    fn from(e: glob::PatternError) -> Self {
        AppError::Message(e.to_string())
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> Self {
        AppError::Message(e.to_string())
    }
}

pub fn build_dic(
    input_dir: &Path,
    output_dir: &Path,
    delimiter: String,
    encoding: &str,
) -> AppResult<i32> {
    info!(
        "output_dir: {}, input_dir: {}, delimiter: {:?}, encoding: {}",
        output_dir.display(),
        input_dir.display(),
        delimiter,
        encoding
    );
    fs::create_dir_all(&output_dir).expect("couldn't create directory");

    let start_time = Instant::now();
    let mut wd = WordDic::new(input_dir, encoding, output_dir, delimiter);
    let cc = CharCategory::new(input_dir, encoding, output_dir);
    println!("### Build word trie");
    wd.build_word_id_map()?;

    println!("### Build word dictionary");
    wd.build_word_info()?;

    println!("### Build matrix");
    matrix::build(input_dir, output_dir)?;

    println!("### Build char-category dictionary");
    cc.build()?;

    let elapsed = start_time.elapsed();
    let ms =
        ((elapsed.as_secs() as f64) * 1000.0) + ((elapsed.subsec_nanos() as f64) / 1_000_000.0);
    println!("DONE");
    println!("elapsed: {} ms", ms);
    Ok(0)
}
