use bril_rs::*;
use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::{fs::File, io::BufReader, rc::Rc};

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
    //

    // TODO: do labels need to be unique?
    let mut res = HashSet::<String>::new();

    for f in v.functions.iter() {
        for i in f.instrs.iter() {
            //TODO: the ergonomics of this part of the library are poor
            if let Code::Instruction(inner) = i {
                if let Instruction::Effect {
                    args: _,
                    funcs,
                    labels,
                    op,
                    pos: _,
                } = inner
                {
                    match op {
                        EffectOps::Jump => {
                            for x in labels.iter() {
                                res.insert(x.clone());
                            }
                        }
                        EffectOps::Branch => {
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
                if let Instruction::Value {
                    args: _,
                    dest: _,
                    funcs,
                    labels: _,
                    op,
                    pos: _,
                    op_type: _,
                } = inner
                {
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

type Block<'a> = (&'a String, Vec<&'a Code>);

fn form_basic_blocks<'a>(v: &'a bril_rs::Program, used_labels: &HashSet<String>) -> Vec<Block<'a>> {
    let mut res = Vec::<Block>::new();
    for f in v.functions.iter() {
        let mut new_block = Vec::<&'a Code>::new();
        for instr in f.instrs.iter() {
            match instr {
                Code::Instruction(i) => {
                    new_block.push(instr);
                    match i {
                        Instruction::Value {
                            args: _,
                            dest: _,
                            funcs: _,
                            labels: _,
                            op,
                            pos: _,
                            op_type: _,
                        } => {
                            if let ValueOps::Call = op {
                                res.push((&f.name, new_block));
                                new_block = Vec::<&'a Code>::new();
                            }
                        }
                        Instruction::Effect {
                            args: _,
                            funcs: _,
                            labels: _,
                            op,
                            pos: _,
                        } => match op {
                            EffectOps::Jump
                            | EffectOps::Branch
                            | EffectOps::Call
                            | EffectOps::Return => {
                                res.push((&f.name, new_block));
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
                            res.push((&f.name, new_block));
                        }
                        new_block = Vec::<&'a Code>::from([instr]);
                    }
                }
            }
        }
        if new_block.len() > 0 {
            res.push((&f.name, new_block));
        }
        //println!("{:?}", f.instrs);
    }
    //println!("{:?}", used_labels);
    res
}

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
    let blocks = form_basic_blocks(&v, &p);
    for (i, block) in blocks.iter().enumerate() {
        println!("block: {} in func {}", i, block.0);

        for code in block.1.iter() {
            println!("{}", code);
        }
    }
    //println!("{:?}", blocks);
    //println!("{:?}", v.functions);
}
