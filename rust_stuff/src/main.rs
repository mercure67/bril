use bril_rs::*;
use clap::Parser;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::{fs::File, io::BufReader};

mod lvn;
mod mrange;
mod resolver;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_hint = clap::ValueHint::FilePath, required=true)]
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
    /*
    let mut info = BlockInfo::default();
    info.populate(&v);
    info.display_blocks();

    let cfg_res = form_cfg(&info);
    //println!("{:?}", cfg_res);

    info.display_cfg(&cfg_res);
    */
    let mut d = resolver::GlobalData::default();
    d.initial_fill(&v);
    d.form_blocks(&v);
    d.print_blocks(&v);

    let c = d.form_cfg(&v);
    d.print_cfg(&v, &c);
    //println!("{:?}", blocks);
    //println!("{:?}", v.functions);
}
