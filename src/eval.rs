use crate::data::*;
use rand::Rng;

pub struct Eval {
    max_height: f32,
    max_height_half: f32,
    max_height_quarter: f32,
    total_holes: f32,
    coveredness: f32,
    row_transitions: f32,
    depth4: f32,
    dependencies: f32,
    i_dependencies: f32,
    spikes: f32,
    concavity: f32,
    b2b_clear: f32,
    b2b_deficit: f32,
    b2b_diff: f32
}

impl Eval {
    pub fn from(arr: [f32; 14]) -> Self {
        Self {
            max_height: arr[0],
            max_height_half: arr[1],
            max_height_quarter: arr[2],
            total_holes: arr[3],
            coveredness: arr[4],
            row_transitions: arr[5],
            depth4: arr[6],
            dependencies: arr[7],
            i_dependencies: arr[8],
            spikes: arr[9],
            concavity: arr[10],
            b2b_clear: arr[11],
            b2b_deficit: arr[12],
            b2b_diff: arr[13]
        }
    }

    // new eval, largely copied from cc2 :3 (sorry mk im still learning)
    pub fn eval(&self, root: &Game, game: &Game, info: &PlacementInfo) -> f32 {
        // height
        let heights: [i32; 10] = game.board.cols.map(|c| 64 - c.leading_zeros() as i32);

        let max_height = *heights.iter().max().unwrap();
        let max_height_half = max_height.max(10) - 10;
        let max_height_quarter = max_height.max(15) - 15;

        // holes
        let total_holes = game.board.cols.iter().map(|&c| {
            let h = 64 - c.leading_zeros();
            let under = (1 << h) - 1;
            (!c & under).count_ones()
        }).sum::<u32>();

        // coveredness
        let mut coveredness = 0;
        for &c in &game.board.cols {
            let h = 64 - c.leading_zeros();
            let under = (1 << h) - 1;
            let mut holes = !c & under;
            while holes != 0 {
                let y = holes.trailing_zeros();
                coveredness += h - y;
                holes &= !(1 << y);
            }
        }

        // row transitions
        let row_transitions = game.board.cols
            .windows(2)
            .map(|c| (c[0] ^ c[1]).count_ones())
            .sum::<u32>();

        // 4 line depth
        let (w_col, w_height) = game.board.cols.iter()
            .enumerate()
            .min_by_key(|&(_, h)| h)
            .unwrap();
        let almost_full_lines = game.board.cols.iter()
            .enumerate()
            .filter(|&(i, _)| i != w_col)
            .fold(!0, |a, (_, b)| a & b);
        let depth4 = (almost_full_lines >> w_height).trailing_ones();

        // dependencies, spikes

        let mut dependencies = 0;
        let mut i_dependencies = 0;
        let mut spikes = 0;
        let mut concavity = 0;
        
        for x in 0..10 {
            if x == w_col {
                continue;
            }

            let a = heights.get(x - 1).copied().unwrap_or(99);
            let b = heights[x];
            let c = heights.get(x + 1).copied().unwrap_or(99);

            dependencies += (a - 1 > b && c - 1 > b) as i32;
            i_dependencies += (a - 2 > b && c - 2 > b) as i32;
            spikes += (a + 1 < b && c + 1 < b) as i32;
            concavity += a - 2 * b + c;
        }

        // other stuff
        let b2b_clear = info.spin && info.lines_cleared > 0;
        let b2b_deficit = game.b2b_deficit;

        // remove this in the future
        let b2b_diff = if game.b2b > root.b2b { game.b2b - root.b2b } else {  0  };

        // final part: add some rng so can encourage exploration

        let res = self.max_height * max_height as f32
        + self.max_height_half * max_height_half as f32
        + self.max_height_quarter * max_height_quarter as f32
        + self.total_holes * total_holes as f32
        + self.coveredness * coveredness as f32
        + self.row_transitions * row_transitions as f32
        + self.depth4 * depth4 as f32
        + self.dependencies * dependencies as f32
        + self.i_dependencies * i_dependencies as f32
        + self.spikes * spikes as f32
        + self.concavity * concavity as f32
        + self.b2b_clear * b2b_clear as u8 as f32
        + self.b2b_deficit * b2b_deficit as f32
        + self.b2b_diff * b2b_diff as f32;

        let mut rng = rand::rng();
        let noise = 0.03 * rng.random_range(-res.abs()..=res.abs());

        res + noise
    }
}