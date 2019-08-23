use ndarray::Array2;

use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Black,
}

#[derive(Clone, Copy, Debug)]
pub enum PieceKind {
    Flat,
    Wall,
    Cap,
}

#[derive(Debug)]
pub enum Move {
    Place(PieceKind, (u8, u8), String),
    Throw((u8, u8, u8), char, Vec<u8>, String), //Source then direction and quantity then ptn
}

#[derive(Debug, PartialEq)]
pub enum Victory {
    Neither,
    WhiteFlat(u32),
    WhiteRoad,
    WhiteOther,
    BlackFlat(u32),
    BlackRoad,
    BlackOther,
    White(u32),
    Black(u32),
    Draw,
}

#[derive(Clone, Copy)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    pub fn new(color: Color, kind: PieceKind) -> Piece {
        Piece { color, kind }
    }
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Color::White = self.color {
            let s = match self.kind {
                PieceKind::Cap => "C",
                PieceKind::Wall => "S",
                PieceKind::Flat => "W",
            };
            write!(f, "{}", s)
        } else {
            //Black
            let s = match self.kind {
                PieceKind::Cap => "D",
                PieceKind::Wall => "T",
                PieceKind::Flat => "B",
            };
            write!(f, "{}", s)
        }
    }
}

#[derive(Default)]
pub struct Tile {
    pub stack: Vec<Piece>,
}

impl Tile {
    pub fn top(&self) -> Option<&Piece> {
        if self.stack.len() == 0 {
            return None;
        } else {
            return Some(&self.stack[self.stack.len() - 1]);
        }
    }
    pub fn top_unchecked(&self) -> &Piece {
        &self.stack.get(self.stack.len() - 1).unwrap()
    }
    pub fn add_piece(&mut self, piece: Piece) {
        self.stack.push(piece);
    }
    pub fn add_pieces(&mut self, mut pieces: Vec<Piece>) {
        self.stack.append(&mut pieces);
    }
    pub fn is_empty(&self) -> bool {
        self.stack.len() == 0
    }
}

impl fmt::Debug for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.stack)
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.stack)
    }
}

///Game state contains the board and the players. For reference, a is the first column, 1 is the
/// first row. Let player1 be white and player2 be black
pub struct State {
    pub board: Array2<Tile>,
    pub size: u8,
    pub player1: Player,
    pub player2: Player,
    pub notation: Vec<String>,
}

impl State {
    pub fn new(size: u8) -> State {
        let (pieces, caps) = {
            match size {
                3 => (10, 0),
                4 => (15, 0),
                5 => (21, 1),
                6 => (30, 1),
                8 => (50, 2),
                _ => (21, 1), //Default 5
            }
        };
        State {
            board: Array2::default((size as usize, size as usize)),
            size,
            player1: Player::new(Color::White, pieces, caps),
            player2: Player::new(Color::Black, pieces, caps),
            notation: Vec::new(),
        }
    }
    pub fn new_with_players(size: u8, player1: Player, player2: Player) -> State {
        State {
            board: Array2::default((size as usize, size as usize)),
            size,
            player1,
            player2,
            notation: Vec::new(),
        }
    }

    /// True if the input square is off the board
    pub fn out_of_bounds(&self, row: u8, col: u8) -> bool {
        row >= self.size || col >= self.size
    }

    /// True if the input square has no pieces on it, or vacuously true if the
    /// square is out of bounds
    pub fn is_empty(&self, row: u8, col: u8) -> bool {
        self.board
            .get((row as usize, col as usize))
            .map(|tile| tile.top())
            .is_some()
    }

    pub fn is_edge(&self, pos: (usize, usize)) -> bool {
        let row = pos.0 as u8;
        let col = pos.1 as u8;
        row == 0 || row == (self.size - 1) || col == 0 || col == (self.size - 1)
    }

    pub fn has_capstone(&self, color: Color) -> bool {
        self.get_player(color).caps > 0
    }

    pub fn get_tile(&self, row: u8, col: u8) -> &Tile {
        self.board.get((row as usize, col as usize)).unwrap()
    }

    pub fn get_mut_tile(&mut self, row: u8, col: u8) -> &mut Tile {
        self.board.get_mut((row as usize, col as usize)).unwrap()
    }

    pub fn get_player(&self, color: Color) -> &Player {
        match color {
            Color::White => &self.player1,
            Color::Black => &self.player2,
        }
    }

    pub fn get_mut_player(&mut self, color: Color) -> &mut Player {
        match color {
            Color::White => &mut self.player1,
            Color::Black => &mut self.player2,
        }
    }

    pub fn add_notation(&mut self, ptn: String) {
        self.notation.push(ptn);
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string = String::from("");
        if self.notation.len() % 2 == 0 {
            string.push_str("White to move: \n");
        } else {
            string.push_str("Black to move: \n");
        }
        for i in 0..self.size as usize {
            for j in 0..self.size as usize {
                string.push_str(
                    &self
                        .board
                        .get((self.size as usize - i - 1, j))
                        .unwrap()
                        .to_string(),
                );
            }
            string.push_str("\n");
        }
        write!(
            f,
            "{}\nWhite: ({}, {}) Black: ({}, {})",
            &string, self.player1.pieces, self.player1.caps, self.player2.pieces, self.player2.caps
        )
    }
}

pub struct Player {
    pub color: Color,
    pub pieces: i32,
    pub caps: i32,
}

impl Player {
    pub fn new(color: Color, pieces: i32, caps: i32) -> Player {
        Player {
            color,
            pieces,
            caps,
        }
    }

    pub fn has_capstone(&self) -> bool {
        self.caps > 0
    }
}