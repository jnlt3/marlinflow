use cozy_chess::Board;

use crate::batch::EntryFeatureWriter;

mod board_768;
mod half_ka;
mod half_ka_t;
mod half_kat_mirror;
mod half_kato_mirror;
mod half_kaxt_mirror;
mod half_kp;

pub use board_768::Board768;
pub use board_768::Board768Cuda;
pub use half_ka::HalfKa;
pub use half_ka::HalfKaCuda;
pub use half_ka_t::HalfKat;
pub use half_ka_t::HalfKatCuda;
pub use half_kat_mirror::HalfKatMirror;
pub use half_kat_mirror::HalfKatMirrorCuda;
pub use half_kato_mirror::HalfKatoMirror;
pub use half_kato_mirror::HalfKatoMirrorCuda;
pub use half_kaxt_mirror::HalfKaxtMirror;
pub use half_kaxt_mirror::HalfKaxtMirrorCuda;
pub use half_kp::HalfKp;
pub use half_kp::HalfKpCuda;

pub trait InputFeatureSet {
    const INDICES_PER_FEATURE: usize;
    const MAX_FEATURES: usize;

    fn add_features(board: Board, entry: EntryFeatureWriter);
}
