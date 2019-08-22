use failure::{bail, Error};
use ndarray::Array2;

use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;

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

pub struct Reached {
    north: bool,
    south: bool,
    east: bool,
    west: bool,
}

pub trait Rules {
    /// Returns true if a given move is legal but does not execute the move
    fn legal_move(&self, m: Move) -> bool {
        match m {
            Move::Place(kind, (row, col), _ptn) => {
                let color = self.current_color();
                let piece = Piece::new(color, kind);
                self.legal_place_move(piece, row, col).is_ok()
            }
            Move::Throw(source, dir, vec, _ptn) => self.legal_stack_move(source, dir, &vec).is_ok(),
        }
    }
    /// Attempts to make a move returning Ok if successful or Error if unsuccessful
    fn make_move(&mut self, m: Move) -> Result<(), Error> {
        let ptn = match m {
            Move::Place(kind, (row, col), ptn) => {
                let color = self.current_color();
                let piece = Piece::new(color, kind);
                self.legal_place_move(piece, row, col)?;
                self.unchecked_place_move(piece, row, col);
                ptn
            }
            Move::Throw(source, dir, vec, ptn) => {
                let res = self.legal_stack_move(source, dir, &vec)?;
                self.unchecked_stack_move(source, dir, vec, res);
                ptn
            }
        };
        self.get_mut_state().add_notation(ptn);
        Ok(())
    }

    fn unchecked_place_move(&mut self, piece: Piece, row: u8, col: u8) {
        let state = self.get_mut_state();
        let color = piece.color;
        match piece.kind {
            PieceKind::Cap => {
                state.get_mut_player(color).caps -= 1;
            }
            _ => {
                state.get_mut_player(color).pieces -= 1;
            }
        }
        state.get_mut_tile(row, col).add_piece(piece);
    }

    fn legal_place_move(&self, piece: Piece, row: u8, col: u8) -> Result<(), Error> {
        let state = self.get_state();
        // Check valid square for placing a piece
        if state.out_of_bounds(row, col) || !state.is_empty(row, col) {
            bail!("Invalid square selected");
        }
        if let PieceKind::Cap = piece.kind {
            if !state.has_capstone(piece.color) {
                bail!("Player has no capstones left");
            }
        }
        Ok(())
    }

    fn unchecked_stack_move(
        &mut self,
        source: (u8, u8, u8),
        dir: char,
        vec: Vec<u8>,
        res: (u8, u8, u8),
    ) {
        let state = self.get_mut_state();
        let (sum, mut x, mut y) = res;
        // Now that we've found the move valid, we execute it, in reverse
        let source_len = state.get_mut_tile(source.1, source.2).stack.len();
        let mut source_vec = state
            .get_mut_tile(source.1, source.2)
            .stack
            .split_off(source_len - sum as usize);

        for val in vec.iter().rev() {
            let val = *val as usize;
            let length = source_vec.len();
            state
                .get_mut_tile(x, y)
                .add_pieces(source_vec.drain(length - val..length).collect());
            match dir {
                //Optimize into one match later, if necessary
                '+' => x -= 1,
                '-' => x += 1,
                '<' => y += 1,
                '>' => y -= 1,
                _ => unreachable!(), // Already checked
            }
        }
    }

    fn legal_stack_move(
        &self,
        source: (u8, u8, u8),
        dir: char,
        vec: &[u8],
    ) -> Result<(u8, u8, u8), Error> {
        let state = self.get_state();
        if source.0 > state.size || state.out_of_bounds(source.1, source.2) {
            bail!("Invalid move signature for this board");
        }
        let source_tile = state.get_tile(source.1, source.2);
        if vec.len() < 1 || source_tile.is_empty() {
            bail!("Moving from an empty tile");
        }
        let mut x = source.1;
        let mut y = source.2;

        //Check if the farthest target is on the board, usize is Copy so no problems here
        match dir {
            '+' => x += vec.len() as u8,
            '-' => {
                if x as usize >= vec.len() {
                    x -= vec.len() as u8
                } else {
                    bail!("Target tile(s) off the board");
                }
            }
            '<' => {
                if y as usize >= vec.len() {
                    y -= vec.len() as u8
                } else {
                    bail!("Target tile(s) off the board");
                }
            }
            '>' => y += vec.len() as u8,
            _ => bail!("Unknown movement direction"), //Invalid
        }
        if state.out_of_bounds(x, y) {
            bail!("Target tile(s) off the board");
        }
        //Delay reset x, y
        //Check the last position in the throw vector for special case wall crush
        let (last_x, last_y) = {
            let last_tile = state.get_tile(x, y);
            //We assume the vec to be in normal stack order.
            if !last_tile.stack.is_empty() {
                match last_tile.top_unchecked().kind {
                    PieceKind::Wall => {
                        //Check for valid crush
                        if let PieceKind::Cap =
                            state.get_tile(source.1, source.2).top().unwrap().kind
                        {
                            if vec[vec.len() - 1] != 1 {
                                bail!(
                                    "The capstone must step alone\
                                     to crush walls"
                                );
                            }
                        } else {
                            bail!("Cannot crush a wall without a capstone");
                        }
                    }
                    PieceKind::Cap => {
                        bail!("Cannot end throw on a capstone");
                    }
                    _ => {}
                }
            }
            (x, y)
        };
        x = source.1;
        y = source.2;
        let mut sum = 0;
        for val in vec.iter() {
            match dir {
                // Optimize into one match later, if necessary
                '+' => x += 1,
                '-' => x -= 1,
                '<' => y -= 1,
                '>' => y += 1,
                _ => unreachable!(), // Already checked
            }
            if !(x == last_x && y == last_y) {
                // Already checked the last tile
                match state.get_tile(x, y).top() {
                    Some(p) => match p.kind {
                        PieceKind::Flat => {}
                        _ => bail!("Cannot move through a wall or capstone"),
                    },
                    None => {}
                }
            }
            sum += *val;
        }

        Ok((sum, x, y))
    }
    /// Whether or not the game is in the opening phase, the phase of the game
    /// where the rules behave differently than normal. In a standard game this
    /// corresponds to the first two plies
    fn is_opening(&self) -> bool {
        self.current_ply() < 2
    }

    /// The color of a flat if one were laid. This usually corresponds to
    /// the active player's color.
    fn current_color(&self) -> Color {
        if self.is_opening() {
            // Colors reversed in opening
            if self.current_ply() % 2 == 0 {
                Color::Black
            } else {
                Color::White
            }
        } else {
            if self.current_ply() % 2 == 0 {
                Color::White
            } else {
                Color::Black
            }
        }
    }
    fn check_win(&self, last_to_move: Color) -> Victory {
        let discovered: Rc<RefCell<HashSet<(usize, usize)>>> =
            Rc::new(RefCell::new(HashSet::new()));
        //This iter generation may be able to be optimized, we'll see
        let iter = self
            .get_state()
            .board
            .indexed_iter()
            .filter(|x| self.get_state().is_edge(x.0));
        let mut white_road = false;
        let mut black_road = false;
        //Road check for both players
        for t in iter {
            if discovered.borrow_mut().contains(&t.0) {
                continue;
            }
            let white_piece = match (t.1).top() {
                Some(&Piece {
                    color: Color::White,
                    ..
                }) => true,
                Some(&Piece {
                    color: Color::Black,
                    ..
                }) => false,
                _ => {
                    continue;
                }
            };
            //If we already found a road for that color, ignore this piece
            if white_road && white_piece {
                continue;
            }
            if black_road && !white_piece {
                continue;
            }
            let mut reached = Reached {
                north: false,
                south: false,
                east: false,
                west: false,
            };
            if (t.0).0 == 0 {
                reached.north = true;
            } else if (t.0).0 == self.get_size() as usize - 1 {
                reached.south = true;
            }
            if (t.0).1 == 0 {
                reached.west = true;
            } else if (t.0).1 == self.get_size() as usize - 1 {
                reached.east = true;
            }
            let road = self.search(
                white_piece,
                Rc::new(RefCell::new(reached)),
                discovered.clone(),
                t.0,
            );
            if road {
                if white_piece {
                    white_road = true;
                } else {
                    black_road = true;
                }
                if white_road && black_road {
                    if let Color::White = last_to_move {
                        return Victory::WhiteRoad;
                    } else {
                        return Victory::BlackRoad;
                    }
                }
            }
        }
        if white_road {
            return Victory::WhiteRoad;
        } else if black_road {
            return Victory::BlackRoad;
        }
        //Out of pieces check for both players
        if self.get_state().player1.pieces == 0 || self.get_state().player2.pieces == 0 {
            return self.flat_game();
        }
        //Board fill check
        let set = discovered.borrow_mut();
        if self.get_state().size as usize * self.get_state().size as usize == set.len() {
            //Guaranteed board fill
            return self.flat_game();
        } else {
            //We actually have to count them "manually"
            for t in self.get_state().board.iter() {
                match t.top() {
                    Some(&Piece { .. }) => {}
                    _ => return Victory::Neither,
                }
            }
            return self.flat_game();
        }

    }
    ///Performs a depth-first search on the board, looking for roads of the color initially passed
    /// in to the function. No optimizations given for direction to look: it prioritizes down,
    /// right, left, up, which should improve the best case due to the way the iterator is
    /// constructed, but nothing else.
    fn search(
        &self,
        white_start: bool,
        r: Rc<RefCell<Reached>>,
        set: Rc<RefCell<HashSet<(usize, usize)>>>,
        node: (usize, usize),
    ) -> bool {
        //Check if we're still on the board
        let tile = match self.get_state().board.get(node) {
            Some(t) => t,
            _ => return false,
        };
        let white = match tile.top() {
            Some(&Piece {
                color: _,
                kind: PieceKind::Wall,
            }) => {
                let mut m_set = set.borrow_mut();
                if m_set.contains(&node) {
                    return false; //Already checked
                }
                m_set.insert(node);
                return false;
            }
            Some(&Piece {
                color: Color::White,
                ..
            }) => true,
            Some(&Piece {
                color: Color::Black,
                ..
            }) => false,
            _ => {
                return false;
            }
        };
        //Add this to the discovered set, then drop the mutability from the scope
        {
            let mut m_set = set.borrow_mut();
            if m_set.contains(&node) {
                return false; //Already checked
            }
            m_set.insert(node);
        }
        if white ^ white_start {
            //If this tile isn't the same color as what we're investigating
            return false;
        }

        //Start flag setting/checking
        let last = (self.get_size() - 1) as usize;
        {
            let mut x = r.borrow_mut();
            if node.0 == 0 {
                x.north = true;
            }
            if node.0 == last {
                x.south = true;
            }
            if node.1 == 0 {
                x.west = true;
            }
            if node.1 == last {
                x.east = true;
            }
            if x.north && x.south {
                return true;
            } else if x.east && x.west {
                return true;
            }
        }

        //Check for usize underflow and then recurse accordingly
        if node.0 == 0 {
            if node.1 == 0 {
                return self.search(white_start, r.clone(), set.clone(), (node.0 + 1, node.1))
                    || self.search(white_start, r.clone(), set.clone(), (node.0, node.1 + 1));
            } else {
                return self.search(white_start, r.clone(), set.clone(), (node.0 + 1, node.1))
                    || self.search(white_start, r.clone(), set.clone(), (node.0, node.1 + 1))
                    || self.search(white_start, r.clone(), set.clone(), (node.0, node.1 - 1));
            }
        } else if node.1 == 0 {
            return self.search(white_start, r.clone(), set.clone(), (node.0 + 1, node.1))
                || self.search(white_start, r.clone(), set.clone(), (node.0, node.1 + 1))
                || self.search(white_start, r.clone(), set.clone(), (node.0 - 1, node.1));
        } else {
            return self.search(white_start, r.clone(), set.clone(), (node.0 + 1, node.1))
                || self.search(white_start, r.clone(), set.clone(), (node.0, node.1 + 1))
                || self.search(white_start, r.clone(), set.clone(), (node.0, node.1 - 1))
                || self.search(white_start, r.clone(), set.clone(), (node.0 - 1, node.1));
        }
    }
    ///Evaluates the result of the game if it goes to a flat count.
    fn flat_game(&self) -> Victory {
        let mut white = 0;
        let mut black = 0;
        for t in self.get_state().board.iter() {
            match t.top() {
                Some(&Piece {
                    color: Color::White,
                    kind: PieceKind::Flat,
                }) => {
                    white += 1;
                }
                Some(&Piece {
                    color: Color::Black,
                    kind: PieceKind::Flat,
                }) => {
                    black += 1;
                }
                _ => {}
            }
        }
        if white > black + self.get_komi() {
            return Victory::White(white - self.get_komi());
        } else if black + self.get_komi() > white {
            return Victory::Black(black + self.get_komi());
        }
        return Victory::Draw;
    }
    fn get_tile(&self, index: (u8, u8)) -> &Tile {
        self.get_state()
            .board
            .get((index.0 as usize, index.1 as usize))
            .unwrap()
    }
    fn get_mut_tile(&mut self, index: (u8, u8)) -> &mut Tile {
        return self
            .get_mut_state()
            .board
            .get_mut((index.0 as usize, index.1 as usize))
            .unwrap();
    }
    fn has_capstone(&self, player: &Player) -> bool {
        player.caps > 0
    }
    fn current_player(&self, color: Color) -> &Player {
        match color {
            Color::White => &self.get_state().player1,
            Color::Black => &self.get_state().player2,
        }
    }
    fn get_size(&self) -> u8 {
        self.get_state().size
    }
    fn get_state(&self) -> &State;
    fn get_mut_state(&mut self) -> &mut State;
    fn add_notation(&mut self, string: String) {
        self.get_mut_state().notation.push(string);
    }
    ///Return the komi for the game, or 0 if there is none
    fn get_komi(&self) -> u32;

    /// The 0-indexed ply count of the game
    fn current_ply(&self) -> u32;
}

pub struct StandardRules {
    pub state: State,
    pub komi: u32,
}

impl StandardRules {
    pub fn new(state: State, komi: u32) -> StandardRules {
        StandardRules { state, komi }
    }
}

impl Rules for StandardRules {
    fn get_state(&self) -> &State {
        &self.state
    }

    fn get_mut_state(&mut self) -> &mut State {
        &mut self.state
    }

    fn get_komi(&self) -> u32 {
        self.komi
    }

    fn current_ply(&self) -> u32 {
        self.get_state().notation.len() as u32
    }
}