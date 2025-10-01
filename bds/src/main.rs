use bril_rs::*;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::{fs::File, io::BufReader};

use crate::constprop::ConstProp;
use crate::resolver::Defn;
use crate::util::CFGPos;

mod constprop;
mod data_flow;
mod dce;
mod global;
mod lvn;
mod pass;
mod reaching_defns;
mod resolver;
mod util;

#[derive(Subcommand)]
enum Task {
    CFG,
    DCE, // dead code elimination
    LVN,
    DFReaching,
    DFConst,
    Global,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    task: Task,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    filename: Option<std::path::PathBuf>,
}

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

    match args.task {
        Task::CFG => {
            let cfg = d.form_cfg();
            d.print_blocks();
            d.print_cfg(&cfg);
        }
        Task::DCE => println!("{}", dce::global_dce(&d)),
        Task::LVN => {
            let mut lvn = lvn::LVNTable::default();
            lvn.global_lvn(&mut d);
            d = resolver::GlobalData {
                data_map: HashMap::<String, resolver::FunctionData>::new(),
                program: &mut prog,
            };
            println!("{}", dce::global_dce(&d));
        }
        Task::DFReaching => {
            let mut w = data_flow::WorklistConfig::<Defn> {
                input_set: HashMap::<CFGPos, Vec<Defn>>::new(),
                output_set: HashMap::<CFGPos, Vec<Defn>>::new(),
                data: &mut d,
            };
            w.worklist();
            w.print();
        }
        Task::DFConst => {
            let mut w = data_flow::WorklistConfig::<ConstProp> {
                input_set: HashMap::<CFGPos, Vec<ConstProp>>::new(),
                output_set: HashMap::<CFGPos, Vec<ConstProp>>::new(),
                data: &mut d,
            };
            w.worklist();
            w.print();
        }
        Task::Global => {
            d.print_blocks();
            for (f, v) in d.data_map.iter() {
                let mut gl = global::DomMapping::default();
                gl.create_initial(v.funcno, v.blocks.len());
                let c = v.local_cfg(&d.program.functions[v.funcno]);
                d.print_cfg(&c);
                gl.find_dominators(&c);
                println!("func: {}", f);
                gl.print_mapping();
                println!();
                gl.create_tree(v.blocks.len());
                gl.print_tree();
                println!();

                gl.populate_dominance_frontier(&c);

                gl.print_frontier();
                println!();
            }
        }
    };
}
