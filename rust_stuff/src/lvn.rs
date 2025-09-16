use crate::*;
use std::collections::BTreeSet;

pub enum Rval {
    Const(bril_rs::Literal),
    Value(bril_rs::ValueOps, BTreeSet<usize>), // op plus args: using a btreeset sorts by default
    Consumer(BTreeSet<usize>),                 // something with only rvals but nothing else
    NoVal,                                     // no rval
}

// TODO: do consumers need dedicated handling?
// TODO: handle id?

impl PartialEq for Rval {
    fn eq(&self, other: &Self) -> bool {
        // https://stackoverflow.com/questions/36297412/how-to-implement-partialeq-for-an-enum

        match (self, other) {
            (Rval::Const(a), Rval::Const(b)) => match (a, b) {
                (bril_rs::Literal::Int(a), bril_rs::Literal::Int(b)) => a == b,
                (bril_rs::Literal::Bool(a), bril_rs::Literal::Bool(b)) => a == b,
                (bril_rs::Literal::Char(a), bril_rs::Literal::Char(b)) => a == b,
                _ => false,
            },
            (Rval::Consumer(a), Rval::Consumer(b)) => a == b,
            (Rval::Value(op_a, args_a), Rval::Value(op_b, args_b)) => {
                (op_a == op_b) && (args_a == args_b)
            }
            _ => false,
        }
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for Rval {}

//TODO: handle the rename case
// TODO: separate lval?

pub struct LVNEntry {
    name: String,
    entryno: usize,
    inst: bril_rs::Instruction,
    expr: Rval,
}

#[derive(Default)]
pub struct LVNTable {
    remaps: HashMap<String, usize>, // mapping of variable name to entry number
    exprs: HashMap<Rval, usize>,    // mapping of expression to entry number
    entries: Vec<LVNEntry>,         // actual entries
}

impl LVNTable {
    pub fn populate(&mut self, r: CodeRange, instrs: &Vec<Code>) {
        for code in instrs[r.0..r.1] {
            // only accept instructions
            if let Code::Instruction(instr) = code {
                // hash the expression
                let mut dest = String::new();

                let expr_val = match instr {
                    Instruction::Constant {
                        dest,
                        op,
                        pos,
                        const_type,
                        value,
                    } => {
                        dest = dest;
                        Rval::Const(value)
                    }
                    Instruction::Value {
                        args,
                        dest,
                        funcs,
                        labels,
                        op,
                        pos,
                        op_type,
                    } => {
                        dest = dest;
                        let mapped_args = args
                            .iter()
                            .map(|a| self.entries[self.remaps.get(a).unwrap().clone()])
                            .collect();

                        Rval::Value(op, BTreeSet::<usize>::from(mapped_args))
                    }
                    Instruction::Effect {
                        args,
                        funcs,
                        labels,
                        op,
                        pos,
                    } => {
                        let mapped_args = args
                            .iter()
                            .map(|a| self.entries[self.remaps.get(a).unwrap().clone()])
                            .collect();

                        Rval::Consumer(BTreeSet::<usize>::from(mapped_args))
                    }
                    _ => Rval::NoVal,
                };
            }
        }
    }
}
