use criterion::{criterion_group, criterion_main, Criterion};
use tetrizz::data::*;
use tetrizz::beam_search::*;
use tetrizz::eval::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let hold = Piece::S;
    let queue = vec![Piece::I,Piece::L,Piece::J,Piece::Z,Piece::O,Piece::T,Piece::I,Piece::L,Piece::T,Piece::J,Piece::O,Piece::Z,Piece::S,Piece::J,Piece::T,Piece::Z,Piece::S,Piece::I,Piece::L,Piece::O,Piece::O,Piece::J,Piece::S,Piece::I,Piece::T,Piece::Z,Piece::L,Piece::L,Piece::O,Piece::T,Piece::Z,Piece::J,Piece::I,Piece::S,Piece::O,Piece::I,Piece::L,Piece::S,Piece::Z,Piece::J,Piece::T,Piece::T,Piece::I,Piece::L,Piece::J,Piece::S,Piece::Z,Piece::O,Piece::L,Piece::Z,Piece::J,Piece::T,Piece::O,Piece::S,Piece::I,Piece::L,Piece::T,Piece::J,Piece::Z,Piece::O,Piece::S,Piece::I,Piece::Z,Piece::T,Piece::L,Piece::I,Piece::S,Piece::J,Piece::O,Piece::S,Piece::Z,Piece::O,Piece::T,Piece::J,Piece::I,Piece::L,Piece::J,Piece::L,Piece::T,Piece::I,Piece::Z,Piece::S,Piece::O,Piece::J,Piece::I,Piece::T,Piece::O,Piece::S,Piece::Z,Piece::L,Piece::T,Piece::S,Piece::I,Piece::O,Piece::Z,Piece::J,Piece::L,Piece::I,Piece::L,Piece::T,Piece::S,Piece::O,Piece::J,Piece::Z,Piece::O,Piece::Z,Piece::T,Piece::I,Piece::J,Piece::S,Piece::L,Piece::L,Piece::T,Piece::Z,Piece::O,Piece::S,Piece::I,Piece::J,Piece::L,Piece::T,Piece::Z,Piece::S,Piece::J,Piece::O,Piece::I,Piece::J,Piece::O,Piece::S,Piece::Z,Piece::T,Piece::L,Piece::I,Piece::T,Piece::L,Piece::I,Piece::J,Piece::O,Piece::S,Piece::Z];
    let eval = Eval::from([1.0; 14]);
    let mut game = Game::new(Some(hold));
    game.board.cols = [2047,1023,511,0,0,340,8062,16383,5119,2047];
    c.bench_function("beam search test", |b| b.iter(|| search(std::hint::black_box(&game), queue.clone(), &eval, std::hint::black_box(8), 1000)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);