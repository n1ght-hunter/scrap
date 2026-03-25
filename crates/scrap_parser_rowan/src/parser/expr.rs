mod atom;

use scrap_lexer::Token;

use crate::{parser::Parser, syntax_kind::SyntaxKind};

impl<'db> Parser<'db> {
    /// Parse an expression (simplified)
    pub(super) fn parse_expr(&mut self) {
        self.parse_expr_bp(0);
    }

    /// Parse expression with binding power (Pratt parsing)
    pub(super) fn parse_expr_bp(&mut self, min_bp: u8) {
        // Create checkpoint BEFORE parsing the left operand
        // This allows us to retroactively wrap it in a BINARY_EXPR if we find an operator
        let checkpoint = self.checkpoint();

        // Parse prefix/atom expression
        // This creates and completes its node (e.g., PATH_EXPR, LITERAL_EXPR, etc.)
        self.parse_atom_expr();

        // Parse infix operators using the Pratt algorithm
        loop {
            // Check if current token is an infix operator
            let op = match self.current_kind() {
                Some(kind) => kind,
                None => break,
            };

            if let Some((l_bp, r_bp)) = infix_binding_power(op) {
                // Check if this operator's precedence is high enough
                if l_bp < min_bp {
                    break;
                }

                // Use checkpoint to retroactively wrap the left operand
                // This creates: BINARY_EXPR { left_operand, operator, right_operand }
                self.start_node_at(checkpoint, SyntaxKind::BINARY_EXPR);

                self.bump(); // consume operator token

                // Parse right operand with appropriate precedence
                self.parse_expr_bp(r_bp);

                self.finish_node(); // finish BINARY_EXPR

            // Important: For chained operators (a + b + c), the checkpoint
            // already captures the previous BINARY_EXPR, so the next iteration
            // will wrap it again, creating proper left-associativity
            } else {
                // Not an infix operator, stop parsing
                break;
            }
        }
    }

    /// Parse an if expression
    pub(super) fn parse_if_expr(&mut self) {
        self.start_node(SyntaxKind::IF_EXPR);

        self.expect(Token::If);
        self.parse_expr(); // condition
        self.parse_block_expr(); // then block

        if self.at(Token::Else) {
            self.bump(); // else
            if self.at(Token::If) {
                self.parse_if_expr(); // else if
            } else {
                self.parse_block_expr(); // else block
            }
        }

        self.finish_node();
    }

    /// Parse argument list for function calls
    pub(super) fn parse_arg_list(&mut self) {
        self.start_node(SyntaxKind::ARG_LIST);

        self.expect(Token::LParen);

        while !self.at(Token::RParen) && !self.at_eof() {
            self.parse_expr();

            if self.at(Token::Comma) {
                self.bump();
            } else if !self.at(Token::RParen) {
                break;
            }
        }

        self.expect(Token::RParen);
        self.finish_node();
    }
}

/// Returns (left_binding_power, right_binding_power) for infix operators
fn infix_binding_power(op: Token) -> Option<(u8, u8)> {
    Some(match op {
        Token::Or => (1, 2),
        Token::And => (3, 4),
        Token::Eq | Token::Ne | Token::Lt | Token::Le | Token::Gt | Token::Ge => (5, 6),
        Token::BitOr => (7, 8),
        Token::BitXor => (9, 10),
        Token::BitAnd => (11, 12),
        Token::Shl | Token::Shr => (13, 14),
        Token::Add | Token::Sub => (15, 16),
        Token::Mul | Token::Div | Token::Rem => (17, 18),
        Token::Assign
        | Token::AddAssign
        | Token::SubAssign
        | Token::MulAssign
        | Token::DivAssign
        | Token::RemAssign
        | Token::AndAssign
        | Token::OrAssign
        | Token::BitXorAssign
        | Token::BitAndAssign
        | Token::BitOrAssign
        | Token::ShlAssign
        | Token::ShrAssign => (1, 2),
        _ => return None,
    })
}
