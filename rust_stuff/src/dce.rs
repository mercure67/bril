use std::collections::BTreeSet;

use crate::lvn::Rval;
use bril_rs::*;

fn block_dce(instrs: Vec<Instruction>) -> Vec<Instruction> {
    let mut new_instrs: Vec<Instruction> = instrs.clone();
    let mut remove_vec: Vec<usize> = Vec::new();
    loop {
        for (i, instr) in new_instrs.iter().enumerate() {
            let destination = match instr {
                Instruction::Constant { dest, .. } | Instruction::Value { dest, .. } => dest,
                _ => {
                    break;
                }
            };
            for j in i + 1..new_instrs.len() {
                match new_instrs.get(j) {
                    Some(Instruction::Value { args, .. })
                    | Some(Instruction::Effect { args, .. })
                        if args.contains(destination) =>
                    {
                        break;
                    }
                    Some(Instruction::Constant { dest, .. })
                    | Some(Instruction::Value { dest, .. })
                        if dest == destination =>
                    {
                        remove_vec.push(i);
                        break;
                    }
                    _ => {}
                }
            }
        }
        if remove_vec.is_empty() {
            break;
        }
        remove_vec.sort();
        remove_vec.reverse();
        for i in remove_vec.iter() {
            new_instrs.remove(*i);
        }
        remove_vec.clear();
    }

    return new_instrs;
}
