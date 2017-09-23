use super::parser::{Parser, ParserError, ParserResult};
pub mod select;
pub mod create;

pub trait Syntax {
    type Output;
    fn parse(parser: &mut Parser) -> ParserResult<Self::Output>
}