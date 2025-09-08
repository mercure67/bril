use bril_rs::*;
use clap::{Parser, Subcommand};
use std::{fs::File, io::BufReader};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_hint = clap::ValueHint::FilePath, required=true)]
    filename: Option<std::path::PathBuf>,
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

    for f in v.functions {
        println!("{:?}", f.instrs);
    }

    //println!("{:?}", v.functions);
}
