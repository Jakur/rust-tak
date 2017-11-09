use regex::Regex;

pub mod database;
pub mod rules;

pub use self::rules::*;

pub struct Game<R, O> where R: RuleSet, O: Opening {
    pub rules: R,
    pub opening: O,
    pub ply: u32,
}

impl<R, O> Game<R, O> where R: RuleSet, O: Opening {
    ///Creates a new game, consuming a given rule set and opening
    pub fn new(rules: R, opening: O) -> Game<R, O> {
        Game {
            rules,
            opening,
            ply: 0,
        }
    }
    ///Attempts to execute a given move. Returns a tuple containing first whether or not the move
    /// was successfully executed and second the victory condition of the board state.
    pub fn read_move(&mut self, m: Move) -> (bool, Victory) {
        if self.execute_move(m) {
            if self.opening.is_opening(&self) {
                self.ply += 1;
                return (true, Victory::Neither)
            } else {
                self.ply += 1;
                return (true, self.rules.check_win(self.current_color()))
            }
        } else {
            return (false, Victory::Neither)
        }
    }
    fn execute_move(&mut self, m: Move) -> bool {
        if self.opening.is_opening(&self) {
            match self.opening.legal_move(&self, &m) {
                Some(color) => {
                    if self.rules.make_move(m, color) {
                        return true
                    }
                    return false
                },
                _ => return false
            }
        } else {
            let color = self.current_color();
            if self.rules.make_move(m, color) {
                return true
            }
            return false
        }
    }
    ///Returns the color of player whose move it is. Note that this may be distinct from the color
    /// of the piece which is being played, as in the opening for a standard game of Tak.
    pub fn current_color(&self) -> Color {
        self.rules.current_color(self.ply)
    }
    ///Prints the board with the most relevant board state information
    pub fn print_board(&self) {
        println!("{}", self.rules.get_state());
        println!("--------------------\n");
    }
}

///Transforms a ptn string into a Move that can be understood by the server, or None if the given
/// string was deemed an invalid ptn string.
pub fn ptn_move(string: &str) -> Option<Move> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(\d)?(?i)([CS])?([a-h])([1-8])(([<>+-])([1-8]+)?(\*)?)?$").unwrap();
    }
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
                dir, vec, String::from(string)))
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
                                                                       |m| m.as_str().to_lowercase()))),
                                    String::from(string)));
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
///Creates a game with standard rules and a standard opening of the given size
pub fn make_standard_game(size: usize) -> Game<StandardRules, StandardOpening> {
    let r = StandardRules::new(State::new(5));
    return Game::new(r, StandardOpening {})
}