use crate::data_flow::DFDomainElement;
use bril_rs::{Code, Instruction};
use std::collections::HashSet;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
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

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ConstProp {
    name: String,
    val: CodeConst,
}

impl DFDomainElement for ConstProp {
    fn merge(input_sets: &Vec<&Vec<Self>>) -> Vec<Self> {
        let mut set = HashSet::<Self>::new();
        for v in input_sets {
            //let tmp: HashSet<ConstProp> = HashSet::from_iter(v.into_iter().cloned());
            //set = &set & &tmp;
            set.extend(v.iter().cloned());
        }

        set.into_iter().collect()
    }

    fn transfer(input_set: &Vec<Self>, code: &[Code]) -> Vec<Self> {
        let mut defs = HashSet::<Self>::new();
        let mut kills = HashSet::<String>::new();
        for line in code {
            match line {
                Code::Instruction(Instruction::Constant { dest, value, .. }) => {
                    let c = ConstProp {
                        name: dest.clone(),
                        val: CodeConst::from(value.clone()),
                    };
                    if !input_set.contains(&c) {
                        let names: Vec<String> = input_set.iter().map(|x| x.name.clone()).collect();
                        if names.contains(dest) {
                            // same name, diff val
                            kills.insert(dest.clone());
                        } else {
                            defs.insert(c);
                        }
                    }
                }
                Code::Instruction(Instruction::Value { dest, .. }) => {
                    kills.insert(dest.clone());
                }
                _ => {}
            }
        }
        let mut res: Vec<Self> = input_set.clone();
        res.retain(|x| !kills.contains(&x.name));
        res.extend(defs.into_iter());
        res
    }
}
