use crate::resolver::*;
use bril_rs::*;

trait Pass {
    fn run(&self, p: &mut Program, data: &GlobalData);
}
