#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

#[guests_macro::proving_entrypoint]
pub fn main(graph: Vec<Vec<bool>>, colors: u32, coloring: Vec<[u32; 2]>) -> bool {
    // Does it use the correct amount of colors?
    let mut max_color = coloring[0][1];
    for nc in &coloring {
        if nc[1] > max_color {
            max_color = nc[1];
        }
    }

    let mut ret = max_color + 1 == colors;

    // Is coloring correct?
    for i in 0..graph.len() {
        for j in 0..graph.len() {
            // graph[i][j] -> coloring[i] != coloring[j]
            ret = ret & (!graph[i][j] | (coloring[i][1] != coloring[j][1]));
        }
    }

    ret
}
