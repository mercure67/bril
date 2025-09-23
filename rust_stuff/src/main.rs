use bril_rs::*;
use clap::{Parser, Subcommand, ValueEnum};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::{fs::File, io::BufReader};

mod data_flow;
mod dce;
mod lvn;
mod resolver;
mod util;
mod reaching_defns;

#[derive(Subcommand)]
enum Task {
    DCE, // dead code elimination
    LVN,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    task: Task,

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

    // https://stackoverflow.com/questions/71885543/accept-optional-file-on-command-line-default-to-stdin

    let mut prog: bril_rs::Program = if let Some(filename) = args.filename {
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
    } else {
        load_program()
    };

    // extract functions
    let mut d = resolver::GlobalData {
        data_map: HashMap::<String, resolver::FunctionData>::new(),
        program: &mut prog,
    };
    d.initial_fill();
    d.form_blocks();
    //d.print_blocks(&v);
    // d.print_blocks_compliance(&v);

    match args.task {
        Task::DCE => println!("{}", dce::global_dce(&d)),
        Task::LVN => {
            let mut lvn = lvn::LVNTable::default();
            lvn.global_lvn(&mut d);
            d = resolver::GlobalData {
                data_map: HashMap::<String, resolver::FunctionData>::new(),
                program: &mut prog,
            };

            d.initial_fill();
            d.form_blocks();
            println!("{}", dce::global_dce(&d));
        }
    };

    //let c = d.form_cfg(&v);
    // d.print_cfg(&v, &c);
    //println!("{:?}", blocks);
    //println!("{:?}", v.functions);
}
