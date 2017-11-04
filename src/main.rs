#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(test)]

extern crate ndarray;
extern crate test;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate sqlite;

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
        let r = StandardRules::new(State::new(5));
        let mut game = Game::new(r, StandardOpening {});
        for x in 1..game.rules.state.size {
            game.rules.place_w_flat((0, x));
            game.rules.place_w_flat((1, x));
            game.rules.place_w_flat((2, x));
            game.rules.place_w_flat((3, x));
            game.rules.place_w_flat((4, x));
        }
        println!("\n{:?}", &game.rules.state.board);
        b.iter(|| game.rules.check_win(Color::White));
    }

    #[test]
    fn test_many_playtak_games() {
        for id in 220000..220586 { //220586
            let (mut moves, res, size) = game::database::get_playtak_game("games_anon.db", 220000);
            let r = StandardRules::new(State::new(size as u8));
            let mut game = Game::new(r, StandardOpening {});
            let last = moves.pop().unwrap();
            for m in moves.into_iter() {
                let attempt_move = game.read_move(m);
                assert!(attempt_move.0);
                assert_eq!(Victory::Neither, attempt_move.1);
            }
            let attempt_move = game.read_move(last);
            assert!(attempt_move.0);
            match res.as_ref() {
                "0-0" => assert_eq!(Victory::Neither, attempt_move.1),
                "F-0" => {
                    match attempt_move.1 {
                        Victory::White(x) => {assert_ne!(x, 0)}
                        _ => {assert!(false)}
                    }
                },
                "R-0" => {
                    match attempt_move.1 {
                        Victory::White(x) => {assert_eq!(x, 0)}
                        _ => {assert!(false)}
                    }
                },
                "0-F" => {
                    match attempt_move.1 {
                        Victory::Black(x) => {assert_ne!(x, 0)}
                        _ => {assert!(false)}
                    }
                },
                "0-R" => {
                    match attempt_move.1 {
                        Victory::Black(x) => {assert_eq!(x, 0)}
                        _ => {assert!(false)}
                    }
                },
                "1/2-1/2" => assert_eq!(Victory::Draw, attempt_move.1),
                _ => assert!(false),
            }
        }
    }
}

