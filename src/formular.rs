use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "formular.pest"]
pub struct FormularParser;

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
}
