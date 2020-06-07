use crate::cells::{Cell, CellRef, Value};

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Table {
    cells: HashMap<CellRef, Cell>,
}

impl Table {
    fn get_value(&self, cr: &CellRef) -> Value {
        self.cells
            .get(cr)
            .map(|c| c.get_value())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_are_initially_default() {
        assert_eq!(
            Value::default(),
            Table::default().get_value(&CellRef::new(12, 34))
        );
    }
}
