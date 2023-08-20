use cozy_chess::Board;

use crate::batch::EntryFeatureWriter;

mod board_768;
mod half_ka;
mod half_ka_t;
pub mod half_kat_mirror;
mod half_kp;

pub use board_768::Board768;
pub use board_768::Board768Cuda;
pub use half_ka::HalfKa;
pub use half_ka::HalfKaCuda;
pub use half_ka_t::HalfKaT;
pub use half_ka_t::HalfKaTCuda;
pub use half_kat_mirror::HalfKaTMirror;
pub use half_kat_mirror::HalfKaTMirrorCuda;
pub use half_kp::HalfKp;
pub use half_kp::HalfKpCuda;

pub trait InputFeatureSet {
    const INDICES_PER_FEATURE: usize;
    const MAX_FEATURES: usize;

    fn add_features(board: Board, entry: EntryFeatureWriter);
}
