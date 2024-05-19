use cozy_chess::{BitBoard, Board, Color, File, Piece, Square};

use crate::batch::EntryFeatureWriter;

use super::InputFeatureSet;

pub fn threats(board: &Board, threats_of: Color) -> (BitBoard, BitBoard) {
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

    let mut queen_attacks = BitBoard::EMPTY;
    for queen in queens & color {
        queen_attacks |= cozy_chess::get_bishop_moves(queen, occupied)
            | cozy_chess::get_rook_moves(queen, occupied);
    }

    let king_surround = cozy_chess::get_king_moves(board.king(!threats_of));
    let threats =
        ((pawn_attacks & pieces) | (minor_attacks & majors) | (rook_attacks & queens)) & n_color;

    let offense = king_surround & minor_attacks;
    (threats, offense)
}

pub struct HalfKatoMirror;
pub struct HalfKatoMirrorCuda;

impl InputFeatureSet for HalfKatoMirror {
    const MAX_FEATURES: usize = 80;
    const INDICES_PER_FEATURE: usize = 2;

    fn add_features(board: Board, entry: EntryFeatureWriter) {
        let mut sparse_entry = entry.sparse();
        let stm = board.side_to_move();

        let stm_king = board.king(stm);
        let nstm_king = board.king(!stm);

        for &color in &Color::ALL {
            let (threats, offense) = threats(&board, !color);
            for &piece in &Piece::ALL {
                for square in board.pieces(piece) & board.colors(color) {
                    let stm_feature = feature(stm, stm_king, color, piece, square);
                    let nstm_feature = feature(!stm, nstm_king, color, piece, square);
                    sparse_entry.add_feature(stm_feature as i64, nstm_feature as i64);
                }
            }
            for square in threats {
                let stm_feature = extra_feature(stm, stm_king, color, square, 0);
                let nstm_feature = extra_feature(!stm, nstm_king, color, square, 0);
                sparse_entry.add_feature(stm_feature as i64, nstm_feature as i64);
            }
            for square in offense {
                let stm_feature = extra_feature(stm, stm_king, color, square, 1);
                let nstm_feature = extra_feature(!stm, nstm_king, color, square, 1);
                sparse_entry.add_feature(stm_feature as i64, nstm_feature as i64);
            }
        }
    }
}

impl InputFeatureSet for HalfKatoMirrorCuda {
    const MAX_FEATURES: usize = 80;
    const INDICES_PER_FEATURE: usize = 1;

    fn add_features(board: Board, entry: EntryFeatureWriter) {
        let mut cuda_entry = entry.cuda();
        let stm = board.side_to_move();

        let stm_king = board.king(stm);
        let nstm_king = board.king(!stm);

        for &color in &Color::ALL {
            let (threats, offense) = threats(&board, !color);
            for &piece in &Piece::ALL {
                for square in board.pieces(piece) & board.colors(color) {
                    let stm_feature = feature(stm, stm_king, color, piece, square);
                    let nstm_feature = feature(!stm, nstm_king, color, piece, square);
                    cuda_entry.add_feature(stm_feature as i64, nstm_feature as i64);
                }
            }
            for square in threats {
                let stm_feature = extra_feature(stm, stm_king, color, square, 0);
                let nstm_feature = extra_feature(!stm, nstm_king, color, square, 0);
                cuda_entry.add_feature(stm_feature as i64, nstm_feature as i64);
            }
            for square in offense {
                let stm_feature = extra_feature(stm, stm_king, color, square, 1);
                let nstm_feature = extra_feature(!stm, nstm_king, color, square, 1);
                cuda_entry.add_feature(stm_feature as i64, nstm_feature as i64);
            }
        }
    }
}

fn king_square_to_index(sq: Square) -> usize {
    sq.file() as usize * 8 + sq.rank() as usize
}

fn feature(perspective: Color, king: Square, color: Color, piece: Piece, square: Square) -> usize {
    let flip_file = (king.file() as usize) > File::D as usize;
    let (mut king, mut square, color) = match perspective {
        Color::White => (king, square, color),
        Color::Black => (king.flip_rank(), square.flip_rank(), !color),
    };
    if flip_file {
        king = king.flip_file();
        square = square.flip_file();
    }
    let mut index = 0;
    index = index * Square::NUM / 2 + king_square_to_index(king);
    index = index * Color::NUM + color as usize;
    index = index * (Piece::NUM + 2) + piece as usize;
    index = index * Square::NUM + square as usize;
    index
}

fn extra_feature(
    perspective: Color,
    king: Square,
    color: Color,
    square: Square,
    extra: usize,
) -> usize {
    let flip_file = (king.file() as usize) > File::D as usize;
    let (mut king, mut square, color) = match perspective {
        Color::White => (king, square, color),
        Color::Black => (king.flip_rank(), square.flip_rank(), !color),
    };
    if flip_file {
        king = king.flip_file();
        square = square.flip_file();
    }
    let mut index = 0;
    index = index * Square::NUM / 2 + king_square_to_index(king);
    index = index * Color::NUM + color as usize;
    index = index * (Piece::NUM + 2) + Piece::NUM + extra;
    index = index * Square::NUM + square as usize;
    index
}
