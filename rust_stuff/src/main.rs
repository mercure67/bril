use bril_rs::*;
use clap::{Parser, Subcommand, ValueEnum};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::{fs::File, io::BufReader};

mod dce;
mod lvn;
mod mrange;
mod resolver;

#[derive(Subcommand)]
enum Task {
    DCE, // dead code elimination
    LVN,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum OpMode {
    Pipe,
    File,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    task: Task,

    #[arg(value_enum, long, default_value_t = OpMode::File, required = true)]
    mode: OpMode,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    filename: Option<std::path::PathBuf>,
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

    let v: bril_rs::Program = if args.mode == OpMode::Pipe {
        load_program()
    } else {
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

        match serde_json::from_reader(reader) {
            Err(why) => panic!("{}", why),
            Ok(v) => v,
        }
    };

    // extract functions
    let mut d = resolver::GlobalData::default();
    d.initial_fill(&v);
    d.form_blocks(&v);
    //d.print_blocks(&v);
    // d.print_blocks_compliance(&v);

    match args.task {
        Task::DCE => println!("{}", dce::global_dce(&v, &d)),
        Task::LVN => (),
    };

    let c = d.form_cfg(&v);
    // d.print_cfg(&v, &c);
    //println!("{:?}", blocks);
    //println!("{:?}", v.functions);
}
