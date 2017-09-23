use super::*;

struct Select(u8);

impl Syntax for Select {
    type Output = Self;
    fn parse(parser: &mut Parser) -> ParserResult<Select> {
        parser.expect(&Token::SELECT)?;
        Ok(Select(10))
    }
}