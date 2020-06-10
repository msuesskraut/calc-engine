mod ast;
mod parser;

use crate::formular::ast::{CellRef, Expr, Value};
use crate::formular::parser::{build_expr, FormularError, FormularParser, Rule};

use pest::Parser;

#[derive(Clone, Debug)]
pub struct Formular {
    deps: Vec<CellRef>,
    expr: Box<Expr>,
}

impl Formular {
    pub fn new(s: &str) -> Result<Formular, FormularError> {
        let ast =
            FormularParser::parse(Rule::formular, s).map_err(FormularError::FormularParserError)?;
        let expr = build_expr(ast)?;
        let deps = Vec::new();
        Ok(Formular { expr, deps })
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
