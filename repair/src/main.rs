use std::io::{Read, Write};

use cozy_chess::Board;
use marlinformat::*;
use rand::{thread_rng, Rng};

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

    let mut games_counted = 0;
    let mut draw_count: usize = 0;

    let mut evals_counted = 0;
    let mut sum_abs_eval: u128 = 0;

    while stdin.read_exact(&mut buffer).is_ok() {
        let packed = bytemuck::cast::<_, PackedBoard>(buffer);
        let (board, eval, wdl, extra) = packed.unpack().unwrap();
        if wdl == 1 {
            draw_count += 1;
        }
        if eval.abs() < 3000 {
            let rand = thread_rng().gen::<f32>();
            let include_eval = 250.0 / eval.abs() as f32;
            if rand < include_eval {
                evals_counted += 1;
                sum_abs_eval += eval.abs() as u128;
            }
        }
        games_counted += 1;
        if games_counted % 10000000 == 0 {
            eprintln!("processed {} games", games_counted);
            eprintln!(
                "draw rate {:.2}%",
                draw_count as f64 / games_counted as f64 * 100.0
            );
            eprintln!("average absolute eval {}", sum_abs_eval / evals_counted);
        }
        /*
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
        boards.push((board, eval, wdl, extra)); */
    }
    //rescore_game(&mut boards);
    //output_game(&boards, &mut stdout);
    //games_rescored += 1;

    eprintln!("finished rescoring {} games", games_rescored);
}

/*
EXPECTED:
0 pieces: 0.00%
1 pieces: 0.00%
2 pieces: 0.00%
3 pieces: 1.45%
4 pieces: 3.25%
5 pieces: 4.88%
6 pieces: 4.94%
7 pieces: 5.26%
8 pieces: 4.74%
9 pieces: 4.81%
10 pieces: 4.22%
11 pieces: 4.09%
12 pieces: 3.64%
13 pieces: 3.52%
14 pieces: 3.33%
15 pieces: 3.17%
16 pieces: 3.14%
17 pieces: 2.98%
18 pieces: 3.00%
19 pieces: 2.77%
20 pieces: 2.98%
21 pieces: 2.71%
22 pieces: 3.06%
23 pieces: 2.66%
24 pieces: 3.32%
25 pieces: 2.69%
26 pieces: 3.65%
27 pieces: 2.70%
28 pieces: 4.03%
29 pieces: 2.39%
30 pieces: 3.75%
31 pieces: 1.09%
32 pieces: 1.78%
*/

/*IBAI
0 pieces: 0.00%
1 pieces: 0.00%
2 pieces: 0.00%
3 pieces: 2.32%
4 pieces: 5.11%
5 pieces: 7.78%
6 pieces: 6.54%
7 pieces: 7.08%
8 pieces: 5.30%
9 pieces: 5.70%
10 pieces: 4.38%
11 pieces: 4.24%
12 pieces: 3.52%
13 pieces: 3.34%
14 pieces: 3.02%
15 pieces: 2.79%
16 pieces: 2.70%
17 pieces: 2.50%
18 pieces: 2.55%
19 pieces: 2.28%
20 pieces: 2.46%
21 pieces: 2.16%
22 pieces: 2.49%
23 pieces: 2.07%
24 pieces: 2.67%
25 pieces: 2.08%
26 pieces: 2.87%
27 pieces: 2.04%
28 pieces: 3.16%
29 pieces: 1.80%
30 pieces: 2.88%
31 pieces: 0.82%
32 pieces: 1.36%
*/
