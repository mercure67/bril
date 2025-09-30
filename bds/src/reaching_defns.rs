use crate::data_flow::DFDomainElement;
use crate::resolver::{Defn, GlobalData};
use crate::util::CFGPos;
use bril_rs::Code;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Display;

/// Implementation of the `DFDomainElement` trait for `Defn`.
///
/// This defines how sets of reaching definitions are merged and updated
/// in the dataflow analysis.
impl DFDomainElement for Defn {
    /// Merge multiple sets of definitions into one set.
    ///
    /// Each element of `input_sets` is treated as a set of definitions.
    /// The merge operation takes their union and
    /// returns a new sorted vector of definitions.
    ///
    /// # Arguments
    /// * `input_sets` – vectors of incoming sets of definitions
    ///
    /// # Returns
    /// A vector of unique `Defn` values, sorted.
    fn merge(input_sets: Vec<&Vec<Self>>) -> Vec<Self> {
        let set: HashSet<&Defn> = HashSet::from_iter(input_sets.into_iter().flatten());
        let mut vec: Vec<Self> = set.into_iter().cloned().collect();
        vec.sort();
        vec
    }

    /// Apply the transfer function for a single CFG block.
    ///
    /// Starting from `input_set`, this function:
    /// - Kills any previous definitions of a variable that are
    ///   redefined in the current block.
    /// - Adds new definitions from the block.
    ///
    /// # Arguments
    /// * `input_set` – set of definitions reaching the start of the block
    /// * `code` – unused
    /// * `d` – global program metadata
    /// * `pos` – position of the block in the CFG
    ///
    /// # Returns
    /// The updated set of reaching definitions after the processed block.
    fn transfer(input_set: &Vec<Self>, code: &[Code], d: &GlobalData, pos: CFGPos) -> Vec<Self> {
        let mut set: HashSet<Self> = HashSet::from_iter(input_set.iter().cloned());

        let f = &d.program.functions[pos.funcno];
        let cr = d.get_func_data(&f.name).unwrap().blocks[pos.blockno];

        for (l, def) in &d.get_func_data(&f.name).unwrap().defns {
            if l >= &cr.0 && l < &cr.1 {
                set.retain(|d| d.var != def.var);
                set.insert(Defn::from(def.var.clone(), *l, f.name.clone()));
            }
        }

        let mut vec: Vec<Self> = set.into_iter().collect();
        vec.sort();
        vec
    }
}

impl Display for Defn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.var, self.name)
    }
}
