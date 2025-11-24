use pretty::{Arena, DocAllocator, DocBuilder};
use scrap_parser_rowan::{SyntaxKind, SyntaxNode, SyntaxToken};

/// Configuration for the formatter
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FormatterConfig {
    pub indent_width: usize,
    pub line_width: usize,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        FormatterConfig {
            indent_width: 4,
            line_width: 100,
        }
    }
}

/// Format a source file from text
pub fn format_file(source: &str, config: &FormatterConfig) -> String {
    // Create a database for parsing
    let db = scrap_shared::salsa::ScrapDb::default();

    // Use a helper tracked function to create tracked structs
    format_file_impl(&db, source, config)
}

/// Helper function to format a file within a tracked function context
#[salsa::tracked]
fn format_file_impl<'db>(
    db: &'db dyn scrap_shared::Db,
    source: &'db str,
    config: &'db FormatterConfig,
) -> String {
    use scrap_shared::InputFile;

    // Create an input file from the source
    let file = InputFile::new(db, std::path::PathBuf::from("<format>"), source.to_string());

    // Lex the file
    let tokens = match scrap_lexer::lex_file(db, file) {
        Some(tokens) => tokens,
        None => return source.to_string(), // Return original on lex error
    };

    // Parse the file using the Rowan parser
    let parsed = scrap_parser_rowan::parse_file(db, file, tokens);

    // Debug
    eprintln!("=== SYNTAX TREE ===\n{:#?}\n===", parsed.syntax(db));

    // Format the syntax tree
    format_syntax_tree(&parsed.syntax(db), config)
}

/// Format a syntax tree
pub fn format_syntax_tree(node: &SyntaxNode, config: &FormatterConfig) -> String {
    let arena = Arena::new();
    let doc = format_node(&arena, node, config);
    let mut result = Vec::new();
    doc.render(config.line_width, &mut result).unwrap();
    String::from_utf8(result).unwrap()
}

/// Format a syntax node
fn format_node<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    match node.kind() {
        SyntaxKind::SOURCE_FILE => format_source_file(arena, node, config),
        SyntaxKind::FUNCTION => format_function(arena, node, config),
        SyntaxKind::STRUCT_DEF => format_struct_def(arena, node, config),
        SyntaxKind::ENUM_DEF => format_enum_def(arena, node, config),
        SyntaxKind::MODULE => format_module(arena, node, config),
        SyntaxKind::USE_TREE => format_use_tree(arena, node, config),
        SyntaxKind::BLOCK_EXPR => format_block_expr(arena, node, config),
        SyntaxKind::IF_EXPR => format_if_expr(arena, node, config),
        SyntaxKind::BINARY_EXPR => format_binary_expr(arena, node, config),
        SyntaxKind::CALL_EXPR => format_call_expr(arena, node, config),
        SyntaxKind::ARRAY_EXPR => format_array_expr(arena, node, config),
        SyntaxKind::RETURN_EXPR => format_return_expr(arena, node, config),
        SyntaxKind::LITERAL_EXPR => format_literal_expr(arena, node, config),
        SyntaxKind::PATH_EXPR => format_path_expr(arena, node, config),
        SyntaxKind::PAREN_EXPR => format_paren_expr(arena, node, config),
        SyntaxKind::LET_STMT => format_let_stmt(arena, node, config),
        SyntaxKind::EXPR_STMT => format_expr_stmt(arena, node, config),
        _ => {
            // For unknown nodes, concatenate children
            let docs: Vec<_> = node
                .children_with_tokens()
                .filter_map(|child| match child {
                    rowan::NodeOrToken::Node(n) => Some(format_node(arena, &n, config)),
                    rowan::NodeOrToken::Token(t) => {
                        if !is_trivia(&t) {
                            Some(arena.text(t.text().to_string()))
                        } else {
                            None
                        }
                    }
                })
                .collect();
            arena.concat(docs)
        }
    }
}

/// Format source file
fn format_source_file<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let items: Vec<_> = node
        .children()
        .map(|child| format_node(arena, &child, config))
        .collect();

    arena.intersperse(items, arena.hardline().append(arena.hardline()))
}

/// Format function definition
fn format_function<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = Vec::new();

    // Visibility
    if let Some(_vis) = node.children().find(|n| n.kind() == SyntaxKind::VISIBILITY) {
        docs.push(arena.text("pub "));
    }

    docs.push(arena.text("fn "));

    // Function name
    if let Some(name) = node.children().find(|n| n.kind() == SyntaxKind::NAME) {
        docs.push(format_node(arena, &name, config));
    }

    // Parameters
    if let Some(params) = node.children().find(|n| n.kind() == SyntaxKind::PARAM_LIST) {
        docs.push(format_param_list(arena, &params, config));
    }

    // Return type
    if let Some(ret_type) = node.children().find(|n| n.kind() == SyntaxKind::RET_TYPE) {
        docs.push(arena.text(" -> "));
        if let Some(ty) = ret_type.children().next() {
            docs.push(format_node(arena, &ty, config));
        }
    }

    // Body
    if let Some(body) = node.children().find(|n| n.kind() == SyntaxKind::BLOCK_EXPR) {
        docs.push(arena.space());
        docs.push(format_block_expr(arena, &body, config));
    }

    arena.concat(docs)
}

/// Format parameter list
fn format_param_list<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let params: Vec<_> = node
        .children()
        .filter(|n| n.kind() == SyntaxKind::PARAM)
        .map(|param| format_param(arena, &param, config))
        .collect();

    arena
        .text("(")
        .append(arena.intersperse(params, arena.text(", ")))
        .append(arena.text(")"))
}

/// Format a single parameter
fn format_param<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = Vec::new();

    // Parameter name
    if let Some(name) = node.children().find(|n| n.kind() == SyntaxKind::NAME) {
        docs.push(format_name(arena, &name));
    }

    // Type annotation
    if let Some(type_ann) = node
        .children()
        .find(|n| n.kind() == SyntaxKind::TYPE_ANNOTATION)
    {
        docs.push(arena.text(": "));
        if let Some(ty) = type_ann.children().next() {
            docs.push(format_node(arena, &ty, config));
        }
    }

    arena.concat(docs)
}

/// Format struct definition
fn format_struct_def<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = Vec::new();

    // Visibility
    if let Some(_vis) = node.children().find(|n| n.kind() == SyntaxKind::VISIBILITY) {
        docs.push(arena.text("pub "));
    }

    docs.push(arena.text("struct "));

    // Struct name
    if let Some(name) = node.children().find(|n| n.kind() == SyntaxKind::NAME) {
        docs.push(format_name(arena, &name));
    }

    // Fields
    if let Some(fields) = node.children().find(|n| n.kind() == SyntaxKind::FIELD_LIST) {
        docs.push(arena.space());
        docs.push(format_field_list(arena, &fields, config));
    }

    arena.concat(docs)
}

/// Format field list
fn format_field_list<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let fields: Vec<_> = node
        .children()
        .filter(|n| n.kind() == SyntaxKind::FIELD)
        .map(|field| format_field(arena, &field, config))
        .collect();

    if fields.is_empty() {
        arena.text("{}")
    } else {
        arena
            .text("{")
            .append(arena.hardline())
            .append(
                arena
                    .intersperse(fields, arena.text(",").append(arena.hardline()))
                    .indent(config.indent_width),
            )
            .append(arena.text(","))
            .append(arena.hardline())
            .append(arena.text("}"))
    }
}

/// Format a field
fn format_field<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = Vec::new();

    // Field name
    if let Some(name) = node.children().find(|n| n.kind() == SyntaxKind::NAME) {
        docs.push(format_name(arena, &name));
    }

    // Type annotation
    if let Some(type_ann) = node
        .children()
        .find(|n| n.kind() == SyntaxKind::TYPE_ANNOTATION)
    {
        docs.push(arena.text(": "));
        if let Some(ty) = type_ann.children().next() {
            docs.push(format_node(arena, &ty, config));
        }
    }

    arena.concat(docs)
}

/// Format enum definition
fn format_enum_def<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = Vec::new();

    // Visibility
    if let Some(_vis) = node.children().find(|n| n.kind() == SyntaxKind::VISIBILITY) {
        docs.push(arena.text("pub "));
    }

    docs.push(arena.text("enum "));

    // Enum name
    if let Some(name) = node.children().find(|n| n.kind() == SyntaxKind::NAME) {
        docs.push(format_name(arena, &name));
    }

    // Variants
    if let Some(variants) = node
        .children()
        .find(|n| n.kind() == SyntaxKind::VARIANT_LIST)
    {
        docs.push(arena.space());
        docs.push(format_variant_list(arena, &variants, config));
    }

    arena.concat(docs)
}

/// Format variant list
fn format_variant_list<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let variants: Vec<_> = node
        .children()
        .filter(|n| n.kind() == SyntaxKind::VARIANT)
        .map(|variant| format_variant(arena, &variant, config))
        .collect();

    if variants.is_empty() {
        arena.text("{}")
    } else {
        arena
            .text("{")
            .append(arena.hardline())
            .append(
                arena
                    .intersperse(variants, arena.text(",").append(arena.hardline()))
                    .indent(config.indent_width),
            )
            .append(arena.text(","))
            .append(arena.hardline())
            .append(arena.text("}"))
    }
}

/// Format a variant
fn format_variant<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    _config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = Vec::new();

    // Variant name
    if let Some(name) = node.children().find(|n| n.kind() == SyntaxKind::NAME) {
        docs.push(format_name(arena, &name));
    }

    // Optional tuple data
    // This is simplified; real implementation would handle tuple variants properly

    arena.concat(docs)
}

/// Format module
fn format_module<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    _config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = Vec::new();

    // Visibility
    if let Some(_vis) = node.children().find(|n| n.kind() == SyntaxKind::VISIBILITY) {
        docs.push(arena.text("pub "));
    }

    docs.push(arena.text("mod "));

    // Module name
    if let Some(name) = node.children().find(|n| n.kind() == SyntaxKind::NAME) {
        docs.push(format_name(arena, &name));
    }

    // Check if it's a semicolon or block module
    let has_semicolon = node.children_with_tokens().any(
        |t| matches!(t, rowan::NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::SEMICOLON),
    );

    if has_semicolon {
        docs.push(arena.text(";"));
    }

    arena.concat(docs)
}

/// Format use tree
fn format_use_tree<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    _config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = Vec::new();

    docs.push(arena.text("use "));

    // Path
    if let Some(path) = node.children().find(|n| n.kind() == SyntaxKind::PATH) {
        docs.push(format_path(arena, &path));
    }

    docs.push(arena.text(";"));

    arena.concat(docs)
}

/// Format block expression
fn format_block_expr<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    if let Some(block) = node.children().find(|n| n.kind() == SyntaxKind::BLOCK) {
        if let Some(stmt_list) = block.children().find(|n| n.kind() == SyntaxKind::STMT_LIST) {
            let stmts: Vec<_> = stmt_list
                .children()
                .map(|stmt| format_node(arena, &stmt, config))
                .collect();

            if stmts.is_empty() {
                return arena.text("{}");
            }

            return arena
                .text("{")
                .append(arena.hardline())
                .append(
                    arena
                        .intersperse(stmts, arena.hardline())
                        .indent(config.indent_width),
                )
                .append(arena.hardline())
                .append(arena.text("}"));
        }
    }

    arena.text("{}")
}

/// Format if expression
fn format_if_expr<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = Vec::new();
    docs.push(arena.text("if "));

    let mut children = node.children();

    // Condition
    if let Some(cond) = children.next() {
        docs.push(format_node(arena, &cond, config));
    }

    // Then block
    if let Some(then_block) = children.next() {
        docs.push(arena.space());
        docs.push(format_node(arena, &then_block, config));
    }

    // Else block
    if let Some(else_part) = children.next() {
        docs.push(arena.text(" else "));
        docs.push(format_node(arena, &else_part, config));
    }

    arena.concat(docs)
}

/// Format binary expression
fn format_binary_expr<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut parts = Vec::new();

    for child in node.children_with_tokens() {
        match child {
            rowan::NodeOrToken::Node(n) => {
                parts.push(format_node(arena, &n, config));
            }
            rowan::NodeOrToken::Token(t) => {
                if !is_trivia(&t) {
                    parts.push(arena.space());
                    parts.push(arena.text(t.text().to_string()));
                    parts.push(arena.space());
                }
            }
        }
    }

    arena.concat(parts)
}

/// Format call expression
fn format_call_expr<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = Vec::new();

    // Function being called (should be first child)
    if let Some(func) = node.children().next() {
        docs.push(format_node(arena, &func, config));
    }

    // Arguments
    if let Some(args) = node.children().find(|n| n.kind() == SyntaxKind::ARG_LIST) {
        docs.push(format_arg_list(arena, &args, config));
    }

    arena.concat(docs)
}

/// Format argument list
fn format_arg_list<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let args: Vec<_> = node
        .children()
        .map(|arg| format_node(arena, &arg, config))
        .collect();

    arena
        .text("(")
        .append(arena.intersperse(args, arena.text(", ")))
        .append(arena.text(")"))
}

/// Format array expression
fn format_array_expr<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let elements: Vec<_> = node
        .children()
        .map(|elem| format_node(arena, &elem, config))
        .collect();

    arena
        .text("[")
        .append(arena.intersperse(elements, arena.text(", ")))
        .append(arena.text("]"))
}

/// Format return expression
fn format_return_expr<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = vec![arena.text("return")];

    if let Some(value) = node.children().next() {
        docs.push(arena.space());
        docs.push(format_node(arena, &value, config));
    }

    arena.concat(docs)
}

/// Format literal expression
fn format_literal_expr<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    _config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    if let Some(token) = node.first_token() {
        arena.text(token.text().to_string())
    } else {
        arena.nil()
    }
}

/// Format path expression
fn format_path_expr<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    _config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    if let Some(path) = node.children().find(|n| n.kind() == SyntaxKind::PATH) {
        format_path(arena, &path)
    } else {
        arena.nil()
    }
}

/// Format paren expression
fn format_paren_expr<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = vec![arena.text("(")];

    if let Some(inner) = node.children().next() {
        docs.push(format_node(arena, &inner, config));
    }

    docs.push(arena.text(")"));

    arena.concat(docs)
}

/// Format let statement
fn format_let_stmt<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = vec![arena.text("let ")];

    // Pattern
    if let Some(pat) = node.children().find(|n| n.kind() == SyntaxKind::IDENT_PAT) {
        if let Some(token) = pat.first_token() {
            docs.push(arena.text(token.text().to_string()));
        }
    }

    // Type annotation
    if let Some(type_ann) = node
        .children()
        .find(|n| n.kind() == SyntaxKind::TYPE_ANNOTATION)
    {
        docs.push(arena.text(": "));
        if let Some(ty) = type_ann.children().next() {
            docs.push(format_node(arena, &ty, config));
        }
    }

    // Initializer
    let has_eq = node
        .children_with_tokens()
        .any(|t| matches!(t, rowan::NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::EQ));

    if has_eq {
        docs.push(arena.text(" = "));
        // Find the expression after the equals
        let found_eq = false;
        for child in node.children() {
            if found_eq {
                docs.push(format_node(arena, &child, config));
                break;
            }
        }
    }

    docs.push(arena.text(";"));

    arena.concat(docs)
}

/// Format expression statement
fn format_expr_stmt<'a>(
    arena: &'a Arena<'a>,
    node: &SyntaxNode,
    config: &FormatterConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let mut docs = Vec::new();

    if let Some(expr) = node.children().next() {
        docs.push(format_node(arena, &expr, config));
    }

    // Check if it has a semicolon
    let has_semicolon = node.children_with_tokens().any(
        |t| matches!(t, rowan::NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::SEMICOLON),
    );

    if has_semicolon {
        docs.push(arena.text(";"));
    }

    arena.concat(docs)
}

/// Format a name node
fn format_name<'a>(arena: &'a Arena<'a>, node: &SyntaxNode) -> DocBuilder<'a, Arena<'a>> {
    if let Some(token) = node.first_token() {
        arena.text(token.text().to_string())
    } else {
        arena.nil()
    }
}

/// Format a path
fn format_path<'a>(arena: &'a Arena<'a>, node: &SyntaxNode) -> DocBuilder<'a, Arena<'a>> {
    let segments: Vec<_> = node
        .children()
        .filter(|n| n.kind() == SyntaxKind::PATH_SEGMENT)
        .filter_map(|seg| seg.first_token().map(|t| arena.text(t.text().to_string())))
        .collect();

    arena.intersperse(segments, arena.text("::"))
}

/// Check if a token is trivia
fn is_trivia(token: &SyntaxToken) -> bool {
    matches!(
        token.kind(),
        SyntaxKind::WHITESPACE | SyntaxKind::COMMENT | SyntaxKind::DOC_COMMENT
    )
}
