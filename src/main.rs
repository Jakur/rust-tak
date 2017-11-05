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
    fn test_illegal_cases() {
        fn execute<R, O>(game: &mut Game<R, O>, vec: Vec<&str>) where R: RuleSet, O: Opening {
            let moves = vec.into_iter().map(|m| ptn_move(m).unwrap());
            moves.for_each(|m| {game.execute_move(m); ()});
        }
        fn assert_illegal<R, O>(game: &mut Game<R, O>, str: &str) where R: RuleSet, O: Opening {
            assert!(!game.execute_move(ptn_move(str).unwrap()));
        }
        let r = StandardRules::new(State::new(5));
        let mut game = Game::new(r, StandardOpening {});
        execute(&mut game, vec!["a5", "a1", "b1", "c1", "b2", "c2", "b3", "c3", "Cb4", "Cb5"]);
        assert_illegal(&mut game, "b4+"); //Cap cannot flatten cap
        execute(&mut game, vec!["a3", "c3<", "b4-", "Sd3"]);
        assert_illegal(&mut game, "3b3>111"); //pass through wall
        assert_illegal(&mut game,"3b3>12"); //crush wall with more than one piece in hand
        assert_illegal(&mut game, "d3-"); //move piece active player doesn't control
        assert_illegal(&mut game, "a3<"); //move piece off board
        assert_illegal(&mut game, "Sd3"); //place on top of existing piece
        assert_illegal(&mut game, "Ce1"); //place cap that player doesn't have
    }

    #[test]
    fn test_many_playtak_games() {
        for id in 220500..220586 { //Verified 150k - 220586
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

