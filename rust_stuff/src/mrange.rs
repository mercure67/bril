// implemenets a coderange which can be changed, unlike the original program
pub struct ModifiableCodeRange {
    start: usize,
    end: usize,
    pub rem: Vec<usize>,
}

impl From<(usize, usize)> for ModifiableCodeRange {
    fn from(value: (usize, usize)) -> Self {
        ModifiableCodeRange {
            start: value.0,
            end: value.1,
            rem: (value.0..value.1).collect(),
        }
    }
}

impl ModifiableCodeRange {
    fn remove_rel(&mut self, idx: usize) {
        self.rem.remove(idx);
    }
}
