use tetrizz::beam_search::*;
use tetrizz::data::*;
use tetrizz::eval::Eval;
use tetrizz::movegen::*;

use std::io::{self};

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OutObj {
    keys: Vec<Key>,
    desired_location: PieceLocation,
}

#[derive(Deserialize)]
struct InObj {
    game: Game,
    queue: Vec<Piece>,
    beam_width: usize,
    beam_depth: usize,
}

fn main() {
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .ok()
            .expect("lol wtf is this");

        let parsed: InObj = serde_json::from_str(&input).unwrap();

        let eval = Eval::from([
            -79.400375,
            -55.564907,
            -125.680145,
            -170.41902,
            10.167948,
            -172.78625,
            -478.7291,
            86.84883,
            368.89203,
            272.57874,
            28.938646,
            -104.59018,
            -496.8832,
            458.29822,
        ]);
        // let eval = Eval::from([159.40056, 148.4561, -53.97785, -237.30595, -272.18283, -355.8393, -77.61428, 106.276245, 266.36646, -68.92533, 173.36801, 344.66238, 70.006424, 667.7196]);
        // Eval::from([-20.0, -50.0, -100.0, -100.0, -100.0, -5.0, 20.0, -10.0, -15.0, -5.0, -10.0, 100.0, -20.0, 300.0]);
        // let eval = Eval::from([-103.72366, -55.162273, 197.23633, 315.8147, -314.1474, -406.53665, 165.95236, -47.005257, 229.73164, 267.24597, 6.9954114, -129.99783, 375.4554, 521.28204]);

        if !parsed.game.can_spawn(parsed.queue[0]) {
            println!(
                "{}",
                serde_json::to_string(&OutObj {
                    keys: vec![Key::hardDrop],
                    desired_location: PieceLocation {
                        piece: parsed.queue[0],
                        rotation: Rotation::Up,
                        spun: false,
                        x: 4,
                        y: 21,
                        possible_line_clear: false
                    }
                })
                .unwrap()
            );
            return;
        }

        let found_move = search(
            &parsed.game,
            parsed.queue.clone(),
            &eval,
            parsed.beam_depth,
            parsed.beam_width,
        );
        let (mut keys, keynodes) = keypress_generation(&parsed.game.board, found_move.clone());

        if found_move.piece == parsed.game.hold {
            keys.insert(0, Key::hold);
        }

        keys.push(Key::hardDrop);

        println!(
            "{}",
            serde_json::to_string(&OutObj {
                keys,
                desired_location: found_move
            })
            .unwrap()
        )
    }
}
