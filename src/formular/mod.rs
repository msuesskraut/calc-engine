mod ast;
mod parser;

use crate::formular::ast::{CellRef, CellValueCalculator, Expr, Value};
use crate::formular::parser::{build_expr, FormularParser, Rule};

use pest::error::Error;
use pest::Parser;

use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub enum FormularError {
    FormularParserError(Error<Rule>),
    CellRefParserError(String),
    ValueParserError(String),
    EvalCycleError,
}

#[derive(Clone, Debug)]
pub struct Formular {
    deps: HashSet<CellRef>,
    expr: Box<Expr>,
}

impl Formular {
    pub fn new(s: &str) -> Result<Formular, FormularError> {
        let ast =
            FormularParser::parse(Rule::formular, s).map_err(FormularError::FormularParserError)?;
        let expr = build_expr(ast)?;
        let deps = expr.calc_deps();
        Ok(Formular { expr, deps })
    }

    pub fn eval(
        &self,
        cell_value_calculator: &impl CellValueCalculator,
    ) -> Result<Value, FormularError> {
        self.expr.eval(cell_value_calculator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::formular::ast::CellValueCache;

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
        assert_eq!(
            Value::Double(5.0),
            form.eval(&CellValueCache::new()).unwrap()
        );
    }
}
