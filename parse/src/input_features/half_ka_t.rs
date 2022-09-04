use cozy_chess::{BitBoard, Board, Color, Piece, Square};

use crate::batch::EntryFeatureWriter;

use super::InputFeatureSet;

pub fn threats(board: &Board, threats_of: Color) -> BitBoard {
    let occupied = board.occupied();
    let color = board.colors(threats_of);
    let n_color = board.colors(!threats_of);

    let pawns = board.pieces(Piece::Pawn);
    let knights = board.pieces(Piece::Knight);
    let bishops = board.pieces(Piece::Bishop);
    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Queen);

    let minors = knights | bishops;
    let majors = rooks | queens;
    let pieces = minors | majors;

    let mut pawn_attacks = BitBoard::EMPTY;
    for pawn in pawns & color {
        pawn_attacks |= cozy_chess::get_pawn_attacks(pawn, threats_of);
    }

    let mut minor_attacks = BitBoard::EMPTY;
    for knight in knights & color {
        minor_attacks |= cozy_chess::get_knight_moves(knight);
    }

    for bishop in bishops & color {
        minor_attacks |= cozy_chess::get_bishop_moves(bishop, occupied);
    }

    let mut rook_attacks = BitBoard::EMPTY;
    for rook in rooks & color {
        rook_attacks |= cozy_chess::get_rook_moves(rook, occupied);
    }

    ((pawn_attacks & pieces) | (minor_attacks & majors) | (rook_attacks & queens)) & n_color
}

pub struct HalfKaT;
pub struct HalfKaTCuda;

impl InputFeatureSet for HalfKaT {
    const MAX_FEATURES: usize = 64;
    const INDICES_PER_FEATURE: usize = 2;

    fn add_features(board: Board, entry: EntryFeatureWriter) {
        let mut sparse_entry = entry.sparse();
        let stm = board.side_to_move();

        let stm_king = board.king(stm);
        let nstm_king = board.king(!stm);

        for &color in &Color::ALL {
            let threats = threats(&board, !color);
            for &piece in &Piece::ALL {
                for square in board.pieces(piece) & board.colors(color) {
                    let stm_feature = feature(stm, stm_king, color, piece, square);
                    let nstm_feature = feature(!stm, nstm_king, color, piece, square);
                    sparse_entry.add_feature(stm_feature as i64, nstm_feature as i64);
                }
            }
            for square in threats {
                let stm_feature = threat_feature(stm, stm_king, color, square);
                let nstm_feature = threat_feature(!stm, nstm_king, color, square);
                sparse_entry.add_feature(stm_feature as i64, nstm_feature as i64);
            }
        }
    }
}

impl InputFeatureSet for HalfKaTCuda {
    const MAX_FEATURES: usize = 64;
    const INDICES_PER_FEATURE: usize = 1;

    fn add_features(board: Board, entry: EntryFeatureWriter) {
        let mut cuda_entry = entry.cuda();
        let stm = board.side_to_move();

        let stm_king = board.king(stm);
        let nstm_king = board.king(!stm);

        for &color in &Color::ALL {
            let threats = threats(&board, !color);
            for &piece in &Piece::ALL {
                for square in board.pieces(piece) & board.colors(color) {
                    let stm_feature = feature(stm, stm_king, color, piece, square);
                    let nstm_feature = feature(!stm, nstm_king, color, piece, square);
                    cuda_entry.add_feature(stm_feature as i64, nstm_feature as i64);
                }
            }
            for square in threats {
                let stm_feature = threat_feature(stm, stm_king, color, square);
                let nstm_feature = threat_feature(!stm, nstm_king, color, square);
                cuda_entry.add_feature(stm_feature as i64, nstm_feature as i64);
            }
        }
    }
}

fn feature(perspective: Color, king: Square, color: Color, piece: Piece, square: Square) -> usize {
    let (king, square, color) = match perspective {
        Color::White => (king, square, color),
        Color::Black => (king.flip_rank(), square.flip_rank(), !color),
    };
    let mut index = 0;
    index = index * Square::NUM + king as usize;
    index = index * Color::NUM + color as usize;
    index = index * (Piece::NUM + 1) + piece as usize;
    index = index * Square::NUM + square as usize;
    index
}

fn threat_feature(perspective: Color, king: Square, color: Color, square: Square) -> usize {
    let (king, square, color) = match perspective {
        Color::White => (king, square, color),
        Color::Black => (king.flip_rank(), square.flip_rank(), !color),
    };
    let mut index = 0;
    index = index * Square::NUM + king as usize;
    index = index * Color::NUM + color as usize;
    index = index * (Piece::NUM + 1) + Piece::NUM;
    index = index * Square::NUM + square as usize;
    index
}
