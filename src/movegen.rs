use crate::data::*;
use std::fmt;
use std::collections::{HashSet};
use std::hash::{Hash, Hasher};
use itertools::Itertools;

use serde::Serialize;

const FULL_HEIGHT: u64 = (1 << 40) - 1;

pub fn movegen(game: &Game, next: Piece) -> Vec<PieceLocation> {
    let mut next_pieces = movegen_piece(&game.board, next);
    let mut hold_pieces = movegen_piece(&game.board, game.hold);
    next_pieces.append(&mut hold_pieces);
    next_pieces
}

pub fn movegen_piece(board: &Board, piece: Piece) -> Vec<PieceLocation> {
    const ROT: [Rotation; 4] = [Rotation::Up, Rotation::Right, Rotation::Down, Rotation::Left];
    const PAIRS: [[usize; 3]; 4] = [[1, 2, 3], [0, 2, 3], [0, 1, 3], [0, 1, 2]];

    let now = std::time::Instant::now();

    // println!("start {}", now.elapsed().as_nanos());

    let mut maps = ROT.map(|r| CollisionMap::new(board, piece, r));

    // println!("new map {}", now.elapsed().as_nanos());

    if piece != Piece::O {
        let mut completed = [false, false, false, false];

        while completed != [true, true, true, true] {
            for i2 in 0..4 {
                let last = maps[i2].explored;
                let all_valid = maps[i2].all_valid;

                // println!("found a filter {}", now.elapsed().as_nanos());
                if last == all_valid {
                    completed[i2] = true;
                    continue;
                }

                for i1 in PAIRS[i2] {
                    let kicks = kicks(piece, ROT[i1], ROT[i2]);
                    let mut p1f = maps[i1].explored;
                    for (kx, ky) in kicks {
                        let mut mask = all_valid;
                        for x in 0..10 {
                            let c = p1f.get((x - kx) as usize).copied().unwrap_or(0);
                            let c = match ky < 0 {
                                true => c >> -ky,
                                false => c << ky
                            };
                            mask[x as usize] &= c;
                            maps[i2].explored[x as usize] |= mask[x as usize];
                            maps[i2].spin_loc[x as usize] |= mask[x as usize];
                        }
                        for x in 0..10 {
                            let c = mask.get((x + kx) as usize).copied().unwrap_or(0);
                            let c = match ky < 0 {
                                true => c << -ky,
                                false => c >> ky
                            };
                            p1f[x as usize] &= !c;
                        }
                        // println!("yekick {}", now.elapsed().as_nanos());
                    }
                }
                // println!("spin {}", now.elapsed().as_nanos());
                maps[i2].floodfill();
                // println!("floodfill {}", now.elapsed().as_nanos());
                if maps[i2].explored == last {
                    completed[i2] = true;
                }
            }
        }
    }
    
    for map in &mut maps {
        for x in 0..10 {
            map.explored[x] &= map.obstructed[x] << 1 | 1;
            map.spin_loc[x] &= map.obstructed[x] << 1 | 1;
        }
    }

    // println!("{}\n\n{}\n\n{}\n\n{}", maps[0].clone(), maps[1].clone(), maps[2].clone(), maps[3].clone());

    let mut new_maps: Vec<CollisionMap> = maps.into_iter().collect();

    if piece == Piece::S || piece == Piece::Z {
        for x in 0..10 {
            // down to up
            new_maps[Rotation::Up as usize].explored[x] |= new_maps[Rotation::Down as usize].explored[x] >> 1;
            new_maps[Rotation::Up as usize].spin_loc[x] |= new_maps[Rotation::Down as usize].spin_loc[x] >> 1;
            // left to right
            new_maps[Rotation::Right as usize].explored[x] |= new_maps[Rotation::Left as usize].explored.get(x + 1).copied().unwrap_or(0);
            new_maps[Rotation::Right as usize].spin_loc[x] |= new_maps[Rotation::Left as usize].spin_loc.get(x + 1).copied().unwrap_or(0);
        }
        new_maps.truncate(2);
    } else if piece == Piece::I {
        for x in 0..10 {
            // down to up
            new_maps[Rotation::Up as usize].explored[x] |= new_maps[Rotation::Down as usize].explored.get(x + 1).copied().unwrap_or(0);
            new_maps[Rotation::Up as usize].spin_loc[x] |= new_maps[Rotation::Down as usize].spin_loc.get(x + 1).copied().unwrap_or(0);
            // left to right
            new_maps[Rotation::Right as usize].explored[x] |= new_maps[Rotation::Left as usize].explored[x] << 1;
            new_maps[Rotation::Right as usize].spin_loc[x] |= new_maps[Rotation::Left as usize].spin_loc[x] << 1;
        }
        new_maps.truncate(2);
    } else if piece == Piece::O {
        new_maps.truncate(1);
    }

    let actual_spin: Vec<[u64; 10]> = match piece {
        Piece::T => {
            let mut s = [0u64; 10];
            for x in 0..10 {
                let left = board.cols.get(x - 1).copied().unwrap_or(FULL_HEIGHT);
                let right = board.cols.get(x + 1).copied().unwrap_or(FULL_HEIGHT);
                
                let c1 = left << 1 | 1;
                let c2 = right << 1 | 1;
                let c3 = right >> 1;
                let c4 = left >> 1;

                s[x] =   (c1 & c2 & (c3 | c4))
                       | (c3 & c4 & (c1 | c2));
            }
            new_maps.iter().map(|map| [0,1,2,3,4,5,6,7,8,9].map(|x| 
                (s[x] 
                | (map.obstructed.get(x - 1).copied().unwrap_or(FULL_HEIGHT)
                   & map.obstructed.get(x + 1).copied().unwrap_or(FULL_HEIGHT)
                   & (map.obstructed[x] >> 1)))
                & map.spin_loc[x]
            )).collect()
        },
        _ => {
            new_maps.iter().map(|map| [0,1,2,3,4,5,6,7,8,9].map(|x| 
                map.obstructed.get(x - 1).copied().unwrap_or(FULL_HEIGHT)
                & map.obstructed.get(x + 1).copied().unwrap_or(FULL_HEIGHT)
                & (map.obstructed[x] >> 1)
                & map.spin_loc[x]
            )).collect()
        }
    };

    // println!("filtering {}", now.elapsed().as_nanos());

    let mut positions: Vec<PieceLocation> = Vec::with_capacity(40);
    for (rot_i, map) in new_maps.iter().enumerate() {
        for x in 0..10 {
            let mut remaining = map.explored[x as usize];
            let mut spin_remaining = actual_spin[rot_i][x as usize];
            
            let mut plc = remaining
                & map.obstructed.get((x - 1) as usize).copied().unwrap_or(FULL_HEIGHT)
                & map.obstructed.get((x + 1) as usize).copied().unwrap_or(FULL_HEIGHT);

            let mut y = 0;
            while remaining != 0 {
                if remaining & 1 == 1 {
                    let loc = PieceLocation {
                        piece,
                        rotation: ROT[rot_i],
                        spun: spin_remaining & 1 == 1,
                        possible_line_clear: plc & 1 == 1,
                        x, y
                    };
                    positions.push(loc);
                    // let mut new_board = board.clone();
                    // let info = new_board.place(loc);
                    // if info.lines_cleared == 0 || (info.spin && info.lines_cleared == 1) {
                    //     positions.push(loc);
                    // }
                }
                remaining >>= 1;
                spin_remaining >>= 1;
                plc >>= 1;
                y += 1;
            }
        }
    }

    // println!("all done {}", now.elapsed().as_nanos());

    positions
}

const fn kicks(piece: Piece, from: Rotation, to: Rotation) -> [(i8, i8); 6] {
    match piece {
        Piece::O => [(0, 0); 6], // just be careful not to rotate the O piece at all lol
        Piece::I => match (from, to) {
            (Rotation::Right, Rotation::Up) => [(-1, 0),(-2, 0),(1, 0),(-2, -2),(1, 1),(-1, 0)],
            (Rotation::Right, Rotation::Down) => [(0, -1),(-1, -1),(2, -1),(-1, 1),(2, -2),(0, -1)],
            (Rotation::Right, Rotation::Left) => [(-1, -1),(0, -1),(-1, -1),(-1, -1),(-1, -1),(-1, -1)],
            (Rotation::Down, Rotation::Up) => [(-1, 1),(-1, 0),(-1, 1),(-1, 1),(-1, 1),(-1, 1)],
            (Rotation::Down, Rotation::Right) => [(0, 1),(-2, 1),(1, 1),(-2, 2),(1, -1),(0, 1)],
            (Rotation::Down, Rotation::Left) => [(-1, 0),(1, 0),(-2, 0),(1, 1),(-2, -2),(-1, 0)],
            (Rotation::Left, Rotation::Up) => [(0, 1),(1, 1),(-2, 1),(1, -1),(-2, 2),(0, 1)],
            (Rotation::Left, Rotation::Right) => [(1, 1),(0, 1),(1, 1),(1, 1),(1, 1),(1, 1)],
            (Rotation::Left, Rotation::Down) => [(1, 0),(2, 0),(-1, 0),(2, 2),(-1, -1),(1, 0)],
            (Rotation::Up, Rotation::Right) => [(1, 0),(2, 0),(-1, 0),(-1, -1),(2, 2),(1, 0)],
            (Rotation::Up, Rotation::Left) => [(0, -1),(-1, -1),(2, -1),(2, -2),(-1, 1),(0, -1)],
            (Rotation::Up, Rotation::Down) => [(1, -1),(1, 0),(1, -1),(1, -1),(1, -1),(1, -1)],
            _ => panic!() // this should never happen lol
        },
        _ => match (from, to) {
            (Rotation::Right, Rotation::Up) => [(0, 0),(1, 0),(1, -1),(0, 2),(1, 2),(0, 0)],
            (Rotation::Right, Rotation::Down) => [(0, 0),(1, 0),(1, -1),(0, 2),(1, 2),(0, 0)],
            (Rotation::Right, Rotation::Left) => [(0, 0),(1, 0),(1, 2),(1, 1),(0, 2),(0, 1)],
            (Rotation::Down, Rotation::Up) => [(0, 0),(0, -1),(-1, -1),(1, -1),(-1, 0),(1, 0)],
            (Rotation::Down, Rotation::Right) => [(0, 0),(-1, 0),(-1, 1),(0, -2),(-1, -2),(0, 0)],
            (Rotation::Down, Rotation::Left) => [(0, 0),(1, 0),(1, 1),(0, -2),(1, -2),(0, 0)],
            (Rotation::Left, Rotation::Up) => [(0, 0),(-1, 0),(-1, -1),(0, 2),(-1, 2),(0, 0)],
            (Rotation::Left, Rotation::Right) => [(0, 0),(-1, 0),(-1, 2),(-1, 1),(0, 2),(0, 1)],
            (Rotation::Left, Rotation::Down) => [(0, 0),(-1, 0),(-1, -1),(0, 2),(-1, 2),(0, 0)],
            (Rotation::Up, Rotation::Right) => [(0, 0),(-1, 0),(-1, 1),(0, -2),(-1, -2),(0, 0)],
            (Rotation::Up, Rotation::Left) => [(0, 0),(1, 0),(1, 1),(0, -2),(1, -2),(0, 0)],
            (Rotation::Up, Rotation::Down) => [(0, 0),(0, 1),(1, 1),(-1, 1),(1, 0),(-1, 0)],
            _ => panic!() // this should never happen lol
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollisionMap {
    pub obstructed: [u64; 10],
    pub all_valid: [u64; 10],
    pub explored: [u64; 10],
    pub spin_loc: [u64; 10]
}

impl CollisionMap {
    pub fn new(board: &Board, piece: Piece, rotation: Rotation) -> Self {
        let mut obstructed = [0u64; 10];
        for (dx, dy) in rotation.rotate_blocks(piece.blocks()) {
            for x in 0..10 {
                let c = board.cols.get((x + dx) as usize).copied().unwrap_or(FULL_HEIGHT);
                let c = match dy < 0 {
                    true => !(!c << -dy),
                    false => c >> dy
                };
                obstructed[x as usize] |= c;
            }
        }

        let max_height = board.cols.iter().map(|x| 64 - x.leading_zeros() as i8).max().unwrap();
        let mut all_valid: [u64; 10] = [(1 << (max_height + 3)) - 1; 10];
        let mut explored = [1 << (max_height + 2); 10];
        for x in 0..10 {
            all_valid[x] &= !obstructed[x];
            explored[x] &= !obstructed[x];
        }

        let mut res = Self {
            obstructed,
            all_valid,
            explored,
            spin_loc: [0u64; 10]
        };
        res.floodfill();
        res
    }

    fn floodfill(&mut self) -> [u64; 10] {
        let mut last = [0u64; 10];
        let mut res = self.explored;
        while last != res {
            last = res;
            for x in 0..10 {
                let obstr = self.obstructed[x] & ((1 << res[x].trailing_zeros()) - 1);
                res[x] |= ((1 << (64 - obstr.leading_zeros())) + (obstr == 0) as u64) & !self.obstructed[x];
                res[x] |= (res.get(x - 1).copied().unwrap_or(0) 
                         | res.get(x + 1).copied().unwrap_or(0)) 
                         & !self.obstructed[x];
            }
        }
        self.explored = res;
        res
    }
}

impl fmt::Display for CollisionMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let outstr = (0..20).rev().map(|x| {
            let explored = self.explored.iter().map(move |v| String::from(if (v & (1 << x)) == 0 { "⬜️" } else { "🟩" }))
            .collect::<Vec<String>>()
            .join("");
            let all_valid = self.all_valid.iter().map(move |v| String::from(if (v & (1 << x)) == 0 { "⬜️" } else { "🟩" }))
            .collect::<Vec<String>>()
            .join("");
            let obstructed = self.obstructed.iter().map(move |v| String::from(if (v & (1 << x)) == 0 { "⬜️" } else { "🟩" }))
            .collect::<Vec<String>>()
            .join("");
            let spin_loc = self.spin_loc.iter().map(move |v| String::from(if (v & (1 << x)) == 0 { "⬜️" } else { "🟩" }))
                .collect::<Vec<String>>()
                .join("");
            format!("{}     {}     {}     {}", obstructed, all_valid, explored, spin_loc)
        }).collect::<Vec<String>>();
        write!(f, "Obstructed               All valid                Explored                 Spin location\n{}", outstr.join("\n"))
    }
}








#[derive(Copy, Clone, Debug)]
pub struct KeyNode {
    pub loc: PieceLocation,
    pub id: u32,
    pub prev_id: u32
}

impl Hash for KeyNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.loc.hash(state);
    }
}

impl PartialEq for KeyNode {
    fn eq(&self, rhs: &Self) -> bool {
        self.loc == rhs.loc
    }
}

impl Eq for KeyNode {  }

#[derive(Debug)]
struct STUPIDCollisionMap {
    obstructed: [u64; 10]
}

impl STUPIDCollisionMap {
    fn new(board: &Board, piece: Piece, rotation: Rotation) -> Self {
        let mut map = Self { obstructed: [0u64; 10] };
        for (dx, dy) in rotation.rotate_blocks(piece.blocks()) {
            for x in 0..10 {
                let c = board.cols.get((x + dx) as usize).copied().unwrap_or(FULL_HEIGHT);
                let c = match dy < 0 {
                    true => !(!c << -dy),
                    false => c >> dy
                };
                map.obstructed[x as usize] |= c;
            }
        }
        map
    }

    fn obstr(&self, x: i8, y: i8) -> bool {
        if x < 0 || x > 9 || y < 0 {
            return true;
        }
        self.obstructed[x as usize] & (1 << y) > 0
    }
}

pub fn keypress_generation(board: &Board, loc: PieceLocation) -> (Vec<Key>, Vec<KeyNode>) {
    let loc = PieceLocation {
        possible_line_clear: false,
        ..loc
    };

    let collision_maps = [Rotation::Up, Rotation::Right, Rotation::Down, Rotation::Left].map(|rot| STUPIDCollisionMap::new(board, loc.piece, rot));

    let mut next_id: u32 = 1;

    let mut nodes_to_search: HashSet<KeyNode> = HashSet::from([KeyNode {
        loc: PieceLocation { x: 4, y: 21, piece: loc.piece, rotation: Rotation::Up, spun: false, possible_line_clear: false },
        id: 0,
        prev_id: 0
    }]);
    let mut visited_nodes: HashSet<KeyNode> = HashSet::new();

    let mut found_node: Option<KeyNode> = None;
    let mut found = false;

    while nodes_to_search.len() > 0 && !found {
        let mut new_nodes: HashSet<KeyNode> = HashSet::new();
        for node in &nodes_to_search {
            let mut node_new_locs: Vec<PieceLocation> = vec![
                PieceLocation { x: node.loc.x - 1, spun: false, ..node.loc },
                PieceLocation { x: node.loc.x + 1, spun: false, ..node.loc },
            ];
            let dist = board.distance_to_ground(node.loc);
            if dist > 0 {
                node_new_locs.push(PieceLocation { y: node.loc.y - dist, spun: false, ..node.loc });
            }
            for to in [
                node.loc.rotation.rotate_left(),
                node.loc.rotation.rotate_right(),
                node.loc.rotation.rotate_180()
            ] {
                for (kx, ky) in kicks(node.loc.piece, node.loc.rotation, to) {
                    if !collision_maps[to as usize].obstr(node.loc.x + kx, node.loc.y + ky) {
                        node_new_locs.push(PieceLocation {
                            x: node.loc.x + kx,
                            y: node.loc.y + ky,
                            piece: loc.piece,
                            rotation: to,
                            spun: true,
                            possible_line_clear: false
                        });
                        break;
                    }
                }
            }
            for new_loc in node_new_locs {
                next_id += 1;
                let new_node = KeyNode {
                    loc: new_loc,
                    id: next_id,
                    prev_id: node.id
                };
                if !collision_maps[new_loc.rotation as usize].obstr(new_loc.x, new_loc.y) && !visited_nodes.contains(&new_node) {
                    new_nodes.insert(new_node.clone());
                    if new_loc == loc {
                        found_node = Some(new_node);
                        found = true;
                    }
                }
            }
        }
        visited_nodes.extend(nodes_to_search);
        nodes_to_search = new_nodes;
    }

    let mut found_node = found_node.unwrap();
    let mut queue_keys: Vec<KeyNode> = vec![found_node.clone()];
    while found_node.id != found_node.prev_id {
        found_node = visited_nodes.iter().find(|node| found_node.prev_id == node.id).copied().unwrap();
        queue_keys.push(found_node.clone());
    }
    queue_keys.reverse();

    let mut new_keys: Vec<Key> = vec![];
    for (node1, node2) in queue_keys.iter().tuple_windows() {
        let rot_diff = (node2.loc.rotation as i8 - node1.loc.rotation as i8).rem_euclid(4);
        if rot_diff != 0 {
            new_keys.push(match rot_diff {
                1 => Key::rotateCW,
                2 => Key::rotate180,
                3 => Key::rotateCCW,
                _ => panic!("??? wtf rotation")
            });
            continue;
        }
        new_keys.push(match (node2.loc.x - node1.loc.x, node2.loc.y - node1.loc.y) {
            (-1, 0) => Key::moveLeft,
            (1, 0) => Key::moveRight,
            (0, y) if y < 0 => Key::softDrop,
            _ => panic!("??? wtf movement")
        })
    }

    (new_keys, queue_keys)
}

#[derive(Debug, Serialize)]
pub enum Key {
    moveLeft,
    moveRight,
    softDrop,
    rotateCW,
    rotateCCW,
    rotate180,
    hold,
    hardDrop
}