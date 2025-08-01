pub mod data;
pub mod beam_search;
pub mod eval;
pub mod movegen;
pub mod genetic;

#[cfg(test)]
mod tests {
    use super::data::*;
    use super::beam_search::*;
    use super::movegen::*;
    use super::eval::Eval;
    use itertools::Itertools;
    use rand::prelude::SliceRandom;

    fn gen_queue(bags: u8) -> (Piece, Vec<Piece>) {
        let mut rng = rand::rng();
        let bag = [Piece::I, Piece::J, Piece::L, Piece::O, Piece::S, Piece::T, Piece::Z];
        let mut queue: Vec<Piece> = vec![];
        for _ in 0..bags {
            let mut new_bag = bag.to_vec();
            new_bag.shuffle(&mut rng);
            queue.extend(new_bag);
        }
        (queue.remove(0), queue)
    }

    #[test]
    fn test_key_gen() {
        let mut game = Game::new(None);
        game.board.cols = [131071,32767,16383,8191,4095,1920,16383,131071,39039,16383];
        let loc = PieceLocation {
            x: 8,
            y: 14,
            piece: Piece::Z,
            rotation: Rotation::Right,
            spun: true,
            possible_line_clear: false
        };
        println!("{:?}", keypress_generation(&game.board, loc));
    }

    // #[ignore]
    // #[test]
    // fn test_agent() {
    //     let (test_hold, mut test_queue) = gen_queue(50);
    //     let eval = Eval::from([0.14436796, -0.53481585, 0.02416946, 0.63493365, -0.20837273, 0.42671585, 0.2527833]);
    //     let mut game = Game::new(Some(test_hold));
    //     for _ in 0..20 {
    //         let loc = search(&game, test_queue.clone(), &eval, 10, 10);
    //         game.advance(test_queue[0], loc);
    //         if loc.piece == game.hold {
    //             game.hold = test_queue[0]
    //         }
    //         test_queue.remove(0);
    //         if game.board.cols.iter().map(|col| 64 - col.leading_zeros()).max().unwrap() > 15 {
    //             break;
    //         }
    //         println!("{:?}", game);
    //     }
    // }

    // #[ignore]
    // #[test]
    // fn test_search() {
    //     let mut game = Game::new(None);
    //     game.board.cols = [2047,1023,511,0,0,340,8062,16383,5119,2047];
    //     let (hold, queue) = gen_queue(5);
    //     let eval = Eval::from([-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 2.0]);
    //     let loc = search(&game, queue, &eval, 4, 10);
    // }

    // #[test]
    // fn movegen() {
    //     let mut game = Game::new(None);
    //     game.board.cols = [2047,1023,511,0,0,340,8062,16383,5119,2047];
    //     println!("{:?}", movegen_piece(&game.board, Piece::Z).iter().map(|x| format!("({},{},{}),", x.x, x.y, x.rotation as i8 + (if x.spun { 100 } else { 0 }))).collect::<String>());
    //     return;
    // }
}