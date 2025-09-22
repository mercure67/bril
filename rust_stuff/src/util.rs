// utility classes and types

use std::collections::{HashMap, HashSet};

pub type CodeRange = (usize, usize);
pub type Blockno = usize;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
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
