use bril_rs::*;

use crate::util::*;

use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Default, PartialEq, Eq, Hash, Clone, Debug, Ord, PartialOrd)]
pub struct Defn {
    pub(crate) name: String,
    pub(crate) var: String, // var name it refers to
}

impl Defn {
    pub fn from(v: String, l: usize, f: String) -> Defn {
        Defn {
            name: format!("f{f}v{v}l{l}"),
            var: v,
        }
    }
}

// TODO: handle args
#[derive(Default)]
pub struct FunctionData {
    pub funcno: usize,
    pub callers: HashSet<CFGPos>, // function plus blockno of callers
    pub calls: HashMap<usize, String>, // map of line to function it calls
    pub defns: HashMap<usize, Defn>, // map of line to definition
    pub blocks: Vec<CodeRange>,
    pub labels: HashMap<String, Blockno>,
    pub returns: Vec<usize>,
}

impl FunctionData {
    pub fn populate(&mut self, f: &bril_rs::Function) -> HashSet<(String, Blockno)> {
        // populate everything other than the callers and funcno.
        // returns a vector of all called function(s) in this function

        let mut block_range: CodeRange = (0, 0);
        let mut calls = HashSet::<(String, Blockno)>::new();
        let mut num_blocks = 0;

        for a in &f.args {
            self.defns
                .insert(0, Defn::from(a.name.clone(), 0, f.name.clone()));
        }

        for (ino, instr) in f.instrs.iter().enumerate() {
            match instr {
                Code::Instruction(i) => {
                    block_range.1 = block_range.1 + 1;

                    let should_add_block = match i {
                        Instruction::Value { op, funcs, .. } => {
                            if let ValueOps::Call = op {
                                // handle calls
                                let func_called = funcs.first().unwrap();
                                calls.insert((func_called.clone(), num_blocks));
                                self.calls.insert(ino, func_called.clone());
                                true
                            } else {
                                false
                            }
                        }
                        Instruction::Effect { op, funcs, .. } => match op {
                            EffectOps::Return => {
                                // handle return
                                self.returns.push(ino);
                                true
                            }
                            EffectOps::Jump | EffectOps::Branch => true,
                            EffectOps::Call => {
                                // handle calls
                                let func_called = funcs.first().unwrap();
                                calls.insert((func_called.clone(), num_blocks));

                                self.calls.insert(ino, func_called.clone());
                                true
                            }
                            _ => false,
                        },
                        _ => false,
                    };
                    if should_add_block {
                        self.blocks.push(block_range);
                        num_blocks = num_blocks + 1;

                        block_range = (block_range.1, block_range.1);
                    }

                    if let Instruction::Constant { dest, .. } | Instruction::Value { dest, .. } = i
                    {
                        self.defns
                            .insert(ino, Defn::from(dest.to_string(), ino, f.name.clone()));
                    }
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

pub struct GlobalData<'a> {
    pub data_map: HashMap<String, FunctionData>,
    pub program: &'a mut Program,
}

impl<'a> GlobalData<'a> {
    pub fn get_func_data(&self, name: &String) -> Option<&FunctionData> {
        self.data_map.get(name)
    }

    pub fn initial_fill(&mut self) {
        // does not yet handle imports!
        // populate GlobalData: create a new data set for each function name
        for (fno, f) in self.program.functions.iter().enumerate() {
            if self.data_map.contains_key(&f.name) {
                panic!("function is defined twice");
            }
            let mut data = FunctionData::default();
            data.funcno = fno;
            self.data_map.insert(f.name.clone(), data);
        }
    }
    pub fn form_blocks(&mut self) {
        for f in self.program.functions.iter() {
            let fdata = self.data_map.get_mut(&f.name).unwrap();
            let calls = fdata.populate(f);
            let fno = fdata.funcno;

            for c in calls {
                self.data_map.entry(c.0).and_modify(|x| {
                    x.callers.insert(CFGPos {
                        funcno: fno,
                        blockno: c.1,
                    });
                });
            }
        }
    }

    pub fn print_blocks(&mut self) {
        for f in self.program.functions.iter() {
            let data = self.data_map.get(&f.name).unwrap();
            println!("function name: {} ({})", f.name, data.funcno);

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
    pub fn print_blocks_compliance(&mut self) {
        // print the basic blocks just like the basic blocks Python script would

        let func = &self.program.functions[0];
        let data = self.get_func_data(&func.name).unwrap();
        for block in data.blocks.iter() {
            for i in block.0..block.1 {
                let curr_instr: &Code = &func.instrs[i];
                if i == block.0 {
                    if let Code::Label { label, .. } = curr_instr {
                        println!("block {}:", label);
                        continue;
                    } else {
                        println!("anonymous block")
                    }
                }
                println!("{}", curr_instr);
            }
        }
    }

    pub fn get_codeslice(&'a self, pos: &CFGPos) -> &'a [Code] {
        let func = &self.program.functions[pos.funcno];
        let cr = self.get_func_data(&func.name).unwrap().blocks[pos.blockno];
        &func.instrs[cr.0..cr.1]
    }

    pub fn form_cfg(&mut self) -> CFG {
        let mut res = CFG::new();

        for func in self.program.functions.iter() {
            let data = self.get_func_data(&func.name).unwrap();
            for (blockno, block) in data.blocks.iter().enumerate() {
                let mut block_res = HashSet::<CFGPos>::new();
                let terminator = func.instrs.get(block.1 - 1).unwrap();

                // TODO: move block_res extension out
                if let Code::Instruction(instr) = terminator {
                    match instr {
                        Instruction::Effect {
                            funcs, op, labels, ..
                        } => match op {
                            EffectOps::Jump | EffectOps::Branch => {
                                let ext: Vec<CFGPos> = labels
                                    .iter()
                                    .map(|x| CFGPos {
                                        funcno: data.funcno,
                                        blockno: data.labels.get(x).unwrap().clone(),
                                    })
                                    .collect();
                                block_res.extend(ext);
                            }
                            EffectOps::Call => {
                                let mut ext: Vec<CFGPos> = funcs
                                    .iter()
                                    .map(|x| CFGPos {
                                        funcno: self.get_func_data(x).unwrap().funcno,
                                        blockno: 0,
                                    })
                                    .collect();
                                if blockno < data.blocks.len() - 1 {
                                    ext.push(CFGPos {
                                        funcno: data.funcno,
                                        blockno: blockno + 1,
                                    })
                                }

                                block_res.extend(ext);
                            }
                            EffectOps::Return => {
                                block_res.extend(data.callers.iter());
                            }

                            _ => {
                                if blockno < data.blocks.len() - 1 {
                                    block_res.insert(CFGPos {
                                        funcno: data.funcno,
                                        blockno: blockno + 1,
                                    });
                                }
                            }
                        },
                        Instruction::Value { funcs, op, .. } => {
                            if let ValueOps::Call = op {
                                let mut ext: Vec<CFGPos> = funcs
                                    .iter()
                                    .map(|x| CFGPos {
                                        funcno: self.get_func_data(x).unwrap().funcno,
                                        blockno: 0,
                                    })
                                    .collect();
                                if blockno < data.blocks.len() - 1 {
                                    ext.push(CFGPos {
                                        funcno: data.funcno,
                                        blockno: blockno + 1,
                                    })
                                }

                                block_res.extend(ext);
                            } else {
                                if blockno < data.blocks.len() - 1 {
                                    block_res.insert(CFGPos {
                                        funcno: data.funcno,
                                        blockno: blockno + 1,
                                    });
                                }
                            }
                        }
                        _ => {
                            if blockno < data.blocks.len() - 1 {
                                block_res.insert(CFGPos {
                                    funcno: data.funcno,
                                    blockno: blockno + 1,
                                });
                            }
                        }
                    }
                }
                let k = CFGPos {
                    funcno: data.funcno,
                    blockno: blockno,
                };
                if res.contains_key(&k) {
                    res.get_mut(&k).unwrap().extend(block_res);
                } else {
                    res.insert(k, block_res);
                }
            }
        }

        res
    }

    pub fn print_cfg(&self, c: &CFG) {
        for (k, v) in c {
            let curr_func = self.program.functions.get(k.funcno).unwrap();

            print!("{}: block {} ->", curr_func.name, k.blockno);
            for pos in v {
                let subname: &String = &self.program.functions.get(pos.funcno).unwrap().name;
                print!(" {}.b{}", subname, pos.blockno);
            }
            println!("");
        }
    }

    // number beyond last block indicates return to end of main

    pub fn all_blocks(&self) -> Vec<CFGPos> {
        let mut res = Vec::<CFGPos>::new();

        let mut funcno = 0;
        for (_, v) in self.data_map.iter() {
            for (blockno, _) in v.blocks.iter().enumerate() {
                res.push(CFGPos {
                    funcno: v.funcno,
                    blockno: blockno,
                });
            }
            funcno = funcno + 1;
        }
        res
    }

    pub fn funcname_from_funcno(&self, fno: usize) -> Option<String> {
        for (k, v) in self.data_map.iter() {
            if v.funcno == fno {
                return Some(k.clone());
            }
        }
        return None;
    }
}
