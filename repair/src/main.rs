use std::io::{Read, Write};

use cozy_chess::Board;
use marlinformat::*;

const THRESHOLD: i16 = 10000;

fn rescore_game(boards: &mut [(Board, i16, u8, u8)]) {
    if let Some(&(_, last_eval, last_wdl, _)) = boards.last() {
        if last_eval <= -THRESHOLD && last_wdl == 1 {
            for (_, _, wdl, _) in boards {
                *wdl = 0;
            }
        } else if last_eval >= THRESHOLD && last_wdl == 1 {
            for (_, _, wdl, _) in boards {
                *wdl = 2;
            }
        }
    }
}

fn output_game(boards: &[(Board, i16, u8, u8)], out: &mut impl Write) {
    for (board, eval, wdl, extra) in boards {
        let packed = PackedBoard::pack(board, *eval, *wdl, *extra);
        out.write_all(&bytemuck::cast::<_, [u8; 32]>(packed))
            .unwrap();
    }
}

fn main() {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    let mut games_rescored = 0;
    let mut boards: Vec<(Board, i16, u8, u8)> = Vec::new();
    let mut buffer = [0; 32];
    while stdin.read_exact(&mut buffer).is_ok() {
        let packed = bytemuck::cast::<_, PackedBoard>(buffer);
        let (board, eval, wdl, extra) = packed.unpack().unwrap();
        if let Some((prev_board, _, _, _)) = boards.last() {
            if board.fullmove_number() < prev_board.fullmove_number() {
                rescore_game(&mut boards);
                output_game(&boards, &mut stdout);
                games_rescored += 1;
                boards.clear();

                if games_rescored % 1000 == 0 {
                    eprintln!("rescored {} games", games_rescored);
                }
            }
        }
        boards.push((board, eval, wdl, extra));
    }
    rescore_game(&mut boards);
    output_game(&boards, &mut stdout);
    games_rescored += 1;

    eprintln!("finished rescoring {} games", games_rescored);
}
