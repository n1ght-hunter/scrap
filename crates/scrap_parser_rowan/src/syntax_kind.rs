use scrap_lexer::Token;

/// Syntax kinds for the Scrap language CST.
/// Maps directly to lexer tokens plus additional syntax nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum SyntaxKind {
    // Trivia
    WHITESPACE,
    COMMENT,
    DOC_COMMENT,

    // Keywords
    ENUM_KW,
    STRUCT_KW,
    FN_KW,
    LET_KW,
    IF_KW,
    ELSE_KW,
    RETURN_KW,
    MOD_KW,
    USE_KW,
    PUB_KW,
    EXTERN_KW,
    MATCH_KW,
    IMPL_KW,
    SPAWN_KW,

    // Literals
    STRING_LIT,
    FLOAT_LIT,
    INT_LIT,
    BOOL_LIT,
    IDENT,

    // Binary operators
    PLUS,
    MINUS,
    STAR,
    SLASH,
    PERCENT,
    AMPAMP,
    PIPEPIPE,
    CARET,
    AMP,
    PIPE,
    SHL,
    SHR,
    EQEQ,
    LT,
    LE,
    NE,
    GE,
    GT,

    // Assignment operators
    PLUS_EQ,
    MINUS_EQ,
    STAR_EQ,
    SLASH_EQ,
    PERCENT_EQ,
    AMPAMP_EQ,
    PIPEPIPE_EQ,
    CARET_EQ,
    AMP_EQ,
    PIPE_EQ,
    SHL_EQ,
    SHR_EQ,

    // Other operators
    ARROW,
    FAT_ARROW,
    EQ,
    BANG,
    DOT,

    // Delimiters
    L_PAREN,
    R_PAREN,
    L_BRACE,
    R_BRACE,
    L_BRACKET,
    R_BRACKET,
    COMMA,
    COLON,
    COLON2,
    SEMICOLON,

    // Special tokens
    DUMMY,
    EOF,
    ERROR,

    // Composite syntax nodes
    SOURCE_FILE,

    // Items
    FUNCTION,
    STRUCT_DEF,
    ENUM_DEF,
    MODULE,
    USE_TREE,
    VISIBILITY,

    // Function parts
    PARAM_LIST,
    PARAM,
    RET_TYPE,

    // Struct parts
    FIELD_LIST,
    FIELD,

    // Enum parts
    VARIANT_LIST,
    VARIANT,

    // Types
    PATH_TYPE,
    PATH,
    PATH_SEGMENT,

    // Expressions
    LITERAL_EXPR,
    PATH_EXPR,
    PAREN_EXPR,
    ARRAY_EXPR,
    CALL_EXPR,
    BINARY_EXPR,
    ASSIGN_EXPR,
    ASSIGN_OP_EXPR,
    IF_EXPR,
    BLOCK_EXPR,
    RETURN_EXPR,

    // Statements
    LET_STMT,
    EXPR_STMT,

    // Patterns
    IDENT_PAT,

    // Other
    ARG_LIST,
    BLOCK,
    STMT_LIST,
    NAME,
    TYPE_ANNOTATION,
}

impl From<Token> for SyntaxKind {
    fn from(token: Token) -> Self {
        match token {
            // Trivia
            Token::Whitespace => SyntaxKind::WHITESPACE,
            Token::Comment => SyntaxKind::COMMENT,
            Token::DocComment => SyntaxKind::DOC_COMMENT,

            // Keywords
            Token::Enum => SyntaxKind::ENUM_KW,
            Token::Struct => SyntaxKind::STRUCT_KW,
            Token::Fn => SyntaxKind::FN_KW,
            Token::Let => SyntaxKind::LET_KW,
            Token::If => SyntaxKind::IF_KW,
            Token::Else => SyntaxKind::ELSE_KW,
            Token::Return => SyntaxKind::RETURN_KW,
            Token::Mod => SyntaxKind::MOD_KW,
            Token::Use => SyntaxKind::USE_KW,
            Token::Pub => SyntaxKind::PUB_KW,
            Token::Extern => SyntaxKind::EXTERN_KW,
            Token::Match => SyntaxKind::MATCH_KW,
            Token::Impl => SyntaxKind::IMPL_KW,
            Token::Spawn => SyntaxKind::SPAWN_KW,

            // Literals
            Token::Str => SyntaxKind::STRING_LIT,
            Token::Float => SyntaxKind::FLOAT_LIT,
            Token::Int => SyntaxKind::INT_LIT,
            Token::Bool => SyntaxKind::BOOL_LIT,
            Token::Ident => SyntaxKind::IDENT,

            // Binary operators
            Token::Add => SyntaxKind::PLUS,
            Token::Sub => SyntaxKind::MINUS,
            Token::Mul => SyntaxKind::STAR,
            Token::Div => SyntaxKind::SLASH,
            Token::Rem => SyntaxKind::PERCENT,
            Token::And => SyntaxKind::AMPAMP,
            Token::Or => SyntaxKind::PIPEPIPE,
            Token::BitXor => SyntaxKind::CARET,
            Token::BitAnd => SyntaxKind::AMP,
            Token::BitOr => SyntaxKind::PIPE,
            Token::Shl => SyntaxKind::SHL,
            Token::Shr => SyntaxKind::SHR,
            Token::Eq => SyntaxKind::EQEQ,
            Token::Lt => SyntaxKind::LT,
            Token::Le => SyntaxKind::LE,
            Token::Ne => SyntaxKind::NE,
            Token::Ge => SyntaxKind::GE,
            Token::Gt => SyntaxKind::GT,

            // Assignment operators
            Token::AddAssign => SyntaxKind::PLUS_EQ,
            Token::SubAssign => SyntaxKind::MINUS_EQ,
            Token::MulAssign => SyntaxKind::STAR_EQ,
            Token::DivAssign => SyntaxKind::SLASH_EQ,
            Token::RemAssign => SyntaxKind::PERCENT_EQ,
            Token::AndAssign => SyntaxKind::AMPAMP_EQ,
            Token::OrAssign => SyntaxKind::PIPEPIPE_EQ,
            Token::BitXorAssign => SyntaxKind::CARET_EQ,
            Token::BitAndAssign => SyntaxKind::AMP_EQ,
            Token::BitOrAssign => SyntaxKind::PIPE_EQ,
            Token::ShlAssign => SyntaxKind::SHL_EQ,
            Token::ShrAssign => SyntaxKind::SHR_EQ,

            // Other operators
            Token::Arrow => SyntaxKind::ARROW,
            Token::FatArrow => SyntaxKind::FAT_ARROW,
            Token::Assign => SyntaxKind::EQ,
            Token::Bang => SyntaxKind::BANG,
            Token::Dot => SyntaxKind::DOT,

            // Delimiters
            Token::LParen => SyntaxKind::L_PAREN,
            Token::RParen => SyntaxKind::R_PAREN,
            Token::LBrace => SyntaxKind::L_BRACE,
            Token::RBrace => SyntaxKind::R_BRACE,
            Token::LBracket => SyntaxKind::L_BRACKET,
            Token::RBracket => SyntaxKind::R_BRACKET,
            Token::Comma => SyntaxKind::COMMA,
            Token::Colon => SyntaxKind::COLON,
            Token::DoubleColon => SyntaxKind::COLON2,
            Token::Semicolon => SyntaxKind::SEMICOLON,

            // Special
            Token::Dummy => SyntaxKind::DUMMY,
            Token::Eof => SyntaxKind::EOF,
        }
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}
