use crate::resolver::*;
use crate::util::*;
use bril_rs::*;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fmt::Display;

/// A domain element for dataflow analysis.
///
/// Must define:
/// - Merging multiple sets of elements (meet)
/// - Transfering within a CFG block (must be monotonic)
pub(crate) trait DFDomainElement: Sized + PartialEq {
    /// Merge multiple sets of elements into a single set.
    ///
    /// # Arguments
    /// * `input_sets` – sets of elements to merge
    fn merge(input_sets: Vec<&Vec<Self>>) -> Vec<Self>;

    /// Apply the transfer function for a single CFG block.
    ///
    /// # Arguments
    /// * `input_set` – set of elements reaching the start of the block
    /// * `code` – code in the block
    /// * `d` – global program metadata
    /// * `pos` – block’s position in the CFG
    fn transfer(input_set: &Vec<Self>, code: &[Code], d: &GlobalData, pos: CFGPos) -> Vec<Self>;
}

/// Configuration for a worklist-based dataflow analysis.
pub struct WorklistConfig<'a, T: DFDomainElement + Display> {
    /// Set of input elements for each CFG block.
    pub input_set: HashMap<CFGPos, Vec<T>>,
    /// Set of output elements for each CFG block.
    pub output_set: HashMap<CFGPos, Vec<T>>,
    /// Reference to global program metadata.
    pub data: &'a mut GlobalData<'a>,
}

impl<'a, T> WorklistConfig<'a, T>
where
    T: DFDomainElement + Debug + Display,
{
    /// Run the worklist algorithm until fixpoint.
    pub fn worklist(&mut self) {
        let cfg = self.data.form_cfg();
        let mut wl = self.data.all_blocks();
        wl.sort();
        while !wl.is_empty() {
            let b = wl.pop().unwrap();

            let pred = predecessors(&b, &cfg);

            let to_merge: Vec<&Vec<T>> = self
                .output_set
                .iter()
                .filter(|x| pred.contains(x.0))
                .map(|x| x.1)
                .collect();
            self.input_set.insert(b, T::merge(to_merge));

            let new_out = T::transfer(
                self.input_set.get(&b).unwrap(),
                self.data.get_codeslice(&b),
                self.data,
                b,
            );
            let existing = self.output_set.get(&b);

            let is_diff = if let Some(e) = existing {
                *e != *new_out
            } else {
                true
            };

            if is_diff {
                let mut tmp: HashSet<CFGPos> = HashSet::from_iter(wl.into_iter());
                tmp.extend(
                    successors(&b, &cfg)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter(),
                );
                wl = tmp.into_iter().collect();
                wl.sort();
            }

            self.output_set.insert(b, new_out);
        }
    }

    /// Print the current input and output sets for all blocks.
    pub fn print(&self) {
        let blocks = self.data.all_blocks();

        for block in blocks {
            println!("block: {}", block);
            println!(
                "  in:  {}",
                self.input_set
                    .get(&block)
                    .unwrap()
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(",\n       ")
            );
            println!(
                "  out: {}",
                self.output_set
                    .get(&block)
                    .unwrap()
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(",\n       ")
            );
            println!();
        }
    }
}
