use crate::resolver::*;
use crate::util::*;
use bril_rs::*;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

//TODO: we might need a more sophisticated type for variables in the future, but ah well
pub(crate) trait DFDomainElement: Sized + PartialEq {
    fn merge(input_sets: &Vec<&Vec<Self>>) -> Vec<Self>;
    fn transfer(input_set: &Vec<Self>, code: &[Code], d: &GlobalData, pos: CFGPos) -> Vec<Self>;
}

pub struct WorklistConfig<'a, T: DFDomainElement> {
    pub input_set: HashMap<CFGPos, Vec<T>>,
    pub output_set: HashMap<CFGPos, Vec<T>>,
    pub data: &'a mut GlobalData<'a>,
}

// TODO: best method for making merge, transfer overrideable? merge, transfer results in dynamic dispatch

impl<'a, T> WorklistConfig<'a, T>
where
    T: DFDomainElement + Debug,
{
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
            self.input_set.insert(b, T::merge(&to_merge));

            let new_out = T::transfer(
                self.input_set.get(&b).unwrap(),
                self.data.get_codeslice(&b),
                self.data,
                b,
            );
            //println!("=> {:?}", new_out);
            let existing = self.output_set.get(&b);
            let mut is_diff = false;
            if let None = existing {
                is_diff = true;
            } else if let Some(e) = existing {
                is_diff = *e != *new_out;
            }
            if is_diff {
                let mut tmp: HashSet<CFGPos> = HashSet::from_iter(wl.into_iter());
                tmp.extend(successors(&b, &cfg).unwrap().into_iter());
                wl = tmp.into_iter().collect();
                wl.sort();
            }

            self.output_set.insert(b, new_out);
        }
    }

    pub fn print(&self) {
        let blocks = self.data.all_blocks();

        for block in blocks {
            println!("block: {}", block);
            println!("in: {:?}", self.input_set.get(&block).unwrap());
            println!("out: {:?}", self.output_set.get(&block).unwrap());
            println!();
        }
    }
}
