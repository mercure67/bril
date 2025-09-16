use crate::resolver;
use bril_rs::*;

enum InstrStatus {
    Unused,
    Rewritten,
    Used,
}

fn function_dce(instrs: &Vec<Code>, blocks: &Vec<(usize, usize)>) -> Vec<Code> {
    let mut new_instrs: Vec<Code> = instrs.clone();
    let mut remove_vec: Vec<usize> = Vec::new();
    loop {
        for (i, instr) in new_instrs.iter().enumerate() {
            let destination = match instr {
                Code::Instruction(Instruction::Constant { dest, .. })
                | Code::Instruction(Instruction::Value { dest, .. }) => dest,
                _ => {
                    break;
                }
            };
            let mut status = InstrStatus::Unused;
            let mut lrw = 0;
            for j in i + 1..new_instrs.len() {
                match new_instrs.get(j) {
                    Some(Code::Instruction(Instruction::Value { args, .. }))
                    | Some(Code::Instruction(Instruction::Effect { args, .. }))
                        if args.contains(destination) =>
                    {
                        status = InstrStatus::Used;
                        break;
                    }
                    Some(Code::Instruction(Instruction::Constant { dest, .. }))
                    | Some(Code::Instruction(Instruction::Value { dest, .. }))
                        if dest == destination =>
                    {
                        status = InstrStatus::Rewritten;
                        lrw = j;
                        break;
                    }
                    _ => {}
                }
            }
            match status {
                InstrStatus::Unused => {
                    remove_vec.push(i);
                }
                InstrStatus::Rewritten => {
                    if blocks
                        .iter()
                        .any(|&(start, end)| start <= i && i < end && start <= lrw && lrw < end)
                    {
                        remove_vec.push(i);
                    }
                }
                InstrStatus::Used => {}
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

pub fn global_dce(p: &Program, d: &resolver::GlobalData) -> Program {
    let mut funcs: Vec<Function> = Vec::new();
    for f in p.functions.iter() {
        let f_data = d.data_map.get(&f.name).unwrap();
        let new_instrs = function_dce(&f.instrs, &f_data.blocks);
        let new_f = Function {
            args: f.args.clone(),
            instrs: new_instrs,
            name: f.name.clone(),
            pos: f.pos.clone(),
            return_type: f.return_type.clone(),
        };
        funcs.push(new_f);
    }
    return Program {
        functions: funcs,
        imports: p.imports.clone(),
    };
}
