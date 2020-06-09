#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Value {
    Double(f64),
}

impl Default for Value {
    fn default() -> Self {
        Self::Double(0.0f64)
    }
}

#[derive(Debug, Default)]
pub struct Cell {
    value: Value,
}

impl Cell {
    pub fn get_value(&self) -> Value {
        self.value
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Default, Clone, Copy)]
pub struct CellRef {
    pub r: usize,
    pub c: usize,
}

impl CellRef {
    pub fn new(r: usize, c: usize) -> Self {
        CellRef { r, c }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_default_is_double_zero() {
        assert_eq!(Value::Double(0.0f64), Value::default());
    }
}
