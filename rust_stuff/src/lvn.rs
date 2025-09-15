use crate::*;

enum Rval {
    Const(bril_rs::Literal),
    Value(bril_rs::ValueOps, Vec<String>), // op plus args
    Effect,                                // no rval
}
// TODO: alphabetically sort args
//TODO: handle the rename case
// TODO: separate lval?

struct LVNEntry {
    name: String,
    entryno: usize,
    inst: bril_rs::Instruction,
    expr: Rval,
}

struct LVNTable {
    remaps: HashMap<String, usize>, // mapping of variable name to entry number
    exprs: HashMap<Rval, usize>,    // mapping of expression to entry number
    entries: Vec<LVNEntry>,         // actual entries
}
