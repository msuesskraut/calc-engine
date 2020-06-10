use crate::cells::{CellRef, Value};
use crate::formular::ast::{Expr, Op};

use lazy_static::lazy_static;
use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "formular/formular.pest"]
pub struct FormularParser;

#[derive(Debug, PartialEq)]
pub enum FormularError {
    FormularParserError(Error<Rule>),
    CellRefParserError(String),
    ValueParserError(String),
}

fn parse_cell_ref_col(s: &str) -> Result<usize, FormularError> {
    s.chars().fold(Ok(0usize), |col, c| {
        if let Ok(col) = col {
            if !c.is_ascii_alphabetic() {
                Err(FormularError::CellRefParserError(format!(
                    "invalid column char {}",
                    c
                )))
            } else {
                Ok((col * 26) + ((c.to_ascii_uppercase() as usize) - ('A' as usize) + 1))
            }
        } else {
            col
        }
    })
}

fn parse_cell_ref(p: Pair<Rule>) -> Result<Box<Expr>, FormularError> {
    let mut row = 0usize;
    let mut col = 0usize;
    for p in p.into_inner() {
        match p.as_rule() {
            Rule::cell_ref_row => {
                row = p
                    .as_str()
                    .parse::<usize>()
                    .map_err(|e| FormularError::CellRefParserError(format!("{}", e)))?
            }
            Rule::cell_ref_col => col = parse_cell_ref_col(p.as_str())?,
            _ => unreachable!(),
        }
    }
    Ok(Box::new(Expr::Cell(CellRef::new(row, col))))
}

fn parse_value(p: Pair<Rule>) -> Result<Box<Expr>, FormularError> {
    let v = p
        .as_str()
        .parse::<f64>()
        .map_err(|e| FormularError::ValueParserError(format!("{}", e)))?;
    Ok(Box::new(Expr::Value(Value::Double(v))))
}

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Assoc::*;
        use Rule::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left) | Operator::new(rem, Left),
            Operator::new(power, Right),
        ])
    };
}

pub fn build_expr(ast: Pairs<Rule>) -> Result<Box<Expr>, FormularError> {
    PREC_CLIMBER.climb(
        ast,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::num => parse_value(pair),
            Rule::cell_ref => parse_cell_ref(pair),
            Rule::expr => build_expr(pair.into_inner()),
            _ => unreachable!(),
        },
        |lhs: Result<Box<Expr>, FormularError>,
         op: Pair<Rule>,
         rhs: Result<Box<Expr>, FormularError>| {
            let lhs = lhs?;
            let rhs = rhs?;
            match op.as_rule() {
                Rule::add => Ok(Box::new(Expr::BinOp(Op::Plus, lhs, rhs))),
                Rule::subtract => Ok(Box::new(Expr::BinOp(Op::Minus, lhs, rhs))),
                Rule::multiply => Ok(Box::new(Expr::BinOp(Op::Times, lhs, rhs))),
                Rule::divide => Ok(Box::new(Expr::BinOp(Op::Div, lhs, rhs))),
                Rule::rem => Ok(Box::new(Expr::BinOp(Op::Rem, lhs, rhs))),
                Rule::power => Ok(Box::new(Expr::BinOp(Op::Power, lhs, rhs))),
                _ => unreachable!(),
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cell_ref_col_uppercase() {
        assert_eq!(Ok(1), parse_cell_ref_col("A"));
        assert_eq!(Ok(6), parse_cell_ref_col("F"));
        assert_eq!(Ok(26), parse_cell_ref_col("Z"));
        assert_eq!(Ok(27), parse_cell_ref_col("AA"));
    }

    #[test]
    fn parse_cell_ref_col_lowercase() {
        assert_eq!(Ok(1), parse_cell_ref_col("a"));
        assert_eq!(Ok(6), parse_cell_ref_col("f"));
        assert_eq!(Ok(26), parse_cell_ref_col("z"));
        assert_eq!(Ok(27), parse_cell_ref_col("aa"));
    }

    #[test]
    fn parse_cell_ref_col_mixedcase() {
        assert_eq!(Ok(28), parse_cell_ref_col("aB"));
        assert_eq!(Ok(53), parse_cell_ref_col("Ba"));
    }
}
