use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use chumsky::{
    input::{Stream, ValueInput},
    prelude::*,
};
use scrap_lexer::Token;

use crate::Span;
use crate::ast::*;

pub fn parse_ident<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Ident, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    select! {
        Token::Ident(s) => s,
    }
    .map_with(|s, e| Ident {
        name: s.to_string(),
        span: e.span(),
    })
}

fn capital_ident<'tokens, 'src: 'tokens, I>(
    err_msg: &'static str,
) -> impl Parser<'tokens, I, Ident, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parse_ident().validate(move |id, _, emitter| {
        if !id.name.chars().next().unwrap().is_uppercase() {
            emitter.emit(Rich::custom(id.span, err_msg));
        }

        id
    })
}

pub fn parse_type<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Type, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parse_ident().map_with(|ident, _| Type(ident))
}

pub fn fields<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Vec<(Ident, Type)>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parse_ident()
        .then_ignore(just(Token::Colon))
        .then(parse_type())
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .validate(|args, _, emitter| {
            let mut field_name = HashSet::new();

            args.iter().for_each(|(ident, _)| {
                if !field_name.insert(ident.name.clone()) {
                    emitter.emit(Rich::custom(
                        ident.span,
                        format!("duplicate identifier '{}'", ident.name),
                    ));
                }
            });
            args
        })
}

pub fn struct_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Item<'src>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let fields = fields()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .labelled("struct fields");

    just(Token::Struct)
        .ignore_then(parse_ident().labelled("struct name"))
        .then(fields)
        .map_with(|(name, fields), e| Item {
            kind: ItemKind::Struct(StructDef {
                _p: PhantomData,
                ident: name,
                fields,
            }),
            span: e.span(),
        })
        .labelled("struct")
}

pub fn enum_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Item<'src>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let err_msg = "Enum variant name must start with an uppercase letter";

    let variant = capital_ident(err_msg)
        .then(
            parse_type()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .or_not(),
        )
        .map(|(ident, ty)| {
            if let Some(ty) = ty {
                EnumVariant::Full { name: ident, ty }
            } else {
                EnumVariant::Unit(ident)
            }
        })
        .separated_by(just(Token::Comma))
        .allow_trailing();

    just(Token::Enum)
        .ignore_then(
            capital_ident("Enum name must start with an uppercase letter").labelled("enum name"),
        )
        .then(variant.delimited_by(just(Token::LBrace), just(Token::RBrace)))
        .map_with(|(name, variants), e| Item {
            kind: ItemKind::Enum(EnumDef {
                _p: PhantomData,
                ident: name,
                variants: Vec::new(),
            }),
            span: e.span(),
        })
        .labelled("enum")
}

pub fn function_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Item<'src>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let args = fields()
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .labelled("function args");

    let body_recover = ignore_block().recover_with(via_parser(nested_delimiters(
        Token::LBracket,
        Token::RBracket,
        [
            (Token::LParen, Token::RParen),
            (Token::LBracket, Token::RBracket),
        ],
        |_| (),
    )));

    just(Token::Fn)
        .ignore_then(parse_ident().labelled("function name"))
        .then(args)
        .map_with(|start, e| (start, e.span()))
        .then_ignore(just(Token::Arrow))
        .then(parse_type().or_not().labelled("return type"))
        .then(body_recover)
        .map_with(|((((name, args), span), ret_type), _body), _| Item {
            kind: ItemKind::Fn(FnDef {
                _p: PhantomData,
                ident: name,
                args: args.into_iter().collect(),
                ret_type,
            }),
            span,
        })
        .labelled("function")
}

fn ignore_block<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, (), extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    recursive(|content| {
        // An item inside the block is one of two things:

        // 1. A fully-formed nested block. We do this by recursively calling ourself
        //    and wrapping the result in the `NestedBlock` AST node.
        let nested = content
            .clone() // Recursive call
            .delimited_by(just(Token::LBrace), just(Token::RBrace));

        // 2. Any single token that is NOT a brace.
        //    We wrap this in the `Token` AST node.
        let any_other_token = none_of([Token::LBrace, Token::RBrace]).repeated();
        // An item is either a nested block or any other token.
        nested.or(any_other_token)
    })
    .ignored()
}
