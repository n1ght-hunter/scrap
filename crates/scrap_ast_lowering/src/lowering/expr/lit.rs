//! Literal lowering

use scrap_ast::lit::{Lit, LitKind};
use scrap_ir as ir;
use scrap_shared::ident::Symbol;

use crate::{lowerer::ExprLowerer, MResult};

/// Unescape a string literal (handle \n, \t, \\, \", etc.)
fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('0') => result.push('\0'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

impl<'db> ExprLowerer<'db> {
    /// Lower a literal to an operand by extracting the actual value from source text
    pub(crate) fn lower_literal(&mut self, lit: &Lit<'db>) -> MResult<ir::Operand<'db>> {
        // Extract the actual text from source using the span
        let start = lit.span.start(self.db);
        let end = lit.span.end(self.db);
        let text = &self.source[start..end];

        // Create the constant based on literal kind
        let constant = match lit.kind {
            LitKind::Integer => {
                // Handle underscores in numeric literals (e.g., 1_000_000)
                let clean = text.replace('_', "");
                let value = clean.parse::<i64>().unwrap_or(0);
                ir::Constant::Int(value)
            }
            LitKind::Bool => {
                let value = text == "true";
                ir::Constant::Bool(value)
            }
            LitKind::Str => {
                // Remove surrounding quotes and unescape
                let inner = if text.len() >= 2 {
                    &text[1..text.len() - 1]
                } else {
                    ""
                };
                let unescaped = unescape_string(inner);
                let sym = Symbol::new(self.db, unescaped);
                ir::Constant::String(sym)
            }
            LitKind::Float => {
                let clean = text.replace('_', "");
                let value = clean.parse::<f64>().unwrap_or(0.0);
                ir::Constant::Float(value.to_bits())
            }
        };

        // Infer the type of the literal
        let ty = self.infer_literal_type(lit);

        // Allocate a temporary for the literal
        let temp = self.allocate_temp(ty);

        // Emit assignment: temp = constant
        let place = ir::Place::Local(temp);
        let rvalue = ir::Rvalue::Constant(constant);
        self.emit_assign(place, rvalue);

        // Return reference to the temporary
        Ok(ir::Operand::Place(ir::Place::Local(temp)))
    }

    /// Infer the IR type from a literal
    pub(crate) fn infer_literal_type(&self, lit: &Lit<'_>) -> ir::Ty<'db> {
        match lit.kind {
            LitKind::Integer => ir::Ty::Int,
            LitKind::Bool => ir::Ty::Bool,
            LitKind::Str => ir::Ty::Str,
            LitKind::Float => ir::Ty::Int, // TODO: Add Float type to IR
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[scrap_macros::salsa_test]
    fn test_lower_int_literal(db: &dyn scrap_shared::Db) {
        let expr = create_int_lit(db, 42);
        let mut lowerer = ExprLowerer::new(db, "");

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        let operand = result.unwrap();
        assert!(matches!(operand, ir::Operand::Place(ir::Place::Local(_))));

        // Should have created one local declaration
        assert_eq!(lowerer.local_decls.len(), 1);

        // Check the local declaration type
        assert_eq!(lowerer.local_decls[0].ty(db), ir::Ty::Int);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_bool_literal(db: &dyn scrap_shared::Db) {
        let expr = create_bool_lit(db, true);
        let mut lowerer = ExprLowerer::new(db, "");

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        assert_eq!(lowerer.local_decls.len(), 1);
        assert_eq!(lowerer.local_decls[0].ty(db), ir::Ty::Bool);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_string_literal(db: &dyn scrap_shared::Db) {
        let expr = create_string_lit(db, "hello");
        let mut lowerer = ExprLowerer::new(db, "");

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        assert_eq!(lowerer.local_decls.len(), 1);
        assert_eq!(lowerer.local_decls[0].ty(db), ir::Ty::Str);
    }
}
