use std::io::{self, BufReader};

use crate::util::*;

/// 形態素の連接コスト表を扱う
#[derive(Clone)]
pub struct Matrix {
    left_size: i32,
    #[allow(dead_code)]
    right_size: i32,
    matrix: Box<[i16]>,
}

impl Matrix {
    pub fn new(dir: &mut dyn DirLike) -> io::Result<Matrix> {
        let mut reader = BufReader::new(dir.open("matrix.bin")?);
        let left_size = reader.get_int()?;
        let right_size = reader.get_int()?;

        Ok(Matrix {
            left_size,
            right_size,
            matrix: reader.get_short_array((left_size * right_size) as usize)?,
        })
    }

    /// 形態素同士の連接コストを求める
    pub fn link_cost(&self, left_id: i16, right_id: i16) -> i32 {
        i32::from(self.matrix[(right_id as usize) * (self.left_size as usize) + (left_id as usize)])
    }
}
