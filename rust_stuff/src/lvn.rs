use crate::resolver::*;
use bril_rs::*;

use std::{collections::HashMap, num::Saturating};

// assume that there are not enough args for Vec sorting to take a long time

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum ValueArg {
    Remapped(usize),
    Arg(String),
}

#[derive(Hash, Eq, PartialEq)]
pub enum Rval {
    Const {
        const_type: bril_rs::Type,
        raw_val: u64,
    },
    Value(bril_rs::ValueOps, Vec<ValueArg>, Vec<String>), // op plus args plus funcs:
    Consumer(Vec<usize>), // something with only rvals but nothing else
    NoVal,                // no rval
}

use bril_rs::Literal;

impl From<bril_rs::Literal> for Rval {
    fn from(value: bril_rs::Literal) -> Self {
        let (t, val) = match value {
            Literal::Int(v) => (Type::Int, v.cast_unsigned()),
            Literal::Float(v) => (Type::Float, v.to_bits()),
            Literal::Char(c) => (Type::Char, c as u64),
            Literal::Bool(b) => (Type::Bool, b as u64),
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
}

#[derive(Default)]
pub struct LVNTable {
    remaps: HashMap<String, usize>, // mapping of variable name to entry number
    exprs: HashMap<Rval, usize>,    // mapping of expression to entry number
    entries: Vec<LVNEntry>,         // actual entries
    args: Vec<String>,
}

impl LVNTable {
    pub fn get_remapped_arg(&self, argname: &String) -> String {
        if self.args.contains(argname) {
            return argname.clone();
        }
        self.entries
            .get(*self.remaps.get(argname).unwrap())
            .unwrap()
            .name
            .clone()
    }
    pub fn instr_to_rval(&self, instr: &Instruction) -> Option<Rval> {
        match instr {
            Instruction::Constant {
                dest,
                value,
                const_type,
                ..
            } => Some(Rval::from(value.clone())),
            Instruction::Value {
                args,
                dest,
                funcs,
                op,
                op_type,
                ..
            } => {
                let mut mapped_args: Vec<ValueArg> = args
                    .iter()
                    .map(|a| {
                        if self.args.contains(a) {
                            ValueArg::Arg(a.clone())
                        } else {
                            ValueArg::Remapped(*self.remaps.get(a).unwrap())
                        }
                    })
                    .collect();
                mapped_args.sort();

                Some(Rval::Value(op.clone(), mapped_args, funcs.clone()))
            } // TODO: consumer?
            _ => None,
        }
    }

    pub fn populate(&mut self, r: &CodeRange, instrs: &Vec<Code>, args: Vec<String>) -> Vec<Code> {
        self.args = args;
        let mut res = Vec::<Code>::new();
        for code in instrs[r.0..r.1].into_iter() {
            // only accept instructions
            if let Code::Instruction(instr) = code {
                //println!("{}", instr);
                match instr {
                    Instruction::Constant {
                        dest,
                        const_type: t,
                        ..
                    }
                    | Instruction::Value {
                        dest, op_type: t, ..
                    } => {
                        let curr_expr = self.instr_to_rval(instr).unwrap();
                        let table_entry = self.exprs.get(&curr_expr);
                        let dest_overwritten = false;

                        let mut inst_dest = dest.clone();

                        let entry_num = if let Some(i) = table_entry {
                            // the table entry already exists, it is i
                            // idx is the entry number
                            // somehow replace the instruction

                            // anything using this as an arg should instead refer to what's been calculated
                            let new_instr = Instruction::Value {
                                args: Vec::from([self.entries[*i].name.clone()]),
                                dest: inst_dest.clone(),
                                funcs: Vec::new(),
                                labels: Vec::new(),
                                op: ValueOps::Id,
                                pos: None, // TODO: handle
                                op_type: t.clone(),
                            };
                            res.push(Code::Instruction(new_instr));
                            *i
                        } else {
                            let num = self.entries.len(); // new value number
                            let mut new_instr = instr.clone();
                            if dest_overwritten {
                                inst_dest = String::from("overwritten_") + &inst_dest;
                                // set new instructions dest to this new thing
                            }

                            // https://stackoverflow.com/questions/54162832/is-there-a-way-to-create-a-copy-of-an-enum-with-some-field-values-updated

                            match &mut new_instr {
                                Instruction::Constant { dest, .. } => *dest = inst_dest.clone(),
                                Instruction::Value { args, dest, .. } => {
                                    *dest = inst_dest.clone();
                                    *args = args.iter().map(|x| self.get_remapped_arg(x)).collect();
                                }
                                _ => (),
                            }

                            self.exprs.insert(curr_expr, num);
                            self.entries.push(LVNEntry {
                                name: inst_dest.clone(),
                            }); // dest

                            res.push(Code::Instruction(new_instr));

                            num
                        };

                        // entry_idx is the index which the assignment should point to

                        // if there is some destination, map it to the correct entry

                        self.remaps.insert(inst_dest, entry_num);
                    }
                    _ => {
                        res.push(code.clone());
                    }
                }
            // hash the expression into expr_val
            // also obtain the destination name inst_dest

            // constant or values are the things we care about
            } else {
                // it's okay to put labels in unmodified
                res.push(code.clone());
            }
        }
        res
    }
    pub fn global_lvn(&mut self, p: &mut Program, d: &GlobalData) {
        for f in p.functions.iter_mut() {
            let f_data = d.data_map.get(&f.name).unwrap();
            let mut full_instrs = Vec::<Code>::new();
            for block in f_data.blocks.iter() {
                let args = f.args.iter().map(|x| x.name.clone()).collect();
                let new_instrs = self.populate(block, &f.instrs, args);
                full_instrs.extend(new_instrs);
            }
            f.instrs = full_instrs;
        }
    }
}

// copy propagation??
