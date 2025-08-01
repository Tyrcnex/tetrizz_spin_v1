use std::collections::BinaryHeap;

use crate::data::*;
use crate::eval::Eval;
use crate::movegen::movegen;

#[derive(Clone, Debug)]
pub struct Node {
    pub game: Game,
    pub id: usize,
    pub score: f32
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for Node {  }

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.score.partial_cmp(&self.score)
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub fn search(root: &Game, queue: Vec<Piece>, eval: &Eval, depth: usize, width: usize) -> PieceLocation {
    assert!(queue.len() > depth + 1);
    let search_loc = movegen(root, queue[0]);
    let mut heap: BinaryHeap<Node> = BinaryHeap::with_capacity(width + 1);
    for (id, &loc) in search_loc.iter().enumerate() {
        let mut game = root.clone();
        let placement_info = game.advance(queue[0], loc);
        let score = eval.eval(root, &game, &placement_info);
        insert_if_better(&mut heap, Node { game, id, score }, width);
    }
    let mut next: BinaryHeap<Node> = BinaryHeap::with_capacity(width + 1);
    for idx in 1..depth {
        for node in &heap {
            for loc in movegen(&node.game, queue[idx]) {
                let mut game = node.game.clone();
                let placement_info = game.advance(queue[idx], loc);
                if !game.can_spawn(queue[idx + 1]) {
                    continue;
                }
                let score = eval.eval(root, &game, &placement_info);
                insert_if_better(&mut next, Node { game, id: node.id, score }, width);
            }
        }
        if next.len() == 0 {
            break;
        }
        heap.clear();
        std::mem::swap(&mut heap, &mut next);
    }
    search_loc[heap.into_iter().min().unwrap().id]
}

fn insert_if_better(heap: &mut BinaryHeap<Node>, node: Node, width: usize) {
    if heap.len() < width {
        heap.push(node);
    } else if let Some(worst) = heap.peek() {
        if node.score > worst.score {
            heap.pop();
            heap.push(node);
        }
    }
}