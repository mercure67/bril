use crate::data_flow::DFDomainElement;
use crate::resolver::{Defn, GlobalData};
use crate::util::CFGPos;
use bril_rs::{Code};
use std::collections::HashSet;

impl DFDomainElement for Defn {
    fn merge(input_sets: &Vec<&Vec<Self>>) -> Vec<Self> {
        let mut set = HashSet::new();
        for v in input_sets {
            set.extend(v.iter().cloned());
        }
        let mut vec: Vec<Self> = set.into_iter().collect();
        vec.sort();
        vec
    }

    fn transfer(
        input_set: &Vec<Self>,
        code: &[Code],
        d: &GlobalData,
        pos: CFGPos,
    ) -> Vec<Self> {
        let mut set: HashSet<Self> = input_set.iter().cloned().collect();

        let fname = &d.program.functions[pos.funcno].name;

        for (l, def) in &d.get_func_data(&fname).unwrap().defns {
            set.retain(|d| d.var != def.var);
            set.insert(Defn::from(def.var.clone(), *l, fname.clone()));
        }

        let mut vec: Vec<Self> = set.into_iter().collect();
        vec.sort();
        vec
    }
}
