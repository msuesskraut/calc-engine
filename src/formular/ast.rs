pub use crate::cells::{CellRef, Value};
use crate::formular::FormularError;

use std::collections::{HashMap, HashSet};

/// Binary operations of values
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Plus,
    Minus,
    Times,
    Div,
    Rem,
    Power,
}

impl Op {
    /// evaluates the binary operation self on the values lhs and rhs
    /// in the form of lhs $ rhs, where $ is the operation self.
    pub fn eval(&self, lhs: Value, rhs: Value) -> Value {
        let Value::Double(lhs) = lhs;
        let Value::Double(rhs) = rhs;
        match self {
            Op::Plus => Value::Double(lhs + rhs),
            Op::Minus => Value::Double(lhs - rhs),
            Op::Times => Value::Double(lhs * rhs),
            Op::Div => Value::Double(lhs / rhs),
            Op::Rem => Value::Double(lhs % rhs),
            Op::Power => Value::Double(lhs.powf(rhs)),
        }
    }
}

/// trait for structs that can calculated cell values
pub trait CellValueCalculator {
    /// returns the value of the cell referenced by cell_ref or an error
    fn get_cell_value(&self, cell_ref: &CellRef) -> Result<Value, FormularError>;
}

/// a cache of values referenced by CellRefs useful for testing
pub struct CellValueCache(HashMap<CellRef, Value>);

impl CellValueCache {
    pub fn new() -> CellValueCache {
        CellValueCache(HashMap::new())
    }

    pub fn add(&mut self, cr: CellRef, v: Value) {
        self.0.insert(cr, v);
    }
}

impl CellValueCalculator for CellValueCache {
    fn get_cell_value(&self, cell_ref: &CellRef) -> Result<Value, FormularError> {
        Ok(*self.0.get(cell_ref).unwrap_or(&Value::default()))
    }
}

/// expression in a formular
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    BinOp(Op, Box<Expr>, Box<Expr>),
    Cell(CellRef),
    Value(Value),
}

impl Expr {
    /// evaluates the expression self
    pub fn eval(
        &self,
        cell_value_calculator: &impl CellValueCalculator,
    ) -> Result<Value, FormularError> {
        match self {
            Expr::BinOp(op, lhs, rhs) => Ok(op.eval(
                lhs.eval(cell_value_calculator)?,
                rhs.eval(cell_value_calculator)?,
            )),
            Expr::Value(value) => Ok(*value),
            Expr::Cell(cell_ref) => cell_value_calculator.get_cell_value(cell_ref),
        }
    }

    pub fn calc_deps(&self) -> HashSet<CellRef> {
        fn traverse(e: &Expr, res: &mut HashSet<CellRef>) {
            match e {
                Expr::BinOp(_, lhs, rhs) => {
                    traverse(&*lhs, res);
                    traverse(&*rhs, res);
                }
                Expr::Cell(cell_ref) => {
                    res.insert(*cell_ref);
                }
                _ => (),
            };
        }
        let mut res = HashSet::new();

        traverse(self, &mut res);

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_eval_plus() {
        assert_eq!(
            Value::Double(5.0),
            Op::Plus.eval(Value::Double(2.0), Value::Double(3.0))
        );
    }

    #[test]
    fn op_eval_minus() {
        assert_eq!(
            Value::Double(1.0),
            Op::Minus.eval(Value::Double(3.0), Value::Double(2.0))
        );
    }

    #[test]
    fn op_eval_times() {
        assert_eq!(
            Value::Double(6.0),
            Op::Times.eval(Value::Double(2.0), Value::Double(3.0))
        );
    }

    #[test]
    fn op_eval_div() {
        assert_eq!(
            Value::Double(2.0),
            Op::Div.eval(Value::Double(6.0), Value::Double(3.0))
        );
    }

    #[test]
    fn op_eval_div_zero() {
        let Value::Double(res) = Op::Div.eval(Value::Double(5.0), Value::default());
        assert!(res.is_infinite());
    }

    #[test]
    fn op_eval_rem() {
        assert_eq!(
            Value::Double(1.0),
            Op::Rem.eval(Value::Double(10.0), Value::Double(3.0))
        );
    }

    #[test]
    fn op_eval_rem_zero() {
        let Value::Double(res) = Op::Rem.eval(Value::Double(6.0), Value::default());
        assert!(res.is_nan());
    }
    #[test]
    fn op_eval_pow() {
        assert_eq!(
            Value::Double(8.0),
            Op::Power.eval(Value::Double(2.0), Value::Double(3.0))
        );
    }

    #[test]
    fn eval_value() {
        assert_eq!(
            Value::Double(7.0),
            Expr::Value(Value::Double(7.0))
                .eval(&CellValueCache::new())
                .unwrap()
        );
    }

    #[test]
    fn eval_binop() {
        assert_eq!(
            Value::Double(5.0),
            Expr::BinOp(
                Op::Plus,
                Box::new(Expr::Value(Value::Double(2.0))),
                Box::new(Expr::Value(Value::Double(3.0)))
            )
            .eval(&CellValueCache::new())
            .unwrap()
        );
    }

    #[test]
    fn eval_cell_ref() {
        let mut cache = CellValueCache::new();
        cache.add(CellRef::new(2, 3), Value::Double(12.0));
        assert_eq!(
            Value::Double(12.0),
            Expr::Cell(CellRef::new(2, 3))
                .eval(&cache)
                .unwrap()
        );
    }

    #[test]
    fn calc_deps_simple() {
        let exp: HashSet<CellRef> = vec![CellRef::new(1, 1)].into_iter().collect();
        assert_eq!(exp, Expr::Cell(CellRef::new(1, 1)).calc_deps());
    }

    #[test]
    fn calc_deps_complex() {
        let exp: HashSet<CellRef> = vec![CellRef::new(1, 1), CellRef::new(2, 3)]
            .into_iter()
            .collect();
        assert_eq!(
            exp,
            Expr::BinOp(
                Op::Plus,
                Box::new(Expr::Cell(CellRef::new(1, 1))),
                Box::new(Expr::BinOp(
                    Op::Minus,
                    Box::new(Expr::Value(Value::default())),
                    Box::new(Expr::Cell(CellRef::new(2, 3)))
                ))
            )
            .calc_deps()
        );
    }

    #[test]
    fn calc_deps_double_cell_refs() {
        let exp: HashSet<CellRef> = vec![CellRef::new(7, 4)].into_iter().collect();
        assert_eq!(
            exp,
            Expr::BinOp(
                Op::Plus,
                Box::new(Expr::Cell(CellRef::new(7, 4))),
                Box::new(Expr::BinOp(
                    Op::Minus,
                    Box::new(Expr::Value(Value::default())),
                    Box::new(Expr::Cell(CellRef::new(7, 4)))
                ))
            )
            .calc_deps()
        );
    }
}
