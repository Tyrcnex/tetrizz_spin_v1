use tetrizz::data::*;
use tetrizz::beam_search::*;
use tetrizz::eval::Eval;

use rand::seq::SliceRandom;

fn gen_queue(bags: u32) -> (Piece, Vec<Piece>) {
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

fn append_queue(queue: &mut Vec<Piece>, bags: u32) {
    let mut rng = rand::rng();
    let bag = [Piece::I, Piece::J, Piece::L, Piece::O, Piece::S, Piece::T, Piece::Z];
    for _ in 0..bags {
        let mut new_bag = bag.to_vec();
        new_bag.shuffle(&mut rng);
        queue.extend(new_bag);
    }
}

fn main() {
    let mut rng = rand::rng();

    let (test_hold, mut test_queue) = gen_queue(4);

    // let eval = Eval::from([124.09421, -367.82962, 306.19385, -213.37228, 25.347483, -389.50592, -255.44745, 357.6906, -31.861994, 318.49466, 7.4310007, 197.0811, -248.70837, 401.57187]);
    // let eval = Eval::from([-103.72366, -55.162273, 197.23633, 315.8147, -314.1474, -406.53665, 165.95236, -47.005257, 229.73164, 267.24597, 6.9954114, -129.99783, 375.4554, 521.28204]);
    // let eval = Eval::from([159.40056, 148.4561, -53.97785, -237.30595, -272.18283, -355.8393, -77.61428, 106.276245, 266.36646, -68.92533, 173.36801, 344.66238, 70.006424, 667.7196]);
    let eval = Eval::from([-79.400375, -55.564907, -125.680145, -170.41902, 10.167948, -172.78625, -478.7291, 86.84883, 368.89203, 272.57874, 28.938646, -104.59018, -496.8832, 458.29822]);
    let mut game = Game::new(Some(test_hold));

    let mut all_locations: Vec<PieceLocation> = vec![];

    for p in 0..20000 {
        if test_queue.len() < 25 {
            append_queue(&mut test_queue, 4);
        }
        let loc = search(&game, test_queue.clone(), &eval, 13, 3000);
        all_locations.push(loc);

        let mut outstr: Vec<String> = vec![];
        for y in (0..20).rev() {
            let mut vstr = String::new();
            for x in 0..10 {
                vstr.push_str(
                    if (game.board.cols[x as usize] & (1 << y)) > 0 { "🟩" }
                    else if loc.blocks().iter().any(|(bx, by)| *bx == x && *by == y) {
                        if loc.spun { "🟨" }
                        else { "🟥" }
                    }
                    else { "⬜️" }
                );
            }
            outstr.push(vstr);
        }

        let mut queue5 = test_queue.clone();
        queue5.truncate(5);

        outstr[5]  += &format!("          b2b:            ⭐️ \x1b[1m{}\x1b[0m ⭐️ ({} pieces/b2b)", game.b2b, (p + 1) as f32 / game.b2b as f32);
        outstr[6]  += &format!("          pieces placed:    {:?}", p + 1);
        outstr[7]  += &format!("          board:            {:?}", game.board.cols);
        outstr[8]  += &format!("          queue (next 5):   {:?}", queue5);
        outstr[9]  += &format!("          hold piece:       {:?}", game.hold);
        
        outstr[11] += &format!("          placed piece:     {:?}", loc.piece);

        println!("\n\n\n\n\n\n\n\n{}", outstr.join("\n"));

        game.advance(test_queue[0], loc);
        // if rng.random_bool(0.05) {
        //     let col = rng.random_range(0..10);
        //     let shift = rng.random_range(1..5) + (if rng.random_bool(0.03) { rng.random_range(1..5) } else { 0 });
        //     for x in 0..10 {
        //         if x == col {
        //             game.board.cols[x] <<= shift;
        //             continue;
        //         }
        //         game.board.cols[x] = !(!game.board.cols[x] << shift);
        //     }
        // }
        if loc.piece == game.hold {
            game.hold = test_queue[0]
        }

        test_queue.remove(0);
        if game.board.cols.iter().map(|col| 64 - col.leading_zeros()).max().unwrap() > 18 {
            break;
        }
        // std::thread::sleep(std::time::Duration::from_millis(200));
    }
    // println!("{}", serde_json::to_string(&all_locations).unwrap());
}