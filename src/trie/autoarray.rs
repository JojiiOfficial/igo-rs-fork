use std::cmp;

/// 範囲外アクセスがあった場合に自動的に拡張が行われるリスト
pub trait AutoArray<E> {
    /// 囲外アクセスがあった場合は、自動的にリストが拡張され、デフォルト値が返される。
    /// 拡張された領域にはデフォルト値が格納される。
    /// # Arguments
    /// * `index`         - リストの添字
    /// * `default_value` - リスト要素のデフォルト値
    fn get_auto(&mut self, index: usize, default_value: E) -> E;

    /// 範囲外アクセスがあった場合は、十分なサイズにまで自動的にリストが拡張される。
    /// 拡張された領域にはデフォルト値が格納される。
    /// # Arguments
    /// * `index`         - リストの添字
    /// * `element`       - 添字の位置に格納する値
    /// * `default_value` - リスト要素のデフォルト値
    fn set_auto(&mut self, index: usize, element: E, default_value: E);
}

impl<E: Copy> AutoArray<E> for Vec<E> {
    fn get_auto(&mut self, index: usize, default_value: E) -> E {
        if index < self.len() {
            self[index]
        } else {
            fill(self, index, default_value);
            default_value
        }
    }

    fn set_auto(&mut self, index: usize, element: E, default_value: E) {
        if index < self.len() {
            self[index] = element;
        } else {
            fill(self, index, default_value);
            self[index] = element;
        }
    }
}

fn fill<E: Copy>(v: &mut Vec<E>, index: usize, default_value: E) {
    let additional = cmp::max((index * 2) - v.len(), 1);
    v.reserve(additional);
    for _ in 0..additional {
        v.push(default_value);
    }
}

#[cfg(test)]
mod tests {
    use super::AutoArray;

    #[test]
    fn test_auto_array() {
        let mut a: Vec<i32> = Vec::new();
        a.set_auto(0, 100, 200);
        assert!(a.len() > 0);
        assert_eq!(a[0], 100);

        let mut a: Vec<i32> = Vec::new();
        assert_eq!(a.get_auto(0, 100), 100);
        assert!(a.len() > 0);
        assert_eq!(a[0], 100);

        let mut a: Vec<i32> = Vec::new();
        a.set_auto(3, 3, 300);
        assert_eq!(a[3], 3);
        assert_eq!(a[0], 300);
        assert_eq!(a[2], 300);
        assert!(a.len() > 3);

        assert_eq!(a.get_auto(100, 400), 400);
        assert!(a.len() > 100);
        assert_eq!(a[99], 400);
    }
}
