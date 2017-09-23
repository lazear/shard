use super::parser::{Parser, ParserError, ParserResult};
use super::token::Token;

pub mod select;
pub mod create;

pub trait Syntax {
    type Output;
    fn parse(parser: &mut Parser) -> ParserResult<Self::Output>;
}