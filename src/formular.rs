use crate::cells::{Value, CellRef};

use lazy_static::lazy_static;
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::error::Error;
use pest_derive::Parser;

use std::num::{ParseIntError, ParseFloatError};

#[derive(Parser)]
#[grammar = "formular.pest"]
struct FormularParser;

#[derive(Debug)]
pub enum FormularError {
    FormularParserError(Error<Rule>),
    CellRefParserError(String),
    ValueParserError(String)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Plus,
    Minus,
    Times,
    Div,
    Rem,
    Power,
}

impl Op {
    fn eval(&self, lhs: Value, rhs: Value) -> Value {
        if let Value::Double(lhs) = lhs {
            if let Value::Double(rhs) = rhs {
                match self {
                    Op::Plus => Value::Double(lhs + rhs),
                    Op::Minus => Value::Double(lhs - rhs),
                    Op::Times => Value::Double(lhs * rhs),
                    Op::Div => Value::Double(lhs / rhs),
                    Op::Rem => Value::Double(lhs % rhs),
                    Op::Power => Value::Double(lhs.powf(rhs)),
                }
            } else {
                Value::Double(f64::NAN)
            }
        } else {
            Value::Double(f64::NAN)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Expr {
    BinOp(Op, Box<Expr>, Box<Expr>),
    Cell(CellRef),
    Value(Value),    
}

impl Expr {
    pub fn eval(&self) -> Value {
        match self {
            Expr::BinOp(op, lhs, rhs) => op.eval(lhs.eval(), rhs.eval()),
            Expr::Value(value) => *value,
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Formular {
    deps: Vec<CellRef>,
    expr: Box<Expr>,
}

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Rule::*;
        use Assoc::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left) | Operator::new(rem, Left),
            Operator::new(power, Right)
        ])
    };
}

fn parse_cell_ref_col(s: &str) -> Result<usize, FormularError> {
    s.chars()
        .fold(
            Ok(0usize),
            |col, c| if let Ok(col) = col {
                if !c.is_ascii_alphabetic() {
                    Err(FormularError::CellRefParserError(format!("invalid column char {}", c)))
                } else {
                    Ok(col + ((c.to_ascii_uppercase() as usize) - ('A' as usize)))
                }
            } else {
                col
            }
        )
}

fn parse_cell_ref(p: Pair<Rule>) -> Result<Box<Expr>, FormularError> {
    let mut row = 0usize;
    let mut col = 0usize;
    for p in p.into_inner() {
        match p.as_rule() {
            Rule::cell_ref_row => row = p.as_str().parse::<usize>().map_err(|e| FormularError::CellRefParserError(format!("{}", e)) )?,
            Rule::cell_ref_col => col = parse_cell_ref_col(p.as_str())?,
            _ => unreachable!(),
        }
    }
    Ok(Box::new(Expr::Cell(CellRef::new(row, col))))
}

fn parse_value(p: Pair<Rule>) -> Result<Box<Expr>, FormularError> {
    let v = p.as_str().parse::<f64>().map_err(|e| FormularError::ValueParserError(format!("{}", e)))?;
    Ok(Box::new(Expr::Value(Value::Double(v))))
}

fn build_expr(ast: Pairs<Rule>) -> Result<Box<Expr>, FormularError> {
    PREC_CLIMBER.climb(
        ast,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::num => parse_value(pair),
            Rule::cell_ref => parse_cell_ref(pair),
            Rule::expr => build_expr(pair.into_inner()),
            _ => unreachable!(),
        },
        |lhs: Result<Box<Expr>, FormularError>, op: Pair<Rule>, rhs: Result<Box<Expr>, FormularError>| {
            let lhs = lhs?;
            let rhs = rhs?;
            match op.as_rule() {
                Rule::add      => Ok(Box::new(Expr::BinOp(Op::Plus, lhs, rhs))),
                Rule::subtract => Ok(Box::new(Expr::BinOp(Op::Minus, lhs, rhs))),
                Rule::multiply => Ok(Box::new(Expr::BinOp(Op::Times, lhs, rhs))),
                Rule::divide   => Ok(Box::new(Expr::BinOp(Op::Div, lhs, rhs))),
                Rule::rem      => Ok(Box::new(Expr::BinOp(Op::Rem, lhs, rhs))),
                Rule::power    => Ok(Box::new(Expr::BinOp(Op::Power, lhs, rhs))),
                _ => unreachable!(),
            }
        },
    )
}

impl Formular {
    pub fn new(s: &str) -> Result<Formular, FormularError> {
        let ast = FormularParser::parse(Rule::formular, s).map_err(|e| FormularError::FormularParserError(e))?;
        let expr = build_expr(ast)?;
        let deps = Vec::new();
        Ok(Formular {
            expr,
            deps,
        })
    }

    pub fn eval(&self) -> Value {
        self.expr.eval()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_formular() {
        assert!(FormularParser::parse(Rule::formular, "1 + 2 * A1").is_ok());
    }

    #[test]
    fn parse_formular_error() {
        assert!(FormularParser::parse(Rule::formular, "1 + 2 * 1A").is_err());
    }

    #[test]
    fn form_eval() {
        let form = Formular::new("1 + 2 * 3 - 2 ^ (3 - 2)").unwrap();
        assert_eq!(Value::Double(5.0), form.eval());
    }
}
