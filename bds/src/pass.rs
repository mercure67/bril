use crate::resolver::*;
use bril_rs::*;

// an informational pass:
// operates on a given dataset (ideally without mutation)
// after calling Run, the thing will contain whatever information is necessary
trait InfoPass {
    fn run(&self, data: &GlobalData);
    fn is_populated() -> bool;
}

// an optimisation pass:
// operates on a given dataset, cloning the information
// additionally accepts some T
// T must be prepopulated (hence the is_populated)
// then, the result can be exported as a GlobalData.
trait OptPass<T: InfoPass> {
    fn run(&self, data: &GlobalData, info: &T);
    fn export(&self) -> GlobalData;
}
