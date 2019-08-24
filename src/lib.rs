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
        let mut game = Game::new(Box::new(r));
        for m in moves.into_iter() {
            assert!(game.do_ply(m).is_ok());
        }
        game.print_board();
    }

    #[test]
    fn search_bench() {
        // Todo fix benchmarks
        let size = 5;
        let r = StandardRules::new(State::new(size), 0);
        let mut game = Game::new(Box::new(r));
        let mut place_w_flat = |index| {
            game.rules.get_mut_tile(index).add_piece(Piece {
                color: Color::White,
                kind: PieceKind::Flat,
            });
        };
        for x in 1..size {
            place_w_flat((0, x));
            place_w_flat((1, x));
            place_w_flat((2, x));
            place_w_flat((3, x));
            place_w_flat((4, x));
        }
        println!("\n{:?}", &game.rules.get_state().board);
        game.rules.check_win();
    }

    #[test]
    fn test_size() {
        use std::mem::size_of;
        println!("{}", size_of::<Piece>());
        println!("{}", size_of::<Move>())
    }

    #[test]
    fn test_illegal_cases() {
        fn execute(game: &mut Game, vec: Vec<&str>) -> Result<(), Error> {
            let moves = vec.into_iter().map(|m| ptn_move(m).unwrap());
            for m in moves {
                game.do_ply(m)?;
            }
            Ok(())
        }
        fn assert_illegal(game: &mut Game, string: &str) {
            assert!(!game.legal_move(ptn_move(string).unwrap()));
        }
        let r = StandardRules::new(State::new(5), 0);
        let mut game = Game::new(Box::new(r));
        execute(
            &mut game,
            vec!["a5", "a1", "b1", "c1", "b2", "c2", "b3", "c3", "Cb4", "Cb5"],
        )
        .unwrap();
        assert_illegal(&mut game, "b4+"); //Cap cannot flatten cap
        execute(&mut game, vec!["a3", "c3<", "b4-", "Sd3"]).unwrap();
        game.print_board();
        assert_illegal(&mut game, "3b3>111"); //pass through wall
        assert_illegal(&mut game, "3b3>12"); //crush wall with more than one piece in hand
                                             // Todo active player fix
                                             //assert_illegal(&mut game, "d3-"); //move piece active player doesn't control
        assert_illegal(&mut game, "a3<"); //move piece off board
        assert_illegal(&mut game, "Sd3"); //place on top of existing piece
        assert_illegal(&mut game, "Ce1"); //place cap that player doesn't have
                                          //game.do_ply(ptn_move("Sd3").unwrap());
                                          //game.print_board();
    }

    #[test]
    fn test_many_playtak_games() {
        for _id in 220000..220586 {
            //Verified 150k - 220586
            let (mut moves, res, size) = get_playtak_game("games_anon.db", 220000);
            let r = StandardRules::new(State::new(size as u8), 0);
            let mut game = Game::new(Box::new(r));
            let last = moves.pop().unwrap();
            for m in moves.into_iter() {
                let attempt_move = game.do_ply(m);
                assert!(attempt_move.is_ok());
                assert_eq!(Victory::Neither, attempt_move.unwrap());
            }
            let attempt_move = game.do_ply(last);
            assert!(attempt_move.is_ok());
            let victory_string = format!("{}", attempt_move.unwrap());
            assert_eq!(victory_string, res);
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
            let res = game.do_ply(m);
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), Victory::Neither);
        }
        let crush = game::ptn_move("3b2+111").expect("Valid ptn");
        println!("{:?}", crush);
        let res = game.do_ply(crush);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Victory::Neither);
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

