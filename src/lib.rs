pub use power_assert_macros::*;

#[derive(Default)]
pub struct Formatter {
    values: Vec<(usize, usize, String)>,
}

impl Formatter {
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    pub fn push(&mut self, line: usize, column: usize, value: &impl std::fmt::Debug) {
        self.values.push((line, column, format!("{:?}", value)));
    }

    pub fn format(self) -> String {
        format!("{:?}", self.values)
    }
}
