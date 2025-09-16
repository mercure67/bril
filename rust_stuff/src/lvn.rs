use crate::resolver::*;
use bril_rs::*;

use std::collections::HashMap;

// assume that there are not enough args for Vec sorting to take a long time

#[derive(Hash, Eq, PartialEq)]
pub enum Rval {
    Const {
        const_type: bril_rs::Type,
        raw_val: u64,
    },
    Value(bril_rs::ValueOps, Vec<usize>), // op plus args: using a btreeset sorts by default
    Consumer(Vec<usize>),                 // something with only rvals but nothing else
    NoVal,                                // no rval
}

use bril_rs::Literal;

impl From<bril_rs::Literal> for Rval {
    fn from(value: bril_rs::Literal) -> Self {
        let (t, val) = match value {
            bril_rs::Literal::Int(v) => (bril_rs::Type::Int, v.cast_unsigned()),
            bril_rs::Literal::Float(v) => (bril_rs::Type::Float, v.to_bits()),
            bril_rs::Literal::Char(c) => (bril_rs::Type::Char, c as u64),
            bril_rs::Literal::Bool(b) => (bril_rs::Type::Bool, b as u64),
        };
        Self::Const {
            const_type: t,
            raw_val: val,
        }
    }
}

impl TryInto<bril_rs::Literal> for Rval {
    type Error = ();

    fn try_into(self) -> Result<bril_rs::Literal, Self::Error> {
        if let Rval::Const {
            const_type: t,
            raw_val: v,
        } = self
        {
            return match t {
                Type::Int => Ok(Literal::Int(v.cast_signed())),
                Type::Bool => Ok(Literal::Bool(v != 0)),
                Type::Float => Ok(bril_rs::Literal::Float(f64::from_bits(v))),
                Type::Char => Ok(Literal::Char(char::from_u32(v as u32).unwrap())),
                _ => Err(()),
            };
        }
        Err(())
    }
}

// TODO: encode function names in consumers, values
// id is handled as a valueop
//
// NOTE: if the Consumer option causes trouble, it can most likely be removed. its goal is to remove redundant function calls.

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
        for code in instrs[r.0..r.1].into_iter() {
            // only accept instructions
            if let Code::Instruction(instr) = code {
                // hash the expression into expr_val
                // also obtain the destination name dest_n
                let (expr_val, dest_n) = match instr {
                    Instruction::Constant { dest, value, .. } => {
                        (Rval::from(value.clone()), Some(dest.clone()))
                    }
                    Instruction::Value {
                        args,
                        dest,
                        funcs,
                        op,
                        ..
                    } => {
                        let mut mapped_args: Vec<usize> =
                            args.iter().map(|a| *self.remaps.get(a).unwrap()).collect();
                        mapped_args.sort();

                        (Rval::Value(op.clone(), mapped_args), Some(dest.clone()))
                    }
                    Instruction::Effect { args, .. } => {
                        let mut mapped_args: Vec<usize> =
                            args.iter().map(|a| *self.remaps.get(a).unwrap()).collect();
                        mapped_args.sort();

                        (Rval::Consumer(mapped_args), None)
                    }
                    _ => (Rval::NoVal, None),
                };

                self.exprs.entry(expr_val);
                // TODO: handle dest_n
            }
        }
    }
}
