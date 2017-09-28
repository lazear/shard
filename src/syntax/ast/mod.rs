use super::parser::{Parser, ParserError, ParserResult};
use super::token::Token;

pub mod select;
pub mod create;
pub mod columns;

pub trait Syntax: Sized {
    type Output;
    fn parse(parser: &mut Parser) -> ParserResult<Self::Output>;
}

struct CommaDelimited<R: Syntax>(R);
impl<R> SyntaxExt for R
where
    R: Syntax,
{
}

pub trait SyntaxExt: Syntax {
    fn parse_comma_delimited(parser: &mut Parser) -> ParserResult<Vec<Self::Output>> {
        CommaDelimited::<Self>::parse(parser)
    }
}

impl<R: Syntax> Syntax for CommaDelimited<R> {
    type Output = Vec<R::Output>;

    fn parse(parser: &mut Parser) -> ParserResult<Self::Output> {
        let mut v: Vec<R::Output> = Vec::new();
        v.push(R::parse(parser).unwrap());
        while parser.pop_if(&Token::COMMA) {
            let value = R::parse(parser).unwrap();
            v.push(value);
        }
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::lexer::Lexer;

    #[test]
    fn comma_delimited() {
        let mut parser = Lexer::lex("select row_1, row_2, row_3 from my_table where row_id > 0;")
            .unwrap();

        parser.pop().unwrap();
        let v = columns::Column::parse_comma_delimited(&mut parser).unwrap();
        let correct = vec![
            Token::Identifier("row_1".into()),
            Token::Identifier("row_2".into()),
            Token::Identifier("row_3".into()),
        ];

        assert_eq!(correct.len(), v.len());
        for (tok, _v) in correct.into_iter().zip(v.into_iter()) {
            match _v {
                columns::Column::Expr(t) => assert_eq!(t, tok),
                _ => panic!("Mismatch!"),
            };
        }
    }
}
