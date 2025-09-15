use bril_rs::*;

use std::collections::HashMap;
use std::collections::HashSet;

type CodeRange = (usize, usize);
type Blockno = usize;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CFGPos {
    funcno: usize,
    blockno: Blockno,
}

impl std::fmt::Display for CFGPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("f{}.b{}", self.funcno, self.blockno))
    }
}

// TODO: handle args
#[derive(Default)]
pub struct FunctionData {
    funcno: usize,
    callers: HashSet<CFGPos>,      // function plus blockno of callers
    calls: HashMap<usize, String>, // map of line to function it calls
    blocks: Vec<CodeRange>,
    labels: HashMap<String, Blockno>,
    returns: Vec<usize>,
}

impl FunctionData {
    pub fn populate(&mut self, f: &bril_rs::Function) -> HashSet<(String, Blockno)> {
        // populate everything other than the callers and funcno.
        // returns a vector of all called function(s) in this function

        let mut block_range: CodeRange = (0, 0);
        let mut calls = HashSet::<(String, Blockno)>::new();
        let mut num_blocks = 0;

        for (ino, instr) in f.instrs.iter().enumerate() {
            match instr {
                Code::Instruction(i) => {
                    block_range.1 = block_range.1 + 1;

                    match i {
                        Instruction::Value { op, funcs, .. } => {
                            if let ValueOps::Call = op {
                                self.blocks.push(block_range);

                                // handle calls
                                let func_called = funcs.first().unwrap();
                                calls.insert((func_called.clone(), num_blocks));
                                self.calls.insert(ino, func_called.clone());

                                num_blocks = num_blocks + 1;
                                block_range = (block_range.1, block_range.1);
                            }
                        }
                        Instruction::Effect { op, funcs, .. } => match op {
                            EffectOps::Return => {
                                self.blocks.push(block_range);

                                // handle return
                                self.returns.push(ino);

                                num_blocks = num_blocks + 1;
                                block_range = (block_range.1, block_range.1);
                            }
                            EffectOps::Jump | EffectOps::Branch => {
                                self.blocks.push(block_range);
                                num_blocks = num_blocks + 1;

                                block_range = (block_range.1, block_range.1);
                            }
                            EffectOps::Call => {
                                self.blocks.push(block_range);

                                // handle calls
                                let func_called = funcs.first().unwrap();
                                calls.insert((func_called.clone(), num_blocks));

                                self.calls.insert(ino, func_called.clone());
                                num_blocks = num_blocks + 1;

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

type CFG = HashMap<CFGPos, HashSet<CFGPos>>;

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

    pub fn form_cfg(&mut self, p: &bril_rs::Program) -> CFG {
        let mut res = CFG::new();

        for func in p.functions.iter() {
            let data = self.data_map.get(&func.name).unwrap();
            for (blockno, block) in data.blocks.iter().enumerate() {
                let mut block_res = HashSet::<CFGPos>::new();
                let terminator = func.instrs.get(block.1 - 1).unwrap();
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
                                        funcno: self.data_map.get(x).unwrap().funcno,
                                        blockno: 0,
                                    })
                                    .collect();
                                if block != data.blocks.last().unwrap() {
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

                            _ => (),
                        },
                        Instruction::Value { funcs, op, .. } => {
                            if let ValueOps::Call = op {
                                let mut ext: Vec<CFGPos> = funcs
                                    .iter()
                                    .map(|x| CFGPos {
                                        funcno: self.data_map.get(x).unwrap().funcno,
                                        blockno: 0,
                                    })
                                    .collect();
                                if block != data.blocks.last().unwrap() {
                                    ext.push(CFGPos {
                                        funcno: data.funcno,
                                        blockno: blockno + 1,
                                    })
                                }

                                block_res.extend(ext);
                            }
                        }
                        _ => (),
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

    pub fn print_cfg(&self, p: &bril_rs::Program, c: &CFG) {
        for (k, v) in c {
            let curr_func = p.functions.get(k.funcno).unwrap();

            print!("{}: block {} ->", curr_func.name, k.blockno);
            for pos in v {
                let subname: &String = &p.functions.get(pos.funcno).unwrap().name;
                print!(" {}.b{}", subname, pos.blockno);
            }
            println!("");
        }
    }

    // number beyond last block indicates return to end of main
}

// TODO: a modified codeblock struct which allows insertion of deleted lines, and can be iterated over such that deleted lines don't show up
