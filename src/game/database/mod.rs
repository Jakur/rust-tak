use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

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