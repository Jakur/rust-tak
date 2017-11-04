use std::fmt;
use ndarray::Array2;
use std::collections::HashSet;
use std::cell::RefCell;
use std::rc::Rc;
use regex::Regex;
use super::Game;

#[derive(Clone)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug)]
pub enum PieceKind {
    Flat,
    Wall,
    Cap,
}

#[derive(Debug)]
pub enum Move {
    Place(PieceKind, (u8, u8)),
    Throw((u8, u8, u8), char, Vec<u8>), //Source then direction and quantity
}

#[derive(Debug, PartialEq)]
pub enum Victory {
    Neither,
    White,
    Black,
    Draw,
}

pub struct Piece {
    color: Color,
    kind: PieceKind,
}

impl Piece {
    pub fn new(color: Color, kind: PieceKind) -> Piece {
        Piece {
            color,
            kind,
        }
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
        } else { //Black
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
    stack: Vec<Piece>,
}

impl Tile {
    pub fn top(&self) -> Option<&Piece> {
        match self.stack.get(self.stack.len() - 1) {
            Some(p) => Some(&p),
            _ => None,
        }
    }
    fn top_unchecked(&self) -> &Piece {
        &self.stack.get(self.stack.len() - 1).unwrap()
    }
    fn add_piece(&mut self, piece: Piece) {
        self.stack.push(piece);
    }
    fn add_pieces(&mut self, mut pieces: Vec<Piece>) {
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

///Game state contains the board and the players. For reference, a is the first column, 1 is the
/// first row. Let player1 be white and player2 be black
pub struct State {
    pub board: Array2<Tile>,
    pub size: u8,
    pub player1: Player,
    pub player2: Player,
}

impl State {
    pub fn new(size: u8, player1: Player, player2: Player) -> State {
        State {
            board: Array2::default((size as usize, size as usize)),
            size,
            player1,
            player2,
        }
    }
}

pub struct Player {
    pub color: Color,
    pub pieces: i32,
    pub caps: i32,
}

pub struct Reached {
    north: bool,
    south: bool,
    east: bool,
    west: bool,
}

pub trait Opening {
    fn legal_move<R, O>(&self, state: &Game<R, O>, m: &Move) -> Option<Color>
        where R: RuleSet, O: Opening;
    fn is_opening<R, O>(&self, game: &Game<R, O>) -> bool where R: RuleSet, O: Opening;
    fn current_color<R, O>(&self, game: &Game<R, O>) -> Color where R: RuleSet, O: Opening;
}

pub struct StandardOpening {

}

impl Opening for StandardOpening {
    ///If the move is illegal under the opening rules returns None. If it is legal, it returns
    /// the color of the piece which will be placed.
    fn legal_move<R, O>(&self, game: &Game<R, O>, m: &Move) -> Option<Color>
        where R: RuleSet, O: Opening {
        match m {
            &Move::Place(ref kind, tuple) => {
                if let &PieceKind::Flat = kind {
                    if game.rules.out_of_bounds(tuple) {
                        return None
                    } else {
                        if game.rules.get_tile(tuple).is_empty() {
                            if game.ply == 0 { //First piece is black
                                return Some(Color::Black)
                            }
                            return Some(Color::White) //Second piece is white
                        }
                        return None
                    }
                } else {return None}
            }
            _ => return None
        }
    }
    ///Returns true if the next ply is considered to be out of the opening
    fn is_opening<R, O>(&self, game: &Game<R, O>) -> bool where R: RuleSet, O: Opening {
        return game.ply < 2
    }
    fn current_color<R, O>(&self, game: &Game<R, O>) -> Color where R: RuleSet, O: Opening {
        match game.rules.current_color(game.ply) {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

pub trait RuleSet {
    ///Makes a move and returns true if a move is valid under this rule set else returns false.
    fn make_move(&mut self, m: Move, color: Color) -> bool {
        if let Move::Place(c, (a, b)) = m {
            if let PieceKind::Cap = c {
                if self.is_empty((a, b)) && !self.out_of_bounds((a, b)) &&
                    self.has_capstone(self.current_player(color.clone())) {
                    self.get_mut_tile((a, b)).add_piece(Piece::new(color.clone(), c));
                    match color {
                        Color::White => {self.get_mut_state().player1.caps -= 1},
                        Color::Black => {self.get_mut_state().player2.caps -= 1}
                    }
                    return true;
                } else {return false;}
            } else {
                if self.is_empty((a, b)) && !self.out_of_bounds((a, b)) {
                    self.get_mut_tile((a, b)).add_piece(Piece::new(color.clone(), c));
                    match color {
                        Color::White => {self.get_mut_state().player1.pieces -= 1},
                        Color::Black => {self.get_mut_state().player2.pieces -= 1}
                    }
                    return true;
                }
                return false;
            }
        } else { //Stack throw
            let data = match m {
                Move::Throw(source, dir, vec) => (source, dir, vec),
                _ => { return false } //This should not happen
            };
            let source= data.0;
            if source.0 > self.get_size() { //Picked up too many pieces
                return false;
            }
            let vec = data.2;
            if vec.len() < 1 || self.get_tile((source.1, source.2)).is_empty() {
                return false;
            }
            let mut x = (data.0).1;
            let mut y = (data.0).2;
            //Check if the farthest target is on the board, usize is Copy so no problems here
            match data.1 {
                '+' => {x += vec.len() as u8},
                '-' => {x -= vec.len() as u8},
                '<' => {y -= vec.len() as u8},
                '>' => {y += vec.len() as u8},
                _ => {return false}, //Invalid
            }
            if self.out_of_bounds((x, y)) {
                return false;
            }
            //Delay reset x, y
            //Check the last position in the throw vector for special case wall crush
            {
                let last_tile = self.get_tile((x, y));
                //We assume the vec to be in normal stack order.
                if !last_tile.stack.is_empty() {
                    match last_tile.top_unchecked().kind {
                        PieceKind::Wall => { //Check for valid crush
                            if let PieceKind::Cap = self.get_tile((source.0, source.1))
                                .top_unchecked().kind {
                                if vec[vec.len() - 1] != 1 {return false} //Only the Cap can crush
                            } else {return false}
                        },
                        PieceKind::Cap => { //Cannot end throw on a Cap either
                            return false
                        }
                        _ => {},
                    }
                }
            }
            x = (data.0).1;
            y = (data.0).2;
            let mut sum = 0;
            for val in vec.iter() {
                match data.1 { //Optimize into one match later, if necessary
                    '+' => {x += 1},
                    '-' => {x -= 1},
                    '<' => {y -= 1},
                    '>' => {y += 1},
                    _ => {return false}, //Invalid
                }
                match self.get_tile((x, y)).top() {
                    Some(p) => {match p.kind {
                        PieceKind::Flat => {},
                        _ => {return false}
                    }}
                    _ => {},
                }
                sum += *val;
            }
            //Now that we've found the move valid, we execute it, in reverse
            let source_len = self.get_mut_tile((source.1, source.2)).stack.len();
            let mut source_vec = self.get_mut_tile((source.1, source.2)).stack.split_off(source_len - sum as usize);


            for val in vec.iter().rev() {
                let val = *val as usize;
                let length = source_vec.len();
                self.get_mut_tile((x, y)).add_pieces(source_vec.drain(length-val..length).collect());
                match data.1 { //Optimize into one match later, if necessary
                    '+' => {x -= 1},
                    '-' => {x += 1},
                    '<' => {y += 1},
                    '>' => {y -= 1},
                    _ => {panic!("Partially executed move found invalid!")}, //Todo not kill whole program with this.
                }
            }
            true
        }
    }
    fn is_empty(&self, index: (u8, u8)) -> bool {
        self.get_tile(index).is_empty()
    }
    fn out_of_bounds(&self, index: (u8, u8)) -> bool {
        if index.0 > self.get_size() || index.1 > self.get_size() {//No lower check because unsigned
            return true
        }
        false
    }
    fn check_win(&self, last_to_move: Color) -> Victory {
        let discovered: Rc<RefCell<HashSet<(usize, usize)>>> = Rc::new(RefCell::new(HashSet::new()));
        //This iter generation may be able to be optimized, we'll see
        let iter = self.get_state().board.indexed_iter().
            filter(|x| self.is_edge(x.0));
        let mut white_road = false;
        let mut black_road = false;
        //Road check for both players
        for t in iter {
            {
                if discovered.borrow_mut().contains(&t.0) {continue;}
            }
            let white_piece = match (t.1).top() {
                Some(&Piece {color: Color::White, ..}) => true,
                Some(&Piece {color: Color::Black, ..}) => false,
                _ => {continue;}
            };
            //If we already found a road for that color, ignore this piece
            if white_road && white_piece {
                continue;
            }
            if black_road && !white_piece {
                continue;
            }
            let mut reached = Reached {north: false, south: false, east: false, west: false};
            if (t.0).0 == 0 {
                reached.north = true;
            }
            if (t.0).0 == self.get_size() as usize - 1 {
                reached.south = true;
            }
            if (t.0).1 == 0 {
                reached.west= true;
            }
            if (t.0).1 == self.get_size() as usize - 1 {
                reached.east = true;
            }
            let road = self.search(white_piece, Rc::new(RefCell::new(reached)), discovered.clone(), t.0);
            if road {
                if white_piece {
                    white_road = true;
                } else {
                    black_road = true;
                }
                if white_road && black_road {
                    if let Color::White = last_to_move {
                        return Victory::White;
                    } else {return Victory::Black;}
                }
            }
        }
        if white_road {
            return Victory::White
        } else if black_road {
            return Victory::Black
        }
        //Out of pieces check for both players
        if self.get_state().player1.pieces == 0 || self.get_state().player2.pieces == 0 {
            return self.flat_game();
        }
        //Board fill check
        let set = discovered.borrow_mut();
        if self.get_state().size as usize * self.get_state().size as usize == set.len() { //Guaranteed board fill
            return self.flat_game()
        } else { //We actually have to count them "manually"
            for t in self.get_state().board.iter() {
                match t.top() {
                    Some(&Piece {..}) => {},
                    _ => {return Victory::Neither}
                }
            }
            return self.flat_game()
        }

    }
    ///Performs a depth-first search on the board, looking for roads of the color initially passed
    /// in to the function. No optimizations given for direction to look: it prioritizes down,
    /// right, left, up, which should improve the best case due to the way the iterator is
    /// constructed, but nothing else.
    fn search(&self, white_start: bool, r: Rc<RefCell<Reached>>, set: Rc<RefCell<HashSet<(usize, usize)>>>,
              node:(usize, usize)) -> bool {
        //Check if we're still on the board
        let tile = match self.get_state().board.get(node) {
            Some(t) => t,
            _ => return false,
        };
        let white = match tile.top() {
            Some(&Piece {color: _, kind: PieceKind::Wall}) => {
                let mut m_set = set.borrow_mut();
                if m_set.contains(&node) {
                    return false; //Already checked
                }
                m_set.insert(node); return false},
            Some(&Piece {color: Color::White, ..}) => true,
            Some(&Piece {color: Color::Black, ..}) => false,
            _ => {return false;},
        };
        //Add this to the discovered set, then drop the mutability from the scope
        {
            let mut m_set = set.borrow_mut();
            if m_set.contains(&node) {
                return false; //Already checked
            }
            m_set.insert(node);
        }
        if white ^ white_start { //If this tile isn't the same color as what we're investigating
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
                x.west= true;
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

        return self.search(white_start, r.clone(), set.clone(), (node.0 + 1, node.1)) ||
            self.search(white_start, r.clone(), set.clone(), (node.0, node.1 + 1)) ||
            self.search(white_start, r.clone(), set.clone(), (node.0, node.1 - 1)) ||
            self.search(white_start, r.clone(), set.clone(), (node.0 - 1, node.1));
    }
    ///Evaluates the result of the game if it goes to a flat count.
    fn flat_game(&self) -> Victory {
        let mut white = 0;
        let mut black = 0;
        for t in self.get_state().board.iter() {
            match t.top() {
                Some(&Piece {color: Color::White, kind: PieceKind::Flat}) => {white += 1;},
                Some(&Piece {color: Color::Black, kind: PieceKind::Flat}) => {black += 1;},
                _ => {},
            }
        }
        if white > black {
            return Victory::White;
        } else if black > white {
            return Victory::Black;
        }
        return Victory::Draw;
    }
    fn get_tile(&self, index: (u8, u8)) -> &Tile {
        self.get_state().board.get((index.0 as usize, index.1 as usize)).unwrap()
    }
    fn get_mut_tile(&mut self, index: (u8, u8)) -> &mut Tile {
        return self.get_mut_state().board.get_mut((index.0 as usize, index.1 as usize)).unwrap()
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
    fn is_edge(&self, pos:(usize, usize)) -> bool {
        pos.0 == 0 || pos.0 == (self.get_size()-1) as usize || pos.1 == 0 ||
            pos.1 == (self.get_size()-1) as usize
    }
    fn current_color(&self, ply: u32) -> Color {
        if ply % 2 == 0 {
            Color::White
        } else {
            Color::Black
        }
    }
}

pub struct StandardRules {
    pub state: State,
}

impl StandardRules {
    pub fn new(state: State) -> StandardRules {
        StandardRules {
            state,
        }
    }

    pub fn place_w_flat(&mut self, index: (u8, u8)) {
        self.get_mut_tile(index).add_piece(Piece {
            color: Color::White,
            kind: PieceKind::Flat,
        });
    }
    pub fn place_b_flat(&mut self, index: (u8, u8)) {
        self.get_mut_tile(index).add_piece(Piece {
            color: Color::Black,
            kind: PieceKind::Flat,
        });
    }
    pub fn place_w_wall(&mut self, index: (u8, u8)) {
        self.get_mut_tile(index).add_piece(Piece {
            color: Color::White,
            kind: PieceKind::Wall,
        });
    }
}

impl RuleSet for StandardRules {
    fn get_state(&self) -> &State {
        &self.state
    }

    fn get_mut_state(&mut self) -> &mut State {
        &mut self.state
    }
}