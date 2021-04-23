use std::path::Path;
use std::io::BufWriter;
use std::io::Write;
use std::fs::File;
use byteorder::{WriteBytesExt, NativeEndian as NE};
use crate::util::*;
use crate::dictionary::build::*;
use log::debug;


/// 形態素の連接コスト表のバイナリデータを作成する
/// # Arguments
/// * `input_dir`  - ソース辞書があるディレクトリ。input_dir+"/matrix.def" ファイルが使用される
/// * `output_dir` - バイナリデータが保存されるディレクトリ。output_dir+"/matrix.bin" ファイルが作成される
pub fn build(input_dir: &Path, output_dir: &Path) -> AppResult<()> {
    let input_file = input_dir.join("matrix.def");
    debug!("input_file: {}", input_file.display());
    let mut rl = ReadLine::new(input_file.as_path(), "UTF-8")?;

    // 一行目はサイズ: [左文脈IDの数] [右文脈IDの数]
    let mut s = String::new();
    rl.next(&mut s)?;
    let mut fields = s.split_whitespace();
    let left_num: i32 = fields.next().and_then(|s| s.parse().ok())
        .ok_or_else(|| rl.parse_error("Parse error"))?;
    let right_num: i32 = fields.next().and_then(|s| s.parse().ok())
        .ok_or_else(|| rl.parse_error("Parse error"))?;
    let mut writer = BufWriter::new(File::create(output_dir.join("matrix.bin").as_path())?);

    writer.write_i32::<NE>(left_num)?;
    writer.write_i32::<NE>(right_num)?;

    // 二行目以降はデータ: [左文脈ID] [右文脈ID] [連接コスト]
    let mut tmp_matrix = vec![0i16; (left_num * right_num) as usize];
    let mut s = String::new();
    for i in 0..left_num {
        for j in 0..right_num {
            rl.next(&mut s)?;
            let mut fields = s.split_whitespace();
            let left_id: i32 = fields.next().and_then(|s| s.parse().ok())
                .ok_or_else(|| rl.parse_error("Parse integer failed."))?;
            let right_id: i32 = fields.next().and_then(|s| s.parse().ok())
                .ok_or_else(|| rl.parse_error("Parse integer failed."))?;
            let cost: i16 = fields.next().and_then(|s| s.parse().ok())
                .ok_or_else(|| rl.parse_error("Parse short integer failed."))?;

            if i != left_id {
                return Err(rl.parse_error(format!("Unexpected left context ID. ID={}, expected={}", left_id, i)));
            }
            if j != right_id {
                return Err(rl.parse_error(format!("Unexpected right context ID. ID={}, expected={}", right_id, j)));
            }

            // NOTE: tmp_matrixという一時配列を用いている理由
            //
            // この段階で、writer.write_i16(cost)、などとしてファイルに書き出した場合、
            // matrix[left_id][right_id]=cost、といった配列ができる。
            //
            // それでも特に問題はないのだが、今回のケースでは、
            // 「right_idが固定で、left_idのみが変動する」といったようなメモリアクセスパターンが多い。
            //
            // そのためtmp_matrix配列を用いて、コスト値の並び順を変更し、
            // matrix[right_id][left_id]とったように、right_idが第一添字になるようにした方が
            // メモリアクセスの局所性が高まり(多分)、若干だが処理速度が向上する。
            tmp_matrix[(j * left_num + i) as usize] = cost;
        }
    }
    for cost in tmp_matrix {
        writer.write_i16::<NE>(cost)?;
    }

    Ok(writer.flush()?)
}

// 添付用に簡略化したmatrix.defを元の書式に復元する
pub fn convert_sparse2dense(input_file: &Path, output_file: &Path, default_cost: i16) -> AppResult<()> {
    let mut rl = ReadLine::new(input_file, "UTF-8")?;

    // 一行目はサイズ: [左文脈IDの数] [右文脈IDの数]
    let mut s = String::new();
    rl.next(&mut s)?;
    let mut fields = s.split_whitespace();
    let left_num: i32 = fields.next().and_then(|s| s.parse().ok())
        .ok_or_else(|| rl.parse_error("Parse error"))?;
    let right_num: i32 = fields.next().and_then(|s| s.parse().ok())
        .ok_or_else(|| rl.parse_error("Parse error"))?;

    let mut writer = BufWriter::new(File::create(output_file)?);
    write!(writer, "{} {}\n", left_num, right_num)?;

    // 二行目以降はデータ: [左文脈ID] [右文脈ID] [連接コスト]
    let mut s = String::new();
    let mut i = 0i32;
    let mut j = 0i32;
    loop {
        rl.next(&mut s)?;
        let s = s.trim();
        if s.is_empty() {
            break;
        }

        let mut fields = s.split_whitespace();
        let left_id: i32 = fields.next().and_then(|s| s.parse().ok())
            .ok_or_else(|| rl.parse_error("Parse integer failed."))?;
        let right_id: i32 = fields.next().and_then(|s| s.parse().ok())
            .ok_or_else(|| rl.parse_error("Parse integer failed."))?;
        let cost: i16 = fields.next().and_then(|s| s.parse().ok())
            .ok_or_else(|| rl.parse_error("Parse short integer failed."))?;

        while i < left_id || j < right_id {
            write!(writer, "{} {} {}\n", i, j, default_cost)?;
            if j < (right_num - 1) {
                j += 1;
            } else {
                j = 0;
                i += 1;
            }
        }

        write!(writer, "{} {} {}\n", left_id, right_id, cost)?;
        if j < (right_num - 1) {
            j += 1;
        } else {
            j = 0;
            i += 1;
        }
    }

    while i < left_num && j < right_num {
        write!(writer, "{} {} {}\n", i, j, default_cost)?;
        if j < (right_num - 1) {
            j += 1;
        } else {
            j = 0;
            i += 1;
        }
    }

    Ok(writer.flush()?)
}
