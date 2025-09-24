use bril_rs::*;
use crate::util::*;
use std::collections::HashMap;
use std::collections::HashSet;

/// Definition of a variable in a Bril program.
/// 
/// # Example
/// Consider this snippet on line 3 of a function `b`:
/// ```
/// a: int = const 5;
/// ```
/// This definition of `a` will be represented with name
/// `fbval3` and var `a`.
#[derive(Default, PartialEq, Eq, Hash, Clone, Debug, Ord, PartialOrd)]
pub struct Defn {
    /// Unique name of the definition.
    pub(crate) name: String,
    /// Non-unique name of the variable that the definition corresponds to.
    pub(crate) var: String,
}

impl Defn {
    /// Get a new definition from a variable name, line number, and function name.
    /// 
    /// # Arguments
    /// * `v` - name of the variable
    /// * `l` - line number of the definition, relative to the function
    /// * `f` - name of the function
    pub fn from(v: String, l: usize, f: String) -> Defn {
        Defn {
            name: format!("f{f}v{v}l{l}"),
            var: v,
        }
    }
}

// TODO: handle args

/// Metadata of a Bril function.
#[derive(Default)]
pub struct FunctionData {
    /// Unique function index, used to locate it within the `GlobalData` struct.
    pub funcno: usize,
    /// Set of basic blocks, inside or outside this function, that call this function.
    pub callers: HashSet<CFGPos>,
    /// Map of line numbers to function calls.
    pub calls: HashMap<usize, String>,
    /// Multimap of line numbers to definitions.
    pub defns: Vec<(usize, Defn)>,
    /// Set of code ranges corresponding to the basic blocks.
    pub blocks: Vec<CodeRange>,
    /// Map of labels and block indices that they correspond.
    pub labels: HashMap<String, Blockno>,
    /// Set of line numbers with return instructions.
    pub returns: Vec<usize>,
}

impl FunctionData {
    /// Populate the function metadata with information from the given Bril function.
    ///
    /// Fills in blocks, labels, definitions, calls, and returns.
    /// Does not update the `callers` or `funcno` fields.
    ///
    /// # Arguments
    /// * `f` - reference to the Bril function
    ///
    /// # Returns
    /// Set of pairs `(function_name, blockno)` for functions called by this function.
    pub fn populate(&mut self, f: &bril_rs::Function) -> HashSet<(String, Blockno)> {
        let mut block_range: CodeRange = (0, 0);
        let mut calls = HashSet::<(String, Blockno)>::new();
        let mut num_blocks = 0;

        for a in &f.args {
            self.defns
                .push((0, Defn::from(a.name.clone(), 0, f.name.clone())));
        }

        for (ino, instr) in f.instrs.iter().enumerate() {
            match instr {
                Code::Instruction(i) => {
                    block_range.1 = block_range.1 + 1;

                    let should_add_block = match i {
                        Instruction::Value { op, funcs, .. } => {
                            if let ValueOps::Call = op {
                                let func_called = funcs.first().unwrap();
                                calls.insert((func_called.clone(), num_blocks));
                                self.calls.insert(ino, func_called.clone());
                            }
                            false
                        }
                        Instruction::Effect { op, funcs, .. } => match op {
                            EffectOps::Return => {
                                self.returns.push(ino);
                                true
                            }
                            EffectOps::Jump | EffectOps::Branch => true,
                            EffectOps::Call => {
                                let func_called = funcs.first().unwrap();
                                calls.insert((func_called.clone(), num_blocks));
                                self.calls.insert(ino, func_called.clone());
                                false
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

                    if let Instruction::Constant { dest, .. } | Instruction::Value { dest, .. } = i {
                        self.defns
                            .push((ino, Defn::from(dest.to_string(), ino, f.name.clone())));
                    }
                }
                Code::Label { label, pos: _ } => {
                    if !self.labels.contains_key(label) {
                        if block_range.1 - block_range.0 > 0 {
                            self.blocks.push(block_range);
                            num_blocks = num_blocks + 1;
                        }
                        self.labels.insert(label.clone(), num_blocks);
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

/// Global metadata for a Bril program.
/// 
/// Stores data for all functions and provides utilities to analyze
/// and print control-flow and block-level information.
pub struct GlobalData<'a> {
    /// Map of function names to their metadata.
    pub data_map: HashMap<String, FunctionData>,
    /// Complete Bril program.
    pub program: &'a mut Program,
}

impl<'a> GlobalData<'a> {
    /// Get a reference to function metadata by name.
    ///
    /// # Arguments
    /// * `name` - function name
    pub fn get_func_data(&self, name: &String) -> Option<&FunctionData> {
        self.data_map.get(name)
    }

    /// Initialize `data_map` by creating entries for each function in the program.
    ///
    /// Panics if a function name is defined twice.
    pub fn initial_fill(&mut self) {
        for (fno, f) in self.program.functions.iter().enumerate() {
            if self.data_map.contains_key(&f.name) {
                panic!("function is defined twice");
            }
            let mut data = FunctionData::default();
            data.funcno = fno;
            self.data_map.insert(f.name.clone(), data);
        }
    }

    /// Populate block and call metadata for all functions.
    ///
    /// Updates `blocks`, `labels`, `defns`, `calls`, and `returns` for each function,
    /// and adds caller information to called functions.
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

    /// Print basic blocks of all functions with their instructions and metadata.
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

    /// Print basic blocks in a format similar to the Python Bril block script.
    pub fn print_blocks_compliance(&mut self) {
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

    /// Get a slice of instructions corresponding to a control-flow graph position.
    ///
    /// # Arguments
    /// * `pos` - CFG position
    pub fn get_codeslice(&'a self, pos: &CFGPos) -> &'a [Code] {
        let func = &self.program.functions[pos.funcno];
        let cr = self.get_func_data(&func.name).unwrap().blocks[pos.blockno];
        &func.instrs[cr.0..cr.1]
    }

    /// Construct the CFG for the whole program.
    ///
    /// Returns a CFG, which is a map from each block position to the set of successor blocks.
    pub fn form_cfg(&mut self) -> CFG {
        let mut res = CFG::new();

        for func in self.program.functions.iter() {
            let data = self.get_func_data(&func.name).unwrap();

            for (blockno, block) in data.blocks.iter().enumerate() {
                let mut block_res = HashSet::<CFGPos>::new();

                let (start, end) = *block;
                if start == end {
                    if blockno + 1 < data.blocks.len() {
                        block_res.insert(CFGPos {
                            funcno: data.funcno,
                            blockno: blockno + 1,
                        });
                    }
                } else {
                    let terminator = &func.instrs[end - 1];
                    if let Code::Instruction(instr) = terminator {
                        match instr {
                            Instruction::Effect { op, labels, .. } => match op {
                                EffectOps::Jump | EffectOps::Branch => {
                                    let ext: Vec<CFGPos> = labels
                                        .iter()
                                        .map(|x| CFGPos {
                                            funcno: data.funcno,
                                            blockno: data.labels[x],
                                        })
                                        .collect();
                                    block_res.extend(ext);
                                }
                                EffectOps::Return => {}
                                _ => {
                                    if blockno + 1 < data.blocks.len() {
                                        block_res.insert(CFGPos {
                                            funcno: data.funcno,
                                            blockno: blockno + 1,
                                        });
                                    }
                                }
                            },
                            Instruction::Value { .. } => {
                                if blockno + 1 < data.blocks.len() {
                                    block_res.insert(CFGPos {
                                        funcno: data.funcno,
                                        blockno: blockno + 1,
                                    });
                                }
                            }
                            _ => {
                                if blockno + 1 < data.blocks.len() {
                                    block_res.insert(CFGPos {
                                        funcno: data.funcno,
                                        blockno: blockno + 1,
                                    });
                                }
                            }
                        }
                    }
                }

                let k = CFGPos {
                    funcno: data.funcno,
                    blockno,
                };
                res.entry(k).or_default().extend(block_res);
            }
        }

        res
    }

    /// Print the CFG.
    ///
    /// # Arguments
    /// * `c` - reference to the CFG
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

    /// Get a list of all block positions across all functions.
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

    /// Get the function name corresponding to a function index.
    ///
    /// # Arguments
    /// * `fno` - function index
    pub fn funcname_from_funcno(&self, fno: usize) -> Option<String> {
        for (k, v) in self.data_map.iter() {
            if v.funcno == fno {
                return Some(k.clone());
            }
        }
        return None;
    }
}
