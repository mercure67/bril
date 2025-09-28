// global analysis

use crate::util::*;
use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    fmt::Display,
};

// TODO: can probably use direct usize rather than cfgpos, considering function local
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

    fn doms_within_scope(&self, p: &CFGPos) -> HashSet<CFGPos> {
        // returns the dominators of a position from mapping, according to
        // the current funcno
        HashSet::from_iter(
            self.mapping
                .get(p)
                .unwrap()
                .iter()
                .filter(|x| x.funcno == self.fno)
                .cloned(),
        )
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
                        .fold(HashSet::<CFGPos>::new(), |acc, n| {
                            if n.funcno != self.fno {
                                return acc;
                            }

                            let tmp = self.doms_within_scope(n);
                            if acc.is_empty() { tmp } else { &acc & &tmp }
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

        // create an initial list of blocks to process
        let mut to_process: Vec<CFGPos> = (0..num_blocks)
            .rev() //https://stackoverflow.com/questions/25170091/how-to-make-a-reverse-ordered-for-loop
            .map(|x| CFGPos {
                funcno: self.fno,
                blockno: x,
            })
            .collect();

        // entry will end up at end (top of stack)

        while let Some(curr) = to_process.pop() {
            if self.tree.is_empty() {
                self.tree.insert(curr, HashSet::<CFGPos>::new());
                continue;
            }
            let mut dominators = self.mapping.get(&curr).unwrap().clone();
            dominators.retain(|x| *x != curr); // remove self
            // every block should at least be dominated by block 0. if not, bad!!

            // there perhaps exists a more efficient approach
            let mut deepest = self.entry;
            // while there is some non-empty route in the tree to go down,
            while let Entry::Occupied(e) = self.tree.entry(deepest)
                && !e.get().is_empty()
            {
                // if the entry contains some descendant in the dominators list, descent further
                if let Some(v) = e.get().iter().filter(|x| dominators.contains(x)).last() {
                    deepest = *v;
                } else {
                    // otherwise, suitable place to modify
                    break;
                }
            }
            // deepest is either occupied or unoccupied.
            // occupied implies adding curr as a descendant of something with descendants already
            // unoccupied implies curr is the first descendant of something
            self.tree
                .entry(deepest)
                .and_modify(|e| {
                    e.insert(curr);
                })
                .or_insert(HashSet::from([curr]));
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
