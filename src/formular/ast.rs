pub use crate::cells::{CellRef, Value};

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

/// expression in a formular
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    BinOp(Op, Box<Expr>, Box<Expr>),
    Cell(CellRef),
    Value(Value),
}

impl Expr {
    /// evaluates the expression self
    ///
    /// # Panics
    ///
    /// `CellRef` is not yet supported.
    pub fn eval(&self) -> Value {
        match self {
            Expr::BinOp(op, lhs, rhs) => op.eval(lhs.eval(), rhs.eval()),
            Expr::Value(value) => *value,
            _ => unimplemented!(),
        }
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
        assert_eq!(Value::Double(7.0), Expr::Value(Value::Double(7.0)).eval());
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
            .eval()
        );
    }
}
