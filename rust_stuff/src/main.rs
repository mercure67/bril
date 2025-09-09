use bril_rs::*;
use clap::{Parser, Subcommand};
use std::collections::{HashMap, HashSet};
use std::{fs::File, io::BufReader};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_hint = clap::ValueHint::FilePath, required=true)]
    filename: Option<std::path::PathBuf>,
}

fn get_used_labels<'a>(v: &'a bril_rs::Program) -> HashSet<String> {
    // return the list of used labels in the program
    // currently pretty simple, doesn't do analysis of conditions / dead code / etc.
    // also doesn't handle imports

    // TODO: do labels need to be unique?
    let mut res = HashSet::<String>::new();

    for f in v.functions.iter() {
        for i in f.instrs.iter() {
            //TODO: the ergonomics of this part of the library are poor
            if let Code::Instruction(inner) = i {
                if let Instruction::Effect {
                    funcs, labels, op, ..
                } = inner
                {
                    match op {
                        EffectOps::Jump | EffectOps::Branch => {
                            for x in labels.iter() {
                                res.insert(x.clone());
                            }
                        }
                        EffectOps::Return => (),
                        EffectOps::Call => {
                            for x in funcs.iter() {
                                res.insert(x.clone());
                            }
                        }
                        _ => (),
                    }
                }
                if let Instruction::Value { op, funcs, .. } = inner {
                    match op {
                        ValueOps::Call => {
                            for x in funcs.iter() {
                                res.insert(x.clone());
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    res
}

type Block<'a> = Vec<&'a Code>;

trait BlockHelpers<'a> {
    fn get_terminator(&self) -> &'a Code;
}

impl<'a> BlockHelpers<'a> for Block<'a> {
    fn get_terminator(&self) -> &'a Code {
        self.last().unwrap()
    }
}

// refactor: move function iteration out

fn form_basic_blocks<'a>(
    f: &'a bril_rs::Function,
    used_labels: &HashSet<String>,
) -> Vec<Block<'a>> {
    let mut res = Vec::<Block>::new();
    let mut new_block = Vec::<&'a Code>::new();
    for instr in f.instrs.iter() {
        match instr {
            Code::Instruction(i) => {
                new_block.push(instr);
                match i {
                    Instruction::Value { op, .. } => {
                        if let ValueOps::Call = op {
                            res.push(new_block);
                            new_block = Vec::<&'a Code>::new();
                        }
                    }
                    Instruction::Effect { op, .. } => match op {
                        EffectOps::Jump
                        | EffectOps::Branch
                        | EffectOps::Call
                        | EffectOps::Return => {
                            res.push(new_block);
                            new_block = Vec::<&'a Code>::new();
                        }
                        _ => (),
                    },
                    _ => (),
                };
            }
            Code::Label { label, pos: _ } => {
                if used_labels.contains(label) {
                    if new_block.len() > 0 {
                        res.push(new_block);
                    }
                    new_block = Vec::<&'a Code>::from([instr]);
                }
            }
        }
    }
    if new_block.len() > 0 {
        res.push(new_block);
    }
    res
}

// when a call is handled, add the return to subsequent block to the function's last block

#[derive(PartialEq, Eq)]
struct BlockPos<'a> {
    func: &'a Function,
    abs_num: usize, // the absolute block number
    rel_num: usize, // the block number within this function
}

struct BlockInfo<'a> {
    full_map: HashMap<&'a Function, Vec<Block<'a>>>,
    full_absolute: Vec<Block<'a>>,
    labelled: HashMap<String, Block<'a>>,
}

impl<'a> BlockInfo<'a> {
    fn label_to_blockno(&self, label: &String) -> usize {
        let req_block = self.labelled.get(label).unwrap();
        self.block_to_pos(req_block)
    }
    fn block_to_pos(&self, b: &Block) -> usize {
        self.full_absolute.iter().position(|x| x == b).unwrap()
    }

    fn func_to_firstblock(&self, f: &String) -> usize {
        for (func, vb) in self.full_map.iter() {
            if func.name == *f {
                return self.block_to_pos(vb.first().unwrap());
            }
        }
        return 0;
    }
}

type BlockPosList<'a> = Vec<BlockPos<'a>>;

type CFG = HashMap<usize, HashSet<usize>>;

// number beyond last block indicates return to end of main
fn form_cfg<'a>(info: BlockInfo) -> CFG {
    let mut res = CFG::new();
    let mut func_calls = HashMap::<String, HashSet<usize>>::new(); // which function is called plus where

    let mut returns_to_handle = HashMap::<String, HashSet<usize>>::new(); // which function returns, plus block containing return

    for (func, blocks) in info.full_map.iter() {
        println!("{}", func.name);
        for block in blocks {
            let mut block_res = HashSet::<usize>::new();
            let hash = info.block_to_pos(block);
            if let Code::Instruction(i) = block.get_terminator() {
                match i {
                    Instruction::Effect {
                        funcs, op, labels, ..
                    } => match op {
                        EffectOps::Jump | EffectOps::Branch => {
                            let ext: Vec<usize> =
                                labels.iter().map(|x| info.label_to_blockno(x)).collect();
                            block_res.extend(ext);
                        }
                        EffectOps::Call => {
                            let ext: Vec<usize> = funcs
                                .iter()
                                .map(|x| {
                                    if func_calls.contains_key(x) {
                                        func_calls.get_mut(x).unwrap().insert(hash);
                                    } else {
                                        func_calls.insert(x.clone(), HashSet::from([hash]));
                                    }
                                    info.func_to_firstblock(x)
                                })
                                .collect();
                            block_res.extend(ext);
                        }
                        EffectOps::Return => {
                            if returns_to_handle.contains_key(&func.name) {
                                returns_to_handle.get_mut(&func.name).unwrap().insert(hash);
                            } else {
                                returns_to_handle.insert(func.name.clone(), HashSet::from([hash]));
                            }
                        }
                        _ => (),
                    },
                    Instruction::Value { funcs, op, .. } => {
                        if let ValueOps::Call = op {
                            let ext: Vec<usize> = funcs
                                .iter()
                                .map(|x| {
                                    if func_calls.contains_key(x) {
                                        func_calls.get_mut(x).unwrap().insert(hash);
                                    } else {
                                        func_calls.insert(x.clone(), HashSet::from([hash]));
                                    }
                                    info.func_to_firstblock(x)
                                })
                                .collect();
                            block_res.extend(ext);

                            // index into full_map, get first block
                            // // also handle next block
                        }
                    }
                    _ => (),
                }
            }
            if res.contains_key(&hash) {
                res.get_mut(&hash).unwrap().extend(block_res);
            } else {
                res.insert(hash, block_res);
            }
        }
    }
    for (func, locs) in returns_to_handle.iter() {
        let calls = func_calls.get(func).unwrap();
        for loc in locs.iter() {
            res.get_mut(loc).unwrap().extend(calls);
        }
    }

    res
}

// TODO: add in matching of names to blocks

// cfg: handle returns?
// -> get callers
// -> some sort of good data struct to use?
// for block: get terminators
// determine where terminators lead (including the immediately subsequent block)
// return list of blocks and where they lead

// usage: target/debug/rust_stuff test.json
fn main() {
    let args = Args::parse();

    let filename = args
        .filename
        .expect("bad file! this shouldn't normally happen");

    let file = match File::open(filename.as_path()) {
        Err(why) => {
            eprintln!("couldn't open file: {}", why);
            std::process::exit(1)
        }
        Ok(file) => file,
    };

    let reader = BufReader::new(file);

    let v: bril_rs::Program = match serde_json::from_reader(reader) {
        Err(why) => panic!("{}", why),
        Ok(v) => v,
    };
    // extract functions

    let p = get_used_labels(&v);
    let mut blocks_map = HashMap::<&Function, Vec<Block>>::new();
    let mut all_blocks = Vec::<Block>::new();

    for f in v.functions.iter() {
        let basic_blocks = form_basic_blocks(f, &p);
        for block in basic_blocks.iter() {
            all_blocks.push(block.clone());
        }
        blocks_map.insert(f, basic_blocks);
    }

    for (f, blocks) in blocks_map.iter() {
        println!("blocks in func {}", f.name);

        for (i, block) in blocks.iter().enumerate() {
            println!("  block {}", i);
            for code in block.iter() {
                println!("   {}", code);
            }
        }
    }
    let mut labelled_blocks = HashMap::<String, Block>::new();
    for block in all_blocks.iter() {
        if let Code::Label { label, .. } = block[0] {
            labelled_blocks.insert(label.clone(), block.clone());
        }
    }

    let cfg_res = form_cfg(BlockInfo {
        full_map: blocks_map,
        full_absolute: all_blocks,
        labelled: labelled_blocks,
    });
    println!("{:?}", cfg_res);
    //println!("{:?}", blocks);
    //println!("{:?}", v.functions);
}
