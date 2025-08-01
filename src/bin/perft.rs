use tetrizz::data::*;
use tetrizz::movegen::*;

fn perft(game: &Game, queue: &[Piece; 7], idx: usize, depth: usize) -> usize {
    if depth == 1 {
        return movegen_piece(&game.board, queue[idx]).len();
    }

    let mut nodes = 0;
    for mv in movegen_piece(&game.board, queue[idx]) {
        let mut next_game = game.clone();
        next_game.advance(queue[idx], mv);
        nodes += perft(&next_game, queue, idx + 1, depth - 1);
    }

    nodes
}

fn main() {
    let game = Game::new(None);
    let queue = [Piece::I, Piece::O, Piece::L, Piece::J, Piece::S, Piece::Z, Piece::T];

    for d in 3..=6 {
        let now = std::time::Instant::now();
        let nodes = perft(&game, &queue, 0, d);
        let elapsed = now.elapsed().as_micros() as usize;
        println!("Depth: {d}  |  Nodes: {nodes}  |  Time: {}ms  |  NPS: {}", elapsed as f32 / 1000.0, nodes as f32 / (elapsed as f32 / 1000000.0));
    }
}