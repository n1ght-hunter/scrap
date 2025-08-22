use chumsky::span::SimpleSpan;

pub mod ast;
pub mod parser;
pub mod utils;

pub type Span = SimpleSpan;


#[derive(Debug, Clone, Copy)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use ariadne::{Color, Label, Report, ReportKind, sources};
    use chumsky::{input::Stream, prelude::*};
    use scrap_lexer::{Logos, Token};

    use crate::parser::file_parser;

    const TEST_AST: &str = r#"
    fn foo(a: f64, b: f64) -> f64 {
        let c = if a > 10.0 {
            a + b
        } else {
            50.0
        };
        c + 2.0
    }

    fn bar() -> String {
        "Hello, \\world!"
    }

    enum MyEnum {
        Variant1,
        Variant2(MyStruct),
    }

    struct MyStruct {
        field1: i32,
        field2: String,
    }
    "#;


    


    #[test]
    fn test_ast() -> anyhow::Result<()> {
        let filename = "test.scrap";
        let src = TEST_AST;
        let (token_iter, lex_errs) = scrap_lexer::Token::lexer(src).spanned().fold(
            (Vec::new(), Vec::new()),
            |(mut tokens, mut token_errors), (new_tok, new_span)| {
                let span = SimpleSpan::from(new_span);
                match new_tok {
                    // Turn the `Range<usize>` spans logos gives us into chumsky's `SimpleSpan` via `Into`, because it's easier
                    // to work with
                    Ok(new_tok) => tokens.push((new_tok, span)),
                    Err(e) => token_errors.push(Rich::<Token, _>::custom(span, e.to_string())),
                }

                (tokens, token_errors)
            },
        );

        println!("token_iter {:#?}", token_iter);

        // Turn the token iterator into a stream that chumsky can use for things like backtracking
        let token_stream = Stream::from_iter(token_iter)
            // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
            // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
            .map((0..src.len()).into(), |(t, s): (_, _)| (t, s));

        let (ast, parse_errs) = file_parser().parse(token_stream).into_output_errors();

        if let Some(sexpr) = ast {
            println!("ast {:?}", sexpr);
        }

        if parse_errs.is_empty() && lex_errs.is_empty() {
            return Ok(());
        }

        parse_errs
            .into_iter()
            .map(|e| e.map_token(|c| c.to_string()))
            .chain(
                lex_errs
                    .into_iter()
                    .map(|e| e.map_token(|tok| tok.to_string())),
            )
            .for_each(|e| {
                Report::build(ReportKind::Error, (filename, e.span().into_range()))
                    .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                    .with_message(e.to_string())
                    .with_label(
                        Label::new((filename, e.span().into_range()))
                            .with_message(e.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .with_labels(e.contexts().map(|(label, span)| {
                        Label::new((filename, span.into_range()))
                            .with_message(format!("while parsing this {label}"))
                            .with_color(Color::Yellow)
                    }))
                    .finish()
                    .print(sources([(filename, src)]))
                    .unwrap()
            });

        return Err(anyhow::anyhow!("parse error"));
    }
}
