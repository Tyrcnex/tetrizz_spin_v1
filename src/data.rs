use rand::prelude::IndexedRandom;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Piece {
    I, O, T, L, J, S, Z
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum Rotation {
    Up, Right, Down, Left
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct PieceLocation {
    pub piece: Piece,
    pub rotation: Rotation,
    pub spun: bool,
    pub x: i8,
    pub y: i8,
    pub possible_line_clear: bool
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Board {
    pub cols: [u64; 10] 
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    pub board: Board,
    pub hold: Piece,
    pub b2b: u64,
    pub b2b_deficit: u32
}

#[derive(Debug)]
pub struct PlacementInfo {
    pub spin: bool,
    pub lines_cleared: u32
}

impl Rotation {
    pub const fn rotate_block(&self, (x,y): (i8, i8)) -> (i8, i8) {
        match self {
            Rotation::Up => (x, y),
            Rotation::Right => (y, -x),
            Rotation::Down => (-x, -y),
            Rotation::Left => (-y, x)
        }
    }

    pub const fn rotate_blocks(&self, blocks: [(i8, i8); 4]) -> [(i8, i8); 4] {
        [
            self.rotate_block(blocks[0]),
            self.rotate_block(blocks[1]),
            self.rotate_block(blocks[2]),
            self.rotate_block(blocks[3])
        ]
    }

    pub const fn rotate_right(&self) -> Rotation {
        match self {
            Rotation::Up => Rotation::Right,
            Rotation::Right => Rotation::Down,
            Rotation::Down => Rotation::Left,
            Rotation::Left => Rotation::Up,
        }
    }

    pub const fn rotate_left(&self) -> Rotation {
        match self {
            Rotation::Up => Rotation::Left,
            Rotation::Left => Rotation::Down,
            Rotation::Down => Rotation::Right,
            Rotation::Right => Rotation::Up,
        }
    }

    pub const fn rotate_180(&self) -> Rotation {
        match self {
            Rotation::Up => Rotation::Down,
            Rotation::Right => Rotation::Left,
            Rotation::Down => Rotation::Up,
            Rotation::Left => Rotation::Right,
        }
    }
}

impl Piece {
    pub const fn blocks(&self) -> [(i8, i8); 4] {
        match self {
            Piece::Z => [(-1, 1), (0, 1), (0, 0), (1, 0)],
            Piece::S => [(-1, 0), (0, 0), (0, 1), (1, 1)],
            Piece::I => [(-1, 0), (0, 0), (1, 0), (2, 0)],
            Piece::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
            Piece::J => [(-1, 0), (0, 0), (1, 0), (-1, 1)],
            Piece::L => [(-1, 0), (0, 0), (1, 0), (1, 1)],
            Piece::T => [(-1, 0), (0, 0), (1, 0), (0, 1)],
        }
    }

    pub const fn from_char(p: char) -> Self {
        match p {
            'i' => Piece::I,
            'o' => Piece::O,
            'j' => Piece::J,
            't' => Piece::T,
            'l' => Piece::L,
            's' => Piece::S,
            'z' => Piece::Z,
            _ => panic!("wtf is this piece")
        }
    }
}

macro_rules! lutify {
    (($e:expr) for $v:ident in [$($val:expr),*]) => {
        [
            $(
                {
                    let $v = $val;
                    $e
                }
            ),*
        ]
    };
}

macro_rules! piece_lut {
    ($v:ident => $e:expr) => {
        lutify!(($e) for $v in [Piece::I, Piece::O, Piece::T, Piece::L, Piece::J, Piece::S, Piece::Z])
    };
}

macro_rules! rotation_lut {
    ($v:ident => $e:expr) => {
        lutify!(($e) for $v in [Rotation::Up, Rotation::Right, Rotation::Down, Rotation::Left])
    };
}


impl PieceLocation {
    pub const fn blocks(&self) -> [(i8, i8); 4] {
        const LUT: [[[(i8, i8); 4]; 4]; 7] =
            piece_lut!(piece => rotation_lut!(rotation => rotation.rotate_blocks(piece.blocks())));
        self.translate_blocks(LUT[self.piece as usize][self.rotation as usize])
        // self.translate_blocks(self.rotation.rotate_blocks(self.piece.blocks()))
    }

    const fn translate(&self, (x, y): (i8, i8)) -> (i8, i8) {
        (x + self.x, y + self.y)
    }

    const fn translate_blocks(&self, cells: [(i8, i8); 4]) -> [(i8, i8); 4] {
        [
            self.translate(cells[0]),
            self.translate(cells[1]),
            self.translate(cells[2]),
            self.translate(cells[3]),
        ]
    }
}

impl Board {
    pub fn place(&mut self, loc: PieceLocation) -> PlacementInfo {
        let spin = loc.spun;
        for &(x, y) in &loc.blocks() {
            self.cols[x as usize] |= 1 << y;
        }
        let line_mask = match loc.possible_line_clear {
            true => self.remove_lines(),
            false => 0
        };
        PlacementInfo {
            spin,
            lines_cleared: line_mask.count_ones()
        }
    }

    pub fn remove_lines(&mut self) -> u64 {
        let lines = self.cols.iter().fold(!0, |a, b| a & b);
        for c in &mut self.cols {
            clear_lines(c, lines);
        }
        lines
    }

    pub fn obstructed(&self, loc: PieceLocation) -> bool {
        for (x, y) in loc.blocks() {
            if x < 0 || x > 9 || y < 0 {
                continue;
            }
            if self.cols[x as usize] & (1 << y) > 0 {
                return true;
            }
        }
        false
    }

    pub fn distance_to_ground(&self, loc: PieceLocation) -> i8 {
        let mut min = 99;
        for (x, y) in loc.blocks() {
            min = min.min(y + (self.cols[x as usize] & ((1 << y) - 1)).leading_zeros() as i8 - 64);
        }
        min
    }
}

fn clear_lines(col: &mut u64, mut lines: u64) {
    while lines != 0 {
        let i = lines.trailing_zeros();
        let mask = (1 << i) - 1;
        *col = *col & mask | *col >> 1 & !mask;
        lines &= !(1 << i);
        lines >>= 1;
    }
}

impl Game {
    pub fn new(p: Option<Piece>) -> Self {
        let mut game = Self {
            board: Board { cols: [0; 10] },
            hold: Piece::Z, // placeholder
            b2b: 0,
            b2b_deficit: 0
        };
        if !p.is_none() {
            game.hold = p.unwrap();
            return game;
        }
        let mut rng = rand::rng();
        game.hold = [Piece::I, Piece::J, Piece::L, Piece::O, Piece::T, Piece::S, Piece::Z].choose(&mut rng).copied().unwrap();
        game
    }

    pub fn advance(&mut self, next: Piece, loc: PieceLocation) -> PlacementInfo {
        if loc.piece != next {
            self.hold = next;
        }
        let info = self.board.place(loc);
        if info.lines_cleared > 0 {
            if info.spin || info.lines_cleared == 4 {
                self.b2b += 1;
                self.b2b_deficit = 0;
            } else if self.board.cols.iter().all(|x| *x == 0) {
                self.b2b += 2;
                self.b2b_deficit = 0;
            } else {
                self.b2b = 0;
            }
        }
        info
    }

    pub fn can_spawn(&self, next: Piece) -> bool {
        !self.board.obstructed(PieceLocation { piece: next, rotation: Rotation::Up, spun: false, x: 4, y: 21, possible_line_clear: false})
        && !self.board.obstructed(PieceLocation { piece: self.hold, rotation: Rotation::Up, spun: false, x: 4, y: 21, possible_line_clear: false})
    }
}