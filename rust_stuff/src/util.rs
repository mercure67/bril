// utility classes and types

use std::collections::{HashMap, HashSet};

pub type CodeRange = (usize, usize);
pub type Blockno = usize;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Ord, PartialOrd, Default)]
pub struct CFGPos {
    pub funcno: usize,
    pub blockno: Blockno,
}

impl std::fmt::Display for CFGPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("f{}.b{}", self.funcno, self.blockno))
    }
}

pub type CFG = HashMap<CFGPos, HashSet<CFGPos>>;
pub fn successors<'a>(p: &'a CFGPos, cfg: &'a CFG) -> Option<&'a HashSet<CFGPos>> {
    // return the successors of a given block in the CFG
    cfg.get(p)
}

pub fn predecessors(p: &CFGPos, cfg: &CFG) -> HashSet<CFGPos> {
    let mut res = HashSet::<CFGPos>::new();
    for (k, v) in cfg.iter() {
        if v.contains(p) {
            res.insert(*k);
        }
    }
    res
}

pub fn postorder(cfg: &CFG, root: &CFGPos) -> Vec<CFGPos> {
    // https://eli.thegreenplace.net/2015/directed-graph-traversal-orderings-and-applications-to-data-flow-analysis/
    let mut visited = HashSet::<CFGPos>::new();
    let mut res = Vec::<CFGPos>::new();

    fn dfs_walk(v: &mut HashSet<CFGPos>, r: &mut Vec<CFGPos>, cfg: &CFG, node: &CFGPos) {
        v.insert(node.clone());
        for succ in successors(node, cfg).unwrap().iter() {
            if !v.contains(succ) {
                dfs_walk(v, r, cfg, succ);
            }
        }
        r.push(node.clone());
    }
    dfs_walk(&mut visited, &mut res, cfg, root);
    res
}
