use crate::data_flow::DFDomainElement;
use bril_rs::{Code, Instruction};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

// https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html apparently
static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn fresh(var: &str) -> String {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}{}", var, id)
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Defn {
    name: String,
    var: String,
}

impl DFDomainElement for Defn {
    fn merge(input_sets: &Vec<&Vec<Self>>) -> Vec<Self> {
        let mut set = HashSet::new();
        for v in input_sets {
            set.extend(v.iter().cloned());
        }
        set.into_iter().collect()
    }

    fn transfer(input_set: &Vec<Self>, code: &[Code]) -> Vec<Self> {
        let mut set: HashSet<Self> = input_set.iter().cloned().collect();
        for line in code {
            match line {
                Code::Instruction(Instruction::Constant { dest, .. })
                | Code::Instruction(Instruction::Value { dest, .. }) => {
                    set.retain(|d| d.var != *dest);
                    set.insert(Defn {
                        name: fresh(dest),
                        var: dest.to_string(),
                    });
                }
                _ => {}
            }
        }
        set.into_iter().collect()
    }
}
