//! Literal lowering

use scrap_ast::lit::{Lit, LitKind};
use scrap_diagnostics::{AnnotationKind, Level, Snippet};
use scrap_ir as ir;
use scrap_shared::NodeId;
use scrap_shared::ident::Symbol;
use scrap_shared::types::{FloatTy, FloatVal, IntTy, IntVal, UintTy, UintVal};

use crate::{MResult, lowerer::ExprLowerer};

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
    /// Lower a literal to an operand by extracting the actual value from source text.
    /// `expr_id` is the NodeId of the parent Expr, used to look up types from the type table.
    pub(crate) fn lower_literal(
        &mut self,
        lit: &Lit<'db>,
        expr_id: NodeId,
    ) -> MResult<ir::Operand<'db>> {
        // Extract the actual text from source using the span
        let start = lit.span.start(self.db);
        let end = lit.span.end(self.db);
        let text = &self.source[start..end];

        // Infer the type of the literal first (needed for typed constants)
        let ty = self.infer_literal_type(lit, expr_id)?;

        // Create the constant based on literal kind
        let constant = match lit.kind {
            LitKind::Integer => {
                // Handle underscores in numeric literals (e.g., 1_000_000)
                let clean = text.replace('_', "");
                match ty {
                    ir::Ty::Uint(uint_ty) => {
                        let val = match uint_ty {
                            UintTy::Usize => UintVal::Usize(clean.parse().unwrap()),
                            UintTy::U8 => UintVal::U8(clean.parse().unwrap()),
                            UintTy::U16 => UintVal::U16(clean.parse().unwrap()),
                            UintTy::U32 => UintVal::U32(clean.parse().unwrap()),
                            UintTy::U64 => UintVal::U64(clean.parse().unwrap()),
                            UintTy::U128 => UintVal::U128(clean.parse().unwrap()),
                        };
                        ir::Constant::Uint(val)
                    }
                    ir::Ty::Int(int_ty) => {
                        let val = match int_ty {
                            IntTy::Isize => IntVal::Isize(clean.parse().unwrap()),
                            IntTy::I8 => IntVal::I8(clean.parse().unwrap()),
                            IntTy::I16 => IntVal::I16(clean.parse().unwrap()),
                            IntTy::I32 => IntVal::I32(clean.parse().unwrap()),
                            IntTy::I64 => IntVal::I64(clean.parse().unwrap()),
                            IntTy::I128 => IntVal::I128(clean.parse().unwrap()),
                        };
                        ir::Constant::Int(val)
                    }
                    _ => {
                        return Err(crate::Error::Error(
                            self.db.dcx().emit_err(
                                Level::ERROR.primary_title("Invalid literal type").element(
                                    Snippet::source(self.source).annotation(
                                        AnnotationKind::Primary
                                            .span(lit.span.to_range(self.db))
                                            .label("Expected an integer literal here"),
                                    ),
                                ),
                            ),
                        ));
                    }
                }
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
                let float_ty = match ty {
                    ir::Ty::Float(k) => k,
                    _ => {
                        return Err(crate::Error::Error(
                            self.db.dcx().emit_err(
                                Level::ERROR.primary_title("Invalid literal type").element(
                                    Snippet::source(self.source).annotation(
                                        AnnotationKind::Primary
                                            .span(lit.span.to_range(self.db))
                                            .label("Expected a float literal here"),
                                    ),
                                ),
                            ),
                        ));
                    }
                };
                let val = match float_ty {
                    FloatTy::F32 => FloatVal::F32(clean.parse().unwrap()),
                    FloatTy::F64 => FloatVal::F64(clean.parse().unwrap()),
                    _ => {
                        return Err(crate::Error::Error(
                            self.db.dcx().emit_err(
                                Level::ERROR
                                    .primary_title("Unsupported float type")
                                    .element(Snippet::source(self.source).annotation(
                                        AnnotationKind::Primary
                                            .span(lit.span.to_range(self.db))
                                            .label(format!(
                                                "{} is not yet supported",
                                                float_ty.name_str()
                                            )),
                                    )),
                            ),
                        ));
                    }
                };
                ir::Constant::Float(val)
            }
        };

        // Allocate a temporary for the literal
        let temp = self.allocate_temp(ty);

        // Emit assignment: temp = constant
        let place = ir::Place::Local(temp);
        let rvalue = ir::Rvalue::Constant(constant);
        self.emit_assign(place, rvalue);

        // Return reference to the temporary
        Ok(ir::Operand::Place(ir::Place::Local(temp)))
    }

    /// Infer the IR type from a literal.
    /// Consults the type table (for type-annotated contexts like `let x: u32 = 42`).
    /// Returns an error if the type cannot be determined.
    pub(crate) fn infer_literal_type(
        &self,
        lit: &Lit<'_>,
        expr_id: NodeId,
    ) -> MResult<ir::Ty<'db>> {
        // Check the type table — the type checker should have resolved this literal's type
        if let Some(resolved) = self.lookup_expr_type(expr_id) {
            return Ok(crate::ty_convert::resolved_to_ir(self.db, resolved));
        }

        // For bool and str, the type is unambiguous from the literal kind
        match lit.kind {
            LitKind::Bool => Ok(ir::Ty::Bool),
            LitKind::Str => Ok(ir::Ty::Str),
            LitKind::Integer | LitKind::Float => Err(crate::Error::Error(
                self.db.dcx().emit_err(
                    Level::ERROR
                        .primary_title("Could not determine literal type")
                        .element(Snippet::source(self.source).annotation(
                            AnnotationKind::Primary
                                .span(lit.span.to_range(self.db))
                                .label("Type not found in type table"),
                        )),
                ),
            )),
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
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        let operand = result.unwrap();
        assert!(matches!(operand, ir::Operand::Place(ir::Place::Local(_))));

        // Should have created one local declaration
        assert_eq!(lowerer.local_decls.len(), 1);

        // Check the local declaration type
        assert_eq!(lowerer.local_decls[0].ty(db), ir::Ty::Int(IntTy::I32));
    }

    #[scrap_macros::salsa_test]
    fn test_lower_bool_literal(db: &dyn scrap_shared::Db) {
        let expr = create_bool_lit(db, true);
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        assert_eq!(lowerer.local_decls.len(), 1);
        assert_eq!(lowerer.local_decls[0].ty(db), ir::Ty::Bool);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_string_literal(db: &dyn scrap_shared::Db) {
        let expr = create_string_lit(db, "hello");
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        assert_eq!(lowerer.local_decls.len(), 1);
        assert_eq!(lowerer.local_decls[0].ty(db), ir::Ty::Str);
    }
}
