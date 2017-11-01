use std::fmt;
use ndarray::Array2;
use std::collections::HashSet;
use std::cell::RefCell;
use std::rc::Rc;
use regex::Regex;

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

#[derive(Debug)]
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
}

pub trait RuleSet {
    ///Makes a move and returns true if a move is valid under this rule set else returns false.
    fn make_move(&mut self, m: Move, color: Color) -> bool {
        if let Move::Place(c, (a, b)) = m {
            if let PieceKind::Cap = c {
                if self.is_empty((a, b)) && !self.out_of_bounds((a, b)) &&
                    self.has_capstone(self.current_player(color.clone())) {
                    self.get_mut_tile((a, b)).add_piece(Piece::new(color, c));
                    return true;
                } else {return false;}
            } else {
                if self.is_empty((a, b)) && !self.out_of_bounds((a, b)) {
                    self.get_mut_tile((a, b)).add_piece(Piece::new(color, c));
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


            for val in vec.iter() {
                let val = *val as usize;
                self.get_mut_tile((x, y)).add_pieces(source_vec.drain(0..val).collect());
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

pub struct Game<R, O> where R: RuleSet, O: Opening {
    pub rules: R,
    pub opening: O,
    pub ply: u32,
}

impl<R, O> Game<R, O> where R: RuleSet, O: Opening {
    pub fn new(rules: R, opening: O) -> Game<R, O> {
        Game {
            rules,
            opening,
            ply: 0,
        }
    }
    pub fn execute_move(&mut self, ptn_string: String) -> bool {
        let m = match ptn_move(&ptn_string) {
            Some(m) => m,
            _ => return false
        };
        if self.opening.is_opening(&self) {
            match self.opening.legal_move(&self, &m) {
                Some(color) => {
                    if self.rules.make_move(m, color) {
                        self.ply += 1;
                        return true
                    }
                    return false
                },
                _ => return false
            }
        } else {
            let color = {
                if self.ply % 2 == 0 {
                    Color::White
                } else {
                    Color::Black
                }
            };
            if self.rules.make_move(m, color) {
                self.ply += 1;
                return true
            }
            return false
        }
    }
}
///Defunct for now...
///Right now going with experimental formatting where the first 6 bits will be piece placed or
/// number for movement, the next 2 for direction direction which will be 0 for +, adding 1 with
/// each rotation clockwise, or 0 if no movement, the next byte will be column a-p and then the
/// second 4 bits will encode the row 0-15. Then the movement bytes, which will be 1 byte per move,
/// which is wasteful of 4 bits per tile. This can be changed, but as it will lead to more complex
/// code, I will do it this simpler way for now.
pub fn byte_ptn(bytes: &[u8]) -> Move {
    //Note: "1 elision" in a move is not allowed
    match bytes[0] {
        0 => {
            let column = bytes[1] & 0xF0 >> 4;
            let row = bytes[1] & 0x0F;
            return Move::Place(PieceKind::Flat, (row - 1, column - 1));
        },
        1 => {
            let column = bytes[1] & 0xF0 >> 4;
            let row = bytes[1] & 0x0F;
            return Move::Place(PieceKind::Wall, (row - 1, column - 1));
        },
        2 => {
            let column = bytes[1] & 0xF0 >> 4;
            let row = bytes[1] & 0x0F;
            return Move::Place(PieceKind::Cap, (row - 1, column - 1));
        },
        _ => {
            let picked_up = (bytes[0] & 0xFC) >> 2;
            let ch = match bytes[0] & 0x03 {
                0 => '+',
                1 => '>',
                2 => '<',
                _ => '-',
            };
            return Move::Place(PieceKind::Flat, (0, 0));//Todo fix this
        }
    }
}

pub fn ptn_move(string: &String) -> Option<Move> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(\d)?(?i)([CS])?([a-h])([1-8])(([<>+-])([1-8]+)?(\*)?)?$").unwrap();
    }
//    let res = RE.captures_iter(string);
//    for r in res {
//        println!("{:?}", r);
//    }
    let res = RE.captures(string);
    let res = match res {
        Some(x) => x,
        _ => {return None},
    };
    match res.get(6) { //Directional symbol
        Some(d) => { //stack move
            let dir = String::from(d.as_str()).pop().unwrap_or('+');
            let vec = res.get(7).map_or("1", |m| m.as_str()); //1 for elision
            let vec: Vec<_> = vec.chars().map(|c| c.to_digit(16).unwrap_or(8) as u8).collect();
            return Some(Move::Throw(
                (res.get(1).map_or(1, |x| x.as_str().parse::<u8>().unwrap_or(1)),
                 res.get(4).map_or(0, |m| m.as_str().parse::<u8>().unwrap_or(1)-1),
                 col_match(res.get(3).map_or(String::from(""), |m| m.as_str().to_lowercase()))),
                dir, vec))
        }
        None => { //place
            let kind = match res.get(2).map_or("", |m| m.as_str()) {
                "S" => PieceKind::Wall,
                "C" => PieceKind::Cap,
                "s" => PieceKind::Wall,
                "c" => PieceKind::Cap,
                _ => PieceKind::Flat,
            };
            return Some(Move::Place(kind, (res.get(4).map_or(0, |m| m.as_str().parse::<u8>().unwrap_or(1)-1),
                                           col_match(res.get(3).map_or(String::from(""),
                                                                       |m| m.as_str().to_lowercase())))));
        }
    }
}

fn col_match(string: String) -> u8 {
    let string = string.as_str();
    match string {
        "a" => 0,
        "b" => 1,
        "c" => 2,
        "d" => 3,
        "e" => 4,
        "f" => 5,
        "g" => 6,
        "h" => 7,
        _ => 0,
    }
}

///Placeholder sandbox testing function
pub fn example() {
    let string = String::from("4a5>112");
    ptn_move(&String::from("b5"));
    ptn_move(&string);
    ptn_move(&String::from("1A5>112"));
    ptn_move(&String::from("SA5"));
    ptn_move(&String::from("hello world 4a5>112"));
    let p1 = Player {
        color: Color::White,
        pieces: 0,
        caps: 0,
    };
    let p2 = Player {
        color: Color::Black,
        pieces: 0,
        caps: 0,
    };
    let r = StandardRules::new(State::new(5, p1, p2));
    let mut game = Game::new(r, StandardOpening {});

    let vec = vec![String::from("a1"), String::from("a2"), String::from("a3")];
    for m in vec {
        game.rules.make_move(ptn_move(&m).unwrap(), Color::White);
    }
//    game.rules.make_move(ptn_move(&String::from("c3")).unwrap());
//    game.rules.make_move(ptn_move(&String::from("a1")).unwrap());
//    let b = game.rules.make_move(ptn_move(&String::from("1c3-")).unwrap());
    game.rules.make_move(ptn_move(&String::from("a1+1")).unwrap(), Color::White);
    game.rules.make_move(ptn_move(&String::from("a3-")).unwrap(), Color::White);
    game.rules.make_move(ptn_move(&String::from("2a2>11")).unwrap(), Color::White);
    game.rules.make_move(ptn_move(&String::from("a5")).unwrap(), Color::Black);

    println!("{:?}", game.rules.state.board);
//    let vic = game.rules.check_win();
//    println!("{:?}", vic);
}