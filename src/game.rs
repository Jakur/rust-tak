use regex::Regex;

pub mod state;
pub mod rules;
pub mod database;

pub use self::rules::*;
pub use self::state::*;

use super::Error;
use ndarray::Array2;
pub trait TakGame {
    /// Attemps to perform all actions necessary to progress forward one ply
    fn do_ply(&mut self, m: Move) -> Result<Victory, Error> {
        self.make_move(m)?;
        Ok(self.check_win())
    }

    /// Attempts to make the specified move
    fn make_move(&mut self, m: Move) -> Result<Victory, Error>;

    /// Checks the victory status of the game
    fn check_win(&self) -> Victory;

    /// Whether or not the game is in the opening phase, the phase of the game
    /// where the rules behave differently than normal. In a standard game this
    /// corresponds to the first two plies
    fn is_opening(&self) -> bool {
        self.current_ply() < 2
    }

    /// The 0-indexed ply count of the game
    fn current_ply(&self) -> u32;

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

    fn get_state(&self) -> &State;

    fn get_mut_state(&self) -> &mut State;
}

pub struct Game {
    pub rules: Box<Rules>,
    pub ply: u32,
}

impl Game {
    ///Creates a new game, consuming a given rule set and opening
    pub fn new(rules: Box<Rules>) -> Game {
        Game { rules, ply: 0 }
    }
    ///Attempts to execute a given move. Returns a tuple containing first whether or not the move
    /// was successfully executed and second the victory condition of the board state.
    pub fn read_move(&mut self, m: Move) -> (bool, Victory) {
        if self.execute_move(m) {
            if self.rules.is_opening() {
                self.ply += 1;
                return (true, Victory::Neither);
            } else {
                self.ply += 1;
                return (true, self.rules.check_win(self.current_player_color()));
            }
        } else {
            return (false, Victory::Neither);
        }
    }
    fn execute_move(&mut self, m: Move) -> bool {
        self.rules.make_move(m).is_ok()
    }
    ///Returns the color of player whose move it is. Note that this may be distinct from the color
    /// of the piece which is being played, as in the opening for a standard game of Tak.
    pub fn current_player_color(&self) -> Color {
        self.rules.current_color()
    }
    ///Prints the board with the most relevant board state information
    pub fn print_board(&self) {
        println!("{}", self.rules.get_state());
        println!("--------------------\n");
    }

    pub fn get_state(&self) -> &State {
        self.rules.get_state()
    }

    pub fn get_mut_state(&mut self) -> &mut State {
        self.rules.get_mut_state()
    }

    pub fn get_board(&self) -> &Array2<Tile> {
        &self.rules.get_state().board
    }

    pub fn get_size(&self) -> usize {
        self.rules.get_state().size as usize
    }
    ///Returns the color of the next piece which will be played. This is not the same as current
    /// player color, which determines whether the white or black player has the right to move
    pub fn next_piece_color(&self) -> Color {
        self.rules.current_color()
    }
}

///Transforms a ptn string into a Move that can be understood by the server, or None if the given
/// string was deemed an invalid ptn string.
pub fn ptn_move(string: &str) -> Option<Move> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^(\d)?(?i)([CS])?([a-h])([1-8])(([<>+-])([1-8]+)?(\*)?)?$").unwrap();
    }
    let res = RE.captures(string);
    let res = match res {
        Some(x) => x,
        _ => return None,
    };
    match res.get(6) {
        //Directional symbol
        Some(d) => {
            //stack move
            let dir = String::from(d.as_str()).pop().unwrap_or('+');
            let vec = res.get(7).map_or("1", |m| m.as_str()); //1 for elision
            let vec: Vec<_> = vec
                .chars()
                .map(|c| c.to_digit(16).unwrap_or(8) as u8)
                .collect();
            return Some(Move::Throw(
                (
                    res.get(1)
                        .map_or(1, |x| x.as_str().parse::<u8>().unwrap_or(1)),
                    res.get(4)
                        .map_or(0, |m| m.as_str().parse::<u8>().unwrap_or(1) - 1),
                    col_match(
                        res.get(3)
                            .map_or(String::from(""), |m| m.as_str().to_lowercase()),
                    ),
                ),
                dir,
                vec,
                String::from(string),
            ));
        }
        None => {
            //place
            let kind = match res.get(2).map_or("", |m| m.as_str()) {
                "S" => PieceKind::Wall,
                "C" => PieceKind::Cap,
                "s" => PieceKind::Wall,
                "c" => PieceKind::Cap,
                _ => PieceKind::Flat,
            };
            return Some(Move::Place(
                kind,
                (
                    res.get(4)
                        .map_or(0, |m| m.as_str().parse::<u8>().unwrap_or(1) - 1),
                    col_match(
                        res.get(3)
                            .map_or(String::from(""), |m| m.as_str().to_lowercase()),
                    ),
                ),
                String::from(string),
            ));
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
pub fn make_standard_game(size: u8, komi: u32) -> Game {
    let r = StandardRules::new(State::new(size), komi);
    return Game::new(Box::new(r));
}