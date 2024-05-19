use cozy_chess::{BitBoard, Board, Color, File, Piece, Square};

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

pub struct HalfKaxtMirror;
pub struct HalfKaxtMirrorCuda;

impl InputFeatureSet for HalfKaxtMirror {
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
                    let stm_feature =
                        feature(stm, stm_king, color, piece, square, threats.has(square));
                    let nstm_feature =
                        feature(!stm, nstm_king, color, piece, square, threats.has(square));
                    sparse_entry.add_feature(stm_feature as i64, nstm_feature as i64);
                }
            }
        }
    }
}

impl InputFeatureSet for HalfKaxtMirrorCuda {
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
                    let stm_feature = feature(stm, stm_king, color, piece, square, false);
                    let nstm_feature = feature(!stm, nstm_king, color, piece, square, false);
                    cuda_entry.add_feature(stm_feature as i64, nstm_feature as i64);
                    if threats.has(square) {
                        let stm_feature = feature(stm, stm_king, color, piece, square, true);
                        let nstm_feature = feature(!stm, nstm_king, color, piece, square, true);
                        cuda_entry.add_feature(stm_feature as i64, nstm_feature as i64);
                    }
                }
            }
        }
    }
}

fn king_square_to_index(sq: Square) -> usize {
    sq.file() as usize * 8 + sq.rank() as usize
}

fn feature(
    perspective: Color,
    king: Square,
    color: Color,
    piece: Piece,
    square: Square,
    as_threat: bool,
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
    let piece_idx = if as_threat {
        match piece {
            Piece::Knight => Piece::NUM,
            Piece::Bishop => Piece::NUM + 1,
            Piece::Rook => Piece::NUM + 2,
            Piece::Queen => Piece::NUM + 3,
            _ => unreachable!(),
        }
    } else {
        piece as usize
    };
    let mut index = 0;
    index = index * Square::NUM / 2 + king_square_to_index(king);
    index = index * Color::NUM + color as usize;
    index = index * (Piece::NUM + 4) + piece_idx;
    index = index * Square::NUM + square as usize;
    index
}
