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
/*trait DF_Merge {
    fn merge(Vec<)
}*/

pub struct WorklistConfig<'a> {
    data: &'a mut GlobalData<'a>,
    p: &'a mut Program,
    transfer: fn(&[Code], &Vec<String>) -> Vec<String>, // the transfer function
    merge: fn(Vec<&Vec<String>>) -> Vec<String>,        // the merge function.
}

// TODO: best method for making merge, transfer overrideable? merge, transfer results in dynamic dispatch

impl<'a> WorklistConfig<'a> {
    fn worklist(&self, data: &mut GlobalData) {
        let mut in_set = HashMap::<CFGPos, Vec<String>>::new();
        let mut out_set = HashMap::<CFGPos, Vec<String>>::new();

        let cfg = data.form_cfg();
        let mut wl = data.all_blocks();
        while !wl.is_empty() {
            let b = wl.first().unwrap().clone();
            let pred = predecessors(&b, &cfg).unwrap();
            let m: Vec<&Vec<String>> = out_set
                .iter()
                .filter(|x| pred.contains(x.0))
                .map(|x| x.1)
                .collect();
            in_set.insert(b, (self.merge)(m));

            let new_out = (self.transfer)(data.get_codeslice(&b), in_set.get(&b).unwrap());
            let existing = out_set.get(&b);
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
            out_set.insert(b, new_out);
        }
    }
}
