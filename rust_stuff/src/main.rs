use bril_rs::*;
use clap::Parser;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
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
) -> Vec<BlockRef<'a>> {
    let mut blocks = Vec::<Rc<Block<'a>>>::new();
    let mut new_block = Block::new();
    for instr in f.instrs.iter() {
        match instr {
            Code::Instruction(i) => {
                new_block.push(instr);
                match i {
                    Instruction::Value { op, .. } => {
                        if let ValueOps::Call = op {
                            blocks.push(Rc::new(new_block));
                            new_block = Vec::<&'a Code>::new();
                        }
                    }
                    Instruction::Effect { op, .. } => match op {
                        EffectOps::Jump
                        | EffectOps::Branch
                        | EffectOps::Call
                        | EffectOps::Return => {
                            blocks.push(Rc::new(new_block));
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
                        let nrc = Rc::new(new_block);
                        blocks.push(Rc::clone(&nrc));
                    }
                    new_block = Vec::<&'a Code>::from([instr]);
                }
            }
        }
    }
    if new_block.len() > 0 {
        blocks.push(Rc::new(new_block));
    }
    blocks
}

// when a call is handled, add the return to subsequent block to the function's last block

type BlockRef<'a> = Rc<Block<'a>>;
type BlockRefSet<'a> = Vec<BlockRef<'a>>;

struct BlockInfo<'a> {
    full_map: HashMap<&'a Function, BlockRefSet<'a>>,
    full_absolute: Vec<BlockRef<'a>>,
    labelled: HashMap<String, BlockRef<'a>>,
}

impl<'a> Default for BlockInfo<'a> {
    fn default() -> Self {
        Self {
            full_map: HashMap::<&Function, BlockRefSet>::new(),
            full_absolute: Vec::<BlockRef>::new(),
            labelled: HashMap::<String, BlockRef>::new(),
        }
    }
}

impl<'a> BlockInfo<'a> {
    fn populate(&mut self, v: &'a bril_rs::Program) {
        let p = get_used_labels(&v);

        for f in v.functions.iter() {
            let basic_blocks = form_basic_blocks(f, &p);
            let full_map_copy: Vec<BlockRef> = basic_blocks.iter().map(|x| Rc::clone(x)).collect();

            for block in basic_blocks.iter() {
                if let Code::Label { label, .. } = block.first().unwrap() {
                    self.labelled.insert(label.clone(), Rc::clone(&block));
                }
            }
            self.full_map.insert(&f, full_map_copy);

            self.full_absolute.extend(basic_blocks);
        }
    }

    fn label_to_blockno(&self, label: &String) -> usize {
        let req_block = self.labelled.get(label).unwrap();
        self.block_to_pos(req_block)
    }

    fn block_to_pos(&self, b: &BlockRef) -> usize {
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
    fn display_cfg(&self, c: &CFG) {
        for (key, val) in c {
            println!(
                "block {}: {:?}",
                *key,
                self.full_absolute.get(*key).unwrap()
            );
            println!("-> blocks: {:?}", val);
        }
    }

    fn display_blocks(&self) {
        for (f, blocks) in self.full_map.iter() {
            println!("blocks in func {}", f.name);

            for (i, block) in blocks.iter().enumerate() {
                println!("  block {}", i);
                for code in block.iter() {
                    println!("   {}", code);
                }
            }
        }
    }
}

type CFG = HashMap<usize, HashSet<usize>>;

// number beyond last block indicates return to end of main
fn form_cfg<'a>(info: &BlockInfo) -> CFG {
    let mut res = CFG::new();
    let mut func_calls = HashMap::<String, HashSet<usize>>::new(); // which function is called plus where

    let mut returns_to_handle = HashMap::<String, HashSet<usize>>::new(); // which function returns, plus block containing return

    for (func, blocks) in info.full_map.iter() {
        let func_max = info
            .full_absolute
            .iter()
            .position(|x| x == blocks.last().unwrap())
            .unwrap();
        for block in blocks {
            let mut block_res = HashSet::<usize>::new();
            let hash = info.full_absolute.iter().position(|x| x == block).unwrap();
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
                            let mut ext: Vec<usize> = funcs
                                .iter()
                                .map(|x| {
                                    func_calls
                                        .entry(x.clone())
                                        .and_modify(|x| {
                                            x.insert(hash);
                                        })
                                        .or_insert(HashSet::from([hash]));
                                    info.func_to_firstblock(x)
                                })
                                .collect();
                            let mut nextblock = info.block_to_pos(block);
                            if nextblock != func_max {
                                nextblock = nextblock + 1;
                            }
                            ext.push(nextblock);

                            block_res.extend(ext);
                        }
                        EffectOps::Return => {
                            returns_to_handle
                                .entry(func.name.clone())
                                .and_modify(|x| {
                                    x.insert(hash);
                                })
                                .or_insert(HashSet::from([hash]));
                        }

                        _ => (),
                    },
                    Instruction::Value { funcs, op, .. } => {
                        if let ValueOps::Call = op {
                            let mut ext: Vec<usize> = funcs
                                .iter()
                                .map(|x| {
                                    func_calls
                                        .entry(x.clone())
                                        .and_modify(|x| {
                                            x.insert(hash);
                                        })
                                        .or_insert(HashSet::from([hash]));

                                    info.func_to_firstblock(x)
                                })
                                .collect();
                            ext.push(info.block_to_pos(block) + 1);
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

    let mut info = BlockInfo::default();
    info.populate(&v);
    info.display_blocks();

    /*
     */

    let cfg_res = form_cfg(&info);
    //println!("{:?}", cfg_res);

    info.display_cfg(&cfg_res);
    //println!("{:?}", blocks);
    //println!("{:?}", v.functions);
}
