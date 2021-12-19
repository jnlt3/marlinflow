use chess::{Board, Color, Piece};
use num_cpus;
use std::ffi::CStr;
use std::fs::File;
use std::io::{self, BufRead};
use std::os::raw::c_char;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

const INPUTS: usize = 768;

#[repr(C)]
pub struct BatchLoader {
    batch_size: usize,
    buckets: usize,
    threads: usize,
    board_buffer: Vec<Board>,
    cp_buffer: Vec<f32>,
    wdl_buffer: Vec<f32>,

    boards: Vec<[f32; INPUTS]>,
    cp: Vec<f32>,
    wdl: Vec<f32>,
    mask: Vec<f32>,
    file: Option<io::Lines<io::BufReader<File>>>,
}

impl BatchLoader {
    pub fn new(batch_size: usize, buckets: usize) -> Self {
        println!("{}", num_cpus::get());
        Self {
            batch_size,
            buckets,
            threads: num_cpus::get(),
            boards: vec![[0_f32; INPUTS]; batch_size],
            cp: vec![0_f32; batch_size * buckets],
            wdl: vec![0_f32; batch_size * buckets],
            mask: vec![0_f32; batch_size * buckets],
            file: None,
            board_buffer: vec![Board::default(); batch_size],
            cp_buffer: vec![0_f32; batch_size * buckets],
            wdl_buffer: vec![0_f32; batch_size * buckets],
        }
    }

    pub fn set_file(&mut self, path: &str) {
        self.file = Some(read_lines(path));
    }

    pub fn close_file(&mut self) {
        self.file = None;
    }

    pub fn read(&mut self) -> bool {
        if let Some(file) = &mut self.file {
            let mut counter = 0;
            for val in &mut self.mask {
                *val = 0.0;
            }
            while counter < self.batch_size {
                if let Some(Ok(line)) = file.next() {
                    let mut values = line.split(" | ");
                    let board = Board::from_str(values.next().unwrap()).unwrap();
                    let cp = values.next().unwrap().parse::<f32>().unwrap();
                    let wdl = values.next().unwrap().parse::<f32>().unwrap();
                    if cp.abs() > 3000.0 {
                        continue;
                    }
                    self.board_buffer[counter] = board;
                    self.cp_buffer[counter * self.buckets] = cp;
                    self.wdl_buffer[counter * self.buckets] = wdl;
                    counter += 1;
                } else {
                    return false;
                }
            }
            let thread_sep = self.batch_size / self.threads;
            let mut board_buffer = self.board_buffer.as_slice();
            let mut cp_buffer = self.cp_buffer.as_slice();
            let mut wdl_buffer = self.wdl_buffer.as_slice();

            let mut boards = self.boards.as_slice();
            let mut cps = self.cp.as_slice();
            let mut wdls = self.wdl.as_slice();
            let mut masks = self.mask.as_slice();

            let mut handles = vec![];
            for thread in 0..self.threads {
                let last_thread = thread == self.threads - 1;
                let split_index = if last_thread {
                    board_buffer.len()
                } else {
                    thread_sep
                };
                let (thread_boards, remaining_boards) = board_buffer.split_at(split_index);
                let (thread_cps, remaining_cps) = cp_buffer.split_at(split_index);
                let (thread_wdls, remaining_wdls) = wdl_buffer.split_at(split_index);
                board_buffer = remaining_boards;
                cp_buffer = remaining_cps;
                wdl_buffer = remaining_wdls;
                let (write_boards, remaining_boards) = boards.split_at(split_index);
                let (write_cps, remaining_cps) = cps.split_at(split_index);
                let (write_wdls, remaining_wdls) = wdls.split_at(split_index);
                let (write_masks, remaining_masks) = masks.split_at(split_index);
                boards = remaining_boards;
                wdls = remaining_wdls;
                cps = remaining_cps;
                masks = remaining_masks;
                let func = Self::fill_buffer(
                    Arc::from(thread_boards),
                    Arc::from(thread_cps),
                    Arc::from(thread_wdls),
                    Arc::from(write_boards),
                    Arc::from(write_wdls),
                    Arc::from(write_cps),
                    Arc::from(write_masks),
                    self.buckets,
                );
                if last_thread {
                    func();
                } else {
                    handles.push(std::thread::spawn(func));
                }
            }
            for handle in handles {
                handle.join().unwrap();
            }
            true
        } else {
            false
        }
    }

    fn fill_buffer(
        board_buffer: Arc<[Board]>,
        cp_buffer: Arc<[f32]>,
        wdl_buffer: Arc<[f32]>,
        boards: Arc<[[f32; INPUTS]]>,
        cps: Arc<[f32]>,
        wdls: Arc<[f32]>,
        masks: Arc<[f32]>,
        buckets: usize,
    ) -> impl Fn() {
        let board_buffer = board_buffer.clone();
        let cp_buffer = cp_buffer.clone();
        let wdl_buffer = wdl_buffer.clone();

        let boards = boards.clone();
        let cps = cps.clone();
        let wdls = wdls.clone();
        let masks = masks.clone();

        move || {
            let boards: &mut [[f32; INPUTS]] =
                unsafe { std::mem::transmute(boards.as_ptr_range()) };
            let cps: &mut [f32] = unsafe { std::mem::transmute(cps.as_ptr_range()) };
            let wdls: &mut [f32] = unsafe { std::mem::transmute(wdls.as_ptr_range()) };
            let masks: &mut [f32] = unsafe { std::mem::transmute(masks.as_ptr_range()) };
            for i in 0..board_buffer.len() {
                let board = board_buffer[i];

                let phase = phase(&board);
                let bucket = (phase * buckets / 24).min(buckets - 1);

                let (board, cp, wdl) = Self::to_input_vector(board, cp_buffer[i], wdl_buffer[i]);

                boards[i] = board;
                cps[i * buckets + bucket] = cp;
                wdls[i * buckets + bucket] = wdl;
                masks[i * buckets + bucket] = 1.0;
            }
        }
    }

    /*
    fn fill_buffer(
        fen_buffer: *const [ArrayString<U128>],
        cp_buffer: *const [f32],
        wdl_buffer: *const [f32],
        boards: *mut [[f32; INPUTS]],
        cps: *mut [f32],
        wdls: *mut [f32],
        masks: *mut [f32],
        buckets: usize,
    ) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let fen_buffer = unsafe { fen_buffer.as_ref().unwrap() };
            let cp_buffer = unsafe { cp_buffer.as_ref().unwrap() };
            let wdl_buffer = unsafe { wdl_buffer.as_ref().unwrap() };

            let boards = unsafe { boards.as_ref().unwrap() };
            let cps = unsafe { cps.as_ref().unwrap() };
            let wdls = unsafe { wdls.as_ref().unwrap() };
            let masks = unsafe { masks.as_ref().unwrap() };
            for i in 0..fen_buffer.len() {
                let board = Board::from_str(&fen_buffer[i]).unwrap();

                let phase = phase(&board);
                let bucket = (phase * buckets / 24).min(buckets - 1);

                let (board, cp, wdl) = Self::to_input_vector(board, cp_buffer[i], wdl_buffer[i]);

                boards[i] = board;
                cps[i * buckets + bucket] = cp;
                wdls[i * buckets + bucket] = wdl;
                masks[i * buckets + bucket] = 1.0;
            }
        })
    }
    */

    fn to_input_vector(board: Board, cp: f32, wdl: f32) -> ([f32; INPUTS], f32, f32) {
        let mut w_perspective = [0_f32; INPUTS as usize];

        let stm = board.side_to_move();
        let (cp, wdl) = match stm {
            Color::White => (cp, wdl),
            Color::Black => (-cp, 1.0 - wdl),
        };
        let white = *board.color_combined(Color::White);
        let black = *board.color_combined(Color::Black);

        let pawns = *board.pieces(Piece::Pawn);
        let knights = *board.pieces(Piece::Knight);
        let bishops = *board.pieces(Piece::Bishop);
        let rooks = *board.pieces(Piece::Rook);
        let queens = *board.pieces(Piece::Queen);
        let kings = *board.pieces(Piece::King);

        let array = [
            (white & pawns),
            (white & knights),
            (white & bishops),
            (white & rooks),
            (white & queens),
            (white & kings),
            (black & pawns),
            (black & knights),
            (black & bishops),
            (black & rooks),
            (black & queens),
            (black & kings),
        ];

        for (index, &pieces) in array.iter().enumerate() {
            for sq in pieces {
                let (index, sq) = match stm {
                    Color::White => (index, sq.to_index()),
                    Color::Black => (((index + 6) % 12), sq.to_index() ^ 56),
                };
                w_perspective[index * 64 + sq] = 1.0;
            }
        }
        (w_perspective, cp, wdl)
    }
}

fn read_lines<P: AsRef<Path>>(filename: P) -> io::Lines<io::BufReader<File>> {
    let file = File::open(filename).unwrap();
    io::BufReader::new(file).lines()
}

fn phase(board: &Board) -> usize {
    (board.pieces(Piece::Knight).popcnt()
        + board.pieces(Piece::Bishop).popcnt()
        + board.pieces(Piece::Rook).popcnt() * 2
        + board.pieces(Piece::Queen).popcnt() * 4)
        .min(24) as usize
}

#[no_mangle]
pub extern "C" fn new_batch_loader(batch_size: i32, buckets: i32) -> *mut BatchLoader {
    let batch_loader = Box::new(BatchLoader::new(batch_size as usize, buckets as usize));
    let batch_loader = Box::leak(batch_loader) as *mut BatchLoader;
    batch_loader
}

#[no_mangle]
pub extern "C" fn open_file(batch_loader: *mut BatchLoader, file: *const c_char) {
    let file = unsafe { CStr::from_ptr(file) }.to_str().unwrap();
    unsafe {
        batch_loader.as_mut().unwrap().set_file(file);
    }
}

#[no_mangle]
pub extern "C" fn close_file(batch_loader: *mut BatchLoader) {
    unsafe {
        batch_loader.as_mut().unwrap().close_file();
    }
}

#[no_mangle]
pub extern "C" fn read_batch(batch_loader: *mut BatchLoader) -> bool {
    unsafe { batch_loader.as_mut().unwrap().read() }
}

#[no_mangle]
pub extern "C" fn board(batch_loader: *mut BatchLoader) -> *mut [f32; 768] {
    unsafe { batch_loader.as_mut().unwrap().boards.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn cp(batch_loader: *mut BatchLoader) -> *mut f32 {
    unsafe { batch_loader.as_mut().unwrap().cp.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn wdl(batch_loader: *mut BatchLoader) -> *mut f32 {
    unsafe { batch_loader.as_mut().unwrap().wdl.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn mask(batch_loader: *mut BatchLoader) -> *mut f32 {
    unsafe { batch_loader.as_mut().unwrap().mask.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn size() -> i32 {
    std::mem::size_of::<BatchLoader>() as i32
}
