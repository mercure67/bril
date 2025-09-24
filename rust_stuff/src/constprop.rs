use crate::{data_flow::DFDomainElement, resolver::GlobalData, util::CFGPos};
use bril_rs::{Code, Instruction};
use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Eq, Hash, Clone, Debug, Ord, PartialOrd)]
pub enum CodeConst {
    Bool { v: bool },
    Int { v: i64 },
    NotImpl,
}

impl From<bril_rs::Literal> for CodeConst {
    fn from(value: bril_rs::Literal) -> Self {
        match value {
            bril_rs::Literal::Int(i) => CodeConst::Int { v: i },
            bril_rs::Literal::Bool(b) => CodeConst::Bool { v: b },
            _ => CodeConst::NotImpl,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Ord, PartialOrd)]
pub struct ConstProp {
    name: String,
    val: CodeConst,
}

impl DFDomainElement for ConstProp {
    fn merge(input_sets: &Vec<&Vec<Self>>) -> Vec<Self> {
        let mut set = HashSet::<Self>::new();

        for v in input_sets {
            if v.is_empty() {
                continue;
            }
            let tmp: HashSet<ConstProp> = HashSet::from_iter(v.into_iter().cloned());
            if set.is_empty() {
                set.extend(tmp);
            } else {
                set = &set & &tmp;
            }

            //set.extend(v.iter().cloned());
        }
        let mut tmp: Vec<Self> = set.into_iter().collect();
        tmp.sort();
        tmp

        /*let mut names = HashMap::<String, u64>::new();

        for j in set.iter() {
            names
                .entry(j.name.clone())
                .and_modify(|x| *x = *x + 1)
                .or_insert(1);
        }

        let mut tmp: Vec<Self> = set.into_iter().collect();
        tmp.sort();
        tmp = tmp
            .into_iter()
            .filter(|x| *names.get(&x.name).unwrap() == 1)
            .collect();
        tmp
        */
    }

    fn transfer(input_set: &Vec<Self>, code: &[Code], d: &GlobalData, pos: CFGPos) -> Vec<Self> {
        let mut defs = HashSet::<Self>::new();
        let mut kills = HashSet::<String>::new();
        let funcname = d.funcname_from_funcno(pos.funcno).unwrap();
        for line in code {
            match line {
                Code::Instruction(Instruction::Constant { dest, value, .. }) => {
                    let rel_dest = funcname.clone() + &dest.clone();
                    let c = ConstProp {
                        name: rel_dest.clone(),
                        val: CodeConst::from(value.clone()),
                    };
                    if !input_set.contains(&c) {
                        let names: Vec<String> = input_set.iter().map(|x| x.name.clone()).collect();
                        if names.contains(&rel_dest) {
                            // same name, diff val
                            kills.insert(rel_dest.clone());
                        } else {
                            defs.insert(c);
                        }
                    }
                }
                Code::Instruction(Instruction::Value { dest, .. }) => {
                    let rel_dest = funcname.clone() + &dest.clone();
                    kills.insert(rel_dest.clone());
                }
                _ => {}
            }
        }
        let mut res: Vec<Self> = input_set.clone();
        //println!("{:?}", res);
        res.retain(|x| !kills.contains(&x.name));
        res.extend(defs.into_iter());
        res.sort();
        //println!("{:?}", res);
        res
    }
}
