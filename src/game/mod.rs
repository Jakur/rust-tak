use regex::Regex;

mod database;
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

pub fn ptn_move(string: &str) -> Option<Move> {
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

pub fn make_standard_game(size: usize) -> Game<StandardRules, StandardOpening> {
    let p1 = Player {
        color: Color::White,
        pieces: 21,
        caps: 1,
    };
    let p2 = Player {
        color: Color::Black,
        pieces: 21,
        caps: 1,
    };
    let r = StandardRules::new(State::new(5, p1, p2));
    return Game::new(r, StandardOpening {})
}

///Placeholder sandbox testing function
pub fn example() {
//    let string = String::from("4a5>112");
//    ptn_move(&String::from("b5"));
//    ptn_move(&string);
//    ptn_move(&String::from("1A5>112"));
//    ptn_move(&String::from("SA5"));
//    ptn_move(&String::from("hello world 4a5>112"));
    let p1 = Player {
        color: Color::White,
        pieces: 21,
        caps: 1,
    };
    let p2 = Player {
        color: Color::Black,
        pieces: 21,
        caps: 1,
    };

//    let vec = vec![String::from("a1"), String::from("a2"), String::from("a3")];
//    for m in vec {
//        game.rules.make_move(ptn_move(&m).unwrap(), Color::White);
//    }
//    game.rules.make_move(ptn_move(&String::from("c3")).unwrap());
//    game.rules.make_move(ptn_move(&String::from("a1")).unwrap());
//    let b = game.rules.make_move(ptn_move(&String::from("1c3-")).unwrap());
//    game.rules.make_move(ptn_move(&String::from("a1+1")).unwrap(), Color::White);
//    game.rules.make_move(ptn_move(&String::from("a3-")).unwrap(), Color::White);
//    game.rules.make_move(ptn_move(&String::from("2a2>11")).unwrap(), Color::White);
//    game.rules.make_move(ptn_move(&String::from("a5")).unwrap(), Color::Black);
//    game.rules.place_w_flat((1, 1));
//    game.rules.place_b_flat((1, 1));
//    game.rules.place_w_wall((1, 1));
//    println!("{:?}", game.rules.state.board);
//    println!("------------------------");
//    game.rules.make_move(ptn_move(&String::from("3b2>111")).unwrap(), Color::White);
//
//    println!("{:?}", game.rules.state.board);
//    let ptn = database::read_ptn_file("game.ptn");
//    let res = database::read_formatted_ptn(ptn.unwrap());
//    let res = res.unwrap();
//    println!("{:?}", res.1);
//    let mut g = res.0;
//    for m in res.1 {
//        let output = g.read_move(m);
//        println!("Ply: {}", g.ply);
//        println!("{:?}", g.rules.state.board);
//        assert!(output.0);
//        println!("{:?}", output.1)
//    }
    let (mut moves, s, size) = database::get_playtak_game("games_anon.db", 220000);
    let r = StandardRules::new(State::new(size as u8, p1, p2));
    let mut game = Game::new(r, StandardOpening {});
    let last = moves.pop().unwrap();
    for m in moves.into_iter() {
//        println!("{:?}", m);
        let attempt_move = game.read_move(m);
        assert!(attempt_move.0);
        assert_eq!(Victory::Neither, attempt_move.1);
    }
    println!("Last move: {:?}", last);
    let attempt_move = game.read_move(last);
    assert!(attempt_move.0);
    assert_ne!(Victory::Neither, attempt_move.1);
    println!("Victory: {:?}, Board: \n{:?}", attempt_move.1, game.rules.state.board);
}