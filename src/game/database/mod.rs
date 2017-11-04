use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use sqlite;
use sqlite::Value;

use super::Move;
use super::Game;
use super::StandardRules;
use super::StandardOpening;

pub fn read_formatted_ptn(string: String) -> Option<(Game<StandardRules, StandardOpening>, Vec<Move>)> {
    let mut game: Option<Game<StandardRules, StandardOpening>> = None;
    let mut vec = Vec::new();
    for s in string.lines() {
        if s.starts_with("[") { //Game information lines
            if s.starts_with("[Size ") {
                let v: Vec<&str> = s.split("\"").collect();
                let num = v[1].parse::<usize>().unwrap();
                game = Some(super::make_standard_game(num));
            }
            continue;
        } else if s.len() < 1 { //Ignore blank lines
            continue;
        }
        let split_line: Vec<&str> = s.split_whitespace().collect();
        vec.push(super::ptn_move(split_line[1]).unwrap());
        if split_line.len() > 2 {
            vec.push(super::ptn_move(split_line[2]).unwrap())
        }
    }
    match game {
        Some(g) => return Some((g, vec)),
        _ => return None,
    }
}

pub fn read_ptn_file(name_string: &str) -> Result<String, Box<Error>> {
    let mut f = File::open(name_string)?;
    let mut out_string = String::new();
    f.read_to_string(&mut out_string)?;
    return Ok(out_string)
}

///Reads a single game from a playtak database, returning the moves and the end of game state, e.g.
/// F-0. This is used for testing purposes only and, as such, data is assumed to be valid.
pub fn get_playtak_game(file: &str, id: i64) -> (Vec<Move>, String, usize) {
    let connection = sqlite::open(file).unwrap();
    let mut cursor = connection.prepare("SELECT size, notation, result FROM games WHERE id = ?")
        .unwrap().cursor();
    cursor.bind(&[Value::Integer(id)]).unwrap();

    if let Some(row) = cursor.next().unwrap() {
        let size = row[0].as_integer().unwrap() as usize;
        let server_notation: &str = row[1].as_string().unwrap();
        return (decode_playtak_notation(server_notation),
                String::from(row[2].as_string().unwrap()), size)
    } else {
        return (Vec::new(), String::from(""), 5);
    }
}

fn decode_playtak_notation(str: &str) -> Vec<Move> {
    let moves = str.split(",");
    let mut vec = Vec::new();
    for m in moves {
        let transformed = transform_notation(m);
        vec.push(transformed);
    }
    let vec: Vec<_> = vec.into_iter().map(|m| super::ptn_move(&m).unwrap()).collect();
    vec
}

fn transform_notation(str: &str) -> String {
    let split_move: Vec<_> = str.split_whitespace().collect();
    match split_move[0] {
        "P" => {
            if split_move.len() <= 2 {
                return String::from(split_move[1].to_lowercase())
            } else {
                let mut s = {
                    if split_move[2] == "C" {String::from("C")} else {String::from("S")}
                };
                s.push_str(&split_move[1].to_lowercase());
                return s;
            }
        }
        "M" => {
            let source: Vec<_> = split_move[1].chars().collect();
            let dest: Vec<_> = split_move[2].chars().collect();
            let direction = {
                let col_dif = decode_column(source[0]) - decode_column(dest[0]);
                if col_dif < 0 {
                    ">"
                } else if col_dif > 0 {
                    "<"
                } else {
                    let row_dif = source[1].to_digit(10).unwrap() as i32 -
                        dest[1].to_digit(10).unwrap() as i32;
                    if row_dif > 0 {
                        "-"
                    } else {
                        "+"
                    }
                }
            };
            let mut res_string = String::from(split_move[1]);
            res_string.push_str(direction);
            let mut picked_up = 0;
            for i in 3..split_move.len() {
                res_string.push_str(split_move[i]);
                picked_up += split_move[i].parse::<u32>().unwrap();
            }
            let mut result = picked_up.to_string();
            result.push_str(&res_string);
            return result
        }
        _ => {return String::from("")}
    }
}

fn decode_column(ch: char) -> i32 {
    match ch {
        'A' => 1,
        'B' => 2,
        'C' => 3,
        'D' => 4,
        'E' => 5,
        'F' => 6,
        _ => 0,
    }
}