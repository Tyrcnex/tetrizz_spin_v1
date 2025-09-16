use crate::data::*;
use crate::beam_search::*;
use crate::eval::Eval;

use rand::seq::SliceRandom;

use rand::Rng;
use rand::prelude::IteratorRandom;
use rayon::prelude::*;

use std::sync::atomic::{AtomicU32, Ordering};
use std::io::Write;

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

pub fn eval_fitness(queue: Vec<Piece>, hold: Piece, weights: [f32; 14]) -> f32 {
    let mut rng = rand::rng();

    const GAMES_PLAYED: usize = 4;
    const MOVES_MADE: usize = 500;

    let mut fitnesses: Vec<f32> = vec![];
    let eval = Eval::from(weights);
    for _ in 0..GAMES_PLAYED {
        let mut test_queue = queue.clone();
        let test_hold = hold.clone();
        let mut game = Game::new(Some(test_hold));
        let mut max: u64 = 0;
        let mut predicted_attack = 0;
        let mut predicted_surge = 0;
        let mut pieces_placed = 0;
        for ii in 0..MOVES_MADE {
            let loc = search(&game, test_queue.clone(), &eval, 15, 3000, predicted_attack);




            // let mut outstr: Vec<String> = vec![];
            // for y in (0..20).rev() {
            //     let mut vstr = String::new();
            //     for x in 0..10 {
            //         vstr.push_str(
            //             if (game.board.cols[x as usize] & (1 << y)) > 0 { "🟩" }
            //             else if loc.blocks().iter().any(|(bx, by)| *bx == x && *by == y) {
            //                 if loc.spun { "🟨" }
            //                 else { "🟥" }
            //             }
            //             else { "⬜️" }
            //         );
            //     }
            //     outstr.push(vstr);
            // }

            // let mut queue5 = test_queue.clone();
            // queue5.truncate(5);

            // outstr[5]  += &format!("          b2b:            ⭐️ \x1b[1m{}\x1b[0m ⭐️ ({} pieces/b2b)", game.b2b, (ii + 1) as f32 / game.b2b as f32);
            // outstr[6]  += &format!("          pieces placed:    {:?}", ii + 1);
            // outstr[7]  += &format!("          board:            {:?}", game.board.cols);
            // outstr[8]  += &format!("          queue (next 5):   {:?}", queue5);
            // outstr[9]  += &format!("          hold piece:       {:?}", game.hold);
            // outstr[10] += &format!("          predicted surge:  {:?}", predicted_surge);
            // outstr[11] += &format!("          predicted attack: {:?}", t_attack);
            
            // outstr[13] += &format!("          placed piece:     {:?}", loc.piece);

            // println!("\n\n\n\n\n\n\n\n{}", outstr.join("\n"));







            let info = game.advance(test_queue[0], loc);
            if loc.piece == game.hold {
                game.hold = test_queue[0];
            }
            if ii % 3 == 0 {
                predicted_surge += 1;
            }

            let difficulty = (ii as f64) / 3000.0;
            if rng.random_bool(0.05 + difficulty) {
                let t = rng.random_range(1..=2);
                predicted_attack += t;
            } else if rng.random_bool(2.0 * (0.05 + difficulty)) {
                let t = rng.random_range(3..=4);
                predicted_attack += t;
            }

            if rng.random_bool(0.01) {
                predicted_attack += predicted_surge;
                predicted_surge = 0;
            }

            if info.lines_cleared == 0 && predicted_attack > 0 {
                let col = rng.random_range(0..10);
                let shift = predicted_attack.min(8);
                for x in 0..10 {
                    if x == col {
                        game.board.cols[x] <<= shift;
                        continue;
                    }
                    game.board.cols[x] = !(!game.board.cols[x] << shift);
                }
                predicted_attack -= shift;
            }
            predicted_attack = predicted_attack.max(info.attack) - info.attack;

            test_queue.remove(0);

            pieces_placed = ii;

            if game.board.cols.iter().map(|col| 64 - col.leading_zeros()).max().unwrap() > 15 {
                break;
            }
            if game.b2b > max {
                max = game.b2b;
            }
        }
        fitnesses.push(max as f32 * pieces_placed as f32 / MOVES_MADE as f32);
        // fitnesses.push(pieces_placed as f32);
    }
    fitnesses.iter().sum::<f32>() / GAMES_PLAYED as f32
}

pub fn normalized(weights: [f32; 14]) -> [f32; 14] {
    let mag = weights.iter().fold(0.0, |a,b| a + b * b).sqrt() / 1000.0;
    weights.map(|x| x / mag)
}

#[derive(Clone, Debug)]
pub struct Agent {
    pub weights: [f32; 14],
    pub fitness: f32
}

impl Agent {
    fn new_random() -> Self {
        let mut rng = rand::rng();
        let mut arr = [0f32; 14];
        for x in &mut arr {
            *x = rng.random_range(-1.0..=1.0);
        }
        Self {
            weights: normalized(arr),
            fitness: 99999999.0
        }
    }

    fn combine(&self, other: &Self) -> Option<Self> {
        if self.fitness == 0.0 && other.fitness == 0.0 {
            return None;
        }
        let mut this_weights = self.weights;
        let other_weights = other.weights;
        for (a, &b) in this_weights.iter_mut().zip(other_weights.iter()) {
            *a += b;
        }
        Some(Self {
            weights: normalized(this_weights),
            fitness: 9999999.0
        })
    }
}

pub fn run_genetic_algo() {
    rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();

    const NUM_AGENTS: usize = 100;
    const GENETIC_ITERATIONS: usize = 50;
    const REPRODUCE: usize = 10;
    const MUTATE: usize = 70;
    const BATCH_POPULATION: usize = 20;

    let mut rng = rand::rng();
    let mut agents: Vec<Agent> = (0..NUM_AGENTS).map(|_| Agent::new_random()).collect();

    let mut best_agent: Agent = Agent::new_random();
    best_agent.fitness = 0.0;

    for n in 0..GENETIC_ITERATIONS {
        println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n🤩🤩🤩🤩🤩🤩🤩🤩\n🤩🤩🤩🤩🤩🤩🤩🤩\n🤩🤩🤩🤩🤩🤩🤩🤩\n🤩🤩🤩🤩🤩🤩🤩🤩\n\x1b[1mITERATION {}/{GENETIC_ITERATIONS}\x1b[0m", n + 1);
        let (hold, queue) = gen_queue(200);
        let started = AtomicU32::new(0);
        let completed = AtomicU32::new(0);
        agents.par_iter_mut()
            .for_each(|agent| {
                let start_prev = started.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| Some(x + 1)).unwrap();
                unsafe { 
                    let scol = if start_prev != NUM_AGENTS as u32 { "\x1b[1;33m" } else { "\x1b[1;32m" };
                    print!("   --- {scol}Started: {start_prev}/{NUM_AGENTS}\x1b[0m\t\t\x1b[1;33mCompleted: {}/{NUM_AGENTS}\x1b[0m\r", *completed.as_ptr());
                    let _ = std::io::stdout().flush();
                }
                agent.fitness = eval_fitness(queue.clone(), hold.clone(), agent.weights);
                let completed_prev = completed.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| Some(x + 1)).unwrap();
                unsafe {
                    let scol = if *started.as_ptr() != NUM_AGENTS as u32 { "\x1b[1;33m" } else { "\x1b[1;32m" };
                    print!("   --- {scol}Started: {}/{NUM_AGENTS}\x1b[0m\t\t\x1b[1;33mCompleted: {completed_prev}/{NUM_AGENTS}\x1b[0m\r", *started.as_ptr());
                    let _ = std::io::stdout().flush();
                }
            });

        best_agent = agents.iter().fold(best_agent, |a, b| if a.fitness > b.fitness { a } else { b.clone() });


        for _ in 0..REPRODUCE {
            let mut select_two_agents = agents.iter().choose_multiple(&mut rng, BATCH_POPULATION);
            select_two_agents.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
            let new_agent = select_two_agents[0].combine(select_two_agents[1]);
            if let Some(agent) = new_agent {
                agents.push(agent);
            }
        }

        for _ in 0..MUTATE {
            agents.push(Agent::new_random());
            // let mut select_two_agents = agents.iter().choose_multiple(&mut rng, BATCH_POPULATION);
            // select_two_agents.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
            // let mut new_agent = select_two_agents[0].clone();
            // for weight in &mut new_agent.weights {
            //     *weight += rng.random_range(-20.0..20.0);
            // }
            // agents.push(new_agent);
        }
        agents.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        agents = agents[0..NUM_AGENTS].iter().map(|x| if x.fitness == 0.0 { Agent::new_random() } else { x.clone() } ).collect::<Vec<Agent>>();

        println!("\x1b[1mAll current agents: \x1b[0m{:?}\n", agents);
        println!("\x1b[1mBest agent (from current queue): \x1b[0m{:?}\n", agents.iter().cloned().fold(Agent::new_random(), |a,b| if a.fitness > b.fitness { a } else { b }));
        println!("\x1b[1mBest agent: \x1b[0m{:?}", best_agent);
    }
}

// pub fn run_genetic_algo() {
//     let agent = Agent {
//         weights: [-79.400375, -55.564907, -125.680145, -170.41902, 10.167948, -172.78625, -478.7291, 86.84883, 368.89203, 272.57874, 28.938646, -104.59018, -496.8832, 458.29822],
//         fitness: 0.0
//     };
//     let (hold, queue) = gen_queue(200);
//     eval_fitness(queue.clone(), hold.clone(), agent.weights);
// }