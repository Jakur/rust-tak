extern crate ndarray;
#[macro_use]
extern crate lazy_static;
extern crate regex;

#[cfg(test)]
extern crate sqlite;

pub mod game;

use failure::Error;

#[cfg(test)]
mod tests {

    use super::*;
    use game::*;
    use sqlite::Value;

    #[test]
    fn display_test() {
        let (moves, _res, size) = get_playtak_game("games_anon.db", 220000);
        let r = StandardRules::new(State::new(size as u8), 0);
        let mut game = Game::new(r, StandardOpening {});
        for m in moves.into_iter() {
            let attempt_move = game.read_move(m);
            assert!(attempt_move.0);
        }
        game.print_board();
    }

    #[test]
    fn search_bench() {
        // Todo fix benchmarks
        let r = StandardRules::new(State::new(5), 0);
        let mut game = Game::new(r, StandardOpening {});
        for x in 1..game.rules.state.size {
            game.rules.place_w_flat((0, x));
            game.rules.place_w_flat((1, x));
            game.rules.place_w_flat((2, x));
            game.rules.place_w_flat((3, x));
            game.rules.place_w_flat((4, x));
        }
        println!("\n{:?}", &game.rules.state.board);
        game.rules.check_win(Color::White);
    }

    #[test]
    fn test_size() {
        use std::mem::size_of;
        println!("{}", size_of::<Piece>());
        println!("{}", size_of::<Move>());
    }

    #[test]
    fn test_illegal_cases() {
        fn execute<R, O>(game: &mut Game<R, O>, vec: Vec<&str>)
        where
            R: RuleSet,
            O: Opening,
        {
            let moves = vec.into_iter().map(|m| ptn_move(m).unwrap());
            moves.for_each(|m| {
                game.read_move(m);
                ()
            });
        }
        fn assert_illegal<R, O>(game: &mut Game<R, O>, string: &str)
        where
            R: RuleSet,
            O: Opening,
        {
            assert!(!game.read_move(ptn_move(string).unwrap()).0);
        }
        let r = StandardRules::new(State::new(5), 0);
        let mut game = Game::new(r, StandardOpening {});
        execute(
            &mut game,
            vec!["a5", "a1", "b1", "c1", "b2", "c2", "b3", "c3", "Cb4", "Cb5"],
        );
        assert_illegal(&mut game, "b4+"); //Cap cannot flatten cap
        execute(&mut game, vec!["a3", "c3<", "b4-", "Sd3"]);
        game.print_board();
        assert_illegal(&mut game, "3b3>111"); //pass through wall
        assert_illegal(&mut game, "3b3>12"); //crush wall with more than one piece in hand
        assert_illegal(&mut game, "d3-"); //move piece active player doesn't control
        assert_illegal(&mut game, "a3<"); //move piece off board
        assert_illegal(&mut game, "Sd3"); //place on top of existing piece
        assert_illegal(&mut game, "Ce1"); //place cap that player doesn't have
    }

    #[test]
    fn test_many_playtak_games() {
        for _id in 220000..220586 {
            //Verified 150k - 220586
            let (mut moves, res, size) = get_playtak_game("games_anon.db", 220000);
            let r = StandardRules::new(State::new(size as u8), 0);
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
                "F-0" => match attempt_move.1 {
                    Victory::White(x) => assert_ne!(x, 0),
                    _ => assert!(false),
                },
                "R-0" => match attempt_move.1 {
                    Victory::White(x) => assert_eq!(x, 0),
                    _ => assert!(false),
                },
                "0-F" => match attempt_move.1 {
                    Victory::Black(x) => assert_ne!(x, 0),
                    _ => assert!(false),
                },
                "0-R" => match attempt_move.1 {
                    Victory::Black(x) => assert_eq!(x, 0),
                    _ => assert!(false),
                },
                "1/2-1/2" => assert_eq!(Victory::Draw, attempt_move.1),
                _ => assert!(false),
            }
        }
    }

    #[test]
    fn test_crush() {
        let ptn_moves = vec![
            "a5", "a1", "b1", "Sb5", "Cc2", "e5", "b3", "b2", "b3-", "d4", "c2<", "c5",
        ];
        let game_moves = ptn_moves
            .into_iter()
            .map(|m| game::ptn_move(m).expect("Valid ptn"));
        let mut game = make_standard_game(5, 0);
        for m in game_moves {
            let (valid, victory) = game.read_move(m);
            assert!(valid);
            assert_eq!(victory, Victory::Neither);
        }
        let crush = game::ptn_move("3b2+111").expect("Valid ptn");
        println!("{:?}", crush);
        let (valid, victory) = game.read_move(crush);
        assert!(valid);
        assert_eq!(victory, Victory::Neither);
    }

    ///Reads a single game from a playtak database, returning the moves and the end of game state, e.g.
    /// F-0. This is used for testing purposes only and, as such, data is assumed to be valid.
    fn get_playtak_game(file: &str, id: i64) -> (Vec<Move>, String, usize) {
        let connection = sqlite::open(file).unwrap();
        let mut cursor = connection
            .prepare("SELECT size, notation, result FROM games WHERE id = ?")
            .unwrap()
            .cursor();
        cursor.bind(&[Value::Integer(id)]).unwrap();

        if let Some(row) = cursor.next().unwrap() {
            let size = row[0].as_integer().unwrap() as usize;
            let server_notation: &str = row[1].as_string().unwrap();
            return (
                game::database::decode_playtak_notation(server_notation),
                String::from(row[2].as_string().unwrap()),
                size,
            );
        } else {
            return (Vec::new(), String::from("0-0"), 5);
        }
    }
}

