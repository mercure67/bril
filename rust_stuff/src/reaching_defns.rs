use bril_rs::Code;
use std::collections::HashSet;

use crate::data_flow::DFDomainElement;

#[derive(PartialEq, Eq, Hash, Clone)]
struct Defn {
    name: String,
    var: String,
    var_type: bril_rs::Type,
}

impl DFDomainElement for Defn {
    fn merge(input_sets: &[Vec<Self>]) -> Vec<Self> {
        let mut set = HashSet::new();
        for v in input_sets {
            set.extend(v.iter().cloned());
        }
        set.into_iter().collect()
    }

    fn transfer(input_set: &[Self], code: &[Code]) -> Vec<Self> {
        todo!()
    }
}
