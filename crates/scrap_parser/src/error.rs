//! Enhanced error handling and reporting for the Scrap parser
//!
//! This module leverages Chumsky's labeling system and Ariadne's rich reporting
//! to provide high-quality error messages with proper spans and context.

use chumsky::error::Rich;
use scrap_lexer::Token;
use crate::Span;
use ariadne::{Color, Label, Report, ReportKind, Source};

/// Type alias for error analysis results to reduce complexity
type ErrorAnalysisResult = (String, Vec<(String, std::ops::Range<usize>, Color)>, Option<String>);

/// Creates detailed error reports using Ariadne's full capabilities
/// and leveraging Chumsky's context information
pub fn report_parse_errors(
    parse_errors: Vec<Rich<'_, Token<'_>, Span>>,
    lex_errors: Vec<Rich<'_, Token<'_>, Span>>,
    filename: &str,
    source: &str,
) {
    // Process parser errors with enhanced context
    for error in parse_errors.iter() {
        create_detailed_report(error, filename, source).print((filename, Source::from(source))).unwrap();
    }

    // Process lexer errors 
    for error in lex_errors.iter() {
        create_lexer_report(error, filename, source).print((filename, Source::from(source))).unwrap();
    }
}

fn create_detailed_report<'a>(
    error: &Rich<'_, Token<'_>, Span>,
    filename: &'a str,
    source: &'a str,
) -> Report<'a, (&'a str, std::ops::Range<usize>)> {
    let span = error.span();
    let range = span.into_range();
    
    let mut report = Report::build(ReportKind::Error, (filename, range.clone()))
        .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte));

    // Determine error type and create appropriate message
    match error.reason() {
        chumsky::error::RichReason::ExpectedFound { expected, found } => {
            let (main_msg, labels, help) = analyze_expected_found(expected, found, *span, filename, source);
            
            report = report.with_message(main_msg);
            
            // Add primary error label
            report = report.with_label(
                Label::new((filename, range))
                    .with_message(format_found_token(found))
                    .with_color(Color::Red)
            );
            
            // Add context labels from Chumsky
            for (label_text, label_span) in error.contexts() {
                report = report.with_label(
                    Label::new((filename, label_span.into_range()))
                        .with_message(format!("while parsing {label_text}"))
                        .with_color(Color::Yellow)
                );
            }
            
            // Add additional contextual labels
            for (label_msg, label_range, color) in labels {
                report = report.with_label(
                    Label::new((filename, label_range))
                        .with_message(label_msg)
                        .with_color(color)
                );
            }
            
            // Add help text if available
            if let Some(help_text) = help {
                report = report.with_help(help_text);
            }
        }
        chumsky::error::RichReason::Custom(msg) => {
            report = report
                .with_message(msg)
                .with_label(
                    Label::new((filename, range))
                        .with_message("custom error occurred here")
                        .with_color(Color::Red)
                );
        }
    }

    report.finish()
}

fn create_lexer_report<'a>(
    error: &Rich<'_, Token<'_>, Span>,
    filename: &'a str,
    _source: &'a str,
) -> Report<'a, (&'a str, std::ops::Range<usize>)> {
    let range = error.span().into_range();
    
    Report::build(ReportKind::Error, (filename, range.clone()))
        .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
        .with_message("Lexical error")
        .with_label(
            Label::new((filename, range))
                .with_message(error.to_string())
                .with_color(Color::Red)
        )
        .finish()
}

/// Analyzes expected/found errors to provide detailed context and suggestions
fn analyze_expected_found<'a>(
    expected: &[chumsky::error::RichPattern<'a, Token<'a>>],
    found: &Option<chumsky::util::Maybe<Token<'a>, &'a Token<'a>>>,
    error_span: Span,
    _filename: &str,
    source: &str,
) -> ErrorAnalysisResult {
    
    // Check for semicolon-related errors
    if let Some(semicolon_error) = detect_semicolon_error(expected, found, error_span, source) {
        return semicolon_error;
    }
    
    // Check for bracket/brace mismatches
    if let Some(bracket_error) = detect_bracket_mismatch(expected, found, error_span, source) {
        return bracket_error;
    }
    
    // Check for keyword context errors
    if let Some(keyword_error) = detect_keyword_context_error(expected, found, error_span, source) {
        return keyword_error;
    }
    
    // Default case: format standard expected/found message
    let found_str = format_found_token(found);
    let expected_str = format_expected_patterns(expected);
    
    let message = if found.is_some() {
        format!("Expected {expected_str}, found {found_str}")
    } else {
        format!("Expected {expected_str}, found end of input")
    };
    
    (message, Vec::new(), None)
}

/// Detects semicolon-related errors and provides targeted feedback
fn detect_semicolon_error<'a>(
    expected: &[chumsky::error::RichPattern<'a, Token<'a>>],
    found: &Option<chumsky::util::Maybe<Token<'a>, &'a Token<'a>>>,
    error_span: Span,
    source: &str,
) -> Option<ErrorAnalysisResult> {
    
    let expects_semicolon = expected.iter().any(|pattern| {
        matches!(pattern, chumsky::error::RichPattern::Token(maybe_tok) if {
            matches!(maybe_tok, chumsky::util::Maybe::Val(Token::Semicolon) | chumsky::util::Maybe::Ref(Token::Semicolon))
        })
    });
    
    if !expects_semicolon {
        return None;
    }
    
    let found_token = found.as_ref().map(|maybe_tok| {
        match maybe_tok {
            chumsky::util::Maybe::Val(tok) => tok,
            chumsky::util::Maybe::Ref(tok) => *tok,
        }
    });
    
    let mut labels = Vec::new();
    
    let message = match found_token {
        Some(Token::Let) => {
            // Try to find the end of the previous statement
            if let Some(prev_stmt_end) = find_previous_statement_end(error_span, source) {
                labels.push((
                    "add `;` here".to_string(),
                    prev_stmt_end..prev_stmt_end,
                    Color::Green
                ));
            }
            
            "Missing semicolon before this statement".to_string()
        }
        Some(Token::RBrace) => {
            "Missing semicolon before end of block".to_string()
        }
        Some(other_token) => {
            format!("Expected `;` but found '{}'", token_to_string(other_token))
        }
        None => {
            "Expected `;` but reached end of input".to_string()
        }
    };
    
    let help = Some("Add a semicolon `;` to complete the statement".to_string());
    
    Some((message, labels, help))
}

/// Detects bracket/brace mismatch errors
fn detect_bracket_mismatch<'a>(
    expected: &[chumsky::error::RichPattern<'a, Token<'a>>],
    found: &Option<chumsky::util::Maybe<Token<'a>, &'a Token<'a>>>,
    _error_span: Span,
    _source: &str,
) -> Option<ErrorAnalysisResult> {
    
    let expects_closing = expected.iter().any(|pattern| {
        matches!(pattern, chumsky::error::RichPattern::Token(maybe_tok) if {
            match maybe_tok {
                chumsky::util::Maybe::Val(tok) => {
                    matches!(tok, Token::RParen | Token::RBrace | Token::RBracket)
                }
                chumsky::util::Maybe::Ref(tok) => {
                    matches!(*tok, Token::RParen | Token::RBrace | Token::RBracket)
                }
            }
        })
    });
    
    if !expects_closing {
        return None;
    }
    
    let found_token = found.as_ref().map(|maybe_tok| {
        match maybe_tok {
            chumsky::util::Maybe::Val(tok) => tok,
            chumsky::util::Maybe::Ref(tok) => *tok,
        }
    });
    
    let message = match found_token {
        Some(token) => {
            format!("Mismatched brackets: expected closing bracket, found '{}'", token_to_string(token))
        }
        None => "Mismatched brackets: expected closing bracket, found end of input".to_string()
    };
    
    let help = Some("Check that all opening brackets have matching closing brackets".to_string());
    
    Some((message, Vec::new(), help))
}

/// Detects keyword context errors (like unexpected keywords)
fn detect_keyword_context_error<'a>(
    _expected: &[chumsky::error::RichPattern<'a, Token<'a>>],
    found: &Option<chumsky::util::Maybe<Token<'a>, &'a Token<'a>>>,
    _error_span: Span,
    _source: &str,
) -> Option<ErrorAnalysisResult> {
    
    let found_token = found.as_ref().map(|maybe_tok| {
        match maybe_tok {
            chumsky::util::Maybe::Val(tok) => tok,
            chumsky::util::Maybe::Ref(tok) => *tok,
        }
    });
    
    match found_token {
        Some(Token::Let) => {
            Some((
                "Unexpected 'let' keyword".to_string(),
                Vec::new(),
                Some("The previous statement might be missing a semicolon".to_string())
            ))
        }
        Some(Token::Fn) => {
            Some((
                "Unexpected 'fn' keyword".to_string(),
                Vec::new(),
                Some("Function definitions must be at the top level".to_string())
            ))
        }
        _ => None
    }
}

/// Attempts to find the end position of the previous statement for better error placement
fn find_previous_statement_end(error_span: Span, source: &str) -> Option<usize> {
    let error_start = error_span.start;
    if error_start == 0 {
        return None;
    }
    
    // Look backwards for statement-ending characters
    let source_bytes = source.as_bytes();
    for i in (0..error_start).rev() {
        match source_bytes[i] {
            b'}' | b';' => return Some(i + 1),
            b' ' | b'\t' | b'\n' | b'\r' => continue,
            _ => break,
        }
    }
    
    None
}

fn format_found_token<'a>(found: &Option<chumsky::util::Maybe<Token<'a>, &'a Token<'a>>>) -> String {
    match found {
        Some(maybe_token) => {
            let token = match maybe_token {
                chumsky::util::Maybe::Val(tok) => tok,
                chumsky::util::Maybe::Ref(tok) => *tok,
            };
            format!("'{}'", token_to_string(token))
        }
        None => "end of input".to_string(),
    }
}

fn format_expected_patterns<'a>(expected: &[chumsky::error::RichPattern<'a, Token<'a>>]) -> String {
    let tokens: Vec<_> = expected
        .iter()
        .filter_map(|pattern| {
            match pattern {
                chumsky::error::RichPattern::Token(maybe_token) => {
                    let token = match maybe_token {
                        chumsky::util::Maybe::Val(tok) => tok,
                        chumsky::util::Maybe::Ref(tok) => *tok,
                    };
                    Some(token_to_string(token))
                }
                _ => None,
            }
        })
        .take(3) // Limit to avoid overwhelming output
        .collect();
    
    match tokens.len() {
        0 => "something else".to_string(),
        1 => format!("'{}'", tokens[0]),
        2 => format!("'{}' or '{}'", tokens[0], tokens[1]),
        _ => {
            let last = tokens.last().unwrap();
            let others = &tokens[..tokens.len()-1];
            format!("'{}', or '{}'", others.join("', '"), last)
        }
    }
}

fn token_to_string(token: &Token<'_>) -> &'static str {
    match token {
        Token::Let => "let",
        Token::Fn => "fn", 
        Token::If => "if",
        Token::Else => "else",
        Token::Return => "return",
        Token::Bool(true) => "true",
        Token::Bool(false) => "false",
        Token::Semicolon => ";",
        Token::Comma => ",",
        Token::LParen => "(",
        Token::RParen => ")",
        Token::LBrace => "{",
        Token::RBrace => "}",
        Token::LBracket => "[",
        Token::RBracket => "]",
        Token::Assign => "=",
        Token::Plus => "+",
        Token::Minus => "-",
        Token::Star => "*",
        Token::Slash => "/",
        Token::Percent => "%",
        Token::Eq => "==",
        Token::Ne => "!=",
        Token::Lt => "<",
        Token::Gt => ">", 
        Token::Le => "<=",
        Token::Ge => ">=",
        Token::And => "&&",
        Token::Or => "||",
        Token::Colon => ":",
        Token::Arrow => "->",
        Token::Ident(_) => "identifier",
        Token::Str(_) => "string literal",
        Token::Int(_) => "integer literal", 
        Token::Float(_) => "float literal",
        Token::Struct => "struct",
        Token::Enum => "enum",
        _ => "unknown token",
    }
}
