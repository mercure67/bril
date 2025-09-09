use bril_rs::*;
use clap::Parser;
use std::{collections::{HashMap, HashSet}, fs::File, io::BufReader};

enum Mode {
    
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required=true)]
    mode: Option<String>,

    #[arg(value_hint = clap::ValueHint::FilePath, required=true)]
    filename: Option<std::path::PathBuf>,
}

fn build_var_set(v: Program) -> HashMap<String, HashSet<String>> {
    let mut var_map = HashMap::new();
    for f in v.functions {
        let mut var_set = HashSet::new();
        for c in f.instrs {
            match c {
                Code::Instruction(Instruction::Constant { dest, .. })
                | Code::Instruction(Instruction::Value { dest, .. }) => {
                    var_set.insert(format!("{dest}"));

                }
                _ => {}
            }
        }
        var_map.insert(f.name, var_set);
    }
    return var_map;
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

    match args.mode.expect("") {
        mode => 
    }

    let set = build_var_set(v);
    print!("{:?}", set);
}
