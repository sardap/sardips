use std::ops::Range;

use serde::{Deserialize, Serialize};
use shared_deps::bevy_turborand::DelegatedRng;

pub fn gen_f32_range<T: DelegatedRng>(rng: &mut T, range: &Range<f32>) -> f32 {
    const MULTIPLY_FACTOR: f32 = 1000.0;
    rng.i32((range.start * MULTIPLY_FACTOR) as i32..(range.end * MULTIPLY_FACTOR) as i32) as f32
        / MULTIPLY_FACTOR
}

pub fn gen_f64_range<T: DelegatedRng>(rng: &mut T, range: &Range<f64>) -> f64 {
    const MULTIPLY_FACTOR: f64 = 10000000.0;
    rng.i32((range.start * MULTIPLY_FACTOR) as i32..(range.end * MULTIPLY_FACTOR) as i32) as f64
        / MULTIPLY_FACTOR
}

fn gcd(a: u32, b: u32) -> u32 {
    let (a, b) = if a < b { (b, a) } else { (a, b) };

    if a % b == 0 { b } else { gcd(b, a % b) }
}

pub fn gcd_for_slice(slice: &[u32]) -> u32 {
    if slice.is_empty() {
        return 0;
    }

    let mut iter = slice.iter().skip_while(|x| x == &&0);
    let first = match iter.next() {
        Some(v) => *v,
        None => return 1,
    };

    let gcd = iter.fold(
        first,
        |acc, cur| {
            if *cur == 0 { acc } else { gcd(*cur, acc) }
        },
    );
    gcd
}

fn sum(index_weights: &[u32]) -> u32 {
    index_weights.iter().fold(0, |acc, cur| acc + cur)
}

/// Calculates the mean of `index_weights`.
fn mean(index_weights: &[u32]) -> u32 {
    sum(index_weights) / index_weights.len() as u32
}

/// Returns the tables of aliases and probabilities.
fn calc_table(index_weights: &[u32]) -> (Vec<usize>, Vec<f32>) {
    let table_len = index_weights.len();
    let (mut below_vec, mut above_vec) = separate_weight(index_weights);
    let mean = mean(index_weights);

    let mut aliases = vec![0; table_len];
    let mut probs = vec![0.0; table_len];
    loop {
        match below_vec.pop() {
            Some(below) => {
                if let Some(above) = above_vec.pop() {
                    let diff = mean - below.1;
                    aliases[below.0] = above.0 as usize;
                    probs[below.0] = diff as f32 / mean as f32;
                    if above.1 - diff <= mean {
                        below_vec.push((above.0, above.1 - diff));
                    } else {
                        above_vec.push((above.0, above.1 - diff));
                    }
                } else {
                    aliases[below.0] = below.0 as usize;
                    probs[below.0] = below.1 as f32 / mean as f32;
                }
            }
            None => break,
        }
    }

    (aliases, probs)
}

/// Divide the values of `index_weights` based on the mean of them.
///
/// The tail value is a weight and head is its index.
fn separate_weight(index_weights: &[u32]) -> (Vec<(usize, u32)>, Vec<(usize, u32)>) {
    let mut below_vec = Vec::with_capacity(index_weights.len());
    let mut above_vec = Vec::with_capacity(index_weights.len());
    for (i, w) in index_weights.iter().enumerate() {
        if *w <= mean(index_weights) {
            below_vec.push((i, *w));
        } else {
            above_vec.push((i, *w));
        }
    }
    (below_vec, above_vec)
}

pub trait NewBuilder<T> {
    fn new(index_weights: &[T]) -> WalkerTable;
}

impl NewBuilder<u32> for WalkerTable {
    fn new(index_weights: &[u32]) -> WalkerTable {
        let index_weights = index_weights.to_vec();

        let table_len = index_weights.len();

        if sum(&index_weights) == 0 {
            // Returns WalkerTable that performs unweighted random sampling.
            return WalkerTable {
                aliases: vec![0; table_len],
                probs: vec![0.0; table_len],
            };
        }

        let (aliases, probs) = calc_table(&index_weights);

        WalkerTable { aliases, probs }
    }
}

impl NewBuilder<f32> for WalkerTable {
    fn new(index_weights: &[f32]) -> WalkerTable {
        let ws = index_weights
            .iter()
            .map(|w| (w * 10000.0).round() as u32)
            .collect::<Vec<u32>>();

        let gcd = gcd_for_slice(&ws);
        let ws = ws.iter().map(|w| w / gcd).collect::<Vec<u32>>();

        WalkerTable::new(&ws)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct WalkerTable {
    /// Alias to another index
    aliases: Vec<usize>,

    /// Probability for whether to output the index attached to `aliases`.
    probs: Vec<f32>,
}

impl WalkerTable {
    /// Returns an index at random using an external RNG which implements Rng.
    pub fn next_rng<T: DelegatedRng>(&self, rng: &mut T) -> usize {
        let i = rng.usize(0..self.probs.len());
        let r = rng.f32();
        if r < self.probs[i] {
            return self.aliases[i];
        }
        i
    }
}
