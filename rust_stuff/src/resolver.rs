use bril_rs::*;

use std::collections::HashMap;
use std::collections::HashSet;

type CodeRange = (usize, usize);
type Blockno = usize;

// TODO: handle args
#[derive(Default)]
pub struct FunctionData {
    funcno: usize,
    callers: Vec<String>,          // function names of callers
    calls: HashMap<usize, String>, // map of line to function it calls
    blocks: Vec<CodeRange>,
    labels: HashMap<String, Blockno>,
    returns: Vec<usize>,
}

impl FunctionData {
    pub fn populate(&mut self, f: &bril_rs::Function) -> HashSet<String> {
        // populate everything other than the callers and funcno.
        // returns a vector of all called function(s) in this function

        let mut block_range: CodeRange = (0, 0);
        let mut calls = HashSet::<String>::new();
        let mut num_blocks = 0;

        for (ino, instr) in f.instrs.iter().enumerate() {
            match instr {
                Code::Instruction(i) => {
                    block_range.1 = block_range.1 + 1;

                    match i {
                        Instruction::Value { op, funcs, .. } => {
                            if let ValueOps::Call = op {
                                self.blocks.push(block_range);
                                num_blocks = num_blocks + 1;

                                // handle calls
                                let func_called = funcs.first().unwrap();
                                calls.insert(func_called.clone());
                                self.calls.insert(ino, func_called.clone());

                                block_range = (block_range.1, block_range.1);
                            }
                        }
                        Instruction::Effect { op, funcs, .. } => match op {
                            EffectOps::Return => {
                                self.blocks.push(block_range);
                                num_blocks = num_blocks + 1;

                                // handle return
                                self.returns.push(ino);

                                block_range = (block_range.1, block_range.1);
                            }
                            EffectOps::Jump | EffectOps::Branch => {
                                self.blocks.push(block_range);
                                num_blocks = num_blocks + 1;

                                block_range = (block_range.1, block_range.1);
                            }
                            EffectOps::Call => {
                                self.blocks.push(block_range);
                                num_blocks = num_blocks + 1;

                                // handle calls
                                let func_called = funcs.first().unwrap();
                                calls.insert(func_called.clone());
                                self.calls.insert(ino, func_called.clone());

                                block_range = (block_range.1, block_range.1);
                            }
                            _ => (),
                        },
                        _ => (),
                    };
                }
                Code::Label { label, pos: _ } => {
                    if !self.labels.contains_key(label) {
                        // TODO: safety: panic on duplicated labels
                        if block_range.1 - block_range.0 > 0 {
                            self.blocks.push(block_range);
                            num_blocks = num_blocks + 1;
                        }
                        self.labels.insert(label.clone(), num_blocks); // TODO: check logic here: for a block which hasn't been created

                        block_range = (block_range.1, block_range.1 + 1);
                    }
                }
            }
        }
        if block_range.1 - block_range.0 > 0 {
            self.blocks.push(block_range);
        }
        calls
    }
}

#[derive(Default)]
pub struct GlobalData {
    data_map: HashMap<String, FunctionData>,
}

impl GlobalData {
    pub fn initial_fill(&mut self, p: &bril_rs::Program) {
        // does not yet handle imports!
        //
        // populate GlobalData: create a new data set for each function name
        for (fno, f) in p.functions.iter().enumerate() {
            if self.data_map.contains_key(&f.name) {
                panic!("function is defined twice");
            }
            let mut data = FunctionData::default();
            data.funcno = fno;
            self.data_map.insert(f.name.clone(), data);
        }
    }
    pub fn form_blocks(&mut self, p: &bril_rs::Program) {
        for f in p.functions.iter() {
            let fdata = self.data_map.get_mut(&f.name).unwrap();
            let calls = fdata.populate(f);
            for c in calls {
                self.data_map
                    .entry(c)
                    .and_modify(|x| x.callers.push(f.name.clone()));
            }
        }
    }

    pub fn print_blocks(&mut self, p: &bril_rs::Program) {
        for f in p.functions.iter() {
            println!("function name: {}", f.name);
            let data = self.data_map.get(&f.name).unwrap();
            let mut lineno = 0;
            for (bno, b) in data.blocks.iter().enumerate() {
                println!("  blockno: {}", bno);
                for i in b.0..b.1 {
                    println!("   l{}  {}", lineno, f.instrs[i]);
                    lineno = lineno + 1;
                }
            }
            println!("callers: {:?}", data.callers);
            println!("calls: {:?}", data.calls);
            println!("labels: {:?}", data.labels);
            println!("returns: {:?}", data.returns);
            println!("");
        }
    }

    // TODO: need to have some means of adding function scope
    // number beyond last block indicates return to end of main
    // TODO: implement CFG
}

// TODO: a modified codeblock struct which allows insertion of deleted lines, and can be iterated over such that deleted lines don't show up
