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
    /// Build an IR constant from a literal (shared by lower_literal and lower_literal_into).
    fn build_constant(
        &self,
        lit: &Lit<'db>,
        ty: &ir::Ty<'db>,
    ) -> MResult<ir::Constant<'db>> {
        let start = lit.span.start(self.db);
        let end = lit.span.end(self.db);
        let text = &self.source[start..end];

        match lit.kind {
            LitKind::Integer => {
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
                        Ok(ir::Constant::Uint(val))
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
                        Ok(ir::Constant::Int(val))
                    }
                    _ => Err(crate::Error::Error(
                        self.db.dcx().emit_err(
                            Level::ERROR.primary_title("Invalid literal type").element(
                                Snippet::source(self.source).annotation(
                                    AnnotationKind::Primary
                                        .span(lit.span.to_range(self.db))
                                        .label("Expected an integer literal here"),
                                ),
                            ),
                        ),
                    )),
                }
            }
            LitKind::Bool => {
                let value = text == "true";
                Ok(ir::Constant::Bool(value))
            }
            LitKind::Str => {
                let inner = if text.len() >= 2 {
                    &text[1..text.len() - 1]
                } else {
                    ""
                };
                let unescaped = unescape_string(inner);
                let sym = Symbol::new(self.db, unescaped);
                Ok(ir::Constant::String(sym))
            }
            LitKind::Float => {
                let clean = text.replace('_', "");
                let float_ty = match ty {
                    ir::Ty::Float(k) => *k,
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
                Ok(ir::Constant::Float(val))
            }
        }
    }

    /// Lower a literal to an operand (returns Operand::Constant directly).
    pub(crate) fn lower_literal(
        &mut self,
        lit: &Lit<'db>,
        expr_id: NodeId,
    ) -> MResult<ir::Operand<'db>> {
        let ty = self.infer_literal_type(lit, expr_id)?;
        let constant = self.build_constant(lit, &ty)?;
        Ok(ir::Operand::Constant(constant))
    }

    /// Lower a literal directly into a destination place.
    pub(crate) fn lower_literal_into(
        &mut self,
        lit: &Lit<'db>,
        expr_id: NodeId,
        dest: ir::Place<'db>,
    ) -> MResult<()> {
        let ty = self.infer_literal_type(lit, expr_id)?;
        let constant = self.build_constant(lit, &ty)?;

        self.emit_assign(dest, ir::Rvalue::Constant(constant));
        Ok(())
    }

    /// Infer the IR type from a literal.
    /// For bool and str, the type is unambiguous from the literal kind.
    /// For integer and float, consults the type table (for type-annotated contexts
    /// like `let x: u32 = 42`). Returns an error if the type cannot be determined.
    pub(crate) fn infer_literal_type(
        &self,
        lit: &Lit<'_>,
        expr_id: NodeId,
    ) -> MResult<ir::Ty<'db>> {
        // For bool and str, the type is unambiguous from the literal kind —
        // no need to consult the type table.
        match lit.kind {
            LitKind::Bool => return Ok(ir::Ty::Bool),
            LitKind::Str => return Ok(ir::Ty::Str),
            _ => {}
        }

        // For integer and float literals, check the type table —
        // the type checker should have resolved this literal's type.
        if let Some(resolved) = self.lookup_expr_type(expr_id) {
            return Ok(crate::ty_convert::resolved_to_ir(self.db, resolved));
        }

        Err(crate::Error::Error(
            self.db.dcx().emit_err(
                Level::ERROR
                    .primary_title("Could not determine literal type")
                    .element(Snippet::source(self.source).annotation(
                        AnnotationKind::Primary
                            .span(lit.span.to_range(self.db))
                            .label("Type not found in type table"),
                    )),
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[scrap_macros::salsa_test]
    fn test_lower_int_literal(db: &dyn scrap_shared::Db) {
        let expr = create_int_lit(db, 42);
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        let operand = result.unwrap();
        assert!(matches!(operand, ir::Operand::Constant(ir::Constant::Int(_))));

        // Literals no longer allocate temporaries
        assert_eq!(lowerer.local_decls.len(), 0);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_bool_literal(db: &dyn scrap_shared::Db) {
        let expr = create_bool_lit(db, true);
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        let operand = result.unwrap();
        assert!(matches!(operand, ir::Operand::Constant(ir::Constant::Bool(_))));
        assert_eq!(lowerer.local_decls.len(), 0);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_string_literal(db: &dyn scrap_shared::Db) {
        let expr = create_string_lit(db, "hello");
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        let operand = result.unwrap();
        assert!(matches!(operand, ir::Operand::Constant(ir::Constant::String(_))));
        assert_eq!(lowerer.local_decls.len(), 0);
    }
}
