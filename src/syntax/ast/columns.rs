use super::*;

#[derive(Debug)]
pub enum Column {
    All,
    Expr(Token)
}

impl Syntax for Column {
    type Output = Column;
    fn parse(parser: &mut Parser) -> ParserResult<Self::Output> {
        if parser.pop_if(&Token::ASTERISK) {
            Ok(Column::All)
        } else {
            let column = parser.pop()?;
            Ok(Column::Expr(column))
        }
    }
}