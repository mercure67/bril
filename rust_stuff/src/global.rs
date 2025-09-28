// global analysis

use crate::util::*;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

#[derive(Default)]
pub struct DomMapping {
    pub mapping: HashMap<CFGPos, HashSet<CFGPos>>,
    pub tree: HashMap<CFGPos, HashSet<CFGPos>>,
    pub entry: CFGPos, // will almost always be the first block in the function. is exposed if override is necessary.
    pub fno: usize,
}

impl DomMapping {
    pub fn create_initial(&mut self, fno: usize, num_blocks: usize) {
        // populate mapping with some initial information

        self.fno = fno;
        let first = CFGPos {
            funcno: fno,
            blockno: 0,
        };

        // first entry should only be mapped to itself
        self.mapping.insert(first, HashSet::from([first.clone()]));
        self.entry = first;

        let all_block_val: HashSet<CFGPos> = (0..num_blocks)
            .map(|x| CFGPos {
                funcno: fno,
                blockno: x,
            })
            .collect();

        for i in 1..num_blocks {
            let key = CFGPos {
                funcno: fno,
                blockno: i,
            };
            self.mapping.insert(key, all_block_val.clone());
        }
    }

    pub fn find_dominators(&mut self, c: &CFG) {
        let mut did_change = true;
        while did_change {
            did_change = false;
            let mut vertices = postorder(c, &self.entry);
            vertices.retain(|x| *x != self.entry);
            for v in vertices {
                let mut dom_isect =
                    predecessors(&v, c)
                        .iter()
                        .fold(HashSet::<CFGPos>::new(), |mut acc, n| {
                            if n.funcno == self.fno {
                                let tmp = HashSet::from_iter(
                                    self.mapping
                                        .get(n)
                                        .unwrap()
                                        .iter()
                                        .filter(|x| x.funcno == self.fno)
                                        .cloned(),
                                );
                                acc = if acc.is_empty() { tmp } else { &acc & &tmp };
                            }
                            acc
                        });

                dom_isect.insert(v.clone());
                self.mapping.entry(v).and_modify(|e| {
                    if *e != dom_isect {
                        *e = dom_isect;
                        did_change = true;
                    }
                });
            }
        }
    }

    pub fn create_tree(&mut self, num_blocks: usize) {
        // this can maybe be done during initial dominator mapping creation, but ah well
        let mut to_process: Vec<CFGPos> = (0..num_blocks)
            .map(|x| CFGPos {
                funcno: self.fno,
                blockno: x,
            })
            .collect();
        to_process.sort();
        // the first block will end up on top
        while !to_process.is_empty() {
            let curr = to_process.remove(0);
            if self.tree.is_empty() {
                self.tree.insert(curr, HashSet::<CFGPos>::new());
                continue;
            }
            let mut dominators = self.mapping.get(&curr).unwrap().clone();
            dominators.retain(|x| *x != curr); // remove self
            // every block should at least be dominated by block 0. if not, we have a problem
            let mut deepest = self.entry;
            while !dominators.is_empty() {
                let possible_suc = self.tree.entry(deepest).or_insert(HashSet::<CFGPos>::new());
                if possible_suc.is_empty() {
                    break;
                }
                if let Some(e) = possible_suc
                    .iter()
                    .filter(|x| dominators.contains(x))
                    .last()
                {
                    deepest = *e;
                } else {
                    break;
                }
            }
            self.tree.entry(deepest).and_modify(|e| {
                e.insert(curr);
            });
        }
    }
}

impl Display for DomMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in self.mapping.iter() {
            f.write_fmt(format_args!("{} ->", k))?;
            for p in v {
                f.write_fmt(format_args!("{} ", p))?;
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}
