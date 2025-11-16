//! Pratt parser (precedence climbing) for binary operators.

use scrap_ast::{
    expr::{Expr, ExprKind},
    operators::{AssocOp, BinOpKind},
};
use scrap_span::Span;

impl<'a, 'db> crate::parser::Parser<'a, 'db> {
    /// Parse an expression with minimum precedence using Pratt parser algorithm.
    ///
    /// This implements operator precedence climbing. Only operators with precedence
    /// greater than or equal to `min_prec` will be parsed at this level.
    pub fn parse_expr_with_min_precedence(
        &mut self,
        min_prec: u8,
    ) -> crate::PResult<'a, Expr<'db>> {
        let mut lhs = self.parse_atom()?;

        loop {
            let op = match AssocOp::from_token(&self.token.node) {
                Some(op) => op,
                None => break,
            };

            let (left_prec, right_prec) = Self::precedence(&op);

            if left_prec < min_prec {
                break;
            }

            let op_span = self.token.span;
            self.bump();

            let rhs = self.parse_expr_with_min_precedence(right_prec)?;

            let start_pos = lhs.span.start(self.db);
            let end_pos = rhs.span.end(self.db);
            let span = Span::new(self.db, start_pos, end_pos);

            lhs = match op {
                AssocOp::Binary(bin_op) => Expr {
                    id: self.state.new_node_id(),
                    kind: ExprKind::Binary(
                        scrap_span::Spanned::new(bin_op, op_span),
                        Box::new(lhs),
                        Box::new(rhs),
                    ),
                    span,
                },
                AssocOp::Assign => Expr {
                    id: self.state.new_node_id(),
                    kind: ExprKind::Assign(Box::new(lhs), Box::new(rhs), op_span),
                    span,
                },
                AssocOp::AssignOp(assign_op) => Expr {
                    id: self.state.new_node_id(),
                    kind: ExprKind::AssignOp(
                        scrap_span::Spanned::new(assign_op, op_span),
                        Box::new(lhs),
                        Box::new(rhs),
                    ),
                    span,
                },
            };
        }

        Ok(lhs)
    }

    /// Get operator precedence and associativity.
    ///
    /// Returns `(left_precedence, right_precedence)` tuple.
    /// - Left-associative operators: `right = left + 1`
    /// - Right-associative operators: `right = left`
    ///
    /// Precedence levels range from 2 (assignment, lowest) to 20 (mul/div/rem, highest).
    fn precedence(op: &AssocOp) -> (u8, u8) {
        match op {
            AssocOp::Assign | AssocOp::AssignOp(_) => (2, 2), // Right-associative
            AssocOp::Binary(BinOpKind::Or) => (3, 4),
            AssocOp::Binary(BinOpKind::And) => (5, 6),
            AssocOp::Binary(
                BinOpKind::Eq
                | BinOpKind::Ne
                | BinOpKind::Lt
                | BinOpKind::Le
                | BinOpKind::Gt
                | BinOpKind::Ge,
            ) => (7, 8),
            AssocOp::Binary(BinOpKind::BitOr) => (9, 10),
            AssocOp::Binary(BinOpKind::BitXor) => (11, 12),
            AssocOp::Binary(BinOpKind::BitAnd) => (13, 14),
            AssocOp::Binary(BinOpKind::Shl | BinOpKind::Shr) => (15, 16),
            AssocOp::Binary(BinOpKind::Add | BinOpKind::Sub) => (17, 18),
            AssocOp::Binary(BinOpKind::Mul | BinOpKind::Div | BinOpKind::Rem) => (19, 20),
        }
    }
}
