use crate::resolver::*;
use crate::util::*;
use bril_rs::*;
use std::collections::{HashMap, HashSet};

pub fn successors<'a>(p: &'a CFGPos, cfg: &'a CFG) -> Option<&'a HashSet<CFGPos>> {
    // return the successors of a given block in the CFG
    cfg.get(p)
}

pub fn predecessors(p: &CFGPos, cfg: &CFG) -> Option<HashSet<CFGPos>> {
    let mut res = HashSet::<CFGPos>::new();
    for (k, v) in cfg.iter() {
        if v.contains(p) {
            res.insert(*k);
        }
    }
    Some(res)
}

//TODO: we might need a more sophisticated type for variables in the future, but ah well
pub(crate) trait DFDomainElement: Sized + PartialEq {
    fn merge(input_sets: &[Vec<Self>]) -> Vec<Self>;
    fn transfer(input_set: &[Self], code: &[Code]) -> Vec<Self>;
}

pub struct WorklistConfig<'a, T: DFDomainElement> {
    input_set: HashMap<CFGPos, Vec<T>>,
    output_set: HashMap<CFGPos, Vec<T>>,
    data: &'a mut GlobalData<'a>,
    p: &'a mut Program,
}

// TODO: best method for making merge, transfer overrideable? merge, transfer results in dynamic dispatch

impl<'a, T> WorklistConfig<'a, T>
where
    T: DFDomainElement,
{
    fn worklist(&self, data: &mut GlobalData) {
        let cfg = data.form_cfg();
        let mut wl = data.all_blocks();
        while !wl.is_empty() {
            let b = wl.first().unwrap().clone();
            let pred = predecessors(&b, &cfg).unwrap();
            let m: &[Vec<T>] = self
                .output_set
                .iter()
                .filter(|x| pred.contains(x.0))
                .map(|x| x.1)
                .collect()
                .as_slice();
            self.input_set.insert(b, T::merge(m));

            let new_out = T::transfer(self.input_set.get(&b).unwrap(), data.get_codeslice(&b));
            let existing = self.output_set.get(&b);
            let mut is_diff = false;
            if let None = existing {
                is_diff = true;
            } else if let Some(e) = existing {
                for i in new_out.iter() {
                    if !e.contains(i) {
                        is_diff = true;
                    }
                }
            }
            if is_diff {
                wl.extend(successors(&b, &cfg).unwrap().iter());
            }
            self.output_set.insert(b, new_out);
        }
    }
}
