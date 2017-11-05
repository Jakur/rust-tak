use regex::Regex;

pub mod database;
mod rules;

pub use self::rules::*;

pub struct Game<R, O> where R: RuleSet, O: Opening {
    pub rules: R,
    pub opening: O,
    pub ply: u32,
}

impl<R, O> Game<R, O> where R: RuleSet, O: Opening {
    pub fn new(rules: R, opening: O) -> Game<R, O> {
        Game { //Todo Store ptn
            rules,
            opening,
            ply: 0,
        }
    }
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
    pub fn execute_move(&mut self, m: Move) -> bool {
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
            let color = {
                if self.ply % 2 == 0 {
                    Color::White
                } else {
                    Color::Black
                }
            };
            if self.rules.make_move(m, color) {
                return true
            }
            return false
        }
    }
    pub fn current_color(&self) -> Color {
        if self.opening.is_opening(&self) {
            self.opening.current_color(&self)
        } else {
            self.rules.current_color(self.ply)
        }
    }
}

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

pub fn make_standard_game(size: usize) -> Game<StandardRules, StandardOpening> {
    let r = StandardRules::new(State::new(5));
    return Game::new(r, StandardOpening {})
}

///Placeholder sandbox testing function
pub fn example() {

    let (mut moves, res, size) = database::get_playtak_game("games_anon.db", 220000);
    let r = StandardRules::new(State::new(size as u8));
    let mut game = Game::new(r, StandardOpening {});
    let last = moves.pop().unwrap();
    for m in moves.into_iter() {
        let attempt_move = game.read_move(m);
        assert!(attempt_move.0);
        assert_eq!(Victory::Neither, attempt_move.1);
    }
    println!("Last move: {:?}", last);
    let attempt_move = game.read_move(last);
    assert!(attempt_move.0);
    if res != "0-0" {
        assert_ne!(Victory::Neither, attempt_move.1);
    }
    println!("Victory: {:?}", attempt_move.1);
}