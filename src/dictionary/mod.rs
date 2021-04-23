mod worddic;
pub use self::worddic::*;

mod unknown;
pub use self::unknown::*;

mod charcategory;
pub use self::charcategory::*;

mod viterbinode;
pub use self::viterbinode::*;

mod matrix;
pub use self::matrix::*;

pub mod build;

pub trait Callback {
    fn call(&mut self, vn: ViterbiNode);
    fn is_empty(&self) -> bool;
}
