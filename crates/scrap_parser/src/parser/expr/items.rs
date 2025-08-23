use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::utils::LocalVec;
use super::{Expr, inline::expr_parser};
use crate::parser::{ScrapParser, ScrapInput};

pub fn items_parser<'tokens, 'src: 'tokens, I>()
-> impl ScrapParser<'tokens, 'src, I, LocalVec<Expr>>
where
    I: ScrapInput<'tokens, 'src>,
{
    expr_parser()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<LocalVec<_>>()
}
