#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(test)]

extern crate ndarray;
extern crate test;
#[macro_use] extern crate lazy_static;
extern crate regex;

mod game;
use game::*;

fn main() {
    game::example();
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn search_bench(b: &mut Bencher) {
        use game::Color;
        use game::Player;
        use game::RuleSet;
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
        for x in 1..game.rules.state.size {
            game.rules.place_w_flat((0, x));
            game.rules.place_w_flat((1, x));
            game.rules.place_w_flat((2, x));
            game.rules.place_w_flat((3, x));
            game.rules.place_w_flat((4, x));
        }
        println!("\n{:?}", &game.rules.state.board);
        b.iter(|| game.rules.check_win());
    }
}

